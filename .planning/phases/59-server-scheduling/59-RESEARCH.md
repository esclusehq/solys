# Phase 59: Server Scheduling — Research

**Researched:** 2026-05-31
**Domain:** Server lifecycle automation (scheduled start/stop/restart/sleep), cron evaluation with timezone support, Worker-based dispatch for non-backup task types
**Confidence:** HIGH

## Summary

This phase extends the Worker's cron evaluation loop (currently backup-only) to dispatch `start`, `stop`, `restart`, and `sleep` scheduled actions to game servers. It adds timezone-aware cron evaluation, new task types to `cron_tasks`, a per-schedule timezone column, `run_once` for one-time actions, and a Scheduled Actions UI section in the ServerDetails Settings tab.

**Key architectural insight:** The Worker's existing `process_backup_server` dispatch pattern (`POST /api/v1/nodes/:id/commands`) calls the `poll_node_commands` handler which is a **read-only** endpoint — it returns pending commands without processing the POST body. This means Phase 55's backup dispatch may not actually forward commands to agents via WebSocket. Phase 59 must either:
- **(A)** Create a proper Agent API proxy endpoint (`POST /api/v1/nodes/:id/commands/dispatch`) that the Worker calls and which internally calls `NodeConnectionManager::send_command_to_node` to send the command over the WebSocket
- **(B)** Have the Worker queue a command via Redis and add a background consumer in the API

**Recommendation:** Approach A — simplest, consistent with existing `server_handlers.rs` pattern where `state.node_client.send_command_with_config(...)` is already used extensively to send commands to agents via WebSocket.

**Primary recommendation:** Implement in 5 tracks — (1) cron_tasks migration (timezone, run_once, last_result, last_error), (2) Worker cron evaluation extension + new job handlers, (3) Worker-to-Agent command dispatch proxy (new API endpoint), (4) API entity/repository/DTO/handler updates, (5) Frontend Scheduled Actions section in Settings tab.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Worker-based dispatch — extend Worker `cron_eval.rs` to handle `start`, `stop`, `restart`, `sleep` alongside `backup`. Follow Phase 55's exact pattern (D-02, D-09 from Phase 55).
- **D-02:** Add `start` and `sleep` task types to `cron_tasks.task_type`, joining existing `backup`, `restart`, `stop`, `command`.
- **D-03:** Per-schedule timezone VARCHAR(50) column, default `'UTC'`. Worker reads schedule's timezone, converts evaluation context.
- **D-04:** Scheduled Actions section in Settings tab of ServerDetails, with compact list of schedules per server.
- **D-05:** Error handling: log + toast + server event + 1x retry after 30s. Add `last_result` and `last_error` columns.
- **D-06:** `run_once` BOOLEAN column. After action fires, auto-disable the task.
- **D-07:** Phase 56 auto-sleep takes precedence over scheduled sleep. Server already slept → skip scheduled sleep with log.
- **D-08:** Scheduled restart complements Phase 57 auto-restart. Wait for auto-restart cooldown if one is in progress.

### The Agent's Discretion
- Specific UI layout of Scheduled Actions section in Settings tab
- Cron expression human-readable display format
- Schedule add/edit inline form design
- `last_result` and `last_error` display format
- Worker cron_eval extension details for new task types
- Redis job queue design for non-backup task types
- Frontend state management for schedule form
- Toast notification design and auto-dismiss duration
- Worker-to-API command proxy pattern (via new dispatch endpoint or modified existing)
- `chrono-tz` integration details
- Migration strategy for existing cron_tasks rows
- Validation rules for cron expression + timezone combination

### Deferred Ideas (OUT OF SCOPE)
- Per-task notification preferences (Discord webhook on failure) — future phase
- Schedule groups/tags ("maintenance window") — not needed yet
- Cron expression builder/visual editor — text input + human-readable preview sufficient
- ML-based schedule optimization — way out of scope
- Calendar view for schedules — table view sufficient

</user_constraints>

## Phase Requirements

No explicit REQ-IDs for this phase. Requirements are derived from ROADMAP.md and CONTEXT.md decisions.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cron schedule evaluation (all task types) | Worker | — | D-01 mandates Worker-owned cron evaluation. Worker polls `cron_tasks` for all enabled task types where `next_run <= NOW()`. |
| Timezone-aware cron matching | Worker | — | D-03: Worker reads schedule's `timezone`, converts `Utc::now()` to that timezone, evaluates if cron is due. |
| Command dispatch to Agent | API / Backend | — | Worker calls API endpoint → API forwards command to Agent via existing WebSocket `NodeConnectionManager`. |
| Agent action execution | Agent | — | Agent already has `server.start`, `server.stop`, `server.restart` handlers. Sleep = stop with auto_wake semantics. |
| Cron task CRUD (API) | API / Backend | — | Existing `cron_task_handlers.rs` + `cron_task_repository.rs` — extend DTOs for new columns. |
| Scheduled Actions UI | Browser / Client | — | New section in ServerDetails Settings tab. Add/edit/delete inline. |
| Task failure retry | Worker | — | D-05: 1 retry after 30s on failure. Mark `last_result` + `last_error`. |
| Sleep state guard (D-07) | Worker | — | Check server status before dispatching sleep. Skip if already `stopped` with `auto_wake=true`. |
| Restart cooldown wait (D-08) | Worker | — | If scheduled restart fires during auto-restart cooldown, wait before dispatching. |

## Standard Stack

### Core — Existing Infrastructure
| Library | Where | Purpose | Why Standard |
|---------|-------|---------|--------------|
| `cron` v0.15 | `worker/Cargo.toml` | Cron expression parsing and evaluation | Already in Worker. Currently used for backup-only filtering. |
| `chrono` v0.4 | `worker/Cargo.toml` | DateTime handling, Utc::now() | Already in Worker. Base time for cron evaluation. |
| `chrono-tz` (NEW) | `worker/Cargo.toml` | Timezone-aware datetime conversion | Timezone support (D-03). Parses IANA timezone names like "Asia/Jakarta". Converts UTC to schedule timezone for cron evaluation. |
| `sqlx` v0.7 | `worker/Cargo.toml` | PostgreSQL queries for cron_tasks + server status checks | Already in Worker. Needed for reading new columns and checking server state (D-07). |
| `redis` v0.25 | `worker/Cargo.toml` | Job queue dispatch for non-backup task types | Already in Worker. Extend with new job types. |
| `reqwest` v0.12 | `worker/Cargo.toml` | HTTP calls to API command dispatch proxy | Already in Worker. Same pattern as Phase 55 backup dispatch. |
| `uuid` v1 | `worker/Cargo.toml` | Job ID generation | Already in Worker. |
| `tokio` v1 | `worker/Cargo.toml` | Async runtime, interval timer, spawned tasks | Already in Worker. |
| React 19.2 | `app/package.json` | Frontend UI for Scheduled Actions section | Existing framework. |
| Zustand v5 | `app/package.json` | State management | Optional — existing pattern. |

