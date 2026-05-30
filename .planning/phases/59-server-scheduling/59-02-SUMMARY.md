---
phase: 59-server-scheduling
plan: 02
subsystem: worker, api
tags: cron, timezone, dispatch, retry, events, websocket

requires:
  - phase: 59-server-scheduling
    provides: 59-01 cron_tasks extended schema (timezone, run_once, last_result, last_error)
provides:
  - Timezone-aware cron evaluation evaluating all task types (backup, start, stop, restart, sleep)
  - 4 scheduled action job handlers with D-05 retry (30s delay, 1 retry)
  - API command dispatch endpoint (POST /api/v1/nodes/:id/dispatch) as Worker→Agent proxy
  - D-05 toast+event emission via ServerEvent::ScheduleFailed and server_events table
  - D-06 run_once auto-disable, D-07 sleep guard, D-08 restart cooldown logic
affects:
  - 59-03 (Frontend cron task UI — backend dispatch and event wiring are complete)

tech-stack:
  added: [chrono_tz]
  patterns:
    - Timezone-aware cron scheduling with chrono-tz and cron crate
    - D-05 retry via inline re-dispatch after 30s sleep
    - Dispatch-through-API-proxy pattern (Worker never talks directly to agents)
    - Best-effort notification pattern (EventBus publish + DB write, logged on failure)

key-files:
  created: []
  modified:
    - worker/src/cron_eval.rs — Extended for all task types with timezone evaluation
    - worker/src/queue/mod.rs — Added 4 scheduled action handlers + D-05 retry helpers
    - api/src/shared/events.rs — Added ScheduleFailed variant
    - api/src/presentation/handlers/node_handlers.rs — Added dispatch endpoint + notification helper
    - api/src/presentation/handlers/ws_handler.rs — ScheduleFailed match arms
    - api/src/presentation/routes/api_routes.rs — /dispatch route registration

key-decisions:
  - "Used state.node_client.send_command (4 args) instead of send_command_with_config (5 args) in dispatch handler — deploy_config not needed for scheduled actions"
  - "dispatch_to_agent_with_params implements D-05 retry inline (re-calls execute_dispatch after 30s), replacing the handle_cron_failure pattern from earlier design"
  - "API uses serde(tag = type, content = payload) on ServerEvent enum — ScheduleFailed follows same tagged-struct pattern as existing variants"

requirements-completed: []

duration: ~15 min
completed: 2026-05-31
---

# Phase 59: Server Scheduling — Plan 02 Summary

**Timezone-aware cron evaluation for all task types, 4 scheduled action job handlers with D-05 retry, and API dispatch proxy endpoint with EventBus toast notifications**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-31T06:00:00Z
- **Completed:** 2026-05-31T06:15:00Z
- **Tasks:** 3
- **Files modified:** 6 (2 worker, 4 api)

## Accomplishments

- Extended `cron_eval.rs` to evaluate ALL enabled cron tasks (not just backup) with timezone-aware `is_cron_due_in_timezone()` function using chrono-tz (D-03)
- Mapped 5 task types to Redis job types: backup→backup_server, start→scheduled_start, stop→scheduled_stop, restart→scheduled_restart, sleep→scheduled_sleep (D-02)
- Added 4 job handlers in `queue/mod.rs` with server-state checks, D-06 run_once auto-disable, D-07 sleep state guard, and D-08 restart cooldown
- Implemented D-05 retry pattern: `execute_dispatch` (single HTTP call) → `dispatch_to_agent_with_params` (attempt 1 → 30s sleep → attempt 2 → mark failed if both fail)
- Created `POST /api/v1/nodes/:id/dispatch` endpoint as Worker→Agent proxy, forwarding commands via existing WebSocket `send_command` on `NodeClient`
- On dispatch failure: publishes `ServerEvent::ScheduleFailed` via EventBus (frontend toast) + writes `server.schedule_failed` server event to DB (D-05 Toast+Event)
- Sleep semantics: dispatch handler sets `auto_wake=true` when `params.sleep` is true
- Updated `ws_handler.rs` to match the new `ScheduleFailed` variant in both server_id extraction blocks

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend cron_eval.rs** — `90331f9` (feat) in main repo
2. **Task 2: Add job handlers to queue/mod.rs** — `29c5a7c` (feat) in main repo
3. **Task 3: Create API dispatch endpoint** — `834bc66` (feat) in api sub-repo

**Plan metadata:** (committed in next step)

## Files Created/Modified

### Worker (main repo)
- `worker/src/cron_eval.rs` — Extended 101→163 lines: added chrono_tz import, is_cron_due_in_timezone(), multi-task-type SQL, task_type→job_type mapping, timezone-aware dispatch
- `worker/src/queue/mod.rs` — Extended 217→521 lines: added 4 match arms, 3 helpers (update_cron_task_result, is_run_once, disable_cron_task), 4 handlers (start/stop/restart/sleep), 3 dispatch helpers (execute_dispatch, dispatch_to_agent_with_params, dispatch_to_agent)

### API (sub-repo)
- `api/src/shared/events.rs` — Added ScheduleFailed variant with server_id, command, reason fields
- `api/src/presentation/handlers/node_handlers.rs` — Added DispatchCommandRequest/DispatchParams DTOs, dispatch_node_command handler, emit_schedule_failure_notifications helper (EventBus publish + sqlx insert)
- `api/src/presentation/handlers/ws_handler.rs` — Added ServerEvent::ScheduleFailed to both match arms
- `api/src/presentation/routes/api_routes.rs` — Added POST /api/v1/nodes/:id/dispatch route

## Decisions Made

- Used `state.node_client.send_command()` (4 params) instead of `send_command_with_config()` (5 params) in the dispatch handler since no deploy_config is needed for scheduled actions
- D-05 retry is inline in `dispatch_to_agent_with_params` — re-calls `execute_dispatch` after 30s sleep, marks failed only if both attempts fail. This replaces the earlier `handle_cron_failure` pattern
- Best-effort notification: EventBus publish and server_events insert both have logging-only error handling — dispatch response is never blocked by notification failures

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

- Worker cron evaluation now handles all task types with timezone awareness
- All 4 scheduled action handlers (start/stop/restart/sleep) are implemented with proper state checks and D-05/D-06/D-07/D-08 behavior
- API dispatch endpoint is ready for Worker to proxy commands to agents
- Toast notifications and server events are wired for schedule failure visibility
- Ready for Plan 59-03 (Frontend: cron task management UI, schedule visualization, manual dispatch trigger)

---

*Phase: 59-server-scheduling*
*Completed: 2026-05-31*
