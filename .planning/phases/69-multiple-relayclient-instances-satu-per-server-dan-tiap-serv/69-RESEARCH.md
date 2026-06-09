# Phase 69: Multiple RelayClient Instances (Per-Server Tunnels with Unique Subdomains) - Research

**Researched:** 2026-06-09
**Domain:** Agent relay tunnel architecture refactoring (Rust, tokio, yamux, WebSocket, AWS Route 53, Minecraft Handshake protocol)
**Confidence:** HIGH — all claims verified against existing codebase, Phase 68 code patterns, and CONTEXT.md user decisions

## Summary

Phase 69 refactors the agent's relay tunnel architecture from a **single global WebSocket + yamux session** (Phase 68 D-02: one tunnel carries all servers on a node) to **`N` independent per-server tunnel instances**. Each server on a node gets its own WebSocket connection to `relay.esluce.net`, its own yamux session, its own reconnect loop with staggered heartbeat, and — critically — its own **unique subdomain** on `*.play.esluce.net` (e.g., `a3f8b.play.esluce.net`). The gateway already routes by subdomain via Minecraft Handshake parsing (Phase 68 `player.rs`), and the registry already supports `HashMap<ServerId, TunnelHandle>` (Phase 68 `registry.rs`), so the gateway changes are minimal — primarily in `auth.rs` (allow 1:N relay_token→server_id) and `tunnel.rs` (N concurrent WS from same agent IP). The bulk of the work is on the **agent side**: replacing `OnceLock<RelayRuntime>` with `RwLock<HashMap<ServerId, PerServerRuntime>>`, splitting `RelayConfig` into shared (global env) + per-server (task payload) fields, and making `connect/disconnect/heartbeat` work per-server. The backend gains a `servers.subdomain` column, generates the short hex hash on server create, and includes it in the `relay.connect` task payload.

**Primary recommendation:** Implement in 3 waves: (1) Backend — `servers.subdomain` migration + subdomain generation + relay.connect payload includes per-server config; (2) Agent — refactor `relay_client.rs` from global to per-server, split `RelayConfig`, update `relay.rs` dispatch; (3) Gateway — auth.rs 1:N fix + tunnel.rs N-WS-from-same-IP handling. The registry, player routing, and heartbeat watcher need zero changes.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Implementation Decisions (Locked)

- **D-01 (Storage):** `tokio::sync::RwLock<HashMap<ServerId, PerServerRuntime>>` — concurrent reads (heartbeats, player streams) with exclusive writes (add/remove server)
- **D-02 (Add/remove servers):** Via backend task dispatch — `relay.connect` and `relay.disconnect` tasks with `server_id` in payload. Agent adds/removes HashMap entries on arrival. No agent-side reconciliation.
- **D-03 (PerServerRuntime struct):** Full per-server struct containing CancellationToken (child of parent), JoinHandle, ControlTx (mpsc channel), BytesCounter (AtomicU64), TunnelStart (Instant), and per-server RelayConfig (server_id, subdomain, token, gateway_url, local_mc_addr, public_port)
- **D-04 (Shutdown cascade):** Parent CancellationToken held at RelayRuntime level. Each PerServerRuntime gets a child_token(). On agent shutdown: parent.cancel() fires all children, all reconnect loops exit naturally, then HashMap is cleared.
- **D-05 (Task routing):** server_id carried in task.payload for relay.connect, relay.disconnect, relay.heartbeat tasks
- **D-06 (Duplicate tunnel):** Replace existing — cancel old tunnel atomically, start new one
- **D-07 (Agent reconnect):** On agent→backend WS reconnect, backend pushes relay.connect for all active servers the agent owns. Backend is the source of truth; no local persistence.
- **D-08 (Server deletion):** Backend sends relay.disconnect when server is deleted. Agent cancels tunnel, removes from HashMap.
- **D-09 (Format):** Short hex hash of server UUID (e.g., a3f8b). Predictable, collision-resistant, short enough for chat sharing.
- **D-10 (Generation timing):** Backend generates subdomain on server create. Stored in servers.subdomain column. Passed to agent in relay.connect task payload.
- **D-11 (DNS):** Wildcard *.play.esluce.net → CNAME/ALIAS → NLB. No per-record DNS management needed.
- **D-12 (Gateway routing):** Gateway parses Minecraft Handshake packet's server address field to extract subdomain. Maps subdomain to server_id, looks up tunnel in registry, opens yamux stream.
- **D-13 (Config delivery):** Backend includes full per-server config in relay.connect task payload. No separate config message.
- **D-14 (Persistence):** No local persistence. On agent restart → backend WS reconnect → backend re-pushes relay.connect for all servers.
- **D-15 (Shared vs per-server fields):** Shared fields (gateway_url, agent_public_ip, region, dns_api_token) remain as global config from env vars. Per-server fields arrive in task payload.
- **D-16 (Max tunnels):** No hard limit per agent. Tens of servers per node is negligible overhead.
- **D-17 (Heartbeat staggering):** Random startup delay (0-10s jitter) per tunnel so heartbeats are evenly distributed.
- **D-18 (Bandwidth accounting):** Per-server AtomicU64 in each PerServerRuntime.
- **D-19 (Auth layer):** relay_token (per-node) must authorize multiple server_id values. Backend's /internal/relay/authorize endpoint validates node.owns(server_id) — no change needed at the endpoint.
- **D-20 (WS connection management):** Gateway must accept N concurrent WS connections from same agent IP (one per server), each authenticating with the same relay_token but different server_id.
- **D-21 (Registry):** No change — already HashMap<ServerId, TunnelHandle> per-server.
- **D-22 (Player routing):** No change — already routes by subdomain → server_id via Handshake parser.

