# Phase 57: Auto Restart Policies - Pattern Map

**Mapped:** 2026-05-30
**Files analyzed:** 12 (1 migration, 6 backend modified, 1 backend enhanced, 2 frontend modified, 1 frontend hook, 1 new backend enhancement)
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `api/migrations/20260530000002_add_restart_policy.sql` | migration | — | `api/migrations/20260530000001_add_auto_wake.sql` | exact |
| `api/src/domain/entities/server.rs` | model | CRUD | self (existing `auto_wake`, `max_restart_attempts`, `restart_cooldown_seconds` fields) | exact |
| `api/src/domain/server/model.rs` | model | CRUD | self (existing `auto_wake`, `sleep_timeout_minutes` fields) | exact |
| `api/src/domain/server/sqlx_repository.rs` | repository | CRUD | self (existing `auto_wake`/`sleep_timeout_minutes` column pattern) | exact |
| `api/src/infrastructure/repositories/postgres_server_repository.rs` | repository | CRUD | self (existing `auto_wake`/`sleep_timeout_minutes` read/write pattern) | exact |
| `api/src/application/dto/server_dtos.rs` | dto | CRUD | self (existing `max_restart_attempts`/`restart_cooldown_seconds` field pattern) | exact |
| `api/src/application/use_cases/create_server_use_case.rs` | use_case | CRUD | self (existing `last_player_activity`/`restart_cooldown_seconds` wiring) | exact |
| `api/src/application/use_cases/update_server_use_case.rs` | use_case | CRUD | self (existing `max_restart_attempts` conditional block) | exact |
| `api/src/presentation/handlers/server_handlers.rs` | controller | request-response | self (existing `auto_wake`/`sleep_timeout_minutes` update handler wiring) | exact |
| `api/src/application/services/monitoring_service.rs` | service | request-response | self (existing `collect_metrics` block + sleep detection block) | exact |
| `app/src/pages/ServerDetails.jsx` | component | request-response | self (existing Sleep & Wake config section) | exact |
| `app/src/hooks/useServers.js` | hook | request-response | self (existing `updateServer` function) | exact |

## Pattern Assignments

### `api/migrations/20260530000002_add_restart_policy.sql` (migration)

**Analog:** `api/migrations/20260530000001_add_auto_wake.sql`

**Full pattern** (Phase 56 migration, 11 lines):
```sql
-- Add sleep/wake and auto-restart backoff fields
-- Phase 56: Auto Online & Sleep Recovery

-- Auto-wake (sleep mode)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS auto_wake BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS sleep_timeout_minutes INTEGER NOT NULL DEFAULT 30;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_player_activity TIMESTAMPTZ;

-- Auto-restart backoff
ALTER TABLE servers ADD COLUMN IF NOT EXISTS max_restart_attempts INTEGER NOT NULL DEFAULT 5;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS restart_cooldown_seconds INTEGER NOT NULL DEFAULT 300;
```

**Follow pattern for Phase 57 migration** (add after auto-restart backoff section):
```sql
-- Add restart policy and health check fields
-- Phase 57: Auto Restart Policies

-- Restart history tracking
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_restart_at TIMESTAMPTZ;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_restart_reason TEXT;

-- Health check configuration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS health_check_timeout_seconds INTEGER NOT NULL DEFAULT 5;
```

---

### `api/src/domain/entities/server.rs` (model, CRUD)

**Analog:** self — existing Phase 56 fields (lines 43-48)

**Imports pattern** (lines 1-3):
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
```

**Existing Phase 56 field block to follow** (lines 43-48):
```rust
    // Sleep/Wake & Auto-Restart Backoff (Phase 56)
    pub auto_wake: bool,
    pub sleep_timeout_minutes: i32,
    pub last_player_activity: Option<DateTime<Utc>>,
    pub max_restart_attempts: i32,
    pub restart_cooldown_seconds: i32,
```

**Add after line 48** (after `restart_cooldown_seconds`):
```rust
    // Restart Policy & Health Check (Phase 57)
    pub last_restart_at: Option<DateTime<Utc>>,
    pub last_restart_reason: Option<String>,
    pub health_check_timeout_seconds: i32,
```

---

### `api/src/domain/server/model.rs` (model, CRUD)

**Analog:** self — existing `auto_wake`, `sleep_timeout_minutes` fields (lines 19-20)

**Imports pattern** (lines 1-3):
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
```

**Existing Phase 56 fields** (lines 19-20):
```rust
    pub auto_wake: Option<bool>,
    pub sleep_timeout_minutes: Option<i32>,
```

