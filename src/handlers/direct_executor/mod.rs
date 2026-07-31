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
// server.properties parsing + RCON self-heal
// ---------------------------------------------------------------------------

/// Parse `server.properties` content into a key → value map.
/// Ignores blank lines and comment lines (`#`). Values are trimmed.
pub fn parse_properties(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

const RCON_KEYS: [&str; 4] = [
    "enable-rcon",
    "rcon.port",
    "rcon.password",
    "broadcast-rcon-to-ops",
];

/// Decide whether `server.properties` content needs the RCON/port block healed.
///
/// Minecraft rewrites server.properties with defaults on first boot, which
/// resets `enable-rcon=false` and clears `rcon.password`. The heal rewrites
/// the file (preserving every other key) with the agent-managed values so the
/// console and graceful RCON shutdown keep working.
pub fn props_needs_heal(content: &str, server_port: u16, rcon_port: u16, rcon_password: &str) -> bool {
    let props = parse_properties(content);
    let enabled = props.get("enable-rcon").map(|v| v == "true").unwrap_or(false);
    let pw_matches = props
        .get("rcon.password")
        .map(|v| v == rcon_password)
        .unwrap_or(false);
    let port_ok = props
        .get("rcon.port")
        .and_then(|v| v.parse::<u16>().ok())
        .map(|p| p == rcon_port)
        .unwrap_or(false);
    let server_port_ok = props
        .get("server-port")
        .and_then(|v| v.parse::<u16>().ok())
        .map(|p| p == server_port)
        .unwrap_or(false);
    !(enabled && pw_matches && port_ok && server_port_ok)
}

/// Produce healed `server.properties` content: drops any existing RCON and
/// server-port keys, then appends the agent-managed values (Minecraft uses
/// the LAST occurrence of a key, so appending wins over any stale duplicates).
pub fn heal_properties_content(
    content: &str,
    server_port: u16,
    rcon_port: u16,
    rcon_password: &str,
) -> String {
    let mut out = String::new();
    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if let Some((key, _)) = line.split_once('=') {
            let key = key.trim();
            if key == "server-port" || RCON_KEYS.contains(&key) {
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("server-port=");
    out.push_str(&server_port.to_string());
    out.push('\n');
    out.push_str("enable-rcon=true\n");
    out.push_str("broadcast-rcon-to-ops=false\n");
    out.push_str("rcon.port=");
    out.push_str(&rcon_port.to_string());
    out.push('\n');
    out.push_str("rcon.password=");
    out.push_str(rcon_password);
    out.push('\n');
    out
}

/// Ensure `{server_dir}/server.properties` carries the agent-managed RCON and
/// port settings. Returns `Ok(true)` when the file was rewritten, `Ok(false)`
/// when it was already correct (or missing). Missing files are left untouched —
/// the create flow writes the template.
pub async fn heal_server_properties(
    server_dir: &Path,
    server_port: u16,
    rcon_port: u16,
    rcon_password: &str,
) -> Result<bool> {
    let props_path = server_dir.join("server.properties");
    let content = match tokio::fs::read_to_string(&props_path).await {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };

    if !props_needs_heal(&content, server_port, rcon_port, rcon_password) {
        return Ok(false);
    }

    let healed = heal_properties_content(&content, server_port, rcon_port, rcon_password);
    tokio::fs::write(&props_path, &healed)
        .await
        .with_context(|| format!("Failed to heal server.properties at {}", props_path.display()))?;
    info!(
        path = %props_path.display(),
        server_port, rcon_port,
        "Healed server.properties (RCON + port enforced)"
    );
    Ok(true)
}

/// Read the port/RCON values persisted in `{server_dir}/server.properties`.
/// Returns (server_port, rcon_port, rcon_password). Missing or unparsable
/// values fall back to defaults; an empty RCON password is replaced with a
/// freshly generated one so the self-heal always has a secret to enforce.
fn read_properties_values(server_dir: &Path) -> (u16, u16, String) {
    let props_path = server_dir.join("server.properties");
    let props = std::fs::read_to_string(&props_path)
        .ok()
        .map(|c| parse_properties(&c))
        .unwrap_or_default();

    let server_port = props
        .get("server-port")
        .and_then(|v| v.parse().ok())
        .unwrap_or(25565u16);
    let rcon_port = props
        .get("rcon.port")
        .and_then(|v| v.parse().ok())
        .unwrap_or(25575u16);
    let rcon_password = props
        .get("rcon.password")
        .filter(|p| !p.is_empty())
        .cloned()
        .unwrap_or_else(generate_rcon_password);
    (server_port, rcon_port, rcon_password)
}

/// Generate a random RCON password (32 hex chars).
pub fn generate_rcon_password() -> String {
    Uuid::new_v4().to_string().replace('-', "")
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
        // If the persisted entry has no RCON password (e.g. created by an old
        // binary before the RCON template), fall back to the file's values —
        // generating a fresh password when the file has none. The start-time
        // self-heal then enforces it.
        let (file_port, file_rcon_port, file_rcon_password) = read_properties_values(&server_dir);
        let port = if entry.port == 0 { file_port } else { entry.port };
        let rcon_port = if entry.rcon_port == 0 { file_rcon_port } else { entry.rcon_port };
        let rcon_password = if entry.rcon_password.is_empty() {
            file_rcon_password
        } else {
            entry.rcon_password.clone()
        };
        let state = ServerState {
            server_id: entry.server_id,
            display_name: entry.name.clone(),
            mc_loader: parse_mc_loader(&entry.mc_loader),
            mc_version: entry.mc_version.clone().unwrap_or_default(),
            status: ServerStatus::Stopped,
            port,
            allocated_ram: entry.allocated_ram,
            path: server_dir,
            rcon_port,
            rcon_password,
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
                        // Read port + RCON values from server.properties if
                        // available (healed back into the file at next start).
                        let (port, rcon_port, rcon_password) =
                            read_properties_values(&dir_entry.path());

                        let state = ServerState {
                            server_id: sid,
                            display_name: sid.to_string(),
                            mc_loader: McLoader::Vanilla,
                            mc_version: String::new(),
                            status: ServerStatus::Stopped,
                            port,
                            allocated_ram: 1024,
                            path: dir_entry.path(),
                            rcon_port,
                            rcon_password,
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_minecraft_props() -> &'static str {
        "#Minecraft server properties\n\
         #Fri Jul 31 12:18:59 GMT 2026\n\
         enable-rcon=false\n\
         rcon.password=\n\
         rcon.port=25575\n\
         server-port=25565\n\
         motd=A Minecraft Server\n\
         gamemode=survival\n"
    }

    #[test]
    fn generate_properties_always_enables_rcon() {
        let props = generate_server_properties(25565, 25575, "secret123", &HashMap::new());
        assert!(props.contains("enable-rcon=true\n"));
        assert!(props.contains("rcon.password=secret123\n"));
        assert!(props.contains("rcon.port=25575\n"));
        assert!(props.contains("broadcast-rcon-to-ops=false\n"));
        assert!(props.contains("server-port=25565\n"));
        assert!(props.starts_with("#Minecraft server properties (generated by escluse-agent)"));
    }

    #[test]
    fn generate_properties_protects_rcon_from_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert("enable-rcon".to_string(), "false".to_string());
        overrides.insert("rcon.password".to_string(), "user-superset".to_string());
        overrides.insert("difficulty".to_string(), "hard".to_string());

        let props = generate_server_properties(25566, 25599, "managed", &overrides);

        // Protected keys keep the agent-managed value...
        assert!(props.contains("enable-rcon=true\n"));
        assert!(props.contains("rcon.password=managed\n"));
        // ...while non-protected overrides are applied (last occurrence wins).
        assert!(props.contains("difficulty=hard\n"));
    }

    #[test]
    fn parse_properties_skips_comments_and_blank_lines() {
        let map = parse_properties(sample_minecraft_props());
        assert_eq!(map.get("enable-rcon"), Some(&"false".to_string()));
        assert_eq!(map.get("server-port"), Some(&"25565".to_string()));
        assert_eq!(map.get("rcon.password"), Some(&"".to_string()));
        assert_eq!(map.get("motd"), Some(&"A Minecraft Server".to_string()));
        assert_eq!(map.len(), 6);
    }

    #[test]
    fn parse_properties_keeps_equals_in_value() {
        let map = parse_properties("motd=Welcome = home\n");
        assert_eq!(map.get("motd"), Some(&"Welcome = home".to_string()));
    }

    #[test]
    fn props_needs_heal_detects_minecraft_rewrite() {
        // Minecraft first-boot rewrite (RCON off, empty password) needs heal.
        assert!(props_needs_heal(sample_minecraft_props(), 25565, 25575, "abc123"));
        // Fully correct file does not.
        let good = "enable-rcon=true\nrcon.password=abc123\nrcon.port=25575\nserver-port=25565\n";
        assert!(!props_needs_heal(good, 25565, 25575, "abc123"));
        // Wrong password or port still triggers heal.
        assert!(props_needs_heal(good, 25565, 25575, "other"));
        assert!(props_needs_heal(good, 25565, 25599, "abc123"));
        assert!(props_needs_heal(good, 25566, 25575, "abc123"));
        // Empty content needs heal.
        assert!(props_needs_heal("", 25565, 25575, "abc123"));
    }

    #[test]
    fn heal_properties_content_preserves_user_keys() {
        let healed = heal_properties_content(sample_minecraft_props(), 25566, 25599, "newpw");
        let map = parse_properties(&healed);

        assert_eq!(map.get("enable-rcon"), Some(&"true".to_string()));
        assert_eq!(map.get("rcon.password"), Some(&"newpw".to_string()));
        assert_eq!(map.get("rcon.port"), Some(&"25599".to_string()));
        assert_eq!(map.get("server-port"), Some(&"25566".to_string()));
        // Non-RCON user keys survive.
        assert_eq!(map.get("motd"), Some(&"A Minecraft Server".to_string()));
        assert_eq!(map.get("gamemode"), Some(&"survival".to_string()));
        // No duplicate RCON keys (last occurrence would win anyway).
        let rcon_count = healed.lines().filter(|l| l.starts_with("enable-rcon=")).count();
        assert_eq!(rcon_count, 1);
    }

    #[test]
    fn generate_rcon_password_is_32_hex_chars() {
        let pw = generate_rcon_password();
        assert_eq!(pw.len(), 32);
        assert!(pw.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(generate_rcon_password(), generate_rcon_password());
    }
}