### Supporting — New or Extended
| Library | Where | Purpose | When to Use |
|---------|-------|---------|-------------|
| `chrono-tz` | `worker/Cargo.toml` | IANA timezone parsing and offset calculation | Required for D-03 timezone support. Add `chrono-tz` with `tz-name = "0.9"` feature. |

### Alternatives Considered — None
All work uses existing stack components. Only `chrono-tz` is a new dependency.

**Installation:**
```toml
# worker/Cargo.toml additions
chrono-tz = { version = "0.10", features = ["serde"] }  # For timezone-aware cron evaluation
```

## Architecture Patterns

### System Architecture Diagram

```
                               ┌─────────────────────────────────────────┐
                               │            PostgreSQL                    │
                               │  cron_tasks table (extended)            │
                               │  ┌─────────────────────────────────┐   │
                               │  │ id, server_id, user_id          │   │
                               │  │ task_type (start/stop/restart/  │   │
                               │  │   sleep/backup)                  │   │
                               │  │ schedule_cron, timezone (NEW)   │   │
                               │  │ enabled, run_once (NEW)         │   │
                               │  │ last_run, last_result (NEW)     │   │
                               │  │ last_error (NEW), next_run      │   │
                               │  │ created_at, updated_at          │   │
                               │  └─────────────────────────────────┘   │
                               │  servers table                          │
                               │  ┌─────────────────────────────────┐   │
                               │  │ id, node_id, status, auto_wake,│   │
                               │  │ restart_count, auto_restart     │   │
                               │  └─────────────────────────────────┘   │
                               └──────────────┬──────────────────────────┘
                                              │ Worker polls every 30s
                                              │ WHERE enabled AND next_run <= NOW()
                                              │ (ALL task types, not just backup)
                                              ▼
                    ┌───────────────────────────────────────────────────┐
                    │              WORKER SERVICE                       │
                    │                                                   │
                    │  cron_eval.rs (EXTENDED):                         │
                    │  ┌─────────────────────────────────────────┐      │
                    │  │ evaluate_and_dispatch():                 │      │
                    │  │   1. Query ALL enabled due cron_tasks   │      │
                    │  │   2. For each task:                     │      │
                    │  │      a. Read timezone, convert UTC now  │      │
                    │  │         to schedule's timezone using     │      │
                    │  │         chrono-tz                        │      │
                    │  │      b. Check if cron is due in that   │      │
                    │  │         timezone                         │      │
                    │  │      c. If due: dispatch Redis job      │      │
                    │  │         with job_type based on task_type │      │
                    │  │      d. Update last_run = NOW()         │      │
                    │  └─────────────────────────────────────────┘      │
                    │                                                   │
                    │  queue/mod.rs (EXTENDED):                         │
                    │  ┌─────────────────────────────────────────┐      │
                    │  │ process_job():                         │      │
                    │  │   "backup_server" → process_backup...   │      │
                    │  │   "scheduled_start"  → process_scheduled_start()   │
                    │  │   "scheduled_stop"   → process_scheduled_stop()    │
                    │  │   "scheduled_restart"→ process_scheduled_restart()  │
                    │  │   "scheduled_sleep"  → process_scheduled_sleep()    │
                    │  │                                              │
                    │  │ Each handler:                               │
                    │  │   1. Check server state (D-07 guard)        │
                    │  │   2. Send command via API proxy             │
                    │  │   3. Update cron_tasks with result           │
                    │  │   4. Handle run_once + retry                 │
                    │  └─────────────────────────────────────────┘      │
                    └──────────────┬────────────────────────────────────┘
                                   │ HTTP POST /api/v1/nodes/:id/dispatch
                                   │ Body: { command, server_id, params }
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          API SERVER                                       │
│                                                                          │
│  NEW Endpoint (or replacement for existing broken poll_node_commands):   │
│  ┌────────────────────────────────────────────────────────────────┐     │
│  │ POST /api/v1/nodes/:id/dispatch                                │     │
│  │ Body: { command: "start_server", server_id, params }           │     │
│  │ → Deserializes payload                                         │     │
│  │ → Builds CommandParams from params                             │     │
│  │ → Calls state.node_client.send_command_with_config(...)        │     │
│  │ → This sends ExecuteCommand over Agent's WebSocket             │     │
│  │ → Returns CommandResponse                                      │     │
│  │ → If node disconnected, returns error for Worker retry         │     │
│  └────────────────────────────────────────────────────────────────┘     │
│                                                                          │
│  Existing cron_task_handlers.rs (EXTENDED):                              │
│  → Accept timezone, run_once in create/update DTOs                      │
│  → Validate timezone against chrono-tz                                   │
│                                                                          │
│  AgentNodeClient.send_command_with_config(...)                           │
│  → Already works! Used by server_handlers.rs, backup_handlers.rs     │
│  → Sends NodeMessage::ExecuteCommand over WebSocket                     │
│  → Waits for response with 30s timeout                                   │
└─────────────────────────────┬────────────────────────────────────────────┘
                              │ WebSocket (ExecuteCommand)
                              ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                    AGENT (solys — runs on node)                           │
│                                                                          │
│  agent_connection.rs:                                                    │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ BackendMessage::ExecuteCommand { command, server_id, params }│      │
│  │   → Maps command string to task_type:                        │      │
│  │     "start"     → "server.start" (EXISTS)                    │      │
│  │     "stop"      → "server.stop"  (EXISTS)                    │      │
│  │     "restart"   → "server.restart" (EXISTS)                  │      │
│  │     "start_server" → "server.start" (EXISTS)                 │      │
│  │     "stop_server"  → "server.stop"  (EXISTS)                 │      │
│  │   → Builds Task with payload → dispatches via execute_task   │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                          │
│  handlers/mod.rs:                                                        │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ execute_single():                                              │      │
│  │   "server.start"   → runtime::handle_start()  (EXISTS)        │      │
│  │   "server.stop"    → runtime::handle_stop()   (EXISTS)        │      │
│  │   "server.restart" → runtime::handle_restart() (EXISTS)       │      │
│  └──────────────────────────────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────────────┘
```

