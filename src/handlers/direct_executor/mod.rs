//! Direct Executor - Run Minecraft server JARs directly via Java
//!
//! Supports Paper, Fabric, Forge, Vanilla, and NeoForge loaders.
//! Server lifecycle: create (download JAR + config) → start (java -jar) →
//! stop (RCON) → restart → delete. Logs piped to file + WebSocket.

pub mod java;
pub mod paper;
pub mod fabric;
pub mod vanilla;
pub mod forge;
pub mod neoforge;
pub mod server;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::state::ServerEntry;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Supported Minecraft server loaders for direct execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McLoader {
    Paper,
    Fabric,
    Forge,
    Vanilla,
    NeoForge,
}

impl McLoader {
    /// Parse from string, case-insensitive. Returns None for unrecognized loaders.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "paper" => Some(Self::Paper),
            "fabric" => Some(Self::Fabric),
            "forge" => Some(Self::Forge),
            "vanilla" => Some(Self::Vanilla),
            "neoforge" => Some(Self::NeoForge),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Running,
    Stopped,
    Crashed,
}

/// Runtime state for a single direct-executor Minecraft server.
#[derive(Debug)]
pub struct ServerState {
    pub server_id: Uuid,
    pub display_name: String,
    pub mc_loader: McLoader,
    pub mc_version: String,
    pub status: ServerStatus,
    pub port: u16,
    pub allocated_ram: u64,        // MB
    pub path: PathBuf,             // {data_dir}/servers/{id}/
    pub rcon_port: u16,
    pub rcon_password: String,
    pub child: Option<tokio::process::Child>,
    pub eula_accepted: bool,
    pub auto_restart: bool,
}

// ---------------------------------------------------------------------------
// Global registry
// ---------------------------------------------------------------------------

/// Global registry of all direct-executor servers.
/// Populated by `direct.server.create`, modified by lifecycle handlers.
pub static DIRECT_SERVERS: LazyLock<Mutex<HashMap<Uuid, ServerState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ---------------------------------------------------------------------------
// JAR download dispatch
// ---------------------------------------------------------------------------

/// Download a server JAR for the given loader + version to the destination path.
/// For Forge/NeoForge, this runs the full installer flow.
pub async fn download_jar(
    loader: &McLoader,
    version: &str,
    dest: &Path,
    server_dir: &Path,
) -> Result<()> {
    match loader {
        McLoader::Paper => {
            let url = paper::get_download_url(version).await?;
            download_to_file(&url, dest).await?;
        }
        McLoader::Fabric => {
            // Fabric meta returns the server JAR directly
            let url = fabric::get_download_url(version).await?;
            download_to_file(&url, dest).await?;
        }
        McLoader::Vanilla => {
            let url = vanilla::get_download_url(version).await?;
            download_to_file(&url, dest).await?;
        }
        McLoader::Forge => {
            // Two-step: download installer JAR → run --installServer → find launcher
            let installer_url = forge::get_installer_url(version).await?;
            let installer_path = server_dir.join("forge-installer.jar");
            download_to_file(&installer_url, &installer_path).await?;
            forge::run_installer(&installer_path, server_dir).await?;
            let launcher = forge::resolve_launcher_jar(server_dir)?;
            // Copy launcher to server.jar
            tokio::fs::copy(&launcher, dest).await?;
            // Cleanup installer
            let _ = tokio::fs::remove_file(&installer_path).await;
        }
        McLoader::NeoForge => {
            let installer_url = neoforge::get_installer_url(version).await?;
            let installer_path = server_dir.join("neoforge-installer.jar");
            download_to_file(&installer_url, &installer_path).await?;
            neoforge::run_installer(&installer_path, server_dir).await?;
            let launcher = neoforge::resolve_launcher_jar(server_dir)?;
            tokio::fs::copy(&launcher, dest).await?;
            let _ = tokio::fs::remove_file(&installer_path).await;
        }
    }
    info!(?loader, version, path = %dest.display(), "Server JAR downloaded");
    Ok(())
}

