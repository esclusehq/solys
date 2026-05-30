# Phase 56: Auto Online & Sleep Recovery - Research

**Researched:** 2026-05-30
**Domain:** Server lifecycle automation (sleep detection, wake-up, auto-restart refinement)
**Confidence:** HIGH

## Summary

This phase delivers automatic sleep/wake recovery for game servers. Three capabilities must be built on the existing MonitoringService infrastructure: (1) **inactivity-based sleep detection** via the existing 30s monitoring loop checking player count from RCON metrics, (2) **manual sleep action** via a new API endpoint that stops the server while setting `auto_wake=true`, and (3) **automatic wake-up** triggered when a player connection attempt is detected (or via API). Additionally, the existing crash→auto-restart logic (lines 137-189 of monitoring_service.rs) needs refinement with exponential backoff, max attempt limits, and cooldown periods.

The core data model change is minimal: add `auto_wake: bool` and `sleep_timeout_minutes: Option<i32>` fields to the Server entity, following the exact same pattern used for `auto_restart` and `auto_pause`. D-01 explicitly states **no new 'sleeping' status** — the existing 'stopped' status is reused, with `auto_wake=true` distinguishing a sleep-stopped server from a user-stopped one.

**Primary recommendation:** Extend MonitoringService by injecting sleep detection between the existing crash-detection and metrics-collection blocks (lines ~189-207). Add a new `SleepService` for configurable timeout tracking. Follow the existing `auto_restart` wiring pattern through DTOs, use cases, and handlers for `auto_wake`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** No new 'sleeping' status. Keep existing server status values ('running', 'starting', 'stopped', 'container_running'). Add an `auto_wake` boolean field to indicate a server that is stopped but will auto-recover. UI shows a different badge for servers with `auto_wake=true` + status=`stopped`.
- **D-02:** Sleep triggers — both manual and automatic. User can click 'Sleep' in the UI (same as stop but sets `auto_wake=true`), AND servers auto-sleep after player inactivity timeout. Both paths produce the same state: status=`stopped`, `auto_wake=true`.
- **D-03:** Inactivity detection via existing monitoring loop. MonitoringService (30s tick) checks player count via executor (RCON). If 0 players for >X configurable minutes, triggers sleep. No agent-side changes needed.
- **D-04:** `auto_pause` kept separate. The existing `auto_pause` field is unrelated to sleep behavior and preserved for future use (pause = freeze in memory, sleep = stop + auto-restore).

### the agent's Discretion
- Specific inactivity timeout duration and configuration (default, min/max)
- Wake-up trigger implementation (player connection attempt detection, API-based wake)
- Auto-restart refinement: max restart attempts, cooldown periods, exponential backoff, failure alerts
- UI placement of sleep/wake configuration (server settings tab vs inline controls)
- Database migration design for `auto_wake` column
- MonitoringService integration details (where in the 30s loop to inject sleep detection)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Inactivity sleep detection | API / Backend | — | Runs in MonitoringService loop (30s tick), evaluates player count from RCON metrics already collected |
| Manual sleep action | API / Backend | Browser / Client | API endpoint stops server + sets auto_wake=true; frontend provides UI trigger |
| Auto wake-up | API / Backend | — | Detected via monitoring loop (node check or player connect attempt), triggers start via executor |
| Auto-restart backoff logic | API / Backend | — | Tracking restart_count, max_attempts, cooldown — all in MonitoringService |
| Sleep/wake config UI | Browser / Client | — | React component in server settings tab, sends via existing updateServer API |
| Status badge distinction | Browser / Client | — | UI-side logic: if status=='stopped' && auto_wake, show 'Sleeping' badge |

## Standard Stack

### Core — No new dependencies needed

