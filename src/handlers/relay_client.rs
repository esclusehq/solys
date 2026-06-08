//! Outbound Esluce Relay tunnel client.
//!
//! Phase 68 (Plan 02). The agent opens an outbound WSS to the Esluce Relay
//! gateway (`wss://relay.esluce.net/relay/tunnel`), authenticates with the
//! per-node `relay_token`, opens a yamux session, and forwards any inbound
//! yamux streams to the local Minecraft Java server (`127.0.0.1:25565` by
//! default).
//!
//! Architecture
//! ------------
//!
//! ```text
//!  gateway <──WSS/binary frames──> WebSocketStream
//!                                     │
//!                              (ws_bridge task)
//!                                     │
//!                              tokio::io::duplex
//!                                     │
//!                              tokio_yamux::Session (client)
//!                                  │
//!                          (per inbound stream)
//!                                  │
//!                          run_relay_session
//!                                  │
//!                              TcpStream 127.0.0.1:25565
//! ```
//!
//! Lifecycle
//! ---------
//!
//! 1. [`connect`] is invoked by the `relay.connect` task arm. It spawns
//!    [`run_relay_client`] (idempotent: a second call while one is running
//!    is a no-op).
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
//!    self-loop (D-13 / RESOLVED Q7) so the stale
//!    `<subdomain>.play.esluce.com` A record is removed.
//! 5. [`disconnect`] cancels the cancel token, awaits the join handle,
//!    and returns.
//! 6. [`send_heartbeat`] (the `relay.heartbeat` task arm) writes a
//!    `TunnelHeartbeat` JSON frame to the current control stream. If
//!    no tunnel is open, returns `Ok(())` silently.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite, DuplexStream};
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::{
    self,
    client::IntoClientRequest,
    http::Uri,
    Message,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use agent_proto::Task;
use tokio_yamux::{Config as YamuxConfig, Session, StreamHandle};

use crate::audit;
use crate::state::{self, RelayConfig};

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
// Shared runtime state
// ---------------------------------------------------------------------------

/// Process-global runtime state for the relay tunnel. Mirrors the
/// `DOCKER_GLOBAL` pattern in [`crate::state`].
struct RelayRuntime {
    /// Join handle for the current [`run_relay_client`] task. `Some`
    /// while the reconnect loop is active.
    join: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Cancel token; set to cancel the current [`run_relay_client`] task.
    cancel: CancellationToken,
    /// Channel for sending `TunnelHeartbeat` / `TunnelDisconnect` JSON
    /// payloads to the currently-active control stream. `None` while no
    /// tunnel is open.
    control_tx: Mutex<Option<tokio::sync::mpsc::UnboundedSender<serde_json::Value>>>,
    /// Aggregated bytes across all in-flight yamux sessions, updated by
    /// every [`super::relay_session::run_relay_session`] call.
    bytes_transferred: Arc<AtomicU64>,
    /// Tunnel start time, set when the current yamux session becomes
    /// active. Used by the heartbeat task to evaluate the 24 h rekey.
    tunnel_start: Mutex<Option<Instant>>,
}

static RELAY_RUNTIME: OnceLock<RelayRuntime> = OnceLock::new();

fn runtime() -> &'static RelayRuntime {
    RELAY_RUNTIME.get_or_init(|| RelayRuntime {
        join: Mutex::new(None),
        cancel: CancellationToken::new(),
        control_tx: Mutex::new(None),
        bytes_transferred: Arc::new(AtomicU64::new(0)),
        tunnel_start: Mutex::new(None),
    })
}

/// Public accessor for the global cancellation token. Used by the
/// bootstrap in `main.rs` to wire the same token the reconnect loop
/// uses, so the main shutdown signal can cascade.
pub fn cancel_token() -> CancellationToken {
    runtime().cancel.clone()
}

// ---------------------------------------------------------------------------
// Public task entrypoints
// ---------------------------------------------------------------------------

/// `relay.connect` task arm: spawn [`run_relay_client`] (idempotent).
pub async fn connect() -> Result<serde_json::Value> {
    let cfg = state::relay_config().ok_or_else(|| {
        anyhow!("RelayClient: no relay config set; check AGENT_RELAY_TOKEN env var")
    })?;

    let mut join_guard = runtime().join.lock().await;
    if join_guard.is_some() {
        info!("RelayClient: reconnect loop already running; connect() is a no-op");
        return Ok(json!({
            "action": "connect",
            "status": "already_running",
        }));
    }

    let cancel = runtime().cancel.clone();
    let handle = tokio::spawn(run_relay_client(cancel));
    *join_guard = Some(handle);

    info!(
        gateway = %cfg.gateway_url,
        "RelayClient: reconnect loop started"
    );
    Ok(json!({
        "action": "connect",
        "status": "started",
    }))
}

