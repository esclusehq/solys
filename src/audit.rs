//! Audit Logger Integration
//!
//! This module provides audit logging integration for web-agent.

use std::path::PathBuf;
use std::sync::Arc;

use agent_security::{AuditEntry, AuditEvent, AuditLogger, AuditResult};
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;


lazy_static::lazy_static! {
    pub static ref AUDIT_LOGGER: Arc<RwLock<Option<AuditLogger>>> = Arc::new(RwLock::new(None));
}

pub async fn init_audit_logger(data_dir: PathBuf) {
    let logger = AuditLogger::new(data_dir);
    let mut audit = AUDIT_LOGGER.write().await;
    *audit = Some(logger);
}

pub async fn log_task_received(task_id: Uuid, task_type: &str) {
    let guard = AUDIT_LOGGER.read().await;
    if let Some(logger) = guard.as_ref() {
        let _ = logger.log_task_received(Uuid::nil(), task_id, task_type);
    }
}

pub async fn log_task_completed(task_id: Uuid) {
    let guard = AUDIT_LOGGER.read().await;
    if let Some(logger) = guard.as_ref() {
        let _ = logger.log_task_completed(Uuid::nil(), task_id);
    }
}

pub async fn log_task_failed(task_id: Uuid, error: &str) {
    let guard = AUDIT_LOGGER.read().await;
    if let Some(logger) = guard.as_ref() {
        let _ = logger.log_task_failed(Uuid::nil(), task_id, error);
    }
}

pub async fn log_agent_registered(agent_id: Uuid) {
    let guard = AUDIT_LOGGER.read().await;
    if let Some(logger) = guard.as_ref() {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event: AuditEvent::AgentRegistered,
            task_id: None,
            task_type: None,
            result: Some(AuditResult::Success),
            agent_id,
        };
        let _ = logger.log(entry);
    }
}

// ---------------------------------------------------------------------------
// Phase 67: Connectivity-specific audit log
// ---------------------------------------------------------------------------

/// Phase 67: log the exact shell command for a connectivity auto-fix action.
/// The `connectivity_audit_log` row in the backend is the primary source; this
/// is the agent's local mirror (D-17).
pub async fn log_connectivity_command(server_id: &str, action: &str, command: &str) {
    let line = format!("[CONNECTIVITY_AUDIT] server={} action={} command={} @{}",
        server_id, action, command, Utc::now().to_rfc3339());
    tracing::info!("{}", line);

    // Append to the local audit file (audit_data_dir/connectivity-audit.log).
    if let Ok(mut f) = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(audit_data_dir().join("connectivity-audit.log")).await {
        use tokio::io::AsyncWriteExt;
        let _ = f.write_all(format!("{}\n", line).as_bytes()).await;
    }
}

/// Resolve the local audit data directory (the directory the
/// `init_audit_logger` was given). Falls back to the state directory or
/// `.` when nothing is configured.
fn audit_data_dir() -> std::path::PathBuf {
    crate::state::audit_data_dir()
}

// ---------------------------------------------------------------------------
// Phase 68: Relay tunnel audit log
// ---------------------------------------------------------------------------

/// Phase 68: emit a relay-tunnel lifecycle event (up / down / heartbeat /
/// rekey threshold / stale). The agent's local `relay-tunnel-audit.log` is the
/// mirror of the backend's `connectivity_audit_log` row (D-17) — used as
/// evidence when a re-handshake race or token-rotation regression needs to
/// be reconstructed.
///
/// `event_type` is one of: `"connecting"`, `"connected"`, `"disconnected"`,
/// `"stale"`, `"heartbeat"`, `"rekey"`, `"auth_failed"`.
pub async fn log_relay_tunnel_event(
    node_id: Uuid,
    server_id: Uuid,
    event_type: &str,
    detail: &str,
) {
    let line = format!(
        "[RELAY_TUNNEL_AUDIT] node={} server={} event=relay.tunnel.{} detail={} @{}",
        node_id, server_id, event_type, detail, Utc::now().to_rfc3339()
    );
    tracing::trace!("{}", line);

    // Append to the local audit file (audit_data_dir/relay-tunnel-audit.log).
    if let Ok(mut f) = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(audit_data_dir().join("relay-tunnel-audit.log"))
        .await
    {
        use tokio::io::AsyncWriteExt;
        let _ = f.write_all(format!("{}\n", line).as_bytes()).await;
    }

    // Mirror to the global AuditLogger so structured JSON ends up in the daily
    // audit-YYYY-MM-DD.log alongside the other Task* / Agent* events. We use
    // a synthetic TaskRejected entry — the AuditEvent enum has no
    // relay-specific variant and we deliberately do not extend the external
    // agent-security crate from this plan.
    let guard = AUDIT_LOGGER.read().await;
    if let Some(logger) = guard.as_ref() {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event: AuditEvent::TaskRejected {
                reason: format!("relay.tunnel.{}: {}", event_type, detail),
            },
            task_id: None,
            task_type: Some(format!("relay.tunnel.{}", event_type)),
            result: Some(AuditResult::Success),
            agent_id: node_id,
        };
        let _ = logger.log(entry);
    }
}

