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
use crate::agent::result_sender::{AgentToBackend, ResultSender};
use crate::task_state;
use agent_config::AgentConfig;
use agent_proto::{TaskResult, TaskStatus};
use agent_runtime::RuntimeDetector;
use agent_capability::CapabilityRegistry;
use sysinfo::System;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub(crate) enum AgentMessage {
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

fn redact(s: &str) -> String {
    if s.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}****{}", &s[..4], &s[s.len()-4..])
    }
}

pub fn redact_json(s: &str) -> String {
    if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(s) {
        let sensitive_keys = ["api_token", "api_key", "relay_token"];
        if let Some(map) = v.as_object_mut() {
            for key in &sensitive_keys {
                if let Some(val) = map.get(*key) {
                    if let Some(s_val) = val.as_str() {
                        map.insert(key.to_string(), serde_json::Value::String(redact(s_val)));
                    }
                }
            }
        }
        v.to_string()
    } else {
        s.to_string()
    }
}

fn redact_url(url: &str) -> String {
    if let Some(pos) = url.find("?api_key=") {
        format!("{}{}", &url[..pos], "?api_key=****")
    } else if let Some(pos) = url.find("&api_key=") {
        format!("{}{}", &url[..pos], "&api_key=****")
    } else {
        url.to_string()
    }
}

