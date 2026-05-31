---
phase: 60-crash-detection
plan: 01
subsystem: backend
tags: database, migration, entity, repository
requires:
  - phase: 60-crash-detection
    provides: Phase context (CONTEXT.md, RESEARCH.md, PATTERNS.md)
provides:
  - server_crash_logs migration table with 9 columns + 2 indexes
  - ServerCrashLog entity with sqlx::FromRow
  - PostgresCrashLogRepository with 6 CRUD methods
affects: []
tech-stack:
  added: []
  patterns:
    - sqlx migration with IF NOT EXISTS idempotency
    - Entity with FromRow derive for sqlx mapping
    - Repository with PgPool and parameterized queries
key-files:
  created:
    - api/migrations/20260531000002_create_server_crash_logs.sql
    - api/src/domain/entities/server_crash_log.rs
    - api/src/infrastructure/repositories/crash_log_repository.rs
  modified:
    - api/src/domain/entities/mod.rs
    - api/src/infrastructure/repositories/mod.rs
key-decisions:
  - "log_excerpt is Option<String> (nullable) to handle crash types without log output"
  - "resolved_at is Option<DateTime<Utc>> (nullable until user acknowledges)"
  - "VARCHAR(32) for crash_type and recovery_action - bounded string values"
  - "B-tree indexes on server_id (filtered lookups) and crashed_at DESC (newest-first pagination)"
requirements-completed: []
duration: 2 min
completed: 2026-05-31
---

# Phase 60 Plan 01: Crash Log Data Layer Summary

**Created database foundation for crash forensic storage** — new `server_crash_logs` table, `ServerCrashLog` entity, and `PostgresCrashLogRepository`.

- Migration creates D-06 table with 9 columns (id, server_id, crashed_at, exit_code, crash_type, log_excerpt, recovery_action, resolved_at, created_at) + 2 B-tree indexes
- Entity struct with `FromRow` derive matches migration columns exactly for sqlx query_as mapping
- Repository provides `insert`, `list_by_server`, `count_by_server`, `count_recent`, `delete_by_server`, `resolve` — all with parameterized queries
- Modules registered in both mod.rs files

## Task Commits

1. **Task 1-3: Migration, entity, and repository** — `3edf8dd` (feat) in api sub-repo

## Files Created/Modified

### New
- `api/migrations/20260531000002_create_server_crash_logs.sql` — 17 lines
- `api/src/domain/entities/server_crash_log.rs` — 17 lines
- `api/src/infrastructure/repositories/crash_log_repository.rs` — 107 lines

### Modified
- `api/src/domain/entities/mod.rs` — added `pub mod server_crash_log`
- `api/src/infrastructure/repositories/mod.rs` — added `pub mod crash_log_repository`

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- Migration file has all 9 D-06 columns with correct types
- Entity fields match migration columns (order matches for sqlx query_as)
- All 6 CRUD methods present in repository
- All queries use parameterized SQL ($1, $2, ...)
- Modules registered in both mod.rs files
