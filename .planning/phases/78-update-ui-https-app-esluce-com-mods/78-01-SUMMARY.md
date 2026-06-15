---
phase: 78-update-ui-https-app-esluce-com-mods
plan: 01
type: execute
subsystem: backend
tags:
  - plugins
  - modrinth
  - dto
  - endpoint
  - game-versions
  - api
dependency_graph:
  requires: []
  provides:
    - "PluginSearchResult DTO: author, latest_version fields"
    - "PluginVersionDto DTO: date_published field"
    - "ModrinthProject struct: author, latest_version fields"
    - "ModrinthVersion struct: date_published field"
    - "GET /api/v1/plugins/game-versions endpoint"
  affects:
    - "api/src/ (5 files)"
    - "migration/src/ (5 files)"
tech_stack:
  added: []
  patterns:
    - "reqwest::Client created inline for proxy-to-Modrinth calls"
    - "serde_json::Value used as response type for proxied tag API"
key_files:
  created: []
  modified:
    - api/src/infrastructure/external_services/modrinth_client.rs
    - api/src/application/dto/plugin_dtos.rs
    - api/src/application/use_cases/plugin_use_cases.rs
    - api/src/presentation/handlers/plugin_handlers.rs
    - api/src/presentation/routes/api_routes.rs
    - migration/src/infrastructure/external_services/modrinth_client.rs
    - migration/src/application/dto/plugin_dtos.rs
    - migration/src/application/use_cases/plugin_use_cases.rs
    - migration/src/presentation/handlers/plugin_handlers.rs
    - migration/src/presentation/routes/api_routes.rs
decisions: []
metrics:
  duration: "~15 min"
  completed_date: "2026-06-15"
---

# Phase 78 Plan 01: Backend DTO fields + game-versions endpoint

**One-liner:** Added `author` and `latest_version` to `ModrinthProject`/`PluginSearchResult`, `date_published` to `ModrinthVersion`/`PluginVersionDto`, updated use case mappings, and created `GET /api/v1/plugins/game-versions` handler + route — applied identically to both `api/src/` and `migration/src/` codebases.

## Tasks Completed

| Task | Name                                                                     | Files Modified                          |
| ---- | ------------------------------------------------------------------------ | --------------------------------------- |
| 1    | Add author, latest_version, date_published to structs + update mappings  | 6 files (3 per codebase)                |
| 2    | Create GET /api/v1/plugins/game-versions handler + route                 | 4 files (2 per codebase)                |

### Task 1 Details

**Change Set A — `modrinth_client.rs`:**
- `ModrinthProject`: added `pub author: String`, `pub latest_version: String`
- `ModrinthVersion`: added `pub date_published: String`

**Change Set B — `plugin_dtos.rs`:**
- `PluginSearchResult`: added `pub author: String`, `pub latest_version: String`
- `PluginVersionDto`: added `pub date_published: String`

**Change Set C — `plugin_use_cases.rs`:**
- `SearchPluginsUseCase::execute`: added `author: hit.author`, `latest_version: hit.latest_version`
- `GetPluginVersionsUseCase::execute`: added `date_published: v.date_published`

### Task 2 Details

**Handler** (`get_game_versions` in `plugin_handlers.rs`):
- Proxies `https://api.modrinth.com/v2/tag/game_version` using a fresh `reqwest::Client`
- Returns `serde_json::Value` wrapped in `ApiResponse::success()`
- Uses `anyhow::anyhow!` for error conversion (existing pattern)

**Route** (in `api_routes.rs`):
- `.route("/api/v1/plugins/game-versions", get(...get_game_versions))`
- Registered alongside existing plugin routes

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

| Check                                                    | Result |
| -------------------------------------------------------- | ------ |
| `api/src` — `pub author: String` in DTO                  | ✅     |
| `api/src` — `pub latest_version: String` in DTO          | ✅     |
| `api/src` — `pub date_published: String` in DTO          | ✅     |
| `api/src` — `get_game_versions` handler                  | ✅     |
| `api/src` — `game-versions` route                        | ✅     |
| `api/src` — `pub author: String` in ModrinthProject      | ✅     |
| `api/src` — `author: hit.author` in use case             | ✅     |
| `migration/src` — all fields mirror `api/src`             | ✅     |
| `cargo check` in `api/` — no errors (80 pre-existing warnings) | ✅     |
| `cargo check` in `migration/` — no errors (78 pre-existing warnings) | ✅     |
| `diff -q` — all 5 file pairs identical                       | ✅     |

## Self-Check: PASSED

All 10 modified files confirmed modified with correct content. Both `cargo check` runs passed with exit code 0. Both codebases confirmed identical via `diff -q`.
