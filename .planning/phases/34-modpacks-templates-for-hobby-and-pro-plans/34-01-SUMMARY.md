---
phase: 34-modpacks-templates-for-hobby-and-pro-plans
plan: 01
subsystem: Backend API
tags: [modpack, template, curseforge, modrinth, minecraft]
dependency_graph:
  requires: []
  provides: [modpack-templates-api]
  affects: [server-creation, plugin-system]
tech_stack:
  added: []
  patterns: [repository-pattern, fallback-pattern, plan-tier-filtering]
key_files:
  created:
    - api/src/domain/server/modpack_template/mod.rs
    - api/src/domain/server/modpack_template/model.rs
    - api/src/domain/server/modpack_template/repository.rs
    - api/src/presentation/handlers/modpack_template_handlers.rs
  modified:
    - api/src/domain/server/mod.rs
    - api/src/presentation/handlers/mod.rs
    - api/src/presentation/routes/api_routes.rs
decisions: []
metrics:
  duration: 120
  completed_date: "2026-05-03T17:59:34Z"
  tasks: 5
  files: 7
---

# Phase 34 Plan 01: Modpacks Templates for Hobby and Pro Plans Summary

**One-liner:** Backend modpack template system with CurseForge/Modrinth project IDs for Minecraft modpacks, filtered by user plan tier (Hobby/Pro/Enterprise).

## Overview

Created backend modpack template system to store pre-configured modpack lists per game type. The system includes:
- ModpackTemplate entity with CurseForge/Modrinth project IDs and versions
- Repository with SQLx async operations and fallback to hardcoded templates
- API endpoint at GET /api/v1/modpack-templates
- 8 popular Minecraft modpacks (Direwolf 20, Enigmatica 2, All The Mods, StoneBlock, etc.)
- Plan tier filtering (Hobby/Pro/Enterprise access levels)

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create modpack_template domain module | 4c0b9f8 | api/src/domain/server/modpack_template/mod.rs |
| 2 | Create modpack_template model/entity | 4c0b9f8 | api/src/domain/server/modpack_template/model.rs |
| 3 | Create modpack_template repository | 4c0b9f8 | api/src/domain/server/modpack_template/repository.rs |
| 4 | Register modpack_template module in server domain | 4c0b9f8 | api/src/domain/server/mod.rs |
| 5 | Add modpack template API routes | 4c0b9f8 | api/src/presentation/routes/api_routes.rs |

## Implementation Details

### ModpackTemplate Model
- `id`: Uuid (primary key)
- `game_type`: String (minecraft only for now)
- `display_name`: String (e.g., "Direwolf 20", "Enigmatica 2")
- `description`: Option<String>
- `source`: String (curseforge or modrinth)
- `project_id`: String (CurseForge/Modrinth project ID)
- `version_id`: String (specific version ID)
- `version_name`: String (e.g., "1.12.2")
- `mod_count`: i32 (number of mods in pack)
- `image_url`: Option<String>
- `min_plan`: String (hobby, pro, enterprise - who can use this template)
- `is_active`: bool
- `created_at`, `updated_at`: NaiveDateTime

### Fallback Templates Included
1. Direwolf 20 (1.20.1, 185 mods, Pro)
2. Enigmatica 2 Expertsky (1.12.2, 280 mods, Pro)
3. All The Mods 9 (1.20.1, 180 mods, Pro)
4. StoneBlock 3 (1.18.2, 180 mods, Hobby)
5. Revelation (1.12.2, 220 mods, Hobby)
6. Better Minecraft (1.20.1, 95 mods, Hobby)
7. The Simple Modpack (1.12.2, 45 mods, Hobby)
8. RLCraft (1.12.2, 150 mods, Pro)

### API Endpoint
- GET /api/v1/modpack-templates - list all modpack templates
- GET /api/v1/modpack-templates?game_type=minecraft - filter by game type

Plan tier filtering happens via repository's `list_by_plan(plan)` method which filters templates based on user plan tier.

## Verification

- [x] Backend compiles: `cargo check` passes
- [x] ModpackTemplate module accessible: `use crate::domain::server::modpack_template::ModpackTemplate`
- [x] Routes registered: check router includes /api/v1/modpack-templates
- [x] Plan check available: `fallback_by_plan()` and `list_by_plan()` methods implemented

## Threats Addressed

| Threat ID | Category | Component | Status |
|-----------|----------|-----------|--------|
| T-34-01 | Injection | modpack_template query | Mitigated via parameterized queries |
| T-34-02 | Authorization | template endpoint | Accept - plan tier check via repository |
| T-34-03 | Information Disclosure | template endpoint | Accept - non-sensitive data |

## Deviations from Plan

### Auto-fixed Issues

None - plan executed exactly as written.

## Self-Check

- [x] PASSED: All created files exist
- [x] PASSED: Commit 4c0b9f8 exists
- [x] PASSED: Module compiles without errors

## Dependencies

None - this plan is self-contained.

## Next Steps

This plan provides the backend API. The corresponding frontend integration would:
1. Add frontend component to display modpack selector during server creation
2. Call GET /api/v1/modpack-templates when game type supports mods
3. Filter displayed modpacks based on user's plan tier
4. Download modpack files using CurseForge/Modrinth APIs during server provision

## Completion Format

**Plan:** 34-modpacks-templates-for-hobby-and-pro-plans-01
**Tasks:** 5/5
**SUMMARY:** .planning/phases/34-modpacks-templates-for-hobby-and-pro-plans/34-01-SUMMARY.md

**Commits:**
- 4c0b9f8: feat(34-modpacks-templates): add modpack template system for hobby and pro plans

**Duration:** ~2 minutes