/// `relay.disconnect` task arm: cancel the reconnect loop and wait for
/// the join handle to finish.
pub async fn disconnect() -> Result<serde_json::Value> {
    info!("RelayClient: disconnect requested");
    runtime().cancel.cancel();

    let handle = {
        let mut join_guard = runtime().join.lock().await;
        join_guard.take()
    };
    if let Some(h) = handle {
        // Wait for the loop to exit; cap at 10 s so we don't hang the
        // task dispatcher.
        let _ = tokio::time::timeout(Duration::from_secs(10), h).await;
    }

    // Clear the bytes/start markers so the next connect() starts fresh.
    runtime().bytes_transferred.store(0, Ordering::Relaxed);
    *runtime().tunnel_start.lock().await = None;
    *runtime().control_tx.lock().await = None;

    Ok(json!({
        "action": "disconnect",
        "status": "stopped",
    }))
}

/// `relay.heartbeat` task arm: send an immediate `TunnelHeartbeat` on
/// the open control stream. If no tunnel is open, returns `Ok(())`
/// silently (the periodic 10 s ticker in the reconnect loop is the
/// primary heartbeat path; this is for backend-initiated liveness
/// probes only).
pub async fn send_heartbeat(_task: &Task) -> Result<serde_json::Value> {
    let uptime = runtime()
        .tunnel_start
        .lock()
        .await
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);

    let payload = json!({
        "type": "tunnel_heartbeat",
        "tunnel_uptime_secs": uptime,
    });

    let tx_guard = runtime().control_tx.lock().await;
    if let Some(tx) = tx_guard.as_ref() {
        let _ = tx.send(payload);
        Ok(json!({
            "action": "heartbeat",
            "status": "sent",
            "tunnel_uptime_secs": uptime,
        }))
    } else {
        Ok(json!({
            "action": "heartbeat",
            "status": "no_tunnel_open",
        }))
    }
}

// ---------------------------------------------------------------------------
// Main reconnect loop
// ---------------------------------------------------------------------------

/// The outer reconnect loop. Runs forever (or until `shutdown` is
/// cancelled). Each iteration performs one full handshake + yamux
/// session lifecycle, then sleeps with exponential backoff before the
/// next attempt.
pub async fn run_relay_client(shutdown: CancellationToken) {
    let cfg = match state::relay_config() {
        Some(c) => c,
        None => {
            error!("RelayClient: no relay config; run_relay_client exiting");
            return;
        }
    };

    let mut backoff_ms: u64 = BACKOFF_INITIAL_MS;
    let mut node_id = Uuid::nil();

    info!(
        gateway = %cfg.gateway_url,
        "RelayClient: entering reconnect loop"
    );

    loop {
        if shutdown.is_cancelled() {
            info!("RelayClient: shutdown requested, exiting reconnect loop");
            break;
        }

        match connect_and_run(&cfg, &shutdown, &mut node_id).await {
            Ok(()) => {
                info!("RelayClient: connect_and_run returned cleanly (re-handshake or shutdown)");
                backoff_ms = BACKOFF_INITIAL_MS;
            }
            Err(e) => {
                warn!(error = %e, "RelayClient: connect_and_run failed");
            }
        }

        if shutdown.is_cancelled() {
            break;
        }

        let sleep_ms = backoff_with_jitter(backoff_ms);
        info!(
            backoff_ms = sleep_ms,
            "RelayClient: sleeping before next reconnect attempt"
        );
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        backoff_ms = (backoff_ms.saturating_mul(2)).min(BACKOFF_MAX_MS);
    }

    info!("RelayClient: reconnect loop exited");
}

