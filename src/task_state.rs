//! Task State Tracker - Track task status and progress
//!
//! This module provides task state tracking for status API and progress monitoring.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::agent::result_sender::OutboundMessage;

use agent_proto::TaskStatus as ProtoTaskStatus;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl From<TaskStatus> for ProtoTaskStatus {
    fn from(s: TaskStatus) -> Self {
        match s {
            TaskStatus::Queued => ProtoTaskStatus::Pending,
            TaskStatus::Running => ProtoTaskStatus::Running,
            TaskStatus::Completed => ProtoTaskStatus::Completed,
            TaskStatus::Failed => ProtoTaskStatus::Failed,
            TaskStatus::Cancelled => ProtoTaskStatus::Cancelled,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskState {
    pub task_id: Uuid,
    pub task_type: String,
    pub status: TaskStatus,
    pub progress: Option<f32>,
    pub message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl TaskState {
    pub fn new(task_id: Uuid, task_type: String) -> Self {
        Self {
            task_id,
            task_type,
            status: TaskStatus::Queued,
            progress: None,
            message: None,
            started_at: None,
            completed_at: None,
            error: None,
        }
    }

    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn update_progress(&mut self, progress: f32, message: &str) {
        self.progress = Some(progress);
        self.message = Some(message.to_string());
    }

    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.progress = Some(100.0);
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self, error: &str) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error.to_string());
    }

    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

pub struct TaskStateTracker {
    states: Arc<RwLock<HashMap<Uuid, TaskState>>>,
    max_states: usize,
}

impl TaskStateTracker {
    pub fn new(max_states: usize) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            max_states,
        }
    }

    pub async fn add(&self, task_id: Uuid, task_type: String) {
        let mut states = self.states.write().await;
        
        // Clean up old states if needed
        if states.len() >= self.max_states {
            let to_remove: Vec<Uuid> = states.iter()
                .filter(|(_, s)| matches!(s.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled))
                .take(10)
                .map(|(k, _)| *k)
                .collect();
            for id in to_remove {
                states.remove(&id);
            }
        }
        
        states.insert(task_id, TaskState::new(task_id, task_type));
    }

    pub async fn get(&self, task_id: &Uuid) -> Option<TaskState> {
        let states = self.states.read().await;
        states.get(task_id).cloned()
    }

    pub async fn update(&self, task_id: Uuid, updater: impl FnOnce(&mut TaskState)) {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(&task_id) {
            updater(state);
        }
    }

    pub async fn list_running(&self) -> Vec<TaskState> {
        let states = self.states.read().await;
        states.values()
            .filter(|s| matches!(s.status, TaskStatus::Running | TaskStatus::Queued))
            .cloned()
            .collect()
    }

    pub async fn list_recent(&self, limit: usize) -> Vec<TaskState> {
        let states = self.states.read().await;
        let mut all: Vec<_> = states.values().cloned().collect();
        all.sort_by(|a, b| {
            let a_time = a.started_at.unwrap_or(a.completed_at.unwrap_or(chrono::DateTime::<Utc>::MIN_UTC));
            let b_time = b.started_at.unwrap_or(b.completed_at.unwrap_or(chrono::DateTime::<Utc>::MIN_UTC));
            b_time.cmp(&a_time)
        });
        all.into_iter().take(limit).collect()
    }
}

impl Default for TaskStateTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

pub fn set_agent_node_id(id: Uuid) {
    if let Ok(mut guard) = AGENT_NODE_ID.lock() {
        *guard = Some(id);
    }
}

pub fn get_agent_node_id() -> Option<Uuid> {
    AGENT_NODE_ID.lock().ok().and_then(|guard| *guard)
}

lazy_static::lazy_static! {
    pub static ref TASK_STATE_TRACKER: TaskStateTracker = TaskStateTracker::default();
    pub static ref AGENT_NODE_ID: std::sync::Mutex<Option<Uuid>> = std::sync::Mutex::new(None);
    pub static ref PROGRESS_SENDER: std::sync::Mutex<Option<mpsc::Sender<OutboundMessage>>> = std::sync::Mutex::new(None);
    pub static ref RESULT_SENDER: std::sync::Mutex<Option<std::sync::Arc<crate::agent::result_sender::ResultSender>>> = std::sync::Mutex::new(None);
}

pub fn set_progress_sender(sender: mpsc::Sender<OutboundMessage>) {
    if let Ok(mut guard) = PROGRESS_SENDER.lock() {
        *guard = Some(sender);
    }
}

pub fn set_result_sender(sender: std::sync::Arc<crate::agent::result_sender::ResultSender>) {
    if let Ok(mut guard) = RESULT_SENDER.lock() {
        *guard = Some(sender);
    }
}

pub fn get_result_sender() -> Option<std::sync::Arc<crate::agent::result_sender::ResultSender>> {
    RESULT_SENDER.lock().ok().and_then(|guard| guard.clone())
}

pub fn get_progress_sender() -> Option<mpsc::Sender<OutboundMessage>> {
    PROGRESS_SENDER.lock().ok().and_then(|guard| guard.clone())
}

/// Send log output via WebSocket - uses ResultSender if available
pub async fn send_log_output(server_id: Uuid, line: String, stream: String) {
    // Try using the global result sender first
    if let Some(sender) = get_progress_sender() {
        let msg = OutboundMessage::Proto(
            agent_proto::messages::AgentToBackend::LogLine(agent_proto::messages::LogLinePayload {
                agent_id: server_id,
                line,
                timestamp: chrono::Utc::now(),
                stream: match stream.as_str() {
                    "stderr" | "err" => agent_proto::messages::LogStream::Stderr,
                    _ => agent_proto::messages::LogStream::Stdout,
                },
            })
        );
        let _ = sender.try_send(msg);
        return;
    }

    // Fallback: try via result sender if available
    if let Some(rs) = get_result_sender() {
        rs.send_log_output(server_id, line, stream).await;
    }
}

/// Send progress update via WebSocket
pub async fn send_progress(task_id: Uuid, status: &str, progress: f32, message: &str) {
    if let Some(sender) = get_progress_sender() {
        let agent_id = get_agent_node_id().unwrap_or(uuid::Uuid::nil());
        let status_enum = match status {
            "running" => agent_proto::messages::AgentStatus::Busy,
            "completed" => agent_proto::messages::AgentStatus::Online,
            "failed" => agent_proto::messages::AgentStatus::Error,
            _ => agent_proto::messages::AgentStatus::Online,
        };
        let msg = OutboundMessage::Proto(
            agent_proto::messages::AgentToBackend::StatusUpdate(agent_proto::messages::AgentStatusPayload {
                agent_id,
                status: status_enum,
                task_id: Some(task_id),
                progress: Some(progress),
                message: Some(message.to_string()),
            })
        );
        let _ = sender.try_send(msg);
    }
}
