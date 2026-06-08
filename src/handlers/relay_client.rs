//! Per-server outbound Esluce Relay tunnel client.
//!
//! Phase 69 (Plan 02). Each active server on this agent node gets its own
//! independent WSS connection to the Esluce Relay gateway, its own yamux
//! session, and its own reconnect loop.  The per-server state is managed
//! in an `RwLock<HashMap<ServerId, PerServerRuntime>>` (D-01).
//!
//! Architecture
//! ------------
//!
//! ```text
//!  RelayRuntime {
//!      shutdown: CancellationToken   // parent — cascades to all children
//!      tunnels: Arc<RwLock<HashMap<Uuid, PerServerRuntime>>>
//!  }
//!
//!  PerServerRuntime {                // × N (one per server)
//!      cancel: child_token(),
//!      join: JoinHandle,
//!      control_tx: mpsc::Sender,
//!      bytes_transferred: AtomicU64,
//!      tunnel_start: Instant,
//!      config: PerServerRelayConfig,
//!  }
//! ```
//!
//! Each per-server tunnel:
//!   ──[WSS]──> Gateway
//!        └── yamux session (client)
//!             ├── control stream (TunnelConnect, heartbeats)
//!             └── inbound player streams → local MC (`local_mc_addr`)
//!
//! Lifecycle
//! ---------
//!
//! 1. [`connect(server_id, config)`] is invoked by the `relay.connect` task
//!    arm. It creates a new `PerServerRuntime`, spawns a per-server
//!    [`run_relay_client`] loop. If a tunnel already exists for the same
//!    server_id, it is cancelled and replaced atomically (D-06).
//! 2. [`run_relay_client`] runs the reconnect loop with exponential
//!    backoff (1s → 30s cap, ±20% jitter). On every successful
//!    handshake it spawns a heartbeat task and watches for new inbound
//!    yamux streams.
//! 3. The heartbeat task watches two thresholds (D-25 / RESOLVED Q4):
//!    24 h uptime OR 100 GB transferred. On either threshold it sends a
//!    clean shutdown of the control stream so the outer reconnect loop
//!    establishes a fresh handshake with a new TLS session.
//! 4. On any disconnect (graceful or errored), the disconnect branch
//!    enqueues a `relay.remove_cname_record` task via the local
//!    self-loop (D-13 / RESOLVED Q7) so the stale DNS record is removed.
//! 5. [`disconnect(server_id)`] cancels the child token, removes from
//!    HashMap, and returns.
//! 6. [`send_heartbeat(server_id)`] sends a `TunnelHeartbeat` JSON frame
//!    on the specified server's control stream.
//! 7. [`shutdown_all()`] cancels the parent token → all child tokens fire
//!    → all reconnect loops exit → HashMap is cleared (D-04).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use agent_proto::Task;
use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite, DuplexStream};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tokio_tungstenite::tungstenite::{
    self,
    client::IntoClientRequest,
    http::Uri,
    Message,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use rand::Rng;

use tokio_yamux::{Config as YamuxConfig, Session, StreamHandle};

use crate::audit;
use crate::state::{self, PerServerRelayConfig, RelayConfig};

use super::relay_session;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Heartbeat cadence for the open tunnel — T-68-03 / T-68-04 (WS liveness).
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

/// D-25 / RESOLVED Q4: re-handshake the WS after this much wall-clock
/// tunnel uptime.
const REKEY_UPTIME: Duration = Duration::from_secs(24 * 60 * 60);

/// D-25 / RESOLVED Q4: re-handshake the WS after this many bytes have
/// crossed the yamux sessions in aggregate. 100 GiB.
const REKEY_BYTES: u64 = 100 * 1024 * 1024 * 1024;

/// Backoff parameters for the reconnect loop (T-68-06).
const BACKOFF_INITIAL_MS: u64 = 1_000;
const BACKOFF_MAX_MS: u64 = 30_000;
const BACKOFF_JITTER_PCT: u64 = 20;

/// Buffer size for the WS ↔ duplex bridge task. 64 KiB matches the
/// yamux default window size.
const BRIDGE_BUFFER_BYTES: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Per-server runtime state (Phase 69 — D-01, D-03, D-04)
// ---------------------------------------------------------------------------

/// Per-server tunnel state. Each active server on this node gets one.
pub struct PerServerRuntime {
    /// Child of the parent RelayRuntime::shutdown token (D-04).
    pub cancel: CancellationToken,
    /// Join handle for the per-server run_relay_client task.
    pub join: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Channel for sending heartbeat/disconnect JSON to this server's
    /// control stream. None while no tunnel is open.
    pub control_tx: Mutex<Option<tokio::sync::mpsc::UnboundedSender<serde_json::Value>>>,
    /// Per-server byte counter, updated by relay_session on each stream close (D-18).
    pub bytes_transferred: Arc<AtomicU64>,
    /// Tunnel start time, used by the heartbeat task for uptime and rekey.
    pub tunnel_start: Mutex<Option<Instant>>,
    /// Per-server config from task payload (D-15).
    pub config: PerServerRelayConfig,
}

/// Parent runtime — holds the shutdown token that cascades to all children.
pub struct RelayRuntime {
    /// Parent token. parent.cancel() fires all child tokens (D-04).
    pub shutdown: CancellationToken,
    /// Per-server tunnel map. Concurrent reads, exclusive writes (D-01).
    pub tunnels: Arc<RwLock<HashMap<Uuid, PerServerRuntime>>>,
}

static RELAY_RUNTIME: OnceLock<RelayRuntime> = OnceLock::new();

fn runtime() -> &'static RelayRuntime {
    RELAY_RUNTIME.get_or_init(|| RelayRuntime {
        shutdown: CancellationToken::new(),
        tunnels: Arc::new(RwLock::new(HashMap::new())),
    })
}

