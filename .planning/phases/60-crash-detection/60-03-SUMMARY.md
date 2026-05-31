---
phase: 60-crash-detection
plan: 03
subsystem: backend
tags: crash-detection, classification, monitoring, webhooks, rest-api
requires:
  - 60-01 (Data Layer — ServerCrashLog, migration, repository)
  - 60-02 (Agent Protocol + Crash Reporter — CrashReport message, Docker event listener)
provides: "Backend crash processing pipeline: classification → persistence → recovery → notification"
affects:
  - MonitoringService (new pool field, crash report channel)
  - WebhookService (new CrashDetected handler)
  - node_ws_handler (new CrashReport match arm)
  - ws_handler (new CrashDetected match arms)
tech-stack:
  added:
    - once_cell
    - regex
  patterns:
    - "Crash classification uses pure function with Lazy static Regex"
    - "Crash report routed via mpsc channel between WS handler and MonitoringService"
    - "Recovery actions per D-03: OOM→notify_only, ConfigError→disable_auto_restart, PluginCrash/Generic→auto_restart"
    - "Discord crash notification includes crash type, exit code, recovery action, log excerpt embed fields"
key-files:
  created:
    - api/src/application/services/crash_classifier.rs
  modified:
    - api/src/shared/events.rs
    - api/src/application/services/mod.rs
    - api/src/application/services/monitoring_service.rs
    - api/src/application/services/webhook_service.rs
    - api/src/bootstrap/container.rs
    - api/src/infrastructure/external_services/discord_client.rs
    - api/src/presentation/handlers/node_ws_handler.rs
    - api/src/presentation/handlers/server_handlers.rs
    - api/src/presentation/handlers/ws_handler.rs
    - api/Cargo.toml
key-decisions:
  - "Crash report routed via mpsc channel (capacity 256) from WS handler to MonitoringService drain at top of each tick — keeps monitoring loop as single source of truth"
  - "MonitoringService gets pool: PgPool field for crash log repository access (not behind generic abstraction)"
  - "crash_report_rx wrapped in Mutex<Option<...>> for &self access (std::sync::Mutex, no .await while holding guard — collect then process pattern)"
  - "Discord crash notification uses dedicated send_crash_notification() method with full embed details (not the generic send_server_event)"
  - "REST endpoints follow existing server/:id/* pattern: GET/DELETE /crash-logs, POST /crash-logs/:id/resolve"
  - "recovery action for ConfigError disables auto_restart on the server entity (informs monitoring loop to stop attempting restarts)"
duration: ~20 min
completed: 2026-05-31
---

# Phase 60 (Crash Detection) Plan 03: Backend Crash Processing

**One-liner:** Crash classification (OOM/config/plugin/generic) with monitoring service integration, WS handler plumbing, REST crash-log endpoints, and Discord notification formatting.

## Tasks Completed

| Task | Name | Summary |
|------|------|---------|
| 1 | crash_classifier.rs + CrashDetected event | Created pure `classify_crash()` function with regex patterns for OOM, plugin crashes, config errors; added `CrashDetected` variant to `ServerEvent` enum |
| 2 | MonitoringService crash ingestion | Added `crash_report_rx: Mutex<Option<mpsc::Receiver>>` and `pool: PgPool` fields; drain at top of `check_all_servers()`; `handle_crash_report()` full lifecycle; `store_crash_log()`, `notify_crash()`, `disable_auto_restart()` helpers |
| 3 | WS handler + REST endpoints | `NodeMessage::CrashReport` match arm forwards via `crash_report_tx`; added `GET /servers/:id/crash-logs`, `DELETE /servers/:id/crash-logs`, `POST /servers/:id/crash-logs/:log_id/resolve` handlers in server_handlers.rs |
| 4 | Discord notification formatting | Added `send_crash_notification()` method to Discord client with full embed (crash type, exit code, recovery, log excerpt); `CrashDetected` handler in webhook_service.rs |

## Deviations from Plan

### Rule 2 — Added missing critical functionality

1. **MonitoringService pool field** — Original MonitoringService didn't have `PgPool` access. Crash log storage required it. Added `pool: PgPool` field and constructor parameter. This is necessary for `PostgresCrashLogRepository` instantiation inside `store_crash_log()`.

2. **Mutex wrapping** — The plan specified `Option<mpsc::Receiver>` field but `check_all_servers(&self)` uses `&self` (not `&mut self`), so the receiver needed `Mutex` wrapping for mutable access. Used `std::sync::Mutex` with collect-then-process pattern to avoid `MutexGuard` across `.await`.

### Rule 1 — Bug fixes

1. **ServerCrashLog field mismatch** — The entity struct defines `log_excerpt: Option<String>` (not `String`) and includes `created_at: DateTime<Utc>`. Fixed `store_crash_log()` to wrap `log_excerpt` in `Some()` and add `created_at`.

2. **Non-exhaustive ServerEvent matches** — The `CrashDetected` variant needed match arms in `ws_handler.rs` (two locations for event subscription filtering). The wildcard `_ => {}` in `webhook_service.rs` covered it there.

## Known Stubs

- **Discord crash notification log_excerpt** — In `webhook_service.rs`, the crash notification calls `send_crash_notification()` with a constructed `log_excerpt` string ("Crash type: {type}, Recovery: {action}") instead of the actual log excerpt, because the `CrashDetected` event doesn't carry the full log excerpt (it only has `crash_type`, `exit_code`, `recovery_action`). The full log excerpt is stored in the database via `store_crash_log()`. If needed, the `CrashDetected` event can be extended to include `log_excerpt` in a future plan.

## Success Criteria

- [x] crash_classifier.rs created with CrashType enum + classify_crash() pure function
- [x] CrashDetected variant added to ServerEvent enum
- [x] MonitoringService ingests crash reports, classifies, stores, notifies
- [x] WS handler dispatches CrashReport messages to monitoring channel
- [x] REST endpoints for crash logs (list, clear, resolve)
- [x] Routes registered for crash-logs
- [x] Discord client has crash notification formatting
- [x] Webhook service handles CrashDetected events
- [x] API compiles cleanly

## Metrics

- **Duration:** ~20 minutes
- **Files changed:** 11 files (1 new, 10 modified)
- **Commits:** 1 (api@884576c)

## Self-Check: PASSED