**Add after line 20** (after `sleep_timeout_minutes`):
```rust
    pub last_restart_at: Option<chrono::NaiveDateTime>,
    pub last_restart_reason: Option<String>,
    pub health_check_timeout_seconds: Option<i32>,
```

**Also update the `INSERT` columns in `SqlxServerRepository::create` SQL** — add `last_restart_at, last_restart_reason, health_check_timeout_seconds` to the columns list and parameters list.

**⚠️ PITFALL NOTE:** The `model.rs` `Server` struct has `sqlx::FromRow` derive. The SELECT queries in `sqlx_repository.rs` list columns explicitly (not `SELECT *`). New columns MUST be added to every SELECT query column list AND the FROM clause SELECT list in that file.

---

### `api/src/domain/server/sqlx_repository.rs` (repository, CRUD)

**Analog:** self — existing `auto_wake`, `sleep_timeout_minutes` column wiring (lines 22-28 for INSERT SELECT, line 55 for find_by_id SELECT, etc.)

**There are TWO sets of methods to update:**

**Set 1: Direct methods** (lines 16-113) — `create`, `find_by_id`, `find_all`, `update`

**INSERT pattern** (lines 22-28, add new columns to both INSERT column list and RETURNING):
```sql
INSERT INTO servers (
    id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, 
    config, resources, auto_wake, sleep_timeout_minutes, endpoints
    -- ADD: , last_restart_at, last_restart_reason, health_check_timeout_seconds
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
--                                                                     ^-- increment to $19
RETURNING id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port,
    config, resources, auto_wake, sleep_timeout_minutes, endpoints, created_at, updated_at, deleted_at
    -- ADD: , last_restart_at, last_restart_reason, health_check_timeout_seconds
```

**Existing bind pattern** (lines 44-46):
```rust
        .bind(server.auto_wake)
        .bind(server.sleep_timeout_minutes)
```
**Add after:** `.bind(server.last_restart_at)`, `.bind(&server.last_restart_reason)`, `.bind(server.health_check_timeout_seconds)`

**SELECT pattern — update ALL SELECT column lists** (lines 55, 66, 152, 163, 174, 185, 196):
```
Add: , last_restart_at, last_restart_reason, health_check_timeout_seconds
To every column list in every method.
```

**UPDATE pattern** (lines 76-82):
```sql
UPDATE servers 
SET agent_id = $2, job_id = $3, name = $4, image = $5, executor_type = $6, node_id = $7, status = $8, remote_id = $9, port = $10,
    config = $11, resources = $12, auto_wake = $13, sleep_timeout_minutes = $14, endpoints = $15, updated_at = NOW()
    -- ADD: , last_restart_at = $16, last_restart_reason = $17, health_check_timeout_seconds = $18
WHERE id = $1 AND deleted_at IS NULL
```

**Set 2: `#[async_trait] impl ServerRepository`** (lines 115-251) — These are the trait impl methods that parallel the direct methods but use `RETURNING *` instead.

**INSERT pattern** (lines 119-121):
```sql
INSERT INTO servers (id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, endpoints, created_at, updated_at, deleted_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
RETURNING *
```
Add `, last_restart_at, last_restart_reason, health_check_timeout_seconds` after `sleep_timeout_minutes`, increment bind params accordingly, and add `.bind()` calls.

**UPDATE pattern** (lines 206-211):
```sql
UPDATE servers 
SET agent_id = $2, job_id = $3, name = $4, image = $5, status = $6, remote_id = $7, 
    config = $8, resources = $9, auto_wake = $10, sleep_timeout_minutes = $11, endpoints = $12, updated_at = NOW()
```
Add `, last_restart_at = $13, last_restart_reason = $14, health_check_timeout_seconds = $15`, increment subsequent params, add `.bind()` calls.

---

### `api/src/infrastructure/repositories/postgres_server_repository.rs` (repository, CRUD)

**Analog:** self — existing `auto_wake`, `sleep_timeout_minutes`, `last_player_activity` read/write pattern

**Updates needed in 7 locations:**

**1. INSERT** (line 26) — add columns to column list and VALUES:
```
-- Add to column list after `restart_cooldown_seconds`: last_restart_at, last_restart_reason, health_check_timeout_seconds
-- Add to VALUE bind after line 72: .bind(server.last_restart_at), .bind(&server.last_restart_reason), .bind(server.health_check_timeout_seconds)
-- Increment parameter numbers $44 → $47
```

**2. `find_by_id` SELECT** (line 84) — add columns to column list:
```
-- Add after `restart_cooldown_seconds`: , last_restart_at, last_restart_reason, health_check_timeout_seconds
```

