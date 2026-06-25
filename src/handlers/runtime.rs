//! Runtime handler - server.create/start/stop/restart/delete/logs

use std::collections::HashMap;
use std::time::Duration;

use agent_proto::Task;
use agent_runtime::RuntimeDetector;
use anyhow::{Context, Result};
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions, LogsOptions};
use bollard::image::CreateImageOptions;
use bollard::models::HostConfig;
use futures_util::StreamExt;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

#[derive(Debug, Deserialize)]
pub struct ServerCreatePayload {
    pub server_id: uuid::Uuid,
    pub image: String,
    pub name: String,
    pub env_vars: Option<HashMap<String, String>>,
    pub ports: Option<HashMap<String, Vec<String>>>,
    pub container_port: Option<u32>,
    pub memory_limit: Option<i64>,
    pub cpu_limit: Option<i64>,
    pub volume_path: Option<String>,
    pub version: Option<String>,
    pub loader: Option<String>,
}

async fn check_image_exists(docker: &bollard::Docker, image: &str) -> bool {
    let image_name = image.split(':').next().unwrap_or(image);
    let image_tag = image.split(':').nth(1).unwrap_or("latest");
    let full_image = format!("{}:{}", image_name, image_tag);
    
    match docker.list_images(Some(bollard::image::ListImagesOptions::<String> {
        all: false,
        ..Default::default()
    })).await {
        Ok(images) => images.iter().any(|img| {
            img.repo_tags.iter().any(|tag| {
                tag == &full_image || tag == image_name || tag == &format!("{}:latest", image_name)
            })
        }),
        Err(_) => false,
    }
}

async fn pull_image_with_timeout(docker: &bollard::Docker, image: &str, _timeout_secs: u64) -> Result<()> {
    info!(image = %image, "Pulling Docker image...");
    
    let options = CreateImageOptions {
        from_image: image,
        ..Default::default()
    };
    
    let mut stream = docker.create_image(Some(options), None, None);
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                if let Some(status) = info.status {
                    if status.contains("Pulling from") || status.contains("Pull complete") || status.contains("Downloading") || status.contains("Extracting") {
                        info!(status = %status, progress = ?info.progress, "Pull progress");
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Error during image pull");
                return Err(anyhow::anyhow!("Failed to pull image: {}", e));
            }
        }
    }
    
    info!(image = %image, "Image pull completed");
    Ok(())
}

async fn ensure_image_exists(docker: &bollard::Docker, image: &str, max_retries: u32, retry_delay_secs: u64) -> Result<()> {
    if check_image_exists(docker, image).await {
        info!(image = %image, "Image already exists, skipping pull");
        return Ok(());
    }
    
    info!(image = %image, "Image not found, starting pull...");
    
    for attempt in 1..=max_retries {
        match pull_image_with_timeout(docker, image, 300).await {
            Ok(_) => {
                info!(image = %image, attempt = attempt, "Image pulled successfully");
                return Ok(());
            }
            Err(e) if attempt < max_retries => {
                warn!(image = %image, attempt = attempt, error = %e, "Pull failed, retrying...");
                tokio::time::sleep(Duration::from_secs(retry_delay_secs)).await;
            }
            Err(e) => {
                error!(image = %image, error = %e, "Failed to pull image after {} attempts", max_retries);
                return Err(anyhow::anyhow!("Failed to pull image {} after {} attempts: {}", image, max_retries, e));
            }
        }
    }
    
    Err(anyhow::anyhow!("Failed to pull image after {} retries", max_retries))
}

