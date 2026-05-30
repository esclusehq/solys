---
phase: 56-auto-online-sleep-recovery
plan: 02
subsystem: backend
tags: dto, use-case, api-handler, sleep-wake, auto-restart
requires:
  - phase: 56-auto-online-sleep-recovery
    provides: Database migration and domain model with auto_wake/sleep_timeout_minutes fields (Plan 01)
provides:
  - DTO field definitions for auto_wake, sleep_timeout_minutes, max_restart_attempts, restart_cooldown_seconds in all 3 DTOs
  - CreateServerUseCase defaults for all sleep/wake fields
  - UpdateServerUseCase conditional update blocks for all sleep/wake fields
  - POST /servers/:id/sleep endpoint (stop + auto_wake=true)
  - POST /servers/:id/wake endpoint (start + reset auto_wake=false)
  - update_server handler accepts and persists auto_wake and sleep_timeout_minutes
affects:
  - 56-03 (MonitoringService consumes auto_wake/sleep_timeout fields for auto-sleep detection)
  - 56-04 (Frontend uses new endpoints and displays sleep/wake status)
tech-stack:
  added: []
  patterns:
    - "if let Some(auto_wake) = req.auto_wake { server.auto_wake = auto_wake; }" conditional update
    - "auto_wake: req.auto_wake.unwrap_or(false)" default initialization
    - "server.auto_wake = Some(true)" for sleep endpoint
    - "updated.auto_wake = Some(false)" for wake endpoint
key-files:
  created: []
  modified:
    - api/src/application/dto/server_dtos.rs
    - api/src/application/use_cases/update_server_use_case.rs
    - api/src/presentation/handlers/server_handlers.rs
key-decisions:
  - "sleep_server: stop via agent/solys + set auto_wake=true + emit server.sleep event"
  - "wake_server: guard with auto_wake==true check + start via agent/solys + reset auto_wake=false + emit server.wake event"
  - "update_server handler (new code path via SqlxServerRepository) accepts auto_wake and sleep_timeout_minutes from UpdateServerRequest"
  - "Tenant isolation check in both new handlers identical to existing stop_server/start_server pattern"
requirements-completed: []
duration: 4 min
completed: 2026-05-30
---

# Phase 56: Auto Online & Sleep Recovery — Plan 02 Summary

**API endpoint layer for sleep/wake: DTOs define the API contract, use cases set defaults and apply updates, handlers expose POST /servers/:id/sleep and POST /servers/:id/wake.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-30T15:16:00Z
- **Completed:** 2026-05-30T15:20:36Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added auto_wake, sleep_timeout_minutes, max_restart_attempts, restart_cooldown_seconds to all 3 DTO structs (CreateServerRequest, UpdateServerRequest, ServerResponse) with correct Option/non-Option typing
- UpdateServerRequest Default impl has all 4 new fields set to None
- CreateServerUseCase auto_wake defaults to false, sleep_timeout_minutes to 30, max_restart_attempts to 5, restart_cooldown_seconds to 300 (already wired in 56-01)
- UpdateServerUseCase applies all 4 fields conditionally via `if let Some(...)` blocks
- `update_server` handler (new code path) accepts and persists `auto_wake` and `sleep_timeout_minutes` from `domain::server::model::UpdateServerRequest`
- Router entries for `/servers/:id/sleep` and `/servers/:id/wake` added
- `sleep_server` handler: agent/solys stop logic + sets auto_wake=true + emits server.sleep event
- `wake_server` handler: validates auto_wake==true guard + agent/solys start logic + resets auto_wake=false + emits server.wake event
- Tenant isolation in both new handlers

## Task Commits

Each task was committed atomically to the nested `api` repository:

1. **Task 1: Add sleep/wake fields to server DTOs** — `ad32d05` (feat)
2. **Task 2: Wire auto_wake defaults in update use case** — `ce76f84` (feat)
3. **Task 3: Add sleep/wake API endpoints and wire update_server handler** — `686975c` (feat)

## Files Modified

- `api/src/application/dto/server_dtos.rs` — 4 new Option fields in CreateServerRequest + UpdateServerRequest, 4 None defaults in Default impl, 4 non-Option fields in ServerResponse
- `api/src/application/use_cases/update_server_use_case.rs` — 4 conditional update blocks for auto_wake, sleep_timeout_minutes, max_restart_attempts, restart_cooldown_seconds
- `api/src/presentation/handlers/server_handlers.rs` — router entries, sleep_server handler (243 lines), wake_server handler, update_server auto_wake/sleep_timeout_minutes wiring

## Decisions Made

- Followed existing `auto_restart`/`auto_pause` patterns for field naming and wiring (per PATTERNS.md lines 68-165)
- `sleep_server` stops server via agent or solys executor, sets auto_wake=true, and emits `server.sleep` audit event
- `wake_server` validates auto_wake==true before starting, resets to false after, and emits `server.wake` audit event
- Tenant isolation checks before any operation in both new handlers

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

- All API endpoints for sleep/wake are implemented
- DTOs support both old (ServerRepository trait) and new (SqlxServerRepository) code paths
- Ready for Plan 56-03: MonitoringService sleep detection (auto-sleep on player inactivity timeout)
- Ready for Plan 56-04: Frontend UI for sleep/wake configuration

## Self-Check: PASSED

- ✅ CreateServerRequest DTO has 4 new Option fields (grep finds 2 matches + 2 more)
- ✅ UpdateServerRequest DTO has 4 new Option fields
- ✅ ServerResponse DTO has 4 new non-Option fields
- ✅ CreateServerUseCase initializes all fields with correct defaults
- ✅ UpdateServerUseCase has 4 conditional update blocks
- ✅ Router has both /:id/sleep and /:id/wake routes
- ✅ sleep_server function exists with stop logic + auto_wake=true
- ✅ wake_server function exists with guard + start logic + auto_wake=false
- ✅ update_server handler processes payload.auto_wake and payload.sleep_timeout_minutes
- ✅ `cargo check` passes (no errors)
- ✅ All 3 task commits found in api repo git log

---

*Phase: 56-auto-online-sleep-recovery*
*Completed: 2026-05-30*
