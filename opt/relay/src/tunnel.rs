use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio_yamux::{Config as YamuxConfig, Session, StreamHandle};
use tracing::{info, warn};
use uuid::Uuid;

use crate::registry::TunnelHandle;
use crate::state::AppState;

/// Buffer size for the WS ↔ duplex bridge. 64 KiB matches yamux's default
/// window size (mirror of BRIDGE_BUFFER_BYTES in src/handlers/relay_client.rs).
const BRIDGE_BUFFER_BYTES: usize = 64 * 1024;

/// The first message an agent sends on a fresh yamux control stream after WS upgrade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConnect {
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Bearer token for `auth::authorize` (HMAC-signed POST to backend's
    /// /internal/relay/authorize). Added in Phase 68 gap-01 — the gateway
    /// previously never received the relay_token on the JSON path (it was
    /// only sent in the WS upgrade `Authorization: Bearer` header, which
    /// the gateway code does not actually validate).
    pub relay_token: Uuid,
    pub server_id: Uuid,
    pub subdomain: String,
    pub public_port: u16,
    pub agent_public_ip: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelHeartbeat {
    #[serde(rename = "type")]
    pub msg_type: String,
    /// `server_id` is optional: the agent's heartbeat JSON (relay_client.rs
    /// Part D) does NOT include `server_id` — the gateway already knows
    /// which server owns the control stream via `handle.server_id` (which
    /// is what `handle_tunnel_message` uses for the backend audit log).
    /// Without `#[serde(default)]`, the first heartbeat (within 10s of
    /// connect) would fail to deserialize and `last_heartbeat` would never
    /// update, causing the heartbeat watcher at heartbeat.rs:36 to mark
    /// every tunnel stale after 90s.
    #[serde(default)]
    pub server_id: Uuid,
    pub tunnel_uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelMessage {
    TunnelConnect(TunnelConnect),
    TunnelHeartbeat(TunnelHeartbeat),
    TunnelDisconnect { server_id: Uuid, reason: String },
}

pub async fn run_tunnel_session(socket: WebSocket, state: Arc<AppState>) {
    // Phase 69 (multi-server): The gateway accepts N concurrent WS connections
    // from the same agent IP, each authenticating with the same relay_token
    // but a different server_id. There is NO agent-IP uniqueness enforcement
    // in this function — the registry enforces one tunnel per server_id, and
    // the authorize() call above validates that the token authorizes the
    // requested server_id. Per-IP rate limiting for player connections is
    // handled separately in player.rs (DoS protection, not tunnel uniqueness).
    //
    // 1. Set up the WS <-> duplex bridge. The bridge task pumps bytes
    //    between WS Binary messages and the duplex's yamux side. yamux
    //    handles all framing; the gateway code below only reads from the
    //    yamux side of the duplex.
    let (yamux_side, ws_byte_side) = tokio::io::duplex(BRIDGE_BUFFER_BYTES);
    let mut bridge_handle = tokio::spawn(ws_bridge(socket, ws_byte_side));

    // 2. Create the yamux server session over the duplex side.
    let yamux_cfg = YamuxConfig::default();
    let mut session = Session::new_server(yamux_side, yamux_cfg);

    // 3. Wait for the first inbound yamux stream (the agent's control
    //    stream — the one on which it sends TunnelConnect, then
    //    TunnelHeartbeat every 10s).
    let mut control_stream: StreamHandle = match session.next().await {
        Some(Ok(s)) => s,
        Some(Err(e)) => {
            warn!("[TUNNEL] yamux session error: {}", e);
            return;
        }
        None => {
            warn!("[TUNNEL] yamux session ended before first stream");
            return;
        }
    };

    // 4. Read the TunnelConnect JSON from the control stream. NDJSON
    //    framing: agent appends '\n' to the connect message.
    let connect: TunnelConnect = match read_json_message(&mut control_stream).await {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(c) => c,
            Err(e) => {
                warn!("[TUNNEL] Invalid TunnelConnect JSON: {}", e);
                return;
            }
        },
        Err(e) => {
            warn!("[TUNNEL] Failed to read TunnelConnect: {}", e);
            return;
        }
    };

    if connect.msg_type != "tunnel_connect" {
        warn!(
            "[TUNNEL] First message was not tunnel_connect: {}",
            connect.msg_type
        );
        return;
    }

    info!(
        "[TUNNEL] TunnelConnect: server_id={}, subdomain={}, agent_ip={}",
        connect.server_id, connect.subdomain, connect.agent_public_ip
    );

    // 5. Validate subdomain.
    if let Err(e) = validate_subdomain(&connect.subdomain) {
        warn!("[TUNNEL] Invalid subdomain '{}': {}", connect.subdomain, e);
        return;
    }

    // 6. Authorize the relay_token against the backend (Phase 69: 1:N mapping).
    //    The backend's /internal/relay/authorize endpoint returns all servers
    //    authorized by this token. We verify that the requested server_id is
    //    in the authorized list — allowing multiple tunnels from the same
    //    agent IP, each with a different server_id (multi-server).
    match crate::auth::authorize(&state, &connect.relay_token).await {
        Ok(mappings) => {
            if !mappings.iter().any(|m| m.server_id == connect.server_id) {
                warn!(
                    "[TUNNEL] server_id={} not authorized by token={}; closing WS",
                    connect.server_id, connect.relay_token
                );
                return;
            }
        }
        Err(e) => {
            warn!(
                "[TUNNEL] auth::authorize failed for token={}: {}; closing WS",
                connect.relay_token, e
            );
            return;
        }
    }

    // 7. Build the TunnelHandle with the real `Control` handle. This is the
    //    BLOCKER #1 fix — the previous code stored an empty Optional in
    //    `yamux_control` and `player.rs:88`'s `control.open_stream()`
    //    would always hit the empty arm and drop the connection.
    let control = session.control(); // tokio_yamux::Control: Clone + Send + Sync
    let handle = Arc::new(TunnelHandle {
        server_id: connect.server_id,
        subdomain: connect.subdomain.clone(),
        agent_public_ip: connect.agent_public_ip.clone(),
        last_heartbeat: std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
        yamux_control: tokio::sync::Mutex::new(Some(control)),
        started_at: std::time::Instant::now(),
        bytes_in: std::sync::atomic::AtomicU64::new(0),
        bytes_out: std::sync::atomic::AtomicU64::new(0),
    });

    if let Err(e) = state.registry.register(handle.clone()) {
        warn!("[TUNNEL] Registry::register failed: {}", e);
        return;
    }

    // 8. Notify backend of the new tunnel.
    if let Err(e) = state
        .backend
        .report_tunnel_event(handle.server_id, "connected", "tunnel_established")
        .await
    {
        warn!("[TUNNEL] Failed to report connected event: {}", e);
    }
    crate::metrics::ACTIVE_TUNNELS.inc();
    crate::metrics::TUNNEL_EVENTS_TOTAL
        .with_label_values(&["connected"])
        .inc();

    // 9. Spawn the control-stream reader task that owns the StreamHandle.
    //    The reader parses NDJSON messages from the agent (TunnelHeartbeat
    //    every 10s, TunnelDisconnect on graceful shutdown) and dispatches
    //    them to `handle_tunnel_message`. The reader does not influence
    //    the backend `heartbeat` metric — that one is incremented in
    //    `handle_tunnel_message` (and is what the `report_tunnel_event`
    //    audit log uses).
    let hb_state = state.clone();
    let hb_handle = handle.clone();
    let hb_task = tokio::spawn(async move {
        read_control_stream(hb_state, hb_handle, control_stream).await;
    });

    // 10. 10s ticker for backend liveness reports + bridge-end detection.
    let mut ticker = tokio::time::interval(Duration::from_secs(
        state.config.tunnel.heartbeat_interval_secs,
    ));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // Server-initiated ping is not required; the heartbeat is
                // a liveness check based on the agent's last message
                // timestamp, which the read_control_stream task updates.
                if let Err(e) = state.backend.report_tunnel_event(handle.server_id, "heartbeat", "ok").await {
                    warn!("[TUNNEL] Heartbeat backend report failed: {}", e);
                }
            }
            _ = &mut bridge_handle => {
                // The WS bridge returned (agent disconnected or yamux
                // session died — yamux_side hits EOF when the agent's
                // WS closes, which the bridge sees).
                info!("[TUNNEL] WS bridge ended: server_id={}", handle.server_id);
                break;
            }
        }
    }

    // 11. Cleanup: abort the reader + bridge, unregister, report disconnected.
    hb_task.abort();
    let _ = hb_task.await;
    bridge_handle.abort();
    let _ = bridge_handle.await;
    state.registry.unregister(&handle.server_id);
    crate::metrics::ACTIVE_TUNNELS.dec();
    crate::metrics::TUNNEL_EVENTS_TOTAL
        .with_label_values(&["disconnected"])
        .inc();
    if let Err(e) = state
        .backend
        .report_tunnel_event(handle.server_id, "disconnected", "ws_closed")
        .await
    {
        warn!("[TUNNEL] Failed to report disconnected event: {}", e);
    }
    crate::session_log::log_session_end(handle.server_id, 0, 0);
}

