# Phase 70: Auto-fetch relay config via WS - Research

**Researched:** 2026-06-09
**Domain:** Agent↔Backend WS protocol, relay config delivery
**Confidence:** HIGH

## Summary

Phase 70 replaces manual env-var relay configuration (`AGENT_RELAY_TOKEN`, `AGENT_RELAY_SERVER_ID`, `AGENT_RELAY_SUBDOMAIN`, etc.) with a dynamic push from backend to agent over the existing WebSocket connection. After Phase 70, the agent ships with zero relay env vars — the user provides only `AGENT_API_KEY` and the backend sends all relay configuration through WS after registration.

The key architectural change is splitting the current monolithic `RelayConfig` (`OnceCell`) into two sources: **GlobalRelayConfig** (`OnceCell`, from env/TOML — rarely changes) and **RelaySessionState** (`RwLock`, from WS push — dynamically replaced). The `RelayConfigSync` message follows the same full-state-replace pattern already proven by `DnsConfig` replay in `node_ws_handler.rs:248-298`.

**Primary recommendation:** Add `BackendMessage::RelayConfigSync` variant on agent side, add `NodeMessage::RelayConfigSync` variant on backend side, implement `apply_relay_config()` in `relay_client.rs` that diffs and hot-updates tunnels, and modify `bootstrap_relay_client()` to only load global config from env.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Relay config storage split | Agent state | — | `GlobalRelayConfig` (OnceCell, immutable) + `RelaySessionState` (RwLock, dynamic). Agent owns both. |
| Relay config push timing | Backend handler | — | Backend pushes immediately after RegisterAck in `node_ws_handler.rs:300` slot, same pattern as DNS replay |
| Config diff & hot update | Agent relay_client | — | `apply_relay_config()` in `relay_client.rs` diffs current tunnels vs new servers, stops/starts atomically under RwLock |
| Server create → config sync | Backend handler | — | Backend sends fresh `RelayConfigSync` (full state replace) when server is created on an already-connected node |
| Server delete → config sync | Backend handler | — | Backend sends existing `relay.disconnect` task (Phase 69 D-08), NOT a full config sync |
| Reconnect config recovery | Agent + Backend | — | On reconnect, Register → RegisterAck → `RelayConfigSync` push. Agent diff vs existing tunnels (should be empty on fresh reconnect) |
| Backward compat (env fallback) | Agent bootstrap | — | If `AGENT_RELAY_TOKEN` env var is set, use as before (fallback for existing deployments) |

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01 (Config storage split):** Split into two sources:
  - **GlobalRelayConfig** (`OnceCell`) — immutable, from env/TOML: `gateway_url`, `region`, `dns_api_token`, `dns_zone_id`. Rarely changes.
  - **RelaySessionState** (`RwLock`) — dynamic, from WS push: `relay_token` + `servers: Vec<ServerRelayConfig>`. Replaced atomically on every push.
  - After Phase 70, `AGENT_RELAY_TOKEN` and `AGENT_RELAY_SERVER_ID` env vars are no longer needed.
  - **Backward compat:** If `AGENT_RELAY_TOKEN` still set, use as fallback (agent can operate without WS push).
- **D-02 (Replace semantics):** Full state replace on every `RelayConfigSync` push. Agent replaces entire `RelaySessionState.servers` vec — no incremental diff on the storage layer.
- **D-03 (Startup flow):** Wait for WS push. If no `AGENT_RELAY_TOKEN` env var, `relay_client` does not start at bootstrap. It starts only after first `RelayConfigSync` arrives from backend via WS. For backward compat: if env var is set, start immediately (existing behavior).
- **D-04 (Hot update):** Diff-based hot update. When new config arrives:
  1. Acquire write lock on `RelaySessionState`
  2. Cancel tunnels for servers no longer in the new list
  3. Start tunnels for new servers
  4. Update tunnels for existing servers if config changed (stagger jitter per Phase 69 D-17)
  - Atomic under the RwLock write guard.
