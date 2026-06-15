---
phase: 79-update-ui-https-app-esluce-com-billing
plan: 01
subsystem: api-routes
tags: [routing, usage, axum]
requires: []
provides: [GET /api/v1/usage/quotas, GET /api/v1/usage/monthly, GET /api/v1/usage/history, GET /api/v1/usage/overage, GET /api/v1/usage/]
affects: [api/src/presentation/routes/api_routes.rs]
tech-stack:
  added: []
  patterns: [axum Router .nest() chaining]
key-files:
  created: []
  modified: [api/src/presentation/routes/api_routes.rs]
decisions: []
metrics:
  duration: ~3m
  completed: 2026-06-15
---

# Phase 79 Plan 01: Mount UsageHandlers router at /api/v1/usage

Mount the existing `UsageHandlers` router into the API route tree via a single `.nest()` call, enabling the frontend to fetch usage quota data and related usage endpoints.

## Tasks Executed

| # | Name | Status |
|---|------|--------|
| 1 | Mount UsageHandlers router at /api/v1/usage | ✅ Complete |

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

- ✅ `grep -n 'nest("/api/v1/usage"'` → line 30 (correct insertion point after billing nest, before users nest)
- ✅ `grep 'UsageHandlers::router' api_routes.rs` → 2 lines (import + nest call)
- ✅ `grep -c 'UsageHandlers' api_routes.rs` → 2 (no duplicate import)
- ✅ `cargo build` → exit code 0, `Finished dev profile`

## Threat Surface Scan

No new threat flags — all usage routes are protected by the existing `AuthUser` extractor (JWT middleware), consistent with the threat model T-79-01/T-79-02 mitigations.

## Known Stubs

None.

## Self-Check: PASSED

All files modified exist, all acceptance criteria verified, build passes.