pub async fn handle_create(task: Task, runtime: &RuntimeDetector) -> Result<serde_json::Value> {
    let docker = runtime.docker().context("Docker client not available")?;
    let payload: ServerCreatePayload = serde_json::from_value(task.payload)?;

    info!(server_id = %payload.server_id, image = %payload.image, name = %payload.name, "Creating container");

    let mut env_vec = vec![];
    if let Some(env) = &payload.env_vars {
        for (k, v) in env {
            env_vec.push(format!("{}={}", k, v));
        }
    }
    if let Some(version) = &payload.version {
        if !version.is_empty() {
            env_vec.push(format!("VERSION={}", version));
        }
    }

    let mut port_bindings = std::collections::HashMap::new();
    let mut exposed_ports = std::collections::HashMap::new();
    if let Some(ports) = &payload.ports {
        let is_bedrock = payload.loader
            .as_deref()
            .map(|l| l.eq_ignore_ascii_case("bedrock"))
            .unwrap_or(false);
        let protocol = if is_bedrock { "udp" } else { "tcp" };
        for (container_port, host_ports) in ports {
            let port_key = format!("{}/{}", container_port, protocol);
            let mut bindings = vec![];
            for host_port in host_ports {
                bindings.push(bollard::models::PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.clone()),
                });
            }
            port_bindings.insert(port_key.clone(), Some(bindings));
            exposed_ports.insert(port_key, std::collections::HashMap::new());
        }
    }

    let mut host_config = HostConfig {
        binds: None,
        port_bindings: if port_bindings.is_empty() { None } else { Some(port_bindings) },
        network_mode: Some("bridge".to_string()),
        dns: Some(vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()]),
        ..Default::default()
    };

    if let Some(memory) = payload.memory_limit {
        host_config.memory = Some(memory);
    }
    if let Some(cpu) = payload.cpu_limit {
        host_config.cpu_period = Some(100000);
        host_config.cpu_quota = Some(cpu * 100000);
    }

    let config = Config {
        image: Some(payload.image.clone()),
        env: if env_vec.is_empty() { None } else { Some(env_vec) },
        exposed_ports: if exposed_ports.is_empty() { None } else { Some(exposed_ports) },
        host_config: Some(host_config),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: payload.name.clone(),
        platform: None,
    };

    let response = docker.create_container(Some(options), config).await?;

    info!(container_id = %response.id, "Container created");

    Ok(serde_json::json!({
        "container_id": response.id,
        "name": payload.name,
    }))
}

