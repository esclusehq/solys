//! Per-server outbound Esluce Relay tunnel client.
//!
//! Phase 70: Each active server on this agent node gets its own
//! independent WSS connection to the Esluce Relay gateway, its own yamux
//! session, and its own reconnect loop.  The per-server state is managed
//! by [`crate::state::RelayManager`].
//!
//! Architecture
//! ------------
//!
//! ```text
//!  RelayManager (state.rs) {
//!      cancel: CancellationToken      // parent — cascades to all children
//!      servers: RwLock<HashMap<Uuid, RelayClientHandle>>
//!  }
//!
//!  RelayClientHandle (state.rs) {      // × N (one per server)
//!      cancel: child_token(),
//!      join: JoinHandle,
//!      control_tx: UnboundedSender,
//!      bytes_transferred: AtomicU64,
//!      tunnel_start: Instant,
//!  }
//! ```
//!
//! Each per-server tunnel:
//!   ──[WSS]──> Gateway
//!        └── yamux session (client)
//!             ├── control stream (TunnelConnect, heartbeats)
//!             └── inbound player streams → local MC (`local_mc_addr`)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
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
use crate::state::RelayServerConfig;

use super::relay_session;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Heartbeat cadence for the open tunnel.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

/// Re-handshake the WS after this much wall-clock tunnel uptime (24 h).
const REKEY_UPTIME: Duration = Duration::from_secs(24 * 60 * 60);

/// Re-handshake the WS after this many bytes (100 GiB).
const REKEY_BYTES: u64 = 100 * 1024 * 1024 * 1024;

/// Backoff parameters for the reconnect loop.
const BACKOFF_INITIAL_MS: u64 = 1_000;
const BACKOFF_MAX_MS: u64 = 30_000;
const BACKOFF_JITTER_PCT: u64 = 20;

/// Buffer size for the WS ↔ duplex bridge task (64 KiB).
const BRIDGE_BUFFER_BYTES: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Per-server reconnect loop
// ---------------------------------------------------------------------------

/// Per-server reconnect loop. Exits when `shutdown` is cancelled.
///
/// Takes a full [`RelayServerConfig`] directly — no shared config lookup
/// needed. Heartbeat staggering (0-10s jitter) is applied after each
/// successful connect.
pub async fn run_relay_client(
    cfg: RelayServerConfig,
    shutdown: CancellationToken,
) {
    let server_id = cfg.server_id;
    let mut backoff_ms: u64 = BACKOFF_INITIAL_MS;

    info!(
        server_id = %server_id,
        subdomain = %cfg.subdomain,
        "PerServer tunnel: entering reconnect loop"
    );

    loop {
        if shutdown.is_cancelled() {
            info!("PerServer[{}]: shutdown requested, exiting", server_id);
            break;
        }

        match connect_and_run(&cfg, &shutdown).await {
            Ok(()) => {
                backoff_ms = BACKOFF_INITIAL_MS;
            }
            Err(e) => {
                let delay_s = backoff_ms / 1000;
                warn!(
                    server_id = %server_id,
                    error = %e,
                    "relay tunnel disconnected, reconnecting in {}s",
                    delay_s,
                );
            }
        }

        if shutdown.is_cancelled() { break; }

        let sleep_ms = backoff_with_jitter(backoff_ms);
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        backoff_ms = (backoff_ms.saturating_mul(2)).min(BACKOFF_MAX_MS);
    }

    info!(
        server_id = %server_id,
        "relay tunnel disconnected permanently",
    );
}

