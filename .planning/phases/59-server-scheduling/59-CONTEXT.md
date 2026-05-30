# Phase 59: Server Scheduling - Context

**Gathered:** 2026-05-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver scheduled server actions — start, stop, restart, and sleep game servers automatically based on recurring schedules with timezone support.

Builds on existing `cron_tasks` infrastructure (table, entity, repository, Worker cron evaluation loop, API SchedulerService, frontend ScheduledTasksPage). Extends Phase 55's Worker-based dispatch pattern from `backup`-only to all server lifecycle actions.

Expands D-04 from Phase 55 ("Only backup task type is fully automated for now") — Phase 59 makes `start`, `stop`, `restart`, and `sleep` task types fully automated alongside `backup`.

</domain>

<decisions>
## Implementation Decisions

### D-01: Execution Architecture — Worker-Based Dispatch
**Decision:** Extend Worker `cron_eval.rs` to handle `start`, `stop`, `restart`, and `sleep` task types alongside existing `backup` type.

The Worker cron evaluation loop polls `cron_tasks` table for due tasks, dispatches them as Redis jobs, and the job processor sends commands to the Agent via existing HTTP proxy pattern (`Worker → API → Agent WebSocket`). Follows Phase 55's exact pattern (D-02, D-09 from Phase 55).

Rationale: Consistent with existing architecture. Worker already has cron eval loop, Redis queue, and DB access. API's `SchedulerService` stubs for restart/stop/command remain but Worker becomes the canonical scheduler.

### D-02: Task Type — Add `start` and `sleep` to cron_tasks
**Decision:** Add two new task type values to the existing `cron_tasks.task_type` field:

- `start` — Start a stopped/sleeping server
- `sleep` — Put a running server to sleep (Phase 56 sleep semantics, not hard stop)

These join existing types: `backup`, `restart`, `stop`, `command`.

The `sleep` type uses Phase 56's sleep mechanism (graceful stop + auto-wake capable). It differs from `stop` (which is a hard stop without auto-wake semantics).

### D-03: Timezone Support — Per-Schedule Timezone Column
**Decision:** Add `timezone` VARCHAR(50) column to `cron_tasks` table, defaulting to `'UTC'`.

The Worker cron evaluation loop reads the schedule's timezone, converts the cron expression's evaluation context to that timezone, then checks if the task is due. This ensures a schedule like "0 8 * * *" with timezone "Asia/Jakarta" runs at 8 AM Jakarta time regardless of server UTC clock.

Implementation approach: Use the `chrono-tz` or `cron` crate's timezone-aware scheduling. The cron expression itself stays in standard UTC-based format — only the evaluation timezone changes. Simpler than converting cron expressions to UTC.

### D-04: UI Placement — Settings Tab Section in ServerDetails
**Decision:** Add a "Scheduled Actions" section in the Settings tab of ServerDetails page, alongside the existing Sleep/Wake and Restart Policy sections. This places all server automation configuration in one consistent location.

The section shows a compact list of schedules for this server (start/stop/restart/sleep) with add/edit/delete inline inline. Each schedule displays:
- Action type (Start, Stop, Restart, Sleep)
- Cron expression (human-readable)
- Timezone
- Enabled/disabled toggle
- Run-once badge (if applicable)
- Last-run timestamp
- Last-result indicator (success/failure)

The existing `ScheduledTasksPage.jsx` remains accessible for power users who want a full table view across all servers, but the primary scheduling UX is inline in ServerDetails Settings.

### D-05: Error Handling — Log + Toast + Event + Retry
**Decision:** When a scheduled action fails:
1. **Log:** Structured error log via tracing
2. **Toast:** Push a toast notification to the frontend via existing WebSocket event bus
3. **Event:** Write a server event (e.g., `server.schedule_failed`) with failure reason
4. **Retry:** Retry once after 30 seconds. If retry also fails, mark with final error status

The `cron_tasks` table gets a `last_result` column (TEXT — `success` or error message) and `last_error` (TEXT — details of last failure) to surface status in the UI.

### D-06: One-Time Actions — `run_once` Flag
**Decision:** Add `run_once` BOOLEAN column to `cron_tasks`, default `false`.

When `run_once = true`:
- The cron schedule determines *when* to fire (e.g., "tomorrow at 8 AM")
- After the action fires, the task auto-disables (`enabled = false`) and `last_run` is set
- The UI shows a "Run Once" badge and disables editing the schedule after execution

