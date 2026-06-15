# Phase 67: Agent auto-resolve Minecraft port reachability issues - Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 19 new/modified
**Analogs found:** 17 / 19

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| **Agent (Rust — `src/`)** | | | | |
| `src/handlers/connectivity.rs` | handler-orchestrator | request-response / event-driven | `src/handlers/dns.rs` | exact |
| `src/handlers/connectivity/mod.rs` | module-root | n/a | `src/handlers/dns.rs` module pattern | exact |
| `src/handlers/connectivity/diagnostics.rs` | service-collector | collect/transform | `src/handlers/dns_watch.rs:132-155` (`detect_public_ip`) | role-match |
| `src/handlers/connectivity/firewall.rs` | service-executor | shell-out | `src/handlers/runtime.rs:213-220` (`Command::new("docker")` shell-out) | role-match |
| `src/handlers/connectivity/upnp.rs` | service-executor | IGD control | (no analog — new) | no-analog |
| `src/handlers/mod.rs` (extend) | dispatcher | request-response | `src/handlers/mod.rs:118-166` (`execute_single`) | exact |
| `src/main.rs` (extend startup) | bootstrap | lifecycle | `src/main.rs:282-293` (DnsWatcher spawn pattern) | exact |
| `Cargo.toml` (extend deps) | config | n/a | `Cargo.toml:36-75` existing deps | exact |
| **Backend API (Rust — `api/`)** | | | | |
| `api/migrations/20260607000001_add_connectivity_columns.sql` | migration | n/a | `api/migrations/20260307000001_add_enhanced_server_features.sql` | exact |
| `api/migrations/20260607000002_create_connectivity_audit_log.sql` | migration | n/a | `api/migrations/20260531000002_create_server_crash_logs.sql` | exact |
| `api/src/domain/entities/connectivity_audit_log.rs` | entity | n/a | `api/src/domain/entities/server_crash_log.rs` | exact |
| `api/src/domain/entities/mod.rs` (extend) | module-root | n/a | `api/src/domain/entities/mod.rs:15` (`server_crash_log` already registered) | exact |
| `api/src/domain/repositories/connectivity_audit_log_repository.rs` | repository-trait | CRUD | `api/src/domain/repositories/cron_task_repository.rs` | exact |
| `api/src/domain/repositories/mod.rs` (extend) | module-root | n/a | `api/src/domain/repositories/mod.rs:11` | exact |
| `api/src/infrastructure/repositories/sqlx_connectivity_audit_log_repository.rs` | repository-impl | CRUD | `api/src/infrastructure/repositories/crash_log_repository.rs` | exact |
| `api/src/infrastructure/repositories/mod.rs` (extend) | module-root | n/a | `api/src/infrastructure/repositories/mod.rs:14` | exact |
| `api/src/presentation/ws/node_protocol.rs` (extend) | ws-protocol | ws-message | `api/src/presentation/ws/node_protocol.rs:74-80` (existing `CrashReport` variant) | exact |
| `api/src/presentation/handlers/node_ws_handler.rs` (extend) | ws-handler | ws-message | `api/src/presentation/handlers/node_ws_handler.rs:397-414` (existing `CrashReport` handler case) | exact |
| `api/src/presentation/handlers/connectivity_handlers.rs` | rest-handler | request-response | `api/src/presentation/handlers/cron_task_handlers.rs` | exact |
| `api/src/presentation/services/connectivity_service.rs` | service | probe/transform | `api/src/application/services/monitoring_service.rs` (background-service pattern) | role-match |
| `api/src/application/services/mod.rs` (extend) | module-root | n/a | `api/src/application/services/mod.rs` (existing exports) | exact |
| `api/src/presentation/routes/api_routes.rs` (mount) | router | n/a | `api/src/presentation/routes/api_routes.rs:34-37` (per-server nested routes pattern) | exact |
| `api/src/bootstrap/container.rs` (extend) | DI-container | n/a | `api/src/bootstrap/container.rs:140-151` (crash_report_tx wiring) | exact |
| **Frontend (React — `app/`)** | | | | |
| `app/src/components/ConnectivityBadge.jsx` | component | display | `app/src/components/StatusBadge.jsx` | exact |
| `app/src/components/ConnectivitySection.jsx` | component | data-display | `app/src/components/ServerBackups.jsx` (per-server sub-section) | role-match |
| `app/src/components/ConnectivityAuditLog.jsx` | component | list-display | (no analog — new) | no-analog |
| `app/src/hooks/useConnectivity.js` | hook | data-fetching | `app/src/hooks/useAlerts.js` | exact |
| `app/src/lib/api.js` (extend) | api-client | n/a | `app/src/lib/api.js:100-115` (`serversApi` extension pattern) | exact |
| `app/src/pages/servers/ServerManagerPage.jsx` (modify) | page | list | `app/src/pages/servers/ServerManagerPage.jsx:25-43` (existing `getStatusColor`) | exact |
| `app/src/pages/servers/ServerDetailsPage.jsx` (modify) | page | detail | `app/src/pages/servers/ServerDetailsPage.jsx:33-39` (existing tabs pattern) | exact |

---

## Pattern Assignments

### `src/handlers/connectivity.rs` (handler-orchestrator, request-response / event-driven)

**Analog:** `src/handlers/dns.rs` (entire file) — same `Task`-in → `Result<serde_json::Value>` shape, same `DNS_CONFIG` lazy_static pattern, same `handle_configure` / `handle_create_record` / `handle_update_record` per-task entrypoint shape.

