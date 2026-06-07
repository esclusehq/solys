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

/// Dispatch a relay.* task to the appropriate handler in
/// [`crate::handlers::relay_client`].
pub async fn handle_relay_task(task: &Task) -> Result<serde_json::Value> {
    let task_type = task.task_type.as_str();
    match task_type {
        "relay.connect" => {
            crate::handlers::relay_client::connect().await
        }
        "relay.disconnect" => {
            crate::handlers::relay_client::disconnect().await
        }
        "relay.heartbeat" => {
            crate::handlers::relay_client::send_heartbeat(task).await
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
