//! Server lifecycle handlers (create, start, stop, restart, delete, logs, status).
//!
//! Phase 10, Plan 03 — Direct executor Minecraft server lifecycle via Java.
//!
//! These handlers manage Minecraft server JARs directly (without Docker):
//! - `handle_create`: download JAR, generate server.properties, write EULA
//! - `handle_start`: spawn `java -Xmx{ram}M -jar server.jar --nogui`, pipe logs
//! - `handle_stop`: graceful RCON `stop` + kill fallback
//! - `handle_restart`: stop then start
//! - `handle_delete`: kill + remove server directory
//! - `handle_logs`: return recent lines from latest.log
//! - `handle_status`: return server metadata and current state

use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;

use agent_proto::Task;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::net::TcpStream;
use tokio::process::Command;
use tracing::{error, info};
use uuid::Uuid;

use crate::agent::result_sender::OutboundMessage;
use crate::handlers::rcon::RconPacket;
use crate::task_state::{send_log_output, send_progress};
use super::*;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Attempt graceful RCON shutdown by sending `stop` command.
///
/// Connects to 127.0.0.1:rcon_port, authenticates, sends "stop".
/// The server will begin its graceful shutdown sequence.
async fn try_rcon_shutdown(server_id: Uuid, rcon_port: u16, rcon_password: &str) -> Result<()> {
    use tokio::io::AsyncReadExt;

    let addr: std::net::SocketAddr = ([127, 0, 0, 1], rcon_port).into();
    let mut stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("RCON connect failed on port {}", rcon_port))?;

    // Authenticate (SERVERDATA_AUTH = 3)
    let auth = RconPacket::new(1, 3, rcon_password);
    stream.write_all(&auth.encode()).await?;

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    if n < 12 {
        anyhow::bail!("Invalid RCON auth response");
    }

    // Send stop command (SERVERDATA_EXECCOMMAND = 2)
    let cmd = RconPacket::new(2, 2, "stop");
    stream.write_all(&cmd.encode()).await?;

    info!(server_id = %server_id, "RCON stop command sent");
    Ok(())
}

// ---------------------------------------------------------------------------
// handle_create
// ---------------------------------------------------------------------------

