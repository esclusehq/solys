---
phase: 59-server-scheduling
plan: 01
subsystem: api
tags: postgres, cron, timezone, chrono-tz, scheduling, rust

# Dependency graph
requires: []
provides:
  - Data layer foundation for cron_task timezone, run_once, and result tracking
  - Extended API handlers accepting new fields and task types
  - Worker chrono-tz dependency for IANA timezone parsing
affects: [59-02, 59-03]

# Tech tracking
tech-stack:
  added: [chrono-tz v0.10 (Worker)]
  patterns:
    - "Conditional DTO field updates (if let Some) for timezone/run_once"
    - "Task type validation via whitelist array"

key-files:
  created:
    - api/migrations/20260531000001_add_cron_task_scheduling_columns.sql
  modified:
    - worker/Cargo.toml
    - api/src/domain/entities/cron_task.rs
    - api/src/infrastructure/repositories/postgres_cron_task_repository.rs
    - api/src/presentation/handlers/cron_task_handlers.rs

key-decisions:
  - "timezone defaults to 'UTC' in both migration (DB) and create_task handler (API)"
  - "run_once defaults to false — explicit opt-in required"
  - "last_result and last_error are NOT user-settable via update — managed by Worker job processor (D-05)"
  - "run_task handler sets last_result='dispatched' for immediate feedback, Worker overwrites on completion"

patterns-established:
  - "Postgres repository selects updated with timezone, run_once, last_result, last_error columns"
  - "INSERT and UPDATE bind params extended with 4 new parameters"
  - "CreateCronTaskRequest uses Option types for new fields with handler-level defaults"

requirements-completed: []

# Metrics
duration: 5 min
completed: 2026-05-30
---

# Phase 59: Server Scheduling — Plan 01 Summary

**Migration, entity, DTO, repository, and handler extensions for cron_tasks timezone-aware scheduling (D-03), run-once actions (D-06), and task result tracking (D-05), plus chrono-tz Worker dependency**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-30T22:44:32Z
- **Completed:** 2026-05-30T22:49:46Z
- **Tasks:** 3
- **Files modified:** 5 (1 created, 4 modified)

## Accomplishments

- Migration SQL adds `timezone` (VARCHAR 50), `run_once` (BOOLEAN), `last_result` (TEXT), `last_error` (TEXT) columns to `cron_tasks` with `IF NOT EXISTS` guard
- Updated `cron_tasks` CHECK constraint to include `'start'` and `'sleep'` task types alongside existing `backup`, `restart`, `stop`, `command`
- Added `chrono-tz` v0.10 with `serde` feature to Worker for IANA timezone parsing
- Extended `CronTask` entity with timezone, run_once, last_result, last_error fields
- Extended `CreateCronTaskRequest` and `UpdateCronTaskRequest` DTOs with timezone and run_once Option fields
- Updated all 3 SELECT queries, INSERT, and UPDATE in Postgres repository with new columns and bind parameters
- Updated `row_to_task` to map all 4 new columns
- Updated `create_task` and `update_task` validation arrays to accept `"start"` and `"sleep"` task types
- Added conditional timezone and run_once field handling in `update_task`
- Set `last_result = "dispatched"` on manual task execution for immediate feedback
- API crate compiles successfully (`cargo check` — no errors, only pre-existing warnings)

## Task Commits

Each task was committed atomically (API nested repo hashes shown):

1. **Task 1: Create migration SQL + chrono-tz Worker dependency** — `095fceb` (feat) in API repo, `f2d9d7b` (feat) in parent repo
2. **Task 2: Extend CronTask entity, DTOs, and Postgres repository** — `9d93e33` (feat)
3. **Task 3: Update cron_task_handlers validation and DTO wiring** — `dc158d2` (feat)

## Files Created/Modified

- `api/migrations/20260531000001_add_cron_task_scheduling_columns.sql` — NEW: 4-column migration + CHECK constraint update
- `worker/Cargo.toml` — ADDED: chrono-tz 0.10 dependency
- `api/src/domain/entities/cron_task.rs` — MODIFIED: CronTask, CreateCronTaskRequest, UpdateCronTaskRequest with 4 new fields
- `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` — MODIFIED: 3 SELECTs, INSERT, UPDATE, row_to_task extended
- `api/src/presentation/handlers/cron_task_handlers.rs` — MODIFIED: validation arrays, CronTask construction, conditional updates, run_task dispatch

## Decisions Made

- timezone defaults to `"UTC"` in both the DB migration (DEFAULT 'UTC') and the create_task handler (unwrap_or_else "UTC") — backward compatible for existing cron_tasks rows
- run_once defaults to `false` — explicit user opt-in only
- last_result and last_error are NOT user-settable via the update endpoint — managed exclusively by the Worker job processor (D-05)
- run_task (manual execution) sets last_result to `"dispatched"` for immediate visual feedback; the Worker overwrites with actual result on completion
- scheduler_service.rs left unchanged per plan guidance — Worker handles all cron evaluation including timezone

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

No stubs found — all created/modified files have fully wired functionality.

## Next Phase Readiness

- Data layer fully extended: migration, entity, DTOs, repository all support timezone, run_once, last_result, last_error
- API handler accepts and validates "start" and "sleep" task types
- Worker dependency ready for timezone conversion (chrono-tz added)
- All changes compile successfully via `cargo check`
- Ready for Plan 59-02 (Worker cron_eval extension + job handlers + API dispatch endpoint)

---

*Phase: 59-server-scheduling*
*Completed: 2026-05-30*
