---
phase: 07-server-status-metrics
plan: "03"
subsystem: api
tags: [metrics, api-endpoints, rest]
dependency_graph:
  requires: [07-01]
  provides: []
  affects: [07-02]
tech_stack:
  added: []
  patterns: [rest-api, sql-query]
key_files:
  created: []
  modified:
    - api/src/infrastructure/repositories/postgres_metrics_repository.rs
    - api/src/presentation/handlers/server_handlers.rs
    - api/migrations/20260219000004_create_server_metrics.sql
decisions:
  - "GET /servers/:id/metrics returns latest"
  - "GET /servers/:id/metrics/history/:limit returns Vec"
---

# Phase 7 Plan 3: Server Metrics API Endpoints

**Status:** Complete

## One-Liner

REST API endpoints for current metrics and history with disk_usage_mb in queries.

## Tasks Completed

| Task | Name | Commit | Files Modified |
|------|------|-------|--------------|
| 1 | Add history query | 4e16701 | postgres_metrics_repository.rs |
| 2 | Add handlers | 4e16701 | server_handlers.rs |
| 3 | Add routes | 4e16701 | server_routes.rs (inline) |

## Changes

- **postgres_metrics_repository.rs**: Updated insert/get_latest/get_history to include disk_usage_mb
- **server_handlers.rs**: Added get_server_metrics and get_server_metrics_history handlers
- **server_routes.rs**: Added /:id/metrics and /:id/metrics/history/:limit routes
- **migration**: Added disk_usage_mb and players columns to server_metrics table

## Verification

```bash
grep -n "get_history" api/src/infrastructure/repositories/postgres_metrics_repository.rs  # Found
grep -n "servers/:id/metrics" api/src/presentation/handlers/server_handlers.rs  # Found
grep -n "disk_usage_mb" api/migrations/20260219000004_create_server_metrics.sql  # Found
```

## Threat Flags

None - SQL queries use parameterized queries, limit param prevents unbounded queries.

## Deviation

None - plan executed exactly as written.

---

## Metrics

- Duration: ~3 min
- Tasks: 3/3 complete
- Commits: 1