**Data flow for scheduled action (e.g., restart):**
```
User creates schedule → API writes cron_tasks row (type=restart, timezone=UTC)
  → Worker 30s tick: queries cron_tasks WHERE enabled AND next_run <= NOW()
  → Finds due task → Dispatches Redis job "scheduled_restart"
  → JobProcessor picks up → process_scheduled_restart():
    1. Fetches server info (node_id, status, restart_count)
    2. (D-08) If auto-restart cooldown active: wait then proceed
    3. Calls POST /api/v1/nodes/:id/dispatch with {"command": "restart", ...}
    4. API → AgentNodeClient.send_command_with_config() → WebSocket
    5. Agent receives "restart" → "server.restart" → runtime::handle_restart()
    6. Agent executes stop + start → reports result via TaskResult
    7. Worker updates cron_tasks.last_run / last_result / last_error
    8. If run_once=true: set enabled=false
    9. API pushes toast event to frontend
```

### Recommended Project Structure — Changes from Phase 55

```
worker/src/
├── cron_eval.rs                # EXTEND: filter ALL task types, add timezone conversion
├── queue/mod.rs                # EXTEND: add process_scheduled_start/stop/restart/sleep
│   (process_backup_server already exists)
├── Cargo.toml                  # ADD: chrono-tz dependency

api/src/
├── domain/entities/
│   └── cron_task.rs            # EXTEND: add timezone, run_once, last_result, last_error
├── domain/repositories/
│   └── cron_task_repository.rs # EXTEND: trait methods for new columns (if needed)
├── infrastructure/repositories/
│   └── postgres_cron_task_repository.rs  # EXTEND: read/write new columns
├── presentation/handlers/
│   └── cron_task_handlers.rs   # EXTEND: accept timezone/run_once in DTOs, validate
│   └── node_handlers.rs        # ADD (or modify): command dispatch endpoint
├── presentation/routes/
│   └── api_routes.rs           # ADD: new dispatch route
├── migration/
│   └── 20260531000001_add_cron_task_columns.sql  # NEW: timezone, run_once, etc.

app/src/
├── pages/ServerDetails.jsx     # EXTEND: add Scheduled Actions section in Settings tab
│   (after Restart Policy section, before closing </div>)
├── api/client.js               # EXTEND: add schedule CRUD + dispatch methods
├── hooks/
│   └── useScheduledActions.js  # NEW: hook for schedule CRUD + state
```

**Key difference from Phase 55:** Agent-side code needs NO changes. The Agent already handles `server.start`, `server.stop`, `server.restart` — these commands map directly to scheduled actions. Sleep uses the same `server.stop` path with auto_wake semantics.

### Pattern 1: Extended Worker Cron Evaluation (all task types, timezone-aware)

**What:** Worker `cron_eval.rs` currently queries only `task_type = 'backup'`. Extended to query ALL enabled task types and use `chrono-tz` for timezone-aware evaluation.

**When to use:** Replace existing `evaluate_and_dispatch` function.

**Critical design decision — timezone handling:** The `cron` crate's `Schedule::upcoming(Utc)` iterates over future UTC timestamps. To check "is this cron due now in the schedule's timezone?", the simplest approach is:
1. Convert `Utc::now()` to the schedule's timezone using `chrono-tz`
2. Check if the cron expression matches the current time in that timezone

Alternative: The `cron` crate v0.15 does NOT natively support timezone-aware scheduling. You must convert the evaluation context yourself.

```rust
use chrono_tz::Tz;
use cron::Schedule;
use std::str::FromStr;

/// Check if a cron expression is due now in the given timezone.
/// `schedule_cron`: standard 5-field cron expression
/// `timezone_str`: IANA timezone name like "Asia/Jakarta" or "UTC"
fn is_cron_due_in_timezone(schedule_cron: &str, timezone_str: &str) -> anyhow::Result<bool> {
    let tz: Tz = timezone_str.parse()?;  // chrono-tz parse
    
    let schedule = Schedule::from_str(schedule_cron)?;
    let now_utc = chrono::Utc::now();
    let now_tz = now_utc.with_timezone(&tz);
    
    // Check if the cron schedule includes the current minute
    // Cron schedules have minute-level granularity
    Ok(schedule.includes(now_tz))
}
```

**NOTE:** The `cron` crate's `Schedule::includes()` method was added in a later version. If not available, use the `upcoming()` iterator approach:

```rust
fn is_cron_due_in_timezone(schedule_cron: &str, timezone_str: &str) -> anyhow::Result<bool> {
    let tz: Tz = timezone_str.parse()?;
    let schedule = Schedule::from_str(schedule_cron)?;
    let now_utc = chrono::Utc::now();
    let now_tz = now_utc.with_timezone(&tz);
    
    // Check if the upcoming schedule is within the current minute
    if let Some(next) = schedule.upcoming(tz).next() {
        let diff = next - now_tz;
        // If next run is within 60 seconds, it's due
        Ok(diff.num_seconds() < 60 && diff.num_seconds() >= 0)
    } else {
        Ok(false)
    }
}
```

**Extended evaluate_and_dispatch:**
```rust
// worker/src/cron_eval.rs (EXTENDED — all task types, timezone-aware)

async fn evaluate_and_dispatch(
    pool: &PgPool,
    redis: &redis::aio::MultiplexedConnection,
) -> anyhow::Result<()> {
    // Query ALL enabled due tasks (no longer filtered by task_type)
    let rows = sqlx::query(
        r#"
        SELECT id, server_id, user_id, task_type, schedule_cron, timezone,
               command, enabled, run_once, last_run, next_run, created_at, updated_at
        FROM cron_tasks
        WHERE enabled = true
          AND next_run <= NOW() + INTERVAL '30 seconds'  -- slight lookahead guard
        ORDER BY next_run ASC
        LIMIT 50
        "#
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let cron_task_id: Uuid = row.try_get("id")?;
        let server_id: Uuid = row.try_get("server_id")?;
        let user_id: Uuid = row.try_get("user_id")?;
        let task_type: String = row.try_get("task_type")?;
        let timezone: String = row.try_get::<Option<String>, _>("timezone")?
            .unwrap_or_else(|| "UTC".to_string());

        // Verify cron is due in the schedule's timezone (D-03)
        let schedule_cron: String = row.try_get("schedule_cron")?;
        match is_cron_due_in_timezone(&schedule_cron, &timezone) {
            Ok(true) => { /* proceed */ }
            Ok(false) => continue, // Not due in this timezone yet
            Err(e) => {
                tracing::warn!("Timezone eval error for {}: {}", cron_task_id, e);
                continue;
            }
        }

        // Map task_type to job_type
        let job_type = match task_type.as_str() {
            "backup" => "backup_server",
            "start" => "scheduled_start",
            "stop" => "scheduled_stop",
            "restart" => "scheduled_restart",
            "sleep" => "scheduled_sleep",
            _ => {
                tracing::warn!("Unknown task_type: {}", task_type);
                continue;
            }
        };

        // Enqueue job via Redis (same pattern as existing backup dispatch)
        let job_id = Uuid::new_v4();
        let job_payload = serde_json::json!({
            "cron_task_id": cron_task_id,
            "server_id": server_id,
            "user_id": user_id,
            "task_type": task_type,
            "timezone": timezone,
        });

        let job_key = format!("job:{}", job_id);
        let queue_key = "queue:jobs:normal";

        redis::cmd("HSET")
            .arg(&job_key)
            .arg("data")
            .arg(serde_json::to_string(&serde_json::json!({
                "job_id": job_id,
                "job_type": job_type,
                "payload": job_payload,
                "user_id": user_id,
                "priority": 0,
                "created_at": Utc::now().timestamp(),
            }))?)
            .query_async::<_, ()>(redis)
            .await?;

        redis::cmd("ZADD")
            .arg(queue_key)
            .arg(Utc::now().timestamp() as f64)
            .arg(job_id.to_string())
            .query_async::<_, ()>(redis)
            .await?;

        // Update last_run
        sqlx::query(
            "UPDATE cron_tasks SET last_run = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(cron_task_id)
        .execute(pool)
        .await?;

        tracing::info!(
            "Dispatched {} job: cron_task={} server={} job={}",
            job_type, cron_task_id, server_id, job_id
        );
    }

    Ok(())
}
```