- **D-05 (Push timing):** Backend sends `RelayConfigSync` immediately after `RegisterAck`, in the same Register handler flow (`node_ws_handler.rs:99-299`). Same code path for first connect and reconnect.
- **D-06 (Message type):** Single `NodeMessage::RelayConfigSync` variant. Not split into separate token/server messages.
- **D-07 (Message shape):**
  ```rust
  #[serde(rename = "relay_config_sync")]
  RelayConfigSync {
      relay_token: String,
      gateway_url: String,
      region: String,
      servers: Vec<ServerRelayInfo>,
  }

  ServerRelayInfo {
      server_id: Uuid,
      subdomain: String,
      local_mc_addr: String,   // "127.0.0.1:<port>"
      public_port: u16,
  }
  ```
  Backend owns authoritative `gateway_url` and `region` — can override env defaults.
- **D-08 (Server create while connected):** Fresh `RelayConfigSync` (full state replace). Agent diffs and starts tunnel for the new server. Same message type as initial config — no separate `relay.connect` task for this path.
- **D-09 (Server delete while connected):** Existing `relay.disconnect` task (Phase 69 D-08). Targeted, proven. No full sync needed for deletes.
- **D-10 (Reconnect flow):** Same as initial connect — Register handler → `RegisterAck` → `RelayConfigSync`. No separate reconnect detection needed.
- **D-11 (Push failure):** Non-critical — log and retry on next WS connect. Agent with existing config continues working. No retry loop, no blocking. If agent has no config (fresh install), `relay_client` simply doesn't start until config arrives.

### the agent's Discretion

- Naming of new structs (GlobalRelayConfig, RelaySessionState, ServerRelayInfo)
- File-level organization of `apply_relay_config()` (inline in relay_client.rs vs separate module)
- Whether to log per-server tunnel diff operations (recommend: yes, at info level for debugging)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| (implicit) | Split relay config into GlobalRelayConfig + RelaySessionState | Current `RelayConfig` struct at `state.rs:140-157` has all fields. Split pattern proven by DnsConfig + existing OnceCell/RwLock usage. |
| (implicit) | Add RelayConfigSync to agent BackendMessage enum | `BackendMessage` at `agent_connection.rs:90-144`. Currently 6 variants. Missing variants for all Phase 68/69 backend→agent messages (relay_connect, relay_disconnect, mode_override_change, tunnel_close_ack). Phase 70 adds ONLY `RelayConfigSync`. |
| (implicit) | Backend pushes RelayConfigSync after RegisterAck | Slot at `node_ws_handler.rs:300` after DNS replay (line 298). Pattern proven by DNS config replay at lines 248-298. |
| (implicit) | Implement apply_relay_config() for diff-based hot update | `relay_client.rs::connect()` at line 169 + `disconnect()` at line 207. Both are per-server, proven by Phase 69. `apply_relay_config()` composes them. |
| (implicit) | Conditional bootstrap: skip if no AGENT_RELAY_TOKEN | `main.rs:399-447` `bootstrap_relay_client()`. Already returns early if token absent. Phase 70 changes: only load global fields from env. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1 (features = ["full"]) | Async runtime | Project standard, used everywhere |
| serde / serde_json | 1 | JSON serialization | WS protocol uses JSON with `#[serde(tag = "type")]` discriminator |
| tokio::sync::OnceCell | (tokio built-in) | Immutable global config | Current pattern for `RELAY_CONFIG`, `DOCKER_GLOBAL` |
| tokio::sync::RwLock | (tokio built-in) | Mutable shared state | Standard pattern for `HashMap<Uuid, PerServerRuntime>` in relay_client |
| tracing | 0.1 | Structured logging | Project standard for all instrumentation |
| uuid | 1 | Server/node IDs | Already used throughout the codebase |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio-tungstenite | 0.26 | WebSocket client | Agent↔backend WS connection (existing) |
| tokio-util | 0.7 | CancellationToken | Shutdown cascading (existing Phase 69 pattern) |