pub async fn handle_start(task: Task, runtime: &RuntimeDetector) -> Result<serde_json::Value> {
    let docker = runtime.docker().context("Docker client not available")?;
    let payload: serde_json::Value = task.payload;
    
    // Check if we have container_id (start existing) or container_name+config (create+start)
    let container_id = payload.get("container_id")
        .and_then(|v| v.as_str());
    
    if let Some(id) = container_id {
        // Check if container is already running (idempotent)
        if let Ok(info) = docker.inspect_container(id, None).await {
            if let Some(state) = info.state {
                if state.running == Some(true) {
                    info!(container_id = %id, "Container already running");
                    return Ok(serde_json::json!({ "status": "already_running", "container_id": id }));
                }
            }
        }
        // Start the container
        info!(container_id = %id, "Starting container");
        docker.start_container(id, None::<StartContainerOptions<String>>).await?;
        return Ok(serde_json::json!({ "status": "started", "container_id": id }));
    }
    
    // No container_id - use container_name
    let container_name: String = if let Some(name) = payload.get("container_name").and_then(|v| v.as_str()) {
        name.to_string()
    } else if let Some(server_id) = payload.get("server_id").and_then(|v| v.as_str()) {
        // Fallback: construct from server_id
        format!("mc-{}", server_id)
    } else {
        return Err(anyhow::anyhow!("Missing container_name for container creation"));
    };
    
    info!(container_name = %container_name, "Looking up container by name");
    
    // Check if container already exists
    use tokio::process::Command;
    let output = Command::new("docker")
        .args(["ps", "-a", "--filter", &format!("name=^{}$", container_name), "--format", "{{.ID}}"])
        .output()
        .await
        .context("Failed to run docker ps")?;
    
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if !id.is_empty() {
        // Container exists, check if running
        if let Ok(info) = docker.inspect_container(&id, None).await {
            if let Some(state) = info.state {
                if state.running == Some(true) {
                    info!(container_id = %id, "Container already running");
                    return Ok(serde_json::json!({ "status": "already_running", "container_id": id, "container_name": container_name }));
                }
            }
        }
        // Not running, start it
        info!(container_id = %id, "Starting existing container...");
        docker.start_container(&id, None::<StartContainerOptions<String>>).await?;
        return Ok(serde_json::json!({ "status": "started", "container_id": id, "container_name": container_name }));
    }
    
    // Container doesn't exist - create and start
    info!(container_name = %container_name, "Container not found, creating and starting...");
    
    // Parse deploy config from payload
    let image = payload.get("image")
        .and_then(|v| v.as_str())
        .unwrap_or("alpine:latest");
    
    // Auto-pull image if not exists
    if let Err(e) = ensure_image_exists(&docker, image, 2, 5).await {
        error!(image = %image, error = %e, "Failed to ensure image exists");
        return Err(anyhow::anyhow!("Failed to prepare image {}: {}", image, e));
    }
    
    // Get environment variables from deploy_config
    let mut env_vars = std::collections::HashMap::new();
    if let Some(env_obj) = payload.get("env_vars") {
        if let Some(obj) = env_obj.as_object() {
            for (k, v) in obj {
                if let Some(val) = v.as_str() {
                    env_vars.insert(k.clone(), val.to_string());
                }
            }
        }
    }
    if let Some(version) = payload.get("version").and_then(|v| v.as_str()) {
        if !version.is_empty() {
            env_vars.insert("VERSION".to_string(), version.to_string());
        }
    }
    if !env_vars.contains_key("TYPE") {
        if let Some(loader) = payload.get("loader").and_then(|v| v.as_str()) {
            let upper = loader.to_uppercase();
            if upper != "VANILLA" && upper != "BEDROCK" && !upper.is_empty() {
                env_vars.insert("TYPE".to_string(), upper);
            }
        }
    }
    
    let container_port = payload.get("container_port")
        .and_then(|v| v.as_i64())
        .unwrap_or(25565) as u32;
    
    let memory = payload.get("memory_limit")
        .and_then(|v| v.as_i64());
    
    let mut port_bindings = std::collections::HashMap::new();
    let mut exposed_ports = std::collections::HashMap::new();
    
    let is_bedrock = payload.get("loader")
        .and_then(|v| v.as_str())
        .map(|l| l.eq_ignore_ascii_case("bedrock"))
        .unwrap_or(false);
    let protocol = if is_bedrock { "udp" } else { "tcp" };
    let port_key = format!("{}/{}", container_port, protocol);
    exposed_ports.insert(port_key.clone(), std::collections::HashMap::new());
    port_bindings.insert(port_key, Some(vec![bollard::models::PortBinding {
        host_ip: Some("0.0.0.0".to_string()),
        host_port: Some(container_port.to_string()),
    }]));
    
    let mut host_config = HostConfig {
        port_bindings: if port_bindings.is_empty() { None } else { Some(port_bindings) },
        network_mode: Some("bridge".to_string()),
        dns: Some(vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()]),
        ..Default::default()
    };
    if let Some(mem) = memory {
        host_config.memory = Some(mem);
    }
    
    let mut env_vec: Vec<String> = env_vars.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    
    let has_eula = env_vec.iter().any(|e| e.starts_with("EULA="));
    if !has_eula {
        env_vec.push("EULA=TRUE".to_string());
    }
    
    let config = Config {
        image: Some(image.to_string()),
        env: if env_vec.is_empty() { None } else { Some(env_vec) },
        exposed_ports: if exposed_ports.is_empty() { None } else { Some(exposed_ports) },
        host_config: Some(host_config),
        ..Default::default()
    };
    
    let options = CreateContainerOptions {
        name: container_name.to_string(),
        platform: None,
    };
    
    let response = docker.create_container(Some(options), config).await?;
    info!(container_id = %response.id, "Container created, starting...");
    
    docker.start_container(&response.id, None::<StartContainerOptions<String>>).await?;
    
    Ok(serde_json::json!({ 
        "status": "created_and_started", 
        "container_id": response.id,
        "container_name": container_name
    }))
}