**Module declaration pattern** (`src/handlers/mod.rs:5-13`):
```rust
pub mod runtime;
pub mod backup;
pub mod rcon;
pub mod metrics;
pub mod ssh;
pub mod sftp;
pub mod dns;
pub mod dns_watch;
pub mod files;
```
Add: `pub mod connectivity;` — and expose its submodule `pub mod connectivity { pub mod diagnostics; pub mod firewall; pub mod upnp; }` if a submodule folder is preferred (matches the flat-module style currently used).

**Per-task entrypoint shape** (`src/handlers/dns.rs:44-60`):
```rust
pub async fn handle_configure(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let payload = task.payload.clone();
    let config: CloudflareDnsConfig = serde_json::from_value(payload)
        .map_err(|e| anyhow!("Invalid DNS config payload: {}", e))?;
    // ... state mutation, log, return json!({...})
    Ok(json!({ "status": "configured", ... }))
}
```
Apply this shape to: `handle_diagnostics`, `handle_open_port`, `handle_close_port`, `handle_upnp_add`, `handle_upnp_remove`.

**Shared mutable state pattern** (`src/handlers/dns.rs:24-29`):
```rust
lazy_static! {
    pub static ref DNS_CONFIG: Arc<RwLock<Option<CloudflareDnsConfig>>> =
        Arc::new(RwLock::new(None));
    pub static ref CURRENT_IP: Arc<RwLock<String>> =
        Arc::new(RwLock::new(String::new()));
}
```
Reuse for connectivity: `CURRENT_DIAGNOSTICS: Arc<RwLock<HashMap<Uuid, serde_json::Value>>>` keyed by server_id, plus `FIREWALL_AUTO_MANAGE: Arc<RwLock<bool>>` for the install-time opt-in (D-09).

**Wiring into the dispatcher** (`src/handlers/mod.rs:118-166`):
```rust
// Add new arm inside the match on task_type in execute_single
"connectivity.diagnostics" => connectivity::handle_diagnostics(task.clone()).await,
"firewall.open_port"      => connectivity::firewall::open(task.clone()).await,
"firewall.close_port"     => connectivity::firewall::close(task.clone()).await,
"upnp.add_mapping"        => connectivity::upnp::add(task.clone()).await,
"upnp.remove_mapping"     => connectivity::upnp::remove(task.clone()).await,
```
And extend `get_task_config` (`src/handlers/mod.rs:186-294`) with new task_config entries (60s default timeout, 0 retries, 0 backoff — these are short shell-outs).

**Audit-log call site** (`src/handlers/mod.rs:36-43`, `src/audit.rs:23-42`):
The agent-side audit is in-memory via `audit::log_task_received/completed/failed`. The backend-side per-server `connectivity_audit_log` row is written when the `ConnectivityReport` / `ConnectivityFixResult` WS message arrives at `node_ws_handler.rs` (see backend patterns below) — NOT from the agent's own `audit.rs`. The agent should still log via `tracing::info!` with the exact command string for the local log file (D-17).

---

### `src/handlers/connectivity/diagnostics.rs` (service-collector, collect/transform)

**Analog:** `src/handlers/dns_watch.rs:132-155` (`detect_public_ip`) — the closest function to "collect one raw fact from the host".

**Imports pattern** (mirror `dns_watch.rs:1-9`):
```rust
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use tracing::{error, info, warn};
use crate::handlers::dns_watch::detect_public_ip;  // REUSE — do not re-implement
```

**Reuse pattern — public IP** (`src/handlers/dns_watch.rs:132-155`):
```rust
pub async fn detect_public_ip() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    for url in IP_CHECK_URLS { /* ... 4 fallback providers ... */ }
    Err(anyhow::anyhow!("Failed to detect public IP from all sources"))
}
```
The Phase 67 `diagnostics.rs::collect_diagnostics` MUST call this function, not re-implement it. Add it to the returned JSON envelope under key `public_ip`.

**Bollard container inspection pattern** (`src/handlers/runtime.rs:186-193`):
```rust
if let Ok(info) = docker.inspect_container(id, None).await {
    if let Some(state) = info.state {
        if state.running == Some(true) { /* ... */ }
    }
}
```
Apply the same `inspect_container(server_id_str, None)` call to read `info.network_settings.ports` for `port_bound` reclassification. The server_id-to-container-name mapping is `format!("mc-{}", server_id)` (same convention as `runtime.rs:204-205`).

**which() tool detection pattern** (`agent/agent-core/crates/agent-runtime/src/detector.rs:41, 72`):
```rust
let docker_path = which::which("docker").ok()?;
let output = Command::new(&docker_path).args(["version", ...]).output().ok()?;
```
Apply for `ufw`, `firewalld`, `iptables`, `nft`, `tailscale`, `cloudflared` — wrap in a helper:
```rust
pub fn tool_present(name: &str) -> bool { which::which(name).is_ok() }
```

**CGN heuristic** — no existing code (new). Pattern: small pure function in `diagnostics.rs`:
```rust
pub fn is_cgnat_suspect(local_ip: Option<std::net::Ipv4Addr>, gateway: Option<std::net::Ipv4Addr>) -> bool {
    let in_cgn = |ip: std::net::Ipv4Addr| -> bool {
        let o = ip.octets();
        o[0] == 100 && o[1] >= 64 && o[1] <= 127   // RFC 6598: 100.64.0.0/10
    };
    local_ip.map(in_cgn).unwrap_or(false) || gateway.map(in_cgn).unwrap_or(false)
}
```

**Default gateway via `ip route`** — no existing code; the shell-out pattern follows `runtime.rs:213-220`:
```rust
let output = tokio::process::Command::new("ip")
    .args(["route", "show", "default"])
    .output().await
    .context("Failed to run ip route")?;
// parse first column after "via"
```

