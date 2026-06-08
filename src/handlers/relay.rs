//! Relay task dispatch
//!
//! Phase 68 (Plan 02). The three task types routed through here are:
//!
//! - `relay.connect` — request the agent to (re)open an outbound WSS to
//!   the Esluce Relay gateway and authenticate with the per-node
//!   `relay_token`. Implementation lives in
//!   [`crate::handlers::relay_client`]. Idempotent: a second call
//!   while one is running is a no-op.
//! - `relay.disconnect` — cancel the in-flight reconnect loop and
//!   close the WS.
//! - `relay.heartbeat` — force an immediate `TunnelHeartbeat` on the
//!   open yamux control stream (liveness probe initiated by the
//!   backend). Distinct from the periodic 10 s ticker in
//!   `run_relay_client`.
//!
//! The task dispatch itself is intentionally a thin shim — all real
//! work happens in [`crate::handlers::relay_client`]. This file
//! exists so the dispatcher in `mod.rs` has a single typed entrypoint
//! and so unit tests can target the routing table in isolation.

use agent_proto::Task;
use anyhow::{anyhow, Result};
use serde_json::json;
use uuid::Uuid;

use crate::state::PerServerRelayConfig;

/// Dispatch a relay.* task to the appropriate handler in
/// [`crate::handlers::relay_client`].
///
/// Phase 69 (Plan 02): all relay.* tasks now require a `server_id` in
/// the task payload. `connect` additionally requires the per-server
/// config fields (subdomain, public_port, local_mc_addr).
pub async fn handle_relay_task(task: &Task) -> Result<serde_json::Value> {
    let task_type = task.task_type.as_str();
    match task_type {
        "relay.connect" => {
            let server_id_str = task.payload.get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing server_id in relay.connect payload"))?;
            let server_id = Uuid::parse_str(server_id_str)
                .map_err(|e| anyhow!("Invalid server_id: {}", e))?;
            let subdomain = task.payload.get("subdomain")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing subdomain in relay.connect payload"))?
                .to_string();
            let public_port = task.payload.get("public_port")
                .and_then(|v| v.as_u64())
                .and_then(|n| u16::try_from(n).ok())
                .ok_or_else(|| anyhow!("Missing or invalid public_port in relay.connect payload"))?;
            let local_mc_addr = task.payload.get("local_mc_addr")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing local_mc_addr in relay.connect payload"))?
                .to_string();
            let dns_record_id = task.payload.get("dns_record_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let per_server_cfg = PerServerRelayConfig {
                server_id,
                subdomain,
                public_port,
                local_mc_addr,
                dns_record_id,
            };
            crate::handlers::relay_client::connect(server_id, per_server_cfg).await
        }
        "relay.disconnect" => {
            let server_id_str = task.payload.get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing server_id in relay.disconnect payload"))?;
            let server_id = Uuid::parse_str(server_id_str)
                .map_err(|e| anyhow!("Invalid server_id: {}", e))?;
            crate::handlers::relay_client::disconnect(server_id).await
        }
        "relay.heartbeat" => {
            let server_id_str = task.payload.get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing server_id in relay.heartbeat payload"))?;
            let server_id = Uuid::parse_str(server_id_str)
                .map_err(|e| anyhow!("Invalid server_id: {}", e))?;
            crate::handlers::relay_client::send_heartbeat(server_id).await
        }
        other => Err(anyhow!("Unknown relay task type: {}", other)),
    }
}

/// Build the canonical success envelope for a relay.* action. Kept
/// here so the success shape is declared once at the dispatch layer.
#[allow(dead_code)]
pub(crate) fn action_envelope(action: &str) -> serde_json::Value {
    json!({ "action": action })
}