### the agent's Discretion
- Exact short-hex length (recommend 5 hex chars = 1M+ namespace with negligible collision)
- Heartbeat jitter formula and exact startup delay distribution
- PerServerRuntime struct field ordering and naming
- HashMap eviction policy (if any)
- Whether to log per-server tunnel state transitions (recommend: yes)
- Whether the gateway's auth layer handles per-server WS via relay_token dedup at the TCP level (recommend: accept N connections, each authenticates independently)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Per-server tunnel lifecycle (connect/reconnect/disconnect) | Agent | — | Each server gets its own RelayClient instance; agent manages the HashMap |
| Shared tunnel config (gateway_url, token, region) | Agent (env vars) | — | `RELAY_CONFIG` OnceCell keeps global fields; per-server fields in task payload |
| PerServerRuntime struct + HashMap management | Agent | — | Replaces current `OnceLock<RelayRuntime>` with `RwLock<HashMap<ServerId, PerServerRuntime>>` |
| Subdomain generation | API/Backend | — | New `servers.subdomain` column; short hex hash generated on server create |
| Per-server config delivery to agent | API/Backend | — | `relay.connect` task payload includes server_id, subdomain, public_port, local_mc_addr |
| Gateway auth (1:N relay_token→server_id) | Relay Gateway | API/Backend (authorize endpoint) | Backend already validates `node.owns(server_id)` — gateway just stops enforcing 1:1 |
| Gateway WS connection (N from same IP) | Relay Gateway | — | tunnel.rs must accept N concurrent WS from same agent IP with same relay_token |
| Gateway subdomain → server_id routing | Relay Gateway | — | Already implemented in player.rs via Handshake parsing (Phase 68) — no change |
| Gateway tunnel registry (HashMap<ServerId, TunnelHandle>) | Relay Gateway | — | Already implemented (Phase 68 registry.rs) — no change |
| Gateway heartbeat watcher | Relay Gateway | — | Already iterates all tunnels independently — no change |
| Backend relay.connect push on agent reconnect | API/Backend | — | Backend pushes relay.connect for all active servers the agent owns |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio` | 1.x (already in agent + gateway) | Async runtime, RwLock, CancellationToken, mpsc channels | Already locked; no new deps needed for this phase |
| `tokio::sync::RwLock` | (standard) | Concurrent-read HashMap for per-server tunnels | D-01 choice: concurrent reads (heartbeats, streams) with exclusive writes (add/remove) |
| `tokio_util::sync::CancellationToken` | 0.7 (already in agent) | Parent-child shutdown cascade | Already used in Phase 68 relay_client.rs — Phase 69 extends to child_token() hierarchy |
| `serde_json` | 1.x (already) | Task payload deserialization (server_id extraction) | Standard across agent |
| `uuid` | 1.x (already) | ServerId type for HashMap key | Already used everywhere |

### No new dependencies required
Phase 69 is a **refactoring** phase — no new libraries needed. All required primitives (RwLock, HashMap, CancellationToken, mpsc channels, AtomicU64) are already in the tokio/stdlib stack used by the existing code.

## Architecture Patterns

### System Architecture Comparison

```
Phase 68 (current — single tunnel):
  Agent ──[1 WSS]───────> Gateway
        └─[1 yamux session for ALL servers]
             ├─ stream for server_1 (player A)
             ├─ stream for server_2 (player B)
             └─ stream for server_1 (player C)

Phase 69 (target — per-server tunnels):
  Agent ──[WSS for server_1]──> Gateway ──[1 yamux session for server_1]
       │                               └ streams: player A, player C
       └─[WSS for server_2]──> Gateway ──[1 yamux session for server_2]
                                        └ streams: player B
  
  Each WS connection:
    • Independent WSS to relay.esluce.net/relay/tunnel
    • Independent yamux session (new_client)
    • Independent reconnect loop (exponential backoff)
    • Independent heartbeat task (10s with 0-10s jitter)
    • Independent bytes_counter (AtomicU64)
    • Same relay_token (per-node) — gateway allows 1:N
    • Different server_id (per-server, carried in TunnelConnect JSON)
    • Different subdomain (per-server, carried in TunnelConnect JSON)
```

### Recommended Project Structure Changes

```
src/handlers/
├── relay_client.rs      # MAJOR REFACTOR: OnceLock<RelayRuntime> → RwLock<HashMap<ServerId, PerServerRuntime>>
│                         #   connect(server_id) / disconnect(server_id) / send_heartbeat(server_id)
│                         #   PerServerRuntime struct (CancelToken child, JoinHandle, ControlTx, BytesCounter, etc.)
├── relay_session.rs     # MINOR CHANGE: sig stays same; stream_from from per-server session instead of global
├── relay.rs             # MINOR CHANGE: extract server_id from task.payload, dispatch to per-server

src/state.rs             # MINOR CHANGE: RelayConfig splits — shared fields as OnceCell, per-server in task payload
                         #   Remove: server_id, subdomain, public_port, local_mc_addr (now per-server)
                         #   Keep: gateway_url, token, agent_public_ip, region, dns_api_token/zone_id

opt/relay/src/
├── auth.rs              # MINOR: remove 1:1 relay_token→server_id enforcement (if any exists)
├── tunnel.rs            # MINOR: handle N WS from same agent IP without conflict
├── registry.rs          # NO CHANGE — already HashMap<ServerId, TunnelHandle>
├── player.rs            # NO CHANGE — already routes by subdomain via Handshake
└── heartbeat.rs         # NO CHANGE — already iterates all tunnels