**3. `find_by_id` row.try_get** — add after line 146:
```rust
                // Restart Policy & Health Check (Phase 57)
                last_restart_at: row.try_get("last_restart_at").ok().flatten(),
                last_restart_reason: row.try_get("last_restart_reason").ok().flatten(),
                health_check_timeout_seconds: row.try_get("health_check_timeout_seconds").unwrap_or(5),
```

**4. `list` SELECT** (line 158) — add columns to column list (same as find_by_id).

**5. `list` row.try_get** — add after line 220 (same block as find_by_id).

**6. `update` SQL** (line 235) — add columns to UPDATE SET and increment param numbers:
```
-- Add after `restart_cooldown_seconds = $41`: , last_restart_at = $42, last_restart_reason = $43, health_check_timeout_seconds = $44
```

**7. `update` .bind** — add after line 280:
```rust
        .bind(server.last_restart_at)
        .bind(&server.last_restart_reason)
        .bind(server.health_check_timeout_seconds)
        .bind(server.updated_at.naive_utc())  // increment existing bind
```

**⚠️ PITFALL:** Both `updated_at` bind already exists on different param numbers in `find_by_id` vs `update`. Check the param numbers carefully — in `update`, `updated_at` is param `$42` currently. Add the 3 new fields before it, shifting it to `$45`.

**8. `find_by_node_id` SELECT + row.try_get** (lines 322, 374-379) — same additions as `list`.

---

### `api/src/application/dto/server_dtos.rs` (dto, CRUD)

**Analog:** self — existing `max_restart_attempts`, `restart_cooldown_seconds`, `auto_wake` field pattern

**Add to `UpdateServerRequest`** (after line 92, before `// Git remote configuration`):
```rust
    // Restart Policy & Health Check (Phase 57)
    pub last_restart_at: Option<DateTime<Utc>>,
    pub last_restart_reason: Option<String>,
    pub health_check_timeout_seconds: Option<i32>,
```

**Add to `Default for UpdateServerRequest`** (after line 138):
```rust
            last_restart_at: None,
            last_restart_reason: None,
            health_check_timeout_seconds: None,
```

**Add to `ServerResponse`** (after line 181, before `created_at`):
```rust
    // Restart Policy & Health Check (Phase 57)
    pub last_restart_at: Option<DateTime<Utc>>,
    pub last_restart_reason: Option<String>,
    pub health_check_timeout_seconds: i32,
```

---

### `api/src/application/use_cases/create_server_use_case.rs` (use_case, CRUD)

**Analog:** self — existing `last_player_activity` and `max_restart_attempts` wiring (lines 66-71)

**Existing Phase 56 field block** (lines 66-71):
```rust
            // Sleep/Wake & Auto-Restart Backoff (Phase 56)
            auto_wake: req.auto_wake.unwrap_or(false),
            sleep_timeout_minutes: req.sleep_timeout_minutes.unwrap_or(30),
            last_player_activity: None,
            max_restart_attempts: req.max_restart_attempts.unwrap_or(5),
            restart_cooldown_seconds: req.restart_cooldown_seconds.unwrap_or(300),
```

**Add after line 71** (after `restart_cooldown_seconds`):
```rust

            // Restart Policy & Health Check (Phase 57)
            last_restart_at: None,
            last_restart_reason: None,
            health_check_timeout_seconds: req.health_check_timeout_seconds.unwrap_or(5),
```

---

### `api/src/application/use_cases/update_server_use_case.rs` (use_case, CRUD)

**Analog:** self — existing `max_restart_attempts` and `restart_cooldown_seconds` conditional block (lines 121-126)

**Existing conditional block** (lines 114-126):
```rust
        // Sleep/Wake & Auto-Restart Backoff (Phase 56)
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

**Add after line 126** (after `restart_cooldown_seconds` block):
```rust
        // Restart Policy & Health Check (Phase 57)
        if let Some(last_restart_at) = req.last_restart_at {
            server.last_restart_at = Some(last_restart_at);
        }
        if let Some(last_restart_reason) = req.last_restart_reason {
            server.last_restart_reason = if last_restart_reason.is_empty() { None } else { Some(last_restart_reason) };
        }
        if let Some(health_check_timeout_seconds) = req.health_check_timeout_seconds {
            server.health_check_timeout_seconds = health_check_timeout_seconds;
        }