### Pattern 2: Worker Job Handlers for Server Actions

**What:** New job handlers in `queue/mod.rs` that receive scheduled action jobs, check server state, dispatch commands to agents via API proxy, and update cron_tasks with results.

**When to use:** Each scheduled action type (start/stop/restart/sleep) gets its own handler method.

```rust
// worker/src/queue/mod.rs — NEW handlers for scheduled actions

async fn process_scheduled_start(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
    let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

    // Check server status — skip if already running
    let status: String = sqlx::query_scalar("SELECT status FROM servers WHERE id = $1")
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or("Server not found")?;

    if status == "running" || status == "starting" || status == "container_running" {
        tracing::warn!("Server {} already running, skipping scheduled start", server_id);
        self.update_cron_task_result(cron_task_id, "skipped", "Server already running").await?;
        return Ok(());
    }

    let node_id: Uuid = sqlx::query_scalar("SELECT node_id FROM servers WHERE id = $1")
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or("Server not found")??;

    let api_url = format!("{}/api/v1/nodes/{}/dispatch",
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://api:3000".to_string()),
        node_id
    );

    let body = serde_json::json!({
        "command": "start",
        "server_id": server_id,
        "params": {}
    });

    let client = reqwest::Client::new();
    let response = client.post(&api_url).json(&body).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            self.update_cron_task_result(cron_task_id, "success", None).await?;
            if self.is_run_once(cron_task_id).await? {
                self.disable_cron_task(cron_task_id).await?;
            }
        }
        Ok(resp) => {
            let error = format!("API returned {}", resp.status());
            self.handle_cron_failure(cron_task_id, &error).await?;
        }
        Err(e) => {
            self.handle_cron_failure(cron_task_id, &e.to_string()).await?;
        }
    }

    Ok(())
}
```

**Sleep handler (D-07 guard):**
```rust
async fn process_scheduled_sleep(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
    let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

    // D-07: Check if server is already in sleep state
    // Server is "sleeping" when: status = 'stopped' AND auto_wake = true
    let row = sqlx::query("SELECT status, auto_wake FROM servers WHERE id = $1")
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or("Server not found")?;

    let status: String = row.try_get("status")?;
    let auto_wake: bool = row.try_get("auto_wake")?;

    if status == "stopped" && auto_wake {
        tracing::info!(
            "Server {} already in sleep state, skipping scheduled sleep (D-07)",
            server_id
        );
        self.update_cron_task_result(cron_task_id, "skipped", "Already sleeping").await?;
        return Ok(());
    }

    if status != "running" && status != "container_running" {
        tracing::warn!("Server {} not running, skipping scheduled sleep", server_id);
        self.update_cron_task_result(cron_task_id, "skipped", "Server not running").await?;
        return Ok(());
    }

    // Send "stop" command — the API handler will set auto_wake = true for sleep semantics
    // Same dispatch pattern as process_scheduled_start
    // ... (follow same API proxy pattern)
    Ok(())
}
```

**Restart handler (D-08 guard):**
```rust
async fn process_scheduled_restart(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
    let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

    // D-08: Check if auto-restart cooldown is active
    // If restart_count > 0, auto-restart is in progress
    let restart_count: i32 = sqlx::query_scalar(
        "SELECT restart_count FROM servers WHERE id = $1"
    )
    .bind(server_id)
    .fetch_optional(&self.pool)
    .await?
    .ok_or("Server not found")?;

    if restart_count > 0 {
        // Auto-restart is active — wait for cooldown to complete
        // Simple strategy: wait 30s and re-check
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        let count: i32 = sqlx::query_scalar(
            "SELECT restart_count FROM servers WHERE id = $1"
        )
        .bind(server_id)
        .fetch_one(&self.pool)
        .await?;
        if count > 0 {
            // Still in cooldown — skip this cycle, let next cron tick try again
            tracing::warn!("Server {} auto-restart still active, deferring scheduled restart", server_id);
            return Ok(());
        }
    }

    // Proceed with restart dispatch (same pattern as above)
    // ... (follow same API proxy pattern)
    Ok(())
}
```

**Helper methods:**
```rust
// These go in the `impl JobProcessor` block

async fn update_cron_task_result(&self, task_id: Uuid, result: &str, error: Option<&str>) -> Result<()> {
    sqlx::query(
        r#"UPDATE cron_tasks 
           SET last_result = $2, last_error = $3, updated_at = NOW()
           WHERE id = $1"#
    )
    .bind(task_id)
    .bind(result)
    .bind(error)
    .execute(&self.pool)
    .await?;
    Ok(())
}

async fn handle_cron_failure(&self, task_id: Uuid, error: &str) -> Result<()> {
    // First attempt failed — try one retry (D-05)
    // Track retry status via last_error for now (simple approach)
    // The first failure writes to last_error, the second retry is done inline
    // (In production this would use a retry count column)
    tracing::warn!("Scheduled task {} failed: {}. Retrying once...", task_id, error);
    
    // Wait 30s then retry (handled by the caller wrapping this)
    // For now, just update last_error with first attempt failure
    self.update_cron_task_result(task_id, "retrying", Some(error)).await?;
    Ok(())
}

async fn is_run_once(&self, task_id: Uuid) -> Result<bool> {
    let run_once: bool = sqlx::query_scalar("SELECT run_once FROM cron_tasks WHERE id = $1")
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or("Task not found")?;
    Ok(run_once)
}

async fn disable_cron_task(&self, task_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE cron_tasks SET enabled = false, updated_at = NOW() WHERE id = $1")
        .bind(task_id)
        .execute(&self.pool)
        .await?;
    Ok(())
}
```