// ---------------------------------------------------------------------------
// Public task entrypoints
// ---------------------------------------------------------------------------

/// Cancel ALL tunnels. Called from agent shutdown sequence.
/// parent.cancel() cascades to all child tokens (D-04).
pub async fn shutdown_all() {
    info!("RelayRuntime: shutdown_all — cancelling parent token");
    runtime().shutdown.cancel();
    // Brief yield so tasks see cancellation
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut tunnels = runtime().tunnels.write().await;
    tunnels.clear();
    info!("RelayRuntime: all tunnels cleared");
}

/// `relay.connect` task: start a per-server tunnel. Replaces existing
/// tunnel for the same server_id atomically (D-06).
pub async fn connect(server_id: Uuid, per_server_cfg: PerServerRelayConfig) -> Result<serde_json::Value> {
    let rt = runtime();
    let mut tunnels = rt.tunnels.write().await;

    // D-06: Replace existing tunnel if one exists
    if let Some(existing) = tunnels.remove(&server_id) {
        info!("Replacing existing tunnel for server_id={}", server_id);
        existing.cancel.cancel();  // child token fires → reconnect loop exits
        // PerServerRuntime drops, JoinHandle detaches
    }

    let child_cancel = rt.shutdown.child_token();
    let config_clone = per_server_cfg.clone();
    let parent_shutdown = rt.shutdown.clone();
    let handle = tokio::spawn(async move {
        run_relay_client(config_clone, parent_shutdown).await;
    });

    let psr = PerServerRuntime {
        cancel: child_cancel,
        join: Mutex::new(Some(handle)),
        control_tx: Mutex::new(None),
        bytes_transferred: Arc::new(AtomicU64::new(0)),
        tunnel_start: Mutex::new(None),
        config: per_server_cfg,
    };

    tunnels.insert(server_id, psr);

    info!(%server_id, "PerServer tunnel: connect started");
    Ok(json!({
        "action": "connect",
        "status": "started",
        "server_id": server_id,
    }))
}

/// `relay.disconnect` task: cancel a specific server's tunnel.
pub async fn disconnect(server_id: Uuid) -> Result<serde_json::Value> {
    let mut tunnels = runtime().tunnels.write().await;
    if let Some(psr) = tunnels.remove(&server_id) {
        info!("Disconnecting tunnel for server_id={}", server_id);
        psr.cancel.cancel();  // child token fires → reconnect loop exits
        // Dispatch remove_cname_record cleanup
        let shared_cfg = state::relay_config();
        if let Some(cfg) = shared_cfg {
            dispatch_remove_cname_record(&cfg, &psr.config).await;
        }
        Ok(json!({
            "action": "disconnect",
            "status": "stopped",
            "server_id": server_id,
        }))
    } else {
        info!("disconnect: no active tunnel for server_id={} (already gone)", server_id);
        Ok(json!({
            "action": "disconnect",
            "status": "already_disconnected",
            "server_id": server_id,
        }))
    }
}

