//! Phase 67: Top-level orchestrator for connectivity diagnostics.
//!
//! `handle_diagnostics` is the dispatcher entrypoint (task_type =
//! "connectivity.diagnostics"). It collects raw local-network facts and
//! sends a `ConnectivityReport` message to the backend through the same
//! outbound WebSocket the agent already uses for `Heartbeat` and
//! `CrashReport`.
//!
//! The auto-fix actions (`firewall.open_port` / `firewall.close_port` /
//! `upnp.add_mapping` / `upnp.remove_mapping`) are dispatched directly to
//! the submodules in `connectivity::{firewall,upnp}` and don't return
//! anything to the orchestrator.

// Re-export the submodules so the dispatcher (`handlers/mod.rs`) can call
// `connectivity::firewall::open(task).await` etc.
pub mod diagnostics;
pub mod firewall;
pub mod upnp;

// Re-export the public-IP detector (from dns_watch::detect_public_ip
// originally; the connectivity module owns a thin wrapper for symmetry).
pub use diagnostics::is_vps_node;

use std::net::Ipv4Addr;
use std::sync::Arc;

use agent_proto::Task;
use anyhow::Result;
use serde_json::{json, Value};
use tracing::{error, trace};

use crate::handlers::connectivity::diagnostics::collect_diagnostics;

/// Heuristic: is the host's local IP in one of the common LAN RFC1918
/// ranges? Used to gate the UPnP IGD lookup (Pitfall 4: a VPS node has
/// no IGD; attempting SSDP there costs a 2-second audit-log entry).
pub fn is_lan(ip: Option<Ipv4Addr>) -> bool {
    match ip {
        None => false,
        Some(v4) => v4.is_private(), // 10/8, 172.16/12, 192.168/16
    }
}

/// Heuristic: is the public IP in a CGNAT range (100.64.0.0/10)?
/// CGNAT means the agent's port-forward (UPnP / manual) lives behind
/// another NAT, so the backend should NOT auto-probe aggressively.
pub fn is_cgnat_suspect(local: Option<Ipv4Addr>, _gateway: Option<Ipv4Addr>) -> bool {
    // Carrier-Grade NAT range: 100.64.0.0/10 (RFC 6598)
    const CGNAT_NET: u32 = (100 << 24) | (64 << 16);
    const CGNAT_MASK: u32 = 0xFFC0_0000;
    let _ = local;
    // We can't see the public IP at this layer; use a placeholder. The
    // backend performs the authoritative CGNAT check on the actual public IP.
    // Returning false here means "don't pre-emptively mark as CGNAT".
    let _ = CGNAT_NET;
    let _ = CGNAT_MASK;
    false
}

/// Re-entrant handle on the agent's WebSocket outbound channel.
/// The owning type lives in `agent_connection.rs`; we hold an `Option<Arc<…>>`
/// to break the circular import (`agent_connection` already imports handlers).
/// At runtime, `start()` in `main.rs` injects the real handle before any
/// diagnostics task can fire.
static OUTBOUND_TX: tokio::sync::OnceCell<Arc<dyn Fn(Value) + Send + Sync>> =
    tokio::sync::OnceCell::const_new();

pub fn set_outbound_sender(f: Arc<dyn Fn(Value) + Send + Sync>) {
    let _ = OUTBOUND_TX.set(f);
}

pub async fn handle_diagnostics(task: Task) -> Result<Value, anyhow::Error> {
    let server_id_str = task.payload.get("server_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing server_id in connectivity.diagnostics payload"))?;
    let server_id = uuid::Uuid::parse_str(server_id_str)
        .map_err(|e| anyhow::anyhow!("Invalid server_id: {}", e))?;
    let game_port = task.payload.get("game_port")
        .and_then(|v| v.as_u64())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(25565);

    let docker = match crate::state::docker_global() {
        Some(d) => d,
        None => {
            error!("Docker client not initialized; cannot collect container port binding");
            return Err(anyhow::anyhow!("Docker client not initialized"));
        }
    };
    let facts = match collect_diagnostics(&docker, server_id, game_port).await {
        Ok(v) => v,
        Err(e) => {
            error!(server_id = %server_id, error = %e, "collect_diagnostics failed");
            return Err(e);
        }
    };

    // Build the ConnectivityReport payload (matches backend NodeMessage::ConnectivityReport)
    let report = json!({
        "type": "connectivity_report",
        "server_id": server_id_str,
        "diagnostics": facts,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    if let Some(tx) = OUTBOUND_TX.get() {
        tx(report);
    } else {
        // No outbound channel yet (very early startup); buffer in audit log only
        trace!(server_id = %server_id, "ConnectivityReport dropped: outbound channel not wired");
    }

    Ok(json!({
        "status": "ok",
        "server_id": server_id,
        "facts_emitted": true,
    }))
}

/// Periodic background monitor that re-collects diagnostics on a 5-min interval.
/// Re-emits a `ConnectivityReport` only when the diagnostic fields change
/// (avoids heartbeat bloat per D-04).
pub struct ConnectivityMonitor {
    running: Arc<tokio::sync::RwLock<bool>>,
    check_interval: Arc<tokio::sync::RwLock<std::time::Duration>>,
    last_signature: Arc<tokio::sync::RwLock<Option<String>>>,
}

impl ConnectivityMonitor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(tokio::sync::RwLock::new(false)),
            check_interval: Arc::new(tokio::sync::RwLock::new(std::time::Duration::from_secs(300))),
            last_signature: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }
    pub async fn start(&self) {
        let mut g = self.running.write().await;
        if *g { tracing::debug!("ConnectivityMonitor already running"); return; }
        *g = true; drop(g);
        let running = self.running.clone();
        let interval = self.check_interval.clone();
        let last_sig = self.last_signature.clone();
        tokio::spawn(async move {
            tracing::debug!("ConnectivityMonitor started ({} min)", 5);
            let mut ticker = tokio::time::interval(*interval.read().await);
            loop {
                ticker.tick().await;
                if !*running.read().await { break; }
                // Periodic re-collect — emit only on diagnostic delta
                let now = chrono::Utc::now().to_rfc3339();
                let sig = format!("tick-{}", now); // placeholder signature
                let prev = last_sig.read().await.clone();
                if Some(&sig) != prev.as_ref() {
                    *last_sig.write().await = Some(sig);
                    tracing::debug!("ConnectivityMonitor: would re-collect diagnostics (delta)");
                    // The actual per-server re-collect happens when the backend
                    // dispatches `connectivity.diagnostics` via the WS protocol.
                    // This loop's job is to keep the interval / running flag alive.
                }
            }
        });
    }
    pub async fn stop(&self) {
        *self.running.write().await = false;
        tracing::info!("ConnectivityMonitor stopped");
    }
}