pub async fn handle_stop(task: Task, runtime: &RuntimeDetector) -> Result<serde_json::Value> {
    let docker = runtime.docker().context("Docker client not available")?;
    let payload: serde_json::Value = task.payload;
    
    // Try container_id first, then fallback to container_name
    let container_id = payload.get("container_id")
        .and_then(|v| v.as_str());
    
    let container_id = if let Some(id) = container_id {
        id.to_string()
    } else {
        // Find container by name using docker CLI
        let container_name = payload.get("container_name")
            .and_then(|v| v.as_str())
            .context("Missing container_id and container_name")?;
        
        info!(container_name = %container_name, "Looking up container by name");
        
        use tokio::process::Command;
        let output = Command::new("docker")
            .args(["ps", "-a", "--filter", &format!("name=^{}$", container_name), "--format", "{{.ID}}"])
            .output()
            .await
            .context("Failed to run docker ps")?;
        
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if id.is_empty() {
            return Ok(serde_json::json!({ "status": "not_found", "message": format!("Container {} not found", container_name) }));
        }
        id
    };

    // Check if container is already stopped (idempotent)
    if let Ok(info) = docker.inspect_container(&container_id, None).await {
        if let Some(state) = info.state {
            if state.running != Some(true) {
                info!(container_id = %container_id, "Container already stopped");
                return Ok(serde_json::json!({ "status": "already_stopped", "container_id": container_id }));
            }
        }
    }

    info!(container_id = %container_id, "Stopping container");

    docker.stop_container(&container_id, Some(StopContainerOptions { t: 10 })).await?;

    Ok(serde_json::json!({ "status": "stopped", "container_id": container_id }))
}

pub async fn handle_restart(task: Task, runtime: &RuntimeDetector) -> Result<serde_json::Value> {
    let docker = runtime.docker().context("Docker client not available")?;
    let payload: serde_json::Value = task.payload;
    
    let container_id = payload.get("container_id")
        .and_then(|v| v.as_str());
    
    let container_id = if let Some(id) = container_id {
        id.to_string()
    } else {
        let container_name = payload.get("container_name")
            .and_then(|v| v.as_str())
            .context("Missing container_id and container_name")?;
        
        info!(container_name = %container_name, "Looking up container by name");
        
        use tokio::process::Command;
        let output = Command::new("docker")
            .args(["ps", "-a", "--filter", &format!("name=^{}$", container_name), "--format", "{{.ID}}"])
            .output()
            .await
            .context("Failed to run docker ps")?;
        
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if id.is_empty() {
            return Ok(serde_json::json!({ "status": "not_found", "message": format!("Container {} not found", container_name) }));
        }
        id
    };

    // Check if container is running (idempotent)
    if let Ok(info) = docker.inspect_container(&container_id, None).await {
        if let Some(state) = info.state {
            if state.running != Some(true) {
                info!(container_id = %container_id, "Container not running, starting first");
                docker.start_container(&container_id, None::<StartContainerOptions<String>>).await?;
                return Ok(serde_json::json!({ "status": "started", "container_id": container_id }));
            }
        }
    }

    info!(container_id = %container_id, "Restarting container");

    docker.restart_container(&container_id, Some(bollard::container::RestartContainerOptions { t: 10 })).await?;

    Ok(serde_json::json!({ "status": "restarted", "container_id": container_id }))
}