```

**⚠️ NOTE:** `last_restart_at` and `last_restart_reason` are set by the MonitoringService (not typically via user update), but should be available in the DTO for admin use or future manual override. `health_check_timeout_seconds` is user-configurable in the Settings tab.

---

### `api/src/presentation/handlers/server_handlers.rs` (controller, request-response)

**Analog:** self — existing `update_server` handler wiring for `auto_wake`/`sleep_timeout_minutes` (lines 576-581)

**This file has TWO server models to handle:**

**For `SqlxServerRepository` (the `update_server` handler, lines 550-588):**

**Existing conditional wiring** (lines 576-581):
```rust
    if let Some(auto_wake) = payload.auto_wake {
        server.auto_wake = Some(auto_wake);
    }
    if let Some(sleep_timeout_minutes) = payload.sleep_timeout_minutes {
        server.sleep_timeout_minutes = Some(sleep_timeout_minutes);
    }
```

**Add after line 581** (before `let updated = repo.update`):
```rust
    if let Some(health_check_timeout_seconds) = payload.health_check_timeout_seconds {
        server.health_check_timeout_seconds = Some(health_check_timeout_seconds);
    }
```

---

### `api/src/application/services/monitoring_service.rs` (service, request-response)

**Analog:** self — existing sleep detection block (lines 215-268) and crash detection block (lines 137-189)

**This is the most complex change.** The RCON health check needs to be injected between `collect_metrics` for player count (lines 218-267) and the existing metrics collection (lines 270-306).

**Current order of operations in `check_all_servers()` for a running server:**

```
Lines 215-268:  Sleep detection (Phase 56) — uses executor.collect_metrics() for player count
Lines 270-306:  Metrics collection (existing) — uses executor.collect_metrics() for TPS/CPU/etc
```

**New order (Phase 57):**

```
Lines 215-268:  Sleep detection (Phase 56) — unchanged
NEW: Lines 268-290: RCON health check — if `health_check_timeout_seconds` > 0, try RCON ping
Lines 270-306:  Metrics collection (existing) — unchanged
```

**Existing sleep detection block structure** (lines 215-268) — the pattern for the health check block:
```rust
                    // === SLEEP DETECTION (Phase 56) ===
                    // Check running servers for player inactivity
                    if status == "running" && server.auto_wake {
                        match executor.collect_metrics(&server).await {
                            Ok(metrics) => {
                                if metrics.players > 0 {
                                    // Players online — reset last_player_activity timestamp
                                    let mut updated = server.clone();
                                    updated.last_player_activity = Some(chrono::Utc::now());
                                    if let Err(e) = self.repository.update(&updated).await {
                                        tracing::warn!("[MONITOR] Failed to update last_player_activity for {}: {}", server.name, e);
                                    }
                                } else if let Some(last_activity) = server.last_player_activity {
                                    // No players — check inactivity timeout
                                    let elapsed = chrono::Utc::now() - last_activity;
                                    let timeout = chrono::Duration::minutes(server.sleep_timeout_minutes as i64);
                                    if elapsed >= timeout {
                                        // Trigger sleep
                                        ...
                                    }
                                }
                            }
                            Err(e) => { ... }
                        }
                    }
```

**RCON health check block to add** (AFTER sleep detection `}` closing brace on line 268, BEFORE metrics collection on line 270):
```rust
                    // === RCON HEALTH CHECK (Phase 57) ===
                    // Probe RCON for running servers to detect unresponsive state
                    if status == "running" && server.auto_restart && server.health_check_timeout_seconds > 0 {
                        match executor.collect_metrics(&server).await {
                            Ok(metrics) => {
                                // If collect_metrics succeeded, RCON responded — server is healthy
                                // Optionally log or reset any unresponsive state
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "[MONITOR] Server {} RCON health check failed: {}. Marking as unresponsive.",
                                    server.name, e
                                );
                                // RCON failed — mark as unresponsive and trigger restart
                                let mut updated = server.clone();
                                updated.last_restart_reason = Some("unresponsive".to_string());
                                updated.last_restart_at = Some(chrono::Utc::now());
                                if let Err(e) = self.repository.update(&updated).await {
                                    tracing::error!("[MONITOR] Failed to update restart fields for {}: {}", server.name, e);
                                }
                                // Trigger auto-restart logic (same pattern as crash detection)
                                // ... follow crash detection restart pattern (lines 140-180) ...
                            }
                        }
                    }
