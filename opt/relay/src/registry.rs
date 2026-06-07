use dashmap::DashMap;
use std::sync::atomic::AtomicU64;
use std::time::Instant;
use uuid::Uuid;
use tokio_yamux::Control;

pub struct TunnelHandle {
    pub server_id: Uuid,
    pub subdomain: String,                // e.g. "abc12345"
    pub agent_public_ip: String,          // for audit; not used for routing
    pub last_heartbeat: AtomicU64,        // unix seconds
    pub yamux_control: tokio::sync::Mutex<Option<Control>>,
    pub started_at: Instant,
    pub bytes_in: AtomicU64,
    pub bytes_out: AtomicU64,
}

#[derive(Clone, Default)]
pub struct Registry {
    // Primary routing index — Handshake-packet subdomain → server_id (BLOCKER 1 fix)
    pub by_subdomain: std::sync::Arc<DashMap<String, Uuid>>,
    // Secondary index for fast lookup from the heartbeat watcher
    pub by_server_id: std::sync::Arc<DashMap<Uuid, std::sync::Arc<TunnelHandle>>>,
    // NOTE: There is intentionally NO `by_agent_ip` map. Vanilla Minecraft Java
    // clients do not send SNI, and the player source IP is unrelated to the
    // server's agent public IP (the whole point of the relay is to bridge
    // CGNAT agents to remote players). Routing is by subdomain only.
}

impl Registry {
    pub fn new() -> Self {
        Self {
            by_subdomain: std::sync::Arc::new(DashMap::new()),
            by_server_id: std::sync::Arc::new(DashMap::new()),
        }
    }

    /// Register a new tunnel. If a tunnel for the same server_id already exists,
    /// drops the older one (D-21). The agent's `subdomain` is what the gateway
    /// uses for player routing, so the same subdomain can only map to one
    /// server at a time.
    pub fn register(&self, handle: std::sync::Arc<TunnelHandle>) -> Result<(), RegistryError> {
        // Check existing tunnel for this server_id (D-21)
        if let Some(old) = self.by_server_id.insert(handle.server_id, handle.clone()) {
            tracing::info!("[REGISTRY] Replacing existing tunnel for server_id={}", handle.server_id);
            // Free the old subdomain (in case it differs)
            self.by_subdomain.remove(&old.subdomain);
        }
        // Enforce 1:1 subdomain → server_id mapping
        if let Some(existing_server_id) = self.by_subdomain.insert(handle.subdomain.clone(), handle.server_id) {
            if existing_server_id != handle.server_id {
                // Subdomain collision: another server is using this subdomain.
                // Rollback: restore the original mapping (existing_server_id) so the
                // existing owner is preserved. The new handle is rejected and is not
                // inserted into by_subdomain.
                self.by_subdomain.insert(handle.subdomain.clone(), existing_server_id);
                return Err(RegistryError::SubdomainInUse);
            }
        }
        Ok(())
    }

    /// Look up server_id by subdomain (the Handshake-routing path used by player.rs)
    pub fn lookup_by_subdomain(&self, subdomain: &str) -> Option<Uuid> {
        self.by_subdomain.get(subdomain).map(|e| *e.value())
    }

    pub fn get(&self, server_id: &Uuid) -> Option<std::sync::Arc<TunnelHandle>> {
        self.by_server_id.get(server_id).map(|e| e.value().clone())
    }

    pub fn unregister(&self, server_id: &Uuid) {
        if let Some((_, handle)) = self.by_server_id.remove(server_id) {
            self.by_subdomain.remove(&handle.subdomain);
        }
    }

    pub fn mark_stale(&self, server_id: &Uuid) {
        if let Some(_handle) = self.by_server_id.get(server_id) {
            tracing::warn!("[REGISTRY] Marking tunnel stale: server_id={}", server_id);
        }
        self.unregister(server_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = std::sync::Arc<TunnelHandle>> + '_ {
        self.by_server_id.iter().map(|e| e.value().clone())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Subdomain already in use by another server")]
    SubdomainInUse,
}