api/migrations/          # NEW: servers.subdomain column
api/ (backend)           # NEW: subdomain generation on server create, inclusion in relay.connect payload
```

### Pattern 1: PerServerRuntime Struct and HashMap Management

**What:** Replacement for the current global `OnceLock<RelayRuntime>` with a `RwLock<HashMap<ServerId, PerServerRuntime>>`. Each `PerServerRuntime` holds per-tunnel state with a child CancellationToken for cascade shutdown.

**When to use:** The entire relay_client.rs refactoring.

**Example:**

```rust
// Source: CONTEXT.md D-01, D-03, D-04; existing relay_client.rs lines 108-138 for reference

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Per-server runtime state — one instance per active server tunnel.
pub struct PerServerRuntime {
    /// Child of the parent RelayRuntime::shutdown token. When parent.cancel()
    /// fires, all child tokens fire, exiting all reconnect loops.
    pub cancel: CancellationToken,
    /// Join handle for the per-server run_relay_client task.
    pub join: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Channel for sending heartbeat/disconnect JSON payloads to this server's
    /// control stream. None while no tunnel is open.
    pub control_tx: Mutex<Option<tokio::sync::mpsc::UnboundedSender<serde_json::Value>>>,
    /// Per-server byte counter, updated by relay_session on each stream close.
    pub bytes_transferred: Arc<AtomicU64>,
    /// Tunnel start time, used by heartbeat task.
    pub tunnel_start: Mutex<Option<Instant>>,
    /// Per-server config from task payload (server_id, subdomain, etc.)
    pub config: PerServerRelayConfig,
}

/// Per-server portion of the relay config (arrives in task payload).
#[derive(Debug, Clone)]
pub struct PerServerRelayConfig {
    pub server_id: Uuid,
    pub subdomain: String,
    pub public_port: u16,
    pub local_mc_addr: String,
}

/// Parent runtime — holds the shutdown token that cascades to all children.
pub struct RelayRuntime {
    pub shutdown: CancellationToken,
    pub tunnels: Arc<RwLock<HashMap<Uuid, PerServerRuntime>>>,
}

static RELAY_RUNTIME: OnceLock<RelayRuntime> = OnceLock::new();

fn runtime() -> &'static RelayRuntime {
    RELAY_RUNTIME.get_or_init(|| RelayRuntime {
        shutdown: CancellationToken::new(),
        tunnels: Arc::new(RwLock::new(HashMap::new())),
    })
}
```

### Pattern 2: Per-Server Reconnect Loop with Child CancellationToken

**What:** Each server tunnel runs its own `run_relay_client` loop with a child CancellationToken. The parent token is held at `RelayRuntime` level.

**When to use:** In the new per-server `connect()` function.

**Example:**

```rust
// Source: CONTEXT.md D-04 (shutdown cascade), D-17 (heartbeat staggering);
// existing relay_client.rs lines 255-303 adapted to per-server

pub async fn run_relay_client(
    per_server_cfg: PerServerRelayConfig,
    parent_shutdown: CancellationToken,
) {
    // Create a child token so parent.cancel() cascades to this loop
    let child_shutdown = parent_shutdown.child_token();
    let mut backoff_ms: u64 = 1_000;

    // Heartbeat staggering: wait random 0-10s before first heartbeat
    // (not before the initial connect — we want fast tunnel establishment)
    let heartbeat_jitter = rand::thread_rng().gen_range(0..=10_000);

    loop {
        tokio::select! {
            _ = child_shutdown.cancelled() => {
                info!("PerServer[{}]: shutdown", per_server_cfg.server_id);
                break;
            }
            result = connect_and_run(&per_server_cfg, &child_shutdown) => {
                match result {
                    Ok(()) => {
                        backoff_ms = 1_000;
                    }
                    Err(e) => {
                        warn!("PerServer[{}]: connect failed: {}", per_server_cfg.server_id, e);
                    }
                }
            }
        }

        if child_shutdown.is_cancelled() { break; }

        let sleep_ms = backoff_with_jitter(backoff_ms);
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        backoff_ms = (backoff_ms.saturating_mul(2)).min(30_000);
    }
}
```

### Pattern 3: Per-Server connect() with Idempotent Add-Replace

**What:** `relay.connect` task handler creates a new PerServerRuntime and starts its loop. If a tunnel already exists for that server_id, the old one is cancelled first (D-06 replacement semantics).

**When to use:** In the new per-server `connect()` function called from `relay.rs` dispatch.

**Example:**

```rust
// Source: CONTEXT.md D-02, D-05, D-06; existing relay_client.rs connect() lines 152-181 adapted

pub async fn connect(server_id: Uuid, per_server_cfg: PerServerRelayConfig) -> Result<serde_json::Value> {
    let runtime = runtime();
    let mut tunnels = runtime.tunnels.write().await;

    // D-06: Replace existing tunnel if one exists
    if let Some(existing) = tunnels.remove(&server_id) {
        info!("Replacing existing tunnel for server_id={}", server_id);
        existing.cancel.cancel();  // child token fires → reconnect loop exits
        // Join handle will be dropped when PerServerRuntime drops
    }

    let child_cancel = runtime.shutdown.child_token();
    let handle = tokio::spawn(run_relay_client(per_server_cfg.clone(), runtime.shutdown.clone()));

    let psr = PerServerRuntime {
        cancel: child_cancel,
        join: Mutex::new(Some(handle)),
        control_tx: Mutex::new(None),
        bytes_transferred: Arc::new(AtomicU64::new(0)),
        tunnel_start: Mutex::new(None),
        config: per_server_cfg,
    };

    tunnels.insert(server_id, psr);

    Ok(json!({
        "action": "connect",
        "status": "started",
        "server_id": server_id,
    }))
}
```

### Pattern 4: Shutdown Cascade Using Parent-Child CancellationToken

**What:** Agent shutdown cancels the parent token, which cascades to all child tokens. Each per-server reconnect loop exits, then the HashMap is cleared.

**When to use:** In the agent's shutdown sequence and in the `disconnect_all()` function.

**Example:**

```rust
// Source: CONTEXT.md D-04 specifics block:
//   "Parent CancellationToken with child_token() per server. parent.cancel()
//    cascades to all children, then clear HashMap. Explicit parent.cancel()
//    required (CancellationToken.drop() does NOT auto-cancel)."

