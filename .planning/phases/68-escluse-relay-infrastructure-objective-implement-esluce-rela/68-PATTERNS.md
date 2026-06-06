# Phase 68: Escluse Relay Infrastructure - Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 50 new/modified across 4 tiers (Agent, Backend, Relay Gateway, Frontend)
**Analogs found:** 38 / 50 (12 are in the brand-new `opt/relay/` crate, no in-repo analog)

## File Classification

### Agent Tier (Rust — `src/`)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/handlers/relay_client.rs` (NEW) | service-client | ws-message / streaming | `src/agent_connection.rs:238-744` (`run()` reconnect loop) | exact |
| `src/handlers/relay_session.rs` (NEW) | service-executor | streaming (yamux stream ↔ local TCP) | `src/handlers/dns_watch.rs:18-80` (background `tokio::spawn` pattern) | role-match |
| `src/handlers/relay.rs` (NEW) | handler-orchestrator | request-response | `src/handlers/dns.rs:44-60` (per-task entrypoint shape) | exact |
| `src/handlers/dns.rs` (EXTEND) | service-orchestrator | request-response | `src/handlers/dns.rs:62-86` (existing `handle_create_record`) | exact |
| `src/handlers/mod.rs` (EXTEND) | dispatcher | request-response | `src/handlers/mod.rs:118-166` (`execute_single`) + `:186-294` (`get_task_config`) | exact |
| `src/main.rs` (EXTEND) | bootstrap | lifecycle | `src/main.rs:282-293` (DnsWatcher startup/shutdown) | exact |
| `Cargo.toml` (EXTEND) | config | n/a | `Cargo.toml:25-50` (existing tokio-tungstenite + reqwest deps) | exact |
| `src/audit.rs` (EXTEND) | service-helper | n/a | `src/audit.rs:23-42` (`log_task_received` etc.) | exact |

### Backend Tier (Rust — `api/`)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `api/migrations/20260607000001_add_relay_columns.sql` (NEW) | migration | n/a | `api/migrations/20260307000001_add_enhanced_server_features.sql` | exact |
| `api/src/domain/entities/node.rs` (EXTEND) | entity | n/a | `api/src/domain/entities/node.rs:8-50` (existing `Node` struct) | exact |
| `api/src/domain/entities/server.rs` (EXTEND) | entity | n/a | `api/src/domain/entities/server.rs:8-75` (existing `Server` struct) | exact |
| `api/src/presentation/ws/node_protocol.rs` (EXTEND) | ws-protocol | ws-message | `api/src/presentation/ws/node_protocol.rs:74-80` (`CrashReport` variant) | exact |
| `api/src/presentation/ws/node_connection_manager.rs` (EXTEND) | ws-manager | ws-message | `api/src/presentation/ws/node_connection_manager.rs:89-102` (`send_to_node`) | exact |
| `api/src/presentation/handlers/node_ws_handler.rs` (EXTEND) | ws-handler | ws-message | `api/src/presentation/handlers/node_ws_handler.rs:301-414` (`Heartbeat` + `CrashReport` cases) | exact |
| `api/src/presentation/handlers/relay_internal_handlers.rs` (NEW) | rest-handler | request-response | `api/src/presentation/handlers/cron_task_handlers.rs:17-44` (router + ownership check) | exact |
| `api/src/presentation/handlers/relay_handlers.rs` (NEW) | rest-handler | request-response | `api/src/presentation/handlers/cron_task_handlers.rs:17-44` (router + ownership check) | exact |
| `api/src/presentation/handlers/connectivity_handlers.rs` (NEW — Phase 67) | rest-handler | request-response | (planned in 67-03) `api/src/presentation/handlers/cron_task_handlers.rs:17-44` | exact |
| `api/src/application/services/relay_service.rs` (NEW) | service | CRUD + transform | `api/src/application/services/monitoring_service.rs:74-91` (background `start()`) | role-match |
| `api/src/application/services/monitoring_service.rs` (EXTEND) | service | metrics-scrape | `api/src/application/services/monitoring_service.rs:74-91` (start loop) | exact |
| `api/src/application/services/connectivity_service.rs` (NEW — Phase 67) | service | classify + dispatch | (planned in 67-03) `api/src/application/services/monitoring_service.rs:455-502` (`handle_crash_report`) | role-match |
| `api/src/presentation/routes/api_routes.rs` (EXTEND) | router | n/a | `api/src/presentation/routes/api_routes.rs:33-37` (per-server nested routes) | exact |
| `api/src/bootstrap/container.rs` (EXTEND) | DI-container | n/a | `api/src/bootstrap/container.rs:67-152` (struct fields) + `:328-342` (channel) + `:388-409` (return block) | exact |
| `api/src/presentation/handlers/mod.rs` (EXTEND) | module-root | n/a | `api/src/presentation/handlers/mod.rs:1-32` (existing `pub mod` list) | exact |
| `api/src/application/services/mod.rs` (EXTEND) | module-root | n/a | `api/src/application/services/mod.rs:1-9` (existing `pub mod` list) | exact |

### Relay Gateway Tier (NEW — `opt/relay/`) — no in-repo analog

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `opt/relay/Cargo.toml` (NEW) | config | n/a | `api/Cargo.toml` (similar dep set) | no-analog |
| `opt/relay/src/main.rs` (NEW) | bootstrap | lifecycle | `opt/umami/Caddyfile` + `docker-compose.yml` (deployment shape) | no-analog |
| `opt/relay/src/config.rs` (NEW) | config | n/a | (new pattern) | no-analog |
| `opt/relay/src/state.rs` (NEW) | shared-state | n/a | `api/src/bootstrap/container.rs:78-152` (DI struct) | role-match (DI pattern) |
| `opt/relay/src/auth.rs` (NEW) | middleware | request-response | (new pattern; closest is `api/src/presentation/middleware/auth.rs`) | no-analog |
| `opt/relay/src/tunnel.rs` (NEW) | ws-handler | ws-message + streaming | `api/src/presentation/handlers/node_ws_handler.rs:72-298` (WS upgrade) | role-match (WS upgrade) |
| `opt/relay/src/player.rs` (NEW) | tcp-forwarder | streaming | (new pattern; yamux + tokio) | no-analog |
| `opt/relay/src/registry.rs` (NEW) | shared-state | n/a | `api/src/presentation/ws/node_connection_manager.rs:21-35` (`DashMap`-style registry) | role-match |
| `opt/relay/src/backend.rs` (NEW) | http-client | request-response | (new pattern; reqwest to api.esluce.com) | no-analog |
| `opt/relay/src/metrics.rs` (NEW) | metrics-endpoint | n/a | `api/src/presentation/handlers/metrics_handlers.rs:1-61` (Axum metrics router) | role-match |
| `opt/relay/src/error.rs` (NEW) | error-type | n/a | `api/src/shared/errors/app_error.rs` | role-match |
| `opt/relay/src/ratelimit.rs` (NEW) | service-helper | n/a | `api/src/presentation/middleware/rate_limit.rs:95-128` (Redis SETEX counter) | exact |
| `opt/relay/src/session_log.rs` (NEW) | service-helper | n/a | (new pattern; Redis `RPUSH` log) | no-analog |
| `opt/relay/Caddyfile` (NEW) | gateway-config | n/a | `opt/umami/Caddyfile` + `gateway/Caddyfile.prod:12-21` (security_headers) | exact |
| `opt/relay/Caddy.Dockerfile` (NEW) | build-config | n/a | `opt/umami/docker-compose.yml` (caddy:2 image) | role-match |
| `opt/relay/relay-gateway.Dockerfile` (NEW) | build-config | n/a | (multi-stage Rust build) | no-analog |
| `opt/relay/docker-compose.yml` (NEW) | deploy-config | n/a | `opt/umami/docker-compose.yml` (same shape) | exact |
| `opt/relay/relay-gateway.toml` (NEW) | config | n/a | (new pattern) | no-analog |
| `opt/relay/.env.example` (NEW) | config | n/a | (new pattern) | no-analog |
| `opt/relay/DEPLOY.md` (NEW) | docs | n/a | `.planning/phases/66-*/DEPLOY.md` (manual AWS setup) | role-match |

### Frontend Tier (React — `app/`)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `app/src/components/ConnectivitySection.jsx` (EXTEND — Phase 67) | component | data-display | `app/src/components/ServerBackups.jsx` (per-server sub-section) | role-match |
| `app/src/components/InviteFriendsModal.jsx` (NEW) | component | display | (new pattern; QR code + share) | no-analog |
| `app/src/components/ModeOverrideDropdown.jsx` (NEW) | component | control | `app/src/components/StatusBadge.jsx` (color-state pattern) | role-match |
| `app/src/components/TunnelHealthCard.jsx` (NEW) | component | data-display | `app/src/components/MetricsCard.jsx` (stat display) | role-match |
| `app/src/hooks/useConnectivity.js` (NEW — Phase 67) | hook | data-fetching | `app/src/hooks/useAlerts.js:1-48` (exact pattern) | exact |
| `app/src/lib/api.js` (EXTEND) | api-client | n/a | `app/src/lib/api.js:100-115` (`serversApi` extension) | exact |
| `app/src/pages/servers/ServerDetailsPage.jsx` (EXTEND) | page | detail | `app/src/pages/servers/ServerDetailsPage.jsx:33-39` (tabs array) + section insertion | exact |
| `app/src/pages/servers/ServerManagerPage.jsx` (EXTEND) | page | list | `app/src/pages/servers/ServerManagerPage.jsx:25-43` (`getStatusColor`) | role-match |

