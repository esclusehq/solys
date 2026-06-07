use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::registry::TunnelHandle;
use crate::state::AppState;

/// The first message an agent sends on a fresh yamux control stream after WS upgrade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConnect {
    #[serde(rename = "type")]
    pub msg_type: String,
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
    let (mut tx, mut rx) = socket.split();

    // 1. Expect the first message to be a TunnelConnect (JSON text frame).
    let first_msg = match rx.next().await {
        Some(Ok(Message::Text(text))) => text,
        Some(Ok(other)) => {
            warn!("[TUNNEL] Expected first message Text, got {:?}", other);
            return;
        }
        Some(Err(e)) => {
            error!("[TUNNEL] WS error on first message: {}", e);
            return;
        }
        None => {
            warn!("[TUNNEL] WS closed before first message");
            return;
        }
    };

    let connect: TunnelConnect = match serde_json::from_str(&first_msg) {
        Ok(c) => c,
        Err(e) => {
            warn!("[TUNNEL] Invalid TunnelConnect JSON: {}", e);
            let _ = tx
                .send(Message::Text(
                    serde_json::json!({"error": "invalid_connect", "message": e.to_string()})
                        .to_string()
                        .into(),
                ))
                .await;
            return;
        }
    };

    if connect.msg_type != "tunnel_connect" {
        warn!("[TUNNEL] First message was not tunnel_connect: {}", connect.msg_type);
        return;
    }

    info!(
        "[TUNNEL] TunnelConnect: server_id={}, subdomain={}, agent_ip={}",
        connect.server_id, connect.subdomain, connect.agent_public_ip
    );

    // 2. Authorize the (relay_token, server_id) pair against the backend.
    //    The bearer token is read from the first WS message's `auth` field; for now
    //    we use a placeholder UUID and rely on the backend's `find_by_relay_token` lookup
    //    during tunnel event reporting.
    //    TODO: when the agent adds the bearer token to TunnelConnect, verify here.
    if let Err(e) = validate_subdomain(&connect.subdomain) {
        warn!("[TUNNEL] Invalid subdomain '{}': {}", connect.subdomain, e);
        return;
    }

    // 3. Build a yamux control handle. We don't have a real yamux session because
    //    the WebSocket is just a control plane in this MVP; yamux streams come over
    //    a side-channel. For now, register a placeholder handle and report lifecycle
    //    events to the backend.
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
        yamux_control: tokio::sync::Mutex::new(None),
        started_at: std::time::Instant::now(),
        bytes_in: std::sync::atomic::AtomicU64::new(0),
        bytes_out: std::sync::atomic::AtomicU64::new(0),
    });

    if let Err(e) = state.registry.register(handle.clone()) {
        warn!("[TUNNEL] Registry::register failed: {}", e);
        return;
    }

    // 4. Notify backend of the new tunnel.
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

    // 5. Heartbeat & message loop. We just track liveness; the actual heartbeat
    //    frame is sent by the agent on its own scheduler and we ack it here.
    let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(
        state.config.tunnel.heartbeat_interval_secs,
    ));
    heartbeat_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                // Server-initiated ping is not required; the heartbeat is purely
                // a liveness check based on the agent's last message timestamp.
                if let Err(e) = state.backend.report_tunnel_event(handle.server_id, "heartbeat", "ok").await {
                    warn!("[TUNNEL] Heartbeat backend report failed: {}", e);
                }
            }
            msg = rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(parsed) = serde_json::from_str::<TunnelMessage>(&text) {
                            handle_tunnel_message(&state, &handle, parsed).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("[TUNNEL] WS closed by agent: server_id={}", handle.server_id);
                        break;
                    }
                    Some(Ok(Message::Ping(p))) => {
                        let _ = tx.send(Message::Pong(p)).await;
                    }
                    Some(Err(e)) => {
                        warn!("[TUNNEL] WS read error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // 6. Cleanup: unregister and report disconnected.
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
            let _ = state
                .backend
                .report_tunnel_event_with_uptime(h.server_id, "heartbeat", "ok", h.tunnel_uptime_secs)
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