async fn connect_and_run(
    cfg: &RelayConfig,
    shutdown: &CancellationToken,
    node_id: &mut Uuid,
) -> Result<()> {
    // 1. Parse the gateway URL.
    let uri: Uri = cfg.gateway_url.parse()
        .with_context(|| format!("invalid gateway_url: {}", cfg.gateway_url))?;

    // 2. Build the WS handshake request with the Bearer token.
    let req = build_ws_request(uri.clone(), &cfg.token)?;

    // 3. Upgrade to WSS. The native-tls feature on tokio-tungstenite
    //    handles TCP+TLS+WS internally and returns
    //    `WebSocketStream<MaybeTlsStream<TcpStream>>`. This works for
    //    both `ws://` (no TLS) and `wss://` (TLS) URLs.
    let (ws_stream, _response) = tokio_tungstenite::connect_async_tls_with_config(
        req,
        None,
        true,   // disable_nagle
        None,   // default connector
    )
    .await
    .with_context(|| "RelayClient: WS handshake failed")?;
    info!("RelayClient: WS handshake complete");

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
        "relay_token": cfg.token,
        "server_id": Uuid::nil(),         // TEMP: per_server_cfg.server_id in Task 2
        "subdomain": "unknown",            // TEMP: per_server_cfg.subdomain in Task 2
        "public_port": 25565,              // TEMP: per_server_cfg.public_port in Task 2
        "agent_public_ip": cfg.agent_public_ip,
        "region": cfg.region,
    });
    let mut connect_bytes = serde_json::to_vec(&connect_msg)?;
    // NDJSON framing: the gateway's read_json_message demuxer reads bytes
    // until it sees a '\n' byte. Without this newline, the gateway would
    // concatenate TunnelConnect with the next heartbeat and fail JSON
    // deserialization with "trailing characters".
    connect_bytes.push(b'\n');
    control
        .write_all(&connect_bytes)
        .await
        .map_err(|e| anyhow!("TunnelConnect write failed: {}", e))?;
    info!("RelayClient: TunnelConnect sent");

    audit::log_relay_tunnel_event(
        *node_id,
        Uuid::nil(),  // server_id is server-specific; resolved by gateway from subdomain
        "connected",
        "subdomain=unknown",              // TEMP: per_server_cfg.subdomain in Task 2
    )
    .await;

    // 7. Reset byte counter + start time; install the control_tx
    // channel so on-demand heartbeats / disconnects can talk to the
    // gateway.
    runtime().bytes_transferred.store(0, Ordering::Relaxed);
    *runtime().tunnel_start.lock().await = Some(Instant::now());

    let (ctrl_tx, ctrl_rx) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
    *runtime().control_tx.lock().await = Some(ctrl_tx);

    // 8. Spawn the heartbeat task (T-68-03 / T-68-04 + D-25 rekey).
    let bytes_counter = runtime().bytes_transferred.clone();
    let tunnel_start = *runtime().tunnel_start.lock().await;
    let heartbeat_cancel = shutdown.clone();
    let heartbeat_handle = tokio::spawn(async move {
        run_heartbeat_task(control, ctrl_rx, bytes_counter, tunnel_start, heartbeat_cancel).await;
    });

    // 9. Drive incoming streams until the session ends.
    let local_mc_addr = "127.0.0.1:25565"; // TEMP: per_server_cfg.local_mc_addr in Task 2
    let session_result = drive_inbound_streams(&mut session, local_mc_addr, shutdown).await;

    // 10. Cleanup: drop the control_tx so on-demand heartbeats fail
    // fast, cancel the heartbeat task, await the bridge.
    *runtime().control_tx.lock().await = None;
    *runtime().tunnel_start.lock().await = None;
    heartbeat_handle.abort();
    let _ = heartbeat_handle.await;
    bridge_handle.abort();
    let _ = bridge_handle.await;

    // 11. D-13 / RESOLVED Q7: dispatch the remove_cname_record
    // self-loop. We call `dns::handle_remove_record` directly with a
    // constructed `Task` rather than going through the
    // `execute_single` dispatcher — the agent has no public local
    // task queue, and the result of this call only needs to be
    // audit-logged, not round-tripped back to the backend. The
    // dispatcher arm in `mod.rs` is the path used when the backend
    // itself sends a `relay.remove_cname_record` task.
    dispatch_remove_cname_record(cfg).await;

    session_result
}

/// Drive the yamux session's incoming stream queue. Each new inbound
/// stream is handed off to [`super::relay_session::run_relay_session`].
async fn drive_inbound_streams(
    session: &mut Session<DuplexStream>,
    local_mc_addr: &str,
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
                        let bytes_counter = runtime().bytes_transferred.clone();
                        tokio::spawn(async move {
                            if let Err(e) = relay_session::run_relay_session(
                                stream,
                                local.clone(),
                                bytes_counter,
                            ).await {
                                warn!(error = %e, local = %local, "Relay session exited with error");
                            }
                        });
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
/// TEMP (Task 2): will take (shared_cfg, per_server_cfg) and use
/// per_server_cfg.subdomain + remove dns_record_id (D-15).
async fn dispatch_remove_cname_record(cfg: &RelayConfig) {
    let payload = json!({
        "api_token": cfg.dns_api_token.clone().unwrap_or_default(),
        "zone_id": cfg.dns_zone_id.clone().unwrap_or_default(),
        "record_id": String::new(),       // TEMP: per-server field removed from RelayConfig
        "subdomain": "unknown",            // TEMP: per_server_cfg.subdomain in Task 2
    });
    let task = Task::new("relay.remove_cname_record".to_string(), payload);
    match super::dns::handle_remove_record(task).await {
        Ok(v) => {
            info!(
                result = %v,
                "RelayClient: remove_cname_record self-loop completed (D-13 / RESOLVED Q7)"
            );
        }
        Err(e) => {
            // We don't want a transient CF failure to take down the
            // reconnect loop — the next tunnel teardown will retry.
            warn!(
                error = %e,
                "RelayClient: remove_cname_record self-loop failed (will retry on next teardown)"
            );
        }
    }
}