/// Generic HTTP(S) download helper. Uses reqwest, verifies response status,
/// validates JAR magic bytes (PK\x03\x04 for ZIP/JAR format).
async fn download_to_file(url: &str, dest: &Path) -> Result<()> {
    // Create parent directories
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to download from {}", url))?;

    if !response.status().is_success() {
        bail!("Download failed with HTTP {} from {}", response.status(), url);
    }

    let bytes = response.bytes().await?;

    // Validate JAR/ZIP magic bytes (PK\x03\x04)
    if bytes.len() < 4 || bytes[0] != 0x50 || bytes[1] != 0x4B || bytes[2] != 0x03 || bytes[3] != 0x04
    {
        bail!("Downloaded file is not a valid JAR/ZIP archive (missing PK magic bytes)");
    }

    tokio::fs::write(dest, &bytes)
        .await
        .with_context(|| format!("Failed to write JAR to {}", dest.display()))?;

    debug!(path = %dest.display(), size = bytes.len(), "Downloaded JAR");
    Ok(())
}

// ---------------------------------------------------------------------------
// Port availability helpers
// ---------------------------------------------------------------------------

/// Check whether a TCP port is free by attempting to bind to it.
/// Drops the listener immediately after the check so the port is released.
pub fn is_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("0.0.0.0", port)).is_ok()
}

/// Find the first available port starting from `preferred`, trying up to
/// `max_attempts` consecutive ports.  Returns the preferred port if none
/// of the candidates are free (the caller will get a bind error at start).
pub fn find_available_port(preferred: u16, max_attempts: u16) -> u16 {
    let end = preferred.saturating_add(max_attempts);
    for port in preferred..end {
        if is_port_available(port) {
            return port;
        }
    }
    preferred
}

// ---------------------------------------------------------------------------
// server.properties generation
// ---------------------------------------------------------------------------

/// Generate server.properties content with RCON enabled and configurable overrides.
///
/// `overrides` map allows backend to override any property (difficulty, gamemode, motd, etc).
/// RCON settings are ALWAYS set (D-12), cannot be overridden by user.
pub fn generate_server_properties(
    port: u16,
    rcon_port: u16,
    rcon_password: &str,
    overrides: &HashMap<String, String>,
) -> String {
    let mut props = String::new();
    props.push_str("#Minecraft server properties (generated by escluse-agent)\n");
    props.push_str("# https://minecraft.fandom.com/wiki/Server.properties\n");
    props.push_str(&format!("server-port={}\n", port));
    props.push_str(&format!("rcon.port={}\n", rcon_port));
    props.push_str(&format!("rcon.password={}\n", rcon_password));
    props.push_str("enable-rcon=true\n");
    props.push_str("broadcast-rcon-to-ops=false\n"); // D-12: security hardening
    props.push_str("enable-query=false\n");
    props.push_str("enable-status=true\n");
    props.push_str("online-mode=true\n");
    props.push_str("max-players=20\n");
    props.push_str("gamemode=survival\n");
    props.push_str("difficulty=easy\n");
    props.push_str("motd=A Minecraft Server\n");
    props.push_str("pvp=true\n");
    props.push_str("hardcore=false\n");
    props.push_str("allow-flight=false\n");
    props.push_str("white-list=false\n");
    props.push_str("enforce-whitelist=false\n");
    props.push_str("spawn-npcs=true\n");
    props.push_str("spawn-animals=true\n");
    props.push_str("spawn-monsters=true\n");
    props.push_str("generate-structures=true\n");
    props.push_str("max-world-size=29999984\n");
    props.push_str("resource-pack=\n");

    // Apply overrides (except RCON settings which are ALWAYS set)
    let protected = [
        "enable-rcon",
        "rcon.port",
        "rcon.password",
        "broadcast-rcon-to-ops",
    ];
    for (key, value) in overrides {
        if !protected.contains(&key.as_str()) {
            // If key already exists in base template, it will be duplicated.
            // Minecraft uses the LAST occurrence. So we append overrides at the end.
            props.push_str(&format!("{}={}\n", key, value));
        }
    }

    props
}

// ---------------------------------------------------------------------------
// EULA
// ---------------------------------------------------------------------------