/// Per-server tunnel lifecycle: WS handshake → TunnelConnect → heartbeat
/// loop + inbound stream drive → cleanup.
async fn connect_and_run(
    cfg: &RelayServerConfig,
    shutdown: &CancellationToken,
) -> Result<()> {
    // 1. Parse the gateway URL.
    let uri: Uri = cfg.gateway_url.parse()
        .with_context(|| format!("invalid gateway_url: {}", cfg.gateway_url))?;

    // 2. Build the WS handshake request with the Bearer token.
    let req = build_ws_request(uri.clone(), &cfg.token)?;

    // 3. Transition: connecting — before the WS upgrade.
    info!(
        server_id = %cfg.server_id,
        "connecting to relay gateway"
    );

    let (ws_stream, _response) = tokio_tungstenite::connect_async_tls_with_config(
        req, None, true, None,
    )
    .await
    .with_context(|| "RelayClient: WS handshake failed")?;
    info!(server_id = %cfg.server_id, "RelayClient: WS handshake complete");

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
        "server_id": cfg.server_id,
        "subdomain": cfg.subdomain,
        "public_port": cfg.public_port,
        "agent_public_ip": cfg.agent_public_ip,
        "region": cfg.region,
        "loader": cfg.loader,
    });
    let mut connect_bytes = serde_json::to_vec(&connect_msg)?;
    connect_bytes.push(b'\n');
    control
        .write_all(&connect_bytes)
        .await
        .map_err(|e| anyhow!("TunnelConnect write failed: {}", e))?;
    info!(
        server_id = %cfg.server_id,
        subdomain = %cfg.subdomain,
        "RelayClient: TunnelConnect sent"
    );

    audit::log_relay_tunnel_event(
        cfg.server_id,
        cfg.server_id,
        "connected",
        &format!("subdomain={}", cfg.subdomain),
    )
    .await;

    // Transition: connected.
    info!(
        server_id = %cfg.server_id,
        subdomain = %cfg.subdomain,
        "relay tunnel connected"
    );

    // 7. Install the control_tx channel on the RelayClientHandle.
    let (ctrl_tx, ctrl_rx) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
    let bytes_counter: Arc<AtomicU64>;
    let tunnel_start: Instant;

    // Look up the handle and update control_tx + counters.
    let rt = crate::state::relay_manager();
    {
        let mut servers = rt.servers.write().await;
        if let Some(handle) = servers.get_mut(&cfg.server_id) {
            handle.bytes_transferred.store(0, Ordering::Relaxed);
            handle.tunnel_start = Instant::now();
            handle.control_tx = ctrl_tx;
            bytes_counter = handle.bytes_transferred.clone();
            tunnel_start = Instant::now();
        } else {
            warn!(
                server_id = %cfg.server_id,
                "RelayClientHandle vanished before tunnel connect completed"
            );
            bridge_handle.abort();
            let _ = bridge_handle.await;
            return Ok(());
        }
    }

    // Phase 69-04: Stagger first heartbeat 0-10s.
    let jitter_ms: u64 = rand::thread_rng().gen_range(0..=10_000);
    tokio::time::sleep(Duration::from_millis(jitter_ms)).await;
    debug!(
        server_id = %cfg.server_id,
        jitter_ms,
        "heartbeat staggered"
    );

    // 8. Spawn the heartbeat task.
    let heartbeat_cancel = shutdown.clone();
    let heartbeat_server_id = cfg.server_id;
    let heartbeat_handle = tokio::spawn(async move {
        run_heartbeat_task(
            control, ctrl_rx, bytes_counter, tunnel_start,
            heartbeat_cancel, heartbeat_server_id,
        ).await;
    });

    // 9. Drive incoming streams until the session ends.
    let is_udp = cfg.loader.as_deref() == Some("bedrock");
    let session_result = drive_inbound_streams(
        &mut session,
        &cfg.local_mc_addr,
        cfg.server_id,
        shutdown,
        is_udp,
    ).await;

    // 10. Cleanup: clear control_tx, abort bridge + heartbeat.
    {
        let mut servers = rt.servers.write().await;
        if let Some(handle) = servers.get_mut(&cfg.server_id) {
            handle.control_tx = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>().0;
        }
    }
    heartbeat_handle.abort();
    let _ = heartbeat_handle.await;
    bridge_handle.abort();
    let _ = bridge_handle.await;

    // 11. Dispatch remove_cname_record cleanup (best-effort).
    dispatch_remove_cname_record(cfg).await;

    session_result
}