/// Cancel all tunnels. Called from agent shutdown sequence.
pub async fn shutdown_all() {
    let runtime = runtime();
    runtime.shutdown.cancel();              // fires all child tokens
    // Brief yield to let tasks see the cancellation
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut tunnels = runtime.tunnels.write().await;
    tunnels.clear();                        // drops all PerServerRuntime structs
}

/// Cancel a single tunnel by server_id. Called from relay.disconnect task.
pub async fn disconnect(server_id: Uuid) -> Result<serde_json::Value> {
    let runtime = runtime();
    let mut tunnels = runtime.tunnels.write().await;
    if let Some(psr) = tunnels.remove(&server_id) {
        psr.cancel.cancel();    // child token fires → reconnect loop exits
    }
    Ok(json!({ "action": "disconnect", "status": "stopped", "server_id": server_id }))
}
```

### Pattern 5: relay.rs Task Dispatch with server_id Extraction

**What:** The task dispatch shim extracts `server_id` from the incoming `task.payload` JSON and routes to the correct per-server handler.

**When to use:** In `src/handlers/relay.rs`, replacing the current global dispatch.

**Example:**

```rust
// Source: CONTEXT.md D-05; existing relay.rs adapted

pub async fn handle_relay_task(task: &Task) -> Result<serde_json::Value> {
    let task_type = task.task_type.as_str();
    match task_type {
        "relay.connect" => {
            let payload = &task.payload;
            let server_id: Uuid = payload.get("server_id")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow!("Missing server_id in relay.connect payload"))?;

            let per_server_cfg = PerServerRelayConfig {
                server_id,
                subdomain: payload.get("subdomain")
                    .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                public_port: payload.get("public_port")
                    .and_then(|v| v.as_u64()).unwrap_or(25565) as u16,
                local_mc_addr: payload.get("local_mc_addr")
                    .and_then(|v| v.as_str()).unwrap_or("127.0.0.1:25565").to_string(),
            };

            relay_client::connect(server_id, per_server_cfg).await
        }
        "relay.disconnect" => {
            let server_id: Uuid = task.payload.get("server_id")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow!("Missing server_id"))?;
            relay_client::disconnect(server_id).await
        }
        "relay.heartbeat" => {
            let server_id: Uuid = task.payload.get("server_id")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow!("Missing server_id"))?;
            relay_client::send_heartbeat(server_id).await
        }
        other => Err(anyhow!("Unknown relay task type: {}", other)),
    }
}
```

### System Architecture Diagram

```
                          Backend (api.esluce.com)
                          ├── POST /internal/relay/authorize (HMAC) — validates node.owns(server_id)
                          ├── servers.subdomain column (NEW) — generated on server create
                          └── Task dispatch: relay.connect/relay.disconnect pushed via agent WS
                                  │
                                  │ relay.connect { server_id, subdomain, ... }
                                  │ relay.disconnect { server_id }
                                  ▼
                          Agent (Solys)
                          ┌─────────────────────────────────────────────┐
                          │  relay_client.rs                            │
                          │  ┌──────────────────────────────────────┐   │
                          │  │  RelayRuntime (OnceLock)              │   │
                          │  │    shutdown: CancellationToken (parent)│   │
                          │  │    tunnels: Arc<RwLock<HashMap<       │   │
                          │  │      ServerId, PerServerRuntime       │   │
                          │  │    >>                                 │   │
                          │  │                                        │   │
                          │  │  PerServerRuntime {                    │   │
                          │  │    cancel: child_token(),              │   │
                          │  │    join: JoinHandle,                   │   │
                          │  │    control_tx: mpsc::Sender,           │   │
                          │  │    bytes_counter: AtomicU64,           │   │
                          │  │    tunnel_start: Instant,              │   │
                          │  │    config: PerServerRelayConfig        │   │
                          │  │      server_id, subdomain,             │   │
                          │  │      public_port, local_mc_addr        │   │
                          │  │  }                                     │   │
                          │  └──────────────────────────────────────┘   │
                          │                                             │
                          │  For each server:                           │
                          │    ┌── WSS ──> relay.esluce.net/tunnel      │
                          │    │   Authenticate with relay_token        │
                          │    │   (same token, different server_id)     │
                          │    │   yamux client session                  │
                          │    │   ┌── Control stream (TunnelConnect,    │
                          │    │   │   TunnelHeartbeat every 10s)        │
                          │    │   │                                     │
                          │    │   └── Inbound streams from gateway      │
                          │    │       │ (one per player, forwarded to   │
                          │    │       │  localhost:public_port)          │
                          │    └──                                     │
                          └─────────────────────────────────────────────┘
                                      │ WS connections (N per agent)
                                      ▼
                          Relay Gateway (relay.esluce.net)
                          ┌─────────────────────────────────────────────┐
                          │  tunnel.rs — accepts N WS per agent IP     │
                          │    Each WS authenticated independently      │
                          │    (auth.rs: 1:N token→server_id)          │
                          │                                             │
                          │  registry.rs (NO CHANGE)                   │
                          │    HashMap<ServerId, TunnelHandle>          │
                          │    HashMap<Subdomain, ServerId>             │
                          │                                             │
                          │  player.rs (NO CHANGE)                     │
                          │    Listens on :25565                        │
                          │    Reads MC Handshake → subdomain            │
                          │    → server_id → yamux stream                │
                          │                                             │
                          │  heartbeat.rs (NO CHANGE)                   │
                          │    Iterates all tunnels, marks stale        │
                          └─────────────────────────────────────────────┘
                                      │ player TCP :25565
                                      ▼
                          Player connects to a3f8b.play.esluce.net:25565