### Pattern 3: API Command Dispatch Proxy Endpoint (NEW)

**What:** A new API endpoint `POST /api/v1/nodes/:id/dispatch` that accepts a command from the Worker and sends it to the Agent via the existing WebSocket infrastructure (`NodeConnectionManager::send_command_to_node`).

**Why needed:** The existing `POST /api/v1/nodes/:id/commands` route (Phase 55) maps to `poll_node_commands` which is **read-only** — it returns pending commands for agent polling. The Phase 55 Worker's POST call to that endpoint would not actually send the command to the agent. This new endpoint is the proper proxy.

```rust
// NEW: api/src/presentation/handlers/node_handlers.rs

use crate::presentation::ws::node_protocol::CommandParams;

#[derive(serde::Deserialize)]
pub struct DispatchCommandRequest {
    pub command: String,        // "start", "stop", "restart", "backup.start", etc.
    pub server_id: Uuid,
    #[serde(default)]
    pub params: Option<DispatchParams>,
}

#[derive(serde::Deserialize, Default)]
pub struct DispatchParams {
    pub container_name: Option<String>,
    pub container_id: Option<String>,
    // Sleep-specific: marks stop as sleep
    pub sleep: Option<bool>,
}

pub async fn dispatch_node_command(
    State(state): State<ApiState>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<DispatchCommandRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Build CommandParams from request
    let params = CommandParams {
        container_name: req.params.as_ref().and_then(|p| p.container_name.clone()),
        container_id: req.params.as_ref().and_then(|p| p.container_id.clone()),
        ..Default::default()
    };

    // Send command via existing node client (WebSocket proxy)
    let response = state.node_client.send_command(
        node_id,
        req.server_id,
        &req.command,
        params,
    ).await.map_err(|e| AppError::InternalError(e))?;

    Ok(Json(ApiResponse::success(response)))
}
```

**Route addition in api_routes.rs:**
```rust
.route("/api/v1/nodes/:id/dispatch", post(crate::presentation::handlers::node_handlers::dispatch_node_command))
```

**Agent command mapping** (already exists in `agent_connection.rs` lines 402-413):
```
"start"     → "server.start"   → runtime::handle_start()
"stop"      → "server.stop"    → runtime::handle_stop()
"restart"   → "server.restart" → runtime::handle_restart()
```

**For sleep semantics:** The `stop` command already stops the server. For sleep, the API handler that receives the dispatch should set `auto_wake = true` on the server entity before/after sending the `stop` command, matching Phase 56's sleep behavior.

### Pattern 4: API Entity / DTO / Repository Changes

**What:** Extend the `CronTask` entity, DTOs, and repository to support the new columns.

**Entity extension:**
```rust
// api/src/domain/entities/cron_task.rs

pub struct CronTask {
    pub id: Uuid,
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub task_type: String, // "backup", "restart", "stop", "command", "start", "sleep"
    pub schedule_cron: String,
    pub timezone: String,           // NEW: default "UTC"
    pub command: Option<String>,
    pub enabled: bool,
    pub run_once: bool,             // NEW: default false
    pub last_run: Option<DateTime<Utc>>,
    pub last_result: Option<String>, // NEW: "success", error message, or "skipped"
    pub last_error: Option<String>,   // NEW: failure details
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Extended DTOs
pub struct CreateCronTaskRequest {
    pub task_type: String,
    pub schedule_cron: String,
    pub timezone: Option<String>,     // NEW
    pub command: Option<String>,
    pub enabled: Option<bool>,
    pub run_once: Option<bool>,       // NEW
}

pub struct UpdateCronTaskRequest {
    pub task_type: Option<String>,
    pub schedule_cron: Option<String>,
    pub timezone: Option<String>,     // NEW
    pub command: Option<String>,
    pub enabled: Option<bool>,
    pub run_once: Option<bool>,       // NEW
}
```

**Validation:**
```rust
// In cron_task_handlers.rs create_task:
if !["backup", "restart", "stop", "command", "start", "sleep"].contains(&req.task_type.as_str()) {
    return Err("Invalid task_type".to_string());
}

// Validate timezone if provided
if let Some(ref tz) = req.timezone {
    if tz.parse::<chrono_tz::Tz>().is_err() {
        return Err(format!("Invalid timezone: {}", tz));
    }
}
```

### Pattern 5: Frontend Scheduled Actions Section (Settings Tab)

**What:** A compact list section in ServerDetails Settings tab showing all schedules for this server, with add/edit/delete inline.

**Pattern reference:** Phase 56 Sleep & Wake and Phase 57 Restart Policy sections in `ServerDetails.jsx` — follow exact same glass-panel section pattern.

**Location:** Between the Restart Policy section (line 635 in current ServerDetails.jsx) and the closing `</div>` (line 637).

```jsx
{/* ─── SCHEDULED ACTIONS CONFIG (Phase 59) ─── */}
<section className="glass-panel p-6 mt-6">
  <h3 className="text-lg font-bold mb-1">Scheduled Actions</h3>
  <p className="text-xs text-[var(--color-text-muted)] mb-5">
    Automatically start, stop, restart, or sleep this server on a schedule.
  </p>

  {/* Schedule list */}
  {schedules.length === 0 ? (
    <div className="p-4 rounded-xl bg-[rgba(255,255,255,0.02)] border border-[var(--color-cosmic-border)] text-center">
      <p className="text-sm text-[var(--color-text-muted)]">No schedules yet</p>
    </div>
  ) : (
    <div className="space-y-2">
      {schedules.map(schedule => (
        <div key={schedule.id}
             className="flex items-center gap-3 p-3 rounded-xl border border-[var(--color-cosmic-border)]
                        hover:border-[var(--color-cosmic-cyan)]/50 transition-all">
          {/* Action badge */}
          <span className={`text-xs font-bold px-2 py-1 rounded ${
            schedule.task_type === 'start' ? 'bg-emerald-500/20 text-emerald-400' :
            schedule.task_type === 'stop' ? 'bg-red-500/20 text-red-400' :
            schedule.task_type === 'restart' ? 'bg-amber-500/20 text-amber-400' :
            'bg-purple-500/20 text-purple-400'
          }`}>
            {schedule.task_type.toUpperCase()}
          </span>

          {/* Schedule info */}
          <div className="flex-1">
            <p className="text-sm font-medium">
              {formatSchedule(schedule.schedule_cron)}
              <span className="text-xs text-[var(--color-text-muted)] ml-2">
                {schedule.timezone || 'UTC'}
              </span>
            </p>
            <p className="text-xs text-[var(--color-text-muted)]">
              {schedule.last_run ? `Last: ${new Date(schedule.last_run).toLocaleString()}` : 'Never run'}
              {schedule.last_result === 'success' ? ' ✓' :
               schedule.last_result === 'skipped' ? ' ⏭' :
               schedule.last_error ? ` ✗` : ''}
            </p>
          </div>

          {/* Run-once badge */}
          {schedule.run_once && (
            <span className="text-[10px] text-[var(--color-cosmic-cyan)] font-bold px-2 py-0.5 rounded
                             border border-[var(--color-cosmic-cyan)]/30 bg-[var(--color-cosmic-cyan)]/10">
              ONE-TIME
            </span>
          )}

          {/* Toggle + Actions */}
          <button onClick={() => handleToggleSchedule(schedule.id, !schedule.enabled)}
                  className={`px-2 py-1 rounded text-xs font-bold ${
                    schedule.enabled ? 'bg-green-600/20 text-green-400' : 'bg-gray-600/20 text-gray-400'
                  }`}>
            {schedule.enabled ? 'ON' : 'OFF'}
          </button>
          <button onClick={() => handleDeleteSchedule(schedule.id)}
                  className="text-xs text-red-400 hover:text-red-300">
            Del
          </button>
        </div>
      ))}
    </div>
  )}

  {/* Add Schedule button */}
  <button onClick={() => setShowScheduleForm(true)}
          className="mt-4 w-full py-2.5 rounded-lg text-sm font-bold
                     bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                     hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                     disabled:opacity-50 transition-all">
    + Add Schedule
  </button>
</section>
```