```

**Key integration points:**
- The health check uses the same `executor.collect_metrics()` that already exists — it will fail if RCON is unreachable
- A single RCON failure may be transient; consider tracking consecutive failures (agent's discretion)
- After marking unresponsive, the server should go through the same restart path as crash detection (lines 140-180)

**Existing crash-detection restart pattern for re-use** (lines 140-180):
```rust
                        if server.status == "running" && status == "stopped" {
                            match self.repository.find_by_id(&server.id).await {
                                Ok(Some(full_server)) => {
                                    if full_server.auto_restart {
                                        let max_attempts = full_server.max_restart_attempts;
                                        let current_count = full_server.restart_count;
                                        if current_count >= max_attempts {
                                            tracing::error!(...);
                                        } else {
                                            let backoff_secs = std::cmp::min(
                                                30u32 * 2u32.pow(current_count as u32),
                                                full_server.restart_cooldown_seconds as u32,
                                            );
                                            // Spawn delayed restart
                                            let repo_clone = self.repository.clone();
                                            let factory_clone = self.executor_factory.clone();
                                            let server_clone = full_server.clone();
                                            tokio::spawn(async move {
                                                tokio::time::sleep(std::time::Duration::from_secs(backoff_secs as u64)).await;
                                                let exec = factory_clone.get_executor(&server_clone);
                                                match exec.start_server(&server_clone).await {
                                                    Ok(_) => {
                                                        let mut updated = server_clone.clone();
                                                        updated.restart_count = current_count + 1;
                                                        let _ = repo_clone.update(&updated).await;
                                                    }
                                                    Err(e) => { ... }
                                                }
                                            });
                                        }
                                        continue;
                                    }
                                }
                                ...
                            }
                        }
```

---

### `app/src/pages/ServerDetails.jsx` (component, request-response)

**Analog:** self — existing Sleep & Wake config section (lines 409-470)

**Sleep & Wake section pattern** (lines 409-470) — follow exactly for Restart Policy section:

```jsx
                        {/* ─── SLEEP & WAKE CONFIG ─── */}
                        <section className="glass-panel p-6 mt-6">
                            <h3 className="text-lg font-bold mb-1">Sleep & Wake</h3>
                            <p className="text-xs text-[var(--color-text-muted)] mb-5">
                                Automatically stop server when inactive and wake on demand.
                            </p>

                            {sleepToast && (
                                <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${sleepToast.type === 'success'
                                    ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
                                    : 'bg-red-500/10 border-red-500/30 text-red-400'
                                    }`}>
                                    {sleepToast.message}
                                </div>
                            )}

                            {/* Auto Sleep Toggle */}
                            <div className="flex items-center gap-3 p-4 rounded-xl border cursor-pointer
                                            hover:border-[var(--color-cosmic-cyan)]/50"
                                 onClick={() => setAutoWake(!autoWake)}>
                                <div className={`w-12 h-6 rounded-full transition-colors
                                                ${autoWake ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-[var(--color-cosmic-border)]'}`}>
                                    <div className={`w-5 h-5 rounded-full bg-white transition-transform
                                                    ${autoWake ? 'translate-x-6' : 'translate-x-0.5'}`} />
                                </div>
                                <div className="flex-1">
                                    <p className="text-sm font-bold">Auto Sleep</p>
                                    <p className="text-xs text-[var(--color-text-muted)]">Stop server after inactivity</p>
                                </div>
                            </div>

                            {/* Sleep Timeout (visible only when toggle ON) */}
                            {autoWake && (
                                <div className="mt-4">
                                    <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">
                                        Sleep after (minutes of 0 players)
                                    </label>
                                    <input type="number"
                                           value={sleepTimeout}
                                           min={5} max={240}
                                           onChange={e => setSleepTimeout(Math.max(5, Math.min(240, parseInt(e.target.value) || 30)))}
                                           className="w-full px-4 py-2.5 rounded-lg text-sm
                                                       bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)]
                                                       text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)]
                                                       focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all" />
                                    <p className="text-[10px] text-[var(--color-text-muted)] mt-2">
                                        Server will auto-sleep after this many minutes with zero players.
                                    </p>
                                </div>
                            )}

                            {/* Save Button */}
                            <button
                                disabled={sleepSaving}
                                onClick={handleSaveSleepConfig}
                                className="mt-5 w-full py-2.5 rounded-lg text-sm font-bold
                                           bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                                           hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                                           disabled:opacity-50 transition-all">
                                {sleepSaving ? 'Saving...' : 'Save Changes'}
                            </button>
                        </section>
