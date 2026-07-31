//! Agent State Persistence
//!
//! Handles saving and loading agent state for auto-recovery after restart.
//! Per D-19: Persists server list + container mapping + metadata
//! Per D-21: JSON with atomic write (write to temp, then rename)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::OnceCell;
use uuid::Uuid;

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
    pub port: u16,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub allocated_ram: u64,
    pub auto_restart: bool,
    pub mc_version: Option<String>,
    pub mc_loader: Option<String>,
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
// Server state persistence helpers
// ---------------------------------------------------------------------------

/// Persist a list of ServerEntry records to state.json.
/// Loads the current state, replaces the server list, and saves atomically.
pub async fn save_server_entries(entries: &[ServerEntry]) {
    let mut current = load_state().await.unwrap_or_default();
    current.servers = entries.to_vec();
    if let Err(e) = save_state(&current).await {
        tracing::error!(error = %e, "Failed to save server state");
    }
}

/// Remove a single server entry from persisted state by server_id.
pub async fn remove_server_entry(server_id: &Uuid) {
    let mut current = load_state().await.unwrap_or_default();
    current.servers.retain(|s| s.server_id != *server_id);
    if let Err(e) = save_state(&current).await {
        tracing::error!(error = %e, "Failed to save server state after removal");
    }
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

// ---------------------------------------------------------------------------
// Phase 70: Multi-server relay tunnel manager + per-server config
// ---------------------------------------------------------------------------

use std::sync::atomic::AtomicU64;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Per-server relay config — sent from backend via RelayConfigSync or
/// used internally by RelayManager.
#[derive(Debug, Clone)]
pub struct RelayServerConfig {
    pub server_id: Uuid,
    pub subdomain: String,
    pub public_port: u16,
    pub local_mc_addr: String,
    pub gateway_url: String,
    pub token: String,
    pub region: String,
    pub agent_public_ip: String,
    pub loader: Option<String>,
}

/// Handle to a running per-server relay tunnel.
pub struct RelayClientHandle {
    pub cancel: CancellationToken,
    pub join: tokio::task::JoinHandle<()>,
    pub control_tx: tokio::sync::mpsc::UnboundedSender<serde_json::Value>,
    pub bytes_transferred: Arc<AtomicU64>,
    pub tunnel_start: Instant,
    /// Config snapshot for diff comparison in set_servers().
    pub subdomain: String,
    pub public_port: u16,
    pub local_mc_addr: String,
    /// I-07: detect token rotation — tunnel is restarted when token changes.
    pub token: String,
}

/// Multi-server relay tunnel manager.
/// Singleton accessed via `relay_manager()`.
pub struct RelayManager {
    cancel: CancellationToken,
    pub(crate) servers: RwLock<HashMap<Uuid, RelayClientHandle>>,
    /// Last known desired config per server (from RelayConfigSync),
    /// used by connect_server() to start single tunnels on demand.
    configs: RwLock<HashMap<Uuid, RelayServerConfig>>,
}

impl RelayManager {
    fn new() -> Self {
        Self {
            cancel: CancellationToken::new(),
            servers: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// Start or update tunnels to match the desired config list (diff-based).
    /// Starts tunnels for new servers, stops tunnels for removed ones, and
    /// restarts tunnels with changed config.
    pub async fn set_servers(&self, configs: Vec<RelayServerConfig>) {
        use std::collections::HashSet;

        // Keep a snapshot of the desired config so connect_server() can
        // start single tunnels later without a full RelayConfigSync.
        {
            let mut snapshot = self.configs.write().await;
            snapshot.clear();
            for cfg in &configs {
                snapshot.insert(cfg.server_id, cfg.clone());
            }
        }

        let new_ids: HashSet<Uuid> = configs.iter().map(|s| s.server_id).collect();
        let mut to_start: Vec<RelayServerConfig> = Vec::new();

        // Phase 1: stop removed / config-changed tunnels
        {
            let mut servers = self.servers.write().await;
            let current_ids: HashSet<Uuid> = servers.keys().copied().collect();

            for removed_id in current_ids.difference(&new_ids) {
                if let Some(handle) = servers.remove(removed_id) {
                    tracing::info!("RelayManager: stopping tunnel for server_id={}", removed_id);
                    handle.cancel.cancel();
                }
            }

            for new_cfg in &configs {
                if let Some(existing) = servers.get(&new_cfg.server_id) {
                    let changed = existing.subdomain != new_cfg.subdomain
                        || existing.public_port != new_cfg.public_port
                        || existing.local_mc_addr != new_cfg.local_mc_addr
                        || existing.token != new_cfg.token;  // I-07: detect token rotation
                    if changed {
                        if existing.token != new_cfg.token {
                            tracing::info!(
                                "RelayManager: token changed for server_id={}, restarting tunnel",
                                new_cfg.server_id
                            );
                        } else {
                            tracing::info!(
                                "RelayManager: config changed for server_id={}, restarting",
                                new_cfg.server_id
                            );
                        }
                        if let Some(handle) = servers.remove(&new_cfg.server_id) {
                            handle.cancel.cancel();
                            to_start.push(new_cfg.clone());
                        }
                    }
                } else {
                    to_start.push(new_cfg.clone());
                }
            }
        }

        // Phase 2: start new / restarted tunnels
        for cfg in to_start {
            self.start_tunnel(cfg).await;
        }
    }

    /// Spawn a relay tunnel for a single server config. Idempotent: no-op
    /// (and keeps the existing handle) if a tunnel for `server_id` is
    /// already running with the same config.
    pub async fn start_tunnel(&self, cfg: RelayServerConfig) {
        if cfg.token.is_empty() || cfg.gateway_url.is_empty() {
            tracing::warn!(
                server_id = %cfg.server_id,
                "RelayManager: skipping tunnel — missing token or gateway_url"
            );
            return;
        }
        {
            let servers = self.servers.read().await;
            if let Some(existing) = servers.get(&cfg.server_id) {
                let same = existing.subdomain == cfg.subdomain
                    && existing.public_port == cfg.public_port
                    && existing.local_mc_addr == cfg.local_mc_addr
                    && existing.token == cfg.token;
                if same {
                    tracing::info!(
                        "RelayManager: tunnel already active for server_id={}, no-op",
                        cfg.server_id
                    );
                    return;
                }
            }
        }
        let child_cancel = self.cancel.child_token();
        let cancel_for_task = child_cancel.clone();
        let config_for_task = cfg.clone();
        let join = tokio::spawn(async move {
            crate::handlers::relay_client::run_relay_client(config_for_task, cancel_for_task).await;
        });
        let handle = RelayClientHandle {
            cancel: child_cancel,
            join,
            control_tx: tokio::sync::mpsc::unbounded_channel::<serde_json::Value>().0,
            bytes_transferred: Arc::new(AtomicU64::new(0)),
            tunnel_start: Instant::now(),
            subdomain: cfg.subdomain.clone(),
            public_port: cfg.public_port,
            local_mc_addr: cfg.local_mc_addr.clone(),
            token: cfg.token.clone(),
        };
        self.servers.write().await.insert(cfg.server_id, handle);
        tracing::info!("RelayManager: started tunnel for server_id={}", cfg.server_id);
    }

    /// Open a relay tunnel for a single server, using the most recent
    /// config snapshot received via RelayConfigSync. Idempotent: no-op if
    /// the tunnel is already running. No-op (with a warning) if no config
    /// is known for the server yet.
    pub async fn connect_server(&self, server_id: &Uuid) {
        let cfg = self.configs.read().await.get(server_id).cloned();
        match cfg {
            Some(cfg) => self.start_tunnel(cfg).await,
            None => {
                tracing::warn!(
                    "RelayManager: connect_server — no known config for server_id={}",
                    server_id
                );
            }
        }
    }

    /// Stop a single server tunnel.
    pub async fn stop_server(&self, server_id: &Uuid) {
        let handle = self.servers.write().await.remove(server_id);
        if let Some(h) = handle {
            tracing::info!("RelayManager: stopping tunnel for server_id={}", server_id);
            h.cancel.cancel();
        }
    }

    /// Stop all tunnels.
    pub async fn stop_all(&self) {
        tracing::info!("RelayManager: stop_all — cancelling parent token");
        self.cancel.cancel();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        self.servers.write().await.clear();
        self.configs.write().await.clear();
        tracing::info!("RelayManager: all tunnels stopped");
    }

    /// Send a heartbeat on the specified server's control stream.
    pub async fn send_heartbeat(
        &self,
        server_id: &Uuid,
    ) -> Result<serde_json::Value, String> {
        use serde_json::json;

        let servers = self.servers.read().await;
        if let Some(handle) = servers.get(server_id) {
            let uptime = handle.tunnel_start.elapsed().as_secs();
            let payload = json!({
                "type": "tunnel_heartbeat",
                "server_id": server_id,
                "tunnel_uptime_secs": uptime,
            });
            if handle.control_tx.send(payload).is_err() {
                return Err("RelayManager: control_tx closed for server".into());
            }
            Ok(json!({
                "action": "heartbeat",
                "status": "sent",
                "server_id": server_id,
                "tunnel_uptime_secs": uptime,
            }))
        } else {
            Ok(json!({
                "action": "heartbeat",
                "status": "no_tunnel_for_server",
                "server_id": server_id,
            }))
        }
    }

    /// List active server IDs and their uptime.
    pub async fn active_servers(&self) -> Vec<(Uuid, u64, std::time::Duration)> {
        let servers = self.servers.read().await;
        servers
            .iter()
            .map(|(id, h)| (*id, h.bytes_transferred.load(std::sync::atomic::Ordering::Relaxed), h.tunnel_start.elapsed()))
            .collect()
    }
}

static RELAY_MANAGER: OnceLock<RelayManager> = OnceLock::new();

/// Access the global RelayManager singleton.
pub fn relay_manager() -> &'static RelayManager {
    RELAY_MANAGER.get_or_init(RelayManager::new)
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

    fn test_cfg(server_id: Uuid, subdomain: &str) -> RelayServerConfig {
        RelayServerConfig {
            server_id,
            subdomain: subdomain.to_string(),
            public_port: 25565,
            local_mc_addr: "127.0.0.1:25565".to_string(),
            gateway_url: "http://127.0.0.1:1".to_string(),
            token: "test-token".to_string(),
            region: "test".to_string(),
            agent_public_ip: "127.0.0.1".to_string(),
            loader: None,
        }
    }

    #[tokio::test]
    async fn test_set_servers_diff_behavior() {
        let mgr = RelayManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        // Initial config: server A → tunnel starts
        mgr.set_servers(vec![test_cfg(a, "aaa")]).await;
        assert_eq!(mgr.servers.read().await.len(), 1);
        assert!(mgr.servers.read().await.contains_key(&a));

        // Grow: add server B → B auto-connects, A untouched
        mgr.set_servers(vec![test_cfg(a, "aaa"), test_cfg(b, "bbb")]).await;
        assert_eq!(mgr.servers.read().await.len(), 2);
        assert!(mgr.servers.read().await.contains_key(&a));
        assert!(mgr.servers.read().await.contains_key(&b));

        // Shrink: remove A → A tunnel torn down, B stays (no mass teardown)
        mgr.set_servers(vec![test_cfg(b, "bbb")]).await;
        assert_eq!(mgr.servers.read().await.len(), 1);
        assert!(!mgr.servers.read().await.contains_key(&a));
        assert!(mgr.servers.read().await.contains_key(&b));

        // Same config again → no churn
        mgr.set_servers(vec![test_cfg(b, "bbb")]).await;
        assert_eq!(mgr.servers.read().await.len(), 1);
    }

    #[tokio::test]
    async fn test_connect_server_uses_snapshot_and_is_idempotent() {
        let mgr = RelayManager::new();
        let a = Uuid::new_v4();

        // No config known yet → no-op, no crash
        mgr.connect_server(&a).await;
        assert_eq!(mgr.servers.read().await.len(), 0);

        // Config sync establishes snapshot
        mgr.set_servers(vec![test_cfg(a, "aaa")]).await;
        assert_eq!(mgr.servers.read().await.len(), 1);

        // connect_server starts from snapshot → no-op (already active)
        mgr.connect_server(&a).await;
        assert_eq!(mgr.servers.read().await.len(), 1);

        // Tunnel is torn down; connect_server can restart it from snapshot
        mgr.stop_server(&a).await;
        assert_eq!(mgr.servers.read().await.len(), 0);
        mgr.connect_server(&a).await;
        assert_eq!(mgr.servers.read().await.len(), 1);
        assert!(mgr.servers.read().await.contains_key(&a));
    }
}