/// Write `eula=true` to the eula.txt file in the server directory.
pub async fn write_eula(server_dir: &Path) -> Result<()> {
    let path = server_dir.join("eula.txt");
    tokio::fs::write(&path, "eula=true\n")
        .await
        .with_context(|| format!("Failed to write EULA to {}", path.display()))?;
    info!(path = %path.display(), "EULA accepted");
    Ok(())
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Construct the server directory path from data_dir and server_id.
pub fn server_dir_path(data_dir: &Path, server_id: &Uuid) -> PathBuf {
    data_dir.join("servers").join(server_id.to_string())
}

/// Construct the server JAR path.
pub fn server_jar_path(data_dir: &Path, server_id: &Uuid) -> PathBuf {
    server_dir_path(data_dir, server_id).join("server.jar")
}

/// Construct the log directory path.
pub fn server_log_dir(data_dir: &Path, server_id: &Uuid) -> PathBuf {
    server_dir_path(data_dir, server_id).join("logs")
}

/// Construct the latest.log path.
pub fn server_log_path(data_dir: &Path, server_id: &Uuid) -> PathBuf {
    server_log_dir(data_dir, server_id).join("latest.log")
}

// ---------------------------------------------------------------------------
// Heartbeat helper
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Startup reconciliation
// ---------------------------------------------------------------------------

/// Parse a loader string into McLoader, defaulting to Vanilla.
fn parse_mc_loader(s: &Option<String>) -> McLoader {
    match s.as_deref() {
        Some("paper") => McLoader::Paper,
        Some("fabric") => McLoader::Fabric,
        Some("forge") => McLoader::Forge,
        Some("vanilla") | Some("neoforge") => McLoader::Vanilla,
        _ => McLoader::Vanilla,
    }
}

/// Rebuild `DIRECT_SERVERS` from persisted state + directory scan.
///
/// Called once at agent startup after `state::load_state()`. All entries are
/// given `status: Stopped` — any process that was running before the restart
/// died with the agent. The next heartbeat cycle will report correct statuses
/// to the backend.
pub fn reconcile_direct_servers(
    loaded_entries: &[crate::state::ServerEntry],
    data_dir: &Path,
) {
    let mut registry = DIRECT_SERVERS.lock().unwrap_or_else(|e| e.into_inner());

    for entry in loaded_entries {
        let server_dir = data_dir.join("servers").join(entry.server_id.to_string());
        let state = ServerState {
            server_id: entry.server_id,
            display_name: entry.name.clone(),
            mc_loader: parse_mc_loader(&entry.mc_loader),
            mc_version: entry.mc_version.clone().unwrap_or_default(),
            status: ServerStatus::Stopped,
            port: entry.port,
            allocated_ram: entry.allocated_ram,
            path: server_dir,
            rcon_port: entry.rcon_port,
            rcon_password: entry.rcon_password.clone(),
            child: None,
            eula_accepted: true,
            auto_restart: entry.auto_restart,
        };
        registry.insert(entry.server_id, state);
    }

    // Fallback: scan {data_dir}/servers/ for UUID directories not in state
    let servers_dir = data_dir.join("servers");
    if servers_dir.exists() {
        if let Ok(read_dir) = std::fs::read_dir(&servers_dir) {
            for dir_entry in read_dir.flatten() {
                let name_os = dir_entry.file_name();
                let name_str = name_os.to_string_lossy();
                if let Ok(sid) = Uuid::parse_str(&name_str) {
                    if !registry.contains_key(&sid) {
                        // Read port from server.properties if available
                        let props_path = dir_entry.path().join("server.properties");
                        let port = std::fs::read_to_string(&props_path)
                            .ok()
                            .and_then(|s| {
                                s.lines()
                                    .find(|l| l.starts_with("server-port="))
                                    .and_then(|l| l.split('=').nth(1))
                                    .and_then(|p| p.trim().parse().ok())
                            })
                            .unwrap_or(25565u16);

                        let state = ServerState {
                            server_id: sid,
                            display_name: sid.to_string(),
                            mc_loader: McLoader::Vanilla,
                            mc_version: String::new(),
                            status: ServerStatus::Stopped,
                            port,
                            allocated_ram: 1024,
                            path: dir_entry.path(),
                            rcon_port: 0,
                            rcon_password: String::new(),
                            child: None,
                            eula_accepted: true,
                            auto_restart: false,
                        };
                        registry.insert(sid, state);
                    }
                }
            }
        }
    }

    let count = registry.len();
    if count > 0 {
        info!(
            servers = count,
            from_state = loaded_entries.len(),
            "Reconciled direct servers after restart"
        );
    }
}

// ---------------------------------------------------------------------------
// State persistence helpers
// ---------------------------------------------------------------------------

/// Build a `ServerEntry` from a `ServerState`.
fn server_state_to_entry(state: &ServerState) -> ServerEntry {
    ServerEntry {
        server_id: state.server_id,
        name: state.display_name.clone(),
        game_type: format!("{:?}", state.mc_loader).to_lowercase(),
        container_id: None,
        status: match state.status {
            ServerStatus::Running => "running",
            ServerStatus::Stopped => "stopped",
            ServerStatus::Crashed => "crashed",
        }
        .to_string(),
        port: state.port,
        rcon_port: state.rcon_port,
        rcon_password: state.rcon_password.clone(),
        allocated_ram: state.allocated_ram,
        auto_restart: state.auto_restart,
        mc_version: Some(state.mc_version.clone()),
        mc_loader: Some(format!("{:?}", state.mc_loader).to_lowercase()),
    }
}

/// Persist `DIRECT_SERVERS` to `state.json`.
/// Called after every lifecycle transition so state survives a crash/restart.
pub async fn persist_server_state() {
    let entries: Vec<ServerEntry> = {
        let registry = DIRECT_SERVERS.lock().unwrap_or_else(|e| e.into_inner());
        registry.values().map(server_state_to_entry).collect()
    };
    crate::state::save_server_entries(&entries).await;
}

/// Remove a single server from persisted state (e.g. after delete).
pub async fn persist_remove_server(server_id: &Uuid) {
    crate::state::remove_server_entry(server_id).await;
}

/// Collect all direct-server entries as `ServerEntry` (for shutdown save).
pub fn collect_direct_server_entries() -> Vec<ServerEntry> {
    let registry = DIRECT_SERVERS.lock().unwrap_or_else(|e| e.into_inner());
    registry.values().map(server_state_to_entry).collect()
}

// ---------------------------------------------------------------------------
// Heartbeat helper
// ---------------------------------------------------------------------------

/// Collect statuses of all direct-executor servers for heartbeat payload.
/// Returns Vec of (server_id, display_name, status_string).
pub fn collect_server_statuses() -> Vec<(Uuid, String, String)> {
    // First pass: collect IDs of servers that are marked Running but whose
    // Java process has died. Drop the lock before updating so we don't deadlock.
    let dead_ids: Vec<Uuid> = {
        let registry = DIRECT_SERVERS.lock().unwrap_or_else(|e| e.into_inner());
        registry
            .iter()
            .filter_map(|(id, state)| {
                if matches!(state.status, ServerStatus::Running) {
                    let is_alive = std::process::Command::new("sh")
                        .args(["-c", &format!("pgrep -f 'java.*{}' >/dev/null 2>&1", id)])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if !is_alive { Some(*id) } else { None }
                } else {
                    None
                }
            })
            .collect()
    };

    // Update DIRECT_SERVERS entries for dead processes so the backend sees
    // the correct status without waiting for the next lifecycle event.
    // Also stop relay tunnels so the backend sees tunnel disconnect.
    if !dead_ids.is_empty() {
        let mut registry = DIRECT_SERVERS.lock().unwrap_or_else(|e| e.into_inner());
        for id in &dead_ids {
            if let Some(entry) = registry.get_mut(id) {
                entry.status = ServerStatus::Stopped;
            }
        }
        drop(registry);
        for id in &dead_ids {
            let sid = *id;
            tokio::spawn(async move {
                crate::state::relay_manager().stop_server(&sid).await;
            });
        }
    }

    // Build status result from the updated registry.
    let registry = DIRECT_SERVERS.lock().unwrap_or_else(|e| e.into_inner());
    registry
        .iter()
        .map(|(id, state)| {
            let status = match state.status {
                ServerStatus::Running => "running",
                ServerStatus::Stopped => "stopped",
                ServerStatus::Crashed => "crashed",
            };
            (*id, state.display_name.clone(), status.to_string())
        })
        .collect()
}
