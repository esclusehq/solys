//! ResultSender - buffers task results during disconnect, with disk persistence

#![allow(dead_code)]

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use bytes::Bytes;
use tokio::fs;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};

use agent_proto::TaskResult;
use uuid::Uuid;

const BUFFER_FILE_NAME: &str = "task_results.buffer";
const MAX_BUFFER_SIZE: usize = 1000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AgentToBackend {
    #[serde(rename = "task_result")]
    TaskResult(TaskResult),
    #[serde(rename = "register")]
    Register {
        id: Option<Uuid>,
        name: String,
        ip: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        podman_version: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        container_runtime: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        os_info: Option<String>,
        capabilities: Vec<String>,
        #[serde(default)]
        containers: Vec<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        total_memory: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cpu_cores: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent_version: Option<String>,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        node_id: Uuid,
        status: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        metrics: Option<serde_json::Value>,
        #[serde(default)]
        containers: Vec<serde_json::Value>,
    },
    #[serde(rename = "command_response")]
    CommandResponse {
        request_id: Uuid,
        command: String,
        server_id: Uuid,
        success: bool,
        output: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
    #[serde(rename = "task_progress")]
    TaskProgress {
        task_id: Uuid,
        status: String,
        progress: f32,
        message: String,
    },
    #[serde(rename = "log_output")]
    LogOutput {
        server_id: Uuid,
        timestamp: String,
        line: String,
        stream: String,
    },
    #[serde(rename = "crash_report")]
    CrashReport {
        server_id: Uuid,
        exit_code: i32,
        log_excerpt: String,
        timestamp: String,
    },
    #[serde(rename = "container_event")]
    ContainerEvent {
        node_id: Uuid,
        event: String,
        container_id: String,
        container_name: String,
    },
    /// WebSocket control frame — handled by the writer task as Message::Pong, not JSON
    #[serde(skip)]
    Pong(Bytes),
}

#[derive(Clone)]
pub struct ResultSender {
    ws_sender: Arc<Mutex<Option<mpsc::Sender<AgentToBackend>>>>,
    buffer: Arc<Mutex<VecDeque<TaskResult>>>,
    connected: Arc<AtomicBool>,
    buffer_path: PathBuf,
}

impl ResultSender {
    pub fn new(ws_sender: Option<mpsc::Sender<AgentToBackend>>, data_dir: PathBuf) -> Self {
        let buffer_path = data_dir.join(BUFFER_FILE_NAME);

        Self {
            ws_sender: Arc::new(Mutex::new(ws_sender)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(AtomicBool::new(false)),
            buffer_path,
        }
    }

    /// Update the sender when reconnecting. Pass `None` to mark the channel as closed
    /// (used when the writer task has exited due to a dead WebSocket).
    pub async fn update_sender(&self, sender: Option<mpsc::Sender<AgentToBackend>>) {
        let mut guard = self.ws_sender.lock().await;
        *guard = sender;
        info!("WebSocket sender updated");
    }

    /// Get a clone of the current sender, if one is registered.
    async fn get_sender(&self) -> Option<mpsc::Sender<AgentToBackend>> {
        self.ws_sender.lock().await.as_ref().cloned()
    }

    pub fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::Relaxed);
        info!(connected, "Connection state changed");
    }

    pub async fn send(&self, result: TaskResult) {
        // Try to send immediately if connected. Use async `send` so the caller
        // applies backpressure to the channel (and waits for the writer task
        // to drain it) instead of silently buffering every result to disk.
        if self.connected.load(Ordering::Relaxed) {
            if let Some(sender) = self.get_sender().await {
                match sender.send(AgentToBackend::TaskResult(result.clone())).await {
                    Ok(_) => {
                        info!(task_id = %result.task_id, "Task result sent immediately");
                        return;
                    }
                    Err(_) => {
                        warn!("WebSocket channel closed, buffering result");
                    }
                }
            }
        }

        // Buffer the result for replay on reconnect
        self.buffer_result(result).await;
    }

    /// Send progress update via WebSocket
    pub async fn send_progress(&self, task_id: Uuid, status: &str, progress: f32, message: &str) {
        let progress_msg = AgentToBackend::TaskProgress {
            task_id,
            status: status.to_string(),
            progress,
            message: message.to_string(),
        };

        // Fire-and-forget — `try_send` so a slow writer never stalls log capture
        if self.connected.load(Ordering::Relaxed) {
            if let Some(sender) = self.get_sender().await {
                match sender.try_send(progress_msg) {
                    Ok(_) => return,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!(task_id = %task_id, "WebSocket channel full, dropping progress");
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        warn!(task_id = %task_id, "WebSocket channel closed, dropping progress");
                    }
                }
            }
        }
        info!(task_id = %task_id, "Progress dropped (not connected)");
    }

    /// Send log output via WebSocket
    pub async fn send_log_output(&self, server_id: Uuid, timestamp: String, line: String, stream: String) {
        let log_msg = AgentToBackend::LogOutput {
            server_id,
            timestamp,
            line,
            stream,
        };

        // Fire-and-forget — `try_send` so a slow writer never stalls log capture
        if self.connected.load(Ordering::Relaxed) {
            if let Some(sender) = self.get_sender().await {
                match sender.try_send(log_msg) {
                    Ok(_) => return,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!(server_id = %server_id, "WebSocket channel full, dropping log output");
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        warn!(server_id = %server_id, "WebSocket channel closed, dropping log output");
                    }
                }
            }
        }
        info!(server_id = %server_id, "Log output dropped (not connected)");
    }

    async fn buffer_result(&self, result: TaskResult) {
        let mut buffer = self.buffer.lock().await;
        
        // Check buffer size limit
        if buffer.len() >= MAX_BUFFER_SIZE {
            warn!(
                buffer_size = buffer.len(),
                "Buffer full, dropping oldest result"
            );
            buffer.pop_front();
        }

        buffer.push_back(result.clone());
        
        // Persist to disk
        if let Err(e) = self.persist_buffer(&buffer).await {
            error!(error = %e, "Failed to persist buffer to disk");
        }

        info!(
            task_id = %result.task_id,
            buffer_size = buffer.len(),
            "Task result buffered"
        );
    }

    async fn persist_buffer(&self, buffer: &VecDeque<TaskResult>) -> Result<()> {
        let json = serde_json::to_string(buffer)?;
        fs::write(&self.buffer_path, json).await?;
        Ok(())
    }

    async fn load_buffer(&self) -> Result<VecDeque<TaskResult>> {
        if !self.buffer_path.exists() {
            return Ok(VecDeque::new());
        }

        let json = fs::read_to_string(&self.buffer_path).await?;
        let buffer: VecDeque<TaskResult> = serde_json::from_str(&json)
            .context("Failed to parse buffered results")?;
        
        info!(count = buffer.len(), "Loaded buffered results from disk");
        Ok(buffer)
    }

    /// Flush buffered results after reconnection
    pub async fn flush_buffer(&self) {
        let mut buffer = self.buffer.lock().await;

        if buffer.is_empty() {
            info!("No buffered results to flush");
            return;
        }

        let mut flushed = 0;
        let mut failed = VecDeque::new();

        if let Some(sender) = self.get_sender().await {
            // Use `send().await` so the writer task can apply backpressure to us too —
            // if the channel is full, wait for the writer to drain before pushing more.
            while let Some(result) = buffer.pop_front() {
                match sender.send(AgentToBackend::TaskResult(result.clone())).await {
                    Ok(_) => {
                        flushed += 1;
                    }
                    Err(_) => {
                        // Channel closed — re-buffer the result and stop
                        failed.push_back(result);
                        break;
                    }
                }
            }
        }

        // Put back failed results
        while let Some(result) = failed.pop_front() {
            buffer.push_front(result);
        }

        // Clear disk buffer if all flushed
        if buffer.is_empty() {
            if let Err(e) = fs::write(&self.buffer_path, "[]").await {
                warn!(error = %e, "Failed to clear disk buffer");
            }
        } else {
            // Persist remaining buffer
            if let Err(e) = self.persist_buffer(&buffer).await {
                error!(error = %e, "Failed to persist remaining buffer");
            }
        }

        info!(flushed, remaining = buffer.len(), "Flushed buffered results");
    }

    /// Initialize - load any existing buffered results from disk
    pub async fn init(&self) -> Result<()> {
        let loaded = self.load_buffer().await?;
        let mut buffer = self.buffer.lock().await;
        *buffer = loaded;
        Ok(())
    }

    /// Get current buffer size
    pub async fn buffer_size(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
}
