# Phase 56: Auto Online & Sleep Recovery - Pattern Map

**Mapped:** 2026-05-30
**Files analyzed:** 11 (6 modified backend, 3 modified frontend, 1 new service, 1 new migration)
**Analogs found:** 11 / 11

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `api/src/domain/entities/server.rs` | model | CRUD | self (existing auto_restart pattern) | exact |
| `api/src/application/dto/server_dtos.rs` | dto | CRUD | self (existing auto_restart field pattern) | exact |
| `api/src/application/use_cases/create_server_use_case.rs` | use_case | CRUD | self (existing auto_restart wiring) | exact |
| `api/src/application/use_cases/update_server_use_case.rs` | use_case | CRUD | self (existing auto_restart wiring) | exact |
| `api/src/application/services/monitoring_service.rs` | service | request-response | self (existing crash-detection block) | exact |
| `api/src/application/services/sleep_service.rs` | service | request-response | `api/src/application/services/node_health_service.rs` | role-match |
| `api/src/presentation/handlers/server_handlers.rs` | controller | request-response | self (stop_server handler, start_server handler) | exact |
| `api/migrations/20260530000001_add_auto_wake.sql` | migration | — | `api/migrations/20260307000001_add_enhanced_server_features.sql` | exact |
| `app/src/pages/ServerDetails.jsx` | component | request-response | self (settings tab Discord webhook section) | exact |
| `app/src/hooks/useServers.js` | hook | request-response | self (startServer/stopServer functions) | exact |
| `app/src/components/StatusBadge.jsx` | component | request-response | self (existing status renderings) | exact |

## Pattern Assignments

### `api/src/domain/entities/server.rs` (model, CRUD)

**Analog:** self — existing `auto_restart` and `auto_pause` field pattern

**Imports pattern** (lines 1-3):
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
```

**Existing field pattern to follow** (lines 22, 34-35):
```rust
pub auto_pause: bool,       // line 22 — boolean feature flag template
pub auto_restart: bool,     // line 34 — boolean server feature flag
pub restart_count: i32,     // line 35 — numeric counter
```

**Add alongside existing fields (after line 42, before "Git remote configuration"):**
```rust
// Sleep/Wake & Auto-Restart Backoff (Phase 56)
pub auto_wake: bool,
pub sleep_timeout_minutes: i32,
pub last_player_activity: Option<DateTime<Utc>>,
pub max_restart_attempts: i32,
pub restart_cooldown_seconds: i32,
```

**Existing helper method pattern** (lines 58-65):
```rust
impl Server {
    pub fn is_running(&self) -> bool {
        matches!(self.status.as_str(), "running" | "container_running")
    }

