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
