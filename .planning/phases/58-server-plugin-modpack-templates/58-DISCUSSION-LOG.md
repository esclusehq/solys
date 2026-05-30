# Phase 58: Server, Plugin, and Modpack Templates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-30
**Phase:** 58-server-plugin-modpack-templates
**Areas discussed:** Template scope, Template storage & format, Plugin/Modpack sourcing, Template management

---

## Template Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Config only | Templates set server settings only. Users add plugins/mods separately. | |
| Config + plugins/mods (full stack) | Templates include config values AND reference plugin/mod lists. One-click full setup. | ✓ |
| | | |
| Creation only | Templates applied when creating a new server. Simpler — no merge logic. | ✓ |
| Creation + existing servers | Templates can also be applied to running/stopped servers. Needs diff/merge logic. | |
| | | |
| Step in creation wizard | During server creation flow, user selects a template before proceeding. | |
| Separate template browser page | Dedicated page to browse/manage templates. | |
| Both — browse then create | Users browse templates from a library. Each template has 'Create Server' button. | ✓ |

**User's choice:** Full stack, creation only, browse then create
**Notes:** Templates are snapshots applied at creation time. Library pattern with direct 'Create Server' action.

---

## Template Storage & Format

| Option | Description | Selected |
|--------|-------------|----------|
| Database + JSONB | Templates stored in DB with JSONB config column. Easy to query/filter. | ✓ |
| YAML files on disk | Templates stored as .yml files in a templates/ directory. | |
| | | |
| Snapshot on apply | Template config copied into server record at creation. Independent from template. | ✓ |
| Live reference | Servers keep reference to template. Updating template updates all servers. | |
| | | |
| By game type + sub-category | Each template belongs to a game type with optional sub-categories. | ✓ |
| Flat list with tags | No hierarchy. Templates searched/filtered by tags. | |

**User's choice:** DB + JSONB, snapshot on apply, by game type + sub-category
**Notes:** Follows existing DB migration + sqlx patterns. Categories: game_type (Minecraft, Palworld) + sub_category (Vanilla, Paper, Forge).

---

## Plugin/Modpack Sourcing

| Option | Description | Selected |
|--------|-------------|----------|
| URL-based references | Template stores URLs to plugin/mod files. System downloads during deployment. | |
| Uploaded files | Admins upload plugin/mod jar files to storage. Templates reference uploaded files. | |
| Mixed — URLs + upload | URLs for common sources, with upload fallback for custom plugins. | ✓ |
| | | |
| Generic URL only | Accept any direct download URL. No API integration needed. | |
| API integration for CurseForge/Modrinth | Search, list versions, auto-resolve download URLs via APIs. | ✓ |
| | | |
| URL resolution only | Use API to resolve download URLs for specific mod/version within templates. No search UI. | |
| Full mod browser | Dedicated mod browser with CurseForge/Modrinth search. Pick mods, build collections. | ✓ |

**User's choice:** Mixed sourcing, CurseForge+Modrinth API, full mod browser included
**Notes:** Full mod browser is part of this phase scope despite being substantial. API keys managed in platform settings.

---

## Template Management

| Option | Description | Selected |
|--------|-------------|----------|
| Admins only | Only platform admins can create/edit templates. | |
| Admins + any user | Any user can create templates. Users can share publicly or keep private. | ✓ |
| | | |
| Private + public collections | Admins feature templates. Users can have private or public templates. | ✓ |
| Private only | Templates are per-user. No public library. | |
| | | |
| Ship with built-in templates | Platform ships with curated templates for popular game types. | ✓ |
| No built-in templates | All templates are user-created. No pre-made ones. | |

**User's choice:** Anyone can create, private+public visibility, built-in templates shipped
**Notes:** Built-in templates seeded via migration, not deletable by users. Public template library with featured/curated section.

---

## The Agent's Discretion

- Template DB schema design (columns beyond game_type, category, config JSONB)
- Template editor UI layout
- Mod browser UI (search, filter, pagination, detail view)
- Plugin download mechanism during deployment (agent-side or API-side)
- JSONB config schema per game type
- Built-in template seeding strategy (migration vs startup)
- API key management UX for CurseForge/Modrinth
- Template forking/cloning UX

## Deferred Ideas

None — discussion stayed within phase scope.
