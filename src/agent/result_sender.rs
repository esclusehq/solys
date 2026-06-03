//! ResultSender - buffers task results during disconnect, with disk persistence

#![allow(dead_code)]

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::fs;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};

use agent_proto::TaskResult;
use uuid::Uuid;

const BUFFER_FILE_NAME: &str = "task_results.buffer";
const MAX_BUFFER_SIZE: usize = 1000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentToBackend {
    TaskResult(TaskResult),
    Heartbeat {
        node_id: Uuid,
        status: String,
        task_count: usize,
    },
    Register {
        id: Option<Uuid>,
        name: String,
        ip: String,
        capabilities: Vec<String>,
    },
    ContainerEvent {
        node_id: Uuid,
        event: String,
        container_id: String,
        container_name: String,
    },
    TaskProgress {
        task_id: Uuid,
        status: String,
        progress: f32,
        message: String,
    },
    LogOutput {
        server_id: Uuid,
        timestamp: String,
        line: String,
        stream: String,
    },
}

#[derive(Clone)]
pub struct ResultSender {
    ws_sender: Arc<Mutex<mpsc::Sender<AgentToBackend>>>,
    buffer: Arc<Mutex<VecDeque<TaskResult>>>,
    connected: Arc<AtomicBool>,
    buffer_path: PathBuf,
}

impl ResultSender {
    pub fn new(ws_sender: mpsc::Sender<AgentToBackend>, data_dir: PathBuf) -> Self {
        let buffer_path = data_dir.join(BUFFER_FILE_NAME);
        
        Self {
            ws_sender: Arc::new(Mutex::new(ws_sender)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(AtomicBool::new(false)),
            buffer_path,
        }
    }

    /// Update the sender when reconnecting
    pub fn update_sender(&self, sender: mpsc::Sender<AgentToBackend>) {
        // Use try_lock for non-async context
        if let Ok(mut guard) = self.ws_sender.try_lock() {
            *guard = sender;
            info!("WebSocket sender updated");
        }
    }

    /// Get sender for sending messages
    fn get_sender_sync(&self) -> Option<mpsc::Sender<AgentToBackend>> {
        self.ws_sender.try_lock().ok().map(|guard| guard.clone())
    }

    pub fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::Relaxed);
        info!(connected, "Connection state changed");
    }

    pub async fn send(&self, result: TaskResult) {
        // Try to send immediately if connected
        if self.connected.load(Ordering::Relaxed) {
            if let Some(sender) = self.get_sender_sync() {
                match sender.try_send(AgentToBackend::TaskResult(result.clone())) {
                    Ok(_) => {
                        info!(task_id = %result.task_id, "Task result sent immediately");
                        return;
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!("WebSocket channel full, buffering result");
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        warn!("WebSocket channel closed, buffering result");
                    }
                }
            }
        }

        // Buffer the result
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
        
        // Try to send immediately if connected
        if self.connected.load(Ordering::Relaxed) {
            if let Some(sender) = self.get_sender_sync() {
                match sender.try_send(progress_msg) {
                    Ok(_) => {
                        info!(task_id = %task_id, progress, "Progress sent immediately");
                        return;
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!("WebSocket channel full, dropping progress");
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        warn!("WebSocket channel closed, dropping progress");
                    }
                }
            }
        }
        // Progress updates are not buffered - they are fire-and-forget
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
        
        // Try to send immediately if connected
        if self.connected.load(Ordering::Relaxed) {
            if let Some(sender) = self.get_sender_sync() {
                match sender.try_send(log_msg) {
                    Ok(_) => {
                        info!(server_id = %server_id, "Log output sent immediately");
                        return;
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!("WebSocket channel full, dropping log output");
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        warn!("WebSocket channel closed, dropping log output");
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

        if let Some(sender) = self.get_sender_sync() {
            while let Some(result) = buffer.pop_front() {
                match sender.try_send(AgentToBackend::TaskResult(result.clone())) {
                    Ok(_) => {
                        flushed += 1;
                    }
                    Err(_) => {
                        // Re-add to failed and stop flushing
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
