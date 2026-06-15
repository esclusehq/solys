# Phase 70: Auto-fetch relay config via WS — Pattern Map

**Mapped:** 2026-06-09
**Files analyzed:** 7
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/state.rs` | config/model | CRUD | Current `RelayConfig` + `OnceCell` / `RwLock` patterns (lines 140-186) | exact |
| `src/main.rs` (lines 397-447) | utility/startup | request-response | Current `bootstrap_relay_client()` (lines 399-447) | exact |
| `src/agent_connection.rs` | controller | event-driven | `BackendMessage::DnsConfig` variant + handler (lines 126-143, 800-816) | exact |
| `src/handlers/relay_client.rs` | service | CRUD | Existing `connect()` (lines 169-204) + `disconnect()` (lines 207-230) | role-match |
| `api/src/presentation/ws/node_protocol.rs` | model/protocol | request-response | `NodeMessage::DnsConfig` variant (lines 138-156) | exact |
| `api/src/presentation/handlers/node_ws_handler.rs` | controller | event-driven | DNS config replay after RegisterAck (lines 248-298) + `push_all_servers()` (lines 300-314) | exact |
| `api/src/application/services/relay_service.rs` | service | CRUD | Existing `push_all_servers()` (lines 290-320) | exact |

## Pattern Assignments

### `src/state.rs` (config/model, CRUD)

**Analog:** Current `RelayConfig` struct + `OnceCell` pattern (lines 140-186)

**Current `RelayConfig` struct + `OnceCell` pattern** (lines 140-186):
```rust
#[derive(Debug, Clone)]
pub struct RelayConfig {
    pub gateway_url: String,
    pub token: String,
    pub agent_public_ip: String,
    pub region: String,
    pub dns_api_token: Option<String>,
    pub dns_zone_id: Option<String>,
}

static RELAY_CONFIG: OnceCell<Arc<RelayConfig>> = OnceCell::const_new();

pub fn set_relay_config(cfg: RelayConfig) {
    let _ = RELAY_CONFIG.set(Arc::new(cfg));
}

pub fn relay_config() -> Option<Arc<RelayConfig>> {
    RELAY_CONFIG.get().cloned()
}
```

**Imports pattern** (lines 1-6 of state.rs):
```rust
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::OnceCell;
use tracing::info;
use uuid::Uuid;
```

**Phase 70 target pattern** — add alongside existing (RESEARCH.md lines 375-423):
```rust
/// Immutable global relay config from env/TOML.
#[derive(Debug, Clone)]
pub struct GlobalRelayConfig {
    pub gateway_url: String,
    pub region: String,
    pub dns_api_token: Option<String>,
    pub dns_zone_id: Option<String>,
    pub agent_public_ip: String,
}

/// Dynamic relay session state from WS push.
#[derive(Debug, Clone, Default)]
pub struct RelaySessionState {
    pub relay_token: String,
    pub servers: Vec<ServerRelayInfo>,
}

#[derive(Debug, Clone)]
pub struct ServerRelayInfo {
    pub server_id: Uuid,
    pub subdomain: String,
    pub local_mc_addr: String,
    pub public_port: u16,
}

static GLOBAL_RELAY_CONFIG: OnceCell<Arc<GlobalRelayConfig>> = OnceCell::const_new();
static RELAY_SESSION_STATE: RwLock<Option<RelaySessionState>> = RwLock::const_new(None);

pub fn set_global_relay_config(cfg: GlobalRelayConfig) {
    let _ = GLOBAL_RELAY_CONFIG.set(Arc::new(cfg));
}

pub fn global_relay_config() -> Option<Arc<GlobalRelayConfig>> {
    GLOBAL_RELAY_CONFIG.get().cloned()
}

pub async fn set_relay_session_state(state: RelaySessionState) {
    let mut guard = RELAY_SESSION_STATE.write().await;
    *guard = Some(state);
}

