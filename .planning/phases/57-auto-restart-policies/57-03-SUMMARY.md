---
phase: 57-auto-restart-policies
plan: 03
subsystem: backend/monitoring
tags: monitoring-service, rcon-health-check, restart-reasons
requires:
  - 57-01 (Data layer — entity fields for last_restart_at/reason)
provides: "RCON health check probe in MonitoringService + restart reason persistence across all restart paths"
affects: "57-04 (frontend UI reads last_restart_at/reason)"
tech-stack:
  added: []
  patterns:
    - "RCON health check between sleep detection and metrics collection in the 30s monitoring loop"
    - "tokio::spawn for non-blocking delayed restart (same as crash detection)"
    - "Last restart reason persistence with 3 enum values: crash_detected, unresponsive, max_attempts_reached"
key-files:
  created: []
  modified:
    - api/src/application/services/monitoring_service.rs
key-decisions:
  - "health_check_timeout_seconds > 0 acts as enable flag for RCON health check"
  - "Single RCON failure triggers unresponsive detection (acceptable for zombie detection)"
  - "3 restart reason values: crash_detected, unresponsive, max_attempts_reached"
  - "No EventBus publish in tokio::spawn (DB update with restart_count + last_restart_at serves as audit trail)"
duration: 5 min
completed: 2026-05-30
---

# Phase 57: Auto Restart Policies — Plan 03 Summary

**RCON health check probe injected into MonitoringService to detect unresponsive zombie servers + restart reason persistence across all restart paths**

Added RCON health check between sleep detection (Phase 56) and metrics collection in the 30s monitoring loop. When a running server has `auto_restart` enabled and `health_check_timeout_seconds > 0`, a failed `collect_metrics()` call triggers unresponsive detection with exponential backoff restart (mirroring crash detection). Also enhanced existing crash detection to persist `last_restart_at` and `last_restart_reason` (crash_detected/unresponsive/max_attempts_reached) in both crash detection and health check paths.

## Commit

**Tasks 1+2: RCON health check + event emission** — `4f1d604`
- Injected RCON health check block between sleep detection and metrics collection
- Unresponsive detection with exponential backoff restart via tokio::spawn
- Added last_restart_at + last_restart_reason to crash detection spawn success path
- Added last_restart_reason to crash detection max_attempts path
- Added last_restart_at + last_restart_reason to RCON health check spawn success path
- Added last_restart_reason to RCON health check max_attempts path

## Files Modified

- `api/src/application/services/monitoring_service.rs` — 85 insertions, 1 deletion

## Verification

- ✅ `cargo check` passes with exit code 0
- ✅ `=== RCON HEALTH CHECK (Phase 57) ===` comment marker exists
- ✅ Health check runs only when `server.auto_restart && server.health_check_timeout_seconds > 0`
- ✅ Failed RCON triggers `last_restart_reason = Some("unresponsive".to_string())`
- ✅ Delayed restart uses `tokio::spawn` with exponential backoff
- ✅ 3 restart reason values: crash_detected, unresponsive, max_attempts_reached

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- ✅ SUMMARY.md exists at expected path
- ✅ File modified correctly
- ✅ `cargo check` passes
- ✅ All required patterns verified by grep
