# Phase 58: Server, Plugin, and Modpack Templates - Context

**Gathered:** 2026-05-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a template system for game servers — pre-configured server templates (config + plugin/mod references), plugin/mod browser with CurseForge/Modrinth API integration, and modpack configuration — to simplify deployment. Users browse templates from a library and create pre-configured servers with one click.

</domain>

<decisions>
## Implementation Decisions

### Template Scope & Composition
- **D-01:** **Full stack templates.** Templates include server config values AND plugin/mod references. Not just config presets — a template represents a complete game server setup.
- **D-02:** **Creation-only application.** Templates are applied when creating a new server. Template config is copied (snapshot) into the server record. Existing servers are unaffected by template updates.
- **D-03:** **Browse then create.** Users browse templates from a template library. Each template has a "Create Server" button that pre-fills the creation form. No inline template selection during creation flow.

### Template Storage & Format
- **D-04:** **Database + JSONB storage.** Templates stored in a `templates` database table with a JSONB `config` column for flexible schema. Follows existing sqlx + sqlx::FromRow patterns.
- **D-05:** **Snapshot on apply.** Template configuration is copied into the server record at creation time. No live reference to the template template — updates to the template don't affect existing servers.
- **D-06:** **Organized by game type + sub-category.** Each template belongs to a game type (Minecraft, Palworld, etc.) with optional sub-categories (Vanilla, Paper, Forge, Fabric for Minecraft).

### Plugin/Modpack Sourcing
- **D-07:** **Mixed sourcing — URLs by default + upload option.** Templates can reference plugins/mods via direct download URLs. For custom plugins, admins can upload files to storage.
- **D-08:** **CurseForge + Modrinth API integration.** The platform integrates with CurseForge and Modrinth APIs to search mods, resolve versions, and auto-generate download URLs. API keys managed on platform settings.
- **D-09:** **Full mod browser included in this phase.** A dedicated mod browser page where users can search CurseForge/Modrinth, pick mods, and build collections. Mod collections can be bundled into templates as modpacks.

### Template Management
- **D-10:** **Admins + any user can create templates.** Any authenticated user can create and manage their own templates.
- **D-11:** **Private + public visibility.** Users can keep templates private or make them public. Admins can feature/promote public templates in the library.
- **D-12:** **Built-in official templates.** Platform ships with curated built-in templates for popular game types (Minecraft Vanilla, Paper, Fabric, Palworld, etc.). These are seeded via migration and cannot be deleted by users.

### The Agent's Discretion
- Specific template DB schema design (columns besides game_type, category, config JSONB)
- Template creation/edit UI layout and form fields
- Mod browser UI design (search, filter, pagination)
- Plugin installation mechanism during server deployment (agent-side task for downloading plugins)
- JSONB config schema for different game types
- Seeding strategy for built-in templates (migration SQL vs startup script)
- CurseForge/Modrinth API key management UX
- Template "forking" — user can clone a public template and modify their own copy

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Technology Stack & Architecture
- `.planning/codebase/STACK.md` — Tech stack (Rust Axum, React 19, Zustand, Tailwind CSS v4)
- `.planning/codebase/ARCHITECTURE.md` — Service layer architecture, clean architecture pattern
- `.planning/codebase/CONVENTIONS.md` — Coding conventions, naming, component patterns

### Prior Phase Context
- `.planning/phases/51-automasi-dns-cloudflare/51-CONTEXT.md` — Phase 58 depends on Phase 51 (Automasi DNS)
- `.planning/phases/57-auto-restart-policies/57-CONTEXT.md` — Settings tab UI pattern (Restart Policy section in Server Settings)
- `.planning/phases/56-auto-online-sleep-recovery/56-CONTEXT.md` — Settings tab pattern (Sleep & Wake section)

### Existing Server Infrastructure
- `api/src/domain/entities/server.rs` — Server entity (source of truth for field additions)
- `api/src/domain/server/model.rs` — Server model with sqlx::FromRow
- `api/src/domain/server/sqlx_repository.rs` — Server repository
- `api/src/infrastructure/repositories/postgres_server_repository.rs` — Legacy Server repository
- `api/src/application/dto/server_dtos.rs` — Server DTOs (CreateServerRequest, ServerResponse)
- `api/src/application/use_cases/create_server_use_case.rs` — Server creation flow
- `api/src/presentation/handlers/server_handlers.rs` — REST handlers for server CRUD

### Frontend Patterns
- `app/src/pages/servers/ServerManagerPage.jsx` — Server list page
- `app/src/hooks/useServers.js` — Server API hooks
- `app/src/pages/ServerDetails.jsx` — Server detail/settings page

### Agent Infrastructure
- `agent-core/crates/agent-task/src/dispatcher.rs` — Task dispatch for agent-side operations
- `agent-core/crates/agent-proto/src/lib.rs` — Task/result protocol definitions

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `useServers.js` — Existing API hook for server operations. Can be extended for template CRUD.
- `ServerDetails.jsx` Settings tab — Phase 56/57 pattern for config sections with toggle + inputs + save.
- `create_server_use_case.rs` — Server creation flow where template application hooks in.
- `server_dtos.rs::CreateServerRequest` — Can add optional `template_id` field.
- `rcon_server_executor.rs` — Existing plugin/mod handling if agent-side download is needed.

### Established Patterns
- Phase 56/57's Settings tab layout (toggle + number inputs + save button + toast) — use for template editing UI.
- DTO + use case + handler wiring for new entities (template CRUD).
- DB migration + entity + repository pattern for new tables.
- Seeding via SQL migration (built-in templates).

### Integration Points
- **Server creation flow** (`create_server_use_case.rs`): When `template_id` is provided, pre-fill server config from template before deployment.
- **Web Agent task dispatch**: If plugins/mods need to be downloaded during deployment, extend agent task protocol with download/install tasks.
- **New API endpoints**: `/api/v1/templates` CRUD, `/api/v1/templates/{id}/create-server`, `/api/v1/mods/search` (CurseForge/Modrinth proxy).
- **New frontend pages**: `/templates` (browse library), `/templates/create` (template editor), `/mods` (mod browser).
- **Settings page**: CurseForge/Modrinth API key management section.

</code_context>

<specifics>
## Specific Ideas

Follow Phase 56/57's Settings tab pattern for template editing UI. The template library should feel like a marketplace — featured templates at top, categorized browsing, search. The mod browser should support searching by name, filtering by game version, and showing mod details before adding to a collection.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 58-server-plugin-modpack-templates*
*Context gathered: 2026-05-30*