---

## Pattern Assignments

### `src/handlers/relay_client.rs` (NEW — service-client, ws-message + streaming)

**Analog:** `src/agent_connection.rs:238-744` (the agent's existing `run()` loop) — exact pattern to copy for outbound WSS with reconnect + exponential backoff.

**Imports pattern** (mirror `agent_connection.rs:1-14`):
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use http::Request;
use rand::RngCore;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_yamux::{Config, Session, Control};
use tracing::{error, info, warn};
use uuid::Uuid;
```

**Outbound WS connect with custom headers** (D-09, D-11) — `IntoClientRequest` trait (tokio-tungstenite 0.26) accepts `http::Request<()>`:
```rust
let request = Request::builder()
    .method("GET")
    .uri("wss://relay.esluce.net/tunnel")
    .header("Host", "relay.esluce.net")
    .header("Connection", "Upgrade")
    .header("Upgrade", "websocket")
    .header("Sec-WebSocket-Version", "13")
    .header("Sec-WebSocket-Key", base64::engine::general_purpose::STANDARD.encode(rand::random::<[u8;16]>()))
    .header("Authorization", format!("Bearer {}", relay_token))
    .header("X-Relay-Nonce", hex::encode(nonce))
    .header("X-Relay-Timestamp", ts.to_string())
    .body(())?;
let (ws, _) = connect_async(request).await?;
```

**Reconnect + exponential backoff loop** (D-04) — mirror `agent_connection.rs:254-258, 274-738`:
```rust
let mut initial_delay = std::time::Duration::from_secs(1);
let max_delay = std::time::Duration::from_secs(30);
let multiplier = 2.0;

loop {
    match connect_tunnel(&relay_token).await {
        Ok((ws, control)) => {
            initial_delay = std::time::Duration::from_secs(1);  // reset on success
            // Heartbeat ticker + tunnel session loop
            // ...
        }
        Err(e) => { warn!("Tunnel connect failed: {}", e); }
    }
    if shutdown.load(Ordering::Relaxed) { break; }
    tokio::time::sleep(initial_delay).await;
    initial_delay = std::time::Duration::from_secs_f64(
        initial_delay.as_secs_f64() * multiplier
    ).min(max_delay);
}
```

**Heartbeat on existing WS** (D-04 — 10s ticker) — mirror `agent_connection.rs:444-481` (the existing `heartbeat_interval` pattern). Build a `TunnelHeartbeat` JSON message with `{ ts, uptime, bytes_in, bytes_out, active_streams }` and send via `ws_sender.send(Message::Text(json.into()))`.

**Wrap WS in yamux session** (D-02):
```rust
let session = Session::new_client(ws, Config::default());
let control = session.control();
tokio::spawn(session);  // drive the session
// Open per-server stream:
let stream = control.open_stream().await?;  // implements tokio::io::AsyncRead+AsyncWrite
```

**Send tunnel events to backend WS** — use the existing `OUTBOUND_TX` pattern (Phase 67's `connectivity.rs:696-701` `set_outbound_sender`) OR add a `NodeMessage::TunnelConnect` / `TunnelDisconnect` variant to `AgentMessage` (`src/agent_connection.rs:27-88`) and serialize via `serde_json::to_string(&msg)`.

---

### `src/handlers/relay_session.rs` (NEW — service-executor, streaming)

**Analog:** `src/handlers/dns_watch.rs:18-80` (background watcher pattern).

**Per-stream forwarder** (D-02 — one yamux stream per player):
```rust
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tokio_yamux::StreamHandle;

pub async fn handle_incoming_stream(
    stream: StreamHandle,
    server_id: Uuid,
    game_port: u16,
) -> anyhow::Result<()> {
    tracing::info!(%server_id, %game_port, "New player yamux stream");
    let local = TcpStream::connect(("127.0.0.1", game_port)).await?;
    let (mut player_side, mut mc_side) = (stream, local);
    let (in_bytes, out_bytes) = copy_bidirectional(&mut player_side, &mut mc_side).await?;
    tracing::info!(%server_id, in_bytes, out_bytes, "Player session ended");
    Ok(())
}
```

**Background-spawn pattern** (D-19 — drop on idle/timeout):
```rust
tokio::spawn(async move {
    let _ = tokio::time::timeout(
        Duration::from_secs(300),  // 5-min idle (D-19)
        handle_incoming_stream(stream, server_id, game_port),
    ).await;
});
```

The shell-out + `tokio::process::Command` style is from `src/handlers/runtime.rs:213-220`. The `tokio::spawn` is from `src/handlers/dns_watch.rs:43-64` (line 43 starts the `tokio::spawn(async move { ... })` block that runs the periodic ticker).

---

### `src/handlers/relay.rs` (NEW — handler-orchestrator, request-response)

**Analog:** `src/handlers/dns.rs:44-86` (the per-task entrypoint pattern). Task type strings: `relay.connect`, `relay.disconnect`, `relay.heartbeat`, `relay.refresh_token`.

**Per-task entrypoint shape** (mirror `dns.rs:44-60`):
```rust
pub async fn handle_connect(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let relay_token = task.payload.get("relay_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing relay_token in relay.connect payload"))?;
    // Store token, start client
    // ...
    Ok(json!({ "status": "connecting", "relay_token": relay_token }))
}
```

**Shared mutable state** (mirror `src/handlers/dns.rs:24-29`):
```rust
lazy_static! {
    pub static ref RELAY_TOKEN: Arc<RwLock<Option<String>>> =
        Arc::new(RwLock::new(None));
    pub static ref RELAY_STATUS: Arc<RwLock<String>> =
        Arc::new(RwLock::new("disconnected".to_string()));
}
```

---

### `src/handlers/dns.rs` (EXTEND — Direct Mode triggers on tunnel events)

**Existing handlers to extend** (`src/handlers/dns.rs:62-86` `handle_create_record`, `:88+` `handle_update_record`).

**Add new entrypoint** `handle_remove_record(task)` (Phase 68 D-13 — remove Cloudflare A record on `tunnel_disconnect`):
```rust
pub async fn handle_remove_record(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let name = task.payload.get("name").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'name' in remove_record payload"))?;
    let config = DNS_CONFIG.read().await;
    let cfg = config.as_ref().ok_or_else(|| anyhow!("DNS not configured"))?;
    let full_name = format!("{}.{}", name, cfg.wildcard_domain);
    let record_id = find_dns_record(&cfg.api_token, &cfg.zone_id, &full_name).await?
        .ok_or_else(|| anyhow!("No record to remove"))?;
    delete_dns_record(&cfg.api_token, &cfg.zone_id, &record_id).await?;
    Ok(json!({ "status": "removed", "domain": full_name }))
}
```

Add `delete_dns_record` helper that calls `DELETE /zones/{zone_id}/dns_records/{record_id}` (mirror of `find_dns_record` at `src/handlers/dns.rs:116`).

---

### `src/handlers/mod.rs` (EXTEND — dispatcher + task config)

**Analog:** `src/handlers/mod.rs:5-13` (module declarations) + `:118-166` (match arms) + `:186-294` (task config).

**Module declaration** (after `pub mod files;` at line 13):
```rust
pub mod relay;
pub mod relay_client;
pub mod relay_session;
```

**Add 4 new match arms** (inside `match task_type.as_str()` at line 118):
```rust
"relay.connect"        => relay::handle_connect(task.clone()).await,
"relay.disconnect"     => relay::handle_disconnect(task.clone()).await,
"relay.heartbeat"      => relay::handle_heartbeat(task.clone()).await,
"relay.refresh_token"  => relay::handle_refresh_token(task.clone()).await,
"dns.remove_record"    => dns::handle_remove_record(task.clone()).await,  // Phase 68 D-13
```

**Add 4 new `get_task_config` entries** (in `get_task_config` at line 186, before the `_` default at line 286):
```rust
"relay.connect" => TaskConfig {
    timeout: Duration::from_secs(30),
    max_retries: 0, retry_delay_ms: 0, max_retry_delay_ms: 0, backoff_multiplier: 1.0,
},
"relay.disconnect" => TaskConfig {
    timeout: Duration::from_secs(10),
    max_retries: 0, retry_delay_ms: 0, max_retry_delay_ms: 0, backoff_multiplier: 1.0,
},
"relay.heartbeat" => TaskConfig {
    timeout: Duration::from_secs(15),
    max_retries: 0, retry_delay_ms: 0, max_retry_delay_ms: 0, backoff_multiplier: 1.0,
},
"relay.refresh_token" => TaskConfig {
    timeout: Duration::from_secs(30),
    max_retries: 0, retry_delay_ms: 0, max_retry_delay_ms: 0, backoff_multiplier: 1.0,
},
"dns.remove_record" => TaskConfig {
    timeout: Duration::from_secs(10),
    max_retries: 0, retry_delay_ms: 0, max_retry_delay_ms: 0, backoff_multiplier: 1.0,
},
```

---

### `src/main.rs` (EXTEND — start RelayClient)

**Analog:** `src/main.rs:282-293` (DnsWatcher startup + shutdown handler).

**Insert after the DNS watcher block (after line 293)**:
```rust
// 9c. Start Relay tunnel client (Phase 68 — outbound WSS to relay.esluce.net)
let relay_client = Arc::new(handlers::relay_client::RelayClient::new(config.clone()));
relay_client.start(shutdown.clone()).await;

