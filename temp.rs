//! Web Agent - WebSocket connection handler
//!
//! This module handles the WebSocket connection to the backend
//! and processes incoming commands.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::handlers;
use crate::agent::result_sender::ResultSender;
use crate::task_state;
use agent_config::AgentConfig;
use agent_proto::{TaskResult, TaskStatus};
use agent_runtime::RuntimeDetector;
use agent_capability::CapabilityRegistry;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
enum AgentMessage {
    #[serde(rename = "register")]
    Register {
        id: Option<Uuid>,
        name: String,
        ip: String,
        podman_version: Option<String>,
        os_info: Option<String>,
        capabilities: Vec<String>,
        containers: Vec<serde_json::Value>,
        #[serde(default)]
        total_memory: Option<i64>,
        #[serde(default)]
        cpu_cores: Option<i32>,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        node_id: Uuid,
        status: String,
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
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum BackendMessage {
    #[serde(rename = "register_ack")]
    RegisterAck {
        node_id: Uuid,
        #[serde(default)]
        _heartbeat_interval_secs: Option<u64>,
        #[serde(default)]
        _status: Option<String>,
        #[serde(default)]
        _message: Option<String>,
    },
    #[serde(rename = "execute_command")]
    ExecuteCommand {
        request_id: Uuid,
        command: String,
        server_id: Uuid,
        #[serde(default)]
        params: Option<CommandParams>,
        #[serde(default)]
        deploy_config: Option<DeployConfig>,
    },
    #[serde(rename = "get_metrics")]
    GetMetrics {
        request_id: Uuid,
        #[serde(default)]
        container_id: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct CommandParams {
    pub container_name: Option<String>,
    pub container_id: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub dest_path: Option<String>,
    #[serde(default)]
    pub new_name: Option<String>,
    #[serde(default)]
    pub backup_path: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct DeployConfig {
    pub image: String,
    #[serde(default)]
    pub game_port: Option<u16>,
    #[serde(default)]
    pub rcon_port: Option<u16>,
    #[serde(default)]
    pub ram_mb: Option<u32>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub volume_path: Option<String>,
    #[serde(default)]
    pub memory_limit: Option<u64>,
    #[serde(default)]
    pub cpu_limit: Option<u64>,
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            image: String::new(),
            game_port: None,
            rcon_port: None,
            ram_mb: None,
            version: None,
            loader: None,
            env: None,
            volume_path: None,
            memory_limit: None,
            cpu_limit: None,
        }
    }
}

pub async fn run(
    config: AgentConfig,
    runtime: RuntimeDetector,
    capabilities: CapabilityRegistry,
    shutdown: Arc<AtomicBool>,
) -> Result<Uuid> {
    let agent_name = config.agent_name.clone();
    let capabilities_list = capabilities.to_string_list();
    
    // Convert HTTP/WS URL to WebSocket URL
    let backend_url = config.backend_url.clone();
    let ws_url = if backend_url.starts_with("ws://") || backend_url.starts_with("wss://") {
        // Already WebSocket URL
        backend_url.trim_end_matches('/').to_string()
    } else if backend_url.starts_with("http://") {
        format!("ws://{}", backend_url.strip_prefix("http://").unwrap_or(&backend_url))
    } else if backend_url.starts_with("https://") {
        format!("wss://{}", backend_url.strip_prefix("https://").unwrap_or(&backend_url))
    } else {
        // Plain hostname, assume ws://
        format!("ws://{}", backend_url)
    };
    
    // Append /api/ws/node if not already present
    if !ws_url.contains("/api/ws/node") {
        return format!("{}/api/ws/node", ws_url.trim_end_matches('/'));
    }
    ws_url.trim_end_matches('/').to_string()
};
    
    // Create data directory for buffering
    let data_dir = config.data_dir.clone();
    std::fs::create_dir_all(&data_dir)?;

    let mut initial_delay = std::time::Duration::from_secs(config.reconnect_initial_secs);
    let max_delay = std::time::Duration::from_secs(config.reconnect_max_secs);
    let multiplier = config.reconnect_multiplier;

    let node_id: std::sync::Mutex<Option<Uuid>> = std::sync::Mutex::new(None);
    
    // ResultSender for buffering
    let (ws_tx, _ws_rx) = mpsc::channel(100);
    let result_sender = ResultSender::new(ws_tx.clone(), data_dir.clone());
    result_sender.init().await?;
    
    // Set progress sender for task state tracking
    crate::task_state::set_progress_sender(ws_tx);

    // Check shutdown before connecting
    if shutdown.load(Ordering::Relaxed) {
        info!("Shutdown requested before initial connection");
        return Ok(Uuid::nil());
    }

    loop {
        info!(url = %ws_url, "Connecting to backend");

        match connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                info!("WebSocket connected");

                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                let ip = "127.0.0.1".to_string();
                let register = AgentMessage::Register {
                    id: config.agent_id,
                    name: agent_name.clone(),
                    ip: ip.clone(),
                    podman_version: runtime.version.clone(),
                    os_info: Some(std::env::consts::OS.to_string()),
                    capabilities: capabilities_list.clone(),
                    containers: vec![],
                    total_memory: None,
                    cpu_cores: None,
                };
                ws_sender.send(Message::Text(serde_json::to_string(&register)?.into())).await?;

                info!("Waiting for register ack...");
                
                let ack_result = ws_receiver.next().await;
                match ack_result {
                    Some(Ok(Message::Text(text))) => {
                        info!(msg = %text, "Received message");
                        
                        match serde_json::from_str::<BackendMessage>(&text) {
                            Ok(backend_msg) => {
                                match backend_msg {
                                    BackendMessage::RegisterAck { node_id: id, .. } => {
                                        info!(node_id = %id, "Agent registered successfully");
                                        *node_id.lock().unwrap() = Some(id);
                                        // Also set in global state for API
                                        task_state::set_agent_node_id(id);
                                        // Log agent registration
                                        crate::audit::log_agent_registered(id).await;
                                    }
                                    BackendMessage::Error { code, message } => {
                                        error!(code = %code, message = %message, "Registration error");
                                    }
                                    _ => {
                                        info!("Other message type received");
                                    }
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to parse message");
                            }
                        }
                    }
                    Some(Ok(other)) => {
                        info!(msg = ?other, "Received non-text message");
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "Error receiving register ack");
                    }
                    None => {
                        info!("Connection closed before register ack");
                    }
                }

                let current_node_id = *node_id.lock().unwrap();
                if current_node_id.is_none() {
                    warn!("No node_id received, continuing anyway");
                }

                initial_delay = std::time::Duration::from_secs(config.reconnect_initial_secs);

                // Mark as connected and flush buffered results
                result_sender.set_connected(true);
                result_sender.flush_buffer().await;

                loop {
                    // Check shutdown signal
                    if shutdown.load(Ordering::Relaxed) {
                        info!("Shutdown requested, closing connection...");
                        let _ = ws_sender.close().await;
                        break;
                    }
                    
                    tokio::select! {
                        Some(msg_result) = ws_receiver.next() => {
                            match msg_result {
                                Ok(Message::Ping(p)) => {
                                    let _ = ws_sender.send(Message::Pong(p)).await;
                                }
                                Ok(Message::Close(_)) | Ok(Message::Pong(_)) | Ok(Message::Binary(_)) | Ok(Message::Frame(_)) => {}
                                Ok(Message::Text(text)) => {
                                    let text_str = text.to_string();
                                    info!(msg = %text_str, "=== RECEIVED TEXT MESSAGE ===");
                                    
                                    // Also print to stderr for debugging
                                    eprintln!("[DEBUG] Raw message received: {}", text_str);
                                    
                                    if let Ok(backend_msg) = serde_json::from_str::<BackendMessage>(&text_str) {
                                        eprintln!("[DEBUG] Parsed message type: {:?}", std::mem::discriminant(&backend_msg));
                                        match backend_msg {
                                            BackendMessage::ExecuteCommand { request_id, command, server_id, params, deploy_config } => {
                                                info!(request_id = %request_id, command = %command, "Executing task");
                                                
                                                let task_type = match command.as_str() {
                                                    "create" => "server.create",
                                                    "start" => "server.start",
                                                    "stop" => "server.stop",
                                                    "restart" => "server.restart",
                                                    "delete" => "server.delete",
                                                    "logs" => "server.logs",
                                                    "command" => "server.command",
                                                    _ => "unknown",
                                                };
                                                
                                                 let container_name = params.as_ref().and_then(|p| p.container_name.clone());
                                                 let container_id = params.as_ref().and_then(|p| p.container_id.clone());
                                                 let path = params.as_ref().and_then(|p| p.path.clone());
                                                 
                                                 let mut payload = serde_json::json!({
                                                     "server_id": server_id,
                                                 });
                                                 
                                                 if let Some(name) = container_name {
                                                     payload["container_name"] = serde_json::json!(name);
                                                 }
                                                 if let Some(id) = container_id {
                                                     payload["container_id"] = serde_json::json!(id);
                                                 }
                                                 if let Some(p) = path {
                                                     payload["path"] = serde_json::json!(p);
                                                 }
                                                
                                                if let Some(config) = deploy_config {
                                                    payload["image"] = serde_json::json!(config.image);
                                                    if let Some(port) = config.game_port {
                                                        payload["ports"] = serde_json::json!({ "25565": [port.to_string()] });
                                                        payload["container_port"] = serde_json::json!(port);
                                                    }
                                                    if let Some(env) = config.env {
                                                        payload["env"] = serde_json::json!(env);
                                                    }
                                                    if let Some(mem) = config.ram_mb {
                                                        payload["memory_limit"] = serde_json::json!(mem * 1024 * 1024);
                                                    }
                                                    if let Some(cpu) = config.cpu_limit {
                                                        payload["cpu_limit"] = serde_json::json!(cpu);
                                                    }
                                                }
                                                
                                                let task = agent_proto::Task::new(
                                                    task_type.to_string(),
                                                    payload,
                                                ).with_id(request_id);
                                                
                                                let result = handlers::execute_task(task, &runtime, &capabilities).await;
                                                
                                                let duration_ms = result.ended_at.signed_duration_since(result.started_at).num_milliseconds() as u64;
                                                
                                                // Send result via ResultSender for buffering
                                                let task_result = agent_proto::TaskResult {
                                                    task_id: request_id,
                                                    status: result.status.clone(),
                                                    output: result.output.clone(),
                                                    error: result.error.clone(),
                                                    started_at: result.started_at,
                                                    ended_at: result.ended_at,
                                                    retry_count: result.retry_count,
                                                };
                                                result_sender.send(task_result).await;
                                                
                                                let response = AgentMessage::CommandResponse {
                                                    request_id,
                                                    command,
                                                    server_id,
                                                    success: result.status == TaskStatus::Completed,
                                                    output: serde_json::to_string(&result.output).unwrap_or_default(),
                                                    duration_ms: Some(duration_ms),
                                                };
                                                if let Ok(msg) = serde_json::to_string(&response) {
                                                    let _ = ws_sender.send(Message::Text(msg.into())).await;
                                                }
                                            }
                                            BackendMessage::GetMetrics { request_id, container_id } => {
                                                info!(request_id = %request_id, "Getting metrics");
                                                
                                                let payload = serde_json::json!({
                                                    "container_id": container_id,
                                                });
                                                
                                                let task = agent_proto::Task::new(
                                                    "metrics.report".to_string(),
                                                    payload,
                                                ).with_id(request_id);
                                                
                                                let result = handlers::execute_task(task, &runtime, &capabilities).await;
                                                
                                                let duration_ms = result.ended_at.signed_duration_since(result.started_at).num_milliseconds() as u64;
                                                
                                                // Send result via ResultSender for buffering
                                                let task_result = TaskResult {
                                                    task_id: request_id,
                                                    status: result.status.clone(),
                                                    output: result.output.clone(),
                                                    error: result.error.clone(),
                                                    started_at: result.started_at,
                                                    ended_at: result.ended_at,
                                                    retry_count: result.retry_count,
                                                };
                                                result_sender.send(task_result).await;
                                                
                                                let response = AgentMessage::CommandResponse {
                                                    request_id,
                                                    command: "get_metrics".to_string(),
                                                    server_id: Uuid::nil(),
                                                    success: result.status == TaskStatus::Completed,
                                                    output: serde_json::to_string(&result.output).unwrap_or_default(),
                                                    duration_ms: Some(duration_ms),
                                                };
                                                if let Ok(msg) = serde_json::to_string(&response) {
                                                    let _ = ws_sender.send(Message::Text(msg.into())).await;
                                                }
                                            }
                                            BackendMessage::Ping => {
                                                if let Some(id) = *node_id.lock().unwrap() {
                                                    let heartbeat = AgentMessage::Heartbeat {
                                                        node_id: id,
                                                        status: "online".to_string(),
                                                    };
                                                    if let Ok(msg) = serde_json::to_string(&heartbeat) {
                                                        let _ = ws_sender.send(Message::Text(msg.into())).await;
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(error = %e, "WebSocket error");
                                    break;
                                }
                            }
                        }
                        else => break
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to connect to backend");
            }
        }

        // Check shutdown before retrying
        if shutdown.load(Ordering::Relaxed) {
            info!("Shutdown requested, exiting");
            break;
        }

        warn!("Reconnecting in {}s", initial_delay.as_secs());
        
        // Check shutdown before retrying
        if shutdown.load(Ordering::Relaxed) {
            info!("Shutdown requested, exiting reconnect loop");
            break;
        }
        
        tokio::time::sleep(initial_delay).await;
        initial_delay = std::time::Duration::from_secs_f64(
            initial_delay.as_secs_f64() * multiplier
