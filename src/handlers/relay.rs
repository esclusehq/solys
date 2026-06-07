//! Relay task dispatch
//!
//! Phase 68 (Plan 02). The three task types routed through here are:
//!
//! - `relay.connect` — request the agent to (re)open an outbound WSS to the
//!   Esluce Relay gateway and authenticate with the per-node `relay_token`.
//!   Implementation lives in [`crate::handlers::relay_client`]
//!   (added in Task 2).
//! - `relay.disconnect` — cancel the in-flight reconnect loop, send a
//!   graceful `TunnelDisconnect` frame, and close the WS.
//! - `relay.heartbeat` — force an immediate `TunnelHeartbeat` on the open
//!   yamux control stream (liveness probe initiated by the backend).
//!   Distinct from the periodic 10s ticker in `run_relay_client`.
//!
//! The task dispatch itself is intentionally a thin shim — all real work
//! happens in [`crate::handlers::relay_client`]. This file exists so the
//! dispatcher in `mod.rs` has a single typed entrypoint and so unit tests
//! can target the routing table in isolation.

use agent_proto::Task;
use anyhow::{anyhow, Result};
use serde_json::json;

/// Dispatch a relay.* task to the appropriate handler in
/// [`crate::handlers::relay_client`].
///
/// Returns the canonical "not yet wired" error in this build of the agent
/// (Task 1 deliverable). Task 2 of Plan 02 fills in the real implementation
/// and replaces this stub with a delegation to `relay_client::connect`,
/// `relay_client::disconnect`, and `relay_client::send_heartbeat`.
pub async fn handle_relay_task(task: &Task) -> Result<serde_json::Value> {
    let task_type = task.task_type.as_str();
    match task_type {
        // Real handlers are added in Task 2 — for now we emit a structured
        // "not yet wired" error so callers (the dispatcher / the dashboard)
        // can observe the routing rather than seeing a generic 500.
        "relay.connect" | "relay.disconnect" | "relay.heartbeat" => {
            Err(anyhow!(
                "relay task '{}' is not yet wired (Phase 68 Plan 02 Task 2)",
                task_type
            ))
        }
        other => Err(anyhow!("Unknown relay task type: {}", other)),
    }
}

/// Build the canonical success envelope for a relay.* action. Used by
/// Task 2's real implementations; kept here so the success shape is
/// declared once at the dispatch layer.
#[allow(dead_code)]
pub(crate) fn action_envelope(action: &str) -> serde_json::Value {
    json!({ "action": action })
}