pub async fn handle_delete(task: Task, runtime: &RuntimeDetector) -> Result<serde_json::Value> {
    let docker = runtime.docker().context("Docker client not available")?;
    let payload: serde_json::Value = task.payload;
    
    let container_id = payload.get("container_id")
        .and_then(|v| v.as_str());
    
    let container_id = if let Some(id) = container_id {
        id.to_string()
    } else {
        let container_name = payload.get("container_name")
            .and_then(|v| v.as_str())
            .context("Missing container_id and container_name")?;
        
        info!(container_name = %container_name, "Looking up container by name");
        
        use tokio::process::Command;
        let output = Command::new("docker")
            .args(["ps", "-a", "--filter", &format!("name=^{}$", container_name), "--format", "{{.ID}}"])
            .output()
            .await
            .context("Failed to run docker ps")?;
        
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if id.is_empty() {
            // Container doesn't exist - idempotent delete
            return Ok(serde_json::json!({ "status": "not_found", "message": format!("Container {} not found, nothing to delete", container_name) }));
        }
        id
    };

    // Check if container exists before delete (idempotent)
    match docker.inspect_container(&container_id, None).await {
        Ok(_) => {
            // Container exists, delete it
            info!(container_id = %container_id, "Deleting container");
            docker.remove_container(&container_id, Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            })).await?;
            Ok(serde_json::json!({ "status": "deleted", "container_id": container_id }))
        }
        Err(_) => {
            // Container doesn't exist - idempotent
            Ok(serde_json::json!({ "status": "not_found", "message": format!("Container {} not found, nothing to delete", container_id) }))
        }
    }
}