```

**Restart Policy section** — add AFTER the Sleep & Wake section (after line 470, before the closing `</div>` on line 472):

```jsx
                        {/* ─── RESTART POLICY CONFIG (Phase 57) ─── */}
                        <section className="glass-panel p-6 mt-6">
                            <h3 className="text-lg font-bold mb-1">Restart Policy</h3>
                            <p className="text-xs text-[var(--color-text-muted)] mb-5">
                                Automatically restart server on crash or unresponsive state.
                            </p>

                            {restartToast && (
                                <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${restartToast.type === 'success'
                                    ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
                                    : 'bg-red-500/10 border-red-500/30 text-red-400'
                                    }`}>
                                    {restartToast.message}
                                </div>
                            )}

                            {/* Auto Restart Toggle */}
                            <div className="flex items-center gap-3 p-4 rounded-xl border cursor-pointer
                                            hover:border-[var(--color-cosmic-cyan)]/50"
                                 onClick={() => setAutoRestart(!autoRestart)}>
                                <div className={`w-12 h-6 rounded-full transition-colors
                                                ${autoRestart ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-[var(--color-cosmic-border)]'}`}>
                                    <div className={`w-5 h-5 rounded-full bg-white transition-transform
                                                    ${autoRestart ? 'translate-x-6' : 'translate-x-0.5'}`} />
                                </div>
                                <div className="flex-1">
                                    <p className="text-sm font-bold">Auto Restart</p>
                                    <p className="text-xs text-[var(--color-text-muted)]">Restart on crash or unresponsive</p>
                                </div>
                            </div>

                            {/* Max Restart Attempts (visible only when toggle ON) */}
                            {autoRestart && (
                                <>
                                    <div className="mt-4">
                                        <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">
                                            Max Restart Attempts
                                        </label>
                                        <input type="number"
                                               value={maxRestartAttempts}
                                               min={1} max={20}
                                               onChange={e => setMaxRestartAttempts(Math.max(1, Math.min(20, parseInt(e.target.value) || 5)))}
                                               className="w-full px-4 py-2.5 rounded-lg text-sm
                                                           bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)]
                                                           text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)]
                                                           focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all" />
                                        <p className="text-[10px] text-[var(--color-text-muted)] mt-2">
                                            Maximum automatic restart attempts before giving up.
                                        </p>
                                    </div>

                                    <div className="mt-4">
                                        <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">
                                            Restart Cooldown (seconds)
                                        </label>
                                        <input type="number"
                                               value={restartCooldown}
                                               min={30} max={3600}
                                               onChange={e => setRestartCooldown(Math.max(30, Math.min(3600, parseInt(e.target.value) || 300)))}
                                               className="w-full px-4 py-2.5 rounded-lg text-sm
                                                           bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)]
                                                           text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)]
                                                           focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all" />
                                        <p className="text-[10px] text-[var(--color-text-muted)] mt-2">
                                            Wait time between restart attempts (exponential backoff up to this cap).
                                        </p>
                                    </div>

                                    <div className="mt-4">
                                        <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">
                                            Health Check Timeout (seconds)
                                        </label>
                                        <input type="number"
                                               value={healthCheckTimeout}
                                               min={1} max={60}
                                               onChange={e => setHealthCheckTimeout(Math.max(1, Math.min(60, parseInt(e.target.value) || 5)))}
                                               className="w-full px-4 py-2.5 rounded-lg text-sm
                                                           bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)]
                                                           text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)]
                                                           focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all" />
                                        <p className="text-[10px] text-[var(--color-text-muted)] mt-2">
                                            RCON health check timeout. Server marked unresponsive if exceeded.
                                        </p>
                                    </div>
                                </>
                            )}

                            {/* Restart History Display (always visible) */}
                            <div className="mt-4 p-4 rounded-xl bg-[rgba(255,255,255,0.02)] border border-[var(--color-cosmic-border)]">
                                <p className="text-xs font-bold text-[var(--color-text-muted)] mb-2 uppercase tracking-wider">
                                    Restart History
                                </p>
                                <div className="grid grid-cols-2 gap-3 text-sm">
                                    <div>
                                        <span className="text-[var(--color-text-muted)] text-xs">Restart Count:</span>
                                        <p className="font-bold">{server.restart_count ?? 0}</p>
                                    </div>
                                    <div>
                                        <span className="text-[var(--color-text-muted)] text-xs">Last Restart:</span>
                                        <p className="font-bold">
                                            {server.last_restart_at
                                                ? new Date(server.last_restart_at).toLocaleString()
                                                : '—'}
                                        </p>
                                    </div>
                                    {server.last_restart_reason && (
                                        <div className="col-span-2">
                                            <span className="text-[var(--color-text-muted)] text-xs">Reason:</span>
                                            <p className="font-bold text-[var(--color-cosmic-orange)]">
                                                {server.last_restart_reason}
                                            </p>
                                        </div>
                                    )}
                                </div>
                            </div>

                            {/* Save Button */}
                            <button
                                disabled={restartSaving}
                                onClick={handleSaveRestartConfig}
                                className="mt-5 w-full py-2.5 rounded-lg text-sm font-bold
                                           bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                                           hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                                           disabled:opacity-50 transition-all">
                                {restartSaving ? 'Saving...' : 'Save Changes'}
                            </button>
                        </section>
