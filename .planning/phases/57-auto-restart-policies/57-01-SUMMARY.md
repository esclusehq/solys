---
phase: 57-auto-restart-policies
plan: 01
subsystem: backend/data
tags: migration, entities, repositories, database
requires: []
provides: "Database columns + entity fields + repository queries for restart policy fields (last_restart_at, last_restart_reason, health_check_timeout_seconds)"
affects: "57-02 (backend API), 57-03 (monitoring service), 57-04 (frontend UI)"
tech-stack:
  added: []
  patterns:
    - "ALTER TABLE ADD COLUMN IF NOT EXISTS for idempotent migrations"
    - "New fields added to both old (server.rs) and new (model.rs) Server structs"
    - "New columns added to all SQL query locations in both repositories"
key-files:
  created:
    - api/migrations/20260530000002_add_restart_policy.sql
  modified:
    - api/src/domain/entities/server.rs
    - api/src/infrastructure/repositories/postgres_server_repository.rs
    - api/src/domain/server/model.rs
    - api/src/domain/server/sqlx_repository.rs
    - api/src/application/use_cases/create_server_use_case.rs
key-decisions:
  - "Migration uses IF NOT EXISTS for idempotent column additions"
  - "health_check_timeout_seconds defaults to 5 in DB (NOT NULL DEFAULT 5)"
  - "last_restart_at and last_restart_reason are nullable (no DEFAULT)"
  - "New model fields are Option<T> for sqlx::FromRow compatibility"
duration: 8 min
completed: 2026-05-30
---

# Phase 57: Auto Restart Policies — Plan 01 Summary

**Database migration + entity fields + repository updates for restart policy tracking and health check configuration**

Created migration file adding 3 columns to the servers table, updated both Server structs (old and new models) with matching fields, and updated all query locations in both repositories (postgres_server_repository.rs — 5 locations across 3 methods, sqlx_repository.rs — 7 SELECT + 2 INSERT + 2 UPDATE across both method sets).

## Task Commits

1. **Task 1: Migration + old entity fields** — `3d2e6da`
   - Created `20260530000002_add_restart_policy.sql` with 3 ALTER TABLE statements
   - Added `last_restart_at`, `last_restart_reason`, `health_check_timeout_seconds` to Server struct in server.rs

2. **Task 2: Update old repository** — `0be6f54`
   - Added new columns to INSERT column list + VALUES + bind calls
   - Added to all 3 SELECT queries (find_by_id, list, find_by_node_id)
   - Added row.try_get calls in all 3 builder blocks
   - Added to UPDATE SET + bind calls with correct param numbering

3. **Task 3: Update new model + repository** — `02245b0`
   - Added fields to model.rs struct, Server::new(), and UpdateServerRequest
   - Added to all SELECT queries in both direct and async_trait method sets
   - Added to INSERT and UPDATE in both method sets

4. **Rule 3 fix: Create use case** — `1867021`
   - Added missing fields to Server initialization in create_server_use_case.rs

## Files Created/Modified

### New
- `api/migrations/20260530000002_add_restart_policy.sql` — 3 ALTER TABLE statements

### Modified
- `api/src/domain/entities/server.rs` — 3 new fields after Phase 56 block
- `api/src/infrastructure/repositories/postgres_server_repository.rs` — 10 insertions/6 deletions across 5 query locations
- `api/src/domain/server/model.rs` — 3 new Option fields + Server::new() + UpdateServerRequest
- `api/src/domain/server/sqlx_repository.rs` — Updated both method sets (direct + async_trait impl)

### Rule 3 Fix
- `api/src/application/use_cases/create_server_use_case.rs` — Added 3 fields to Server initialization

## Verification

- ✅ `cargo check` passes with exit code 0 (only warnings)
- ✅ Migration file has 3 ALTER TABLE statements with IF NOT EXISTS
- ✅ All 3 column names appear in INSERT, SELECT (×3), UPDATE in postgres repo
- ✅ All 3 columns have row.try_get calls in all 3 row builder blocks
- ✅ New model fields are Option<T> for FromRow compatibility
- ✅ Bind param numbers are correct in both repositories

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing fields in create_server_use_case.rs Server initialization**

- **Found during:** Verification (cargo check)
- **Issue:** Server struct now requires `last_restart_at`, `last_restart_reason`, `health_check_timeout_seconds` but `create_server_use_case.rs` didn't initialize them
- **Fix:** Added `last_restart_at: None, last_restart_reason: None, health_check_timeout_seconds: 5` in the Server initialization block
- **Files modified:** `api/src/application/use_cases/create_server_use_case.rs`
- **Commit:** `1867021`

## Self-Check: PASSED

- ✅ SUMMARY.md exists at expected path
- ✅ Migration file exists
- ✅ All 4 source files modified correctly
- ✅ `cargo check` passes
- ✅ All git commits found in log
