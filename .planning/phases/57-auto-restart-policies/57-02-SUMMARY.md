---
phase: 57-auto-restart-policies
plan: 02
subsystem: backend/api
tags: dtos, use-cases, handlers, settings, routes
requires:
  - 57-01 (Data layer — migration + entity fields + repositories)
provides: "Backend API chain for restart policy fields + global defaults storage endpoints"
affects: "57-03 (monitoring service), 57-04 (frontend UI)"
tech-stack:
  added: []
  patterns:
    - "UpdateServerRequest/ServerResponse DTO fields with Option<T> for safe partial updates"
    - "Conditional if let Some blocks in update use case"
    - "app_settings key-value table for global defaults storage (JSON)"
    - "Admin-only access enforced via user.is_admin() in handler"
    - "GET/PUT handlers following S3Config + CloudflareConfig pattern"
key-files:
  created: []
  modified:
    - api/src/application/dto/server_dtos.rs
    - api/src/application/use_cases/update_server_use_case.rs
    - api/src/presentation/handlers/server_handlers.rs
    - api/src/domain/entities/settings.rs
    - api/src/domain/repositories/settings_repository.rs
    - api/src/infrastructure/repositories/postgres_settings_repository.rs
    - api/src/presentation/handlers/settings_handlers.rs
    - api/src/presentation/routes/api_routes.rs
key-decisions:
  - "Only health_check_timeout_seconds wired in update_server handler (last_restart_at/reason are monitoring-service-only)"
  - "Global defaults stored as JSON in existing app_settings table with key 'restart_defaults'"
  - "RestartDefaults default values: max_attempts=5, cooldown=300"
  - "API endpoint /api/v1/settings/restart-defaults requires admin access"
duration: 6 min
completed: 2026-05-30
---

# Phase 57: Auto Restart Policies — Plan 02 Summary

**Backend API chain for restart policy fields (DTOs → use cases → handlers) + global defaults storage with admin-only endpoints**

Wired restart policy fields through the full API chain. Added `last_restart_at`, `last_restart_reason`, `health_check_timeout_seconds` to `UpdateServerRequest`, `Default` impl, and `ServerResponse`. Added conditional blocks in `update_server_use_case.rs` and wired `health_check_timeout_seconds` in the `update_server` handler. Created `RestartDefaults` entity with default values, added trait methods and implementations using the `app_settings` table, added GET/PUT handlers with admin auth, and registered the route at `/api/v1/settings/restart-defaults`.

## Task Commits

1. **Task 1: DTOs + use cases** — `e9ec53d`
   - 3 new fields in UpdateServerRequest, Default impl, ServerResponse
   - Phase 57 conditional block in update_server_use_case.rs

2. **Task 2: Handler wiring** — `bff4db5`
   - `health_check_timeout_seconds` conditional wiring in server_handlers.rs update_server

3. **Task 3: Global defaults storage + API** — `47a328b`
   - RestartDefaults entity (max_attempts=5, cooldown=300)
   - SettingsRepository trait + PostgresSettingsRepository impl
   - GET/PUT handlers with admin auth
   - Route at /api/v1/settings/restart-defaults

## Files Modified

- `api/src/application/dto/server_dtos.rs` — 3 new fields in UpdateServerRequest, Default, ServerResponse
- `api/src/application/use_cases/update_server_use_case.rs` — Phase 57 conditional block
- `api/src/presentation/handlers/server_handlers.rs` — health_check_timeout_seconds wiring
- `api/src/domain/entities/settings.rs` — RestartDefaults struct + Default impl
- `api/src/domain/repositories/settings_repository.rs` — 2 new trait methods
- `api/src/infrastructure/repositories/postgres_settings_repository.rs` — 2 new impl methods
- `api/src/presentation/handlers/settings_handlers.rs` — get/save restart defaults handlers
- `api/src/presentation/routes/api_routes.rs` — restart-defaults route

## Verification

- ✅ `cargo check` passes with exit code 0
- ✅ All 3 fields in UpdateServerRequest, Default, ServerResponse
- ✅ Conditional blocks for all 3 fields in update use case
- ✅ health_check_timeout_seconds wired in server handler
- ✅ RestartDefaults entity with Default impl
- ✅ SettingsRepository + PostgresSettingsRepository with restart_defaults methods
- ✅ settings_handlers with GET/PUT + admin auth
- ✅ Route registered at /api/v1/settings/restart-defaults

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- ✅ SUMMARY.md exists at expected path
- ✅ All 8 files modified correctly
- ✅ `cargo check` passes
- ✅ All git commits found in log
