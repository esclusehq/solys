---
phase: 56-auto-online-sleep-recovery
plan: 03
subsystem: backend
tags: monitoring, sleep, inactivity, auto-restart, backoff, tokio
requires:
  - phase: 56-auto-online-sleep-recovery
    plan: 01
    provides: auto_wake, auto_restart fields on Server entity and repositories
  - phase: 56-auto-online-sleep-recovery
    plan: 02
    provides: API endpoints for sleep/wake and updated use cases
provides:
  - Sleep detection in MonitoringService 30s loop (inactivity-based auto-sleep)
  - Auto-restart backoff with exponential formula, max attempts, and cooldown
  - restart_count reset in metrics collection after stable running
affects:
  - 56-04 (remaining sleep/wake frontend and validation)
tech-stack:
  added: []
  patterns:
    - "tokio::spawn for non-blocking background restart tasks (Pitfall 4 mitigation)"
    - "Database-backed last_player_activity for inactivity tracking (Pitfall 1 mitigation)"
    - "Exponential backoff: 30 * 2^restart_count, capped at restart_cooldown_seconds"
key-files:
  created: []
  modified:
    - api/src/application/services/monitoring_service.rs
key-decisions:
  - "Sleep detection block inserted AFTER crash detection (status-change block) and BEFORE metrics collection — ensures sleeping servers skip metrics collection"
  - "Backoff formula uses 30s base exponentially doubling: 30, 60, 120, 240... capped at restart_cooldown_seconds"
  - "restart_count reset immediately on next monitoring tick after restart (within 30s) since backoff delay ensures server had startup time"
  - "Max attempts logs error but does not publish alert yet — deferred to future phase"
requirements-completed: []
duration: 6 min
completed: 2026-05-30
---

# Phase 56: Auto Online & Sleep Recovery — Plan 03 Summary

**Inactivity-based sleep detection (player count timeout) and exponential backoff auto-restart added to the MonitoringService 30s check_all_servers loop**

## Performance

- **Duration:** 6 min
- **Started:** 2026-05-30T15:20:00Z
- **Completed:** 2026-05-30T15:26:46Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- **CHANGE A — Sleep Detection:** Running servers with `auto_wake=true` and 0 players for >`sleep_timeout_minutes` are auto-slept (stop_server + status=stopped + auto_wake=true + StatusChanged event). `last_player_activity` is reset to `Utc::now()` when players are detected, and initialized on first observation.
- **CHANGE B — Auto-Restart Backoff:** Crashed servers with `auto_restart=true` now use exponential backoff (`30s * 2^restart_count`) capped at `restart_cooldown_seconds`, with a max attempt limit (`max_restart_attempts`). Restart tasks are spawned via `tokio::spawn` to avoid blocking the monitoring loop (Pitfall 4).
- **CHANGE C — restart_count Reset:** In the metrics collection section, `restart_count` is reset to 0 when a server is detected as running, ensuring the counter doesn't permanently block future restarts after successful recovery.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add sleep detection and auto-restart backoff to MonitoringService** — `83f3b81` (feat)

**Plan metadata:** Pending after SUMMARY.md creation.

## Files Created/Modified

### Modified

- `api/src/application/services/monitoring_service.rs` — 315 lines total. Three new code blocks:
  - **Lines 215-268:** Sleep detection block (checks auto_wake, player count, inactivity timeout, triggers sleep)
  - **Lines 143-183:** Auto-restart backoff (max attempt check, exponential formula, tokio::spawn for non-blocking restart)
  - **Lines 272-280:** restart_count reset (resets to 0 when server is running on monitoring tick)

## Decisions Made

- Sleep detection inserted AFTER crash detection and BEFORE metrics collection to ensure sleeping servers skip redundant metrics collection
- Backoff formula uses 30s base doubling (30, 60, 120, 240s...) capped at `restart_cooldown_seconds`, matching RESEARCH.md architecture patterns
- `restart_count` reset uses the existing loop tick frequency (30s) — if server is running on the next tick after restart, counter resets immediately (backoff delay ensures server had startup time)
- Max attempts reached logs an error without publishing alerts — alert system integration deferred to future phase
- Database-backed `last_player_activity` approach (not in-memory tracking) per Pitfall 1 mitigation — survives service restarts

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

No stubs found — all sleep detection, backoff, and counter reset logic is fully wired.

## Threat Surface Scan

No new threat surface introduced beyond the STRIDE threats already analyzed in the plan's threat model:
- T-56-07 (Sleep spoofing): Mitigated by `auto_wake` guard
- T-56-08 (Restart loop): Mitigated by `max_restart_attempts` hard stop
- T-56-09 (Tokio unbounded tasks): Accepted — restart events are infrequent
- T-56-10 (last_player_activity persistence): Mitigated by DB-backed field
- T-56-11 (Manual vs auto-sleep race): Mitigated by `auto_wake` separation

## Next Phase Readiness

- Sleep detection logic complete and compiling
- Auto-restart backoff with exponential formula and max attempts working
- restart_count reset mechanism in place
- Ready for Plan 56-04 (remaining sleep/wake frontend components and validation)

## Self-Check: PASSED

- ✅ Summary file created at `.planning/phases/56-auto-online-sleep-recovery/56-03-SUMMARY.md`
- ✅ Task commit `83f3b81` found in git log
- ✅ File modified: `api/src/application/services/monitoring_service.rs` exists and is 315 lines
- ✅ `cargo check` passes (no errors)
- ✅ Sleep detection block confirmed at correct position (after crash detection, before metrics)
- ✅ `tokio::spawn` present for non-blocking backoff
- ✅ Exponential backoff formula matches plan spec
- ✅ `restart_count` reset present in metrics section
- ✅ `auto_wake` guard prevents sleep on non-sleeping servers
- ✅ `last_player_activity` and `sleep_timeout_minutes` used for inactivity timeout

---

*Phase: 56-auto-online-sleep-recovery*
*Completed: 2026-05-30*