**Tailscale / Cloudflared detection** — no existing code; the pattern is `which` + try `subcommand --json` and treat `status.success()` as "up":
```rust
let tailscale_up = which::which("tailscale").is_ok()
    && tokio::process::Command::new("tailscale")
        .args(["status", "--json"]).output().await
        .map(|o| o.status.success()).unwrap_or(false);
```
(Falls under D-11/D-12 "detect only" — never install.)

**Tailscale IP extraction** (referenced in `.planning/debug/server-details-wrong-address-version-status.md:139-145` — `tailscale ip -4`):
```rust
let tailscale_ip = tokio::process::Command::new("tailscale")
    .args(["ip", "-4"]).output().await
    .ok().and_then(|o| {
        if o.status.success() { String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string()) }
        else { None }
    });
```

---

### `src/handlers/connectivity/firewall.rs` (service-executor, shell-out)

**Analog:** `src/handlers/runtime.rs:213-220` (the `docker ps` shell-out, the closest in-repo `tokio::process::Command` pattern).

**Shell-out pattern** (`src/handlers/runtime.rs:213-220`):
```rust
use tokio::process::Command;
let output = Command::new("docker")
    .args(["ps", "-a", "--filter", &format!("name=^{}$", container_name), "--format", "{{.ID}}"])
    .output().await
    .context("Failed to run docker ps")?;
let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
```
Apply: `Command::new("iptables").args(["-I", "INPUT", "-p", "tcp", "--dport", &port.to_string(), "-m", "comment", "--comment", &format!("esluse:{}", server_id), "-j", "ACCEPT"]).output().await`.

**Tool-priority helper (pattern after `RuntimeDetector::detect_docker` / `detect_podman` in `agent-runtime/src/detector.rs:40-94`)**:
```rust
fn pick_firewall_cli() -> Option<(&'static str, &'static str)> {
    if which::which("ufw").is_ok()       { Some(("ufw", "ufw allow")) }
    else if which::which("firewalld").is_ok() { Some(("firewalld", "firewall-cmd")) }
    else if which::which("iptables").is_ok()  { Some(("iptables", "iptables")) }
    else if which::which("nft").is_ok()       { Some(("nft", "nft")) }
    else { None }
}
```

**Cleanup loop pattern (Pitfall 3 — iptables comment-match races)**: re-read the chain via `iptables -S INPUT | grep "esluse:<server-id>"` and delete each match explicitly. Loop pattern is the same as `runtime.rs:79-105` (`ensure_image_exists` retry loop):
```rust
for attempt in 1..=max_attempts {
    match run_close(&server_id, port).await { /* ... */ }
}
```

**Persistence call (Pitfall 7)**: append `; netfilter-persistent save` (Debian/Ubuntu) or `firewall-cmd --runtime-to-permanent` (RHEL) to the audit-logged command string. For `ufw`, persistence is built-in.

---

### `src/handlers/connectivity/upnp.rs` (service-executor, IGD control)

**No existing analog.** This is the first use of the `upnp-rs` crate. The general pattern follows `runtime.rs:108` for getting a resource handle, then `runtime.rs:127-130` for issuing the operation and returning a `serde_json::Value` result envelope.

**Result envelope shape** (mirroring `runtime.rs:170-173`):
```rust
Ok(serde_json::json!({
    "mapping_id": mapping_id,
    "external_port": port,
    "internal_port": port,
    "lease_seconds": LEASE_SECS,
    "expires_at": (Utc::now() + chrono::Duration::seconds(LEASE_SECS as i64)).to_rfc3339(),
}))
```

**Background renewal** (D-08 — re-extend at 50% of lease): mirror `tokio::spawn` pattern from `dns_watch.rs:43-64`:
```rust
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(LEASE_SECS as u64 / 2)).await;
    let _ = add_port_mapping(port, protocol).await;
});
```

**Graceful absence** (Pitfall 4 — VPS detection): if `local_ip` is None or not in RFC 1918, return `Ok(json!({ "skipped": true, "reason": "no_upnp_in_vps" }))` and skip the call. Use the same `is_cgnat_suspect` helper from `diagnostics.rs` plus a `is_lan()` inverse helper.

---

### `src/main.rs` (extend startup — add ConnectivityMonitor)

**Analog:** `src/main.rs:282-293` (DnsWatcher startup) — exact pattern to copy.

**DnsWatcher startup pattern** (`src/main.rs:282-293`):
```rust
// 9. Start DNS watcher (DDNS-like auto-refresh)
let dns_watcher = Arc::new(handlers::dns_watch::DnsWatcher::new());
dns_watcher.start().await;

let watcher_for_shutdown = dns_watcher.clone();
let shutdown_clone2 = shutdown.clone();
tokio::spawn(async move {
    while !shutdown_clone2.load(Ordering::Relaxed) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    watcher_for_shutdown.stop().await;
});
```
Add: same shape for `ConnectivityMonitor` immediately after the DNS watcher block. The monitor's `start()` re-emits a `ConnectivityReport` WS message on diagnostic delta only (D-04 raw-facts reduction — CONTEXT specifies to avoid heartbeat bloat).

**Periodic-loop pattern** (mirroring `dns_watch.rs:51-64`):
```rust
let mut ticker = interval(*check_interval.read().await);
loop {
    ticker.tick().await;
    if !*running.read().await { break; }
    if let Err(e) = collect_and_maybe_emit().await {
        error!("ConnectivityMonitor tick failed: {}", e);
    }
}
```

---

### `api/migrations/20260607000001_add_connectivity_columns.sql` (migration)

**Analog:** `api/migrations/20260307000001_add_enhanced_server_features.sql` (entire file) — exact pattern. Add three new columns to `servers`:

```sql
ALTER TABLE servers
  ADD COLUMN IF NOT EXISTS connectivity_status TEXT NOT NULL DEFAULT 'unknown',
  ADD COLUMN IF NOT EXISTS connectivity_mode   TEXT NOT NULL DEFAULT 'direct',
  ADD COLUMN IF NOT EXISTS last_probe_at      TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS connectivity_details JSONB DEFAULT '{}'::jsonb;
```
`ADD COLUMN IF NOT EXISTS` matches the non-breaking precedent of the analog migration. Use `TEXT` not `ENUM` (precedent: `connectivity_status` free-form to allow forward compatibility with new states).

---

### `api/migrations/20260607000002_create_connectivity_audit_log.sql` (migration)

**Analog:** `api/migrations/20260531000002_create_server_crash_logs.sql` (entire file) — exact pattern, just larger column set. Also mirror the immutability pattern from `20260325000001_make_audit_logs_immutable.sql` if immutability is required (D-17 audit log is append-only, A9 in RESEARCH).

```sql
CREATE TABLE IF NOT EXISTS connectivity_audit_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id   UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    node_id     UUID REFERENCES nodes(id),
    event_type  TEXT NOT NULL,    -- 'connectivity.diagnostics' | 'firewall.open_port' | 'upnp.add_mapping' | 'connectivity.probe' | etc.
    command     TEXT,             -- exact shell command or action description
    status      TEXT NOT NULL,    -- 'ok' | 'failed' | 'attempted' | 'reachable' | 'unreachable'
    details     JSONB DEFAULT '{}'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_connectivity_audit_log_server_ts
    ON connectivity_audit_log (server_id, created_at DESC);
```
Add the immutability trigger block from `20260325000001_make_audit_logs_immutable.sql:5-27` (D-17).

---

### `api/src/domain/entities/connectivity_audit_log.rs` (entity)

**Analog:** `api/src/domain/entities/server_crash_log.rs` (entire file) — exact pattern (16 lines, `FromRow` + `Serialize/Deserialize` + chrono + uuid).

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConnectivityAuditLog {
    pub id: Uuid,
    pub server_id: Uuid,
    pub node_id: Option<Uuid>,
    pub event_type: String,
    pub command: Option<String>,
    pub status: String,
    pub details: serde_json::Value,  // defaults to '{}'::jsonb in DB
    pub created_at: DateTime<Utc>,
}
```
**Registration** in `api/src/domain/entities/mod.rs` — add: `pub mod connectivity_audit_log;` (alongside `pub mod server_crash_log;` at line 15).

---

### `api/src/domain/repositories/connectivity_audit_log_repository.rs` (trait)

**Analog:** `api/src/domain/repositories/cron_task_repository.rs` (look up for shape) and `api/src/infrastructure/repositories/crash_log_repository.rs:16-93` (concrete methods to mirror). Use `async_trait` (precedent: `api/src/infrastructure/repositories/postgres_settings_repository.rs:20-37`).

```rust
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::connectivity_audit_log::ConnectivityAuditLog;

#[async_trait]
pub trait ConnectivityAuditLogRepository: Send + Sync {
    async fn insert(&self, log: &ConnectivityAuditLog) -> anyhow::Result<()>;
    async fn list_by_server(&self, server_id: Uuid, limit: i64, offset: i64) -> anyhow::Result<Vec<ConnectivityAuditLog>>;
    async fn count_by_server(&self, server_id: Uuid) -> anyhow::Result<i64>;
}
```

**Registration** in `api/src/domain/repositories/mod.rs` — add: `pub mod connectivity_audit_log_repository;`.

---

### `api/src/infrastructure/repositories/sqlx_connectivity_audit_log_repository.rs` (impl)

**Analog:** `api/src/infrastructure/repositories/crash_log_repository.rs` (entire file, 107 lines) — exact pattern. Direct copy with field rename `crash_type → event_type`, `crashed_at → created_at`.

Constructor + methods (mirroring `crash_log_repository.rs:7-69`):
```rust
pub struct PostgresConnectivityAuditLogRepository { pool: PgPool }

impl PostgresConnectivityAuditLogRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn insert(&self, log: &ConnectivityAuditLog) -> Result<()> { /* ... */ }
    pub async fn list_by_server(&self, server_id: Uuid, limit: i64, offset: i64) -> Result<Vec<ConnectivityAuditLog>> { /* ... */ }
    pub async fn count_by_server(&self, server_id: Uuid) -> Result<i64> { /* ... */ }
}
```

**Registration** in `api/src/infrastructure/repositories/mod.rs` — add: `pub mod sqlx_connectivity_audit_log_repository;` (alongside `pub mod crash_log_repository;` at line 14).

---

### `api/src/presentation/ws/node_protocol.rs` (extend — new WS message variants)

**Analog:** `api/src/presentation/ws/node_protocol.rs:74-80` (existing `CrashReport` variant) — exact pattern for the agent→backend direction. Plus `api/src/presentation/ws/node_protocol.rs:83-92` (`ExecuteCommand` variant) for backend→agent direction.

**Add to the `NodeMessage` enum** (agent → backend, modeled on `CrashReport`):
```rust
// Phase 67: Connectivity diagnostics
#[serde(rename = "connectivity_report")]
ConnectivityReport {
    server_id: Uuid,
    node_id: Uuid,
    diagnostics: serde_json::Value,  // raw facts: public_ip, local_ip, port_bound, firewall_active, default_gateway, is_cgn_suspect, tailscale_up, cloudflared_up
    timestamp: String,
},