let relay_for_shutdown = relay_client.clone();
let shutdown_clone_relay = shutdown.clone();
tokio::spawn(async move {
    while !shutdown_clone_relay.load(Ordering::Relaxed) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    relay_for_shutdown.stop().await;
});
```

The `RelayClient::start` follows the same shape as `src/handlers/dns_watch.rs:31-65` — `tokio::spawn(async move { ... })` with a `running: Arc<RwLock<bool>>` flag for cooperative shutdown.

---

### `Cargo.toml` (EXTEND — add `tokio-yamux`)

**Analog:** `Cargo.toml:25-50` (existing tokio-tungstenite + reqwest + uuid + chrono).

Add after `tokio-stream` at line 28:
```toml
# Phase 68: Relay tunnel multiplexing (D-02 — yamux over WebSocket)
tokio-yamux = "0.3"
```

Plus add at the end of `[dependencies]`:
```toml
# Phase 68: Custom WS request headers (IntoClientRequest)
http = "1"
rand = "0.8"
hex = "0.4"
base64 = "0.22"
```

---

### `api/migrations/20260607000001_add_relay_columns.sql` (NEW — schema additions)

**Analog:** `api/migrations/20260307000001_add_enhanced_server_features.sql` (entire file — the exact pattern for adding `ADD COLUMN IF NOT EXISTS`).

```sql
-- Phase 68: Escluse Relay infrastructure
-- Per-agent bearer token for tunnel handshake (D-09)
-- The token is generated at node registration (gen_random_uuid), returned
-- in the RegisterAck message, and persisted in the agent's TOML config.

ALTER TABLE nodes
    ADD COLUMN IF NOT EXISTS relay_token UUID UNIQUE DEFAULT gen_random_uuid();

-- Phase 68: Per-server mode override (D-12 — user-pinned, nullable)
-- 'auto' = agent decides; 'direct' = Direct-only; 'relay' = Relay-only
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS connectivity_mode_override TEXT
    CHECK (connectivity_mode_override IN ('auto', 'direct', 'relay') OR connectivity_mode_override IS NULL);

-- Phase 68: Relay tunnel state (D-22 — gateway → backend heartbeat)
-- 'connected' | 'connecting' | 'disconnected'
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS relay_status TEXT
    CHECK (relay_status IN ('connected', 'connecting', 'disconnected') OR relay_status IS NULL);

-- Phase 68: Last successful tunnel connection timestamp
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS last_tunnel_connected_at TIMESTAMPTZ;

-- Backfill: existing nodes need a relay_token (UNIQUE NOT NULL would fail
-- without DEFAULT for the existing rows; the DEFAULT above handles it).
```

Use `ADD COLUMN IF NOT EXISTS` (precedent: `20260307000001:5`). Use `TEXT` not `ENUM` for forward compatibility with future states. The `CHECK` constraint per SECURITY V4 (T-67-09 / Phase 68 T-68-09) prevents arbitrary user input.

---

### `api/src/domain/entities/node.rs` (EXTEND — add `relay_token` field)

**Analog:** `api/src/domain/entities/node.rs:8-50` (existing struct).

Add after `pub api_key_hash: Option<String>,` (line 40):
```rust
/// Phase 68: per-agent bearer token for tunnel handshake (D-09)
/// Generated at registration via `gen_random_uuid()`, returned in `RegisterAck`,
/// sent on every tunnel connect as `Authorization: Bearer <token>`.
pub relay_token: Uuid,
```

Update the `Node::new` constructor (`node.rs:53-78`) to default the token:
```rust
relay_token: Uuid::new_v4(),  // or DEFAULT in DB if not supplied
```

And update `update_info` (line 80) signature if it needs to take a new token (probably not — relay_token is set once at registration and only rotated via a future token-rotation endpoint).

---

### `api/src/domain/entities/server.rs` (EXTEND — add 3 fields)

**Analog:** `api/src/domain/entities/server.rs:8-75` (existing struct).

Add after `pub discord_webhook_url: Option<String>,` (line 30):
```rust
// Phase 68: Relay mode override (D-12, set via /connectivity/mode-override)
pub connectivity_mode_override: Option<String>,

// Phase 68: Current tunnel status (D-22 — gateway → backend heartbeat)
pub relay_status: Option<String>,