    pub fn is_stopped(&self) -> bool {
        !self.is_running()
    }
}
```

---

### `api/src/application/dto/server_dtos.rs` (dto, CRUD)

**Analog:** self — existing `auto_restart` and `auto_pause` field pattern

**Existing field pattern for `CreateServerRequest`** (lines 23, 27):
```rust
pub auto_pause: Option<bool>,       // line 23
pub auto_restart: Option<bool>,     // line 27
```

**Add to `CreateServerRequest`** (after line 28):
```rust
pub auto_wake: Option<bool>,
pub sleep_timeout_minutes: Option<i32>,
pub max_restart_attempts: Option<i32>,
pub restart_cooldown_seconds: Option<i32>,
```

**Existing field pattern for `UpdateServerRequest`** (lines 64, 74):
```rust
pub auto_pause: Option<bool>,       // line 64
pub auto_restart: Option<bool>,     // line 74
```

**Add to `UpdateServerRequest`** (after line 75):
```rust
pub auto_wake: Option<bool>,
pub sleep_timeout_minutes: Option<i32>,
pub max_restart_attempts: Option<i32>,
pub restart_cooldown_seconds: Option<i32>,
```

**Existing Default impl pattern** (lines 91-129): Add same `None` fields in the same position.

**Add to `ServerResponse`** (after line 153):
```rust
pub auto_wake: bool,
pub sleep_timeout_minutes: i32,
pub max_restart_attempts: i32,
pub restart_cooldown_seconds: i32,
```

---

### `api/src/application/use_cases/create_server_use_case.rs` (use_case, CRUD)

**Analog:** self — existing `auto_restart` wiring pattern

**Existing wiring pattern** (line 57-58):
```rust
auto_restart: req.auto_restart.unwrap_or(false),
restart_count: 0,
```

**Add after line 58 (alongside auto_restart block):**
```rust
auto_wake: req.auto_wake.unwrap_or(false),
sleep_timeout_minutes: req.sleep_timeout_minutes.unwrap_or(30),
max_restart_attempts: req.max_restart_attempts.unwrap_or(5),
restart_cooldown_seconds: req.restart_cooldown_seconds.unwrap_or(300),
last_player_activity: None,
```

---

### `api/src/application/use_cases/update_server_use_case.rs` (use_case, CRUD)

**Analog:** self — existing `auto_pause` wiring pattern

**Existing wiring pattern for boolean fields** (lines 72-74):
```rust
if let Some(auto_pause) = req.auto_pause {
    server.auto_pause = auto_pause;
}
```

**Add after line 74 (after auto_pause block):**
```rust
if let Some(auto_wake) = req.auto_wake {
    server.auto_wake = auto_wake;
}
if let Some(sleep_timeout_minutes) = req.sleep_timeout_minutes {
    server.sleep_timeout_minutes = sleep_timeout_minutes;
}
if let Some(max_restart_attempts) = req.max_restart_attempts {
    server.max_restart_attempts = max_restart_attempts;
}
if let Some(restart_cooldown_seconds) = req.restart_cooldown_seconds {
    server.restart_cooldown_seconds = restart_cooldown_seconds;
}
```

---

### `api/src/application/services/monitoring_service.rs` (service, request-response)

**Analog:** self — existing crash-detection block (lines 137-189) and metrics collection block (lines 207-234)

**Existing loop structure** (lines 78-242): `check_all_servers()` iterates servers, skips offline-node servers (92-100) and non-running servers (102-105), then checks status (116-205), collects metrics (207-234).

**Crash detection + auto-restart block** (lines 137-189) — the pattern for sleep detection:

```rust
// Check for crash detection and auto-restart
if server.status == "running" && status == "stopped" {
    match self.repository.find_by_id(&server.id).await {
        Ok(Some(full_server)) => {
            if full_server.auto_restart {
                tracing::warn!("[MONITOR] Server {} detected as crashed, auto-restarting...", full_server.name);
                let executor = self.executor_factory.get_executor(&full_server);
                match executor.start_server(&full_server).await {
                    Ok(_) => {
                        // Update restart count
                        let mut updated = full_server.clone();
                        updated.restart_count += 1;
                        if let Err(e) = self.repository.update(&updated).await { ... }
                    }
                    Err(e) => { ... }
                }
                continue;
            }
        }
        ...
    }
}
```

**Sleep detection injection point** — AFTER the crash-detection block (after `continue` on line 175, or after the `} else {` on line 190), BEFORE metrics collection (line 207):

```rust
// === SLEEP DETECTION (Phase 56) ===
// Check for servers that should sleep due to player inactivity
if status == "running" {
    // Collect metrics to get player count
    match executor.collect_metrics(&server).await {
        Ok(metrics) => {
            if metrics.players > 0 {
                // Players online — update last_player_activity
                let mut updated = server.clone();
                updated.last_player_activity = Some(Utc::now());
                let _ = self.repository.update(&updated).await;
            } else if server.last_player_activity.is_some() {
                // No players — check inactivity timeout
                let elapsed = Utc::now() - server.last_player_activity.unwrap();
                let timeout = chrono::Duration::minutes(server.sleep_timeout_minutes as i64);
                if elapsed >= timeout {
                    // Trigger sleep: stop server + set auto_wake
                    executor.stop_server(&server).await?;
                    let mut updated = server.clone();
                    updated.status = "stopped".to_string();
                    updated.auto_wake = true;
                    self.repository.update(&updated).await?;
                    self.event_bus.publish(ServerEvent::StatusChanged {
                        server_id: server.id,
                        status: "stopped".to_string(),
                    });
                    continue;  // Skip metrics collection for sleeping servers
                }
            }
        }
        Err(e) => { tracing::warn!("Failed to collect metrics for sleep detection: {}", e); }
    }
}
```

**Auto-restart backoff pattern** — modify existing crash-restart block (lines 143-173) to add backoff:

```rust
// Instead of immediate start, check backoff:
if full_server.auto_restart && full_server.restart_count < full_server.max_restart_attempts {
    let backoff = std::cmp::min(
        30 * 2_u32.pow(full_server.restart_count as u32),  // exponential: 30, 60, 120, 240...
        full_server.restart_cooldown_seconds as u32,        // cap at max
    );
    tracing::warn!("[MONITOR] Server {} crashed, restarting in {}s (attempt {}/{})",
        full_server.name, backoff, full_server.restart_count + 1, full_server.max_restart_attempts);

    // Spawn delayed restart to avoid blocking the loop (Pitfall 4)
    let repo_clone = self.repository.clone();
    let factory_clone = self.executor_factory.clone();
    let server_clone = full_server.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(backoff as u64)).await;
        let executor = factory_clone.get_executor(&server_clone);
        if let Err(e) = executor.start_server(&server_clone).await {
            tracing::error!("[MONITOR] Backed-off restart failed: {}", e);
        } else {
            let mut updated = server_clone;
            updated.restart_count += 1;
            let _ = repo_clone.update(&updated).await;
        }
    });
}