All work uses the existing stack. No new Rust crates or npm packages required.

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio::time | v1 | Async interval timer | MonitoringService already uses it for 30s loop |
| rcon | v0.6 | Player count via RCON `list` | Already integrated in `rcon_server_executor.rs::collect_metrics` |
| sqlx | v0.7 | DB migration + queries | Existing migration pattern for auto_wake column |
| chrono | v0.4 | Time tracking for inactivity | Used for `last_player_activity` comparison |
| react | v19.2.4 | Frontend UI | Existing framework |

### Installation — None required

```
# No new dependencies. All work extends existing services.
```

### Version verification

Existing deps already confirmed via `api/Cargo.toml`.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  API Backend (Axum)                                             │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  MonitoringService (30s loop)                            │   │
│  │                                                          │   │
│  │  1. Check all servers on node                             │   │
│  │  2. Skip if offline node / not-running                   │   │
│  │  3. ┌─────────────────────────────────────┐              │   │
│  │     │ CRASH DETECTION (existing)            │             │   │
│  │     │ status=="running" → "stopped"         │             │   │
│  │     │ auto_restart=true → executor.start()  │             │   │
│  │     └─────────────────────────────────────┘              │   │
│  │  4. ┌─────────────────────────────────────┐  ← NEW       │   │
│  │     │ SLEEP DETECTION                      │             │   │
│  │     │ auto_wake=true → check inactivity    │             │   │
│  │     │ players==0 for >timeout → sleep()    │             │   │
│  │     └─────────────────────────────────────┘              │   │
│  │  5. ┌─────────────────────────────────────┐  ← NEW       │   │
│  │     │ AUTO-RESTART BACKOFF                 │             │   │
│  │     │ track attempt count + cooldown       │             │   │
│  │     │ max attempts hit → alert             │             │   │
│  │     └─────────────────────────────────────┘              │   │
│  │  6. Collect metrics (existing)                            │   │
│  └───────┬─────────────────────────────────────────────┘   │
│          │                                                    │
│  ┌───────▼─────────────────────────────────────────────┐   │
│  │  SleepService (NEW — optional extracted type)         │   │
│  │  - tracks timeout duration per server                 │   │
│  │  - stores last_player_activity timestamp              │   │
│  │  - evaluates if inactivity threshold exceeded         │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Wake-up Handler (NEW)                              │   │
│  │  - POST /api/v1/servers/:id/wake                    │   │
│  │  - changes status to "starting" + executor.start()  │   │
│  │  - resets auto_wake to false after wake              │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Sleep Handler (NEW)                                │   │
│  │  - POST /api/v1/servers/:id/sleep                   │   │
│  │  - executor.stop() + status="stopped" + auto_wake   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  Server Status Model                                        │
│  status: running / stopped / starting / container_running   │
│  auto_wake: bool (distinguishes sleeping from stopped)      │
│  sleep_timeout_minutes: i32 (server-level timeout)           │
│  restart_count: i32 (shared with crash-restart)             │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  Frontend (React)                                           │
│  - ServerDetails.jsx Settings tab: sleep timeout config     │
│  - StatusBadge component: detect auto_wake+stopped→"💤"     │
│  - Action button: Sleep / Wake toggle (new)                 │
│  - ServerManager.jsx: show sleeping badge in list           │
└─────────────────────────────────────────────────────────────┘
```

### Flow: Inactivity Sleep Detection

```
MonitoringService::check_all_servers()
  │
  ├─ For each running server:
  │   ├─ Collect metrics (includes players count via RCON)
  │   ├─ If auto_wake == true (already sleeping): skip
  │   ├─ If players == 0:
  │   │   ├─ Read last_player_activity from server record
  │   │   ├─ If no activity recorded → set now as last activity
  │   │   ├─ If elapsed > sleep_timeout_minutes:
  │   │   │   ├─ executor.stop_server(server).await
  │   │   │   ├─ repository.update_status(id, "stopped")
  │   │   │   ├─ repository.update({ auto_wake: true })
  │   │   │   ├─ event_bus.publish(StatusChanged { status: "stopped" })
  │   │   │   └─ (Frontend receives WS event, shows "Sleeping")
  │   │   └─ If elapsed < timeout: continue (not yet expired)
  │   └─ If players > 0: reset last_player_activity timer
  └─ Done