/// `relay.heartbeat` task: send an immediate TunnelHeartbeat on the
/// per-server control stream. Returns silently if no tunnel open.
pub async fn send_heartbeat(server_id: Uuid) -> Result<serde_json::Value> {
    let tunnels = runtime().tunnels.read().await;
    if let Some(psr) = tunnels.get(&server_id) {
        let uptime = psr.tunnel_start.lock().await
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);
        let payload = json!({
            "type": "tunnel_heartbeat",
            "server_id": server_id,
            "tunnel_uptime_secs": uptime,
        });
        let tx_guard = psr.control_tx.lock().await;
        if let Some(tx) = tx_guard.as_ref() {
            let _ = tx.send(payload);
            Ok(json!({
                "action": "heartbeat",
                "status": "sent",
                "server_id": server_id,
                "tunnel_uptime_secs": uptime,
            }))
        } else {
            Ok(json!({
                "action": "heartbeat",
                "status": "no_tunnel_open",
                "server_id": server_id,
            }))
        }
    } else {
        Ok(json!({
            "action": "heartbeat",
            "status": "no_tunnel_for_server",
            "server_id": server_id,
        }))
    }
}

// ---------------------------------------------------------------------------
// Per-server reconnect loop (Phase 69 — D-04)
// ---------------------------------------------------------------------------

/// Per-server reconnect loop. Exits when child_shutdown is cancelled
/// (via parent.cancel() or disconnect()). Heartbeat staggering (0-10s
/// jitter) added by Plan 69-04.
pub async fn run_relay_client(
    per_server_cfg: PerServerRelayConfig,
    parent_shutdown: CancellationToken,
) {
    let shared_cfg = match state::relay_config() {
        Some(c) => c,
        None => {
            error!("PerServer[{}]: no shared relay config", per_server_cfg.server_id);
            return;
        }
    };

    // D-04: child token — parent.cancel() cascades to this loop
    let child_shutdown = parent_shutdown.child_token();
    let mut backoff_ms: u64 = BACKOFF_INITIAL_MS;

    info!(
        server_id = %per_server_cfg.server_id,
        subdomain = %per_server_cfg.subdomain,
        "PerServer tunnel: entering reconnect loop"
    );

    loop {
        if child_shutdown.is_cancelled() {
            info!("PerServer[{}]: shutdown requested, exiting", per_server_cfg.server_id);
            break;
        }

        match connect_and_run(&shared_cfg, &per_server_cfg, &child_shutdown).await {
            Ok(()) => {
                backoff_ms = BACKOFF_INITIAL_MS;
            }
            Err(e) => {
                warn!(server_id = %per_server_cfg.server_id, error = %e, "connect_and_run failed");
            }
        }

        if child_shutdown.is_cancelled() { break; }

        let sleep_ms = backoff_with_jitter(backoff_ms);
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        backoff_ms = (backoff_ms.saturating_mul(2)).min(BACKOFF_MAX_MS);
    }
}

