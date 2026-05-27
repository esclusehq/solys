---
phase: 32-server-templates-for-hobby-and-pro-plans
plan: "01"
subsystem: backend-api
tags: [template, game-server, backend]
dependency_graph:
  requires: []
  provides:
    - api/src/domain/server/template/model.rs
    - api/src/domain/server/template/repository.rs
    - api/src/presentation/handlers/template_handlers.rs
  affects:
    - api/src/domain/server/mod.rs
    - api/src/presentation/routes/api_routes.rs
tech_stack:
  added:
    - Template entity with game_type, variant, display_name fields
    - SqlxTemplateRepository with async DB operations
    - Template API endpoint GET /api/v1/templates
  patterns:
    - Fallback pattern for empty database (same as GameType)
    - Repository trait pattern with async_trait
    - SQL injection mitigation via parameterized queries
key_files:
  created:
    - api/src/domain/server/template/mod.rs (5 lines)
    - api/src/domain/server/template/model.rs (185 lines)
    - api/src/domain/server/template/repository.rs (114 lines)
    - api/src/presentation/handlers/template_handlers.rs (46 lines)
  modified:
    - api/src/domain/server/mod.rs (+4 lines)
    - api/src/presentation/handlers/mod.rs (+1 line)
    - api/src/presentation/routes/api_routes.rs (+2 lines)
decisions:
  - Used fallback pattern matching existing GameType entity pattern
  - Implemented TemplateRepository trait for testability
  - Added game_type filter query parameter for filtering
metrics:
  duration: ~5 min
  completed: 2026-05-04
  tasks: 5/5
  files: 4 created, 3 modified
---

# Phase 32 Plan 01: Server Templates for Hobby and Pro Plans

## Summary

Created backend template system to store pre-configured server templates per game type and variant. Implemented Template entity, repository with async DB operations, and API endpoint for listing available templates.

## One-Liner

Server template system with game type + variant support and fallback to hardcoded defaults when database is empty.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create template domain module | 916f496 | api/src/domain/server/template/mod.rs |
| 2 | Create template model/entity | 87cfce8 | api/src/domain/server/template/model.rs |
| 3 | Create template repository | b0b113e | api/src/domain/server/template/repository.rs |
| 4 | Register template module in server domain | 67c6ca6 | api/src/domain/server/mod.rs |
| 5 | Add template API routes | af9853c | api/src/presentation/handlers/template_handlers.rs, api_routes.rs |

### Fixes Applied

- **fix(32-01): use correct pool accessor in template handlers** (392acd7) - Fixed incorrect `db_pool()` method call to use `state.pool.clone()` following existing handler patterns.

## Verification

- Backend compiles: `cd api && cargo check` - PASSED
- Template module accessible: `use crate::domain::server::template::Template` - PASSED
- Routes registered: `/api/v1/templates` endpoint added - PASSED

## API Endpoints

- `GET /api/v1/templates` - List all templates
- `GET /api/v1/templates?game_type=minecraft` - Filter templates by game type

## Template Data

Fallback templates include:
- **Minecraft**: vanilla, paper, spigot, forge, fabric variants
- **Palworld**: default variant
- **Rust**: default variant
- **Valheim**: default variant

## Threat Model Compliance

- T-32-01 (Injection): Mitigated via parameterized queries in sqlx
- T-32-02 (Information Disclosure): Accept - templates are non-sensitive

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- [x] Backend compiles: cargo check
- [x] All 5 tasks committed individually
- [x] Summary created at .planning/phases/32-server-templates-for-hobby-and-pro-plans/32-01-SUMMARY.md