```

**State variables needed** (add after line 60, after sleep toast state):
```jsx
    // Restart Policy state
    const [autoRestart, setAutoRestart] = useState(false);
    const [maxRestartAttempts, setMaxRestartAttempts] = useState(5);
    const [restartCooldown, setRestartCooldown] = useState(300);
    const [healthCheckTimeout, setHealthCheckTimeout] = useState(5);
    const [restartSaving, setRestartSaving] = useState(false);
    const [restartToast, setRestartToast] = useState(null);
```

**Load from server data** (add to the `fetchApi` block, after line 82):
```jsx
            setAutoRestart(data.auto_restart || false);
            setMaxRestartAttempts(data.max_restart_attempts ?? 5);
            setRestartCooldown(data.restart_cooldown_seconds ?? 300);
            setHealthCheckTimeout(data.health_check_timeout_seconds ?? 5);
```

**Save handler** (add after `handleSaveSleepConfig`, before line 216 renders):
```jsx
    const handleSaveRestartConfig = async () => {
        try {
            setRestartSaving(true);
            await updateServer(id, {
                auto_restart: autoRestart,
                max_restart_attempts: maxRestartAttempts,
                restart_cooldown_seconds: restartCooldown,
                health_check_timeout_seconds: healthCheckTimeout,
            });
            setServer(prev => ({
                ...prev,
                auto_restart: autoRestart,
                max_restart_attempts: maxRestartAttempts,
                restart_cooldown_seconds: restartCooldown,
                health_check_timeout_seconds: healthCheckTimeout,
            }));
            setRestartToast({ type: 'success', message: '✅ Restart policy saved' });
        } catch (e) {
            setRestartToast({ type: 'error', message: `❌ Could not save restart policy. ${e.message}` });
        } finally {
            setRestartSaving(false);
            setTimeout(() => setRestartToast(null), 4000);
        }
    };
```

---

### `app/src/hooks/useServers.js` (hook, request-response)

**Analog:** self — existing `updateServer` function (lines 92-94)

**No changes needed.** The `updateServer` function already exists and accepts arbitrary JSON:
```js
export async function updateServer(id, data) {
    return fetchApi(`/servers/${id}`, { method: 'PUT', body: JSON.stringify(data) });
}
```

The frontend Restart Policy section will call `updateServer(id, { auto_restart: ..., max_restart_attempts: ..., ... })` the same way the Sleep & Wake section already does with `updateServer(id, { auto_wake: ..., sleep_timeout_minutes: ... })`.

---

## Shared Patterns

### Authentication & Tenant Access
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 563-565 for update_server, lines 596-598 for delete_server)
**Apply to:** `update_server` handler (already has it)
```rust
    // Check tenant access
    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }
```

### Error Handling — Handlers
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 587, 1031, 1048)
**Apply to:** `update_server` handler
```rust
    // Success response
    Ok(Json(ApiResponse::success(updated)))
    
    // Error response
    Err("Server not found".to_string())
    .map_err(|e| e.to_string())?