### Alternatives Considered
No meaningful alternatives — all decisions locked in CONTEXT.md.

**Installation:**
No new dependencies. All crates already in Cargo.toml.

**Version verification:**
```bash
# Already in Cargo.toml — no new packages needed
cargo metadata --format-version=1 --no-deps | jq '.packages[] | select(.name == "tokio" or .name == "serde" or .name == "uuid") | {name, version}'
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        AGENT STARTUP                                │
│                                                                     │
│  main.rs: read env vars                                             │
│    → GlobalRelayConfig { gateway_url, region, dns_* }               │
│      stored in OnceCell at state.rs                                 │
│    → If no AGENT_RELAY_TOKEN, skip relay bootstrap                  │
│      (relay_client doesn't start at boot)                           │
│    → If AGENT_RELAY_TOKEN set, legacy: start tunnels immediately    │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    AGENT → BACKEND WS CONNECT                       │
│                                                                     │
│  agent_connection.rs:                                               │
│  1. Connect WS with AGENT_API_KEY query param                       │
│  2. Send Register { name, ip, capabilities, ... }                   │
│  3. Receive RegisterAck { node_id }                                 │
│  4. Backend sends:                                                  │
│     ├── DnsConfig (existing, Phase 51)                              │
│     └── [NEW] RelayConfigSync {                                     │
│           relay_token, gateway_url, region,                         │
│           servers: [{server_id, subdomain, local_mc_addr, ...}]     │
│         }                                                           │
│                                                                     │
│  5. Agent stores RelaySessionState (RwLock) →                       │
│     calls apply_relay_config() to diff and start tunnels            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   apply_relay_config() FLOW                         │
│                                                                     │
│  1. Acquire write lock on RelaySessionState                         │
│  2. Build server_id set from new servers vec                        │
│  3. For each server_id in current tunnels NOT in new set:           │
│     → relay_client::disconnect(server_id)                           │
│     → Cancel child token, remove from HashMap                       │
│  4. For each server in new set NOT in current tunnels:              │
│     → relay_client::connect(server_id, config)                      │
│     → Start new PerServerRuntime, insert into HashMap               │
│  5. For servers in BOTH (update if config changed):                 │
│     → relay_client::disconnect() then connect()                     │
│       (atomically replace — already proven by D-06)                 │
│  6. Release lock                                                    │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   SERVER CREATE (while connected)                   │
│                                                                     │
│  Backend handler (server create endpoint):                          │
│  1. Create server in DB                                             │
│  2. Send RelayConfigSync (full state replace)                       │
│  3. Agent diffs: new server in list → starts tunnel                │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   SERVER DELETE (while connected)                   │
│                                                                     │
│  Backend handler (server delete endpoint):                          │
│  1. Delete server in DB                                             │
│  2. Send relay.disconnect task (Phase 69 D-08)                      │
│  3. Agent closes tunnel, removes PerServerRuntime                   │
│  (No full RelayConfigSync needed for delete)                        │
└─────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

No structural changes — all modifications are within existing files:

```
api/src/presentation/ws/node_protocol.rs      # +RelayConfigSync variant
api/src/presentation/handlers/node_ws_handler.rs  # +RelayConfigSync push after RegisterAck
api/src/application/services/relay_service.rs     # +push_relay_config() method

