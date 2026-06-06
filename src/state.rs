//! Agent State Persistence
//!
//! Handles saving and loading agent state for auto-recovery after restart.
//! Per D-19: Persists server list + container mapping + metadata
//! Per D-21: JSON with atomic write (write to temp, then rename)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::OnceCell;

/// Agent state to persist (D-19)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentState {
    /// Server list - what we manage
    pub servers: Vec<ServerEntry>,
    /// Container ID mapping (server_id -> container_id)
    pub container_map: HashMap<String, String>,
    /// Agent metadata for health tracking
    pub metadata: AgentMetadata,
}

/// Individual server entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub server_id: uuid::Uuid,
    pub name: String,
    pub game_type: String,
    pub container_id: Option<String>,
    pub status: String,
}

/// Agent metadata for health tracking (D-20)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    pub restart_count: u32,
    pub last_start: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
}

/// Get the state file path (D-02: Use XDG data directory)
pub fn get_state_path() -> Option<PathBuf> {
    // D-02: Use XDG data directory
    let data_dir = if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(dir).join("escluse-agent")
    } else if let Some(dir) = dirs::data_local_dir() {
        dir.join("escluse-agent")
    } else {
        return None;
    };

    // Create directory if needed
    let _ = std::fs::create_dir_all(&data_dir);

    Some(data_dir.join("state.json"))
}

/// Load state from disk (D-23: auto-recovery step 1)
pub async fn load_state() -> Option<AgentState> {
    let path = get_state_path()?;

    let contents = fs::read_to_string(&path).await.ok()?;
    let state: AgentState = serde_json::from_str(&contents).ok()?;

    tracing::info!(
        servers = state.servers.len(),
        containers = state.container_map.len(),
        "Loaded persisted state"
    );

    Some(state)
}

/// Save state to disk with atomic write (D-21)
/// Write to temp file first, then rename (atomic on POSIX)
pub async fn save_state(state: &AgentState) -> std::io::Result<()> {
    let path = get_state_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "No state path")
    })?;

    let temp_path = path.with_extension("tmp");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Write to temp file (D-21: atomic write via temp + rename)
    fs::write(&temp_path, json).await?;

    // Atomic rename (POSIX guarantees atomicity for rename over same filesystem)
    tokio::fs::rename(&temp_path, &path).await?;

    tracing::debug!("State saved to {:?}", path);

    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 67: process-global Docker client
// ---------------------------------------------------------------------------

static DOCKER_GLOBAL: OnceCell<Arc<bollard::Docker>> = OnceCell::const_new();

/// Set the global Docker client. Called once at startup from `main.rs` /
/// `service_main.rs` after runtime detection. Subsequent calls are no-ops.
pub fn set_docker_global(client: Arc<bollard::Docker>) {
    let _ = DOCKER_GLOBAL.set(client);
}

/// Borrow the global Docker client, or `None` if it hasn't been initialised.
pub fn docker_global() -> Option<Arc<bollard::Docker>> {
    DOCKER_GLOBAL.get().cloned()
}

/// Convenience: resolve the local audit data directory (the directory the
/// `init_audit_logger` was given). Falls back to `state.json`'s directory
/// or `.` when nothing is configured.
pub fn audit_data_dir() -> std::path::PathBuf {
    get_state_path()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_state_path() {
        // Should return a path
        let path = get_state_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().ends_with("escluse-agent/state.json"));
    }
}