/// Per-server tunnel lifecycle: WS handshake → TunnelConnect → heartbeat
/// loop + inbound stream drive → cleanup → remove_cname_record.
async fn connect_and_run(
    shared_cfg: &RelayConfig,
    per_server_cfg: &PerServerRelayConfig,
    shutdown: &CancellationToken,
) -> Result<()> {
    // 1. Parse the gateway URL.
    let uri: Uri = shared_cfg.gateway_url.parse()
        .with_context(|| format!("invalid gateway_url: {}", shared_cfg.gateway_url))?;

    // 2. Build the WS handshake request with the Bearer token.
    let req = build_ws_request(uri.clone(), &shared_cfg.token)?;

    // 3. Upgrade to WSS.
    let (ws_stream, _response) = tokio_tungstenite::connect_async_tls_with_config(
        req, None, true, None,
    )
    .await
    .with_context(|| "RelayClient: WS handshake failed")?;
    info!(server_id = %per_server_cfg.server_id, "RelayClient: WS handshake complete");

    // 4. Build a duplex bridge between the WS and the yamux session.
    let (yamux_side, ws_byte_side) = tokio::io::duplex(BRIDGE_BUFFER_BYTES);
    let bridge_handle = tokio::spawn(ws_bridge(ws_stream, ws_byte_side));

    // 5. Open the yamux client session over the duplex side.
    let yamux_cfg = YamuxConfig::default();
    let mut session = Session::new_client(yamux_side, yamux_cfg);

    // 6. Open the control stream and send TunnelConnect.
    let mut control: StreamHandle = session
        .open_stream()
        .map_err(|e| anyhow!("yamux open_stream failed: {}", e))?;
    let connect_msg = json!({
        "type": "tunnel_connect",
        "relay_token": shared_cfg.token,
        "server_id": per_server_cfg.server_id,
        "subdomain": per_server_cfg.subdomain,
        "public_port": per_server_cfg.public_port,
        "agent_public_ip": shared_cfg.agent_public_ip,
        "region": shared_cfg.region,
    });
    let mut connect_bytes = serde_json::to_vec(&connect_msg)?;
    connect_bytes.push(b'\n');
    control
        .write_all(&connect_bytes)
        .await
        .map_err(|e| anyhow!("TunnelConnect write failed: {}", e))?;
    info!(
        server_id = %per_server_cfg.server_id,
        subdomain = %per_server_cfg.subdomain,
        "RelayClient: TunnelConnect sent"
    );

    audit::log_relay_tunnel_event(
        per_server_cfg.server_id,
        per_server_cfg.server_id,
        "connected",
        &format!("subdomain={}", per_server_cfg.subdomain),
    )
    .await;

    // 7. Reset byte counter + start time; install the control_tx
    // channel on the per-server runtime.
    let (ctrl_tx, ctrl_rx) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
    let bytes_counter;
    let tunnel_start;
    {
        let mut tunnels = runtime().tunnels.write().await;
        if let Some(psr) = tunnels.get_mut(&per_server_cfg.server_id) {
            psr.bytes_transferred.store(0, Ordering::Relaxed);
            *psr.tunnel_start.lock().await = Some(Instant::now());
            *psr.control_tx.lock().await = Some(ctrl_tx);
            bytes_counter = psr.bytes_transferred.clone();
            tunnel_start = Some(Instant::now());
        } else {
            warn!(
                server_id = %per_server_cfg.server_id,
                "PerServerRuntime vanished before tunnel connect completed"
            );
            bridge_handle.abort();
            let _ = bridge_handle.await;
            return Ok(());
        }
    }

    // Phase 69-04: Stagger first heartbeat 0-10s to prevent thundering
    // herd when many per-server loops start simultaneously (e.g. after
    // agent WS reconnect).  The jitter is a one-time offset before the
    // heartbeat task starts its 10 s ticker loop.
    let jitter_ms: u64 = rand::thread_rng().gen_range(0..=10_000);
    tokio::time::sleep(Duration::from_millis(jitter_ms)).await;
    debug!(
        server_id = %per_server_cfg.server_id,
        jitter_ms,
        "heartbeat staggered"
    );

    // 8. Spawn the heartbeat task (T-68-03 / T-68-04 + D-25 rekey).
    let heartbeat_cancel = shutdown.clone();
    let heartbeat_handle = tokio::spawn(async move {
        run_heartbeat_task(control, ctrl_rx, bytes_counter, tunnel_start, heartbeat_cancel).await;
    });

    // 9. Drive incoming streams until the session ends.
    let session_result = drive_inbound_streams(
        &mut session,
        &per_server_cfg.local_mc_addr,
        per_server_cfg.server_id,
        shutdown,
    ).await;

    // 10. Cleanup: clear per-server runtime state, abort bridge.
    {
        let mut tunnels = runtime().tunnels.write().await;
        if let Some(psr) = tunnels.get_mut(&per_server_cfg.server_id) {
            *psr.tunnel_start.lock().await = None;
            *psr.control_tx.lock().await = None;
        }
    }
    heartbeat_handle.abort();
    let _ = heartbeat_handle.await;
    bridge_handle.abort();
    let _ = bridge_handle.await;

    // 11. Dispatch remove_cname_record cleanup.
    dispatch_remove_cname_record(shared_cfg, per_server_cfg).await;

    session_result
}