```

### Flow: Wake-up (Auto or Manual)

```
Wake Trigger (any of):
  A. Player attempts to connect to stopped server
     ─ Needs port monitoring OR DNS/web proxy detection
     ─ Complex: requires external probe. Defer to API-based.
  
  B. User clicks "Wake" in UI
     ─ POST /api/v1/servers/:id/wake
     ├─ Fetch server, verify auto_wake==true
     ├─ executor.start_server(server).await
     ├─ repository.update({ auto_wake: false, status: "starting" })
     ├─ event_bus.publish(StatusChanged)
     └─ (Frontend updates via WS)

  C. Scheduled wake (Phase 59 — deferred)
```

### Auto-Restart Backoff Pattern

Extend the existing crash→restart block in monitoring_service.rs (lines 137-189):

```
trigger_auto_restart(server):
  if restart_count >= max_restart_attempts:
    publish AlertTriggered("Max restart attempts reached")
    return  // Give up
      
  backoff_seconds = min(
    initial_delay * 2^restart_count,  // exponential: 30s, 60s, 120s, 240s...
    max_cooldown_seconds              // cap at e.g. 300s (5 min)
  )
  
  tokio::time::sleep(Duration::from_secs(backoff_seconds)).await
  executor.start_server(server)
  repository.update({ restart_count: restart_count + 1 })
  
  // Reset restart_count after server runs for N minutes successfully
  // (done in monitoring loop: if status=="running" for >5min → reset count)
```

### Recommended Project Structure

```
api/src/
├── application/
│   └── services/
│       ├── monitoring_service.rs    ← MODIFY: add sleep detection, backoff
│       ├── sleep_service.rs         ← NEW (optional): sleep timeout tracking
│       └── node_health_service.rs   ← REFERENCE: configurable-interval pattern
├── domain/
│   └── entities/
│       └── server.rs                ← MODIFY: add auto_wake, sleep_timeout_minutes
├── application/
│   └── dto/
│       └── server_dtos.rs           ← MODIFY: add auto_wake to DTOs
├── application/
│   └── use_cases/
│       ├── create_server_use_case.rs ← MODIFY: wire auto_wake default
│       └── update_server_use_case.rs ← MODIFY: wire auto_wake update
├── presentation/
│   └── handlers/
│       ├── server_handlers.rs       ← MODIFY: add sleep/wake endpoints
│       └── settings_handlers.rs     ← REFERENCE: settings endpoint pattern
├── infrastructure/
│   └── events/
│       └── event_bus.rs             ← UNCHANGED: existing pattern sufficient
└── migrations/
    └── 20260530000001_add_auto_wake.sql ← NEW migration

app/src/
├── pages/
│   └── ServerDetails.jsx            ← MODIFY: add sleep/wake in Settings tab
├── hooks/
│   └── useServers.js                ← MODIFY (minor): add sleep/wake API calls
└── components/
    └── StatusBadge.jsx              ← MODIFY: show sleeping badge