// Reset restart_count after stable running time (add in metrics collection section):
if status == "running" && server.restart_count > 0 {
    // Check if server has been running for >5 minutes without crash
    // Reset counter
    let mut updated = server.clone();
    updated.restart_count = 0;
    let _ = self.repository.update(&updated).await;
}
```

---

### `api/src/application/services/sleep_service.rs` (NEW — service, request-response)

**Analog:** `api/src/application/services/node_health_service.rs` — configurable-interval evaluation service pattern

**Full analog structure** (lines 1-115 of `node_health_service.rs`):
```rust
use std::sync::Arc;
use chrono::Utc;
// ... domain entity and repository imports ...

// Configurable constants
const DEFAULT_SLEEP_TIMEOUT_MINUTES: i32 = 30;

pub struct SleepService {
    server_repository: Arc<dyn ServerRepository>,
    // could hold executor_factory or event_bus if needed
}

impl SleepService {
    pub fn new(
        server_repository: Arc<dyn ServerRepository>,
    ) -> Self {
        Self { server_repository }
    }

    /// Check if a server has been idle long enough to sleep
    pub fn should_sleep(&self, server: &Server) -> bool {
        if !server.auto_wake {
            return false; // Sleep disabled for this server
        }
        if let Some(last_activity) = server.last_player_activity {
            let elapsed = Utc::now() - last_activity;
            let timeout = chrono::Duration::minutes(server.sleep_timeout_minutes as i64);
            elapsed >= timeout
        } else {
            false // No activity recorded yet
        }
    }

