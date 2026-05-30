---
phase: 56-auto-online-sleep-recovery
plan: 01
subsystem: backend
tags: database, migration, domain-model, repository
requires: []
provides:
  - Database migration for auto_wake, sleep_timeout_minutes, last_player_activity, max_restart_attempts, restart_cooldown_seconds
  - Old Server entity with 5 new fields
  - New Server model with auto_wake and sleep_timeout_minutes
  - Both repositories updated for read/write of new fields
affects:
  - 56-02 (API endpoints consume the new model fields)
  - 56-03 (MonitoringService uses old entity fields)
tech-stack:
  added: []
  patterns:
    - "ALTER TABLE ADD COLUMN IF NOT EXISTS for idempotent migrations"
    - "Option<bool>/Option<i32> for nullable model fields"
    - "try_get().unwrap_or(default) for backward-compatible row mapping"
key-files:
  created:
    - api/migrations/20260530000001_add_auto_wake.sql
  modified:
    - api/src/domain/entities/server.rs
    - api/src/infrastructure/repositories/postgres_server_repository.rs
    - api/src/domain/server/model.rs
    - api/src/domain/server/sqlx_repository.rs
key-decisions:
  - "auto_wake: Option<bool> on new model for null-safe query contexts"
  - "sleep_timeout_minutes: Option<i32> on new model matching auto_wake pattern"
  - "Row mapping defaults: auto_wake=false, sleep_timeout_minutes=30, max_restart_attempts=5, restart_cooldown_seconds=300"
  - "last_player_activity nullable with .ok().flatten() for Option<DateTime<Utc>>"
requirements-completed: []
duration: 3 min
completed: 2026-05-30
---

# Phase 56: Auto Online & Sleep Recovery — Plan 01 Summary

**Database migration and domain model extension for sleep/wake and auto-restart backoff fields across both Server models and repositories.**

- **Duration:** 3 min
- **Tasks:** 3
- **Files modified:** 5 (1 created, 4 modified)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create database migration** — `eed48c1` (feat)
2. **Task 2: Add fields to old Server entity + PostgresServerRepository** — `cca3a86` (feat)
3. **Task 3: Add fields to new Server model + SqlxServerRepository** — `6a6616a` (feat)

## Files Created/Modified

### Created

- `api/migrations/20260530000001_add_auto_wake.sql` — 5 ALTER TABLE statements with IF NOT EXISTS for auto_wake, sleep_timeout_minutes, last_player_activity, max_restart_attempts, restart_cooldown_seconds

### Modified

- `api/src/domain/entities/server.rs` — Added 5 new fields after network_name: auto_wake (bool), sleep_timeout_minutes (i32), last_player_activity (Option<DateTime<Utc>>), max_restart_attempts (i32), restart_cooldown_seconds (i32)
- `api/src/infrastructure/repositories/postgres_server_repository.rs` — INSERT/UPDATE/SELECT queries and row mappings for all 5 fields in both find_by_id and list methods
- `api/src/domain/server/model.rs` — Added auto_wake (Option<bool>) and sleep_timeout_minutes (Option<i32>) to both Server struct and UpdateServerRequest
- `api/src/domain/server/sqlx_repository.rs` — All SELECT/INSERT/UPDATE queries updated with auto_wake and sleep_timeout_minutes columns and bindings across all query variants (standalone + trait impl)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Self-Check: PASSED

- ✅ Migration file exists with 5 ALTER TABLE ADD COLUMN IF NOT EXISTS statements
- ✅ Old Server entity has all 5 new fields
- ✅ PostgresServerRepository has INSERT/UPDATE/SELECT + row mappings for all fields
- ✅ New Server model has auto_wake and sleep_timeout_minutes
- ✅ SqlxServerRepository has all SELECT/INSERT/UPDATE with new columns
- ✅ All 3 task commits found
