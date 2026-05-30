---
phase: 58-server-plugin-modpack-templates
plan: 03
subsystem: api
tags: curseforge, modrinth, external-api, settings, api-keys

requires:
  - phase: 58-server-plugin-modpack-templates
    provides: ModrinthClient pattern, SettingsRepository trait, API route registration pattern
provides:
  - CurseForgeClient for search, version resolution, and file download URL retrieval (optional API key)
  - 4 settings handlers for Modrinth and CurseForge API key management (admin-only)
  - Settings route registration for modrinth-api-key and curseforge-api-key
  - SettingsRepository trait methods + PostgresSettingsRepository impl for API key persistence
affects:
  - 58-04 through 58-05 (frontend pages, plugin search handlers)

tech-stack:
  added: []
  patterns:
    - Settings repository pattern for API key storage (app_settings table, JSON-serialized string)
    - Admin-only GET/PUT handler pattern returning api_key_set: bool (masking actual key)

key-files:
  created:
    - api/src/infrastructure/external_services/curseforge_client.rs
  modified:
    - api/src/infrastructure/external_services/mod.rs
    - api/src/presentation/handlers/settings_handlers.rs
    - api/src/domain/repositories/settings_repository.rs
    - api/src/infrastructure/repositories/postgres_settings_repository.rs
    - api/src/presentation/routes/api_routes.rs

key-decisions:
  - "API keys stored as raw strings in app_settings table using serde_json serialization (consistent with existing settings pattern)"
  - "GET handlers return api_key_set: bool only — never expose actual key values (T-58-09 mitigation, consistent with S3 config secret_key pattern)"
  - "SettingsRepository trait methods use anyhow::Result<String> for get and anyhow::Result<()> for save (same pattern as existing settings)"
  - "POST/PUT handlers accept raw JSON payload { \"api_key\": \"...\" } — no dedicated DTO struct needed for single-field key settings"

requirements-completed: []

duration: 12 min
completed: 2026-05-31
---

# Phase 58: Server, Plugin, and Modpack Templates — Plan 03 Summary

**CurseForge API client (search, file resolution, download URL) + admin-only settings endpoints for Modrinth and CurseForge API key configuration, persisted via SettingsRepository trait**

## Performance

- **Duration:** 12 min
- **Started:** 2026-05-31T12:15:00Z
- **Completed:** 2026-05-31T12:27:00Z
- **Tasks:** 3
- **Files modified:** 6 (1 created, 5 modified)

## Accomplishments

- Created `CurseForgeClient` with `search_mods`, `get_mod_files`, `get_file_download_url` methods following `ModrinthClient` pattern
- CurseForgeClient accepts optional API key (set via constructor or `set_api_key()`), used as `x-api-key` header
- Custom `urlencoding()` helper for query parameter percent-encoding (no external dependency)
- Response types: `CurseForgeSearchResponse`, `CurseForgeMod`, `CurseForgeFile`, `CurseForgePagination`, `CurseForgeFileResponse`
- Added 4 settings handler functions: `get_modrinth_api_key`, `save_modrinth_api_key`, `get_curseforge_api_key`, `save_curseforge_api_key`
- All handlers enforce `user.is_admin()` check (403 Forbidden for non-admin users)
- GET handlers return `api_key_set: bool` only — never expose actual key values (T-58-09 mitigation)
- PUT handlers accept `{ "api_key": "..." }` JSON payload and persist via `settings_repository`
- Extended `SettingsRepository` trait with 4 new methods: `get_modrinth_api_key`, `save_modrinth_api_key`, `get_curseforge_api_key`, `save_curseforge_api_key`
- Implemented all 4 methods in `PostgresSettingsRepository` using `app_settings` table pattern
- Registered `GET+PUT /api/v1/settings/modrinth-api-key` and `GET+PUT /api/v1/settings/curseforge-api-key` routes in API router
- Verified full compilation: `cargo check` passes with exit code 0 (no new errors)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create CurseForgeClient for mod search and version resolution** — `d3d02eb` (feat)
2. **Task 2: Add Modrinth/CurseForge API key settings handlers** — `4362168` (feat)
3. **Task 3: Register settings routes for modrinth-api-key and curseforge-api-key** — `bb8bfa1` (feat)

## Files Created/Modified

### Created
- `api/src/infrastructure/external_services/curseforge_client.rs` — CurseForge API v1 client with search, version resolution, and file download URL retrieval (194 lines)

### Modified
- `api/src/infrastructure/external_services/mod.rs` — Added `pub mod curseforge_client;` declaration
- `api/src/presentation/handlers/settings_handlers.rs` — Added 4 admin-only handler functions (get/save for modrinth and curseforge API keys)
- `api/src/domain/repositories/settings_repository.rs` — Extended trait with 4 new methods for API key persistence
- `api/src/infrastructure/repositories/postgres_settings_repository.rs` — Implemented 4 new methods using `app_settings` table pattern
- `api/src/presentation/routes/api_routes.rs` — Registered `modrinth-api-key` and `curseforge-api-key` routes

## Decisions Made

- **API key storage pattern:** Keys stored as raw strings in `app_settings` table using `serde_json::to_value(api_key)` — consistent with existing settings pattern (S3 config, Cloudflare config, restart defaults).
- **Key masking (T-58-09):** GET handlers return `api_key_set: bool` only, never the actual key value. Consistent with `secret_key_set` pattern in S3 config handler.
- **No DTO struct:** Single-field `{ "api_key": "..." }` payload parsed inline via `payload.get("api_key").and_then(|v| v.as_str())` — avoids unnecessary DTO for one field.
- **No `percent_encoding` dependency:** Custom `urlencoding()` function reimplements basic percent-encoding inline to avoid adding an external dependency for simple URL encoding.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Threat Surface Scan

No threat surface outside documented `<threat_model>`:
- T-58-09 (Information Disclosure): Mitigated — GET handlers only return `api_key_set: bool`, never the actual key
- T-58-10 (Spoofing): Accepted — API key transmitted via HTTPS with reqwest cert validation
- T-58-11 (Elevation of Privilege): Mitigated — all handlers enforce `user.is_admin()` with 403 Forbidden

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CurseForgeClient ready for use in plugin search/version endpoints (planned for later plans)
- Settings endpoints for both Modrinth and CurseForge API keys operational
- Frontend settings pages can consume these endpoints for API key configuration
- Ready for 58-04 (frontend integration for template and plugin settings)

## Self-Check: PASSED

- ✅ curseforge_client.rs exists with `pub struct CurseForgeClient`, `new()`, `search_mods`, `get_mod_files`, `get_file_download_url`
- ✅ All 5 response types defined (CurseForgeSearchResponse, CurseForgeMod, CurseForgeFile, CurseForgePagination, CurseForgeFileResponse)
- ✅ Search accepts: query, game_version, mod_loader, class_id, offset, sort
- ✅ API key set via constructor or `set_api_key()`, used in `x-api-key` header
- ✅ 4 handler functions with admin-only checks
- ✅ GET handlers return `api_key_set: bool`
- ✅ PUT handlers accept `{ "api_key": "..." }` payload
- ✅ SettingsRepository trait has all 4 methods
- ✅ Routes registered for modrinth-api-key and curseforge-api-key
- ✅ `cargo check` exit code 0 (no compilation errors)
- ✅ 3 commits found in git log with proper `feat(58-03)` format
- ✅ No threat surface outside documented threat_model

---

*Phase: 58-server-plugin-modpack-templates*
*Completed: 2026-05-31*