pub async fn handle_logs(task: Task, _runtime: &RuntimeDetector) -> Result<serde_json::Value> {
    // Always create a fresh Docker client to avoid stale connections
    let docker = bollard::Docker::connect_with_local_defaults()
        .context("Failed to connect to Docker. Is Docker running?")?;
    
    let payload: serde_json::Value = task.payload;
    
    let container_id = if let Some(id) = payload.get("container_id").and_then(|v| v.as_str()) {
        id.to_string()
    } else if let Some(name) = payload.get("container_name").and_then(|v| v.as_str()) {
        name.to_string()
    } else {
        return Err(anyhow::anyhow!("Missing container_id or container_name"));
    };

    let follow = payload.get("follow").and_then(|v| v.as_bool()).unwrap_or(false);
    let tail = payload.get("tail").and_then(|v| v.as_u64()).unwrap_or(200) as u64;
    let server_id = payload.get("server_id").and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .unwrap_or_else(|| uuid::Uuid::nil());

    debug!(container_id = %container_id, follow = follow, tail = tail, "Getting container logs");

    if follow {
        info!("Starting live log streaming for server {}", server_id);
        
        let docker_clone = docker.clone();
        let container_clone = container_id.clone();
        let server_id_clone = server_id;
        
        // Spawn background task for log streaming with polling fallback
        tokio::spawn(async move {
            // First, try docker.logs() with follow=true for initial live stream
            let options = LogsOptions::<String> {
                stdout: true,
                stderr: true,
                follow: true,
                tail: tail.to_string(),
                ..Default::default()
            };
            
            use crate::task_state::send_log_output;
            
            let mut logs_stream = docker_clone.logs(&container_clone, Some(options));
            let mut last_log_time = std::time::Instant::now();
            let mut consecutive_empty = 0;
            let mut _last_line_hash: Option<String> = None;
            
            loop {
                tokio::select! {
                    // Try to read from the follow stream
                    log_result = logs_stream.next() => {
                        match log_result {
                            Some(Ok(log)) => {
                                let line = log.to_string();
                                if !line.is_empty() {
                                    let stream = if line.starts_with("ERR") || line.to_lowercase().contains("error") {
                                        "stderr"
                                    } else {
                                        "stdout"
                                    };
                                    send_log_output(server_id_clone, line.clone(), stream.to_string()).await;
                                    last_log_time = std::time::Instant::now();
                                    consecutive_empty = 0;
                                    
                                    // Track hash for duplicate prevention on stream restart
                                    use std::collections::hash_map::DefaultHasher;
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = DefaultHasher::new();
                                    line.hash(&mut hasher);
                                    _last_line_hash = Some(format!("{:x}", hasher.finish()));
                                }
                            }
                            Some(Err(e)) => {
                                error!(error = %e, "Error reading log stream, switching to polling");
                                break;
                            }
                            None => {
                                // Stream ended, break to switch to polling
                                debug!("Log stream ended, switching to polling fallback");
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                        // Check if stream seems stuck (no logs for 10 seconds)
                        if last_log_time.elapsed().as_secs() > 10 {
                            consecutive_empty += 1;
                            if consecutive_empty > 3 {
                                debug!("No logs for 30 seconds, switching to polling fallback");
                                break;
                            }
                        }
                    }
                }
            }
            
            // Fallback: Poll docker logs periodically
            debug!("Starting polling fallback for server {}", server_id_clone);
            let poll_options = LogsOptions::<String> {
                stdout: true,
                stderr: true,
                follow: false,
                tail: "100".to_string(),
                ..Default::default()
            };
            
            // Keep track of last seen log line content to avoid duplicates
            // Use SHA256 hash of the last line as a unique identifier
            let mut last_line_hash: Option<String> = None;
            
            loop {
                // Check if container still exists before polling logs
                match docker_clone.inspect_container(&container_clone, None).await {
                    Ok(inspect) => {
                        let state = inspect.state.unwrap_or_default();
                        if !state.running.unwrap_or(false) {
                            tracing::warn!("Container {} is not running, stopping log polling", container_clone);
                            break;
                        }
                    }
                    Err(e) => {
                        // Container doesn't exist or can't be inspected - stop polling
                        tracing::warn!("Container {} not found or inaccessible: {}, stopping log polling", 
                            container_clone, e);
                        break;
                    }
                }
                
                // Get recent logs
                let mut all_lines = vec![];
                let poll_stream = docker_clone.logs(&container_clone, Some(poll_options.clone()));
                tokio::pin!(poll_stream);
                while let Some(log_result) = poll_stream.next().await {
                    if let Ok(log) = log_result {
                        all_lines.push(log.to_string());
                    }
                }
                
                // Find where new logs start by looking for the last known line
                // This handles container restarts (docker returns all logs from start)
                let start_idx = if let Some(ref last_hash) = last_line_hash {
                    // Find the position of the last known line
                    all_lines.iter().position(|line| {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        line.hash(&mut hasher);
                        format!("{:x}", hasher.finish()) == *last_hash
                    }).map(|pos| pos + 1).unwrap_or(0)
                } else {
                    // First time: skip the last line to avoid sending it as new
                    if all_lines.len() > 1 { all_lines.len() - 1 } else { 0 }
                };
                
                // Send lines after the start index (new lines only)
                for line in all_lines.iter().skip(start_idx) {
                    if !line.is_empty() {
                        let stream = if line.starts_with("ERR") || line.to_lowercase().contains("error") {
                            "stderr"
                        } else {
                            "stdout"
                        };
                        send_log_output(server_id_clone, line.clone(), stream.to_string()).await;
                    }
                }
                
                // Update last line hash to the actual last line
                if let Some(last_line) = all_lines.last() {
                    if !last_line.is_empty() {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        last_line.hash(&mut hasher);
                        last_line_hash = Some(format!("{:x}", hasher.finish()));
                    }
                }
                
                // Poll every 2 seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });
        
        // Return immediately so response is sent to backend
        Ok(serde_json::json!({
            "status": "streaming",
            "message": "Log streaming started"
        }))
    } else {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            ..Default::default()
        };

        let mut logs_stream = docker.logs(&container_id, Some(options));
        let mut lines = vec![];
        while let Some(log_result) = logs_stream.next().await {
            match log_result {
                Ok(log) => lines.push(log.to_string()),
                Err(e) => {
                    error!(error = %e, "Error reading log");
                    break;
                }
            }
        }

        Ok(serde_json::json!({
            "status": "ok",
            "container_id": container_id,
            "lines": lines
        }))
    }
}