```

### Anti-Patterns to Avoid

- **Assuming drop() cancels child CancellationToken:** CancellationToken's `Drop` does NOT auto-cancel — it only decrements the ref count. You MUST call `parent.cancel()` explicitly for the cascade to work. (CONTEXT.md D-04 discussion log explicitly documents this.)
- **Storing JoinHandles without cleanup:** When replacing a tunnel (D-06), the old JoinHandle must be cancelled. The old PerServerRuntime's `cancel.child_token()` cancellation will cause the old join handle to exit naturally, but the handle should not be `.await`ed in the connect path (that would block). Drop the old PerServerRuntime struct after cancelling its token.
- **Global control_tx for per-server heartbeat:** The current code uses a single global `control_tx` channel. With per-server tunnels, each PerServerRuntime gets its own `control_tx` channel so heartbeats are directed to the correct control stream.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Subdomain generation | Custom hash function | `blake3::hash(server_uuid.as_bytes())[..3]` encoded in hex | Deterministic, collision-resistant, well-known. 3 bytes = 6 hex chars = 16M namespace. |
| CancellationToken hierarchy | Manual `Vec<CancellationToken>` with manual iteration | `CancellationToken::child_token()` | Tokio's CancellationToken already supports parent-child relationships natively (internal ref-counted linked list). Manual iteration is error-prone. |
| Per-server heartbeat jitter | Manual timer with fixed stagger | `rand::thread_rng().gen_range(0..=10_000)` ms sleep before first heartbeat loop tick | Simple, proven, matches existing tokio patterns. |

**Key insight:** Phase 69 is primarily a structural refactoring of existing working code — no new libraries, no new protocols, no new external dependencies. The hardest part is correctly splitting the global state machine into N independent state machines without introducing race conditions or leaky abstractions.

## Common Pitfalls

### Pitfall 1: CancellationToken Drop vs Cancel Confusion
**What goes wrong:** The developer assumes that when a `CancellationToken` is dropped (because an old `PerServerRuntime` struct is dropped), the cancellation is automatically propagated to anyone who cloned it.
**Why it happens:** Tokio's `CancellationToken` is `Arc`-based internally. `Drop` only decrements the reference count — it does NOT trigger cancellation. Only an explicit `.cancel()` call triggers it. This is explicitly documented in tokio-util docs and in CONTEXT.md D-04.
**How to avoid:** Always call `.cancel()` before dropping the token (or the struct holding it). In the shutdown logic: call `parent.cancel()` first, THEN clear the HashMap. In the reconnect path: call `old.cancel()` before removing the old entry.
**Warning signs:** Tunnel tasks keep running after the struct is removed from the HashMap.

### Pitfall 2: Shared relay_token Across Multiple WS Connections
**What goes wrong:** The gateway's `tunnel.rs` uses the `Authorization: Bearer <token>` header as a unique session identifier. When multiple WS from the same agent arrive (one per server), the gateway might interpret the second connection as a duplicate and reject it.
**Why it happens:** The current tunnel session code in `tunnel.rs` only has per-server_id dedup logic. There's no explicit handling of N connections from the same relay_token.
**How to avoid:** Ensure the gateway differentiates sessions by `server_id` (from the TunnelConnect JSON), not by `relay_token`. The `registry.rs` already keys by `server_id`, so as long as each WS carries a different `server_id` in its TunnelConnect message, there's no conflict. The auth check in `auth.rs` should validate `node.owns(server_id)` but should NOT reject multiple server_ids for the same token.
**Warning signs:** Second server's tunnel fails to register with "already connected" error.

### Pitfall 3: Heartbeat Thundering Herd
**What goes wrong:** If an agent has 10 servers, and all 10 tunnels start their heartbeat timers at the same moment, the gateway receives 10 heartbeat messages simultaneously every 10 seconds.
**Why it happens:** Without staggering, all per-server heartbeat loops tick in sync.
**How to avoid:** D-17 specifies 0-10s random jitter at startup. Add `tokio::time::sleep(Duration::from_millis(rand::thread_rng().gen_range(0..=10_000)))` before the heartbeat loop starts (but AFTER the initial tunnel connect succeeds, so tunnel establishment isn't delayed).
**Warning signs:** Gateway heartbeat logs show bursts of 10 messages at identical timestamps.

### Pitfall 4: Race on connect() While disconnect() Is In Flight
**What goes wrong:** If the backend sends `relay.disconnect` followed immediately by `relay.connect` for the same server_id (e.g., due to a server restart), the connect might find the old tunnel still in the HashMap (disconnect haven't acquired the write lock yet) or the disconnect might cancel the new tunnel.
**Why it happens:** Task dispatch is async — there's no guaranteed ordering between two tasks.
**How to avoid:** D-06 already specifies "replace existing": `connect()` cancels and removes any existing tunnel for the same server_id before inserting the new one. This is the correct atomic replacement. The `disconnect()` handler should check if the tunnel it's cancelling is still the same one (by JoinHandle comparison) to avoid cancelling a tunnel that was already replaced.
**Warning signs:** New tunnel immediately disconnects with "already cancelled" errors.

## Subdomain Strategy

### Generation Algorithm
Short hex hash of the server's UUID. The backend generates this on server create.

```
server UUID: 550e8400-e29b-41d4-a716-446655440000
blake3::hash(uuid.as_bytes())[..3] → [0xa3, 0xf8, 0x5b]
hex → "a3f85b"
```

- **Length:** 5 hex chars = 3 bytes = 16M namespace. At project scale (< 100K servers), collision probability is negligible (birthday paradox gives ~0.01% at 100K entries with 16M space).
- **Predictability:** Same UUID always produces the same subdomain. Deterministic — no storage needed for the hash mapping.
- **Stability:** Not derived from server name (which can change). Server UUID is immutable.
- **Suggested:** Use `blake3` crate (or SHA-256 first 3 bytes then hex). `blake3` is fast and already used elsewhere in the Rust ecosystem. If avoiding new deps, use `sha2::{Sha256, Digest}` (already in the stack via hmac/sha2).

### Generation Timing
1. User creates server via API → backend receives create request
2. Backend generates server UUID (standard V4 UUID) + short hex hash
3. Backend stores in `servers.subdomain` column
4. Backend pushes `relay.connect` task with full per-server config including subdomain

[VERIFIED: CONTEXT.md D-09, D-10]

### DNS Wildcard Strategy
- `*.play.esluce.net` → existing CNAME/ALIAS → NLB
- One-time setup (already done in Phase 68)
- No per-record DNS management needed — every subdomain resolves to the same NLB
- Gateway handles routing entirely via Handshake parsing (not DNS)

[VERIFIED: CONTEXT.md D-11; confirmed against existing Route 53 setup]

### Handshake Parsing Flow (already implemented in Phase 68 player.rs)
```
Player connects to a3f8b.play.esluce.net:25565
  ↓ DNS → NLB (wildcard resolves all subdomains)
  ↓ NLB → gateway:25565 (raw TCP)
  ↓ Gateway reads first bytes (MC Handshake packet)
  ↓ Parses VarInt packet length → VarInt packet ID = 0x00
  ↓ Parses String server address → "a3f8b.play.esluce.net"
  ↓ Extracts subdomain before ".play.esluce.net" → "a3f8b"
  ↓ Looks up server_id in registry.by_subdomain["a3f8b"]
  ↓ Opens yamux stream on that server's tunnel
  ↓ Forwards Handshake prefix + bidi copy