/// Drive the yamux session's incoming stream queue. Each new inbound
/// stream is handed off to [`super::relay_session::run_relay_session`]
/// with the per-server bytes counter looked up from the HashMap.
async fn drive_inbound_streams(
    session: &mut Session<DuplexStream>,
    local_mc_addr: &str,
    server_id: Uuid,
    shutdown: &CancellationToken,
) -> Result<()> {
    loop {
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => {
                info!("RelayClient: shutdown during stream drive, returning");
                return Ok(());
            }
            next = session.next() => {
                match next {
                    Some(Ok(stream)) => {
                        let local = local_mc_addr.to_string();
                        // Look up the per-server bytes counter (D-18)
                        let bytes_counter = {
                            let tunnels = runtime().tunnels.read().await;
                            tunnels.get(&server_id)
                                .map(|psr| psr.bytes_transferred.clone())
                        };
                        if let Some(counter) = bytes_counter {
                            let local_for_task = local.clone();
                            tokio::spawn(async move {
                                if let Err(e) = relay_session::run_relay_session(
                                    stream, local_for_task.clone(), counter,
                                ).await {
                                    warn!(error = %e, local = %local_for_task, "Relay session error");
                                }
                            });
                        }
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "RelayClient: yamux session stream error");
                        return Err(anyhow!("yamux stream error: {}", e));
                    }
                    None => {
                        info!("RelayClient: yamux session ended (no more streams)");
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// 10 s heartbeat ticker that writes a `TunnelHeartbeat` frame onto
/// the control stream and watches the 24 h / 100 GB rekey thresholds.
/// On threshold hit, closes the control stream (which drops the
/// whole yamux session, which drops the duplex side, which makes the
/// bridge see EOF and send a WS Close).
async fn run_heartbeat_task(
    mut control: StreamHandle,
    mut ctrl_rx: tokio::sync::mpsc::UnboundedReceiver<serde_json::Value>,
    bytes_counter: Arc<AtomicU64>,
    tunnel_start: Option<Instant>,
    shutdown: CancellationToken,
) {
    let mut ticker = interval(HEARTBEAT_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => {
                info!("RelayClient: heartbeat task shutdown");
                return;
            }
            _ = ticker.tick() => {
                let uptime = tunnel_start.map(|t| t.elapsed().as_secs()).unwrap_or(0);
                let bytes = bytes_counter.load(Ordering::Relaxed);

                // D-25 rekey thresholds.
                if tunnel_start.map(|t| t.elapsed() >= REKEY_UPTIME).unwrap_or(false)
                    || bytes >= REKEY_BYTES
                {
                    info!(
                        uptime_secs = uptime,
                        bytes_transferred = bytes,
                        "RelayClient: D-25 rekey threshold hit; closing tunnel for fresh handshake"
                    );
                    audit::log_relay_tunnel_event(
                        Uuid::nil(),
                        Uuid::nil(),
                        "rekey",
                        &format!("uptime_secs={} bytes={}", uptime, bytes),
                    ).await;
                    // Closing the control stream is enough to drop the
                    // whole yamux session, which drops the duplex side,
                    // which makes the bridge see EOF and send a WS Close.
                    let _ = shutdown_control(&mut control).await;
                    return;
                }

                // Normal heartbeat.
                let msg = json!({
                    "type": "tunnel_heartbeat",
                    "tunnel_uptime_secs": uptime,
                });
                let mut bytes = serde_json::to_vec(&msg).unwrap_or_default();
                // NDJSON framing: see connect_and_run for rationale. The
                // gateway's read_control_stream reads bytes until '\n'.
                bytes.push(b'\n');
                if control.write_all(&bytes).await.is_err() {
                    warn!("RelayClient: heartbeat write failed, exiting heartbeat loop");
                    return;
                }
                audit::log_relay_tunnel_event(
                    Uuid::nil(),
                    Uuid::nil(),
                    "heartbeat",
                    &format!("uptime_secs={}", uptime),
                ).await;
            }
            // On-demand backend-initiated commands (immediate heartbeat,
            // disconnect signal) get forwarded onto the control stream
            // verbatim. The trailing '\n' is required for the gateway's
            // NDJSON read_json_message demuxer; without it, a payload
            // would be concatenated with the next regular heartbeat
            // and fail JSON deserialization with "trailing characters".
            Some(payload) = ctrl_rx.recv() => {
                let mut bytes = serde_json::to_vec(&payload).unwrap_or_default();
                bytes.push(b'\n');
                if control.write_all(&bytes).await.is_err() {
                    warn!("RelayClient: on-demand control message write failed");
                    return;
                }
            }
        }
    }
}

/// Close the control stream half cleanly. Uses [`AsyncWrite::shutdown`]
/// which sends a yamux FIN; the parent session's stream loop sees the
/// FIN, the duplex side drops, the bridge task sees EOF.
async fn shutdown_control<S>(control: &mut S) -> std::io::Result<()>
where
    S: AsyncWrite + Unpin,
{
    control.shutdown().await
}

// ---------------------------------------------------------------------------
// WS ↔ duplex bridge
// ---------------------------------------------------------------------------

/// Pump bytes from the yamux duplex side into WS Binary messages and
/// vice versa. Exits when either side hits EOF.
async fn ws_bridge<S>(
    ws: tokio_tungstenite::WebSocketStream<S>,
    mut yamux_side: DuplexStream,
)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (mut ws_sink, mut ws_stream) = ws.split();
    let mut buf = vec![0u8; BRIDGE_BUFFER_BYTES];

    loop {
        tokio::select! {
            biased;
            n = yamux_side.read(&mut buf) => {
                match n {
                    Ok(0) => {
                        info!("RelayClient: bridge — yamux side closed");
                        break;
                    }
                    Ok(n) => {
                        let msg = Message::Binary(buf[..n].to_vec().into());
                        if let Err(e) = ws_sink.send(msg).await {
                            warn!(error = %e, "RelayClient: bridge — WS send failed");
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "RelayClient: bridge — yamux read failed");
                        break;
                    }
                }
            }
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Err(e) = yamux_side.write_all(&data).await {
                            warn!(error = %e, "RelayClient: bridge — yamux write failed");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("RelayClient: bridge — WS closed");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Text / Ping / Pong — skip.
                        continue;
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "RelayClient: bridge — WS recv failed");
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build the WS upgrade request with the Bearer authorization header.
fn build_ws_request(uri: Uri, token: &str) -> Result<tungstenite::handshake::client::Request> {
    let auth_value = format!("Bearer {}", token);
    let builder = tungstenite::ClientRequestBuilder::new(uri)
        .with_header("Authorization", auth_value);
    builder
        .into_client_request()
        .map_err(|e| anyhow!("failed to build WS request: {}", e))
}

/// Apply ±20% jitter to a backoff value (T-68-06).
fn backoff_with_jitter(backoff_ms: u64) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let jitter = (nanos % (BACKOFF_JITTER_PCT * 2 + 1)).saturating_sub(BACKOFF_JITTER_PCT);
    let delta = backoff_ms.saturating_mul(jitter) / 100;
    backoff_ms.saturating_add(delta)
}

/// D-13 / RESOLVED Q7: dispatch the remove_cname_record cleanup. We
/// call `dns::handle_remove_record` directly with a constructed
/// `Task`; the dispatch arm in `mod.rs` handles the case where the
/// backend itself sends this task type.
///
/// Uses `per_server_cfg.dns_record_id` when available; falls back to
/// subdomain-based lookup if the record_id was not stored (D-15).
async fn dispatch_remove_cname_record(
    shared_cfg: &RelayConfig,
    per_server_cfg: &PerServerRelayConfig,
) {
    let payload = json!({
        "api_token": shared_cfg.dns_api_token.clone().unwrap_or_default(),
        "zone_id": shared_cfg.dns_zone_id.clone().unwrap_or_default(),
        "record_id": per_server_cfg.dns_record_id.clone().unwrap_or_default(),
        "subdomain": per_server_cfg.subdomain,
    });
    let task = Task::new("relay.remove_cname_record".to_string(), payload);
    match super::dns::handle_remove_record(task).await {
        Ok(v) => {
            info!(
                result = %v,
                server_id = %per_server_cfg.server_id,
                "RelayClient: remove_cname_record self-loop completed (D-13 / RESOLVED Q7)"
            );
        }
        Err(e) => {
            warn!(
                error = %e,
                server_id = %per_server_cfg.server_id,
                "RelayClient: remove_cname_record self-loop failed (will retry on next teardown)"
            );
        }
    }
}