This enables single-use scheduled actions like "restart at 3 AM tomorrow" without users needing to manually disable a recurring schedule.

### D-07: Sleep Interaction — Phase 56 Auto-Sleep Takes Precedence
**Decision:** If the server is already in sleep state (Phase 56 auto-sleep triggered due to inactivity), a scheduled `sleep` action is skipped with a log message. No duplicate sleep or error.

Conversely, if the server was manually started before a scheduled `sleep` fires, the schedule fires normally (Phase 56 auto-sleep checks inactivity timeout, scheduled sleep is an explicit action).

### D-08: Scheduled Restart vs Phase 57 Auto-Restart
**Decision:** These are complementary, not competing:

- **Scheduled restart (Phase 59):** User-defined recurring restart (e.g., "restart every Sunday 3 AM"). Initiates a clean restart regardless of server health. Bypasses auto-restart crash detection.
- **Auto-restart (Phase 57):** Automatic restart triggered by crash or unresponsive detection. Respects max_attempts and cooldown.

A scheduled restart that fires while auto-restart is in progress should wait for the auto-restart cooldown to complete before executing.

### The Agent's Discretion
- Specific UI layout of the Scheduled Actions section in Settings tab
- Cron expression human-readable display format (e.g., "Daily at midnight")
- Schedule add/edit inline form design
- `last_result` and `last_error` display format
- Worker cron_eval extension details for new task types
- Redis job queue design for non-backup task types
- Frontend state management for schedule form (local state vs API fetch)
- Toast notification design and auto-dismiss duration
- Worker-to-API command proxy pattern (same as Phase 55's APPROACH A)
- `chrono-tz` integration details for timezone conversion
- Migration strategy for existing cron_tasks rows (add timezone default 'UTC')
- Validation rules for cron expression + timezone combination

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Cron Infrastructure (Already Built)
- `api/migrations/20260409000006_create_cron_tasks_table.sql` — cron_tasks table
- `api/src/domain/entities/cron_task.rs` — CronTask entity (source of truth for field additions)
- `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` — Postgres cron task repo
- `api/src/presentation/handlers/cron_task_handlers.rs` — REST handlers for cron_tasks CRUD
- `api/src/application/services/scheduler_service.rs` — API-side SchedulerService (stubs for restart/stop/command)

### Worker Cron Loop (Phase 55 Pattern)
- `worker/src/cron_eval.rs` — Worker cron evaluation loop (currently backup-only)
- `worker/src/queue/mod.rs` — Job queue processor (process_backup_server pattern)
- `worker/src/main.rs` — Worker entry point with cron_eval::run_cron_evaluation_loop spawn

### Phase 55 Scheduled Backups (Reference Pattern)
- `.planning/phases/55-scheduled-backups/55-CONTEXT.md` — Worker-based dispatch decisions
- `.planning/phases/55-scheduled-backups/55-RESEARCH.md` — Architecture patterns, pitfalls
- `.planning/phases/55-scheduled-backups/55-PATTERNS.md` — File-level analogs

### Phase 56 Auto Online & Sleep Recovery
- `.planning/phases/56-auto-online-sleep-recovery/56-CONTEXT.md` — Sleep/wake semantics
- `api/src/application/services/monitoring_service.rs` — Sleep detection, auto-wake

### Phase 57 Auto Restart Policies
- `.planning/phases/57-auto-restart-policies/57-CONTEXT.md` — Auto-restart decisions
- `.planning/phases/57-auto-restart-policies/57-RESEARCH.md` — Health check, restart history

### Frontend Patterns
- `app/src/pages/ServerDetails.jsx` — Server details page with Settings tab
- `app/src/features/scheduling/ScheduledTasksPage.jsx` — Existing cron task CRUD page
- `app/src/hooks/useServers.js` — Server API hooks
- `app/src/api/client.js` — API client with fetch pattern

### Codebase Maps
- `.planning/codebase/STACK.md` — Tech stack (Rust Axum, React 19, Zustand)
- `.planning/codebase/ARCHITECTURE.md` — Service layer, WebSocket, Agent communication

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `worker/src/cron_eval.rs` — Cron evaluation loop (30s poll, Redis dispatch, 101 lines). Directly extensible for new task types.
- `worker/src/queue/mod.rs` — Job processor with `process_backup_server` pattern. New handlers: `process_scheduled_start`, `process_scheduled_stop`, `process_scheduled_restart`, `process_scheduled_sleep`.
- `api/src/domain/entities/cron_task.rs` — CronTask entity. Add `timezone`, `run_once`, `last_result`, `last_error` fields.
- `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` — Full CRUD. Extend for new columns.
- `api/src/presentation/handlers/cron_task_handlers.rs` — CRUD endpoints. Already handles `schedule_cron`.
- `api/src/application/services/scheduler_service.rs` — `calculate_next_run` method reusable for next_run calculation with timezone.
- `app/src/features/scheduling/ScheduledTasksPage.jsx` — Existing cron task UI for reference.
- `cron` crate (v0.15) — Already in API Cargo.toml. Worker may already have it (added in Phase 55).

### Established Patterns
- Worker-based cron evaluation + Redis dispatch (Phase 55, D-02 pattern)
- Worker → API HTTP proxy for Agent commands (Phase 55 research, Approach A)
- Settings tab section layout in ServerDetails (Phase 56 Sleep/Wake, Phase 57 Restart Policy patterns)
- Per-server conditional updates (if let Some(field) = req.field pattern)
- Toast notifications + server event timeline (Phase 57 D-05 pattern)

### Integration Points
- **Worker cron_eval.rs**: Extend `evaluate_and_dispatch` to filter all enabled task types (not just `backup`). Dispatch different job types to Redis based on `task_type`.
- **Worker queue/mod.rs**: Add job handlers for `scheduled_start`, `scheduled_stop`, `scheduled_restart`, `scheduled_sleep`. Each sends command to Agent via API HTTP proxy.
- **API cron_task handlers**: Extend DTOs and CRUD for `timezone`, `run_once` fields.
- **API cron_task repository**: Add read/write for new columns.
- **Frontend ServerDetails.jsx Settings tab**: Add "Scheduled Actions" section after Restart Policy section.

### Migration Plan
1. `ALTER TABLE cron_tasks ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) NOT NULL DEFAULT 'UTC'`
2. `ALTER TABLE cron_tasks ADD COLUMN IF NOT EXISTS run_once BOOLEAN NOT NULL DEFAULT false`
3. `ALTER TABLE cron_tasks ADD COLUMN IF NOT EXISTS last_result TEXT`
4. `ALTER TABLE cron_tasks ADD COLUMN IF NOT EXISTS last_error TEXT`

</code_context>

<specifics>
## Specific Ideas

Data flow for scheduled server action:
```
cron_tasks table → Worker cron eval (every 30s) → Worker dispatches Redis job
  → Worker job handler → API HTTP proxy (POST /api/v1/nodes/:id/commands)
  → Agent receives WebSocket command (start/stop/restart)
  → Agent executes action → Reports result
  → Worker updates cron_tasks.last_run / last_result / last_error
  → Worker disables task if run_once=true
  → API pushes WebSocket event → Frontend shows toast
```

Task type → Agent command mapping:
- `start` → `start_server` (Agent starts the container)
- `stop` → `stop_server` (graceful stop, no auto-wake)
- `restart` → `restart_server` (stop + start)
- `sleep` → Phase 56 sleep path (stop with auto-wake enabled)
- `backup` → `backup.start` (existing Phase 55 path)

Frontend Scheduled Actions section layout (Settings tab):
```
[Scheduled Actions]
Action    Schedule        TZ      Status   Last Run    Actions
──────────────────────────────────────────────────────────────
Start     Daily 8:00 AM   UTC     ✓ Done   Today 8:01  [Edit] [Del]
Restart   Sun 3:00 AM     UTC     ⏳ Idle  Yesterday   [Edit] [Del]
Stop      Daily 11 PM     UTC     ❌ Fail  Today 11:01 [Edit] [Del]
                                                 Error: Server was already stopping
                                       [+ Add Schedule]
```

</specifics>

<deferred>
## Deferred Ideas

- **Per-task notification preferences** (e.g., send Discord webhook on schedule failure) — future phase
- **Schedule groups/tags** (e.g., "maintenance window" grouping multiple schedules) — not needed yet
- **Cron expression builder/visual editor** — text input + human-readable preview sufficient for MVP
- **Machine learning-based schedule optimization** — way out of scope
- **Calendar view for schedules** — table view sufficient for initial release

</deferred>

---

*Phase: 59-server-scheduling*
*Context gathered: 2026-05-31*
