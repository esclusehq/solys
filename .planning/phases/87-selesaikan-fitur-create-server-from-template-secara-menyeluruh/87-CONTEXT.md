# Phase 87: Selesaikan fitur 'Create server from template' secara menyeluruh - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete the end-to-end flow of creating a game server from a template (game server, plugin, modpack). The backend API (`POST /templates/:id/create-server`) and template CRUD infrastructure already exist from Phase 58. This phase fills the gaps: a dedicated template detail page, the server creation modal with config overrides, resource-aware warnings, and plugin/modpack dependency preview during creation.

No new backend endpoints — uses existing `POST /templates/:id/create-server`. Purely frontend work + deployment pipeline integration.
</domain>

<decisions>
## Implementation Decisions

### Entry Point — Template Detail Page
- **D-01:** Clicking "Create Server" on a TemplateCard navigates to a dedicated template detail page at `/templates/:id`
- **D-02:** The detail page shows: template name, description, version badge, game type, config preview (default RAM, port, version), resource requirements, plugin/modpack dependency list, and a prominent "Create Server" button
- **D-03:** "Create Server" button opens a configuration modal where user customizes settings before deploying

### Config Overrides — What Users Can Change
- **D-04:** User can override: server name, RAM allocation, DISK allocation, target agent/node selection, Online/Offline mode toggle, World seed, player limit
- **D-05:** All overridable fields are pre-filled from template defaults. Template defaults are the starting point.
- **D-06:** Fields not listed above (e.g., game type, port strategy) are locked by the template — user cannot change them

### Resource Enforcement — Plan/Node Compatibility
- **D-07:** When template requires more resources than the target agent node can provide, show a warning: "This template requires X GB RAM. Your computer/VPS/homelab (where the agent is located) can't handle it"
- **D-08:** User can still proceed with reduced resources despite the warning — not a hard block
- **D-09:** Warning is about the **agent node's available capacity**, not the billing plan

### Plugin/Modpack Auto-Install
- **D-10:** Before creating, show the user a preview of all plugin/modpack dependencies that will be installed (with versions)
- **D-11:** User confirms the dependency list before deployment proceeds
- **D-12:** Installation happens during the server creation pipeline as a progress step — no post-creation manual step needed for template dependencies

### the agent's Discretion
- Exact visual layout of the template detail page (config preview, dependency list presentation)
- UI design of the creation modal (step form vs single scrollable form)
- Progress indicators during deployment with dependency installation
- Warning message wording and visual treatment for resource mismatches
- How template version and usage_count are displayed on detail page

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend — Template API
- `api/src/presentation/handlers/template_handlers.rs` — `POST /templates/:id/create-server` handler (apply_template_to_server, lines 144-230)
- `api/src/application/dto/template_dtos.rs` — `CreateServerFromTemplateRequest` DTO (fields: name, node_id, game_type, minecraft_version, ram_mb, max_players, config_overrides)
- `api/src/application/use_cases/template_use_cases.rs` — Template CRUD use cases + ApplyTemplateUseCase
- `api/src/application/use_cases/create_server_use_case.rs` — CreateServerUseCase that builds Server from CreateServerRequest

### Frontend — Existing Template UI
- `app/src/api/templatesApi.js` — Frontend API client: list, get, create, update, delete, createServer
- `app/src/hooks/useTemplateLibrary.js` — useTemplateLibrary hook + createServerFromTemplate helper
- `app/src/components/TemplateCard.jsx` — Existing template card with "Create Server" link (currently links to `/templates/:id`)
- `app/src/features/server/CreateServerModal.jsx` — Existing create modal with template selection, filtering, auto-fill
- `app/src/features/server/ServerOnboardingWizard.jsx` — Phase 83 4-step onboarding wizard (reference for creation modal UX)
- `app/src/hooks/useModpackTemplates.js` — Modpack template hook (dependency references)
- `app/src/hooks/usePluginTemplates.js` — Plugin template hook (dependency references)

### Frontend — Layout/Theme
- `app/src/index.css` — Cosmic theme CSS variables (glass-panel, cosmic borders, status badge patterns)
- `app/src/pages/templates/TemplateLibraryPage.jsx` — Existing template library page (routes for reference)

### API Routes
- `api/src/presentation/routes/api_routes.rs` — Route mounting for template handlers

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TemplateCard.jsx` — Can be reused on the detail page header or as a summary card
- `CreateServerModal.jsx` — Contains template selection/filtering logic, game type → version mapping, server creation API call. Reference for config modal structure.
- `templatesApi.createServer(id, data)` — Ready-to-use API call for template-based server creation
- `createServerFromTemplate(templateId, data)` — Already exported from useTemplateLibrary.js
- `ServerOnboardingWizard.jsx` — Reference for multi-step creation flow pattern
- Status badge patterns, glass-panel containers, cosmic theme from Phase 75-84

### Established Patterns
- Cosmic theme: glass-panel containers, cosmic borders, cyan accent buttons (Phases 75-84)
- Modal pattern: Zustand-based open/close state, glass-panel container, backdrop blur
- Server creation: API call returns created server, redirect to server detail page
- Loading states: skeleton loaders for data, spinner for actions

### Integration Points
- TemplateCard "Create Server" link: currently `<Link to={\`/templates/${template.id}\`}>` in TemplateCard.jsx line 83 — needs a real detail page at that route
- New route needed in App.jsx: `/templates/:id` → template detail page
- Creation modal needs to call `templatesApi.createServer(id, overrideData)` to use the existing backend endpoint
- Plugin/modpack dependencies may need a preview UI that shows version + what will be installed

</code_context>

<specifics>
## Specific Ideas

- Config override fields: server name (text), RAM (slider/select with MB units), DISK (slider/select), agent/node (dropdown of user's registered nodes), Online/Offline mode (toggle), World seed (text, optional), player limit (number)
- Resource warning should reference the specific agent node name and its available resources
- Dependency preview should show plugin/mod name, version, and game version compatibility
- After successful creation, redirect to the new server's detail page

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope
</deferred>

---

*Phase: 87-selesaikan-fitur-create-server-from-template-secara-menyeluruh*
*Context gathered: 2026-06-17*