```

[VERIFIED: opt/relay/src/player.rs lines 138-183 — read_mc_handshake_subdomain() already implements this exactly]

## Gateway Changes

### auth.rs: 1:N relay_token → server_id

**Current behavior:** `auth.rs` calls `backend.authorize(relay_token, server_id)` which returns 200/403. The backend validates `node.owns(server_id)` via the `/internal/relay/authorize` endpoint.

**Phase 69 behavior:** **No change needed at the auth layer itself.** The backend already validates `node.owns(server_id)` correctly. The only risk is if the gateway code somewhere assumes 1:1 token→server_id mapping — grep for such assumptions and remove them.

**Verify:**
- `opt/relay/src/auth.rs` — `authorize()` takes `(token, server_id)` and calls backend. This is correct — it validates that the token's node owns the specific server_id.
- Backend's `/internal/relay/authorize` — already validates `node.owns(server_id)` which is exactly the right check.
- **No changes needed to the auth logic itself.**

[VERIFIED: opt/relay/src/auth.rs lines 17-23; opt/relay/src/backend.rs lines 69-118; CONTEXT.md D-19]

### tunnel.rs: N Concurrent WS from Same Agent IP

**Current behavior:** `tunnel.rs` `run_tunnel_session()` accepts one WS, authenticates, creates `TunnelHandle`, registers in `registry.rs`, runs control stream reader. No explicit handling of multiple WS from same IP.

**Phase 69 behavior:** The gateway must accept N concurrent WS connections from the same agent IP, each authenticating with the same `relay_token` but different `server_id`. The `registry.rs` already handles this correctly — it maps by `server_id`, so as long as each TunnelConnect sends a unique `server_id`, the registry insert succeeds.

**Check for conflicts:**
1. **Rate limiting** (`ratelimit.rs`): Currently applies per source IP. N WS from same IP will consume N rate limit tokens. This is fine — the rate limit is for connection attempts, not concurrent connections. But verify the rate limiter doesn't reject after the first WS from an IP.
   - **Recommendation:** Ensure rate limiting applies to CONNECTION ATTEMPTS, not ACTIVE CONNECTIONS. The current token-bucket refill should handle this — N WS connections within the bucket window are fine as long as total attempts stay under 100/min.

2. **Nonce dedup** (`auth.rs` Redis nonce check): Each WS has its own nonce. No conflict.

3. **TunnelHandle key**: Registry keys by server_id. Different server_ids → different entries. No conflict.

4. **Control stream per WS**: Each WS has its own yamux session with its own control stream. Independent.

**Summary:** **No changes needed in tunnel.rs** beyond what registry.rs already handles. The registry's `by_server_id` map naturally supports N tunnels. The `by_subdomain` map enforces 1:1 subdomain→server_id, which is correct (each subdomain maps to exactly one server).

[VERIFIED: opt/relay/src/registry.rs lines 42-61 — registry.register() supports multiple server_ids; CONTEXT.md D-20, D-21]

### registry.rs: NO CHANGE
Already `HashMap<ServerId, TunnelHandle>`. Phase 68 design (D-21) already specified 1 active tunnel per server_id. The registry handles replacement semantics. No changes needed.

[VERIFIED: opt/relay/src/registry.rs; CONTEXT.md D-21]

### player.rs: NO CHANGE
Already routes by subdomain via Handshake parsing (Phase 68 gap-01 fix). The flow is: subdomain → registry.lookup_by_subdomain → server_id → registry.get(server_id) → TunnelHandle → yamux stream. This works correctly with per-server tunnels because each tunnel handle has its own yamux Control.

[VERIFIED: opt/relay/src/player.rs lines 59-95; CONTEXT.md D-22]

### heartbeat.rs: NO CHANGE
Already iterates all tunnels in `registry.iter()` and checks each tunnel's `last_heartbeat` independently. Works correctly with N tunnels.

[VERIFIED: opt/relay/src/heartbeat.rs lines 31-42]

## Backend Changes

### New Migration: servers.subdomain

```sql
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS subdomain TEXT;