pub async fn relay_session_state() -> Option<RelaySessionState> {
    RELAY_SESSION_STATE.read().await.clone()
}
```

**Key pattern:** Keep existing `RelayConfig` + `RELAY_CONFIG` for backward compat. Add new structs alongside. The `OnceCell<Arc<...>>` pattern is already proven (line 174). The `RwLock::const_new(None)` pattern matches the `Option` wrapper, letting callers check `is_some()` before reading.

---

### `src/main.rs` lines 397-447 (utility/startup, request-response)

**Analog:** Current `bootstrap_relay_client()` (lines 399-447)

**Current pattern** (lines 399-447):
```rust
async fn bootstrap_relay_client(
    _config: &agent_config::AgentConfig,
    _shutdown: Arc<AtomicBool>,
) -> Result<()> {
    let token = match std::env::var("AGENT_RELAY_TOKEN").ok() {
        Some(t) if !t.is_empty() => t,
        _ => {
            info!("[RELAY] No AGENT_RELAY_TOKEN set; RelayClient not started");
            return Ok(());
        }
    };
    let gateway_url = std::env::var("AGENT_RELAY_GATEWAY_URL")
        .unwrap_or_else(|_| "wss://relay.esluce.com/relay/tunnel".to_string());
    let region = std::env::var("AGENT_RELAY_REGION")
        .unwrap_or_else(|_| "ap-southeast-1".to_string());
    let dns_api_token = std::env::var("AGENT_RELAY_DNS_API_TOKEN").ok();
    let dns_zone_id = std::env::var("AGENT_RELAY_DNS_ZONE_ID").ok();
    let agent_public_ip = match crate::handlers::dns_watch::detect_public_ip().await {
        Ok(ip) => ip,
        Err(_) => "0.0.0.0".to_string(),
    };
    let relay_cfg = state::RelayConfig {
        gateway_url: gateway_url.clone(),
        token: token.clone(),
        agent_public_ip,
        region,
        dns_api_token,
        dns_zone_id,
    };
    state::set_relay_config(relay_cfg);
    info!(
        "[RELAY] Shared relay config set (token {}..., gateway={})",
        &token[..token.len().min(8)],
        gateway_url,
    );
    Ok(())
}
```

**Phase 70 target pattern** (RESEARCH.md lines 503-552):
- Always load global config from env into `GlobalRelayConfig` (OnceCell)
- Only load `RelayConfig` (legacy) if `AGENT_RELAY_TOKEN` env var is set
- Info log "waiting for WS push" when no token

**Imports needed** (from main.rs top, roughly line 30-40): `use crate::state;` already exists.

---

### `src/agent_connection.rs` (controller, event-driven)

**Analog:** `BackendMessage::DnsConfig` variant (lines 126-143) + handler (lines 800-816)

**`BackendMessage` enum pattern** (lines 90-144):
```rust
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum BackendMessage {
    #[serde(rename = "register_ack")]
    RegisterAck { node_id: Uuid, ... },
    // ... other variants ...
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
        #[serde(default)]
        extra_subdomains: Vec<String>,
    },
}
```

**DnsConfig handler pattern** (lines 800-816):
```rust
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
```

**Phase 70 target pattern** — add new variant + handler following same pattern:

New enum variant (after line 143, before closing `}`):
```rust
    #[serde(rename = "relay_config_sync")]
    RelayConfigSync {
        relay_token: String,
        gateway_url: String,
        region: String,
        servers: Vec<ServerRelayInfo>,
    },
}
// Also add the ServerRelayInfo struct at module level:
#[derive(Debug, Clone, serde::Deserialize)]
struct ServerRelayInfo {
    server_id: Uuid,
    subdomain: String,
    local_mc_addr: String,
    public_port: u16,
}
```

New handler arm (following `DnsConfig` pattern, after line 816, before `BackendMessage::Ping`):
```rust
BackendMessage::RelayConfigSync { relay_token, gateway_url, region, servers } => {
    info!(
        "RelayConfigSync received: token={}..., gateway={}, {} servers",
        &relay_token[..relay_token.len().min(8)],
        gateway_url,
        servers.len(),
    );
    let session = state::RelaySessionState {
        relay_token,
        servers: servers.iter().map(|s| state::ServerRelayInfo {
            server_id: s.server_id,
            subdomain: s.subdomain.clone(),
            local_mc_addr: s.local_mc_addr.clone(),
            public_port: s.public_port,
        }).collect(),
    };
    let _ = state::set_relay_session_state(session).await;
    if let Err(e) = crate::handlers::relay_client::apply_relay_config(servers).await {
        warn!("RelayConfigSync apply failed: {} — existing tunnels continue", e);
    }
}
```

**Key pattern:** Same structure as `DnsConfig` handler — destructure message, convert to internal domain types, store in state via `RwLock`, log at info level. Error handling is non-fatal (warn log only, no return/break from loop).

---

### `src/handlers/relay_client.rs` (service, CRUD)

**Analog:** Existing `connect()` (lines 169-204) + `disconnect()` (lines 207-230)

**`connect()` pattern** (lines 169-204):
```rust
pub async fn connect(server_id: Uuid, per_server_cfg: PerServerRelayConfig) -> Result<serde_json::Value> {
    let rt = runtime();
    let mut tunnels = rt.tunnels.write().await;

    // D-06: Replace existing tunnel if one exists
    if let Some(existing) = tunnels.remove(&server_id) {
        info!("Replacing existing tunnel for server_id={}", server_id);
        existing.cancel.cancel();
    }

    let child_cancel = rt.shutdown.child_token();
    let config_clone = per_server_cfg.clone();
    let parent_shutdown = rt.shutdown.clone();
    let handle = tokio::spawn(async move {
        run_relay_client(config_clone, parent_shutdown).await;
    });

    let psr = PerServerRuntime {
        cancel: child_cancel,
        join: Mutex::new(Some(handle)),
        control_tx: Mutex::new(None),
        bytes_transferred: Arc::new(AtomicU64::new(0)),
        tunnel_start: Mutex::new(None),
        config: per_server_cfg,
    };
    tunnels.insert(server_id, psr);
    info!(%server_id, "PerServer tunnel: connect started");
    Ok(json!({ "action": "connect", "status": "started", "server_id": server_id }))
}
```

**`disconnect()` pattern** (lines 207-230):
```rust
pub async fn disconnect(server_id: Uuid) -> Result<serde_json::Value> {
    let mut tunnels = runtime().tunnels.write().await;
    if let Some(psr) = tunnels.remove(&server_id) {
        info!("Disconnecting tunnel for server_id={}", server_id);
        psr.cancel.cancel();
        let shared_cfg = state::relay_config();
        if let Some(cfg) = shared_cfg {
            dispatch_remove_cname_record(&cfg, &psr.config).await;
        }
        Ok(json!({
            "action": "disconnect",
            "status": "stopped",
            "server_id": server_id,
        }))
    } else {
        info!("disconnect: no active tunnel for server_id={} (already gone)", server_id);
        Ok(json!({
            "action": "disconnect",
            "status": "already_disconnected",
            "server_id": server_id,
        }))
    }
}
```

**Phase 70 `apply_relay_config()` target pattern** (RESEARCH.md lines 238-273):
```rust
pub async fn apply_relay_config(new_servers: Vec<ServerRelayInfo>) -> Result<()> {
    let mut tunnels = RELAY_RUNTIME.tunnels.write().await;
    let new_ids: HashSet<Uuid> = new_servers.iter().map(|s| s.server_id).collect();
    let current_ids: HashSet<Uuid> = tunnels.keys().copied().collect();

    // Cancel removed tunnels
    for removed_id in current_ids.difference(&new_ids) {
        if let Some(psr) = tunnels.remove(removed_id) {
            info!("RelayConfigSync: stopping tunnel {}", removed_id);
            psr.cancel.cancel();
            // dispatch_remove_cname_record would need PerServerRelayConfig
            // which is owned by psr — extract before dropping
        }
    }

    // Start new tunnels
    for server in &new_servers {
        if !current_ids.contains(&server.server_id) {
            let per_server_cfg = PerServerRelayConfig {
                server_id: server.server_id,
                subdomain: server.subdomain.clone(),
                public_port: server.public_port,
                local_mc_addr: server.local_mc_addr.clone(),
                dns_record_id: None,
            };
            // Drop lock before calling connect() which also acquires the lock
            drop(tunnels);
            relay_client::connect(server.server_id, per_server_cfg).await?;
            tunnels = RELAY_RUNTIME.tunnels.write().await;
        }
    }
    Ok(())
}
```

**Imports to add:** `use std::collections::HashSet;` (alongside existing `HashMap` import at line 58).

**Key pattern:** Write lock → compute diffs with `HashSet` → drop lock before calling `connect()`/`disconnect()` to avoid deadlock (anti-pattern from Pitfall 2 in RESEARCH.md lines 297-302). The `CancellationToken` hierarchy already handles tunnel lifecycle shutdown (proven by Phase 69).

---

### `api/src/presentation/ws/node_protocol.rs` (model/protocol, request-response)

**Analog:** `NodeMessage::DnsConfig` variant (lines 138-156)

**`DnsConfig` variant pattern** (lines 137-156):
```rust
    // Backend -> Agent (broadcast)
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
        #[serde(default)]
        extra_subdomains: Vec<String>,
    },