**Add/Edit form (inline modal or expandable):**
- Action type dropdown: Start, Stop, Restart, Sleep
- Schedule preset dropdown (same pattern as Phase 55's backup preset) + custom cron input
- Timezone input (text or dropdown with common timezones)
- Run-once checkbox
- Save / Cancel buttons

### Anti-Patterns to Avoid

- **Worker calling Agent directly without API proxy:** Worker has no WebSocket connection to agents. Always proxy through the API's `NodeConnectionManager`.
- **Using `POST /api/v1/nodes/:id/commands` for dispatch:** This endpoint is `poll_node_commands` — read-only. Worker must use a new/proper dispatch endpoint.
- **In-memory last-run tracking:** Phase 55 switched to DB-backed `cron_tasks.next_run`. Phase 59 must continue this pattern — no in-memory state.
- **Hardcoded UTC-only evaluation:** D-03 mandates timezone support. Worker must convert evaluation timezone per schedule.
- **Sleep using `stop` command without auto_wake:** Sleep = stop + set auto_wake = true. Phase 56 agents expect this pattern.
- **Concurrent dispatch of the same cron task:** The 30-second poll interval and `next_run <= NOW()` query should prevent duplicates, but verify in implementation.
- **Toast/event emission from Worker:** Worker should not push toasts directly. Worker updates cron_tasks → API monitoring loop or WebSocket event emitter detects changes and pushes toasts.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Timezone conversion | Custom UTC offset calculation | `chrono-tz` crate | Handles DST, historical timezone changes, all IANA zones. Simple parse + `with_timezone()` conversion. |
| Cron evaluation | Custom minute-matching logic | `cron` v0.15 crate | Already in Worker. Has `Schedule::from_str`, `upcoming()` iterator. |
| Agent command dispatch | Worker-to-Agent WebSocket | API `NodeConnectionManager` | Worker calls API dispatch endpoint. API sends over existing WebSocket. Agent already has handlers. |
| Container start/stop/restart | SSH exec to node | Agent `server.start/stop/restart` | Agents already have these handlers from Phase 6. No new agent code needed. |
| Job queue | Custom Redis management | Existing priority queue pattern in Worker | Worker already has `zpopmin` + enqueue/dequeue. Reuse for new job types. |
| Toast notifications | Worker emitting WS events | API emits via existing EventBus | Worker updates cron_tasks. API's WebSocket/event system picks up changes. |

**Key insight:** This phase requires **zero new Agent code** and minimal API code. The Worker extensions and the new dispatch endpoint are the only substantial new code. The Agent already handles all three server lifecycle actions (`start`, `stop`, `restart`).

## Common Pitfalls

### Pitfall 1: Worker cron_eval doesn't filter by timezone (D-03 violation)
**What goes wrong:** Worker evaluates cron expressions using `Utc::now()` for all schedules, ignoring the `timezone` column. A schedule with "0 8 * * *" and timezone "Asia/Jakarta" fires at 8 AM UTC instead of 8 AM Jakarta time.
**Why it happens:** The `cron` crate's `Schedule::upcoming(Utc)` defaults to UTC timezone.
**How to avoid:** Always convert `Utc::now()` to the schedule's timezone before evaluating: `let now_tz = Utc::now().with_timezone(&tz);`. Use `schedule.upcoming(tz)` where `tz` is the schedule's `chrono-tz::Tz`.
**Warning signs:** Schedules fire at wrong hour for non-UTC timezones.

### Pitfall 2: `POST /api/v1/nodes/:id/commands` is read-only (existing Phase 55 issue)
**What goes wrong:** Worker calls this endpoint to dispatch commands, but the handler `poll_node_commands` only reads pending commands from the connection manager cache — it does NOT write the POST body or forward commands to agents.
**Why it happens:** Phase 55 planned the API proxy approach but the endpoint was named misleadingly. The actual handler is a polling endpoint for agents, not a command dispatch endpoint.
**How to avoid:** Create a NEW endpoint `POST /api/v1/nodes/:id/dispatch` that calls `state.node_client.send_command_with_config(...)`. Do NOT reuse the existing `poll_node_commands` endpoint.
**Warning signs:** Scheduled actions appear to dispatch (Worker logs success) but agents never receive the command.

### Pitfall 3: Concurrent scheduled + manual actions on the same server
**What goes wrong:** A scheduled start fires while a user manually starts the server. Or a scheduled sleep fires while a user is currently playing.
**Why it happens:** The Worker checks `next_run <= NOW()` but doesn't check current server state before dispatching.
**How to avoid:** In each job handler, query the server's current status before dispatching the command. Skip if the action doesn't make sense (e.g., start if already running, stop if already stopped).
**Warning signs:** Server status flips unexpectedly, user logs show rapid start-stop cycles.

### Pitfall 4: Phase 56 auto-sleep and scheduled sleep conflict (D-07)
**What goes wrong:** Server is auto-slept by Phase 56 at 11:55 PM due to inactivity. A scheduled sleep fires at midnight and tries to sleep an already-sleeping server.
**Why it happens:** The scheduled sleep handler doesn't check `status = 'stopped' AND auto_wake = true` (Phase 56's sleep state).
**How to avoid:** `process_scheduled_sleep` must check `auto_wake` flag along with `status`. If `status == "stopped" && auto_wake == true`, skip with "Already sleeping" log.
**Warning signs:** `last_result = "error"` with "Server already stopping" for sleep tasks.