async fn handle_tunnel_message(state: &AppState, handle: &Arc<TunnelHandle>, msg: TunnelMessage) {
    match msg {
        TunnelMessage::TunnelConnect(c) => {
            warn!("[TUNNEL] Ignoring mid-session TunnelConnect: server_id={}", c.server_id);
        }
        TunnelMessage::TunnelHeartbeat(h) => {
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            handle
                .last_heartbeat
                .store(now_secs, std::sync::atomic::Ordering::Relaxed);
            crate::metrics::TUNNEL_EVENTS_TOTAL
                .with_label_values(&["heartbeat"])
                .inc();
            // Use `handle.server_id` (not `h.server_id`) — the heartbeat
            // JSON's `server_id` is `#[serde(default)]` (Uuid::nil() if
            // absent), so we must use the value the gateway already
            // validated at TunnelConnect time. Otherwise the backend
            // audit log would record `Uuid::nil()` for every heartbeat.
            let _ = state
                .backend
                .report_tunnel_event_with_uptime(handle.server_id, "heartbeat", "ok", h.tunnel_uptime_secs)
                .await;
        }
        TunnelMessage::TunnelDisconnect { server_id, reason } => {
            info!("[TUNNEL] Agent-initiated disconnect: server_id={}, reason={}", server_id, reason);
            state.registry.unregister(&server_id);
            crate::metrics::ACTIVE_TUNNELS.dec();
            crate::metrics::TUNNEL_EVENTS_TOTAL
                .with_label_values(&["disconnected"])
                .inc();
            let _ = state
                .backend
                .report_tunnel_event(server_id, "disconnected", &reason)
                .await;
        }
    }
}