#[serde(rename = "connectivity_fix_result")]
ConnectivityFixResult {
    request_id: Uuid,
    server_id: Uuid,
    action: String,             // "firewall.open_port" | "upnp.add_mapping" | "recreate_container_with_port_bindings"
    success: bool,
    command: String,            // exact command run (D-17)
    output: String,
    details: Option<serde_json::Value>,
    timestamp: String,
},
```

**Add to the `NodeMessage` enum** (backend → agent, modeled on `ExecuteCommand`):
```rust
// Phase 67: Backend requests agent to attempt a fix
#[serde(rename = "connectivity_fix_request")]
ConnectivityFixRequest {
    request_id: Uuid,
    server_id: Uuid,
    action: String,             // "firewall.open_port" | "firewall.close_port" | "upnp.add_mapping" | "upnp.remove_mapping" | "recreate_container_with_port_bindings"
    params: serde_json::Value,  // { port, proto, server_id, lease_secs, ... }
},
```

---

### `api/src/presentation/handlers/node_ws_handler.rs` (extend — new dispatch arms)

**Analog:** `api/src/presentation/handlers/node_ws_handler.rs:397-414` (existing `CrashReport` handler case) — exact pattern for `ConnectivityReport`.

**CrashReport handler** (`node_ws_handler.rs:397-414`):
```rust
NodeMessage::CrashReport { server_id, exit_code, log_excerpt, timestamp: _ } => {
    tracing::warn!("[CRASH] Received crash report: server={}, exit_code={}", server_id, exit_code);
    if let Some(ref tx) = container.crash_report_tx {
        let data = CrashReportData { server_id, exit_code, log_excerpt };
        if let Err(e) = tx.try_send(data) { tracing::error!("[CRASH] Failed to forward crash report: {}", e); }
    } else {
        tracing::warn!("[CRASH] Crash report channel not available");
    }
}
```

**Add analogous cases** inside the `match node_msg` block:
```rust
NodeMessage::ConnectivityReport { server_id, node_id, diagnostics, timestamp: _ } => {
    // Persist the raw diagnostics in servers.connectivity_details
    // and insert a connectivity_audit_log row with event_type="connectivity.diagnostics"
    // Then kick off a probe (probe scheduler lives in connectivity_service)
    if let Err(e) = container.connectivity_service
        .handle_agent_diagnostics(server_id, node_id, diagnostics).await {
        tracing::error!("[CONNECTIVITY] Failed to handle diagnostics: {}", e);
    }
}