```

**Phase 70 target** — after `TunnelCloseAck` at line 233, before closing `}` at line 260:
```rust
    // ===== Phase 70: Auto-fetch relay config via WS =====

    /// Backend pushes the complete relay configuration for this node.
    /// Sent after RegisterAck (D-05) and on server create (D-08).
    /// Agent replaces its entire RelaySessionState on receipt (D-02).
    #[serde(rename = "relay_config_sync")]
    RelayConfigSync {
        relay_token: String,
        gateway_url: String,
        region: String,
        servers: Vec<ServerRelayInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRelayInfo {
    pub server_id: Uuid,
    pub subdomain: String,
    pub local_mc_addr: String,
    pub public_port: u16,
}
```

**Key pattern:** Same as `DnsConfig` — backend-pushed, full-state-replace message. The `#[serde(rename = "...")]` attribute on the variant enables tagged JSON deserialization (the outer `#[serde(tag = "type")]` on the enum). Struct definition goes outside the enum, below `DeployConfig` at line 348.

**Imports** (lines 1-3): `serde::{Deserialize, Serialize}`, `Uuid` already imported. No new imports needed.

---

### `api/src/presentation/handlers/node_ws_handler.rs` (controller, event-driven)

**Analog:** DNS config replay after RegisterAck (lines 248-298) + Phase 69 `push_all_servers()` (lines 300-314)

**DnsConfig replay pattern** (lines 248-298):
```rust
match container
    .settings_repository
    .get_cloudflare_config()
    .await
{
    Ok(cf) if cf.is_configured() => {
        let extra_subdomains = collect_per_server_subdomains(
            container.server_repository.as_ref(),
        )
        .await;
        let dns_msg = NodeMessage::DnsConfig {
            api_token: cf.api_token.clone(),
            zone_id: cf.zone_id.clone(),
            zone_name: cf.zone_name.clone(),
            wildcard_domain: cf.wildcard_domain.clone(),
            auto_refresh: cf.auto_refresh,
            refresh_interval_secs: cf.refresh_interval_secs,
            public_ip: None,
            subdomain: cf.subdomain.clone(),
            extra_subdomains,
        };
        match manager
            .send_to_node(&node_id_val, &dns_msg)
            .await
        {
            Ok(()) => tracing::info!(...),
            Err(e) => tracing::warn!(...),
        }
    }
    Ok(_) => {
        tracing::debug!("... skipping replay ...");
    }
    Err(e) => {
        tracing::warn!(...);
    }
}
```

**Phase 69 `push_all_servers()` pattern** (lines 300-314, currently sits after DnsConfig replay):
```rust
if let Err(e) = container
    .relay_service
    .push_all_servers(&node_id_val)
    .await
{
    tracing::warn!(
        "[RELAY] Failed to push servers for node {}: {}",
        node_id_val,
        e
    );
}
```

**Phase 70 target** — replace `push_all_servers()` with new `push_relay_config()` call in the same code position (after line 298, after DnsConfig replay, replacing lines 300-314):
```rust
// Phase 70: Push RelayConfigSync for this node so the agent knows
// its relay token, gateway URL, region, and per-server config.
// Replaces Phase 69's per-server push_all_servers() call.
if let Err(e) = container
    .relay_service
    .push_relay_config(&node_id_val)
    .await
{
    tracing::warn!(
        "[RELAY] Failed to push RelayConfigSync for node {}: {}",
        node_id_val,
        e
    );
}
```

**Key pattern:** Same error-handling approach as DnsConfig replay — non-fatal `warn!` log on failure. Same position in the Register handler: right after DnsConfig replay and before the closing brace of the Register match arm. Same `manager.send_to_node()` pattern used internally by `relay_service.push_relay_config()`.

---

### `api/src/application/services/relay_service.rs` (service, CRUD)

**Analog:** Existing `push_all_servers()` (lines 290-320)

**`push_all_servers()` pattern** (lines 290-320):
```rust
pub async fn push_all_servers(&self, node_id: &Uuid) -> Result<()> {
    let servers = self.server_repository.find_by_node_id(node_id).await?;
    for server in servers {
        let subdomain = match &server.subdomain {
            Some(s) => s.clone(),
            None => {
                let sd = self.server_repository.generate_subdomain(&server.id);
                self.server_repository
                    .set_subdomain(&server.id, &sd)
                    .await?;
                sd
            }
        };
        let msg = NodeMessage::RelayConnect {
            server_id: server.id,
            subdomain,
            public_port: server.port as u16,
            local_mc_addr: format!("127.0.0.1:{}", server.port),
        };
        if let Err(e) = self.node_connection_manager.send_to_node(node_id, &msg).await {
            tracing::warn!(
                "[RELAY] Failed to push RelayConnect for server {} to node {}: {} (skipping)",
                server.id,
                node_id,
                e
            );
        }
    }
    Ok(())
}
```

**Phase 70 `push_relay_config()` target pattern** (RESEARCH.md lines 426-474):
```rust
/// Phase 70: Push the complete relay config for a node as a single
/// RelayConfigSync message. Called after RegisterAck (D-05) and
/// on server create (D-08).
pub async fn push_relay_config(&self, node_id: &Uuid) -> Result<()> {
    let node = self.node_repository
        .find_by_id(node_id)
        .await?
        .ok_or_else(|| anyhow!("Node {} not found", node_id))?;

    let relay_token = node.relay_token
        .ok_or_else(|| anyhow!("Node {} has no relay_token", node_id))?
        .to_string();

    let servers = self.server_repository
        .find_by_node_id(node_id)
        .await?;

    let server_infos: Vec<ServerRelayInfo> = servers.iter().map(|s| {
        ServerRelayInfo {
            server_id: s.id,
            subdomain: s.subdomain.clone().unwrap_or_default(),
            local_mc_addr: format!("127.0.0.1:{}", s.port),
            public_port: s.port as u16,
        }
    }).collect();

    let gateway_url = std::env::var("RELAY_GATEWAY_URL")
        .unwrap_or_else(|_| "wss://relay.esluce.com/relay/tunnel".to_string());
    let region = std::env::var("RELAY_REGION")
        .unwrap_or_else(|_| "ap-southeast-1".to_string());

    let msg = NodeMessage::RelayConfigSync {
        relay_token,
        gateway_url,
        region,
        servers: server_infos,
    };

    self.node_connection_manager.send_to_node(node_id, &msg).await
        .map_err(|e| anyhow!("Failed to send RelayConfigSync: {}", e))
}
```

**Import to add** (alongside existing `node_protocol::NodeMessage` at line 27):
```rust
use crate::presentation::ws::node_protocol::ServerRelayInfo;
```

**Key patterns:**
- Same repository pattern as existing methods: `self.node_repository.find_by_id()`, `self.server_repository.find_by_node_id()`
- Same `self.node_connection_manager.send_to_node()` for WS message delivery
- Error handling: returns `Result<()>` with `anyhow!` errors, following the existing `push_all_servers()` pattern
- `gateway_url` and `region` from env vars with hardcoded defaults, matching the current agent-side defaults

---

## Shared Patterns

### Full-state Push After RegisterAck (PROVEN PATTERN)
**Source:** `node_ws_handler.rs` lines 248-298 (DnsConfig replay)
**Apply to:** `node_ws_handler.rs` + `relay_service.rs`

The pattern is:
1. After `RegisterAck` is sent to the agent (line 222-224)
2. Read persisted config from repository (`settings_repository` for DNS, `node_repository` + `server_repository` for relay)
3. Construct a backend-pushed message (`NodeMessage::DnsConfig` / `NodeMessage::RelayConfigSync`)
4. Send via `manager.send_to_node()`
5. Log successes at `info!`, failures at `warn!` (non-fatal — agent can be reconfigured later)

### OnceCell for Immutable Config, RwLock for Mutable State
**Source:** `state.rs` lines 140-186 + existing `OnceCell`/`RwLock` usage
**Apply to:** `state.rs`

- `OnceCell<Arc<T>>` for truly immutable config set once at startup (`GlobalRelayConfig`)
- `RwLock<Option<T>>` for state that is dynamically replaced (`RelaySessionState`)
- Setter/getter functions encapsulate the synchronization primitive

### RwLock Write Guard Drop Before Async Calls
**Source:** Pitfall 2 (RESEARCH.md lines 297-302)
**Apply to:** `relay_client.rs` — `apply_relay_config()`

```rust
// BAD: Holding write lock across connect() causes deadlock
let tunnels = lock.write().await;
connect().await;  // DEADLOCK — connect() acquires same lock

// GOOD: Drop outer lock before calling inner functions
let tunnels = lock.write().await;
// ... read state ...
drop(tunnels);  // Release before async call
connect().await;
tunnels = lock.write().await;  // Re-acquire after
```

### WS Message Tagged JSON Deserialization
**Source:** `agent_connection.rs:90-144`, `node_protocol.rs` (entire file)
**Apply to:** `agent_connection.rs` + `node_protocol.rs`

All WS messages use `#[serde(tag = "type")]` on the enum, with `#[serde(rename = "...")]` on each variant. The `type` field in the JSON determines which variant to deserialize. New variants follow this exact pattern.

### Server Ownership Verification for Relay Operations
**Source:** `relay_service.rs` lines 90-101
**Apply to:** `relay_service.rs::push_relay_config()` — already reads from `node_repository` which is authorized context

```
pub async fn verify_server_ownership(&self, node_id: &Uuid, server_id: &Uuid) -> Result<bool> {
    let server = self.server_repository.find_by_id(server_id).await?
        .ok_or_else(|| anyhow!("Server {} not found", server_id))?;
    Ok(server.node_id == Some(*node_id))
}
```

### CancellationToken Hierarchy for Tunnel Lifecycle
**Source:** `relay_client.rs` lines 142-149, 180, 207-211
**Apply to:** `relay_client.rs::apply_relay_config()`

Parent `RelayRuntime.shutdown` token → child tokens per `PerServerRuntime`. `parent.cancel()` cascades to all children. `disconnect()` cancels individual child token.

---

## No Analog Found

All 7 files have close analogs in the codebase. No file requires RESEARCH.md-only reference.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| *(none)* | | | All files have exact or role-match analogs |

## Metadata

**Analog search scope:** `src/`, `api/src/application/services/`, `api/src/presentation/ws/`, `api/src/presentation/handlers/`
**Files scanned:** 12 (state.rs, main.rs, agent_connection.rs, relay_client.rs, node_protocol.rs, node_ws_handler.rs, relay_service.rs, handlers/mod.rs, node.rs, server.rs, node_connection_manager.rs, container.rs)
**Pattern extraction date:** 2026-06-09