```

### Event Emission Pattern (Restart Events)
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 1029, 1048)
**Apply to:** MonitoringService when triggering restart (for `server.restarted`, `server.restart_limit_reached`)
```rust
emit_server_event(&state.pool, "server.restarted", user_id, id, &server_name).await;
```

**Pattern for limit reached:**
```rust
emit_server_event(&state.pool, "server.restart_limit_reached", user_id, id, &server_name).await;
```

### MonitoringService Event Bus
**Source:** `api/src/application/services/monitoring_service.rs` (lines 130-133, 246-249)
**Apply to:** RCON health check restart triggering
```rust
let _ = self.event_bus.publish(ServerEvent::StatusChanged {
    server_id: server.id,
    status: "stopped".to_string(),
});
```

### Tokio::spawn Delayed Restart (Non-Blocking)
**Source:** `api/src/application/services/monitoring_service.rs` (lines 163-180)
**Apply to:** RCON health check restart (to avoid blocking the monitoring loop)
```rust
let repo_clone = self.repository.clone();
let factory_clone = self.executor_factory.clone();
let server_clone = full_server.clone();
tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs as u64)).await;
    let exec = factory_clone.get_executor(&server_clone);
    match exec.start_server(&server_clone).await {
        Ok(_) => {
            let mut updated = server_clone.clone();
            updated.restart_count = current_count + 1;
            let _ = repo_clone.update(&updated).await;
        }
        Err(e) => {
            tracing::error!("[MONITOR] Backed-off auto-restart failed for {}: {}", server_clone.name, e);
        }
    }
});
```

### Toast Notification Pattern (Frontend)
**Source:** `app/src/pages/ServerDetails.jsx` (lines 360-367, 416-423)
**Apply to:** Restart Policy section save toast + restart event toasts
```jsx
{sleepToast && (
    <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${sleepToast.type === 'success'
        ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
        : 'bg-red-500/10 border-red-500/30 text-red-400'
        }`}>
        {sleepToast.message}
    </div>
)}
```

### Tailwind Input Field Pattern
**Source:** `app/src/pages/ServerDetails.jsx` (lines 370-378, 449-453)
**Apply to:** All number input fields in Restart Policy section
```jsx
<input type="number" value={...} min={...} max={...}
       onChange={e => setState(Math.max(..., Math.min(..., parseInt(e.target.value) || default)))}
       className="w-full px-4 py-2.5 rounded-lg text-sm
                   bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)]
                   text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)]
                   focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all" />
```

---

## No Analog Found

All files have a direct match in the existing codebase — no file requires RESEARCH.md-only patterns.

| File | Reason for Full Match |
|------|----------------------|
| All 12 files | Each new/modified file follows the exact pattern of an existing analog in the same role, most from Phase 56's auto_wake/sleep implementation |

## Key Warnings from Code Analysis

### Pitfall A: Two Server Models Exist
There are **two** Server structs in the codebase:
- `api/src/domain/entities/server.rs` (old — used by `monitoring_service.rs`, `create_server_use_case.rs`, `update_server_use_case.rs` via `ServerRepository` trait, and `PostgresServerRepository`)
- `api/src/domain/server/model.rs` (new — used by `server_handlers.rs` via `SqlxServerRepository`)

**ALL THREE field additions must appear in BOTH models** plus all their repositories.

### Pitfall B: SqlxRepository Has Dual Method Sets
`sqlx_repository.rs` has direct methods (lines 16-113) AND `#[async_trait] impl ServerRepository` methods (lines 115-251). Both method sets have their own SQL strings with explicit column lists. **Every column list must be updated in both sets.**

### Pitfall C: PostgresServerRepository Has 6+ Query Locations
The `postgres_server_repository.rs` has different SQL strings and row-building in:
1. `create` (line 26) — INSERT
2. `find_by_id` (line 84) — SELECT + row.try_get builder (lines 96-150)
3. `list` (line 158) — SELECT + row.try_get builder (lines 170-225)
4. `update` (line 235) — UPDATE
5. `find_by_node_id` (line 322) — SELECT + row.try_get builder (lines 334-383)

All 5 need column list updates in the SQL + `try_get` calls in the builder blocks.

### Pitfall D: New Model Fields in `model.rs` Must Be `Option`
The `model.rs` Server struct uses `sqlx::FromRow`, so `last_restart_at` must be `Option<chrono::NaiveDateTime>`, `last_restart_reason` must be `Option<String>`, and `health_check_timeout_seconds` must be `Option<i32>` since they have `NOT NULL DEFAULT` in the migration but `FromRow` needs nullable-compatible types for ADD COLUMN migration safety.

### Pitfall E: Monitoring Loop Is Sequential
The `check_all_servers()` loop (lines 78-314) processes servers one by one. Any blocking operation inside will stall ALL server monitoring. The RCON health check should be quick (timeout default 5s) but even so, `tokio::spawn` is recommended for the restart action itself (as already done in crash detection lines 163-180).

### Pitfall F: EventBus vs Direct emit_server_event
The codebase uses two event mechanisms:
1. **EventBus** (`event_bus.publish`) — internal in-process, used by monitoring_service for status changes
2. **emit_server_event** — writes to DB for WebSocket broadcast + audit trail, used by server_handlers

The RCON health check runs inside `monitoring_service.rs`, so restart events should use `event_bus.publish` for internal state tracking and optionally the WebhookService directly if WebSocket broadcast is needed. Alternatively, the handler path (when user triggers restart) uses `emit_server_event`.

## Metadata

**Analog search scope:** `api/src/domain/entities/server.rs`, `api/src/domain/server/model.rs`, `api/src/domain/server/sqlx_repository.rs`, `api/src/infrastructure/repositories/postgres_server_repository.rs`, `api/src/application/dto/server_dtos.rs`, `api/src/application/use_cases/create_server_use_case.rs`, `api/src/application/use_cases/update_server_use_case.rs`, `api/src/application/services/monitoring_service.rs`, `api/src/presentation/handlers/server_handlers.rs`, `api/migrations/20260530000001_add_auto_wake.sql`, `app/src/pages/ServerDetails.jsx`, `app/src/hooks/useServers.js`
**Files scanned:** 12 primary files, 40+ traversed via Phase 56 pattern map
**Pattern extraction date:** 2026-05-30
