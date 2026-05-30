---
phase: 58-server-plugin-modpack-templates
plan: 02
subsystem: api
tags: templates, crud, axum, serde, anyhow, sqlx

requires:
  - phase: 58-server-plugin-modpack-templates
    provides: Template domain entity, repository trait, SqlxTemplateRepository implementation
provides:
  - Template CRUD DTOs (CreateTemplateRequest, UpdateTemplateRequest, TemplateResponse, CreateServerFromTemplateRequest)
  - 6 template use cases (Create, List, Get, Update, Delete, Apply) with ownership/builtin enforcement
  - 6 REST handlers for full template CRUD + apply-template-to-server
  - Template route registration (GET+POST /api/v1/templates, GET+PUT+DELETE /api/v1/templates/:id, POST /api/v1/templates/:id/create-server)
  - Container wiring for template use cases
  - template_id field on CreateServerRequest for template-referenced server creation

affects:
  - 58-03 through 58-05 (frontend pages, hook integration depend on these endpoints)

tech-stack:
  added: []
  patterns:
    - Generic use case with Arc<dyn Repository> pattern for dependency injection
    - Error conversion from Box<dyn StdError> to anyhow::Error via explicit map_err
    - Template deep-clone pattern using serde_json::Value::clone() for D-05 snapshot

key-files:
  created:
    - api/src/application/dto/template_dtos.rs
    - api/src/application/use_cases/template_use_cases.rs
  modified:
    - api/src/presentation/handlers/template_handlers.rs (replaced 40-line handler with full 210-line CRUD)
    - api/src/presentation/routes/api_routes.rs (single GET → 4 route registrations)
    - api/src/bootstrap/container.rs (added 6 template use case fields + initialization)
    - api/src/application/dto/server_dtos.rs (added template_id field)
    - api/src/application/dto/mod.rs (added template_dtos module)
    - api/src/application/use_cases/mod.rs (added template_use_cases module)

key-decisions:
  - "Handler-level template-to-server integration: apply_template_to_server deep-clones template.config, constructs CreateServerRequest with merged config, and delegates to CreateServerUseCase"
  - "Template ownership enforced at use case level (Update/Delete verify user_id), not handler level"
  - "Explicit .map_err(|e| anyhow::anyhow!(\"{}\", e)) for all repository calls to convert Box<dyn StdError> to anyhow::Error"

requirements-completed: []

duration: 10 min
completed: 2026-05-30
---

# Phase 58: Server, Plugin, and Modpack Templates — Plan 02 Summary

**Full backend API layer for template CRUD: 4 DTOs, 6 use cases (with ownership/builtin enforcement), 6 REST handlers, route registration, container wiring, and CreateServerRequest.template_id integration point**

## Performance

- **Duration:** 10 min
- **Started:** 2026-05-30T20:51:27Z
- **Completed:** 2026-05-30T21:02:20Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Created 4 template DTO structs: CreateTemplateRequest, UpdateTemplateRequest, TemplateResponse (with From<Template> impl), CreateServerFromTemplateRequest
- Created 6 template use cases: Create (sets user ownership, private default), List (public + owned with dedup by game_type filter), Get (by ID), Update (ownership check + partial updates), Delete (ownership + builtin guard), Apply (deep-clone config for snapshot)
- Replaced single list_templates handler with 6 full handlers: list_templates, get_template, create_template, update_template, delete_template, apply_template_to_server
- apply_template_to_server handler fetches template, deep-clones config, applies user config_overrides, builds CreateServerRequest with merged values, creates server via CreateServerUseCase
- Registered 4 template routes: GET+POST /api/v1/templates, GET+PUT+DELETE /api/v1/templates/:id, POST /api/v1/templates/:id/create-server
- Registered 6 use cases in AppContainer with SqlxTemplateRepository dependency
- Added template_id: Option<Uuid> to CreateServerRequest for referential trail during template-based server creation
- Verified full compilation: cargo check passes with exit code 0 (no new errors)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create template DTOs and use cases** — `c57f23e` (feat)
2. **Task 3: Integrate template_id into CreateServer flow** — `e8201b4` (feat)
3. **Deviation: Fix repository error conversion in template use cases** — `f3a4168` (fix)
4. **Task 2: Extend template handlers and register routes + container** — `d867ae5` (feat)