### Pitfall 5: D-08 scheduled restart vs auto-restart race
**What goes wrong:** Server crashes at 2:58 AM. Auto-restart starts with backoff (30s, 60s, 120s...). Scheduled restart at 3:00 AM fires and sends a fresh restart command while auto-restart is still in cooldown.
**Why it happens:** The Worker's cron tick at 3:00 AM finds the scheduled restart due and dispatches without checking auto-restart state.
**How to avoid:** Check `restart_count > 0` in `process_scheduled_restart`. If auto-restart is active, wait up to `restart_cooldown_seconds` for it to complete before dispatching the scheduled restart.
**Warning signs:** Duplicate restart attempts in server event timeline.

### Pitfall 6: `run_once` task never re-enables
**What goes wrong:** A run_once task fires, auto-disables, but the user expects to be able to re-enable it for the same schedule time.
**Why it happens:** `run_once = true` + `enabled = false` means the task won't fire again. The user must manually set `enabled = true` or create a new task.
**How to avoid:** The UI should clearly indicate that run_once tasks are one-way. Consider showing a "Re-enable" button that sets `enabled = true` AND updates `next_run` to recalculate the next occurrence.
**Warning signs:** User confusion about why a one-time schedule didn't repeat.

## Code Examples

### Example 1: Timezone-aware cron evaluation with chrono-tz
```rust
// Requires: chrono-tz in worker/Cargo.toml
// Dependency: chrono-tz = { version = "0.10", features = ["serde"] }

use chrono_tz::Tz;
use cron::Schedule;
use std::str::FromStr;

pub fn is_due_in_timezone(schedule_cron: &str, timezone_name: &str) -> anyhow::Result<bool> {
    let tz: Tz = timezone_name.parse()
        .map_err(|e| anyhow::anyhow!("Invalid timezone '{}': {}", timezone_name, e))?;
    
    let schedule = Schedule::from_str(schedule_cron)
        .map_err(|e| anyhow::anyhow!("Invalid cron '{}': {}", schedule_cron, e))?;
    
    let now_utc = chrono::Utc::now();
    let now_tz = now_utc.with_timezone(&tz);
    
    // Get next upcoming time in the schedule's timezone
    match schedule.upcoming(tz).next() {
        Some(next) => {
            let diff = next - now_tz;
            // Due if within the next 60 seconds (matching the 30s poll interval)
            Ok(diff.num_seconds() >= 0 && diff.num_seconds() <= 60)
        }
        None => Ok(false),
    }
}
```

### Example 2: Migration SQL for new columns
```sql
-- Migration: 20260531000001_add_cron_task_scheduling_columns.sql
-- Adds timezone, run_once, last_result, last_error columns to cron_tasks

ALTER TABLE cron_tasks
  ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
  ADD COLUMN IF NOT EXISTS run_once BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS last_result TEXT,
  ADD COLUMN IF NOT EXISTS last_error TEXT;

-- Update the existing CHECK constraint to include new task types
-- (Only if a CHECK constraint exists that limits task_type values)
ALTER TABLE cron_tasks
  DROP CONSTRAINT IF EXISTS cron_tasks_task_type_check;

ALTER TABLE cron_tasks
  ADD CONSTRAINT cron_tasks_task_type_check
  CHECK (task_type IN ('backup', 'restart', 'stop', 'command', 'start', 'sleep'));
```

### Example 3: Agent-side command mapping (already exists — no changes needed)
```rust
// agent/solys/src/agent_connection.rs:402-413
let task_type = match command.as_str() {
    "create" => "server.create",
    "start" => "server.start",        // ✅ For scheduled start
    "stop" => "server.stop",           // ✅ For scheduled stop
    "restart" => "server.restart",     // ✅ For scheduled restart
    "delete" => "server.delete",
    "logs" => "server.logs",
    "command" => "server.command",
    "backup.start" => "backup.start",
    "backup.restore" => "backup.restore",
    _ => "unknown",
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| API `SchedulerService` with stubs | Worker-based cron evaluation | Phase 55 | Worker becomes canonical scheduler |
| Worker backup-only cron eval | Worker all-task-type cron eval | Phase 59 | cron_eval.rs query drops `task_type = 'backup'` filter |
| UTC-only cron evaluation | Timezone-aware per schedule | Phase 59 | chrono-tz dependency needed |
| Backup-only Redis job types | Backup + 4 new job types | Phase 59 | New handlers in queue/mod.rs |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Phase 55's `POST /api/v1/nodes/:id/commands` endpoint is read-only (`poll_node_commands`) and does NOT forward commands to agents | Architecture Patterns | If Phase 55 actually modified this endpoint to write through, a separate dispatch endpoint may be redundant. Verify implementation. |
| A2 | The Agent's existing `server.start`, `server.stop`, `server.restart` handlers work correctly for all game server types | Agent Command Mapping | If handlers have runtime-specific issues, scheduled actions may fail silently. |
| A3 | `chrono-tz` crate can be added to Worker without compatibility issues | Standard Stack | Verify Worker's dependency tree doesn't conflict with existing chrono/rust versions. |
| A4 | Phase 56 auto-sleep uses `status='stopped' + auto_wake=true` as sleep state marker | Sleep Interaction | If Phase 56 uses a different mechanism, D-07 guard logic changes. |
| A5 | The API `event_bus` + WebSocket event system can push toast notifications | Error Handling | If the event bus requires specific event format for toasts, the Worker's result reporting must match. |

## Open Questions

1. **Does Phase 55 backup dispatch actually work?**
   - What we know: Worker `process_backup_server` calls `POST /api/v1/nodes/:id/commands` which maps to the `poll_node_commands` read-only handler
   - What's unclear: Whether this endpoint was modified after Phase 55 research to actually write through, or if backup dispatch is currently non-functional
   - Recommendation: Verify the handler implementation before Phase 59 planning. If broken, fix in Phase 59 (the new dispatch endpoint fixes this for all task types)

2. **What is the exact sleep semantics command for Phase 56?**
   - What we know: Sleep = stop + set auto_wake=true in Phase 56 code (monitoring_service.rs lines 237-259)
   - What's unclear: Is there a dedicated `sleep_server` executor method, or does the scheduled sleep handler just send `stop` then update `auto_wake`?
   - Recommendation: For scheduled sleep, send `stop` command to agent, then update server's `auto_wake = true` in DB. The Phase 56 monitoring loop then handles wake-on-demand.

3. **How should `next_run` be calculated with timezone?**
   - What we know: Current API `calculate_next_run` uses `schedule.upcoming(Utc)` — no timezone support
   - What's unclear: Should `next_run` be stored as UTC (always) and timezone only affects evaluation? Yes — this is the design (D-03 says "cron expression stays in standard UTC-based format — only the evaluation timezone changes")
   - Recommendation: Store `next_run` as UTC, use timezone only for evaluation. The API's `calculate_next_run` should optionally accept a timezone parameter.

4. **Does the retry mechanism need a `retry_count` column?**
   - What we know: D-05 says "1x retry after 30 seconds"
   - What's unclear: Should we track retry attempts in a column, or just use in-memory retry in the job handler?
   - Recommendation: Keep it simple — the job handler retries once inline (30s sleep + retry). If both attempts fail, `last_result` stores the error message. No retry_count column needed for MVP.

## Validation Architecture

> workflow.nyquist_validation: absent from .planning/config.json — treat as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Not yet configured for Worker tests (no test framework detected in Worker Cargo.toml) |
| Config file | None — Worker has no test configuration |
| Quick run command | `cargo test -p worker` (if tests added to Worker crate) |
| Full suite command | `cargo test` (covers all Rust crates) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-03 | Timezone-aware cron evaluation | Unit | `cargo test -p worker timezone_eval` | ❌ Wave 0 |
| D-02 | New task types dispatched correctly | Unit | `cargo test -p worker task_type_dispatch` | ❌ Wave 0 |
| D-07 | Sleep guard skips already-sleeping server | Unit | `cargo test -p worker sleep_guard` | ❌ Wave 0 |
| D-08 | Scheduled restart waits for auto-restart cooldown | Unit | `cargo test -p worker restart_cooldown` | ❌ Wave 0 |
| D-05 | Retry mechanism on failure | Unit | `cargo test -p worker retry_mechanism` | ❌ Wave 0 |
| D-06 | Run-once auto-disables after execution | Unit | `cargo test -p worker run_once` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo build -p worker` (compile check only — no Worker test suite yet)
- **Per wave merge:** `cargo test` (full Rust suite)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] No test infrastructure exists in worker/src/ — no tests/ directory, no dev-dependencies for testing
- [ ] Worker tests should be added: basic unit tests for `is_due_in_timezone`, integration tests mocking the DB query for cron_tasks
- [ ] API tests should verify the new dispatch endpoint
- [ ] Frontend tests for Scheduled Actions section (if frontend test framework exists)