/// Create a new direct-executor server: download JAR, generate config, write EULA.
///
/// Expected payload fields:
/// - `server_id: Uuid` — unique server identifier
/// - `name: String` — display name
/// - `loader: String` — one of "paper", "fabric", "forge", "vanilla", "neoforge"
/// - `version: String` — Minecraft version (e.g. "1.21.4")
/// - `port: u16` — server port (default 25565)
/// - `ram: u64` — allocated RAM in MB
/// - `data_dir: String` — base data directory path (from agent config)
/// - `overrides: Option<HashMap<String, String>>` — server.properties overrides
///
/// Per D-13: EULA is auto-accepted by writing eula=true.
/// Per D-05/D-07: JAR auto-downloaded to {data_dir}/servers/{id}/server.jar.
pub async fn handle_create(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;
    let server_id: Uuid = payload
        .get("server_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .context("Missing or invalid server_id")?;
    let name: String = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Minecraft Server")
        .to_string();
    let loader_str: String = payload
        .get("loader")
        .and_then(|v| v.as_str())
        .context("Missing loader")?
        .to_string();
    let version: String = payload
        .get("version")
        .and_then(|v| v.as_str())
        .context("Missing version")?
        .to_string();
    let port: u16 = payload
        .get("port")
        .and_then(|v| v.as_u64())
        .map(|p| p as u16)
        .unwrap_or(25565u16);
    let ram: u64 = payload
        .get("ram")
        .and_then(|v| v.as_u64())
        .unwrap_or(1024);
    let data_dir_str: String = payload
        .get("data_dir")
        .and_then(|v| v.as_str())
        .context("Missing data_dir")?
        .to_string();
    let overrides: HashMap<String, String> = payload
        .get("overrides")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default();

    let data_dir = std::path::PathBuf::from(&data_dir_str);

    // Parse loader
    let loader =
        McLoader::from_str(&loader_str).ok_or_else(|| anyhow::anyhow!("Unsupported loader: {}", loader_str))?;

    // Create server directory
    let server_dir = server_dir_path(&data_dir, &server_id);
    tokio::fs::create_dir_all(&server_dir)
        .await
        .with_context(|| format!("Failed to create server directory: {}", server_dir.display()))?;

    // Download JAR
    let jar_path = server_jar_path(&data_dir, &server_id);
    download_jar(&loader, &version, &jar_path, &server_dir).await?;

    // Generate RCON password (random 32-char hex)
    let rcon_password = Uuid::new_v4().to_string().replace("-", "");
    // RCON port: 25575 + deterministic offset from server_id
    let rcon_port: u16 = 25575u16.wrapping_add((server_id.as_u128() % 1024) as u16);

    // Write server.properties
    let props = generate_server_properties(port, rcon_port, &rcon_password, &overrides);
    let props_path = server_dir.join("server.properties");
    tokio::fs::write(&props_path, &props)
        .await
        .with_context(|| format!("Failed to write server.properties to {}", props_path.display()))?;

    // Write EULA (D-13: auto-accept)
    write_eula(&server_dir).await?;

    // Create logs directory
    let log_dir = server_log_dir(&data_dir, &server_id);
    tokio::fs::create_dir_all(&log_dir).await?;

    // Register in global state
    let state = ServerState {
        server_id,
        display_name: name.clone(),
        mc_loader: loader,
        mc_version: version.clone(),
        status: ServerStatus::Stopped,
        port,
        allocated_ram: ram,
        path: server_dir,
        rcon_port,
        rcon_password,
        child: None,
        eula_accepted: true,
        auto_restart: false,
    };

    {
        let mut registry = DIRECT_SERVERS.lock().unwrap();
        registry.insert(server_id, state);
    }

    info!(server_id = %server_id, loader = %loader_str, version = %version, "Direct server created");

    Ok(serde_json::json!({
        "status": "created",
        "server_id": server_id,
        "name": name,
    }))
}

// ---------------------------------------------------------------------------
// handle_start
// ---------------------------------------------------------------------------

/// Start a direct-executor server.
///
/// Spawns: `java -Xmx{ram}M -jar server.jar --nogui`
/// Pipes stdout/stderr to log file and WS (LogLinePayload).
/// Monitors child process for crashes.
///
/// Per D-08: validates Java version against MC version before launching.
/// Per D-11: uses `--nogui` flag for headless operation.
/// Per D-14: detects crashes via non-zero exit code.
/// Per D-15: optional auto-restart.
/// Per T-10-13: kill_on_drop(true) to prevent orphan processes.
pub async fn handle_start(task: Task) -> Result<serde_json::Value> {
    let task_id = task.id;
    let server_id: Uuid = task
        .payload
        .get("server_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .context("Missing or invalid server_id")?;

    // Clone state fields while holding lock
    let state_clone = {
        let registry = DIRECT_SERVERS.lock().unwrap();
        let state = registry
            .get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?;
        // Idempotent check — already running
        if state.status == ServerStatus::Running {
            return Ok(serde_json::json!({ "status": "already_running", "server_id": server_id }));
        }
        (
            state.server_id,
            state.display_name.clone(),
            state.mc_version.clone(),
            state.path.clone(),
            state.allocated_ram,
            state.auto_restart,
        )
    };

    let (_sid, _name, mc_version, path, ram, _auto_restart) = state_clone;

    // Validate Java version (D-08)
    let java_info = java::detect_java_version()
        .ok_or_else(|| anyhow::anyhow!("Java not found on PATH"))?;
    java::validate_java_for_version(&mc_version, java_info.0)?;

    let jar_path = path.join("server.jar");
    if !jar_path.exists() {
        return Err(anyhow::anyhow!("Server JAR not found at {}", jar_path.display()));
    }

    send_progress(task_id, "running", 10.0, "Starting server process").await;

    // Spawn Java process (D-11)
    let mut child = Command::new("java")
        .arg(format!("-Xmx{}M", ram))
        .arg("-jar")
        .arg(&jar_path)
        .arg("--nogui")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true) // T-10-13: prevent orphan processes
        .spawn()
        .context("Failed to spawn java process")?;

    // Capture stdio BEFORE storing child in registry
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

    // Store child handle in registry
    {
        let mut registry = DIRECT_SERVERS.lock().unwrap();
        if let Some(state) = registry.get_mut(&server_id) {
            state.status = ServerStatus::Running;
            state.child = Some(child);
        }
    }

    let log_path = path.join("logs").join("latest.log");
    let sid_log = server_id;
    let sid_stderr = server_id;
    let sid_crash = server_id;
    let log_path_for_monitor = log_path.clone();

    // Spawn stdout log piping
    tokio::spawn(async move {
        let _ = tokio::fs::create_dir_all(log_path.parent().unwrap()).await;
        let mut log_file = match tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
        {
            Ok(f) => f,
            Err(e) => {
                error!(error = %e, "Failed to open log file");
                return;
            }
        };

        let reader = TokioBufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = log_file.write_all(line.as_bytes()).await;
            let _ = log_file.write_all(b"\n").await;
            let _ = log_file.flush().await;
            send_log_output(sid_log, line, "stdout".to_string()).await;
        }
    });

    // Spawn stderr log piping
    tokio::spawn(async move {
        let reader = TokioBufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            send_log_output(sid_stderr, line, "stderr".to_string()).await;
        }
    });

    // Spawn crash monitoring (D-14)
    tokio::spawn(async move {
        // Wait briefly for child to be stored in registry
        tokio::time::sleep(Duration::from_millis(100)).await;

        let child = {
            let mut registry = DIRECT_SERVERS.lock().unwrap();
            registry.get_mut(&server_id).and_then(|s| s.child.take())
        };

        let mut child = match child {
            Some(c) => c,
            None => return,
        };

        let exit = child.wait().await;
        let (exit_code, crashed) = match exit {
            Ok(status) => (status.code().unwrap_or(-1), !status.success()),
            Err(_) => (-1, true),
        };

        if crashed {
            error!(server_id = %sid_crash, exit_code, "Server process crashed");

            // Update status to Crashed
            {
                let mut registry = DIRECT_SERVERS.lock().unwrap();
                if let Some(state) = registry.get_mut(&sid_crash) {
                    state.status = ServerStatus::Crashed;
                    state.child = None;
                }
            }

            // Build + send crash report (T-10-12: truncated to 4KB max)
            let log_excerpt = std::fs::read_to_string(log_path_for_monitor.join("latest.log"))
                .ok()
                .map(|s| {
                    let lines: Vec<&str> = s.lines().rev().take(10).collect();
                    let excerpt: String = lines.iter().rev().cloned().collect::<Vec<&str>>().join("\n");
                    if excerpt.len() > 4096 {
                        format!("... (truncated) ...\n{}", &excerpt[excerpt.len() - 4000..])
                    } else {
                        excerpt
                    }
                })
                .unwrap_or_default();

            let report = crate::crash_reporter::build_crash_report(sid_crash, exit_code, log_excerpt);

            // Send crash report via progress sender (same channel as log output)
            if let Some(sender) = crate::task_state::get_progress_sender() {
                let msg = OutboundMessage::Proto(
                    agent_proto::messages::AgentToBackend::CrashReport(report),
                );
                let _ = sender.try_send(msg);
            }
        } else {
            // Normal exit — update status to Stopped
            {
                let mut registry = DIRECT_SERVERS.lock().unwrap();
                if let Some(state) = registry.get_mut(&sid_crash) {
                    state.status = ServerStatus::Stopped;
                    state.child = None;
                }
            }
            info!(server_id = %sid_crash, "Server stopped normally");
        }
    });

    info!(server_id = %server_id, "Server started");
    send_progress(task_id, "running", 100.0, "Server started").await;

    Ok(serde_json::json!({ "status": "started", "server_id": server_id }))
}