```

### Pattern 1: Boolean Field Addition (same as `auto_restart`)
**What:** Adding a boolean server feature flag following the exact `auto_restart` and `auto_pause` patterns implemented previously.
**When to use:** Adding `auto_wake` to the Server entity, DTOs, use cases, and migrations.

**Example — DB Migration:**
```sql
-- api/migrations/20260530000001_add_auto_wake.sql
ALTER TABLE servers ADD COLUMN IF NOT EXISTS auto_wake BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS sleep_timeout_minutes INTEGER NOT NULL DEFAULT 30;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS max_restart_attempts INTEGER NOT NULL DEFAULT 5;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS restart_cooldown_seconds INTEGER NOT NULL DEFAULT 300;
```

**Example — Server Entity:**
```rust
// api/src/domain/entities/server.rs — add fields:
pub auto_wake: bool,
pub sleep_timeout_minutes: i32,
pub max_restart_attempts: i32,
pub restart_cooldown_seconds: i32,
```

**Example — DTO (following auto_restart pattern):**
```rust
// api/src/application/dto/server_dtos.rs — in UpdateServerRequest:
pub auto_wake: Option<bool>,
pub sleep_timeout_minutes: Option<i32>,
```

**Example — Use Case wiring:**
```rust
// api/src/application/use_cases/update_server_use_case.rs:
if let Some(auto_wake) = req.auto_wake {
    server.auto_wake = auto_wake;
}
if let Some(sleep_timeout) = req.sleep_timeout_minutes {
    server.sleep_timeout_minutes = sleep_timeout;
}
```

### Pattern 2: Sleep Detection Injection
**What:** Inject sleep detection logic into the existing monitoring loop.
**When to use:** After crash detection (line ~189) and before metrics collection (line ~207).

**Example — monitoring_service.rs sleep detection block:**
```rust
// Insert after crash-detection block (after line ~189), before metrics (line ~207)

// === SLEEP DETECTION (Phase 56) ===
// Check servers with auto_wake=true that are NOT currently running
if server.status == "stopped" && server.auto_wake {
    // This server is "sleeping" — check if it should wake up
    // (wake-up logic: player connection attempt or timed wake)
    continue;  // Skip metrics for sleeping servers
}