src/state.rs                                    # Split RelayConfig → GlobalRelayConfig + RelaySessionState
src/main.rs                                     # Conditional bootstrap (skip if no AGENT_RELAY_TOKEN)
src/agent_connection.rs                         # +BackendMessage::RelayConfigSync handling
src/handlers/relay_client.rs                    # +apply_relay_config() diff-based hot update
```

### Pattern 1: Full-state push after RegisterAck
**What:** After agent registration, backend pushes a complete state snapshot (DnsConfig, RelayConfigSync) as a single message. Agent replaces its local state entirely — no incremental updates.
**When to use:** For any configuration that the backend owns authoritatively and the agent should fully replace.
**Example:**
```rust
// Source: node_ws_handler.rs:248-298 (proven by DnsConfig replay)
// Phase 70 extends with RelayConfigSync push at line 300:
match container.settings_repository.get_relay_config().await {
    Ok(config) => {
        let servers = collect_server_relay_infos(
            container.server_repository.as_ref(), node_id_val
        ).await;
        let msg = NodeMessage::RelayConfigSync {
            relay_token: config.relay_token.to_string(),
            gateway_url: config.gateway_url,
            region: config.region,
            servers,
        };
        if let Err(e) = manager.send_to_node(&node_id_val, &msg).await {
            tracing::warn!("[RELAY] Failed to push RelayConfigSync: {}", e);
        }
    }
    Err(e) => tracing::warn!("[RELAY] Failed to load relay config: {}", e),
}
```

### Pattern 2: Hot update via diff on state replace
**What:** When new config arrives, compare current running tunnels against desired state, apply diffs atomically.
**When to use:** Whenever a batch config update arrives that may add, remove, or modify active resources.
**Example:**
```rust
// Source: relay_client.rs apply_relay_config() (to be implemented)
pub async fn apply_relay_config(new_servers: Vec<ServerRelayInfo>) {
    let mut tunnels = RELAY_RUNTIME.tunnels.write().await;

    let new_ids: HashSet<Uuid> = new_servers.iter().map(|s| s.server_id).collect();

    // Cancel removed tunnels
    let current_ids: HashSet<Uuid> = tunnels.keys().copied().collect();
    for removed_id in current_ids.difference(&new_ids) {
        if let Some(psr) = tunnels.remove(removed_id) {
            info!("RelayConfigSync: stopping tunnel {}", removed_id);
            psr.cancel.cancel();
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
                dns_record_id: None,  // Phase 68: sent via task but not persisted
            };
            // relay_client::connect takes write lock on tunnels
            // So we drop our lock first, then acquire again inside
            drop(tunnels);
            relay_client::connect(server.server_id, per_server_cfg).await?;
            tunnels = RELAY_RUNTIME.tunnels.write().await;
        }
    }
    Ok(())
}
```

### Anti-Patterns to Avoid
- **Holding RwLock write guard across async connect() calls:** `connect()` spawns a tokio task and inserts into the HashMap. Acquire write lock, check/update, release before calling connect. Re-acquire after to handle updated state. The lock should not be held across the `connect()` await point.
- **Sending both RelayConfigSync AND individual RelayConnect messages:** After Phase 70, backend should send either `RelayConfigSync` (on initial connect / server create) or `relay.disconnect` task (on server delete). Sending both would cause duplicate tunnels.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Config diffing logic | Custom set-diff algorithm | `std::collections::HashSet` difference/union | Standard library, proven, no edge cases |
| RwLock write guard management | Roll-your-own lock ordering | `drop(tunnels)` before async calls | Prevents deadlocks when calling connect() which also takes the lock |
| CancellationToken hierarchy | Manual shutdown flags | `tokio_util::sync::CancellationToken` | Already proven in Phase 69 (parent-child cascade). Don't reinvent. |

**Key insight:** All the building blocks for Phase 70 already exist in the codebase. The `DnsConfig` replay pattern is the template for `RelayConfigSync`. The `CancellationToken` hierarchy already handles tunnel lifecycle. The `PerServerRuntime` HashMap already supports add/remove. Phase 70 is about wiring these existing patterns together.

## Common Pitfalls

### Pitfall 1: Deserialization Failure on Unknown Message Types
**What goes wrong:** Agent silently drops backend messages it can't parse. When backend sends `RelayConfigSync`, if the agent's `BackendMessage` enum doesn't have a matching variant, `serde_json::from_str` returns `Err` and the message is silently dropped (agent_connection.rs `if let Ok(backend_msg) = ...` has no else branch).
**Why it happens:** `BackendMessage` enum currently only has 6 variants (agent_connection.rs:90-144). Unknown `"type"` values cause parse errors that are silently swallowed.
**How to avoid:** Add the `BackendMessage::RelayConfigSync` variant to the agent-side enum BEFORE deploying the backend push. Use feature flags or coordinated deploy.
**Warning signs:** Phase 68/69 backend→agent messages (relay_connect, relay_disconnect, mode_override_change, tunnel_close_ack) are also silently dropped. If those work, it means they're sent as `ExecuteCommand` tasks, not as dedicated messages.

### Pitfall 2: Deadlock on RwLock Write Guard
**What goes wrong:** `apply_relay_config()` acquires write lock on `RELAY_SESSION_STATE` or `tunnels`, then calls `connect()` or `disconnect()` which also acquires the same lock. Mutex/RwLock is not reentrant in tokio.
**Why it happens:** Nested RwLock acquisition. The outer function holds the lock while calling inner functions that also take the lock.
**How to avoid:** Drop the outer lock before calling inner functions that need the same lock. Pattern: `let guard = lock.write().await; /* read state */ drop(guard); connect().await;`
**Warning signs:** Deadlocked tokio tasks that never complete.

### Pitfall 3: Backend Sends Both RelayConnect AND RelayConfigSync
**What goes wrong:** After Phase 70, the backend's Register handler currently sends individual `RelayConnect` messages per server (line 300-314 in `node_ws_handler.rs`). Adding `RelayConfigSync` without removing the old `push_all_servers` call would send both, causing duplicate tunnels.
**Why it happens:** The existing `RelayConnect` push per server (via `push_all_servers`) runs after DNS config replay. Adding `RelayConfigSync` alongside creates two parallel relay startup paths.
**How to avoid:** Replace `push_all_servers()` with a single `push_relay_config()` that sends `RelayConfigSync`. Remove or disable the per-server `RelayConnect` send in the Register handler.
**Warning signs:** Agent logs show both "RelayConnect" and "RelayConfigSync" messages arriving.

### Pitfall 4: Race Between RelayConfigSync and Server Lifecycle Events
**What goes wrong:** If a server is created/deleted while `RelayConfigSync` is in-flight, the agent might get out of sync (start a tunnel for a just-deleted server, or miss a just-created server).
**Why it happens:** Multiple backend events (RelayConfigSync + relay.disconnect) can cross in the WebSocket channel.
**How to avoid:** The `relay.disconnect` task (Phase 69 D-08) is idempotent — sending it for a non-existent tunnel is a no-op. D-08 ensures delete is always handled independently. For creates: the `RelayConfigSync` for the create event is sent after the DB write, so it's the authoritative source. Ordering: if a delete task crosses with a sync, the tunnel is stopped (delete), then the sync's diff won't re-create it because the server is gone from the list.
**Warning signs:** Transient tunnel for a deleted server that auto-resolves on next sync.

## Code Examples

### Adding RelayConfigSync to BackendMessage (agent_connection.rs)
```rust
// Source: Phase 70 D-07, D-06 — new variant in agent's BackendMessage enum
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum BackendMessage {
    // ... existing variants ...

    #[serde(rename = "relay_config_sync")]
    RelayConfigSync {
        relay_token: String,
        gateway_url: String,
        region: String,
        servers: Vec<ServerRelayInfo>,
    },
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ServerRelayInfo {
    server_id: Uuid,
    subdomain: String,
    local_mc_addr: String,
    public_port: u16,
}
```

### Handling RelayConfigSync in agent message loop (agent_connection.rs)
```rust
// Source: agent_connection.rs message loop at ~line 800-816 (pattern from DnsConfig handler)
BackendMessage::RelayConfigSync { relay_token, gateway_url, region, servers } => {
    info!(
        "RelayConfigSync received: token={}..., gateway={}, {} servers",
        &relay_token[..relay_token.len().min(8)],
        gateway_url,
        servers.len(),
    );

    // Store in RelaySessionState
    let session = state::RelaySessionState {
        relay_token,
        servers: servers.iter().map(|s| state::ServerRelayInfo {
            server_id: s.server_id,
            subdomain: s.subdomain.clone(),
            local_mc_addr: s.local_mc_addr.clone(),
            public_port: s.public_port,
        }).collect(),
    };
    let _ = state::set_relay_session_state(session);

    // Apply diff-based hot update
    if let Err(e) = crate::handlers::relay_client::apply_relay_config(servers).await {
        warn!("RelayConfigSync apply failed: {} — existing tunnels continue", e);
    }
}
```

### GlobalRelayConfig + RelaySessionState (state.rs)
```rust
// Source: Phase 70 D-01 — split from current RelayConfig

/// Immutable global relay config from env/TOML.
/// Set once at startup, never changes at runtime.
#[derive(Debug, Clone)]
pub struct GlobalRelayConfig {
    pub gateway_url: String,
    pub region: String,
    pub dns_api_token: Option<String>,
    pub dns_zone_id: Option<String>,
    pub agent_public_ip: String,  // auto-detected at startup
}

/// Dynamic relay session state from WS push.
/// Fully replaced on every RelayConfigSync.
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

### Backend push_relay_config (relay_service.rs)
```rust
// Source: Phase 70 D-05, D-07 — new method on RelayService

use crate::presentation::ws::node_protocol::ServerRelayInfo;

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

    // gateway_url and region from env or default — backend owns authoritative values
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

### Add NodeMessage::RelayConfigSync variant (node_protocol.rs)
```rust
// Source: Phase 70 D-06, D-07 — inserted after TunnelCloseAck (line 233)

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

### Modified bootstrap_relay_client (main.rs)
```rust
// Source: Phase 70 D-03 — only read global config from env

async fn bootstrap_relay_client(
    _config: &agent_config::AgentConfig,
    _shutdown: Arc<AtomicBool>,
) -> Result<()> {
    // Global config is always loaded (from env/TOML)
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

    let global_cfg = state::GlobalRelayConfig {
        gateway_url,
        region,
        dns_api_token,
        dns_zone_id,
        agent_public_ip,
    };
    state::set_global_relay_config(global_cfg);

    // Backward compat: if AGENT_RELAY_TOKEN is set, also set legacy RelayConfig
    // so existing deployments continue to work without WS push.
    if let Some(token) = std::env::var("AGENT_RELAY_TOKEN").ok().filter(|t| !t.is_empty()) {
        info!("[RELAY] AGENT_RELAY_TOKEN set — using legacy bootstrap (Phase 70 WS push will replace this)");
        let legacy_cfg = state::RelayConfig {
            gateway_url: gateway_url.clone(),
            token,
            agent_public_ip,
            region: region.clone(),
            dns_api_token,
            dns_zone_id,
        };
        state::set_relay_config(legacy_cfg);
    } else {
        info!("[RELAY] No AGENT_RELAY_TOKEN — waiting for WS push (Phase 70)");
        // relay_client won't start until first RelayConfigSync arrives
    }

    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `RelayConfig` OnceCell (all fields from env) | `GlobalRelayConfig` OnceCell (static) + `RelaySessionState` RwLock (dynamic) | Phase 70 | Agent no longer needs relay_token at startup |
| `AGENT_RELAY_TOKEN`, `AGENT_RELAY_SERVER_ID` env vars | WS push from backend | Phase 70 | Zero relay env vars needed |
| `push_all_servers()` sends N × `RelayConnect` messages | Single `RelayConfigSync` message with all servers | Phase 70 | One message vs N messages; agent diffs on receipt |
| Per-server `relay.connect` task on reconnect | `RelayConfigSync` → agent diff → start tunnels | Phase 70 | Eliminates task dispatch latency per server |

**Deprecated/outdated:**
- `RelayConfig` struct in `state.rs` (lines 140-157) — replaced by `GlobalRelayConfig` + `RelaySessionState`
- `relay_config()` function in `state.rs` (line 184) — replaced by `global_relay_config()` + `relay_session_state()`
- Per-server config fields in task payload (`relay.connect` task) — still used for backward compat; Phase 70 config comes from both global config + session state

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | All Phase 68/69 backend→agent messages (relay_connect, relay_disconnect, mode_override_change, tunnel_close_ack) are sent/processed via ExecuteCommand tasks, not via dedicated NodeMessage variants. | Architecture | If some are sent as dedicated NodeMessage variants, they're silently dropped by the agent. This doesn't block Phase 70 but means tunnels won't start/stop correctly. |
| A2 | The gateway_url and region defaults in the backend (`RELAY_GATEWAY_URL` env var, `RELAY_REGION` env var) don't exist yet and need to be added or the values hardcoded. | Code Examples | If these env vars already exist in the backend deployment, the default fallbacks in the code example might not match. |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions

1. **How does the agent currently process `RelayConnect`/`RelayDisconnect` from backend?**
   - What we know: Backend sends `NodeMessage::RelayConnect` via WS (proven in `relay_service.rs:304-309`). Agent's `BackendMessage` enum has no matching variant.
   - What's unclear: Whether these messages are silently dropped (agent ignores them) or the agent handles them through the `ExecuteCommand` task dispatch path (backend sends `ExecuteCommand { command: "relay.connect" }` instead).
   - **Recommendation:** Check if the existing Phase 69 per-server tunnels actually work. If yes, they must arrive via `ExecuteCommand` tasks and the `RelayConnect` NodeMessage variant is a parallel path that Phase 70 should either remove or align. If Phase 69 tunnels don't work (silently dropped), Phase 70 must fix this.
   - **Resolution path:** Grep backend code for where `ExecuteCommand` with `command: "relay.connect"` is sent. If found, the task dispatch path is proven. If not found, the backend only sends `NodeMessage::RelayConnect` and the agent's parse fails silently.

2. **Should the backend read `gateway_url` and `region` from its own env vars?**
   - What we know: D-07 says "Backend owns authoritative gateway_url and region — can override env defaults." The agent's defaults are hardcoded in `main.rs:411-414`.
   - What's unclear: Whether these values should come from backend env vars (new `RELAY_GATEWAY_URL`, `RELAY_REGION` vars) or be hardcoded in the backend.
   - **Recommendation:** Read from env vars with hardcoded defaults matching the agent's current defaults. This ensures consistency and allows ops override without code changes.

3. **What happens to `agent_public_ip` in the backward compat path?**
   - What we know: Current `RelayConfig.agent_public_ip` is auto-detected at startup. In the new design, it stays in `GlobalRelayConfig`.
   - What's unclear: The backward compat path (when `AGENT_RELAY_TOKEN` is set) still sets the old `RelayConfig` via `state::set_relay_config()`. But the new code path (WS push) won't set `RelayConfig` at all. Will `run_relay_client()` still read `state::relay_config()`?
   - **Recommendation:** Modify `run_relay_client()` to read from `GlobalRelayConfig` + `RelaySessionState` instead of the old `RelayConfig`. The backward compat path should also set both global and session state for code consistency.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Compile agent + backend | ✓ | rustc 1.95.0 | — |
| tokio | Async runtime | ✓ | 1.x (Cargo.toml) | — |
| serde | JSON serialization | ✓ | 1.x (Cargo.toml) | — |

**Missing dependencies with no fallback:** None — all dependencies are already in Cargo.toml.

**Missing dependencies with fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + standard Rust test patterns |
| Config file | Cargo.toml (no separate test config) |
| Quick run command | `cargo test -p solys --lib -- handlers::relay_client::tests 2>&1 | tail -20` |
| Full suite command | `cargo test --workspace 2>&1 | tail -20` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | Config split: GlobalRelayConfig OnceCell + RelaySessionState RwLock | unit | `cargo test -p solys -- state::tests 2>&1` | ❌ New tests needed |
| D-02 | Full state replace on RelayConfigSync | unit | `cargo test -p solys -- handlers::relay_client::tests 2>&1` | ❌ New tests needed |
| D-04 | apply_relay_config diff: add/remove/update tunnels | unit | `cargo test -p solys -- handlers::relay_client::tests 2>&1` | ❌ New tests needed |
| D-07 | Message serialization round-trip | unit | `cargo test -p solys -- state::tests 2>&1` | ❌ New tests needed |
| D-11 | Push failure is non-fatal (no blocking) | verification | Manual code review | N/A |

### Sampling Rate
- **Per task commit:** `cargo check --workspace 2>&1 | tail -10` (compilation check)
- **Per wave merge:** `cargo test --workspace 2>&1 | tail -20` (full test suite)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/state.rs` — Add `GlobalRelayConfig` struct, `RelaySessionState` struct, `ServerRelayInfo` struct, setter/getter functions. Unit tests for OnceCell + RwLock access patterns.
- [ ] `src/handlers/relay_client.rs` — Add `apply_relay_config()` function. Unit tests for diff logic (add, remove, update scenarios).
- [ ] `src/agent_connection.rs` — Add `BackendMessage::RelayConfigSync` variant. Unit test for JSON deserialization.
- [ ] `api/src/presentation/ws/node_protocol.rs` — Add `NodeMessage::RelayConfigSync` variant. Unit test for JSON serialization round-trip.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Relay token is pushed from backend (already authenticated via API key). No new auth. |
| V5 Input Validation | Yes | `RelayConfigSync` message parsing uses serde with defined types. UUIDs and strings validated at parse time. |
| V8 Data Protection | No | Config data is already in memory (OnceCell/RwLock). No new persistence. |

### Known Threat Patterns for {Rust WS protocol}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Config injection via malformed RelayConfigSync | Tampering | serde strict deserialization rejects unknown fields. UUID validation on server_id. |
| Race condition on concurrent config updates | Spoofing | RwLock write guard ensures atomic state replacement. D-02: full replace, no partial updates. |
| Replay of stale RelayConfigSync | Spoofing | D-10: reconnect flow sends fresh config on every connect. No caching of old configs. |

## Sources

### Primary (HIGH confidence)
- [codebase] `agent_connection.rs:90-144` — Current `BackendMessage` enum confirming only 6 variants. Missing all Phase 68/69 backend→agent messages.
- [codebase] `src/state.rs:140-186` — Current `RelayConfig` struct and `OnceCell` pattern. Proof of split target.
- [codebase] `src/handlers/relay_client.rs` — Per-server tunnel lifecycle functions. `connect()` and `disconnect()` proven by Phase 69.
- [codebase] `api/src/presentation/ws/node_protocol.rs` — Current `NodeMessage` enum. Slot after `TunnelCloseAck` (line 233) for new variant.
- [codebase] `api/src/presentation/handlers/node_ws_handler.rs:248-298` — `DnsConfig` replay pattern. Template for `RelayConfigSync` push.
- [codebase] `api/src/application/services/relay_service.rs` — `push_all_servers()` at line 290-320. Proof of existing per-server push pattern.
- [codebase] `src/main.rs:399-447` — Current `bootstrap_relay_client()`. Already returns early if no token (D-03 base).

### Secondary (MEDIUM confidence)
- [codebase] `api/src/domain/entities/node.rs:50` — `relay_token: Option<Uuid>` on Node entity. Backend reads this for `RelayConfigSync.relay_token`.
- [codebase] `src/main.rs:331` — `bootstrap_relay_client()` call at startup. Placement after DNS watcher, before agent connection.

### Tertiary (LOW confidence)
None — all claims verified through codebase analysis.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All crates already in Cargo.toml, patterns proven by existing code.
- Architecture: HIGH - Designs match existing patterns (DnsConfig replay, CancellationToken, HashMap tunnels).
- Pitfalls: HIGH - Identified by tracing the actual code paths (silent drop, RwLock deadlock, duplicate messages).

**Research date:** 2026-06-09
**Valid until:** 2026-07-09 (stable project — no fast-moving dependencies in this phase)