-- Add unique constraint to prevent subdomain collisions
CREATE UNIQUE INDEX IF NOT EXISTS idx_servers_subdomain
    ON servers (subdomain) WHERE subdomain IS NOT NULL;
```

[ASSUMED: migration pattern based on Phase 68 `20260607000001_add_relay_columns.sql`; CONTEXT.md D-10 confirms column name and purpose]

### Subdomain Generation on Server Create

When a server is created (in the server creation flow), the backend:

1. Generates the server UUID (already done)
2. Computes short hex hash: `hex::encode(&sha2::Sha256::digest(server_uuid.as_bytes())[..3])`
3. Stores in `servers.subdomain`
4. On collision (extremely rare with 16M namespace), retry with a different UUID or salt

**Collision handling:** The `UNIQUE` constraint on `servers.subdomain` catches collisions at the DB level. On insert collision, generate a new server UUID and retry.

[ASSUMED: collision handling strategy; CONTEXT.md D-09 specifies short hex hash format]

### relay.connect Task Payload

When the backend needs to push `relay.connect` for a server, the task payload includes:

```json
{
  "server_id": "550e8400-e29b-41d4-a716-446655440000",
  "subdomain": "a3f8b",
  "public_port": 25565,
  "local_mc_addr": "127.0.0.1:25565",
  "token": "<relay_token>",           // from nodes table (redundant with global config)
  "gateway_url": "wss://relay.esluce.net/relay/tunnel"  // shared, may be in global config
}
```

The agent merges shared fields from its global `RELAY_CONFIG` OnceCell and per-server fields from the payload.

[VERIFIED: CONTEXT.md D-13, D-15]

### Backend Push on Agent Reconnect (D-07)

When an agent reconnects to the backend WS, the backend must push `relay.connect` for ALL active servers the agent owns. This means:

1. On agent WS reconnect, the backend receives a `Register` message
2. Backend queries all servers where `node_id = agent.node_id`
3. For each server with `subdomain IS NOT NULL`, push `relay.connect` with the full per-server config
4. On server deletion, push `relay.disconnect` with server_id

[VERIFIED: CONTEXT.md D-07, D-08]

## Resource Considerations

### Per-Server Overhead
Each PerServerRuntime adds:
- 1 WebSocket connection (TLS + TCP) — ~64 KB memory for buffers
- 1 yamux session (Client) — ~100 KB memory (default window size)
- 1 heartbeat task — tokio task (~8 KB stack + ~100 bytes state)
- 1 reconnect loop — tokio task (same)
- 1 control stream (yamux stream handle) — negligible
- HashMap entry — ~200 bytes
**Total:** ~180 KB per server tunnel.

At 50 servers per node: ~9 MB overhead. Negligible.

### Heartbeat Staggering
Without staggering, 50 concurrent heartbeat tasks would send 50 messages to the gateway simultaneously every 10s. With 0-10s jitter:
- Average spread: ~5s between heartbeats (at 50 servers)
- Gateway receives ~5 heartbeats per second instead of 50 every 10s

### Bandwidth Accounting
Each PerServerRuntime has its own `AtomicU64` bytes counter. The gateway receives per-server byte counts in tunnel heartbeats. This enables per-server metrics in the backend/dashboard.

## Implementation Approach (Recommended Order)

### Wave 1: Backend (no agent/gateway coupling)
1. Create migration `servers.subdomain` column + unique index
2. Add subdomain generation to server creation flow
3. Add subdomain field to `Server` entity and repository
4. On agent WS reconnect → push `relay.connect` for all servers
5. On server delete → push `relay.disconnect`

### Wave 2: Agent (the core refactoring)
1. **state.rs**: Split `RelayConfig` — keep shared fields in OnceCell, remove per-server fields
2. **relay_client.rs**: 
   a. Replace `OnceLock<RelayRuntime>` with new struct containing `RwLock<HashMap<ServerId, PerServerRuntime>>`
   b. Define `PerServerRuntime` struct and `PerServerRelayConfig` struct
   c. Rewrite `connect()` to be per-server with add-replace semantics (D-06)
   d. Rewrite `disconnect()` to be per-server (cancel child token, remove from HashMap)
   e. Rewrite `send_heartbeat()` to be per-server (find by server_id, send on per-server control_tx)
   f. Rewrite `run_relay_client()` to take per-server config + child CancellationToken
   g. Add `shutdown_all()` for agent shutdown (parent.cancel() → clear HashMap)
3. **relay.rs**: Extract `server_id` from `task.payload`, call per-server functions
4. **mod.rs**: Update match arms (same relay.* task types, no structural change needed)

### Wave 3: Gateway (minimal changes)
1. **auth.rs**: Verify no 1:1 token→server_id assumption exists (likely none)
2. **tunnel.rs**: Handle N WS from same agent IP (likely no change needed)
3. Verify registry, player, heartbeat need no changes

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust's built-in `#[cfg(test)]` + `cargo test` |
| Config file | `Cargo.toml` (workspace root) |
| Quick run command | `cargo test -p escluse-relay-gateway --lib` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Test Scope | Behavior | Test Type | How to Validate |
|------------|----------|-----------|-----------------|
| Per-server connect/disconnect | Connect starts per-server reconnect loop; disconnect cancels it | Unit | Mock task dispatch, verify HashMap state before/after |
| Replace semantics (D-06) | Second connect for same server_id cancels first tunnel | Unit | Capture CancellationToken state; verify old token cancelled |
| Shutdown cascade (D-04) | parent.cancel() fires all child tokens | Unit | Spawn N child tokens, cancel parent, verify all children report cancelled() |
| Wrong server_id in disconnect | disconnect(non_existent_id) returns Ok, no error | Unit | HashMap should return None silently |
| Per-server heartbeat | send_heartbeat(server_id) sends on correct control_tx | Unit | Mock mpsc channel per server; verify correct channel receives |
| Gateway auth 1:N | Same token+different server_ids → all authorize OK | Integration | Mock backend /authorize to return 200 for any valid token+server_id pair |
| Gateway N WS from same IP | N concurrent WS from same IP → all accepted | Integration | Connect N WS, each with different server_id, verify N handles in registry |
| RelayConfig split | Global config unchanged; per-server config from payload | Unit | Mock relay_config() returns shared fields; task payload provides per-server fields |

