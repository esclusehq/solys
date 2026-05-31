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
use crate::handlers::metrics;
use crate::handlers::dns::{self, CloudflareDnsConfig};
use crate::agent::result_sender::ResultSender;
use crate::task_state;
use agent_config::AgentConfig;
use agent_proto::{TaskResult, TaskStatus};
use agent_runtime::RuntimeDetector;
use agent_capability::CapabilityRegistry;
use sysinfo::System;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
enum AgentMessage {
    #[serde(rename = "register")]
    Register {
        id: Option<Uuid>,
        name: String,
        ip: String,
        podman_version: Option<String>,
        container_runtime: Option<String>,
        os_info: Option<String>,
        capabilities: Vec<String>,
        containers: Vec<serde_json::Value>,
        #[serde(default)]
        total_memory: Option<i64>,
        #[serde(default)]
        cpu_cores: Option<i32>,
        #[serde(default)]
        agent_version: Option<String>,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        node_id: Uuid,
        status: String,
        #[serde(default)]
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
    // Phase 60: Crash Detection — Report container crash forensic data
    #[serde(rename = "crash_report")]
    CrashReport {
        server_id: Uuid,
        exit_code: i32,
        log_excerpt: String,
        timestamp: String,
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
    #[serde(rename = "dns_config")]
    DnsConfig {
        api_token: String,
        zone_id: String,
        zone_name: String,
        wildcard_domain: String,
        auto_refresh: bool,
        refresh_interval_secs: u64,
        #[serde(default)]
        public_ip: Option<String>,
        #[serde(default)]
        subdomain: Option<String>,
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
    #[serde(default)]
    pub follow: Option<bool>,
    #[serde(default)]
    pub tail: Option<u32>,
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

fn prepare_ws_url(backend_url: &str) -> String {
    let ws_url = if backend_url.starts_with("ws://") || backend_url.starts_with("wss://") {
        backend_url.trim_end_matches('/').to_string()
    } else if backend_url.starts_with("http://") {
        format!("ws://{}", backend_url.strip_prefix("http://").unwrap_or(backend_url))
    } else if backend_url.starts_with("https://") {
        format!("wss://{}", backend_url.strip_prefix("https://").unwrap_or(backend_url))
    } else {
        format!("ws://{}", backend_url)
    };
    
    if !ws_url.contains("/api/ws/node") {
        return format!("{}/api/ws/node", ws_url.trim_end_matches('/'));
    }
    ws_url.trim_end_matches('/').to_string()
}

pub async fn run(
    config: AgentConfig,
    runtime: RuntimeDetector,
    capabilities: CapabilityRegistry,
    shutdown: Arc<AtomicBool>,
) -> Result<Uuid> {
    let agent_name = config.agent_name.clone();
    let capabilities_list = capabilities.to_string_list();
    
    let ws_url = prepare_ws_url(&config.backend_url);
    
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

                // Collect system info for node registration
                let mut sys = System::new_all();
                sys.refresh_all();
                let total_memory = Some(sys.total_memory() as i64);
                let cpu_cores = Some(sys.cpus().len() as i32);
                let agent_version = Some(env!("CARGO_PKG_VERSION").to_string());

                let ip = "127.0.0.1".to_string();
                let register = AgentMessage::Register {
                    id: config.agent_id,
                    name: agent_name.clone(),
                    ip: ip.clone(),
                    podman_version: runtime.version.clone(),
                    container_runtime: Some(runtime.runtime_name().to_string()),
                    os_info: Some(std::env::consts::OS.to_string()),
                    capabilities: capabilities_list.clone(),
                    containers: vec![],
                    total_memory,
                    cpu_cores,
                    agent_version,
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

                // Create heartbeat interval
                let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_secs(30));

                // Mark as connected and flush buffered results
                result_sender.set_connected(true);
                result_sender.flush_buffer().await;

                // Phase 60: Crash Detection — Docker events listener for container crashes
                let (crash_tx, mut crash_rx) = mpsc::unbounded_channel::<AgentMessage>();
                if let Some(docker_client) = runtime.docker() {
                    let docker_clone = docker_client.clone();
                    let crash_tx_clone = crash_tx.clone();
                    tokio::spawn(async move {
                        use bollard::system::EventsOptions;
                        use futures_util::StreamExt;

                        let mut events_stream = docker_clone.system_events::<&str>(None);

                        while let Some(Ok(event)) = events_stream.next().await {
                            if event.typ.as_deref() == Some("container")
                                && event.action.as_deref() == Some("die")
                            {
                                if let Some(ref actor) = event.actor {
                                    if let Some(container_id) = &actor.id {
                                        // Skip if container_id is empty
                                        if container_id.is_empty() {
                                            continue;
                                        }

                                        // Try to find server_id from container labels
                                        // Managed containers have "server_id" label set on creation
                                        let server_id = actor.attributes.as_ref()
                                            .and_then(|attrs| attrs.get("server_id"))
                                            .and_then(|v| uuid::Uuid::parse_str(v).ok());

                                        if let Some(sid) = server_id {
                                            if let Ok((exit_code, log_excerpt)) =
                                                crash_reporter::capture_crash_data(&docker_clone, container_id).await
                                            {
                                                let report = crash_reporter::build_crash_report(
                                                    sid, exit_code, log_excerpt,
                                                );
                                                let _ = crash_tx_clone.send(report);
                                            }
                                        } else {
                                            // No server_id label — try inspecting container to find labels
                                            if let Ok(inspect) = docker_clone.inspect_container(container_id, None).await {
                                                let server_id = inspect.config.as_ref()
                                                    .and_then(|c| c.labels.as_ref())
                                                    .and_then(|labels| labels.get("server_id"))
                                                    .and_then(|v| uuid::Uuid::parse_str(v).ok());

                                                if let Some(sid) = server_id {
                                                    if let Ok((exit_code, log_excerpt)) =
                                                        crash_reporter::capture_crash_data(&docker_clone, container_id).await
                                                    {
                                                        let report = crash_reporter::build_crash_report(
                                                            sid, exit_code, log_excerpt,
                                                        );
                                                        let _ = crash_tx_clone.send(report);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }

                loop {
                    // Check shutdown signal
                    if shutdown.load(Ordering::Relaxed) {
                        info!("Shutdown requested, closing connection...");
                        let _ = ws_sender.close().await;
                        break;
                    }
                    
                    tokio::select! {
                        // Phase 60: Process crash reports from Docker event listener
                        Some(crash_msg) = crash_rx.recv() => {
                            if let Ok(msg) = serde_json::to_string(&crash_msg) {
                                let _ = ws_sender.send(Message::Text(msg.into())).await;
                            }
                        }
                        _ = heartbeat_interval.tick() => {
                            // Send heartbeat with metrics
                            let node_id_value = *node_id.lock().unwrap();
                            if let Some(id) = node_id_value {
                                if let Ok(metrics_report) = metrics::collect_full_metrics().await {
                                    let m = serde_json::json!({
                                        "cpu_usage": metrics_report.system.cpu_percent,
                                        "memory_used": metrics_report.system.memory_used_bytes,
                                        "memory_total": metrics_report.system.memory_total_bytes,
                                        "disk_used": metrics_report.system.disk_usage.first().map(|d| d.used_bytes).unwrap_or(0),
                                        "disk_total": metrics_report.system.disk_usage.first().map(|d| d.total_bytes).unwrap_or(0),
                                    });
                                    let c: Vec<serde_json::Value> = metrics_report.containers.iter().map(|cm| {
                                        serde_json::json!({
                                            "id": cm.container_id,
                                            "name": cm.container_name,
                                            "status": "running",
                                            "cpu": cm.cpu_percent,
                                            "memory": cm.memory_used_bytes,
                                            "memory_limit": cm.memory_limit_bytes,
                                            "disk_usage": cm.disk_usage_bytes,
                                            "players": cm.players,
                                            "tps": cm.tps,
                                        })
                                    }).collect();
                                    
                                    let heartbeat = AgentMessage::Heartbeat {
                                        node_id: id,
                                        status: "online".to_string(),
                                        metrics: Some(m),
                                        containers: c,
                                    };
                                    if let Ok(msg) = serde_json::to_string(&heartbeat) {
                                        let _ = ws_sender.send(Message::Text(msg.into())).await;
                                    }
                                }
                            }
                        }
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
                                                    "backup.start" => "backup.start",
                                                    "backup.restore" => "backup.restore",
                                                    _ => "unknown",
                                                };
                                                
                                                 let container_name = params.as_ref().and_then(|p| p.container_name.clone());
                                                  let container_id = params.as_ref().and_then(|p| p.container_id.clone());
                                                  let path = params.as_ref().and_then(|p| p.path.clone());
                                                  let follow = params.as_ref().and_then(|p| p.follow);
                                                  let tail = params.as_ref().and_then(|p| p.tail);
                                                  
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
                                                  if let Some(f) = follow {
                                                      payload["follow"] = serde_json::json!(f);
                                                  }
                                                  if let Some(t) = tail {
                                                      payload["tail"] = serde_json::json!(t);
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
                                            BackendMessage::DnsConfig { api_token, zone_id, zone_name, wildcard_domain, auto_refresh, refresh_interval_secs, public_ip, subdomain } => {
                                                let config = CloudflareDnsConfig {
                                                    api_token,
                                                    zone_id,
                                                    zone_name,
                                                    wildcard_domain,
                                                    auto_refresh,
                                                    refresh_interval_secs,
                                                    subdomain,
                                                };
                                                let mut guard = dns::DNS_CONFIG.write().await;
                                                *guard = Some(config);
                                                info!("DNS configuration updated from backend");
                                            }
                                            BackendMessage::Ping => {
                                                let node_id_value = *node_id.lock().unwrap();
                                                if let Some(id) = node_id_value {
                                                    // Collect and send heartbeat
                                                    if let Ok(metrics_report) = metrics::collect_full_metrics().await {
                                                        let m = serde_json::json!({
                                                            "cpu_usage": metrics_report.system.cpu_percent,
                                                            "memory_used": metrics_report.system.memory_used_bytes,
                                                            "memory_total": metrics_report.system.memory_total_bytes,
                                                            "disk_used": metrics_report.system.disk_usage.first().map(|d| d.used_bytes).unwrap_or(0),
                                                            "disk_total": metrics_report.system.disk_usage.first().map(|d| d.total_bytes).unwrap_or(0),
                                                        });
                                                        let c: Vec<serde_json::Value> = metrics_report.containers.iter().map(|cm| {
                                                            serde_json::json!({
                                                                "id": cm.container_id,
                                                                "name": cm.container_name,
                                                                "status": "running",
                                                                "cpu": cm.cpu_percent,
                                                                "memory": cm.memory_used_bytes,
                                                                "memory_limit": cm.memory_limit_bytes,
                                                                "disk_usage": cm.disk_usage_bytes,
                                                                "players": cm.players,
                                                                "tps": cm.tps,
                                                            })
                                                        }).collect();
                                                        
                                                        let heartbeat = AgentMessage::Heartbeat {
                                                            node_id: id,
                                                            status: "online".to_string(),
                                                            metrics: Some(m),
                                                            containers: c,
                                                        };
                                                        if let Ok(msg) = serde_json::to_string(&heartbeat) {
                                                            let _ = ws_sender.send(Message::Text(msg.into())).await;
                                                        }
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
        ).min(max_delay);
    }
    
    // Return node_id (nil if shutdown)
    let result = node_id.lock().unwrap().unwrap_or(Uuid::nil());
    Ok(result)
}