NodeMessage::ConnectivityFixResult { request_id, server_id, action, success, command, output, details, timestamp: _ } => {
    // Persist a connectivity_audit_log row (status = success ? "ok" : "failed")
    // If success, schedule a re-probe
    if let Err(e) = container.connectivity_service
        .handle_fix_result(request_id, server_id, &action, success, &command, &output, details).await {
        tracing::error!("[CONNECTIVITY] Failed to handle fix result: {}", e);
    }
}
```

The service-layer method `connectivity_service::handle_agent_diagnostics` is responsible for triggering the probe + classifying + dispatching the auto-fix + storing the row — keeping the handler non-blocking (CONTEXT pitfall note: "Synchronous re-probe blocking the WS handler" must be avoided). See pattern below.

**Container wiring** — add to `AppContainer` struct (`api/src/bootstrap/container.rs:140-151`):
```rust
pub connectivity_audit_log_repository: Arc<PostgresConnectivityAuditLogRepository>,
pub connectivity_service: Arc<ConnectivityService<...>>,   // see service pattern below
```

**Wiring in `AppContainer::new`** (`container.rs:328-342`, the crash_report_tx precedent):
```rust
// Phase 67: Connectivity audit log repository
let connectivity_audit_log_repository = Arc::new(
    PostgresConnectivityAuditLogRepository::new(pool.clone())
);
let connectivity_service = Arc::new(ConnectivityService::new(
    repo.clone(),
    node_repo.clone(),
    node_connection_manager.clone(),
    connectivity_audit_log_repository.clone(),
    discord_client.clone(),
    redis_pool.clone(),  // for the 30s manual-probe cooldown
));
```

---

### `api/src/presentation/handlers/connectivity_handlers.rs` (new REST handler)

**Analog:** `api/src/presentation/handlers/cron_task_handlers.rs` (entire file, 196 lines) — exact pattern. Same `router()` builder returning `Router<ApiState>`, same `auth_user: AuthUser` extractor, same `server.user_id != auth_user.tenant_id` ownership check (V4 Access Control per RESEARCH Security Domain), same `ApiResponse::success(...)` envelope.

**Router builder pattern** (`cron_task_handlers.rs:17-23`):
```rust
pub fn router(state: ApiState) -> Router<ApiState> {
    Router::new()
        .route("/:server_id/tasks", get(list_tasks).post(create_task))
        .route("/:server_id/tasks/:task_id", patch(update_task).delete(delete_task))
        .route("/:server_id/tasks/:task_id/run", post(run_task))
        .with_state(state)
}
```

**New endpoints**:
```rust
.route("/:server_id/connectivity", get(get_status))           // GET — current status + last probe + mode
.route("/:server_id/connectivity/probe", post(trigger_probe)) // POST — manual "Reachable" button
.route("/:server_id/connectivity/audit", get(get_audit_log))  // GET — paginated audit log
```

**Trigger-probe handler skeleton** (mirroring `cron_task_handlers.rs:25-44`):
```rust
pub async fn trigger_probe(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let server = state.server_repository.find_by_id(&server_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?
        .ok_or_else(|| AppError::NotFound)?;
    if server.user_id != auth_user.tenant_id { return Err(AppError::Forbidden); }

    // Rate-limit (30s cooldown per server) via Redis
    if let Some(ref redis) = state.redis_pool {
        let key = format!("connectivity:probe_cooldown:{}", server_id);
        // try SET with NX EX 30 → if exists, return 429
    }

    let result = state.connectivity_service.probe_server(server_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;
    Ok(Json(ApiResponse::success(result)))
}
```

---

### `api/src/presentation/services/connectivity_service.rs` (service)

**No exact analog** (this is the first probe-originating service on the backend). The closest precedent is `api/src/application/services/monitoring_service.rs:74-91` (background-service startup with `tokio::time::interval` and `start(self: Arc<Self>)`) and `monitoring_service.rs:455-502` (full crash lifecycle: classify → persist → notify → execute recovery).

**Background-service startup pattern** (`monitoring_service.rs:74-91`):
```rust
pub async fn start(self: Arc<Self>) {
    tracing::info!("Starting Background Monitoring Service...");
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    interval.tick().await;
    loop {
        interval.tick().await;
        if let Err(e) = self.check_all_servers().await { tracing::error!("Monitoring Loop Error: {}", e); }
    }
}
```
The Phase 67 connectivity service starts the same way with a 5-min interval for the periodic re-probe of all running servers. Spawned in `bootstrap/mod.rs` next to where `monitoring_service.start()` is spawned.

**Classification function shape** (`monitoring_service.rs:455-502`, `handle_crash_report`):
```rust
async fn handle_crash_report(&self, report: CrashReportData) -> Result<()> {
    let server = self.repository.find_by_id(&report.server_id).await?
        .ok_or_else(|| anyhow::anyhow!("Server {} not found", report.server_id))?;
    let crash_type = crash_classifier::classify_crash(report.exit_code, &report.log_excerpt);
    let recovery_action = match crash_type { /* ... */ };
    self.store_crash_log(&server, /* ... */).await?;
    match crash_type { /* ... execute recovery ... */ }
    Ok(())
}
```
Mirror this for `handle_agent_diagnostics`:
1. Look up server (404 if missing).
2. Persist raw diagnostics to `servers.connectivity_details` and append a `connectivity_audit_log` row.
3. Trigger a probe (TCP + SLP) from the backend to `(public_ip, port)`.
4. Classify probe outcome + diagnostics → `FailureMode` enum (4 MVP variants).
5. If `FailureMode::auto_fixable()`, dispatch a `NodeMessage::ConnectivityFixRequest` via `node_connection_manager.send_to_node()`.
6. If not auto-fixable, render the hybrid failure report and emit alert (D-13).

**TCP+SLP probe function** — **no analog** in the codebase. See RESEARCH Code Examples (lines 545-602) for the Java Edition handshake + status request parser. Bedrock Edition RakNet ping (lines 605-680) is a separate function. Pick protocol from `server.mc_loader` (Java vs "bedrock") with port-based fallback (25565 vs 19132) per Pitfall 6.

**Minecraft protocol crate usage** — none (deliberate per "Don't Hand-Roll" row 3). Implement inline, ~80 lines for Java SLP.

**Probe error type** — modeled on `node_ws_handler.rs` and the `TaskError` pattern from `agent-proto`:
```rust
pub enum ProbeError {
    Timeout,
    TcpConnect(io::Error),
    Io(io::Error),
    Handshake,
    Protocol,
    Json(serde_json::Error),
}
```

**Module registration** — add `pub mod connectivity_service;` to `api/src/application/services/mod.rs` and `api/src/presentation/services/mod.rs` (or whichever directory it's placed in — RESEARCH names `api/src/presentation/services/connectivity_service.rs`, but no such dir exists today; `api/src/application/services/` is the existing home for `monitoring_service.rs`). Prefer `api/src/application/services/connectivity_service.rs` for consistency with `monitoring_service.rs`.

---

### `api/src/presentation/routes/api_routes.rs` (mount new routes)

**Analog:** `api/src/presentation/routes/api_routes.rs:34-37` (per-server nested routes pattern) and `:32-33` (the `ServerHandlers::router(state.clone())` nest).

**Existing pattern** (`api_routes.rs:33-37`):
```rust
.nest("/api/v1/servers", ServerHandlers::router(state.clone()))
.route("/api/v1/servers/:server_id/backup-config", get(...).put(...))
.route("/api/v1/servers/:server_id/tasks", get(list_tasks).post(create_task))
.route("/api/v1/servers/:server_id/tasks/:task_id", patch(update_task).delete(delete_task))
.route("/api/v1/servers/:server_id/tasks/:task_id/run", post(run_task))
```

**Add**:
```rust
.nest("/api/v1/servers", ServerHandlers::router(state.clone())
    .merge(crate::presentation::handlers::connectivity_handlers::router(state.clone())))
```
Or follow the inline `.route(...)` precedent. The `connectivity_handlers::router` already nests its own `:server_id` prefix, so merging is cleaner.

---

### `app/src/components/ConnectivityBadge.jsx` (component, display)

**Analog:** `app/src/components/StatusBadge.jsx` (entire file, 38 lines) — exact pattern. Same color+dot layout, same `bg-... text-...` Tailwind tokens, same `inline-flex items-center gap-2 px-3 py-1 rounded-full` base.

**Style template** (`StatusBadge.jsx:1-37`):
```jsx
export default function StatusBadge({ status, autoWake = false }) {
    const isRunning = status === 'running';
    // ...
    let bgColor = 'bg-[rgba(255,255,255,0.05)] text-[var(--color-text-muted)]';
    let dotColor = 'bg-[var(--color-text-muted)]';
    let label = 'Stopped';

    if (isRunning) { bgColor = 'bg-[rgba(16,185,129,0.15)] text-[var(--color-cosmic-green)]'; /* ... */ label = 'Running'; }
    // ...

    return (
        <span className={`inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-semibold transition-colors ${bgColor}`}>
            <span className={`w-2 h-2 rounded-full ${dotColor}`} />
            {label}
        </span>
    );
}
```
Apply the same 3-state mapping per UI-SPEC §1: `Reachable` (green, no shadow animation), `Unreachable` (red, clickable), `Unknown` (grey, plain).

---

### `app/src/components/ConnectivitySection.jsx` (component, data-display)

**Analog:** `app/src/components/ServerBackups.jsx` (closest per-server sub-section) — same `useState` + `useEffect` + `fetchApi` shape, same `card` Tailwind layout used by `ServerDetailsPage.jsx`. Look up `ServerBackups.jsx` and the `<ResourceGraph>` precedent in `ServerDetailsPage.jsx` for the card layout.

Skeleton shape:
```jsx
import { useEffect, useState } from 'react'
import { fetchApi } from '../api/client'
import ConnectivityBadge from './ConnectivityBadge'
import ConnectivityAuditLog from './ConnectivityAuditLog'

export default function ConnectivitySection({ serverId, onProbe }) {
    const [status, setStatus] = useState(null)        // { status, mode, last_probe_at, details: {...} }
    const [audit, setAudit] = useState([])
    const [cooldown, setCooldown] = useState(0)
    // ... fetch /servers/:id/connectivity + /connectivity/audit on mount and on probe-complete
    return (
        <div className="card bg-base-100 shadow-xl">
            <div className="card-body">
                <h2 className="card-title">Connectivity</h2>
                <ConnectivityBadge status={status?.status} />
                <Diagnostics details={status?.details} />
                <ConnectivityAuditLog entries={audit.slice(0, 50)} />
                <button disabled={cooldown > 0} onClick={onProbe}>Reachable</button>
            </div>
        </div>
    )
}
```

---

### `app/src/hooks/useConnectivity.js` (hook, data-fetching)

**Analog:** `app/src/hooks/useAlerts.js` (entire file, 48 lines) — exact pattern. Same `useState` + `useEffect` + `useCallback` + `fetchApi` shape, same `refetch()` returned to consumers.

**Shape** (mirroring `useAlerts.js:4-19`):
```js
import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

export function useConnectivity(serverId) {
    const [status, setStatus] = useState(null);
    const [audit, setAudit] = useState([]);
    const [loading, setLoading] = useState(true);

    const refetch = useCallback(async () => {
        try {
            setLoading(true);
            const [s, a] = await Promise.all([
                fetchApi(`/servers/${serverId}/connectivity`),
                fetchApi(`/servers/${serverId}/connectivity/audit?limit=50`),
            ]);
            setStatus(s);
            setAudit(a);
        } catch { /* silent */ } finally { setLoading(false); }
    }, [serverId]);

    useEffect(() => { refetch(); }, [refetch]);

    const triggerProbe = useCallback(async () => {
        return fetchApi(`/servers/${serverId}/connectivity/probe`, { method: 'POST' });
    }, [serverId]);

    return { status, audit, loading, refetch, triggerProbe };
}
```

---

### `app/src/lib/api.js` (extend api client)

**Analog:** `app/src/lib/api.js:100-115` (`serversApi` object) — exact extension pattern. Add three new methods to the `serversApi` object.

**Extension pattern** (`api.js:100-115`):
```js
export const serversApi = {
    list: () => api.get('/servers'),
    get: (id) => api.get(`/servers/${id}`),
    // ...
    getServerProperties: (id) => api.get(`/servers/${id}/properties`),
    updateServerProperties: (id, properties) => api.patch(`/servers/${id}/properties`, properties),
};
```
Add:
```js
getConnectivity: (id) => api.get(`/servers/${id}/connectivity`),
triggerProbe: (id) => api.post(`/servers/${id}/connectivity/probe`),
getConnectivityAudit: (id, params) => api.get(`/servers/${id}/connectivity/audit`, { params }),
```

---

### `app/src/pages/servers/ServerManagerPage.jsx` (modify)

**Analog:** `app/src/pages/servers/ServerManagerPage.jsx:25-43` (existing `getStatusColor` function). The badge slot can be inserted into the existing server-card layout (lines 80-130) right next to the status dot. Use the `ConnectivityBadge` component rather than a color-function (cleaner separation).

**Existing card layout snippet** (look at lines 80-130 of the file):
```jsx
<div className="server-card ...">
    <div className="server-status">...</div>
    <h3>{server.name}</h3>
    <p>{server.game} · v{server.version}</p>
</div>
```
Add the `ConnectivityBadge` line after the status dot, gated on the server's `connectivity_status` field being present (graceful fallback to no badge for legacy rows).

---

### `app/src/pages/servers/ServerDetailsPage.jsx` (modify)

**Analog:** `app/src/pages/servers/ServerDetailsPage.jsx:33-39` (existing `tabs` array) and `:100+` (tab content switch). Add the `ConnectivitySection` as a new opt-in section below the existing tab content (per UI-SPEC: "new section is opt-in" — D-15). Don't try to add it as a tab; UI-SPEC says it's a card in the overview area.

```jsx
import ConnectivitySection from '../../components/ConnectivitySection'
// ...
{server && <ConnectivitySection serverId={server.id} />}
```

---

## Shared Patterns

### WebSocket NodeMessage extension
**Source:** `api/src/presentation/ws/node_protocol.rs:5-135` (the `NodeMessage` enum shape with `#[serde(tag = "type")]` and `#[serde(rename = "...")]` on each variant)
**Apply to:** `node_protocol.rs` (extend with new variants) and `node_ws_handler.rs` (add new dispatch arms)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeMessage {
    // ... existing variants ...
    #[serde(rename = "crash_report")]
    CrashReport { server_id: Uuid, exit_code: i32, log_excerpt: String, timestamp: String },
}
```
Always add new variants as enum cases (no separate struct types) so `serde_json::from_str::<NodeMessage>` keeps working unchanged.

---

### Cross-tier audit log persistence (D-17)
**Source:** `api/src/application/services/monitoring_service.rs:505-527` (the `store_crash_log` method that builds an entity and calls `repo.insert`) AND `api/src/infrastructure/repositories/crash_log_repository.rs:16-35` (the `insert` method with `sqlx::query`).
**Apply to:** `api/src/presentation/services/connectivity_service.rs` (after every auto-fix attempt, store a `ConnectivityAuditLog` row).
```rust
async fn store_audit(&self, log: &ConnectivityAuditLog) -> Result<()> {
    self.connectivity_audit_log_repository.insert(log).await
}
```

---

### Per-tenant ownership check (V4 Access Control)
**Source:** `api/src/presentation/handlers/cron_task_handlers.rs:30-37` (the `server.user_id != auth_user.tenant_id → access denied` check at the start of every handler)
**Apply to:** `api/src/presentation/handlers/connectivity_handlers.rs` (every endpoint)
```rust
let server = state.server_repository.find_by_id(&server_id).await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Server not found".to_string())?;
if server.user_id != auth_user.tenant_id { return Err("Access denied".to_string()); }
```

---

### Background-task periodic-loop pattern
**Source:** `api/src/application/services/monitoring_service.rs:74-91` AND `src/handlers/dns_watch.rs:43-64` (the `tokio::spawn` + `tokio::time::interval` + `running` flag pattern)
**Apply to:** `src/main.rs` (the agent's `ConnectivityMonitor`) and `api/src/application/services/connectivity_service.rs` (the backend's probe scheduler)

The monitor must be `Arc<Self>`-cloned and stored so the shutdown handler in `main.rs:286-293` can call `.stop().await` on it (mirror the dns_watcher pattern line-for-line).

---

### Send WS message to a specific node
**Source:** `api/src/presentation/handlers/node_ws_handler.rs:269-271` (calls `manager.send_to_node(&node_id_val, &dns_msg).await`)
**Apply to:** `api/src/presentation/services/connectivity_service.rs` (when dispatching `ConnectivityFixRequest` from the auto-fix pipeline)
```rust
manager.send_to_node(&node_id, &NodeMessage::ConnectivityFixRequest {
    request_id: Uuid::new_v4(),
    server_id,
    action: "firewall.open_port".into(),
    params: json!({ "port": 25565, "proto": "tcp", "server_id": server_id }),
}).await?;
```

---

### "Don't Hand-Roll" (RESEARCH §Don't Hand-Roll summary, applied to Phase 67)

| Reuse From | Don't Write Yourself |
|------------|----------------------|
| `src/handlers/dns_watch.rs:132` `detect_public_ip()` | a second public-IP fetcher |
| `bollard::Docker::inspect_container()` (used at `runtime.rs:186`) | a custom Docker SDK call for `port_bound` |
| `upnp-rs 0.2` crate (new dep, CONTEXT D-07) | raw SSDP/UPnP packets |
| `api/src/presentation/ws/node_protocol.rs:7` `NodeMessage` enum | a separate WS message envelope |
| `api/migrations/20260324000007_create_audit_logs_table.sql` + the new `connectivity_audit_log` migration | a custom file-based audit logger |
| `api/src/domain/billing/webhooks.rs` (per server `discord_webhook_url`) | a custom SMTP/HTTP path for connectivity alerts |
| `tokio::time::interval` (mirrors `dns_watch.rs:51`) | a custom timer wheel for periodic checks |
| `which::which(...)` (precedent: `agent-runtime/src/detector.rs:41, 72`) | a custom `PATH`-scanning tool detector |

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns + Code Examples instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/handlers/connectivity/upnp.rs` | service-executor | IGD control | No UPnP code exists yet; first use of `upnp-rs` crate |
| `app/src/components/ConnectivityAuditLog.jsx` | component | list-display | Audit log UI is unique to Phase 67 |
| `api/src/presentation/services/connectivity_service.rs` | service | probe + classify | First outbound probe origin on the backend; uses Tokio TCP/UDP directly to reach the user's public IP. Closest in spirit is `monitoring_service.rs` but that runs an internal `executor.check_status` (Docker SDK), not an outbound network probe |

For the missing patterns:
- **UPnP** — see RESEARCH §"Code Examples → Pattern 3" (lines 421-462) for the full `add_port_mapping` skeleton using `upnp_rs::discovery::discover`.
- **Connectivity audit log UI** — see UI-SPEC.md §2 ("Server Details — Connectivity Section" lines 53-57) for the visual structure (timestamp + event_type + status, 50 visible rows).
- **TCP + Minecraft Java SLP probe** — see RESEARCH §"Code Examples → Minecraft Java Edition Server List Ping" (lines 545-602) for the wire-format implementation. Bedrock RakNet ping is at lines 605-680.

---

## Metadata

**Analog search scope:** `src/handlers/`, `src/audit.rs`, `src/main.rs`, `Cargo.toml`, `api/migrations/`, `api/src/domain/{entities,repositories}/`, `api/src/infrastructure/repositories/`, `api/src/presentation/{handlers,ws,routes}/`, `api/src/application/services/`, `api/src/bootstrap/container.rs`, `agent/agent-core/crates/agent-runtime/src/detector.rs`, `app/src/{components,hooks,pages,lib}/`
**Files scanned:** ~40
**Pattern extraction date:** 2026-06-07
**Confidence:** HIGH — 17/19 files have an exact or role-match analog; the 2 no-analog files have well-documented RESEARCH.md code examples to follow.