fn validate_subdomain(s: &str) -> Result<(), String> {
    if s.is_empty() || s.len() > 63 {
        return Err("subdomain must be 1-63 chars".into());
    }
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err("subdomain has invalid characters".into());
    }
    if s.starts_with('-') || s.ends_with('-') {
        return Err("subdomain cannot start or end with '-'".into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// WS ↔ duplex bridge (mirror of agent's ws_bridge in src/handlers/relay_client.rs)
// ---------------------------------------------------------------------------

/// Pump bytes between the axum WebSocket and the yamux duplex. WS Binary
/// frames carry the yamux session bytes; Text / Ping / Pong are skipped.
/// The gateway only writes `Message::Binary` (yamux bytes) and accepts
/// `Message::Binary` from the agent (the agent's `ws_bridge` only sends
/// Binary — see relay_client.rs:565, 579). Exits when either side hits EOF.
async fn ws_bridge(socket: WebSocket, mut yamux_side: DuplexStream) {
    let (mut ws_sink, mut ws_stream) = socket.split();
    let mut buf = vec![0u8; BRIDGE_BUFFER_BYTES];

    loop {
        tokio::select! {
            biased;
            n = yamux_side.read(&mut buf) => {
                match n {
                    Ok(0) => {
                        info!("[TUNNEL] bridge — yamux side closed");
                        break;
                    }
                    Ok(n) => {
                        let msg = Message::Binary(buf[..n].to_vec().into());
                        if let Err(e) = ws_sink.send(msg).await {
                            warn!(error = %e, "[TUNNEL] bridge — WS send failed");
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "[TUNNEL] bridge — yamux read failed");
                        break;
                    }
                }
            }
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Err(e) = yamux_side.write_all(&data).await {
                            warn!(error = %e, "[TUNNEL] bridge — yamux write failed");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("[TUNNEL] bridge — WS closed");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Text / Ping / Pong — skip.
                        continue;
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "[TUNNEL] bridge — WS recv failed");
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Control stream reader (NDJSON demuxer + dispatcher)
// ---------------------------------------------------------------------------

/// Read NDJSON messages from the agent's control stream forever (or until
/// the stream ends). Each message is dispatched to `handle_tunnel_message`.
/// The control stream is long-lived: it carries the initial `TunnelConnect`
/// then repeated `TunnelHeartbeat` messages every 10s, plus an
/// occasional `TunnelDisconnect` on graceful agent shutdown.
async fn read_control_stream(
    state: Arc<AppState>,
    handle: Arc<TunnelHandle>,
    mut stream: StreamHandle,
) {
    loop {
        match read_json_message(&mut stream).await {
            Ok(bytes) => {
                if bytes.is_empty() {
                    // Clean EOF — agent closed the control stream.
                    info!(
                        "[TUNNEL] Control stream closed by agent: server_id={}",
                        handle.server_id
                    );
                    break;
                }
                match serde_json::from_slice::<TunnelMessage>(&bytes) {
                    Ok(parsed) => {
                        handle_tunnel_message(&state, &handle, parsed).await;
                    }
                    Err(e) => {
                        warn!(
                            "[TUNNEL] Failed to parse control message: {} ({} bytes)",
                            e,
                            bytes.len()
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    "[TUNNEL] Control stream read error: server_id={}: {}",
                    handle.server_id, e
                );
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// NDJSON framing helper
// ---------------------------------------------------------------------------

/// Read a single JSON document from a yamux stream using newline-delimited
/// JSON (NDJSON) framing. The control stream is long-lived (one stream per
/// tunnel, used for the initial `TunnelConnect` then repeated
/// `TunnelHeartbeat` messages every 10s), so raw `write_all`/`read` calls
/// cannot reliably demultiplex the messages. The protocol is:
///
/// 1. The agent writes `serde_json::to_vec(&msg) + b"\n"` (Tasks 1 Part C,
///    Part D, and Part E in src/handlers/relay_client.rs add the trailing
///    newline).
/// 2. The gateway reads from the yamux stream into a 64 KiB buffer, growing
///    if needed, until it finds `\n` (0x0A) or EOF.
/// 3. Returns the bytes BEFORE the newline (i.e., the JSON document), with
///    the newline stripped.
/// 4. If the buffer grows past 64 KiB without finding `\n`, treat that as a
///    protocol error (`Err(InvalidData)`).
/// 5. On clean EOF without any data, returns `Ok(empty)` (caller treats as
///    disconnect).
///
/// Note on implementation: the `bytes::BytesMut` API exposes a method
/// that splits off a prefix and returns it as a new `BytesMut`. That
/// method is NOT available on `std::vec::Vec<u8>`. We use slice + `drain`
/// to extract the message and discard the consumed bytes (including
/// the newline).
async fn read_json_message(stream: &mut StreamHandle) -> Result<Vec<u8>, std::io::Error> {
    let mut buf: Vec<u8> = Vec::with_capacity(BRIDGE_BUFFER_BYTES);
    let mut tmp = [0u8; BRIDGE_BUFFER_BYTES];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            if buf.is_empty() {
                return Ok(Vec::new()); // clean EOF
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "NDJSON: stream closed mid-message",
            ));
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            // Drain the consumed bytes (including the newline) from the
            // buffer. We use `Vec::drain(..=pos)` rather than the
            // `BytesMut`-style split helper because the latter is not
            // available on `std::vec::Vec<u8>`.
            let out = buf[..pos].to_vec();
            buf.drain(..=pos);
            return Ok(out);
        }
        if buf.len() > BRIDGE_BUFFER_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "NDJSON message exceeds 64 KiB",
            ));
        }
    }
}