// Check running servers for inactivity sleep
if server.status == "running" && server.metrics_available {
    let metrics = executor.collect_metrics(&server).await?;
    if metrics.players == 0 {
        // Track inactivity using server-fields approach:
        // track zero-player duration, trigger sleep if exceeded
        if zero_player_elapsed >= server.sleep_timeout_minutes * 60 {
            // Trigger sleep
            executor.stop_server(&server).await?;
            repository.update_status(&server.id, "stopped").await?;
            // Set auto_wake
            let mut updated = server.clone();
            updated.auto_wake = true;
            repository.update(&updated).await?;
            event_bus.publish(ServerEvent::StatusChanged {
                server_id: server.id,
                status: "stopped".to_string(),
            });
            continue;  // Skip further processing
        }
    }
}
```

**Simpler approach (recommended):** Instead of tracking zero-player duration separately in memory, track `last_player_activity` as a `TIMESTAMPTZ` column on the servers table. The monitoring loop checks this field:

```
if metrics.players > 0 {
    // Players online — reset last activity timestamp
    sqlx::query("UPDATE servers SET last_player_activity = NOW() WHERE id = $1")
        .bind(server.id).execute(&pool).await?;
} else if let Some(last_activity) = server.last_player_activity {
    // No players — check if timeout exceeded
    let elapsed = Utc::now() - last_activity;
    let timeout = chrono::Duration::minutes(server.sleep_timeout_minutes as i64);
    if elapsed >= timeout {
        // Trigger sleep
        ...
    }
}
```

### Anti-Patterns to Avoid
- **In-memory-only tracking:** Don't track inactivity timeouts only in memory (lost on restart). Use the `last_player_activity` database field approach for persistence.
- **Sleep as separate status:** D-01 explicitly forbids this. Use `auto_wake=true + status=stopped`.
- **Agent-side changes:** D-03 says no agent-side changes. All logic stays in the API backend.
- **Repurposing auto_pause:** D-04 keeps auto_pause separate. Don't reuse it for sleep.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Async interval timer | Custom timer | `tokio::time::interval` | Already used by MonitoringService |
| RCON player count | Custom protocol | `rcon` v0.6 crate | Already integrated, `parse_player_count()` exists |
| Status change notifications | Custom pub/sub | `EventBus` broadcast channel | Already used for StatusChanged events |

**Key insight:** This phase is entirely about extending existing infrastructure patterns. The `auto_restart` field provides a complete template for how `auto_wake` should be implemented — migration, entity field, DTO field, use case wiring, UI toggle. No hand-rolling needed.

## Common Pitfalls

### Pitfall 1: Inactivity Timer Restarts on Every Zero-Player Tick
**What goes wrong:** Every 30s monitoring tick sees 0 players and resets the inactivity timer, so the server never actually sleeps.
**Why it happens:** The naive approach checks "are there players?" on each tick and starts counting from 0 every time.
**How to avoid:** Use a database field `last_player_activity` that is only UPDATED when players are present. Check elapsed time since that timestamp on zero-player ticks. Only update the field when `metrics.players > 0`, not when `metrics.players == 0`.
**Warning signs:** Server never sleeps despite prolonged zero-player periods.

### Pitfall 2: Race Condition Between Manual Stop and Auto-Sleep
**What goes wrong:** User manually stops a server, then the monitoring loop detects it as stopped-with-auto_wake and tries to wake it up.
**Why it happens:** Manual stop sets status="stopped" without setting auto_wake. Auto-sleep sets both. The monitoring loop needs to distinguish them.
**How to avoid:** Auto-restart should only trigger for sleeping servers (`auto_wake=true`) when the wake condition is met (player connection). Manual stop (`auto_wake=false`) must never auto-trigger. The monitoring loop checks `auto_wake && status=="stopped"` to decide.
**Warning signs:** Servers automatically restarting after user intentionally stops them.

### Pitfall 3: Restart Counter Never Resets
**What goes wrong:** After reaching max_restart_attempts, the server never tries to recover again.
**Why it happens:** The counter monotonically increments without a reset mechanism.
**How to avoid:** Reset `restart_count` to 0 when a server has been running successfully for >N minutes (e.g., 5 minutes of "running" status without crash). Add this check in the monitoring loop.
**Warning signs:** Server permanently stuck after one crash cycle.

### Pitfall 4: Exponential Backoff Eats Tokio Runtime
**What goes wrong:** Using `tokio::time::sleep()` inside the per-server monitoring loop blocks the entire loop for all servers.
**Why it happens:** The monitoring loop is sequential. A `sleep()` in the crash→restart path delays checking all other servers.
**How to avoid:** For backoff, use `tokio::spawn` for the wait-and-restart task rather than blocking the loop. The crash detection should spawn a background task, not sleep inline. Alternatively, track `next_allowed_restart_at` on the server and check it synchronously.
**Warning signs:** Monitoring loop takes longer than 30s, metrics become stale.

## Code Examples

### 1. Server Entity Extension (following auto_restart pattern)
**File:** `api/src/domain/entities/server.rs`
```rust
// Source: [VERIFIED: codebase pattern from existing auto_restart + auto_pause fields]
// Add alongside existing fields (~line 22-36):
pub auto_wake: bool,
pub sleep_timeout_minutes: i32,
pub max_restart_attempts: i32,
pub restart_cooldown_seconds: i32,
pub last_player_activity: Option<DateTime<Utc>>,
```

### 2. Sleep Handler Endpoint
**File:** `api/src/presentation/handlers/server_handlers.rs` (new function, following `stop_server` pattern at line 824)
```rust
// Source: [VERIFIED: codebase pattern from stop_server handler]
async fn sleep_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let mut server = repo.find_by_id(id)
        .await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    // (same access check pattern as stop_server)
    
    // Same agent-executor stop logic as stop_server...
    // But additionally: set auto_wake = true after stopping
    server.auto_wake = true;
    repo.update(&server).await.map_err(|e| e.to_string())?;
    
    Ok(Json(ApiResponse::success(serde_json::json!({ "status": "stopped", "auto_wake": true }))))
}
```

### 3. Wake Handler Endpoint
```rust
// Source: [VERIFIED: codebase pattern from start_server handler at line 670]
async fn wake_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let mut server = repo.find_by_id(id)
        .await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;
    
    // Only wake servers that were sleeping
    if !server.auto_wake {
        return Err("Server is not in sleep mode".to_string());
    }
    
    // Same start logic as start_server handler, but reset auto_wake
    // ...executor.start_server()...
    server.auto_wake = false;
    repo.update(&server).await.map_err(|e| e.to_string())?;
    
    Ok(Json(ApiResponse::success(serde_json::json!({ "status": "starting" }))))
}
```

### 4. Frontend: Settings Tab Sleep Config (following Discord webhook pattern)
```jsx
// Source: [VERIFIED: codebase pattern from ServerDetails.jsx settings tab ~line 329]
// In the settings tab section:

<section className="glass-panel p-6">
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

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| — | Auto-wake field (bool) on Server entity | Phase 56 | Distinguishes sleeping from stopped |
| Simple restart (immediate) | Backoff + max attempt + cooldown | Phase 56 | Prevents restart loops, saves resources |
| Only crash→restart | Crash→restart + inactivity→sleep + wake | Phase 56 | Full lifecycle automation |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | RCON player count from `collect_metrics` is reliable for inactivity detection | Architecture Patterns | If RCON is unreliable or slow, sleep detection may fire incorrectly; alternate approach would be agent-side container metrics |
| A2 | The existing `executor.stop_server()` works for servers using both agent and RCON executors | Sleep Handler | If agent executor stop doesn't support sleep semantics, wake may fail; verify agent stop preserves container state |
| A3 | Player connection auto-detection is deferred to API-based wake only | Wake-up Flow | If "auto" in the phase name implies fully automatic wake without user action, need port-probing mechanism |

## Common Pitfalls

### Pitfall 1: Inactivity Timer Restarts on Every Zero-Player Tick
**How to avoid:** Database-backed `last_player_activity` field. Update only when `players > 0`.

### Pitfall 2: Race Condition Between Manual Stop and Auto-Sleep
**How to avoid:** Check `auto_wake` field. Manual stop = auto_wake false. Sleep = auto_wake true.

### Pitfall 3: Restart Counter Never Resets
**How to avoid:** Reset `restart_count` to 0 after server runs successfully for >5 min.

### Pitfall 4: Exponential Backoff Eats Tokio Runtime
**How to avoid:** `tokio::spawn` for restart delay tasks, don't block the monitoring loop.

## Open Questions (RESOLVED)

1. **Wake-up trigger mechanism** — D-02 says "automatic wake-up mechanisms" but doesn't specify how. The simplest approach is API-based (user clicks or cron schedule), but auto-detection of a player attempting to connect requires port monitoring or proxy interception.
   - What we know: API-based wake is safe and straightforward
   - What's unclear: Whether the phase requires automatic player-detection wake (complex) or if API/manual wake is sufficient
   - Recommendation: Implement API-based wake only (`POST /servers/:id/wake`). Defer automatic player-detect wake to a future phase. Mark as `[ASSUMED]` for confirmation.

2. **Sleep timeout defaults** — The configurable timeout for inactivity detection needs sensible defaults.
   - What we know: Typical game server idle timeout ranges 15-60 minutes
   - What's unclear: User's preferred default
   - Recommendation: Default 30 minutes, configurable 5-240 minutes range. Mark as `[the agent's Discretion]` per D-01.

3. **Auto-restart max attempts** — The existing `restart_count` field tracks attempts but hasn't been used for backoff logic yet.
   - What we know: Code already has `restart_count: i32` field
   - What's unclear: What values for max attempts and cooldown
   - Recommendation: Default max 5 attempts, initial cooldown 30s (doubles each attempt to max 300s). Mark as `[the agent's Discretion]` per D-01.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — No new dependencies needed, all patterns verified in codebase
- Architecture: HIGH — Extends existing MonitoringService pattern; all changes follow established conventions
- Pitfalls: MEDIUM — Race conditions and timer edge cases identified from codebase analysis

**Research date:** 2026-05-30
**Valid until:** 2026-07-01 (stable stack, no fast-moving deps)