// Phase 68: Last successful tunnel connection timestamp
pub last_tunnel_connected_at: Option<DateTime<Utc>>,
```

Update `Server::is_running` (line 78) and any code that constructs a `Server` literal (search the repo for `Server { id: Uuid::new_v4()` and similar) — three new fields need to be filled in. If the literal is `..Default::default()`, no change needed.

---

### `api/src/presentation/ws/node_protocol.rs` (EXTEND — 5 new variants)

**Analog:** `api/src/presentation/ws/node_protocol.rs:74-80` (the existing `CrashReport` agent→backend variant) and `:116-134` (`DnsConfig` backend→agent broadcast).

**Agent → Backend additions** (insert after `CrashReport` at line 80, before `// Backend -> Agent`):
```rust
// Phase 68: Tunnel lifecycle events (agent → backend → dashboard)
#[serde(rename = "tunnel_connect")]
TunnelConnect {
    node_id: Uuid,
    server_ids: Vec<Uuid>,    // which servers this tunnel carries
    public_ip: String,
    timestamp: String,
},

#[serde(rename = "tunnel_disconnect")]
TunnelDisconnect {
    node_id: Uuid,
    reason: String,            // "agent_stopped" | "heartbeat_timeout" | "shutdown" | "mode_override"
    duration_secs: u64,
    timestamp: String,
},

#[serde(rename = "tunnel_heartbeat")]
TunnelHeartbeat {
    node_id: Uuid,
    uptime_secs: u64,
    bytes_in: u64,
    bytes_out: u64,
    active_streams: u32,
    timestamp: String,
},
```

**Backend → Agent additions** (insert after `DnsConfig` at line 134, before the closing brace):
```rust
// Phase 68: User-pinned mode override (backend → agent, D-12)
#[serde(rename = "mode_override_change")]
ModeOverrideChange {
    server_id: Uuid,
    override_mode: Option<String>,  // None = back to automatic
    timestamp: String,
},
```

The `RegisterAck` variant (line 105-110) also needs to be extended to include the new `relay_token` field for the FIRST registration. Add a new field:
```rust
#[serde(rename = "register_ack")]
RegisterAck {
    node_id: Uuid,
    status: String,
    message: String,
    /// Phase 68: per-agent relay token (D-09). Returned on every registration
    /// so a re-registering agent can pick up a rotated token.
    #[serde(default)]
    relay_token: Option<Uuid>,
},
```

This is a **non-breaking change** (the new field is `Option<Uuid>` with `#[serde(default)]`).

---

### `api/src/presentation/ws/node_connection_manager.rs` (EXTEND — no API change)

**No extension needed.** The existing `send_to_node(&node_id, msg)` at lines 89-102 is fully generic over `&NodeMessage` — it works for any new variant added to the enum. The planner can call:
```rust
manager.send_to_node(&node_id, &NodeMessage::ModeOverrideChange {
    server_id, override_mode, timestamp: now.to_rfc3339(),
}).await?;
```

exactly like the existing DnsConfig replay at `api/src/presentation/handlers/node_ws_handler.rs:269-271`.

---

### `api/src/presentation/handlers/node_ws_handler.rs` (EXTEND — 3 new dispatch arms + extended RegisterAck)

**Analog:** `api/src/presentation/handlers/node_ws_handler.rs:301-414` (Heartbeat + CrashReport cases) + `:216-224` (RegisterAck construction).

**Extended RegisterAck** (line 216-220):
```rust
let ack = NodeMessage::RegisterAck {
    node_id: node_id_val,
    status: "online".to_string(),
    message: "Node registered successfully".to_string(),
    relay_token: Some(new_node.relay_token),  // Phase 68 — send token to agent
};
```

(For the `find_by_id` path, look up the existing node's `relay_token` and include it. For the `create` path, the `gen_random_uuid()` DEFAULT in the migration sets it; fetch it back via `find_by_id` after the `create` succeeds.)

**3 new dispatch arms** (insert AFTER `CrashReport` case at line 414, before `_ => { ... }` at line 416):
```rust
NodeMessage::TunnelConnect { node_id: tn_node_id, server_ids, public_ip, timestamp: _ } => {
    tracing::info!("[RELAY] Tunnel connected: node={}, servers={}", tn_node_id, server_ids.len());
    if let Err(e) = container.relay_service
        .handle_tunnel_connect(tn_node_id, &server_ids, &public_ip).await {
        tracing::error!("[RELAY] handle_tunnel_connect failed: {}", e);
    }
}

NodeMessage::TunnelDisconnect { node_id: tn_node_id, reason, duration_secs, timestamp: _ } => {
    tracing::info!("[RELAY] Tunnel disconnected: node={}, reason={}", tn_node_id, reason);
    if let Err(e) = container.relay_service
        .handle_tunnel_disconnect(tn_node_id, &reason, duration_secs).await {
        tracing::error!("[RELAY] handle_tunnel_disconnect failed: {}", e);
    }
}

NodeMessage::TunnelHeartbeat { node_id: tn_node_id, uptime_secs, bytes_in, bytes_out, active_streams, timestamp: _ } => {
    if let Err(e) = container.relay_service
        .update_tunnel_metrics(tn_node_id, uptime_secs, bytes_in, bytes_out, active_streams).await {
        tracing::warn!("[RELAY] update_tunnel_metrics failed: {}", e);
    }
}
```

The `node_id` field is required because the WS frame can come in before `Register` (same as Heartbeat — see the safer form at `67-03/PLAN.md:184`).

---

### `api/src/presentation/handlers/relay_internal_handlers.rs` (NEW — internal REST handler)

**Analog:** `api/src/presentation/handlers/cron_task_handlers.rs:17-44` (router + ownership check) — except no `auth_user` extractor since this is an internal endpoint (D-10 — gateway calls it from a known EC2 IP, NOT from a user).

**HMAC signature verification** (V9 Communication — RESEARCH pitfall A2): add a shared-secret header check BEFORE the handler body.

```rust
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::presentation::routes::api_routes::ApiState;
use crate::shared::errors::app_error::AppError;

type HmacSha256 = Hmac<Sha256>;

pub fn router(state: ApiState) -> Router<ApiState> {
    Router::new()
        .route("/relay/authorize", post(authorize))
        .route("/relay/tunnel-event", post(tunnel_event))
        .with_state(state)
}

fn verify_hmac(secret: &str, body: &str, signature: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(body.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    // Constant-time compare (use subtle::ConstantTimeEq if available)
    expected == signature
}

#[derive(Deserialize)]
pub struct AuthorizeRequest {
    pub relay_token: Uuid,
    pub server_id: Uuid,
}

pub async fn authorize(
    State(state): State<ApiState>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    // V9: HMAC validation
    let sig = headers.get("X-Internal-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Missing HMAC signature"))?;
    let secret = std::env::var("RELAY_HMAC_SECRET")
        .map_err(|_| AppError::service_unavailable("HMAC secret not configured"))?;
    if !verify_hmac(&secret, std::str::from_utf8(&body).map_err(|_| AppError::bad_request("invalid utf8"))?, sig) {
        return Err(AppError::unauthorized("Invalid HMAC"));
    }

    let req: AuthorizeRequest = serde_json::from_slice(&body)
        .map_err(|e| AppError::bad_request(format!("Bad JSON: {}", e)))?;

    // Look up node by relay_token, verify it owns the server (D-10)
    let node = state.node_repository.find_by_relay_token(&req.relay_token).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?
        .ok_or_else(|| AppError::forbidden("Unknown relay token"))?;
    let owns = state.server_repository.owns(&node.id, &req.server_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;
    if !owns { return Err(AppError::forbidden("Node does not own server")); }

    Ok(Json(serde_json::json!({
        "node_id": node.id,
        "user_id": node.user_id,
    })))
}

#[derive(Deserialize)]
pub struct TunnelEventRequest {
    pub relay_token: Uuid,
    pub server_id: Uuid,
    pub event: String,  // "connected" | "heartbeat" | "disconnected" | "stale"
    pub uptime_secs: Option<u64>,
    pub bytes_in: Option<u64>,
    pub bytes_out: Option<u64>,
    pub active_streams: Option<u32>,
}

pub async fn tunnel_event(
    State(state): State<ApiState>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    let sig = headers.get("X-Internal-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Missing HMAC signature"))?;
    let secret = std::env::var("RELAY_HMAC_SECRET")
        .map_err(|_| AppError::service_unavailable("HMAC secret not configured"))?;
    if !verify_hmac(&secret, std::str::from_utf8(&body).map_err(|_| AppError::bad_request("invalid utf8"))?, sig) {
        return Err(AppError::unauthorized("Invalid HMAC"));
    }

    let req: TunnelEventRequest = serde_json::from_slice(&body)
        .map_err(|e| AppError::bad_request(format!("Bad JSON: {}", e)))?;
    if let Err(e) = container.relay_service
        .handle_gateway_tunnel_event(&req).await {
        tracing::error!("[RELAY] handle_gateway_tunnel_event failed: {}", e);
        return Err(AppError::InternalError(anyhow::anyhow!(e.to_string())));
    }
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
```

**Module registration** (`api/src/presentation/handlers/mod.rs:32` — add at end):
```rust
pub mod relay_internal_handlers;
```

---

### `api/src/presentation/handlers/relay_handlers.rs` (NEW — user-facing REST)

**Analog:** `api/src/presentation/handlers/cron_task_handlers.rs:17-44` (router + ownership check).

```rust
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::domain::auth::middleware::AuthUser;
use crate::presentation::routes::api_routes::ApiState;
use crate::presentation::responses::api_response::ApiResponse;
use crate::shared::errors::app_error::AppError;

pub fn router(state: ApiState) -> Router<ApiState> {
    Router::new()
        .route("/:server_id/connectivity/mode-override", post(set_mode_override).delete(clear_mode_override))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct ModeOverrideRequest {
    pub mode: String,  // "auto" | "direct" | "relay"
}

pub async fn set_mode_override(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
    Json(req): Json<ModeOverrideRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    // Per-tenant ownership check (mirror cron_task_handlers.rs:30-37)
    let server = state.server_repository.find_by_id(&server_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?
        .ok_or(AppError::NotFound)?;
    if server.user_id != auth_user.tenant_id { return Err(AppError::Forbidden); }

    if !["auto", "direct", "relay"].contains(&req.mode.as_str()) {
        return Err(AppError::bad_request("mode must be 'auto'|'direct'|'relay'"));
    }

    state.relay_service.set_mode_override(&server_id, Some(&req.mode)).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;

    // Notify the agent via the existing NodeConnectionManager (no new infrastructure)
    if let Some(node_id) = server.node_id {
        if state.node_connection_manager.is_connected(&node_id).await {
            let _ = state.node_connection_manager.send_to_node(&node_id, &NodeMessage::ModeOverrideChange {
                server_id,
                override_mode: Some(req.mode.clone()),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await;
        }
    }

    Ok(Json(ApiResponse::success(serde_json::json!({
        "server_id": server_id, "mode_override": req.mode,
    }))))
}

pub async fn clear_mode_override(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let server = state.server_repository.find_by_id(&server_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?
        .ok_or(AppError::NotFound)?;
    if server.user_id != auth_user.tenant_id { return Err(AppError::Forbidden); }

    state.relay_service.set_mode_override(&server_id, None).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;

    if let Some(node_id) = server.node_id {
        if state.node_connection_manager.is_connected(&node_id).await {
            let _ = state.node_connection_manager.send_to_node(&node_id, &NodeMessage::ModeOverrideChange {
                server_id,
                override_mode: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await;
        }
    }

    Ok(Json(ApiResponse::success(serde_json::json!({
        "server_id": server_id, "mode_override": null,
    }))))
}
```

**Module registration** (`api/src/presentation/handlers/mod.rs:32` — add at end):
```rust
pub mod relay_handlers;
```

---

### `api/src/application/services/relay_service.rs` (NEW — service)

**Analog:** `api/src/application/services/monitoring_service.rs:74-91` (the `start(self: Arc<Self>)` background-service pattern) and `:455-502` (the `handle_crash_report` classification + persist + notify flow).

```rust
use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::repositories::server_repository::ServerRepository;
use crate::domain::repositories::node_repository::NodeRepository;

pub struct RelayService {
    server_repository: Arc<dyn ServerRepository>,
    node_repository: Arc<dyn NodeRepository>,
    pool: PgPool,
}

impl RelayService {
    pub fn new(
        server_repository: Arc<dyn ServerRepository>,
        node_repository: Arc<dyn NodeRepository>,
        pool: PgPool,
    ) -> Self {
        Self { server_repository, node_repository, pool }
    }

    /// Persist mode override on the servers row (D-12).
    pub async fn set_mode_override(&self, server_id: &Uuid, mode: Option<&str>) -> Result<()> {
        sqlx::query("UPDATE servers SET connectivity_mode_override = $1, updated_at = $2 WHERE id = $3")
            .bind(mode)
            .bind(Utc::now().naive_utc())
            .bind(server_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Agent sent a `TunnelConnect`: persist relay_status + last_tunnel_connected_at
    /// for ALL servers in the agent's tunnel (D-22).
    pub async fn handle_tunnel_connect(
        &self, node_id: Uuid, server_ids: &[Uuid], _public_ip: &str,
    ) -> Result<()> {
        for sid in server_ids {
            sqlx::query(
                "UPDATE servers SET relay_status = 'connected',
                                    last_tunnel_connected_at = $1,
                                    updated_at = $1
                 WHERE id = $2 AND node_id = $3"
            )
            .bind(Utc::now().naive_utc())
            .bind(sid)
            .bind(node_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Agent sent a `TunnelDisconnect`: mark all its servers' relay_status as
    /// 'disconnected' (D-22). The next periodic re-probe will pick the new mode.
    pub async fn handle_tunnel_disconnect(
        &self, node_id: Uuid, _reason: &str, _duration_secs: u64,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE servers SET relay_status = 'disconnected', updated_at = $1
             WHERE node_id = $2"
        )
        .bind(Utc::now().naive_utc())
        .bind(node_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update Prometheus/Redis counters from the agent's heartbeat (D-22).
    pub async fn update_tunnel_metrics(
        &self, _node_id: Uuid, _uptime_secs: u64, _bytes_in: u64,
        _bytes_out: u64, _active_streams: u32,
    ) -> Result<()> {
        // The actual Prometheus emission happens via the relay gateway's own
        // /metrics endpoint (D-22). Backend just records the latest per-server
        // bandwidth snapshot in the `servers.config` JSONB for the dashboard.
        Ok(())
    }

    /// Internal endpoint: gateway called us with (relay_token, server_id) — verify
    /// ownership. Used for caching decisions in the relay; backend is the source
    /// of truth.
    pub async fn handle_gateway_authorize(
        &self, relay_token: Uuid, server_id: Uuid,
    ) -> Result<Option<Uuid>> {
        let node = self.node_repository.find_by_relay_token(&relay_token).await?;
        let Some(node) = node else { return Ok(None); };
        let owns = self.server_repository.owns(&node.id, &server_id).await?;
        if !owns { return Ok(None); }
        Ok(Some(node.id))
    }
}
```

**Module registration** (`api/src/application/services/mod.rs:9` — add at end):
```rust
pub mod relay_service;
```

---

### `api/src/bootstrap/container.rs` (EXTEND — wire RelayService + NodeRepository.find_by_relay_token)

**Analog:** `api/src/bootstrap/container.rs:67-152` (struct fields) + `:328-342` (channel construction) + `:354-409` (return block).

**Add field** to `AppContainer` struct (after `crash_report_tx: Option<mpsc::Sender<CrashReportData>>` at line 151, before closing brace):
```rust
// Phase 68: Relay service
pub relay_service: Arc<RelayService>,
```

**Add import** (alongside other service imports at line 67):
```rust
use crate::application::services::relay_service::RelayService;
```

**Construct in `AppContainer::new`** (after `monitoring_service` construction at line 333, before `billing_service` at line 344):
```rust
// Phase 68: Relay service
let relay_service = Arc::new(RelayService::new(
    repo.clone(),
    node_repo.clone(),
    pool.clone(),
));
```

**Add to return block** (next to `crash_report_tx: Some(crash_report_tx)` at line 408):
```rust
relay_service,
```

The `NodeRepository` trait also needs a new method `find_by_relay_token(token: Uuid) -> Result<Option<Node>>` — add to `api/src/domain/repositories/node_repository.rs` and the corresponding `PostgresNodeRepository::find_by_relay_token` impl that does `SELECT * FROM nodes WHERE relay_token = $1`.

---

### `api/src/presentation/routes/api_routes.rs` (EXTEND — mount new routes)

**Analog:** `api/src/presentation/routes/api_routes.rs:33-37` (per-server nested routes pattern).

**Add** (insert after the connectivity routes block planned for Phase 67, before line 39):
```rust
// Phase 68: User-facing mode-override
.route("/api/v1/servers/:server_id/connectivity/mode-override",
    post(crate::presentation::handlers::relay_handlers::set_mode_override)
    .delete(crate::presentation::handlers::relay_handlers::clear_mode_override))

// Phase 68: Internal relay gateway endpoints (HMAC-protected)
.nest("/api/v1/internal/relay", crate::presentation::handlers::relay_internal_handlers::router(state.clone()))
```

---

### Relay Gateway (NEW — `opt/relay/`)

**No in-repo analog for the gateway itself** (it's a new Rust service). Below are the cross-tier patterns to follow.

#### `opt/relay/Cargo.toml`

**Analog:** `api/Cargo.toml` and `Cargo.toml:25-50` (project dep set).

```toml
[package]
name = "escluse-relay-gateway"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio             = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.26", features = ["rustls-tls-native-roots"] }
tokio-yamux       = "0.3"
tokio-stream      = { version = "0.1", features = ["sync"] }
futures-util      = "0.3"

axum              = { version = "0.7", features = ["ws", "macros"] }
tower             = "0.5"
tower-http        = { version = "0.5", features = ["trace", "timeout", "cors"] }
hyper             = { version = "1", features = ["full"] }

serde             = { version = "1", features = ["derive"] }
serde_json        = "1"
uuid              = { version = "1", features = ["v4", "serde"] }
chrono            = { version = "0.4", features = ["serde"] }
reqwest           = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }

redis             = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
dashmap           = "6"

prometheus        = "0.13"
tracing           = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
anyhow            = "1"
thiserror         = "2"
serde_yaml        = "0.9"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
```

#### `opt/relay/src/main.rs`

**Analog:** `api/src/bootstrap/mod.rs:19-185` (`build_app()`) and `opt/umami/docker-compose.yml` (deployment shape).

```rust
use anyhow::Result;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod backend;
mod config;
mod error;
mod metrics;
mod player;
mod ratelimit;
mod registry;
mod session_log;
mod state;
mod tunnel;

use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Logging (mirror api/src/bootstrap/mod.rs:25-30)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // 2. Config (TOML or env)
    let cfg = config::Config::load()?;

    // 3. AppState (DI container — mirror api/src/bootstrap/container.rs:78-152)
    let app_state = Arc::new(AppState::new(cfg).await?);

    // 4. Axum router — /tunnel is the WebSocket, /metrics is Prometheus, /healthz is liveness
    let app = Router::new()
        .route("/tunnel", get(tunnel::tunnel_upgrade))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/healthz", get(|| async { "OK" }))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    // 5. Bind + serve
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.listen_port));
    tracing::info!("Escluse Relay Gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
```

#### `opt/relay/src/auth.rs` (Tunnel auth middleware)

**Analog:** `api/src/presentation/middleware/auth.rs` (axum middleware pattern) and the WebSocket upgrade in `api/src/presentation/handlers/node_ws_handler.rs:26-53`.

```rust
use axum::{extract::ws::WebSocketUpgrade, http::HeaderMap, response::IntoResponse};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::RelayError;
use crate::state::AppState;

pub async fn tunnel_upgrade(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Result<impl IntoResponse, RelayError> {
    // 1. Bearer token (D-09)
    let token = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(RelayError::Unauthorized)?;

    // 2. Nonce (D-11) — Redis SET NX EX 600
    let nonce = headers.get("X-Relay-Nonce")
        .and_then(|v| v.to_str().ok())
        .ok_or(RelayError::BadRequest("missing X-Relay-Nonce"))?;
    let ts_str = headers.get("X-Relay-Timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or(RelayError::BadRequest("missing X-Relay-Timestamp"))?;
    let ts: u64 = ts_str.parse().map_err(|_| RelayError::BadRequest("bad timestamp"))?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if now.abs_diff(ts) > 300 { return Err(RelayError::Unauthorized); }  // 5-min replay window

    let nonce_key = format!("relay:nonce:{}", nonce);
    let was_set: Option<String> = redis::cmd("SET")
        .arg(&nonce_key).arg("1").arg("NX").arg("EX").arg(600)
        .query_async(&mut state.redis.get().await?)
        .await?;
    if was_set.is_none() { return Err(RelayError::Unauthorized); }  // replay attack

    // 3. Rate limit (D-20) — 100 connects/min per source IP
    ratelimit::check_connect(&state.redis, &addr.ip().to_string(), "tunnel", 100, 60).await?;

    // 4. Backend introspection (D-10) — POST to /internal/relay/authorize
    // Caller passes server_id in `?server_id=<uuid>` query param
    // (server_id is one specific server; tunnel carries many)
    // For the initial accept, we just verify the relay_token is valid;
    // per-server authorization happens per-stream when a player connects.

    Ok(ws.on_upgrade(move |socket| tunnel::handle_tunnel_session(socket, state, token.to_string())))
}
```

#### `opt/relay/src/tunnel.rs`

**Analog:** `api/src/presentation/handlers/node_ws_handler.rs:72-298` (the WS upgrade + receive loop pattern).

```rust
use axum::extract::ws::{WebSocket, Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_yamux::{Config, Session};
use crate::state::AppState;

pub async fn handle_tunnel_session(
    socket: WebSocket,
    state: Arc<AppState>,
    relay_token: String,
) {
    let (mut tx, mut rx) = socket.split();

    // Adapt WS as AsyncRead+AsyncWrite for yamux
    let ws_adapter = WebSocketIo::new(tx.by_ref(), rx.by_ref());

    // Start yamux server session (gateway side; agent is client)
    let session = Session::new_server(ws_adapter, Config::default());
    let control = session.control();
    tokio::spawn(session);

    // Spawn the heartbeat watchdog (D-04 — 10s ticker, 3 missed = stale)
    let watchdog_state = state.clone();
    let watchdog_token = relay_token.clone();
    tokio::spawn(async move {
        registry::heartbeat_watchdog(watchdog_state, watchdog_token).await;
    });

    // Wait for incoming yamux streams
    // (yamux library spawns per-stream tasks internally; this loop just keeps the WS alive)
    // The actual per-stream forwarding happens in player.rs (gateway side) and
    // relay_session.rs (agent side).

    // Park this task; yamux will drive the streams
    futures::future::pending::<()>().await;
}

// Adapter: WebSocketSink + WebSocketStream → AsyncRead+AsyncWrite
// (or use `tokio_tungstenite::WebSocketStream` directly — agent side pattern)
struct WebSocketIo<Si, St> { sink: Si, stream: St, /* ... */ }
impl<Si: SinkExt<Message> + Unpin, St: StreamExt<Item = Result<Message, ...>> + Unpin> AsyncRead for WebSocketIo<Si, St> { /* ... */ }
// ... + AsyncWrite impl
```

#### `opt/relay/src/player.rs` (Player TCP forwarder)

**Analog:** None in repo (new pattern). Follow the bidi-copy pattern from `src/agent_connection.rs:482-700` (the existing `select!` loop).

```rust
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use crate::state::AppState;

pub async fn handle_player_connection(
    state: Arc<AppState>,
    player_stream: TcpStream,
    player_addr: SocketAddr,
) {
    // D-06 + Pitfall 9: resolve server_id by player source IP = agent public IP
    let server_id = match state.registry.find_by_agent_ip(player_addr.ip()).await {
        Some(id) => id,
        None => {
            tracing::warn!(%player_addr, "No active tunnel for player IP — closing");
            drop(player_stream);  // D-18: clean close
            return;
        }
    };

    // Open a yamux stream on the agent's tunnel
    let tunnel = match state.registry.get(&server_id).await {
        Some(t) => t,
        None => { drop(player_stream); return; }
    };
    let yamux_stream = match tunnel.control.open_stream().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(?server_id, ?e, "Failed to open yamux stream");
            drop(player_stream);
            return;
        }
    };

    // Bidi copy (5-min idle timeout, D-19)
    let copy_result = tokio::time::timeout(
        Duration::from_secs(300),
        copy_bidirectional(&mut &player_stream, &mut yamux_stream),
    ).await;

    if let Ok(Ok((in_bytes, out_bytes))) = copy_result {
        metrics::PLAYER_BYTES_IN.inc_by(in_bytes as f64);
        metrics::PLAYER_BYTES_OUT.inc_by(out_bytes as f64);
    }
}
```

#### `opt/relay/src/registry.rs` (Tunnel registry + heartbeat watchdog)

**Analog:** `api/src/presentation/ws/node_connection_manager.rs:21-35` (the `Arc<RwLock<HashMap>>` registry pattern), generalized to a `DashMap<Uuid, TunnelEntry>` for lock-free reads on the hot path.

```rust
use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;
use tokio_yamux::Control;
use uuid::Uuid;
use crate::state::AppState;

pub struct TunnelEntry {
    pub node_id: Uuid,
    pub server_id: Uuid,
    pub agent_public_ip: IpAddr,
    pub control: Control,
    pub last_heartbeat: Instant,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub active_streams: u32,
}

#[derive(Clone)]
pub struct Registry {
    pub by_server: Arc<DashMap<Uuid, Arc<TunnelEntry>>>,
    pub by_agent_ip: Arc<DashMap<IpAddr, Uuid>>,  // Pitfall 9 — player source IP → server_id
}

impl Registry {
    pub fn new() -> Self { /* ... */ }
    pub async fn insert(&self, entry: Arc<TunnelEntry>) { /* ... */ }
    pub async fn get(&self, server_id: &Uuid) -> Option<Arc<TunnelEntry>> { /* ... */ }
    pub async fn find_by_agent_ip(&self, ip: IpAddr) -> Option<Uuid> { /* ... */ }
    pub async fn remove(&self, server_id: &Uuid) { /* ... */ }
}

pub async fn heartbeat_watchdog(state: Arc<AppState>, relay_token: String) {
    let mut ticker = interval(Duration::from_secs(10));
    let stale_threshold = 3;
    let mut missed = 0u32;
    loop {
        ticker.tick().await;
        // Find tunnel for this relay_token (linear scan over DashMap is fine — <1k entries)
        let tunnel = state.registry.by_server.iter()
            .find(|e| e.value().relay_token == relay_token)
            .map(|e| e.value().clone());
        let Some(tunnel) = tunnel else { break };
        if tunnel.last_heartbeat.elapsed() > Duration::from_secs(30) {
            missed += 1;
            if missed >= stale_threshold {
                tracing::warn!(node=%tunnel.node_id, "Tunnel stale — closing");
                let _ = tunnel.control.close().await;
                state.registry.remove(&tunnel.server_id).await;
                // Notify backend via POST /internal/relay/tunnel-event
                let _ = state.backend_client
                    .notify_tunnel_event(tunnel.server_id, tunnel.node_id, "stale").await;
                break;
            }
        } else { missed = 0; }
    }
}
```

#### `opt/relay/src/backend.rs` (reqwest client to api.esluce.com)

**Analog:** None in repo (new). Standard `reqwest` pattern with HMAC-signed requests.

```rust
use reqwest::Client;
use uuid::Uuid;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct BackendClient {
    client: Client,
    base_url: String,
    hmac_secret: String,
}

impl BackendClient {
    pub fn new(base_url: String, hmac_secret: String) -> Self {
        Self { client: Client::new(), base_url, hmac_secret }
    }

    pub async fn introspect_token(&self, relay_token: Uuid, server_id: Uuid) -> Result<Option<Uuid>> {
        let body = serde_json::json!({ "relay_token": relay_token, "server_id": server_id });
        let body_str = body.to_string();
        let mut mac = HmacSha256::new_from_slice(self.hmac_secret.as_bytes())?;
        mac.update(body_str.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        let resp = self.client.post(format!("{}/api/v1/internal/relay/authorize", self.base_url))
            .header("X-Internal-Signature", sig)
            .body(body_str)
            .send().await?;
        if !resp.status().is_success() { return Ok(None); }
        let v: serde_json::Value = resp.json().await?;
        Ok(v["node_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()))
    }

    pub async fn notify_tunnel_event(&self, server_id: Uuid, node_id: Uuid, event: &str) -> Result<()> {
        let body = serde_json::json!({ "relay_token": "...", "server_id": server_id, "event": event });
        // ... + HMAC sign + POST /internal/relay/tunnel-event
        Ok(())
    }
}
```

#### `opt/relay/src/metrics.rs` (Prometheus)

**Analog:** `api/src/presentation/handlers/metrics_handlers.rs:1-61` (Axum metrics router pattern) — for the endpoint. The actual metric registration is standard `prometheus::register_int_counter!` etc.

```rust
use axum::response::IntoResponse;
use prometheus::{IntCounter, IntGauge, Histogram, HistogramOpts, register_int_counter, register_int_gauge, register_histogram, Encoder, TextEncoder};

lazy_static::lazy_static! {
    pub static ref RELAY_ACTIVE_TUNNELS: IntGauge = register_int_gauge!(
        "relay_active_tunnels_total", "Number of currently active relay tunnels"
    ).unwrap();
    pub static ref RELAY_BANDWIDTH_IN: IntCounter = register_int_counter!(
        "relay_bandwidth_in_bytes", "Total bytes received from players"
    ).unwrap();
    pub static ref RELAY_BANDWIDTH_OUT: IntCounter = register_int_counter!(
        "relay_bandwidth_out_bytes", "Total bytes sent to players"
    ).unwrap();
    pub static ref RELAY_LATENCY: Histogram = register_histogram!(
        HistogramOpts::new("relay_latency_seconds", "End-to-end latency")
            .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
    ).unwrap();
    pub static ref PLAYER_BYTES_IN: IntCounter = register_int_counter!(
        "relay_player_bytes_in_total", "Bytes received from players per session"
    ).unwrap();
    pub static ref PLAYER_BYTES_OUT: IntCounter = register_int_counter!(
        "relay_player_bytes_out_total", "Bytes sent to players per session"
    ).unwrap();
}

pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (axum::http::StatusCode::OK, [("content-type", "text/plain")], buffer)
}
```

#### `opt/relay/Caddyfile`

**Analog:** `opt/umami/Caddyfile:10-15` (security headers block) + `gateway/Caddyfile.prod:12-21` (the `(security_headers)` snippet).

```
(relay_security_headers) {
    header {
        X-Frame-Options DENY
        X-Content-Type-Options nosniff
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        Referrer-Policy "strict-origin-when-cross-origin"
        -Server
    }
}

relay.esluce.net {
    @wspath path /tunnel /metrics /healthz
    reverse_proxy @wspath relay-gateway:8443
    import relay_security_headers
}

*.play.esluce.net {
    @wspath path /tunnel
    reverse_proxy @wspath relay-gateway:8443
    import relay_security_headers
    # Port 25565 is NLB-only (raw TCP passthrough; Caddy doesn't see it)
    tls {
        dns route53 {
            # IAM role on the EC2 instance provides credentials
        }
    }
}
```

#### `opt/relay/docker-compose.yml`

**Analog:** `opt/umami/docker-compose.yml` (exact same shape — caddy + backend service on a private network, Caddy exposes 80/443).

```yaml
services:
  relay-gateway:
    build:
      context: .
      dockerfile: relay-gateway.Dockerfile
    container_name: relay-gateway
    restart: unless-stopped
    env_file: .env
    expose:
      - "8443"
      - "9100"  # Prometheus
    networks:
      - relay-net
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:8443/healthz"]
      interval: 30s
      timeout: 5s
      retries: 3

  caddy:
    build:
      context: .
      dockerfile: Caddy.Dockerfile
    container_name: relay-caddy
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
      - "443:443/udp"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      relay-gateway:
        condition: service_started
    networks:
      - relay-net

networks:
  relay-net:
    driver: bridge

volumes:
  caddy_data:
  caddy_config:
```

#### `opt/relay/Caddy.Dockerfile`

**Analog:** `opt/umami/docker-compose.yml:26` (uses `caddy:2` directly; for Route 53 DNS-01 we need the custom build with `caddy-dns/route53`).

```dockerfile
FROM caddy:builder AS builder
RUN xcaddy build \
    --with github.com/caddy-dns/route53

FROM caddy:2
COPY --from=builder /usr/bin/caddy /usr/bin/caddy
```

---

### Frontend (React — `app/`)

#### `app/src/components/ConnectivitySection.jsx` (EXTEND)

**Analog:** `app/src/components/ServerBackups.jsx` (per-server sub-section, `useState` + `useEffect` + `fetchApi` shape) and `app/src/components/StatusBadge.jsx` (color-state pattern for the 3 modes).

Add a new section below the existing connectivity section:
```jsx
import { useEffect, useState } from 'react'
import { fetchApi } from '../api/client'
import ConnectivityBadge from './ConnectivityBadge'
import TunnelHealthCard from './TunnelHealthCard'
import ModeOverrideDropdown from './ModeOverrideDropdown'

export default function ConnectivitySection({ serverId, onProbe }) {
    const [status, setStatus] = useState(null)
    const [tunnel, setTunnel] = useState(null)
    // ... existing fetch /servers/:id/connectivity on mount
    
    useEffect(() => {
        // NEW: fetch tunnel health every 30s
        const fetchTunnel = async () => {
            try {
                const t = await fetchApi(`/servers/${serverId}/connectivity/tunnel`)
                setTunnel(t)
            } catch { /* silent */ }
        }
        fetchTunnel()
        const interval = setInterval(fetchTunnel, 30_000)
        return () => clearInterval(interval)
    }, [serverId])

    return (
        <div className="card bg-base-100 shadow-xl">
            <div className="card-body">
                <h2 className="card-title">Connectivity</h2>
                <ConnectivityBadge status={status?.status} mode={status?.mode} />
                <TunnelHealthCard tunnel={tunnel} />
                <ModeOverrideDropdown serverId={serverId} current={status?.connectivity_mode_override} onChange={refetch} />
                {/* ... existing fields + addresses + audit log ... */}
            </div>
        </div>
    )
}
```

#### `app/src/components/TunnelHealthCard.jsx` (NEW)

**Analog:** `app/src/components/MetricsCard.jsx` (stat display).

```jsx
export default function TunnelHealthCard({ tunnel }) {
    if (!tunnel) return <div className="text-muted text-sm">Tunnel not established</div>
    return (
        <div className="grid grid-cols-3 gap-2 text-sm">
            <div><span className="text-muted">Latency:</span> {tunnel.latency_ms}ms</div>
            <div><span className="text-muted">Last heartbeat:</span> {tunnel.last_heartbeat_secs}s ago</div>
            <div><span className="text-muted">Uptime:</span> {tunnel.uptime_human}</div>
        </div>
    )
}
```

#### `app/src/components/ModeOverrideDropdown.jsx` (NEW)

**Analog:** `app/src/components/StatusBadge.jsx:1-37` (color-state pattern) for the dot + label.

```jsx
import { useState } from 'react'
import { fetchApi } from '../api/client'

export default function ModeOverrideDropdown({ serverId, current, onChange }) {
    const [saving, setSaving] = useState(false)
    const handle = async (mode) => {
        setSaving(true)
        try {
            if (mode === 'auto') {
                await fetchApi(`/servers/${serverId}/connectivity/mode-override`, { method: 'DELETE' })
            } else {
                await fetchApi(`/servers/${serverId}/connectivity/mode-override`, {
                    method: 'POST', body: JSON.stringify({ mode }),
                })
            }
            onChange?.()
        } finally { setSaving(false) }
    }
    return (
        <div className="flex gap-2">
            {['auto', 'relay', 'direct'].map((m) => (
                <button
                    key={m}
                    disabled={saving}
                    onClick={() => handle(m)}
                    className={current === m || (!current && m === 'auto')
                        ? 'btn btn-sm btn-primary' : 'btn btn-sm btn-ghost'}>
                    {m === 'auto' ? 'Auto' : m === 'relay' ? 'Force Relay' : 'Force Direct'}
                </button>
            ))}
        </div>
    )
}
```

#### `app/src/lib/api.js` (EXTEND)

**Analog:** `app/src/lib/api.js:100-115` (`serversApi` extension pattern).

Add at the end of `serversApi` (line 116):
```js
setModeOverride: (id, mode) => api.post(`/servers/${id}/connectivity/mode-override`, { mode }),
clearModeOverride: (id) => api.delete(`/servers/${id}/connectivity/mode-override`),
getTunnelHealth: (id) => api.get(`/servers/${id}/connectivity/tunnel`),
```

Or add a separate `relayApi` object (cleaner separation):
```js
export const relayApi = {
    setModeOverride: (id, mode) => api.post(`/servers/${id}/connectivity/mode-override`, { mode }),
    clearModeOverride: (id) => api.delete(`/servers/${id}/connectivity/mode-override`),
    getTunnelHealth: (id) => api.get(`/servers/${id}/connectivity/tunnel`),
}
```

---

## Shared Patterns

### WebSocket `NodeMessage` extension

**Source:** `api/src/presentation/ws/node_protocol.rs:5-135` (the `NodeMessage` enum with `#[serde(tag = "type")]` and per-variant `#[serde(rename = "...")]`).
**Apply to:** `node_protocol.rs` (add 4 new variants) and `node_ws_handler.rs` (add 3 new dispatch arms) and `src/agent_connection.rs:27-88` (mirror enum in agent for outbound messages).

Always add new variants as enum cases (no separate struct types) so `serde_json::from_str::<NodeMessage>` and `serde_json::to_string(&NodeMessage)` keep working unchanged.

### Per-tenant ownership check (V4 Access Control)

**Source:** `api/src/presentation/handlers/cron_task_handlers.rs:30-37` (the `server.user_id != auth_user.tenant_id → access denied` check at the start of every handler).
**Apply to:** `api/src/presentation/handlers/relay_handlers.rs` (every endpoint).

```rust
let server = state.server_repository.find_by_id(&server_id).await
    .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?
    .ok_or(AppError::NotFound)?;
if server.user_id != auth_user.tenant_id { return Err(AppError::Forbidden); }
```

### Background-task periodic loop (agent)

**Source:** `src/handlers/dns_watch.rs:31-65` (the `tokio::spawn` + `tokio::time::interval` + `running` flag pattern).
**Apply to:** `src/handlers/relay_client.rs` (heartbeat ticker + reconnect loop), and `src/handlers/relay_session.rs` (per-yamux-stream task).

The relay client's main loop is two-level:
- Outer: `connect_tunnel` with exponential backoff (mirror `agent_connection.rs:274-738`).
- Inner: yamux session driver + heartbeat ticker + per-stream spawners.

### Background-service start (backend)

**Source:** `api/src/bootstrap/mod.rs:49-58` (the `monitoring_service.start()` + `webhook_service.start()` spawn pattern) and `api/src/application/services/monitoring_service.rs:74-91` (the `start(self: Arc<Self>)` interval loop).
**Apply to:** `api/src/application/services/relay_service.rs` (if Phase 68 adds a periodic re-validation loop) — NOT the initial cut (initial cut is event-driven via WS dispatch).

### Send WS message to a specific node

**Source:** `api/src/presentation/ws/node_connection_manager.rs:89-102` (`send_to_node`).
**Apply to:** `api/src/application/services/relay_service.rs` (when dispatching `ModeOverrideChange` to the agent) and `api/src/presentation/handlers/relay_handlers.rs` (when user pins a mode).

```rust
manager.send_to_node(&node_id, &NodeMessage::ModeOverrideChange {
    server_id, override_mode, timestamp: now.to_rfc3339(),
}).await?;
```

### HMAC-signed backend ↔ gateway communication (V9)

**Source:** None in repo (new pattern). Pattern: shared secret in `RELAY_HMAC_SECRET` env var, HMAC-SHA256 over the request body, sent in `X-Internal-Signature` header. Alternative: mTLS via ACM PCA (heavier setup).
**Apply to:** `opt/relay/src/auth.rs` (signs all `/internal/relay/*` calls) and `api/src/presentation/handlers/relay_internal_handlers.rs` (verifies the signature).

### Redlock-style Redis nonce dedup (D-11)

**Source:** None in repo. Pattern: `SET key value NX EX 600` returns `Option<String>`; `Some` = new nonce accepted, `None` = replay.
**Apply to:** `opt/relay/src/auth.rs` (per-tunnel-connect nonce dedup, 10-min TTL) and `opt/relay/src/ratelimit.rs` (per-source-IP rate limit counter, 60s window).

### Outbound WS with custom headers (D-09, D-11)

**Source:** `src/agent_connection.rs:204-227` (`prepare_ws_url` + urlencode helper — though this only handles `api_key` query param, not headers).
**Apply to:** `src/handlers/relay_client.rs` (outbound to `wss://relay.esluce.net/tunnel` with `Authorization: Bearer`, `X-Relay-Nonce`, `X-Relay-Timestamp`).

Use the `IntoClientRequest` trait (tokio-tungstenite 0.26) with `http::Request::builder()` to set custom headers. See `RESEARCH.md:899-922` for the canonical pattern.

### Axum WebSocket upgrade with header inspection

**Source:** `api/src/presentation/handlers/node_ws_handler.rs:26-53` (`ws_node_handler`) — the `axum::extract::ws::WebSocketUpgrade` + `HeaderMap` + `Query` extractors pattern.
**Apply to:** `opt/relay/src/auth.rs` (`tunnel_upgrade` — reads `Authorization`, `X-Relay-Nonce`, `X-Relay-Timestamp` before calling `ws.on_upgrade`).

### Cloudflare A record lifecycle (Phase 51 carryforward + Phase 68 D-13)

**Source:** `src/handlers/dns.rs:62-86` (`handle_create_record`) + `:88+` (`handle_update_record`).
**Apply to:** `src/handlers/dns.rs` (add `handle_remove_record` for the `tunnel_disconnect` trigger, D-13).

The agent's `dns_watch` loop continues to call `create_dns_record` on `tunnel_reconnect` after re-probe (D-13).

### Per-tenant audit log (D-17, Phase 67 carryforward)

**Source:** `api/src/domain/entities/connectivity_audit_log.rs` (planned in 67-01) + `api/src/infrastructure/repositories/sqlx_connectivity_audit_log_repository.rs`.
**Apply to:** `api/src/application/services/relay_service.rs` (append audit row for every `tunnel_event` received from the gateway — `event_type = "relay.tunnel_event"`, `status = "ok"|"failed"`, `details = { uptime, bytes_in, bytes_out, active_streams, reason }`).

### Caddy + Route 53 DNS-01 for wildcard cert

**Source:** `opt/umami/Caddyfile` (Caddy 2 base config) + Phase 66 EC2 IAM role pattern (D-06).
**Apply to:** `opt/relay/Caddyfile` + `opt/relay/Caddy.Dockerfile` (custom build with `caddy-dns/route53` plugin) + `DEPLOY.md` (document the IAM role for `route53:ChangeResourceRecordSets` on zone `esluce.net` for `_acme-challenge.*` records only).

### Outbound WSS reconnect with exponential backoff

**Source:** `src/agent_connection.rs:254-258, 274-738` (the existing `initial_delay * multiplier → max_delay` pattern).
**Apply to:** `src/handlers/relay_client.rs` (D-04 — 1s → 30s, ±20% jitter; agent's existing `reconnect_initial_secs` / `reconnect_max_secs` / `reconnect_multiplier` config from `agent-config` crate is the same shape).

### "Don't Hand-Roll" (RESEARCH.md §"Don't Hand-Roll", applied to Phase 68)

| Reuse From | Don't Write Yourself |
|------------|----------------------|
| `tokio-tungstenite 0.26` (already in `Cargo.toml:27`) | a custom HTTP/1.1 upgrade + frame parser |
| `tokio-yamux 0.3.18` (new dep) | a custom length-prefixed binary protocol for stream multiplexing |
| `src/agent_connection.rs:254-258` reconnect loop | a custom timer wheel for backoff |
| `src/handlers/dns_watch.rs:18-80` background watcher | a custom periodic-task scheduler |
| `api/src/presentation/ws/node_connection_manager.rs:89-102` `send_to_node` | a custom WS dispatch for `ModeOverrideChange` |
| `api/src/domain/entities/cloudflare_settings.rs` (entire file) | a new `Route53Config` (we use Cloudflare for the gateway's outbound calls; Route 53 is only for the wildcard A record, configured once via AWS Console) |
| `api/src/domain/billing/webhooks.rs` + `discord_webhook_url` column (already on `servers`) | a custom SMTP/HTTP path for relay alerts (D-23) |
| `api/src/infrastructure/cache/redis.rs` (RedisPool) | a custom Redis client for nonce dedup + rate limit |
| `api/src/presentation/middleware/rate_limit.rs:95-128` (Redis SETEX pattern) | a custom in-memory rate limit counter |
| `opt/umami/Caddyfile` + `opt/umami/docker-compose.yml` | a custom Caddy config (same Caddyfile + docker-compose shape) |

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns + Code Examples + crates.io docs):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `opt/relay/Cargo.toml` | config | n/a | New crate |
| `opt/relay/src/main.rs` | bootstrap | lifecycle | New service, no in-repo gateway |
| `opt/relay/src/config.rs` | config | n/a | New service, no in-repo TOML loader |
| `opt/relay/src/auth.rs` | middleware | request-response | First use of relay-specific token+nonce+HMAC auth |
| `opt/relay/src/tunnel.rs` | ws-handler | ws-message + streaming | First WSS server with custom header validation in the project |
| `opt/relay/src/player.rs` | tcp-forwarder | streaming | First yamux-based TCP forwarder |
| `opt/relay/src/backend.rs` | http-client | request-response | First internal service-to-service HMAC-signed client |
| `opt/relay/src/session_log.rs` | service-helper | n/a | First Redis `RPUSH` ephemeral log |
| `opt/relay/relay-gateway.Dockerfile` | build-config | n/a | First multi-stage Rust binary Docker build for this project |
| `opt/relay/relay-gateway.toml` | config | n/a | New pattern |
| `opt/relay/.env.example` | config | n/a | New pattern |
| `opt/relay/DEPLOY.md` | docs | n/a | First AWS NLB+ALB+EC2+Route 53 manual setup doc |
| `app/src/components/InviteFriendsModal.jsx` | component | display | First QR-code + share UI |
| `api/src/presentation/handlers/connectivity_service.rs` (Phase 67) | service | classify + dispatch | First outbound probe origin on the backend (analog: `monitoring_service.rs` but does internal Docker check, not outbound network probe) |
| `app/src/components/ConnectivityAuditLog.jsx` (Phase 67) | component | list-display | First audit log UI in the project |

For these files, use:
- **Relay gateway `Cargo.toml`** — see `RESEARCH.md:148-194` (the canonical dependency list with versions).
- **Relay gateway `auth.rs`** — see `RESEARCH.md:466-589` (Pattern 1: Tunnel WebSocket Handshake) and `RESEARCH.md:1026-1050` (Redis nonce dedup code).
- **Relay gateway `tunnel.rs` / yamux integration** — see `RESEARCH.md:934-956` (tokio-yamux 0.3 client) + `RESEARCH.md:957-1024` (Axum 0.7 WS upgrade with custom header validation).
- **Relay gateway `player.rs` / TCP forwarder** — see `RESEARCH.md:592-661` (Pattern 2: Player TCP → Yamux Stream Forwarder).
- **Relay gateway `registry.rs` / heartbeat watchdog** — see `RESEARCH.md:696-741` (Pattern 4: Heartbeat Watchdog + Stale Tunnel Detection).
- **Relay gateway `session_log.rs` / Redis ephemeral log** — see `RESEARCH.md:743-774` (Pattern 5: Tunnel Session Log).
- **Relay gateway `backend.rs` / HMAC client** — see `RESEARCH.md:1287-1308` (security domain, V9 communication) for the HMAC signing pattern.
- **Relay gateway `metrics.rs` / Prometheus** — see `RESEARCH.md:1087-1123` (Prometheus registration + Axum `/metrics` handler).
- **AWS deployment (`DEPLOY.md`)** — see `.planning/phases/66-*/DEPLOY.md` for the same shape (EC2 + Docker + Caddy + RDS pattern), adapted for NLB+ALB+Route 53 instead of RDS.
- **Caddy + Route 53 DNS-01** — see `RESEARCH.md:114-115` (the `caddy-dns/route53` plugin) + `.planning/phases/66-*/66-CONTEXT.md` (D-06 declarative manual setup).
- **Agent `relay_client.rs` reconnect loop** — see `RESEARCH.md:472-531` (Pattern 1 example) + `src/agent_connection.rs:254-258` (existing pattern).
- **Agent `relay_session.rs` yamux stream forwarder** — see `RESEARCH.md:663-694` (Pattern 3: Agent Yamux Stream → Local Minecraft TCP).
- **Frontend `InviteFriendsModal.jsx`** — see `RESEARCH.md:268-291` (the "Invite friends" UI mock from CONTEXT) + any standard `qrcode.react` npm library for the QR code.

---

## Metadata

**Analog search scope:** `src/handlers/`, `src/agent_connection.rs`, `src/main.rs`, `src/audit.rs`, `Cargo.toml`, `api/migrations/`, `api/src/domain/{entities,repositories}/`, `api/src/infrastructure/{cache,repositories}/`, `api/src/presentation/{handlers,ws,middleware,routes,responses}/`, `api/src/application/services/`, `api/src/bootstrap/`, `gateway/Caddyfile.prod`, `opt/umami/`, `app/src/{components,hooks,pages,lib}/`

**Files scanned:** ~50
**Pattern extraction date:** 2026-06-07
**Confidence:** HIGH for agent + backend tiers (all have direct in-repo analogs); MEDIUM for the relay gateway (no in-repo analog, but RESEARCH.md provides detailed patterns with crates.io verification); HIGH for the frontend (mirror of Phase 67's ConnectivitySection pattern).

**Key cross-tier carryover from Phase 67:**
- The `NodeMessage` enum extension pattern is identical (Phase 67 added `ConnectivityReport`/`ConnectivityFixRequest`/`ConnectivityFixResult`; Phase 68 adds `TunnelConnect`/`TunnelDisconnect`/`TunnelHeartbeat`/`ModeOverrideChange`).
- The `servers.connectivity_*` columns from Phase 67 are extended, not replaced (Phase 68 adds `relay_status` + `last_tunnel_connected_at` to the same row).
- The `dns_watch` pattern in the agent is the model for `relay_client` (background watcher + interval ticker + state mutation).
- The `cron_task_handlers` ownership check pattern is reused for `relay_handlers` (mode override).
- The `app/components/ConnectivitySection.jsx` from Phase 67 is the parent that Phase 68 extends (adds TunnelHealthCard + ModeOverrideDropdown + InviteFriendsModal).

**Key new architectural pattern:**
- The **two-WebSocket per agent** model: existing `wss://api.esluce.com/api/ws/node` (control plane) + new `wss://relay.esluce.net/tunnel` (data plane). Both outbound-only from the agent. The `RegisterAck` carries the new `relay_token` so the same agent can connect to both.
- The **gateway-as-stateless-forwarder** model: relay holds NO state in PostgreSQL (only ephemeral in-memory `DashMap<Uuid, TunnelEntry>` + Redis). All persistent state (token, server mapping) is on the backend; the gateway is a pure network function.
- The **HMAC-signed internal API** model: the gateway's `POST /internal/relay/authorize` call is signed with a shared secret (env var on both sides). This is the project's first use of HMAC for service-to-service auth; future internal services should follow the same pattern.