// ---------------------------------------------------------------------------
// handle_stop
// ---------------------------------------------------------------------------

/// Stop a running server via RCON clean shutdown with kill fallback.
///
/// Per D-12: connects via TCP on 127.0.0.1:rcon_port, authenticates,
/// sends `stop` command, then kills the process.
pub async fn handle_stop(task: Task) -> Result<serde_json::Value> {
    let server_id: Uuid = task
        .payload
        .get("server_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .context("Missing or invalid server_id")?;

    let (rcon_port, rcon_password) = {
        let registry = DIRECT_SERVERS.lock().unwrap();
        let state = registry
            .get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?;
        if state.status != ServerStatus::Running {
            return Ok(serde_json::json!({ "status": "already_stopped", "server_id": server_id }));
        }
        (state.rcon_port, state.rcon_password.clone())
    };

    // Try RCON shutdown (best-effort — server may not respond)
    let rcon_result = try_rcon_shutdown(server_id, rcon_port, &rcon_password).await;

    // Kill the child process (regardless of RCON result)
    let child = {
        let mut registry = DIRECT_SERVERS.lock().unwrap();
        registry.get_mut(&server_id).and_then(|s| s.child.take())
    };

    if let Some(mut c) = child {
        let _ = c.kill().await;
        let _ = c.wait().await;
    }

    // Update status
    {
        let mut registry = DIRECT_SERVERS.lock().unwrap();
        if let Some(state) = registry.get_mut(&server_id) {
            state.status = ServerStatus::Stopped;
            state.child = None;
        }
    }

    info!(
        server_id = %server_id,
        rcon_ok = rcon_result.is_ok(),
        "Server stopped"
    );

    Ok(serde_json::json!({ "status": "stopped", "server_id": server_id }))
}

// ---------------------------------------------------------------------------
// handle_restart
// ---------------------------------------------------------------------------

/// Restart a server — stop then start.
///
/// Idempotent: if running → stop then start; if stopped/crashed → start.
pub async fn handle_restart(task: Task) -> Result<serde_json::Value> {
    let server_id: Uuid = task
        .payload
        .get("server_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .context("Missing or invalid server_id")?;

    // Check current state
    let status = {
        let registry = DIRECT_SERVERS.lock().unwrap();
        registry.get(&server_id).map(|s| s.status)
    };

    match status {
        Some(ServerStatus::Running) | Some(ServerStatus::Stopped) | Some(ServerStatus::Crashed) => {
            // Stop if running
            if status == Some(ServerStatus::Running) {
                handle_stop(Task::new(
                    "direct.server.stop".into(),
                    serde_json::json!({ "server_id": server_id }),
                ))
                .await?;
            }
            // Start
            handle_start(Task::new(
                "direct.server.start".into(),
                serde_json::json!({ "server_id": server_id }),
            ))
            .await
        }
        None => Err(anyhow::anyhow!("Server {} not found", server_id)),
    }
}

// ---------------------------------------------------------------------------
// handle_delete
// ---------------------------------------------------------------------------

/// Delete a server: kill the process and remove its directory.
pub async fn handle_delete(task: Task) -> Result<serde_json::Value> {
    let server_id: Uuid = task
        .payload
        .get("server_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .context("Missing or invalid server_id")?;

    let server_path = {
        let mut registry = DIRECT_SERVERS.lock().unwrap();
        let state = registry.remove(&server_id);
        match state {
            Some(mut s) => {
                // Kill if running
                if let Some(mut child) = s.child.take() {
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                }
                s.path
            }
            None => {
                return Ok(serde_json::json!({
                    "status": "not_found",
                    "server_id": server_id,
                }));
            }
        }
    };

    // Remove server directory
    if server_path.exists() {
        tokio::fs::remove_dir_all(&server_path)
            .await
            .with_context(|| format!("Failed to remove server directory: {}", server_path.display()))?;
    }

    info!(server_id = %server_id, "Server deleted");
    Ok(serde_json::json!({ "status": "deleted", "server_id": server_id }))
}

// ---------------------------------------------------------------------------
// handle_logs
// ---------------------------------------------------------------------------

/// Return recent log lines from latest.log.
///
/// Payload: `tail` (optional, default 200) — number of recent lines to return.
pub async fn handle_logs(task: Task) -> Result<serde_json::Value> {
    let server_id: Uuid = task
        .payload
        .get("server_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .context("Missing or invalid server_id")?;

    let tail = task.payload.get("tail").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

    let log_path = {
        let registry = DIRECT_SERVERS.lock().unwrap();
        let state = registry
            .get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?;
        state.path.join("logs").join("latest.log")
    };

    let lines = if log_path.exists() {
        let content = tokio::fs::read_to_string(&log_path)
            .await
            .with_context(|| format!("Failed to read log: {}", log_path.display()))?;
        content
            .lines()
            .rev()
            .take(tail)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    Ok(serde_json::json!({
        "status": "ok",
        "server_id": server_id,
        "lines": lines,
    }))
}

// ---------------------------------------------------------------------------
// handle_status
// ---------------------------------------------------------------------------

/// Return current server metadata and state.
pub async fn handle_status(task: Task) -> Result<serde_json::Value> {
    let server_id: Uuid = task
        .payload
        .get("server_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .context("Missing or invalid server_id")?;

    let registry = DIRECT_SERVERS.lock().unwrap();
    let state = registry
        .get(&server_id)
        .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?;

    let status_str = match state.status {
        ServerStatus::Running => "running",
        ServerStatus::Stopped => "stopped",
        ServerStatus::Crashed => "crashed",
    };

    Ok(serde_json::json!({
        "server_id": state.server_id,
        "name": state.display_name,
        "loader": format!("{:?}", state.mc_loader).to_lowercase(),
        "version": state.mc_version,
        "status": status_str,
        "port": state.port,
        "ram_mb": state.allocated_ram,
        "rcon_port": state.rcon_port,
        "auto_restart": state.auto_restart,
    }))
}