    /// Get servers eligible for sleep (running, auto_wake=true, idle)
    pub async fn get_sleep_eligible_servers(&self) -> Vec<Server> {
        // Query servers where status=running AND auto_wake=true
        // Evaluate inactivity timeout
    }
}
```

---

### `api/src/presentation/handlers/server_handlers.rs` (controller, request-response)

**Analog:** self — `stop_server` handler (lines 824-937) for sleep endpoint, `start_server` handler (lines 670-822) for wake endpoint

**Route registration pattern** (lines 305-378 in `ServerHandlers::router`):
```rust
.route("/:id/start", post(start_server))       // line 309
.route("/:id/stop", post(stop_server))           // line 310
.route("/:id/restart", post(restart_server))     // line 311
```

**Add to router (after line 311):**
```rust
.route("/:id/sleep", post(sleep_server))
.route("/:id/wake", post(wake_server))
```

**Existing handler auth pattern** (e.g., stop_server lines 824-837):
```rust
async fn stop_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }
    // ... action logic ...
}
```

**Sleep handler** (following stop_server pattern):
```rust
async fn sleep_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let mut server = repo.find_by_id(id)
        .await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    // Same stop logic as stop_server (agent or solys path)...
    // But set auto_wake=true after stopping
    // ... executor stop ...
    server.status = "stopped".to_string();
    server.auto_wake = true;
    repo.update(&server).await.map_err(|e| e.to_string())?;

    emit_server_event(&state.pool, "server.sleep", user_id, id, &server_name).await;

    Ok(Json(ApiResponse::success(serde_json::json!({ "status": "stopped", "auto_wake": true }))))
}
```

**Wake handler** (following start_server pattern):
```rust
async fn wake_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let mut server = repo.find_by_id(id)
        .await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    // Only wake sleeping servers
    if !server.auto_wake {
        return Err("Server is not in sleep mode".to_string());
    }

    // Same start logic as start_server handler (agent path lines 691-801)...
    // Reset auto_wake after waking
    server.auto_wake = false;
    repo.update(&server).await.map_err(|e| e.to_string())?;

    emit_server_event(&state.pool, "server.wake", user_id, id, &server_name).await;

    Ok(Json(ApiResponse::success(serde_json::json!({ "status": "starting" }))))
}
```

**Existing error response pattern** (lines 792-794, 913-914):
```rust
return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "started" }))));
return Err("Agent failed to stop server: ...".to_string());
```

---

### `api/migrations/20260530000001_add_auto_wake.sql` (migration)

**Analog:** `api/migrations/20260307000001_add_enhanced_server_features.sql`

**Full pattern** (16 lines):
```sql
-- Add new fields for sleep/wake and auto-restart backoff
-- Auto-wake (sleep mode)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS auto_wake BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS sleep_timeout_minutes INTEGER NOT NULL DEFAULT 30;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_player_activity TIMESTAMPTZ;
-- Auto-restart backoff
ALTER TABLE servers ADD COLUMN IF NOT EXISTS max_restart_attempts INTEGER NOT NULL DEFAULT 5;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS restart_cooldown_seconds INTEGER NOT NULL DEFAULT 300;
```

---

### `app/src/pages/ServerDetails.jsx` (component, request-response)

**Analog:** self — Settings tab Discord webhook section (lines 328-382)

**Settings tab rendering pattern** (lines 327-385):
```jsx
{activeTab === 'settings' ? (
    <div className="max-w-2xl">
        <section className="glass-panel p-6">
            <h3 className="text-lg font-bold mb-1">Discord Webhook</h3>
            <p className="text-xs text-[var(--color-text-muted)] mb-5">...</p>
            {/* ... input and save button ... */}
        </section>
    </div>
) : /* ... */}
```

**Add sleep/wake section BEFORE the closing `</div>` of settings tab** (after line 382, before `)` on line 385):

```jsx
<section className="glass-panel p-6 mt-6">
    <h3 className="text-lg font-bold mb-1">Sleep & Wake</h3>
    <p className="text-xs text-[var(--color-text-muted)] mb-5">
        Automatically stop server when inactive and wake on player connection.
    </p>

    <div className="flex items-center gap-3 p-4 rounded-xl border cursor-pointer hover:border-[var(--color-cosmic-cyan)]/50"
         onClick={() => setForm(f => ({ ...f, auto_wake: !f.auto_wake }))}>
        <div className={`w-12 h-6 rounded-full transition-colors ${form.auto_wake ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-gray-600'}`}>
            <div className={`w-5 h-5 rounded-full bg-white transition-transform ${form.auto_wake ? 'translate-x-6' : 'translate-x-0.5'}`} />
        </div>
        <div className="flex-1">
            <p className="text-sm font-bold">Auto Sleep</p>
            <p className="text-xs text-[var(--color-text-muted)]">Stop server after inactivity</p>
        </div>
    </div>

    {form.auto_wake && (
        <div className="mt-3">
            <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">
                Sleep after (minutes of 0 players)
            </label>
            <input type="number" value={form.sleep_timeout_minutes} min={5} max={240}
                   onChange={e => setForm(f => ({ ...f, sleep_timeout_minutes: parseInt(e.target.value) }))}
                   className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border ..." />
        </div>
    )}
</section>
```

**Existing fetch/state pattern for server data** (lines 71-83):
```jsx
useEffect(() => {
    fetchApi(`/servers/${id}`).then(data => {
        setServer(data);
        setWebhookUrl(data.discord_webhook_url || '');
        // ... add: set form sleep fields from data
        setLoading(false);
    }).catch(() => setLoading(false));
}, [id]);
```

---

### `app/src/hooks/useServers.js` (hook, request-response)

**Analog:** self — `startServer`/`stopServer` functions (lines 100-106)

**Existing API call pattern** (lines 100-106):
```js
export async function startServer(id) {
    return fetchApi(`/servers/${id}/start`, { method: 'POST' });
}

export async function stopServer(id) {
    return fetchApi(`/servers/${id}/stop`, { method: 'POST' });
}
```

**Add after line 106:**
```js
export async function sleepServer(id) {
    return fetchApi(`/servers/${id}/sleep`, { method: 'POST' });
}