impl BackendMessage {
    fn type_name(&self) -> &'static str {
        match self {
            BackendMessage::RegisterAck { .. } => "register_ack",
            BackendMessage::ExecuteCommand { .. } => "execute_command",
            BackendMessage::GetMetrics { .. } => "get_metrics",
            BackendMessage::Ping => "ping",
            BackendMessage::Error { .. } => "error",
            BackendMessage::DnsConfig { .. } => "dns_config",
            BackendMessage::RelayConfigSync { .. } => "relay_config_sync",
        }
    }
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
        /// Per-server subdomains to keep in sync alongside the global one
        /// (e.g. `["mantap-wou", "server-lain"]` — the watcher prepends
        /// `<sub>.<global_subdomain>.<wildcard_domain>`).
        #[serde(default)]
        extra_subdomains: Vec<String>,
    },
    // Phase 70: Backend pushes complete relay configuration after RegisterAck.
    // Matches NodeMessage::RelayConfigSync on the backend side.
    #[serde(rename = "relay_config_sync")]
    RelayConfigSync {
        relay_token: String,
        gateway_url: String,
        region: String,
        servers: Vec<ServerRelayInfo>,
    },
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ServerRelayInfo {
    server_id: Uuid,
    subdomain: String,
    local_mc_addr: String,
    public_port: u16,
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
    #[serde(default)]
    pub connection_key: Option<String>,
    #[serde(default)]
    pub local_path: Option<String>,
    #[serde(default)]
    pub remote_path: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub rcon_port: Option<u16>,
    #[serde(default)]
    pub rcon_password: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
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

fn prepare_ws_url(backend_url: &str, api_key: Option<&str>) -> String {
    let ws_url = if backend_url.starts_with("ws://") || backend_url.starts_with("wss://") {
        backend_url.trim_end_matches('/').to_string()
    } else if backend_url.starts_with("http://") {
        format!("ws://{}", backend_url.strip_prefix("http://").unwrap_or(backend_url))
    } else if backend_url.starts_with("https://") {
        format!("wss://{}", backend_url.strip_prefix("https://").unwrap_or(backend_url))
    } else {
        format!("ws://{}", backend_url)
    };
    
    let base = if !ws_url.contains("/api/ws/node") {
        format!("{}/api/ws/node", ws_url.trim_end_matches('/'))
    } else {
        ws_url.trim_end_matches('/').to_string()
    };

    if let Some(key) = api_key {
        if !key.is_empty() {
            return format!("{}?api_key={}", base, urlencode(key));
        }
    }
    base
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

pub async fn run(
    config: AgentConfig,
    runtime: RuntimeDetector,
    capabilities: CapabilityRegistry,
    shutdown: Arc<AtomicBool>,
) -> Result<Uuid> {
    let agent_name = config.agent_name.clone();
    let capabilities_list = capabilities.to_string_list();
    
    let api_key_str = config.api_key.expose_secret().to_string();
    let ws_url = prepare_ws_url(&config.backend_url, Some(&api_key_str));
    
    // Create data directory for buffering
    let data_dir = config.data_dir.clone();
    std::fs::create_dir_all(&data_dir)?;

    let mut initial_delay = std::time::Duration::from_secs(config.reconnect_initial_secs);
    let max_delay = std::time::Duration::from_secs(config.reconnect_max_secs);
    let multiplier = config.reconnect_multiplier;

    let node_id: std::sync::Mutex<Option<Uuid>> = std::sync::Mutex::new(None);

    // ResultSender starts with no sender; the inner reconnect loop installs a fresh
    // sender (and a writer task that drains it into the WebSocket) on every connect.
    let result_sender = ResultSender::new(None, data_dir.clone());
    result_sender.init().await?;

    // Check shutdown before connecting
    if shutdown.load(Ordering::Relaxed) {
        info!("Shutdown requested before initial connection");
        return Ok(Uuid::nil());
    }

    let mut reconnect_attempt: u32 = 0;

    loop {
        reconnect_attempt = reconnect_attempt.saturating_add(1);
        info!(
            url = %redact_url(&ws_url),
            attempt = reconnect_attempt,
            delay_secs = initial_delay.as_secs(),
            "Connecting to backend (reconnect loop)"
        );

        // Wrap connect_async in a timeout so a hung TCP/WS handshake doesn't block forever
        let connect_result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            connect_async(&ws_url),
        )
        .await;

        match connect_result {
            Ok(Ok((ws_stream, _))) => {
                info!(attempt = reconnect_attempt, "WebSocket connected");

                let (ws_sender, mut ws_receiver) = ws_stream.split();

                // Collect system info for node registration
                let mut sys = System::new_all();
                sys.refresh_all();
                let total_memory = Some(sys.total_memory() as i64);
                let cpu_cores = Some(sys.cpus().len() as i32);
                let agent_version = Some(env!("CARGO_PKG_VERSION").to_string());

                let ip = match crate::handlers::dns_watch::detect_public_ip().await {
                    Ok(ip) => ip,
                    Err(_) => "127.0.0.1".to_string(),
                };

                // Outbound channel: result_sender, task_state, and the inner loop all
                // push `AgentToBackend` messages here. A single writer task drains the
                // channel and serialises/writes them to the WebSocket. This is the
                // fix for "WebSocket channel full, buffering result" — previously no
                // task consumed the channel at all.
                let (ws_tx, ws_rx) = mpsc::channel::<crate::agent::result_sender::AgentToBackend>(1000);
                result_sender.update_sender(Some(ws_tx.clone())).await;
                crate::task_state::set_progress_sender(ws_tx.clone());

                // Writer task: owns the WebSocket sink and the channel receiver.
                // Exits when the channel is closed (old sender clones dropped) or
                // when a WS write fails — either way, the inner loop notices via
                // the read side and triggers a reconnect.
                let writer_handle = tokio::spawn(async move {
                    use crate::agent::result_sender::AgentToBackend;
                    let mut rx = ws_rx;
                    let mut sink = ws_sender;
                    while let Some(msg) = rx.recv().await {
                        match msg {
                            AgentToBackend::Pong(payload) => {
                                if let Err(e) = sink.send(Message::Pong(payload)).await {
                                    error!(error = %e, "Writer: Pong send failed, exiting");
                                    return;
                                }
                            }
                            other => {
                                match serde_json::to_string(&other) {
                                    Ok(text) => {
                                        if let Err(e) = sink.send(Message::Text(text.into())).await {
                                            error!(error = %e, "Writer: Text send failed, exiting");
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        error!(error = %e, "Writer: serialise failed, skipping");
                                    }
                                }
                            }
                        }
                    }
                    info!("Writer exiting: channel closed");
                });

                // Build the Register message; now send it via the channel so it
                // gets serialised + written by the same writer task as everything else.
                let register = AgentToBackend::Register {
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
                ws_tx.send(register).await?;

                info!("Waiting for register ack...");
                
                let ack_result = ws_receiver.next().await;
                match ack_result {
                    Some(Ok(Message::Text(text))) => {
                        info!("Received message (register_ack)");
                        
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

                        let mut events_stream = docker_clone.events::<&str>(None);

                        while let Some(Ok(event)) = events_stream.next().await {
                            if event.typ == Some(bollard::models::EventMessageTypeEnum::CONTAINER)
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
                                                crate::crash_reporter::capture_crash_data(&docker_clone, container_id).await
                                            {
                                                let report = crate::crash_reporter::build_crash_report(
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
                                                        crate::crash_reporter::capture_crash_data(&docker_clone, container_id).await
                                                    {
                                                        let report = crate::crash_reporter::build_crash_report(
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
                        info!("Shutdown requested, aborting writer to close connection...");
                        writer_handle.abort();
                        break;
                    }

                    tokio::select! {
                        // Phase 60: Process crash reports from Docker event listener
                        Some(crash_msg) = crash_rx.recv() => {
                            // crash_msg is an AgentMessage::CrashReport — route via the
                            // outbound channel so the writer task serialises + writes it.
                            if let AgentMessage::CrashReport { server_id, exit_code, log_excerpt, timestamp } = crash_msg {
                                let _ = ws_tx.send(AgentToBackend::CrashReport {
                                    server_id,
                                    exit_code,
                                    log_excerpt,
                                    timestamp,
                                }).await;
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
                                    
                                    let heartbeat = AgentToBackend::Heartbeat {
                                        node_id: id,
                                        status: "online".to_string(),
                                        metrics: Some(m),
                                        containers: c,
                                    };
                                    // `ws_tx.send` applies backpressure to the channel —
                                    // if the writer is stuck on a dead WS, the channel fills
                                    // and this `await` will block. Wrap with a short timeout
                                    // so a stuck writer doesn't stall heartbeats forever; if
                                    // it does stall, the writer is probably wedged and we
                                    // should break out and let the reconnect loop re-establish.
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(5),
                                        ws_tx.send(heartbeat),
                                    )
                                    .await
                                    {
                                        Ok(Ok(_)) => {}
                                        Ok(Err(_closed)) => {
                                            error!("Heartbeat channel closed, WS writer likely exited");
                                            break;
                                        }
                                        Err(_elapsed) => {
                                            error!(
                                                timeout_secs = 5,
                                                "Heartbeat channel send timed out, writer is likely wedged; \
                                                 breaking inner loop to trigger reconnect"
                                            );
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        Some(msg_result) = ws_receiver.next() => {
                            match msg_result {
                                Ok(Message::Ping(p)) => {
                                    let _ = ws_tx.send(AgentToBackend::Pong(p)).await;
                                }
                                Ok(Message::Close(close_frame)) => {
                                    warn!(
                                        frame = ?close_frame,
                                        "Received close frame from backend, ending inner loop"
                                    );
                                    break;
                                }
                                Ok(Message::Pong(_)) | Ok(Message::Binary(_)) | Ok(Message::Frame(_)) => {}
                                Ok(Message::Text(text)) => {
                                    let text_str = text.to_string();
                                    
                                    if let Ok(backend_msg) = serde_json::from_str::<BackendMessage>(&text_str) {
                                        info!("=== RECEIVED TEXT MESSAGE: {} ===", backend_msg.type_name());
                                        eprintln!("[DEBUG] Received message type: {}", backend_msg.type_name());
                                        match backend_msg {
                                            BackendMessage::ExecuteCommand { request_id, command, server_id, params, deploy_config } => {
                                                info!(request_id = %request_id, command = %command, "Executing task");
                                                
                                                let task_type = match command.as_str() {
                                                    "create" | "server.create" => "server.create",
                                                    "start" | "server.start" => "server.start",
                                                    "stop" | "server.stop" => "server.stop",
                                                    "restart" | "server.restart" => "server.restart",
                                                    "delete" | "server.delete" => "server.delete",
                                                    "logs" | "server.logs" => "server.logs",
                                                    "command" | "server.command" => "server.command",
                                                    "backup.start" => "backup.start",
                                                    "backup.restore" => "backup.restore",
                                                    "list_dir" | "file.list_dir" => "file.list_dir",
                                                    "read_file" | "file.read_file" => "file.read_file",
                                                    "write_file" | "file.write_file" => "file.write_file",
                                                    "delete_path" | "file.delete" => "file.delete",
                                                    "mkdir" | "file.mkdir" => "file.mkdir",
                                                    "rename_path" | "file.rename" => "file.rename",
                                                    "copy_path" | "file.copy" => "file.copy",
                                                    "sftp_upload" | "sftp.upload" => "sftp.upload",
                                                    "sftp_download" | "sftp.download" => "sftp.download",
                                                    _ => "unknown",
                                                };
                                                
                                                  let container_name = params.as_ref().and_then(|p| p.container_name.clone());
                                                   let container_id = params.as_ref().and_then(|p| p.container_id.clone());
                                                   let path = params.as_ref().and_then(|p| p.path.clone());
                                                   let content = params.as_ref().and_then(|p| p.content.clone());
                                                   let source_path = params.as_ref().and_then(|p| p.source_path.clone());
                                                   let dest_path = params.as_ref().and_then(|p| p.dest_path.clone());
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
                                                   if let Some(c) = content {
                                                       payload["content"] = serde_json::json!(c);
                                                   }
                                                   if let Some(s) = source_path {
                                                       payload["source_path"] = serde_json::json!(s);
                                                   }
                                                   if let Some(d) = dest_path {
                                                       payload["dest_path"] = serde_json::json!(d);
                                                   }
                                                   if let Some(f) = follow {
                                                       payload["follow"] = serde_json::json!(f);
                                                   }
                                                    if let Some(t) = tail {
                                                        payload["tail"] = serde_json::json!(t);
                                                    }
                                                    if let Some(ck) = params.as_ref().and_then(|p| p.connection_key.clone()) {
                                                        payload["connection_key"] = serde_json::json!(ck);
                                                    }
                                                    if let Some(lp) = params.as_ref().and_then(|p| p.local_path.clone()) {
                                                        payload["local_path"] = serde_json::json!(lp);
                                                    }
                                                     if let Some(rp) = params.as_ref().and_then(|p| p.remote_path.clone()) {
                                                         payload["remote_path"] = serde_json::json!(rp);
                                                     }
                                                     if let Some(c) = params.as_ref().and_then(|p| p.command.clone()) {
                                                         payload["command"] = serde_json::json!(c);
                                                     }
                                                     if let Some(rp) = params.as_ref().and_then(|p| p.rcon_port) {
                                                         payload["rcon_port"] = serde_json::json!(rp);
                                                     }
                                                     if let Some(pw) = params.as_ref().and_then(|p| p.rcon_password.clone()) {
                                                         payload["rcon_password"] = serde_json::json!(pw);
                                                     }
                                                     if let Some(nn) = params.as_ref().and_then(|p| p.new_name.clone()) {
                                                         payload["new_name"] = serde_json::json!(nn);
                                                     }
                                                     if let Some(bp) = params.as_ref().and_then(|p| p.backup_path.clone()) {
                                                         payload["backup_path"] = serde_json::json!(bp);
                                                     }
                                                     if let Some(h) = params.as_ref().and_then(|p| p.host.clone()) {
                                                         payload["host"] = serde_json::json!(h);
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
                                                
                                                let response = AgentToBackend::CommandResponse {
                                                    request_id,
                                                    command,
                                                    server_id,
                                                    success: result.status == TaskStatus::Completed,
                                                    output: match &result.output {
                                                        Some(v) => serde_json::to_string(v).unwrap_or_default(),
                                                        None => match &result.error {
                                                            Some(e) => format!("{}: {}", e.code, e.message),
                                                            None => "null".to_string(),
                                                        },
                                                    },
                                                    duration_ms: Some(duration_ms),
                                                };
                                                let _ = ws_tx.send(response).await;
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
                                                
                                                let response = AgentToBackend::CommandResponse {
                                                    request_id,
                                                    command: "get_metrics".to_string(),
                                                    server_id: Uuid::nil(),
                                                    success: result.status == TaskStatus::Completed,
                                                    output: match &result.output {
                                                        Some(v) => serde_json::to_string(v).unwrap_or_default(),
                                                        None => match &result.error {
                                                            Some(e) => format!("{}: {}", e.code, e.message),
                                                            None => "null".to_string(),
                                                        },
                                                    },
                                                    duration_ms: Some(duration_ms),
                                                };
                                                let _ = ws_tx.send(response).await;
                                            }
                                            BackendMessage::DnsConfig { api_token, zone_id, zone_name, wildcard_domain, auto_refresh, refresh_interval_secs, public_ip, subdomain, extra_subdomains } => {
                                                let per_server_count = extra_subdomains.len();
                                                let config = CloudflareDnsConfig {
                                                    api_token,
                                                    zone_id,
                                                    zone_name,
                                                    wildcard_domain,
                                                    auto_refresh,
                                                    refresh_interval_secs,
                                                    subdomain,
                                                    extra_subdomains,
                                                };
                                                let mut guard = dns::DNS_CONFIG.write().await;
                                                *guard = Some(config);
                                                drop(guard);
                                                info!("DNS configuration updated from backend ({} per-server subdomains)", per_server_count);
                                            }
                                            BackendMessage::RelayConfigSync { relay_token, gateway_url, region, servers } => {
                                                info!(
                                                    "RelayConfigSync received: token={}, gateway={}, {} servers",
                                                    redact(&relay_token),
                                                    gateway_url,
                                                    servers.len(),
                                                );

                                                // Build configs and apply via RelayManager
                                                let agent_public_ip =
                                                    crate::handlers::dns_watch::detect_public_ip().await
                                                        .unwrap_or_else(|_| "0.0.0.0".to_string());

                                                let configs: Vec<crate::state::RelayServerConfig> = servers
                                                    .iter()
                                                    .map(|s| crate::state::RelayServerConfig {
                                                        server_id: s.server_id,
                                                        subdomain: s.subdomain.clone(),
                                                        public_port: s.public_port,
                                                        local_mc_addr: s.local_mc_addr.clone(),
                                                        gateway_url: gateway_url.clone(),
                                                        token: relay_token.clone(),
                                                        region: region.clone(),
                                                        agent_public_ip: agent_public_ip.clone(),
                                                    })
                                                    .collect();

                                                crate::state::relay_manager().set_servers(configs).await;

                                                // Remove relay subdomains from DNS extra_subdomains so the
                                                // DnsWatcher doesn't create A records pointing to the agent's
                                                // local IP (which would override the wildcard
                                                // *.play.esluce.com → relay VPS).
                                                let relay_subs: Vec<String> = servers.iter().map(|s| s.subdomain.clone()).collect();
                                                if !relay_subs.is_empty() {
                                                    let mut dns_guard = crate::handlers::dns::DNS_CONFIG.write().await;
                                                    if let Some(ref mut cfg) = *dns_guard {
                                                        cfg.extra_subdomains.retain(|sub| !relay_subs.contains(sub));
                                                    }
                                                    drop(dns_guard);
                                                }
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
                                                        
                                                        let heartbeat = AgentToBackend::Heartbeat {
                                                            node_id: id,
                                                            status: "online".to_string(),
                                                            metrics: Some(m),
                                                            containers: c,
                                                        };
                                                        let _ = ws_tx.send(heartbeat).await;
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        let err = serde_json::from_str::<serde_json::Value>(&text_str)
                                            .map(|v| format!("raw json type field: {}", v["type"]))
                                            .unwrap_or_else(|_| "not valid json".into());
                                        eprintln!("[DEBUG] Failed to parse BackendMessage — {}", redact_json(&text_str));
                                        warn!("Failed to parse BackendMessage — {}", err);
                                    }
                                }
                                Err(e) => {
                                    error!(error = %e, "WebSocket error");
                                    break;
                                }
                            }
                        }
                        else => {
                            warn!("All select branches exhausted, breaking inner loop");
                            break;
                        }
                    }
                }

                warn!("Inner loop exited, will attempt reconnection");
            }
            Ok(Err(e)) => {
                error!(
                    error = %e,
                    attempt = reconnect_attempt,
                    "Failed to connect to backend (handshake error)"
                );
            }
            Err(_elapsed) => {
                error!(
                    attempt = reconnect_attempt,
                    timeout_secs = 15,
                    "Connect to backend timed out (handshake hung)"
                );
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

