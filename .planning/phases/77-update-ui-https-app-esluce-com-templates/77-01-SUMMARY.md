---
phase: 77-update-ui-https-app-esluce-com-templates
plan: 01
subsystem: api
tags: [rust, sqlx, template, domain-model, dto]
requires: []
provides:
  - Template domain model with version and usage_count fields
  - TemplateResponse DTO with version and usage_count
  - list_public_templates SQL with LEFT JOIN usage_count computation
affects: [77-update-ui-https-app-esluce-com-templates]
tech-stack:
  added: []
  patterns:
    - "LEFT JOIN subquery for computed aggregate fields"
    - "#[sqlx(default)] for fields not selected by all queries"
key-files:
  created: []
  modified:
    - api/src/domain/server/template/model.rs
    - api/src/application/dto/template_dtos.rs
    - api/src/application/use_cases/template_use_cases.rs
    - api/src/domain/server/template/repository.rs
    - migration/src/domain/server/template/model.rs
    - migration/src/application/dto/template_dtos.rs
    - migration/src/application/use_cases/template_use_cases.rs
key-decisions:
  - "Used #[sqlx(default)] on usage_count so other SQL queries (list_templates, get_template, etc.) continue working without modification"
  - "LEFT JOIN subquery with COALESCE to compute usage_count from servers table — templates with 0 servers show 0, not NULL"
  - "version: Option<String> enables sqlx auto-default to None for queries that don't SELECT the column (no #[sqlx(default)] needed)"
patterns-established:
  - "Computed aggregate fields: LEFT JOIN subquery + COALESCE for default zero"
  - "Non-breaking field addition: #[sqlx(default)] on non-optional fields, Option<T> on optional fields"
requirements-completed: [D-02]
duration: 6 min
completed: 2026-06-15
---

# Phase 77 Plan 01: Add version and usage_count to Template model & DTO

**version: Option<String> and usage_count: i64 added to Template domain model and TemplateResponse DTO across both api/ and migration/ Rust codebases, with LEFT JOIN usage_count computation in list_public_templates SQL**

## Performance

- **Duration:** 6 min
- **Started:** 2026-06-15T03:36:00Z
- **Completed:** 2026-06-15T03:42:01Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added `version: Option<String>` and `usage_count: i64` (with `#[sqlx(default)]`) to Template struct in both codebases
- Updated all fallback template constructors (11 in api, 9 in migration) with `version: None, usage_count: 0`
- Added matching fields to `TemplateResponse` DTO in both codebases, plus `From` impl mappings
- Updated `CreateTemplateUseCase::execute` in both codebases with `version: None, usage_count: 0`
- Replaced `list_public_templates` SQL with `LEFT JOIN` subquery computing `usage_count` from `servers` table
- `cargo check` passes in both `api/` and `migration/` with zero errors

## Task Commits

No git operations performed (instructed to skip). Changes were applied directly to files.

## Files Created/Modified

- `api/src/domain/server/template/model.rs` - Added `version: Option<String>`, `usage_count: i64` with `#[sqlx(default)]`; updated 11 fallback constructors
- `api/src/application/dto/template_dtos.rs` - Added `version`, `usage_count` to `TemplateResponse` and `From` impl
- `api/src/application/use_cases/template_use_cases.rs` - Added `version: None, usage_count: 0` to `CreateTemplateUseCase`
- `api/src/domain/server/template/repository.rs` - `list_public_templates` SQL changed to `LEFT JOIN` subquery with `COALESCE`
- `migration/src/domain/server/template/model.rs` - Same model changes as api (9 fallback constructors)
- `migration/src/application/dto/template_dtos.rs` - Same DTO and From impl changes as api
- `migration/src/application/use_cases/template_use_cases.rs` - Same use case change as api

## Decisions Made

- Used `#[sqlx(default)]` on `usage_count` (non-optional) so all other SQL queries (`list_templates`, `get_template`, `get_template_by_id`, `create_template`, `update_template`, `list_templates_by_user`) continue working without modification — sqlx defaults to 0 when column is absent from SELECT.
- `version: Option<String>` needs no `#[sqlx(default)]` — sqlx auto-defaults optional fields to `None` for queries that don't SELECT the column.
- LEFT JOIN subquery with `COALESCE(usage_stats.usage_count, 0)` ensures templates with zero servers show 0 instead of NULL in the API response.
- `WHERE template_id IS NOT NULL` in subquery filters out servers not created from templates.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## Next Phase Readiness

- Template model and DTO fields are ready for Plan 02 (frontend card enrichment)
- `list_public_templates` now returns `version` and `usage_count` — the API response shape is complete
- All other template queries (single lookup, user-specific, create, update) return `version: None` and `usage_count: 0` via sqlx defaults, which is correct behavior until those queries are explicitly updated

---

*Phase: 77-update-ui-https-app-esluce-com-templates*
*Completed: 2026-06-15*