export async function wakeServer(id) {
    return fetchApi(`/servers/${id}/wake`, { method: 'POST' });
}
```

---

### `app/src/components/StatusBadge.jsx` (component, request-response)

**Analog:** self — all existing status renderings (lines 1-34)

**Existing status rendering pattern** (lines 1-34):
```jsx
export default function StatusBadge({ status }) {
    const isRunning = status === 'running';
    // ...
    let bgColor = 'bg-[rgba(255,255,255,0.05)] text-[var(--color-text-muted)]';
    let dotColor = 'bg-[var(--color-text-muted)]';
    let label = 'Stopped';

    if (isRunning) {
        bgColor = 'bg-[rgba(16,185,129,0.15)] text-[var(--color-cosmic-green)]';
        dotColor = 'bg-[var(--color-cosmic-green)] shadow-[0_0_8px_var(--color-cosmic-green)]';
        label = 'Running';
    } else if (status === 'crashed') { ... }
    // ...
}
```

**Add sleeping detection** — component needs to accept `autoWake` prop:
```jsx
export default function StatusBadge({ status, autoWake }) {
    // ... existing checks ...

    // Add after stopped/default check:
    if (status === 'stopped' && autoWake) {
        bgColor = 'bg-[rgba(13,223,242,0.1)] text-[var(--color-cosmic-cyan)]';
        dotColor = 'bg-[var(--color-cosmic-cyan)] animate-pulse';
        label = 'Sleeping';
    }
}
```

**Usage in parent** — in `ServerDetails.jsx` line 476:
```jsx
<StatusBadge status={server.status} autoWake={server.auto_wake} />
```

---

## Shared Patterns

### Authentication & Tenant Access
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 383-384, 541-543, 681-683)
**Apply to:** All handler-level operations (sleep/wake endpoints)
```rust
// Handler signature pattern — use VerifiedUser for auth
async fn handler_name(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> { ... }

// Tenant isolation check
if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
    return Err("Access denied".to_string());
}
```

### Error Handling — Handlers
**Source:** `api/src/presentation/responses/api_response.rs` (lines 196-233)
**Apply to:** All handler responses
```rust
// Success response
Ok(Json(ApiResponse::success(serde_json::json!({ "status": "started" }))))

// Error response
Err("Server not found".to_string())
```

### Background Service Startup
**Source:** `api/src/bootstrap/mod.rs` (lines 50-53)
**Apply to:** `sleep_service.rs` if it needs its own background loop (or use `monitoring_service.rs`)
```rust
let monitoring = container.monitoring_service.clone();
tokio::spawn(async move {
    monitoring.start().await;
});
```

### EventBus Status Change Publishing
**Source:** `api/src/infrastructure/events/event_bus.rs` (lines 16-19), used in `monitoring_service.rs` (lines 130-133, 199-203, 215-222)
**Apply to:** Sleep/wake status changes
```rust
let _ = self.event_bus.publish(ServerEvent::StatusChanged {
    server_id: server.id,
    status: "stopped".to_string(),
});
```

### Webhook Event Emission (for audit trail)
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 524, 790, 818)
**Apply to:** Sleep/wake handlers
```rust
emit_server_event(&state.pool, "server.started", user_id, id, &server_name).await;
```

---

## No Analog Found

All files have a direct match in the existing codebase — no file requires RESEARCH.md-only patterns.

| File | Reason for Full Match |
|------|----------------------|
| All 11 files | Each new/modified file follows the exact pattern of an existing file in the same role |

## Key Warnings from Code Analysis

### Pitfall A: Two Server Models Exist
There are **two** Server structs in the codebase:
- `api/src/domain/entities/server.rs` (old — used by monitoring_service, use_cases via `ServerRepository`)
- `api/src/domain/server/model.rs` (new — used by server_handlers.rs via `SqlxServerRepository`)

**`monitoring_service.rs` uses the OLD model** (`crate::domain::entities::server::Server` at line 73). The `auto_wake` field must be added to THIS model for monitoring to detect it.

### Pitfall B: Monitoring Loop Is Sequential
The `check_all_servers()` loop (lines 78-242) processes servers one by one. Any `sleep()` inside this loop will **block ALL server monitoring** (Pitfall 4 from RESEARCH). Use `tokio::spawn` for delayed restart tasks.

### Pitfall C: EventBus vs Direct emit_server_event
The codebase uses two event mechanisms:
1. **EventBus** (`event_bus.publish`) — internal in-process, used by monitoring_service
2. **emit_server_event** — writes to DB for WebSocket broadcast, used by server_handlers

Sleep detection in `monitoring_service` should use EventBus. Manual sleep/wake in handlers should use both.

## Metadata

**Analog search scope:** `api/src/domain/`, `api/src/application/`, `api/src/presentation/`, `api/src/infrastructure/`, `api/migrations/`, `app/src/`
**Files scanned:** 40+ Rust files, 30+ JSX/JS files
**Pattern extraction date:** 2026-05-30