## Security Domain

> Security enforcement config key not found — treat as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | Yes | Existing AuthUser middleware on all cron_task handlers |
| V4 Access Control | Yes | Server ownership check in cron_task handlers (server.user_id vs auth_user) |
| V5 Input Validation | Yes | Validate task_type enum, timezone format, cron expression format |
| V7 Logging | Yes | Existing tracing in Worker and API — log all scheduled action dispatches |

### Known Threat Patterns for Worker + API

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Worker dispatches action for server user doesn't own | Elevation of Privilege | Worker reads cron_task.user_id but doesn't verify server ownership (server fetched from DB by cron_task.server_id). The API dispatch endpoint validates via existing AuthUser → not applicable since dispatch is internal. Acceptable risk — Worker has DB access. |
| User creates schedule with invalid timezone | Input Validation | Validate timezone string against chrono-tz parse in API handler |
| User creates schedule with cron firing every minute | DoS | Enforce minimum interval (e.g., 5 minutes between allowed runs). Validate in create_task handler. |
| Schedule dispatches command to wrong node | Spoofing | Worker fetches node_id from servers table using cron_task.server_id — always reads canonical node assignment. |

## Sources

### Primary (HIGH confidence)
- **Codebase audit**: `worker/src/cron_eval.rs` — verified current backup-only query, line 38 `task_type = 'backup'`
- **Codebase audit**: `worker/src/queue/mod.rs` — verified job processor dispatch + existing `process_backup_server` implementation
- **Codebase audit**: `api/src/domain/entities/cron_task.rs` — verified current entity fields (no timezone/run_once/last_result)
- **Codebase audit**: `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` — verified SQL queries (no new columns)
- **Codebase audit**: `api/src/presentation/handlers/cron_task_handlers.rs` — verified task_type validation (no 'start'/'sleep')
- **Codebase audit**: `api/src/presentation/routes/api_routes.rs` — verified `POST /api/v1/nodes/:id/commands` maps to `poll_node_commands`
- **Codebase audit**: `api/src/presentation/handlers/node_handlers.rs` — verified `poll_node_commands` is read-only (returns `get_commands`)
- **Codebase audit**: `agent/solys/src/agent_connection.rs` — verified command-to-task_type mapping (start→server.start, stop→server.stop, restart→server.restart already exist)
- **Codebase audit**: `agent/solys/src/handlers/mod.rs` — verified handle_start, handle_stop, handle_restart already implemented
- **Codebase audit**: `api/src/application/services/monitoring_service.rs` — verified Phase 56 sleep detection pattern (status='stopped' + auto_wake=true)
- **Codebase audit**: `app/src/pages/ServerDetails.jsx` — verified Settings tab layout (Settings sections pattern)
- **Codebase audit**: `api/src/infrastructure/node_client/agent_client.rs` — verified `send_command_with_config` WebSocket dispatch pattern
- **Codebase audit**: `api/src/presentation/ws/node_connection_manager.rs` — verified `send_command_to_node` WebSocket pattern
- **Codebase audit**: `worker/Cargo.toml` — verified existing deps (cron 0.15, chrono, sqlx, redis, reqwest)

### Secondary (MEDIUM confidence)
- [CITED: chrono-tz crate docs](https://docs.rs/chrono-tz/latest/chrono_tz/) — IANA timezone parsing, `Tz::from_str`, `with_timezone()`
- [CITED: cron crate docs](https://docs.rs/cron/latest/cron/) — `Schedule::from_str`, `upcoming()`, includes()
- Phase 55 research (55-RESEARCH.md) — verified Worker dispatch architecture patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified via codebase audit (worker/Cargo.toml, existing imports)
- Architecture: HIGH — Agent command mapping verified in agent_connection.rs + handlers/mod.rs; Worker dispatch flow verified
- Pitfalls: HIGH — based on codebase audit of actual implementation patterns
- Phase 55 dispatch issue: MEDIUM — `poll_node_commands` is clearly read-only but Phase 55 code sends POST to it. Could have been fixed since research.

**Research date:** 2026-05-31
**Valid until:** 2026-06-30 (stable dependencies — cron, chrono, chrono-tz are mature crates)
