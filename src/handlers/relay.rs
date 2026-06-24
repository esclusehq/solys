//! Relay task dispatch (Phase 70 — heartbeat only).
//!
//! `relay.connect` and `relay.disconnect` are DEPRECATED and handled in
//! `handlers/mod.rs` as hard errors. Tunnel lifecycle is driven entirely
//! by `RelayConfigSync` WebSocket push.
//!
//! The only remaining relay.* task is `relay.heartbeat`, which sends an
//! immediate TunnelHeartbeat on the per-server control stream.

use agent_proto::Task;
use anyhow::{anyhow, Result};
use serde_json::json;
use uuid::Uuid;

/// Dispatch a relay.* task.
///
/// Phase 70: only `relay.heartbeat` is supported. `connect` / `disconnect`
/// return an error at the dispatch level in `mod.rs`.
pub async fn handle_relay_task(task: &Task) -> Result<serde_json::Value> {
    match task.task_type.as_str() {
        "relay.heartbeat" => {
            let server_id_str = task.payload.get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing server_id in relay.heartbeat payload"))?;
            let server_id = Uuid::parse_str(server_id_str)
                .map_err(|e| anyhow!("Invalid server_id: {}", e))?;
            crate::state::relay_manager().send_heartbeat(&server_id).await
                .map_err(|e| anyhow!("{}", e))
        }
        other => Err(anyhow!("Unknown relay task type: {}", other)),
    }
}