### Sampling Rate
- **Per task commit:** `cargo test --workspace` (quick grep for compile errors + existing tests)
- **Per wave merge:** Full `cargo test` run against affected crates
- **Phase gate:** Full suite green before `/gsd-verify-work`

## Sources

### Primary (HIGH confidence)
- [VERIFIED: codebase] `src/handlers/relay_client.rs` — Current global OnceLock<RelayRuntime> architecture (lines 108-138), connect() (152-181), disconnect() (185-208), run_relay_client (255-303)
- [VERIFIED: codebase] `src/handlers/relay.rs` — Current global dispatch (lines 28-42) — no server_id extraction
- [VERIFIED: codebase] `src/state.rs` — Current RelayConfig struct (lines 137-172) with all fields flat
- [VERIFIED: codebase] `opt/relay/src/auth.rs` — Authorize takes (token, server_id) — no 1:1 assumption
- [VERIFIED: codebase] `opt/relay/src/tunnel.rs` — Single WS session per call (run_tunnel_session)
- [VERIFIED: codebase] `opt/relay/src/registry.rs` — HashMap<ServerId> + HashMap<Subdomain> — already per-server
- [VERIFIED: codebase] `opt/relay/src/player.rs` — Handshake parsing, subdomain routing — already correct
- [VERIFIED: codebase] `opt/relay/src/heartbeat.rs` — Iterates all tunnels independently
- [VERIFIED: codebase] `src/handlers/mod.rs` — Current match arms (lines 177-184) — no server_id in dispatch
- [VERIFIED: CONTEXT.md] All 22 implementation decisions documented

### Secondary (MEDIUM confidence)
- [CITED: tokio-util docs] CancellationToken child_token() pattern — verified against existing usage in relay_client.rs lines 130-145
- [CITED: Rust std docs] `tokio::sync::RwLock` semantics — concurrent reads, exclusive writes

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Gateway registry already supports N tunnels from same agent IP (keyed by server_id) | Gateway Changes | Low — registry.rs confirms HashMap keyed by server_id; N tunnels with different server_ids insert independently |
| A2 | Gateway auth.rs has no 1:1 relay_token→server_id enforcement | Gateway Changes | Low — auth.rs passes (token, server_id) to backend which validates node.owns(server_id) — this is exactly the correct check |
| A3 | relay_session.rs sig needs no change (takes StreamHandle + addr + bytes_counter) | Agent Changes | Low — session receives yamux stream from per-server session, same local addr. The stream handle type is the same |
| A4 | Backend generate subdomain uses SHA-256 first 3 bytes → hex | Backend Changes | Low — exact hash function is the agent's discretion; SHA-256 is already in dep tree via hmac/sha2 |

## Open Questions (RESOLVED)

1. **Where in the backend is server creation handled?** — RESOLVED: Plan 69-01 Task 2 handles this by locating the server creation handler and adding subdomain generation (SHA-256 first 3 bytes → 6 hex chars) at the entity constructor level.
2. **How does the backend push `relay.connect` on agent WS reconnect?** — RESOLVED: Plan 69-01 Task 3 adds the `push_all_servers()` call in the agent re-registration flow of `node_ws_handler.rs`, iterating all servers owned by the node and dispatching `relay.connect` for each.
3. **Are there any auth.rs changes needed for the gateway 1:N case?** — RESOLVED: Plan 69-05 Task 1 verifies `auth.rs` has no 1:1 assumption and updates the authorize flow to return all authorized server IDs for a token. Research confirmed the backend endpoint already supports `node.owns(server_id)` check which is correct for 1:N.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps needed; all primitives already in use
- Architecture: HIGH — all patterns verified against existing codebase
- Pitfalls: HIGH — CancellationToken behavior verified via CONTEXT.md discussion; heartbeat staggering per D-17; replacement semantics per D-06
- Gateway changes: MEDIUM — auth/tunnel verification done but implicit assumptions need grep confirmation

**Research date:** 2026-06-09
**Valid until:** 2026-06-16 (stable codebase — no fast-moving deps)
