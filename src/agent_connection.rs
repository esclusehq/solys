//! Web Agent - WebSocket connection handler
//!
//! This module handles the WebSocket connection to the backend
//! and processes incoming commands.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::client::IntoClientRequest,
    tungstenite::Message,
};
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::ClientRequestBuilder;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::handlers::direct_executor::{download_jar, McLoader};

use zeroize::Zeroizing;

use crate::handlers;
use crate::handlers::metrics;
use crate::handlers::dns::{self, CloudflareDnsConfig};
use crate::agent::result_sender::{OutboundMessage, ResultSender};
use crate::task_state;
use agent_config::AgentConfig;
use agent_proto::messages::{AgentToBackend, BackendToAgent, RegisterPayload, HeartbeatPayload, NodeMetrics, CrashReportPayload, AgentStatus};
use agent_runtime::RuntimeDetector;
use agent_capability::CapabilityRegistry;
use sysinfo::System;

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
        format!("{}/api/ws/node", ws_url.trim_end_matches('/'))
    } else {
        ws_url.trim_end_matches('/').to_string()
    }
}

fn build_ws_request(uri: Uri, api_key: &str) -> Result<tokio_tungstenite::tungstenite::handshake::client::Request> {
    let auth_value = format!("Bearer {}", api_key);
    let builder = ClientRequestBuilder::new(uri)
        .with_header("Authorization", auth_value);
    builder
        .into_client_request()
        .map_err(|e| anyhow::anyhow!("failed to build WS request: {}", e))
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
        debug!(
            url = %ws_url,
            attempt = reconnect_attempt,
            delay_secs = initial_delay.as_secs(),
            "Connecting to backend (reconnect loop)"
        );

        // Wrap connect_async_tls_with_config in a timeout so a hung TCP/WS handshake doesn't block forever
        let uri: Uri = ws_url.parse().context("Failed to parse WebSocket URL")?;
        let api_key = config.api_key.expose_secret().to_string();
        let request = build_ws_request(uri, &api_key)?;
        // Zeroize the key after building the request — no longer needed
        drop(api_key);

        let connect_result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            connect_async_tls_with_config(request, None, true, None),
        )
        .await;

        match connect_result {
            Ok(Ok((ws_stream, _))) => {
                info!(attempt = reconnect_attempt, "WebSocket connected");

                let (ws_sender, mut ws_receiver) = ws_stream.split();

                // Collect system info for node registration
                let mut sys = System::new_all();
                sys.refresh_all();

                let ip = match crate::handlers::dns_watch::detect_public_ip().await {
                    Ok(ip) => ip,
                    Err(_) => "127.0.0.1".to_string(),
                };

                // Outbound channel: result_sender, task_state, and the inner loop all
                // push `AgentToBackend` messages here. A single writer task drains the
                // channel and serialises/writes them to the WebSocket. This is the
                // fix for "WebSocket channel full, buffering result" — previously no
                // task consumed the channel at all.
                let (ws_tx, ws_rx) = mpsc::channel::<OutboundMessage>(1000);
                result_sender.update_sender(Some(ws_tx.clone())).await;
                crate::task_state::set_progress_sender(ws_tx.clone());

                // Writer task: owns the WebSocket sink and the channel receiver.
                // Exits when the channel is closed (old sender clones dropped) or
                // when a WS write fails — either way, the inner loop notices via
                // the read side and triggers a reconnect.
                let writer_handle = tokio::spawn(async move {
                    let mut rx = ws_rx;
                    let mut sink = ws_sender;
                    while let Some(msg) = rx.recv().await {
                        match msg {
                            OutboundMessage::Pong(payload) => {
                                if let Err(e) = sink.send(Message::Pong(payload)).await {
                                    error!(error = %e, "Writer: WebSocket send failed (Pong), exiting writer task");
                                    return;
                                }
                            }
                            OutboundMessage::Proto(proto_msg) => {
                                match serde_json::to_string(&proto_msg) {
                                    Ok(text) => {
                                        if let Err(e) = sink.send(Message::Text(text.into())).await {
                                            error!(error = %e, "Writer: WebSocket send failed (Text), exiting writer task");
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        error!(error = %e, "Writer: serialise failed, skipping");
                                    }
                                }
                            }
                            OutboundMessage::Raw(text) => {
                                if let Err(e) = sink.send(Message::Text(text.into())).await {
                                    error!(error = %e, "Writer: WebSocket send failed (Raw), exiting writer task");
                                    return;
                                }
                            }
                        }
                    }
                    trace!("Writer exiting: channel closed");
                });

                // Build the Register message; now send it via the channel so it
                // gets serialised + written by the same writer task as everything else.
                let mut register_payload = RegisterPayload::new(agent_name.clone(), capabilities_list.clone())
                    .with_system_info(sys.total_memory() as u64, sys.cpus().len() as u32)
                    .with_node_info(
                        config.agent_id.unwrap_or_else(uuid::Uuid::new_v4),
                        ip.clone(),
                        std::env::consts::OS.to_string(),
                        runtime.version.clone(),
                    );
                // Set container_runtime via the runtime field
                register_payload.runtime = Some(runtime.runtime_name().to_string());
                let register = OutboundMessage::Proto(
                    AgentToBackend::Register(register_payload)
                );
                ws_tx.send(register).await?;

                info!("Waiting for register ack...");
                
                let ack_result = ws_receiver.next().await;
                match ack_result {
                    Some(Ok(Message::Text(text))) => {
                        info!("Received message (register_ack)");
                        
                        match serde_json::from_str::<BackendToAgent>(&text) {
                            Ok(backend_msg) => {
                                match backend_msg {
                                    BackendToAgent::RegisterAck(ref ack) => {
                                        info!(agent_id = ?ack.agent_id, "Agent registered successfully");
                                        if let Some(id) = ack.agent_id {
                                            *node_id.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                                            task_state::set_agent_node_id(id);
                                            crate::audit::log_agent_registered(id).await;
                                        }
                                    }
                                    BackendToAgent::BackendError(ref err) => {
                                        error!(code = %err.code, message = %err.message, "Registration error");
                                    }
                                    _ => {
                                        info!("Other message type received during registration: {:?}", std::mem::discriminant(&backend_msg));
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

                let current_node_id = *node_id.lock().unwrap_or_else(|e| e.into_inner());
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
                let (crash_tx, mut crash_rx) = mpsc::unbounded_channel::<CrashReportPayload>();
                if let Some(docker_client) = runtime.docker() {
                    let docker_clone = docker_client.clone();
                    let crash_tx_clone = crash_tx.clone();
                    tokio::spawn(async move {
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
                            let _ = ws_tx.send(OutboundMessage::Proto(
                                AgentToBackend::CrashReport(crash_msg)
                            )).await;
                        }
                        _ = heartbeat_interval.tick() => {
                            let node_id_value = *node_id.lock().unwrap_or_else(|e| e.into_inner());
                            if let Some(id) = node_id_value {
                                let metrics = match metrics::collect_full_metrics().await {
                                    Ok(report) => {
                                        let total_disk: u64 = report.system.disk_usage.iter().map(|d| d.total_bytes).sum();
                                        let used_disk: u64 = report.system.disk_usage.iter().map(|d| d.used_bytes).sum();
                                        Some(NodeMetrics {
                                            cpu_usage: report.system.cpu_percent,
                                            memory_used: report.system.memory_used_bytes,
                                            memory_total: report.system.memory_total_bytes,
                                            disk_used: used_disk,
                                            disk_total: total_disk,
                                        })
                                    }
                                    Err(e) => {
                                        warn!(error = %e, "Metrics collection failed, sending heartbeat without metrics");
                                        None
                                    }
                                };

                                // Collect direct executor server statuses (Phase 10)
                                let mut containers: Vec<agent_proto::messages::ContainerStatus> = Vec::new();
                                for (server_id, name, status) in crate::handlers::direct_executor::collect_server_statuses() {
                                    containers.push(agent_proto::messages::ContainerStatus {
                                        id: server_id.to_string(),
                                        name,
                                        status,
                                        cpu: 0.0,
                                        memory: 0,
                                        memory_limit: None,
                                        disk_usage: None,
                                        players: None,
                                        tps: None,
                                    });
                                }

                                let heartbeat = OutboundMessage::Proto(
                                    AgentToBackend::Heartbeat(HeartbeatPayload {
                                        agent_id: id,
                                        status: "online".to_string(),
                                        metrics,
                                        containers,
                                    })
                                );
                                match tokio::time::timeout(
                                    std::time::Duration::from_secs(5),
                                    ws_tx.send(heartbeat),
                                ).await {
                                    Ok(Ok(_)) => {
                                        info!("Heartbeat sent");
                                    }
                                    Ok(Err(_closed)) => {
                                        error!("Heartbeat channel closed, WS writer likely exited");
                                        break;
                                    }
                                    Err(_elapsed) => {
                                        error!("Heartbeat channel send timed out, writer likely wedged; breaking inner loop");
                                        break;
                                    }
                                }
                            }
                        }
                        Some(msg_result) = ws_receiver.next() => {
                            match msg_result {
                                Ok(Message::Ping(p)) => {
                                    let _ = ws_tx.send(OutboundMessage::Pong(p)).await;
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

                                    if let Ok(backend_msg) = serde_json::from_str::<BackendToAgent>(&text_str) {
                                        match backend_msg {
                                            BackendToAgent::TaskAssign(task) => {
                                                info!(task_id = %task.id, task_type = %task.task_type, "Executing task");
                                                let result = handlers::execute_task(task, &runtime, &capabilities).await;
                                                result_sender.send(result).await;
                                            }
                                            BackendToAgent::TaskCancel(cancel) => {
                                                info!(task_id = %cancel.task_id, reason = ?cancel.reason, "Task cancelled");
                                                crate::task_state::TASK_STATE_TRACKER.update(cancel.task_id, |s| s.cancel()).await;
                                            }
                                            BackendToAgent::ConfigUpdate(value) => {
                                                info!("ConfigUpdate received: {:?}", value);
                                            }
                                            BackendToAgent::Ping => {
                                                trace!("Ping received (ignored — heartbeat is on 30s autonomous timer)");
                                            }
                                            BackendToAgent::DnsConfig(dns_cfg) => {
                                                let config = CloudflareDnsConfig {
                                                    api_token: Zeroizing::new(dns_cfg.api_token),
                                                    zone_id: dns_cfg.zone_id,
                                                    zone_name: dns_cfg.zone_name,
                                                    wildcard_domain: dns_cfg.wildcard_domain,
                                                    auto_refresh: dns_cfg.auto_refresh,
                                                    refresh_interval_secs: dns_cfg.refresh_interval_secs,
                                                    subdomain: dns_cfg.subdomain,
                                                    extra_subdomains: Vec::new(),
                                                };
                                                let mut guard = dns::DNS_CONFIG.write().await;
                                                *guard = Some(config);
                                                drop(guard);
                                                info!("DNS configuration updated from backend");
                                            }
                                            BackendToAgent::RelayConfigSync(relay_cfg) => {
                                                info!(
                                                    "RelayConfigSync received: token={}, gateway={}, {} servers",
                                                    redact(&relay_cfg.relay_token),
                                                    relay_cfg.gateway_url,
                                                    relay_cfg.servers.len(),
                                                );

                                                let agent_public_ip =
                                                    crate::handlers::dns_watch::detect_public_ip().await
                                                        .unwrap_or_else(|_| "0.0.0.0".to_string());

                                                let configs: Vec<crate::state::RelayServerConfig> = relay_cfg.servers
                                                    .iter()
                                                    .map(|s| crate::state::RelayServerConfig {
                                                        server_id: s.server_id,
                                                        subdomain: s.subdomain.clone(),
                                                        public_port: s.public_port,
                                                        local_mc_addr: s.local_mc_addr.clone(),
                                                        gateway_url: relay_cfg.gateway_url.clone(),
                                                        token: relay_cfg.relay_token.clone(),
                                                        region: relay_cfg.region.clone(),
                                                        agent_public_ip: agent_public_ip.clone(),
                                                        loader: s.loader.clone(),
                                                    })
                                                    .collect();

                                                let relay_subs: Vec<String> = relay_cfg.servers.iter().map(|s| s.subdomain.clone()).collect();
                                                if !relay_subs.is_empty() {
                                                    let mut dns_guard = crate::handlers::dns::DNS_CONFIG.write().await;
                                                    if let Some(ref mut cfg) = *dns_guard {
                                                        cfg.extra_subdomains.retain(|sub| !relay_subs.contains(sub));
                                                    }
                                                    drop(dns_guard);
                                                }

                                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                                crate::state::relay_manager().set_servers(configs).await;
                                            }
                                            BackendToAgent::BackendError(err) => {
                                                error!(code = %err.code, message = %err.message, "Backend error received");
                                            }

                                            BackendToAgent::RegisterAck(_) => {
                                                info!("Unexpected RegisterAck in main dispatch loop");
                                            }
                                        }
                                    } else {
                                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text_str) {
                                            let msg_type = val["type"].as_str().unwrap_or("");
                                            if msg_type == "execute_command" {
                                                info!("Executing command via raw JSON fallback");
                                                let cmd = val["command"].as_str().unwrap_or("").to_string();
                                                let request_id = val["request_id"].as_str().and_then(|s| uuid::Uuid::parse_str(s).ok()).unwrap_or_else(uuid::Uuid::nil);
                                                let server_id = val["server_id"].as_str().and_then(|s| uuid::Uuid::parse_str(s).ok()).unwrap_or_else(uuid::Uuid::nil);
                                                let container_name = val["params"]["container_name"].as_str().unwrap_or("").to_string();

                                                let start = std::time::Instant::now();
                                                let (success, output) = match cmd.as_str() {
                                                    "start" | "stop" | "restart" => {
                                                        // Try container runtime via CLI (podman/docker)
                                                        let container = if container_name.is_empty() {
                                                            format!("mc-{}", server_id)
                                                        } else {
                                                            container_name.clone()
                                                        };

                                                        let action = match cmd.as_str() {
                                                            "start" => "start",
                                                            "stop" => "stop",
                                                            "restart" => "restart",
                                                            _ => unreachable!(),
                                                        };

                                                        // Check which CLIs are available
                                                        // Uses `command -v` (POSIX) instead of `which` (not always installed)
                                                        let check_cmd =
                                                            |name: &'static str| async move {
                                                                tokio::process::Command::new("sh")
                                                                    .args(["-c", &format!("command -v {}", name)])
                                                                    .output().await
                                                                    .map(|o| o.status.success())
                                                                    .unwrap_or(false)
                                                            };
                                                        let has_podman = check_cmd("podman").await;
                                                        let has_docker = check_cmd("docker").await;
                                                        let has_java = check_cmd("java").await;

                                                        if has_podman {
                                                            let r = tokio::process::Command::new("podman")
                                                                .arg(action).arg(&container).output().await;
                                                            match r {
                                                                Ok(out) => {
                                                                    let s = String::from_utf8_lossy(&out.stdout).to_string();
                                                                    let e = String::from_utf8_lossy(&out.stderr).to_string();
                                                                    let combined = if e.is_empty() { s } else { format!("{}\n{}", s, e) };
                                                                    (out.status.success(), combined)
                                                                }
                                                                Err(e) => (false, format!("podman failed: {}", e)),
                                                            }
                                                        } else if has_docker {
                                                            let r = tokio::process::Command::new("docker")
                                                                .arg(action).arg(&container).output().await;
                                                            match r {
                                                                Ok(out) => {
                                                                    let s = String::from_utf8_lossy(&out.stdout).to_string();
                                                                    let e = String::from_utf8_lossy(&out.stderr).to_string();
                                                                    let combined = if e.is_empty() { s } else { format!("{}\n{}", s, e) };
                                                                    (out.status.success(), combined)
                                                                }
                                                                Err(e) => (false, format!("docker failed: {}", e)),
                                                            }
                                                        } else if has_java {
                                                            let server_dir = format!("{}/servers/{}", config.data_dir.display(), server_id);
                                                            match action {
                                                                "start" => {
                                                                    let _ = tokio::process::Command::new("mkdir")
                                                                        .args(["-p", &server_dir]).output().await;
                                                                    let jar_path = format!("{}/server.jar", server_dir);
                                                                    let jar = Path::new(&jar_path);
                                                                    if !jar.exists() {
                                                                        info!("server.jar not found, auto-downloading Paper 1.21.4...");
                                                                        if let Err(e) = download_jar(
                                                                            &McLoader::Paper,
                                                                            "1.21.4",
                                                                            jar,
                                                                            Path::new(&server_dir),
                                                                        ).await {
                                                                            error!("Failed to download server.jar: {}", e);
                                                                        }
                                                                    }
                                                                    if jar.exists() {
                                                                        let r = tokio::process::Command::new("java")
                                                                            .arg("-Xmx1024M").arg("-Xms1024M")
                                                                            .arg("-jar").arg(jar_path)
                                                                            .arg("--nogui")
                                                                            .current_dir(&server_dir)
                                                                            .spawn();
                                                                        match r {
                                                                            Ok(_) => (true, format!("Java server started in {}", server_dir)),
                                                                            Err(e) => (false, format!("java failed: {}", e)),
                                                                        }
                                                                    } else {
                                                                        (false, format!("Cannot start: server.jar not found and download failed"))
                                                                    }
                                                                }
                                                                "stop" => {
                                                                    // Try pkill by server_id UUID, then by server.jar, then try docker on the off chance
                                                                    let result = tokio::process::Command::new("sh")
                                                                        .args(["-c", &format!(
                                                                            "pkill -f 'java.*{}' 2>/dev/null; pkill -f '{}' 2>/dev/null; docker stop {} 2>/dev/null",
                                                                            server_id, container, container
                                                                        )])
                                                                        .output().await;
                                                                    let out = String::from_utf8_lossy(&result.as_ref().map(|o| &o.stdout[..]).unwrap_or(&[])).to_string();
                                                                    let err = String::from_utf8_lossy(&result.as_ref().map(|o| &o.stderr[..]).unwrap_or(&[])).to_string();
                                                                    let success = result.map(|o| o.status.success()).unwrap_or(false);
                                                                    if success {
                                                                        (true, format!("Server stopped ({}{})", out, err))
                                                                    } else {
                                                                        (false, format!("Could not stop server: no Java process ({}) and Docker not available. Delete this server and create a new one with Java type.", server_id))
                                                                    }
                                                                }
                                                                "restart" => {
                                                                    let _ = tokio::process::Command::new("sh")
                                                                        .args(["-c", &format!("pkill -f 'java.*{}' 2>/dev/null; pkill -f '{}' 2>/dev/null", server_id, container)])
                                                                        .output().await;
                                                                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                                                    let _ = tokio::process::Command::new("mkdir")
                                                                        .args(["-p", &server_dir]).output().await;
                                                                    let jar_path = format!("{}/server.jar", server_dir);
                                                                    let jar = Path::new(&jar_path);
                                                                    if !jar.exists() {
                                                                        info!("server.jar not found, auto-downloading Paper 1.21.4...");
                                                                        let _ = download_jar(
                                                                            &McLoader::Paper,
                                                                            "1.21.4",
                                                                            jar,
                                                                            Path::new(&server_dir),
                                                                        ).await;
                                                                    }
                                                                    if jar.exists() {
                                                                        let r = tokio::process::Command::new("java")
                                                                            .arg("-Xmx1024M").arg("-Xms1024M")
                                                                            .arg("-jar").arg(jar_path)
                                                                            .arg("--nogui")
                                                                            .current_dir(&server_dir)
                                                                            .spawn();
                                                                        match r {
                                                                            Ok(_) => (true, format!("Java server restarted in {}", server_dir)),
                                                                            Err(e) => (false, format!("java restart failed: {}", e)),
                                                                        }
                                                                    } else {
                                                                        (false, "Cannot restart: server.jar not found and download failed".into())
                                                                    }
                                                                }
                                                                _ => (false, format!("Unknown action: {}", action)),
                                                            }
                                                        } else {
                                                            let mut hints = Vec::new();
                                                            if !has_podman && !has_docker {
                                                                hints.push("No container runtime (podman/docker)");
                                                            }
                                                            if !has_java {
                                                                hints.push("No Java runtime");
                                                            }
                                                            (false, format!("Cannot {} server: {}", action, hints.join("; ")))
                                                        }
                                                    }
                                                    _ if cmd == "logs" => {
                                                        // Handle logs for DirectExecutor: read latest.log
                                                        let server_log = format!("{}/servers/{}/logs/latest.log", config.data_dir.display(), server_id);
                                                        match tokio::fs::read_to_string(&server_log).await {
                                                            Ok(content) => {
                                                                if content.is_empty() {
                                                                    (true, "Server is running but no logs yet.".into())
                                                                } else {
                                                                    (true, content)
                                                                }
                                                            }
                                                            Err(_) => {
                                                                (true, "Server logs not available yet.".into())
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        // Generic shell fallback for unknown commands
                                                        let r = tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await;
                                                        match r {
                                                            Ok(out) => {
                                                                let s = String::from_utf8_lossy(&out.stdout).to_string();
                                                                let e = String::from_utf8_lossy(&out.stderr).to_string();
                                                                (out.status.success(), if e.is_empty() { s } else { format!("{}\n{}", s, e) })
                                                            }
                                                            Err(e) => (false, format!("Failed: {}", e)),
                                                        }
                                                    }
                                                };
                                                let duration_ms = start.elapsed().as_millis() as u64;
                                                let response = serde_json::json!({
                                                    "type": "command_response",
                                                    "request_id": request_id,
                                                    "command": cmd,
                                                    "server_id": server_id,
                                                    "success": success,
                                                    "output": output,
                                                    "duration_ms": duration_ms,
                                                });
                                                let _ = ws_tx.send(OutboundMessage::Raw(response.to_string())).await;
                                                continue;
                                            }
                                            warn!("Failed to parse BackendToAgent — type: {}", msg_type);
                                        } else {
                                            warn!("Failed to parse BackendToAgent — not valid json");
                                        }
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
        
        tokio::time::sleep(initial_delay).await;
        initial_delay = std::time::Duration::from_secs_f64(
            initial_delay.as_secs_f64() * multiplier
        ).min(max_delay);
    }
    
    // Return node_id (nil if shutdown)
    let result = node_id.lock().unwrap_or_else(|e| e.into_inner()).unwrap_or(Uuid::nil());
    Ok(result)
}

