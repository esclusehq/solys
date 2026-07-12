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
    props.push_str("level-type=default\n");
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

/// Collect statuses of all direct-executor servers for heartbeat payload.
/// Returns Vec of (server_id, display_name, status_string).
pub fn collect_server_statuses() -> Vec<(Uuid, String, String)> {
    let registry = DIRECT_SERVERS.lock().unwrap();
    registry
        .iter()
        .map(|(id, state)| {
            let status = match state.status {
                ServerStatus::Running => {
                    let is_alive = std::process::Command::new("sh")
                        .args(["-c", &format!("pgrep -f 'java.*{}' >/dev/null 2>&1", id)])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if is_alive { "running" } else { "stopped" }
                }
                ServerStatus::Stopped => "stopped",
                ServerStatus::Crashed => "crashed",
            };
            (*id, state.display_name.clone(), status.to_string())
        })
        .collect()
}