/// Drive the yamux session's incoming stream queue.
async fn drive_inbound_streams(
    session: &mut Session<DuplexStream>,
    local_mc_addr: &str,
    server_id: Uuid,
    shutdown: &CancellationToken,
    is_udp: bool,
) -> Result<()> {
    // Resolve container IP via Docker so multiple servers on the same port
    // don't collide on 127.0.0.1. For UDP (Bedrock), use local_mc_addr
    // directly — no Docker resolve needed (D-11).
    let resolved_addr = if is_udp {
        local_mc_addr.to_string()
    } else {
        resolve_container_addr(&server_id, local_mc_addr).await
            .unwrap_or_else(|| local_mc_addr.to_string())
    };
    if resolved_addr != local_mc_addr {
        info!(from = %local_mc_addr, to = %resolved_addr, "RelayClient: resolved container address");
    }

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
                        let local = resolved_addr.clone();
                        let bytes_counter = {
                            let servers = crate::state::relay_manager().servers.read().await;
                            servers.get(&server_id)
                                .map(|h| h.bytes_transferred.clone())
                        };
                        if let Some(counter) = bytes_counter {
                            let local_for_task = local.clone();
                            tokio::spawn(async move {
                                if is_udp {
                                    if let Err(e) = relay_session::run_udp_relay_session(
                                        stream, local_for_task.clone(), counter,
                                    ).await {
                                        warn!(error = %e, local = %local_for_task, "UDP relay session error");
                                    }
                                } else {
                                    if let Err(e) = relay_session::run_relay_session(
                                        stream, local_for_task.clone(), counter,
                                    ).await {
                                        warn!(error = %e, local = %local_for_task, "Relay session error");
                                    }
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

/// Look up a container's IP via Docker and return it as `{ip}:{port}`.
/// Falls back to `None` if Docker isn't available or the container
/// isn't running. Uses the same network resolution pattern as rcon.rs.
///
/// The port is taken from the container's port bindings — we prefer the
/// standard Minecraft port (25565), then the first exposed TCP port, and
/// finally the host port from `local_mc_addr`. This avoids mismatches
/// where the Docker host port differs from the container's internal port.
async fn resolve_container_addr(server_id: &Uuid, local_mc_addr: &str) -> Option<String> {
    let docker = crate::state::docker_global()?;
    let container_name = format!("mc-{}", server_id);
    let inspect = docker.inspect_container(&container_name, None).await.ok()?;
    let ip = inspect
        .network_settings
        .as_ref()
        .and_then(|ns| ns.networks.as_ref())
        .and_then(|networks| {
            networks.values().find_map(|ep| {
                let ip = ep.ip_address.as_ref()?;
                if !ip.is_empty() && ip != "0.0.0.0" {
                    Some(ip.clone())
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            inspect.network_settings.as_ref()?.ip_address.as_ref().filter(|ip| !ip.is_empty() && *ip != "0.0.0.0").cloned()
        })?;
    let port = inspect
        .network_settings
        .as_ref()
        .and_then(|ns| ns.ports.as_ref())
        .and_then(|ports| {
            if ports.contains_key("25565/tcp") {
                Some("25565".to_string())
            } else {
                ports.keys()
                    .find(|p| p.ends_with("/tcp"))
                    .and_then(|p| p.split('/').next())
                    .map(|p| p.to_string())
            }
        })
        .or_else(|| {
            local_mc_addr.split(':').nth(1).map(|p| p.to_string())
        })
        .unwrap_or_else(|| "25565".to_string());
    Some(format!("{}:{}", ip, port))
}

/// 10 s heartbeat ticker + rekey thresholds.
async fn run_heartbeat_task(
    mut control: StreamHandle,
    mut ctrl_rx: tokio::sync::mpsc::UnboundedReceiver<serde_json::Value>,
    bytes_counter: Arc<AtomicU64>,
    tunnel_start: Instant,
    shutdown: CancellationToken,
    server_id: Uuid,
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
                let uptime = tunnel_start.elapsed().as_secs();
                let bytes = bytes_counter.load(Ordering::Relaxed);

                if tunnel_start.elapsed() >= REKEY_UPTIME || bytes >= REKEY_BYTES {
                    info!(
                        server_id = %server_id,
                        uptime_secs = uptime,
                        bytes_transferred = bytes,
                        "rekeying relay session"
                    );
                    audit::log_relay_tunnel_event(
                        Uuid::nil(), Uuid::nil(), "rekey",
                        &format!("uptime_secs={} bytes={}", uptime, bytes),
                    ).await;
                    let _ = control.shutdown().await;
                    return;
                }

                debug!(server_id = %server_id, "sending heartbeat");
                let msg = json!({
                    "type": "tunnel_heartbeat",
                    "tunnel_uptime_secs": uptime,
                });
                let mut bytes = serde_json::to_vec(&msg).unwrap_or_default();
                bytes.push(b'\n');
                if control.write_all(&bytes).await.is_err() {
                    warn!("RelayClient: heartbeat write failed, exiting heartbeat loop");
                    return;
                }
                audit::log_relay_tunnel_event(
                    Uuid::nil(), Uuid::nil(), "heartbeat",
                    &format!("uptime_secs={}", uptime),
                ).await;
            }
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

// ---------------------------------------------------------------------------
// WS ↔ duplex bridge
// ---------------------------------------------------------------------------

/// Pump bytes between the yamux duplex side and WS Binary messages.
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
                    Some(Ok(_)) => continue,
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

/// Apply ±20% jitter to a backoff value.
fn backoff_with_jitter(backoff_ms: u64) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let jitter = (nanos % (BACKOFF_JITTER_PCT * 2 + 1)).saturating_sub(BACKOFF_JITTER_PCT);
    let delta = backoff_ms.saturating_mul(jitter) / 100;
    backoff_ms.saturating_add(delta)
}

/// Best-effort DNS record cleanup on tunnel teardown.
async fn dispatch_remove_cname_record(cfg: &RelayServerConfig) {
    use crate::handlers::dns;
    let payload = json!({
        "api_token": "",
        "zone_id": "",
        "record_id": "",
        "subdomain": cfg.subdomain,
    });
    let task = agent_proto::Task::new("relay.remove_cname_record".to_string(), payload);
    match dns::handle_remove_record(task).await {
        Ok(v) => {
            info!(
                result = %v,
                server_id = %cfg.server_id,
                "RelayClient: remove_cname_record self-loop completed"
            );
        }
        Err(e) => {
            warn!(
                error = %e,
                server_id = %cfg.server_id,
                "RelayClient: remove_cname_record self-loop failed"
            );
        }
    }
}