## Files Created/Modified

- `api/src/application/dto/template_dtos.rs` — NEW: 4 DTO structs + From<Template> for TemplateResponse (111 lines)
- `api/src/application/use_cases/template_use_cases.rs` — NEW: 6 use case structs with ownership/builtin enforcement (199 lines)
- `api/src/presentation/handlers/template_handlers.rs` — REPLACED: 40-line single handler → 210-line full CRUD + apply handler (210 lines)
- `api/src/presentation/routes/api_routes.rs` — MODIFIED: single GET route → 4 route registrations (CRUD + apply)
- `api/src/bootstrap/container.rs` — MODIFIED: 6 template use case fields + initialization with SqlxTemplateRepository
- `api/src/application/dto/server_dtos.rs` — MODIFIED: added template_id: Option<Uuid> field
- `api/src/application/dto/mod.rs` — MODIFIED: added template_dtos module declaration
- `api/src/application/use_cases/mod.rs` — MODIFIED: added template_use_cases module declaration

## Decisions Made

- **Handler-level template-to-server integration:** apply_template_to_server handler deep-clones template.config via `serde_json::Value::clone()`, applies config_overrides from request payload, constructs a fully-populated CreateServerRequest with merged config values, and delegates to CreateServerUseCase. This avoids changing CreateServerUseCase's generic parameters.
- **Ownership at use case level:** Template ownership checks (`template.user_id == Some(user_id)`) happen in Update/Delete use cases, not handlers. This keeps security logic centralized in the domain layer.
- **Error conversion pattern:** Repository trait methods return `Box<dyn std::error::Error + Send + Sync>` which can't auto-convert to `anyhow::Error` (not Sized). All repository calls use explicit `.map_err(|e| anyhow::anyhow!("{}", e))?`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed repository error conversion in template use cases**
- **Found during:** Compilation check after Task 1
- **Issue:** `?` operator can't convert `Box<dyn std::error::Error + Send + Sync>` to `anyhow::Error` because `dyn StdError` doesn't implement `Sized`
- **Fix:** Replaced all `await?` on repository calls with `await.map_err(|e| anyhow::anyhow!("{}", e))?`
- **Files modified:** api/src/application/use_cases/template_use_cases.rs
- **Verification:** `cargo check -p backend` passes with exit code 0
- **Committed in:** `f3a4168` (separate fix commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Required for compilation — the mismatch between repository trait error type and anyhow::Result is a known pattern in the codebase

## Issues Encountered

- Repository trait error type (`Box<dyn std::error::Error + Send + Sync>`) requires explicit conversion to `anyhow::Error` — established pattern from Plan 01's repository but not documented in PATTERNS.md. Fixed with explicit map_err calls.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 8 template API endpoints implemented, compiled, and ready
- Container wiring complete — use cases accessible via AppContainer fields
- Frontend pages (TemplateLibraryPage, TemplateCreatePage, etc.) can use these endpoints directly
- Ready for 58-03 through 58-05: template library frontend pages, hooks, components

## Self-Check: PASSED

- ✅ template_dtos.rs has all 4 DTO structs (CreateTemplateRequest, UpdateTemplateRequest, TemplateResponse, CreateServerFromTemplateRequest)
- ✅ TemplateResponse has `From<Template>` impl
- ✅ template_use_cases.rs has 6 use case structs (Create, List, Get, Update, Delete, Apply)
- ✅ Ownership checking in Update/Delete (template.user_id == Some(user_id) → Forbidden)
- ✅ Built-in template deletion guard in Delete use case
- ✅ template_handlers.rs has 6 exported handler functions
- ✅ api_routes.rs registers all template CRUD routes
- ✅ container.rs has all 6 template use case fields + initialization
- ✅ server_dtos.rs has template_id: Option<Uuid> with #[serde(default)]
- ✅ `cargo check -p backend` exit code 0 (no compilation errors)
- ✅ 8 files created/modified properly tracked
- ✅ No threat surface outside documented threat_model

---

*Phase: 58-server-plugin-modpack-templates*
*Completed: 2026-05-30*
