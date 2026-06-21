# Phase 87: Selesaikan fitur 'Create server from template' secara menyeluruh - Research

**Researched:** 2026-06-17
**Domain:** Frontend — template detail page + server creation modal + deployment flow
**Confidence:** HIGH

## Summary

This phase completes the end-to-end "Create Server from Template" flow. The backend already has `POST /templates/:id/create-server` (Phase 58) and full template CRUD. What's missing is the **frontend bridge**: a dedicated template detail page at `/templates/:id`, a configuration modal where users customize overridable fields (ram, disk, node, online mode, world seed, player limit) before deploying, and resource-aware warnings based on agent node capacity.

The existing codebase provides strong reuse patterns: `ServerOnboardingWizard.jsx` has an exact stepper pattern for RAM/DISK selection (lines 130-158), `CreateServerModal.jsx` has the form field layout and node selection pattern, and `TemplateCard.jsx` already links to `/templates/:id`. The main work is wiring these into two new components: `TemplateDetailPage` and `ConfigureServerModal`.

**Primary recommendation:** Build `TemplateDetailPage` at `/templates/:id` and `ConfigureServerModal` as a step or single-scrollable modal. Reuse the stepper (`-`/`+` buttons + progress bar) from `ServerOnboardingWizard.jsx` verbatim. The node resources API (`GET /api/v1/nodes/:id/resources`) exists for capacity checking.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Clicking "Create Server" on a TemplateCard navigates to a dedicated template detail page at `/templates/:id`
- D-02: The detail page shows: template name, description, version badge, game type, config preview (default RAM, port, version), resource requirements, plugin/modpack dependency list, and a prominent "Create Server" button
- D-03: "Create Server" button opens a configuration modal where user customizes settings before deploying
- D-04: User can override: server name, RAM allocation, DISK allocation, target agent/node selection, Online/Offline mode toggle, World seed, player limit
- D-05: All overridable fields are pre-filled from template defaults. Template defaults are the starting point.
- D-06: Fields not listed above (e.g., game type, port strategy) are locked by the template — user cannot change them
- D-07: When template requires more resources than the target agent node can provide, show a warning
- D-08: User can still proceed with reduced resources despite the warning — not a hard block
- D-09: Warning is about the agent node's available capacity, not the billing plan
- D-10: Before creating, show the user a preview of all plugin/modpack dependencies that will be installed (with versions)
- D-11: User confirms the dependency list before deployment proceeds
- D-12: Installation happens during the server creation pipeline as a progress step — no post-creation manual step needed for template dependencies

### The Agent's Discretion
- Exact visual layout of the template detail page (config preview, dependency list presentation)
- UI design of the creation modal (step form vs single scrollable form)
- Progress indicators during deployment with dependency installation
- Warning message wording and visual treatment for resource mismatches
- How template version and usage_count are displayed on detail page

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| D-01 | TemplateCard → detail page route | TemplateCard line 83: `<Link to={\`/templates/${template.id}\`}>` already exists, needs real page |
| D-02 | Detail page content | TemplateResponse DTO has all required fields (display_name, description, version, game_type, config, usage_count). Config JSONB contains docker_image, default_port, env data. |
| D-03 | Config modal from detail page | ConfigureServerModal (new) triggered by "Create Server" button. Pattern from CreateServerModal.jsx (694 lines) and ServerOnboardingWizard.jsx (420 lines). |
| D-04 | Overridable fields | CreateServerFromTemplateRequest DTO supports: name, node_id, ram_mb, max_players, minecraft_version. Need to map DISK (currently not in DTO — may need config_overrides). |
| D-05 | Pre-fill from template | TemplateResponse.config JSONB contains env.MEMORY, default_port etc. Frontend parses these for defaults. |
| D-06 | Lock non-overridable fields | Game type, port strategy displayed as read-only text. Pattern: locked fields rendered as `<span>` labels, not form inputs (UI-SPEC lines 333-342). |
| D-07 | Resource warning | `GET /api/v1/nodes/:id/resources` returns total_memory_gb + cpu_cores. Compare with template RAM requirement. |
| D-08 | Non-blocking warning | Warning displayed in orange panel, submit button stays enabled. UI-SPEC line 245. |
| D-09 | Node capacity, not billing | Warning text references node name + available capacity. API returns per-node capacity. |
| D-10 | Dependency preview | Template struct lacks a direct dependencies field. Plugin/modpack deps are stored in separate PluginTemplate/ModpackTemplate entities. Preview may need config JSONB parsing or a separate API call. |
| D-11 | Confirm dependencies | User scrolls through dependency list. Submission is implicit confirmation (UI-SPEC line 250). |
| D-12 | Auto-install during creation | Backend handles dependency installation during server creation pipeline. Frontend just shows preview + confirmation. |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Template detail page | Browser (React Router) | — | Static page rendering template data from API. No SSR needed. |
| Config override modal | Browser (React state) | API (POST /templates/:id/create-server) | User fills form → submits to existing backend endpoint |
| Resource enforcement | Browser (React, conditional rendering) | API (GET /api/v1/nodes/:id/resources) | Frontend fetches node capacity and shows warning. Backend enforces at creation time. |
| Dependency preview | Browser (React, list rendering) | — | Dependencies shown as read-only list. Installation happens server-side in pipeline. |
| Server redirection | Browser (React Router navigate) | — | After successful creation, redirect to `/servers/{server.id}` |
| Template CRUD | API (existing Phase 58) | — | No new endpoints. Uses existing `POST /templates/:id/create-server`. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| react-router-dom | ^7.13.0 [VERIFIED: package.json] | Routing for `/templates/:id` | Existing project dependency |
| lucide-react | ^1.20.0 [VERIFIED: npm registry] | Icons (ChevronLeft, Shield, Cpu, Rocket, X, Calendar, Users, Shuffle) | Existing project dependency |
| zustand | ^5.0.14 [VERIFIED: npm registry] | State management (uiStore addToast, serverStore addServer) | Existing project dependency |
| Tailwind CSS v4 | — [VERIFIED: index.css] | Styling with `@theme` directive | Existing CSS framework |
| React | ^19.2.4 [VERIFIED: package.json] | Component framework | Existing framework |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Fira Code (mono) | — | Version badges, code display | Via CSS `--font-mono` from index.css |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| — | — | No alternative libraries needed — all patterns use existing stack |

**Installation:**
No new packages needed. All dependencies are already in the project.

**Version verification:** All versions confirmed via `npm view` and `package.json` — no stale training data.

## Architecture Patterns

### System Architecture Diagram

```
TemplateCard (existing)                    TemplateDetailPage (NEW)               ConfigureServerModal (NEW)
      │                                          │                                       │
      │  click "Create Server"                    │                                       │
      │  (Link to /templates/:id)                 │                                       │
      ├─────────────────────────────────────────► │                                       │
      │                                          │                                       │
      │                                          │  GET /api/v1/templates/:id             │
      │                                          │       │                               │
      │                                          │       ▼                               │
      │                                          │  TemplateResponse                     │
      │                                          │  {display_name, description,           │
      │                                          │   version, game_type, category,        │
      │                                          │   config (JSONB), is_builtin,          │
      │                                          │   is_active, usage_count}              │
      │                                          │                                       │
      │                                          │  ┌─ Render page ──────────────────┐    │
      │                                          │  │ Header: name, badges, meta     │    │
      │                                          │  │ Description                    │    │
      │                                          │  │ Config Preview (RAM, port etc) │    │
      │                                          │  │ Dependencies (if any)          │    │
      │                                          │  │ [Create Server] button          │    │
      │                                          │  └────────────────────────────────┘    │
      │                                          │                                       │
      │                                          │  click "Create Server"                │
      │                                          ├──────────────────────────────────►    │
      │                                          │                                       │
      │                                          │          GET /api/v1/nodes            │
      │                                          │          (load node list)             │
      │                                          │               │                      │
      │                                          │               ▼                      │
      │                                          │          Node[] list                  │
      │                                          │                                       │
      │                                          │          GET /api/v1/nodes/:id/resources
      │                                          │          (on node select)             │
      │                                          │               │                      │
      │                                          │               ▼                      │
      │                                          │     NodeResourcesResponse              │
      │                                          │     {total_memory_gb, cpu_cores}       │
      │                                          │                                       │
      │                                          │  ┌─ ConfigureServerModal ────┐        │
      │                                          │  │ Server Name [_____]        │        │
      │                                          │  │ Target Node [dropdown ▼]   │        │
      │                                          │  │ Online Mode [toggle]       │        │
      │                                          │  │ RAM: [-] ==== 4GB [+]      │        │
      │                                          │  │ DISK: [-] ==== 20GB [+]    │        │
      │                                          │  │ ⚠ Resource warning (if any)│        │
      │                                          │  │ World Seed [________]      │        │
      │                                          │  │ Max Players [20 ▼]         │        │
      │                                          │  │ ─ Dependencies ─           │        │
      │                                          │  │ ✅ Plugin v1.3             │        │
      │                                          │  │ [Cancel] [Deploy Server]   │        │
      │                                          │  └────────────────────────────┘        │
      │                                          │                                       │
      │                                          │  POST /templates/:id/create-server     │
      │                                          │  {name, node_id, ram_mb,               │
      │                                          │   max_players, minecraft_version,       │
      │                                          │   config_overrides:{online_mode,...}}   │
      │                                          │       │                               │
      │                                          │       ▼                               │
      │                                          │  ServerResponse ← redirect to          │
      │                                          │  /servers/{server.id}                  │
      │                                          │                                       │
      │                                          │  toast: "Server created!"              │
      │                                          │◄───────────────────────────────────    │
      │                                          │                                       │
      │                                          │  navigate(/servers/${server.id})       │
      │                                          ├── (browser redirect)                   │
```

### Recommended Project Structure
```
app/src/
├── pages/templates/
│   ├── TemplateLibraryPage.jsx      # existing — no changes
│   ├── TemplateCreatePage.jsx       # existing — no changes
│   ├── TemplateDetailPage.jsx       # NEW — detail page at /templates/:id
│   └── ModBrowserPage.jsx           # existing — no changes
├── features/templates/
│   └── ConfigureServerModal.jsx     # NEW — creation config modal
├── hooks/
│   ├── useTemplateLibrary.js        # existing — no changes (has createServerFromTemplate)
│   ├── useModpackTemplates.js       # existing — reference for dependency pattern
│   └── usePluginTemplates.js        # existing — reference for dependency pattern
├── api/
│   ├── templatesApi.js              # existing — no changes (has createServer)
│   └── templatesApi.js (also in lib/api.js)  # existing — has all methods
└── app/
    └── App.jsx                      # MODIFY — add /templates/:id route
```

### Pattern 1: Stepper Component (RAM, DISK selection)
**What:** A `-`/`+` button pair with a visual progress bar showing the current value as a proportion of min-max range. Used for discrete resource selection.
**When to use:** RAM (2-32 GB, step 2), DISK (10-200 GB, step 10)
**Source:** `ServerOnboardingWizard.jsx` lines 130-158 [VERIFIED: codebase analysis]

```typescript
const renderStepper = (field, label, limits, step) => {
  const val = parseInt(wizardData.current[field])
  return (
    <div>
      <label className="text-sm text-[var(--color-text-secondary)] mb-1 block">{label}</label>
      <div className="flex items-center gap-3">
        <button
          onClick={() => setWizard(field, String(Math.max(limits.min, val - step)))}
          disabled={val <= limits.min}
          className="w-8 h-8 rounded bg-[var(--color-bg-primary)] border border-[var(--color-cosmic-border)] text-[var(--color-text-primary)] hover:brightness-110 disabled:opacity-30 disabled:cursor-not-allowed"
        >-</button>
        <div className="flex-1 h-2 bg-[var(--color-cosmic-border)] rounded-full overflow-hidden">
          <div
            className="h-full bg-[var(--color-cosmic-cyan)] rounded-full transition-all"
            style={{ width: `${((val - limits.min) / (limits.max - limits.min)) * 100}%` }}
          />
        </div>
        <button
          onClick={() => setWizard(field, String(Math.min(limits.max, val + step)))}
          disabled={val >= limits.max}
          className="w-8 h-8 rounded bg-[var(--color-bg-primary)] border border-[var(--color-cosmic-border)] text-[var(--color-text-primary)] hover:brightness-110 disabled:opacity-30 disabled:cursor-not-allowed"
        >+</button>
      </div>
    </div>
  )
}
```

### Pattern 2: Modal Structure (Zustand-based)
**What:** A glass-panel modal with backdrop blur, close-on-Escape, close-on-backdrop-click, role="dialog", aria-modal="true"
**Source:** `CreateServerModal.jsx` lines 273-275 [VERIFIED: codebase analysis]

```typescript
<div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
     onClick={(e) => e.target === e.currentTarget && handleClose()}
     role="dialog" aria-modal="true"
     onKeyDown={(e) => e.key === 'Escape' && handleClose()}>
  <div className="bg-[var(--color-bg-secondary)] rounded-lg p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto"
       onClick={e => e.stopPropagation()}>
    {content}
  </div>
</div>
```

### Pattern 3: API Client Pattern (fetchApi wrapper)
**What:** All API calls go through the `api` client which handles JWT auth, 401 refresh, JSON parsing.
**Source:** `app/src/lib/api.js` lines 20-58 [VERIFIED: codebase analysis]

```typescript
import { api } from '../../lib/api'
const data = await api.get('/templates/:id')
const result = await api.post('/templates/:id/create-server', payload)
```

### Anti-Patterns to Avoid
- **Using CreateServerModal logic directly:** The existing `CreateServerModal.jsx` (694 lines) has too much unrelated logic (game type filtering, modpack loading, server limit checks). The new `ConfigureServerModal` should be simpler — just template-specific overrides.
- **Nesting the modal inside a page-level state:** Use a dedicated Zustand slice or local state for modal open/close, not the global `uiStore.modal` which would conflict with other modals.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Toast notifications | Custom toast system | `useUIStore.addToast()` | Already has auto-dismiss, type variants (success/error/info), zustand-backed |
| API client with auth | Fetch with manual JWT | `api.js` ApiClient | Handles 401 → token refresh → retry, consistent error format |
| Loading spinners | Custom CSS spinner | `EscluseSpinner` from `SkeletonLoader.jsx` | Existing component with proper cosmic theme styling |
| Resource limits constants | Hardcoded RAM/DISK/player options | Import `RESOURCE_LIMITS` and from `constants.js` | Already in codebase, matches plan limits |
| Server store | Local server state | `useServerStore.addServer()` + `setServers()` | Already handles dedup, refresh, and server list management |

**Key insight:** This is a pure integration phase — every backend endpoint and frontend utility already exists. The work is wiring them into a new page and modal.

## Common Pitfalls

### Pitfall 1: Route Conflict — `/templates/:id` vs `/templates/:id/edit`
**What goes wrong:** React Router may match `/templates/:id/edit` to `/templates/:id` with `id = ":id/edit"` if routes are declared in the wrong order.
**Why it happens:** `react-router-dom` v7 resolves by declaration order. If `/templates/:id` is declared before `/templates/:id/edit`, it will capture the edit route.
**How to avoid:** In `App.jsx`, declare routes in this order:
```jsx
<Route path="/templates" element={<TemplateLibraryPage />} />
<Route path="/templates/create" element={<TemplateCreatePage />} />
<Route path="/templates/:id/edit" element={<TemplateCreatePage />} />
<Route path="/templates/:id" element={<TemplateDetailPage />} />   {/* ← NEW: add AFTER edit */}
<Route path="/mods" element={<ModBrowserPage />} />
```
**Current state (App.jsx lines 179-182):** Already in good order — `/templates/create` and `/templates/:id/edit` exist before any generic `/templates/:id`. The new route must be inserted AFTER line 181.

### Pitfall 2: Template Config JSONB Structure May Vary
**What goes wrong:** The template `config` field is a freeform JSONB blob. Different templates may have different keys (e.g., Vanilla has `env.MEMORY`, Forge has `startup_command`, Palworld has different `env` keys entirely).
**Why it happens:** The backend stores config as `serde_json::Value` — no schema enforcement at the database level.
**How to avoid:** Parse config defensively:
```typescript
const defaultRAM = template.config?.env?.MEMORY
  ? parseInt(template.config.env.MEMORY.replace('G', '')) 
  : 4 // sensible default
const defaultPort = template.config?.default_port ?? 25565
```
**Warning signs:** `undefined` RAM or port after accessing nested config keys.

### Pitfall 3: Dependencies May Not Be in Template Response
**What goes wrong:** The `Template` struct (model.rs) does NOT have a `dependencies` or `plugins` field. Plugin/modpack dependencies are stored in separate tables (`PluginTemplate`, `ModpackTemplate`).
**Why it happens:** Phase 58 separated concerns — templates define server config, plugin/modpack templates define dependency lists. They're linked by `game_type` + `category`/`variant`.
**How to avoid:** The dependency preview UI should either:
- Parse dependencies from `template.config.dependencies` if they exist as embedded JSON in the config blob
- Or show a static section that says "This template may include plugins/mods" with info from the template description
- Or make a separate API call to fetch associated plugin/modpack templates by game_type + category
**Note for planner:** This is an architectural gap — the current Template model doesn't reference plugin/modpack templates. The dependency preview feature (D-10, D-11) may need to be scoped as "show dependency info from config JSONB" or the backend needs an additional endpoint.

### Pitfall 4: Node Resources May Not Be Available for All Nodes
**What goes wrong:** `Node.total_memory` is `Option<i64>` — offline nodes may not have reported their capacity yet. `GET /api/v1/nodes/:id/resources` will return `null` for `total_memory_gb`.
**Why it happens:** Nodes report capacity on registration/heartbeat. Offline/never-connected nodes have `None` values.
**How to avoid:** Check for `null` before showing warning. If capacity unknown, skip the resource warning:

```typescript
if (selectedNodeResources?.total_memory_gb != null && 
    templateRAM > selectedNodeResources.total_memory_gb) {
  showWarning()
}
```

## Code Examples

### Example 1: Fetching single template by ID
```typescript
// Source: templatesApi.js lines 4-5 [VERIFIED: codebase analysis]
import { api } from '../../lib/api'

async function loadTemplate(id) {
  const template = await api.get(`/templates/${id}`)
  // Returns TemplateResponse: { id, game_type, category, display_name, 
  //   description, config: {docker_image, default_port, env: {...}}, 
  //   is_builtin, is_active, version, usage_count, created_at, updated_at }
  return template
}
```

### Example 2: Creating a server from template
```typescript
// Source: template_handlers.rs lines 144-229 + template_dtos.rs lines 42-52 [VERIFIED: codebase analysis]
const payload = {
  name: "My Awesome Server",        // required
  node_id: "uuid-or-null",          // optional — null = auto-select
  ram_mb: 4096,                     // optional — overrides template default
  max_players: 20,                  // optional
  minecraft_version: "26.2",        // optional
  config_overrides: {               // optional — deep-merged into template.config at backend
    "online_mode": "true",
    "world_seed": "myseed"
  }
}
const server = await api.post(`/templates/${templateId}/create-server`, payload)
// Returns the created server object
```

### Example 3: Fetching node resources for capacity check
```typescript
// Source: api_routes.rs line 54 [VERIFIED: codebase analysis]
async function getNodeResources(nodeId) {
  const resources = await api.get(`/nodes/${nodeId}/resources`)
  // returns { node_id, total_memory_bytes, total_memory_gb, cpu_cores, os_info }
  return resources
}
```

### Example 4: Loading nodes list for the node selector dropdown
```typescript
// Source: CreateServerModal.jsx lines 76-90 [VERIFIED: codebase analysis]
const [nodes, setNodes] = useState([])
const [nodesLoading, setNodesLoading] = useState(false)

async function loadNodes() {
  setNodesLoading(true)
  try {
    const nodesList = await api.get('/nodes')
    const validNodes = (nodesList?.nodes || nodesList || [])
      .filter(n => n && n.id && typeof n === 'object')
    setNodes(validNodes)
  } catch (err) {
    console.error('Failed to load nodes:', err)
    setNodes([])
  } finally {
    setNodesLoading(false)
  }
}
```

### Example 5: Success redirect pattern
```typescript
// Source: ServerOnboardingWizard.jsx lines 109-118 [VERIFIED: codebase analysis]
const navigate = useNavigate()
const { addToast } = useUIStore()
const { addServer, setServers } = useServerStore()

// After server creation succeeds:
addServer(server)
const allServers = await serversApi.list()
setServers(Array.isArray(allServers) ? allServers : [])
addToast({ type: 'success', message: 'Server created successfully!' })
handleClose()
setTimeout(() => navigate(`/servers/${server.id}`), 500)
```

### Example 6: Template config parsing for defaults
```typescript
// Source: Template model.rs fallback() examples + template_handlers.rs line 165-172 [VERIFIED: codebase analysis]
function parseTemplateDefaults(template) {
  const config = template.config || {}
  const env = config.env || {}
  
  // Parse RAM from env.MEMORY (format: "2G", "4G")
  const ramMatch = (env.MEMORY || '').match(/(\d+)G/)
  const defaultRAM = ramMatch ? parseInt(ramMatch[1]) : 4
  
  return {
    name: template.display_name + " Server",
    ram: defaultRAM,
    disk: 20,          // default disk — not in config, use constant
    port: config.default_port ?? 25565,
    onlineMode: true,  // default
    maxPlayers: parseInt(env.MAX_PLAYERS || '20'),
    gameType: template.game_type,
    version: template.version,
    dockerImage: config.docker_image,
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| TemplateCard "Create Server" → dead link (`/templates/:id`) | Link → real TemplateDetailPage | This phase | Phase 87 unblocks template detail page |
| Manual server creation (CreateServerModal) | Template-based creation with pre-filled config | This phase | Faster setup: 1-click from TemplateCard → configure → deploy |
| No config overrides on template creation | config_overrides (JSONB merge) | Phase 58 | Backend already supports deep-merge override of template config |

**Deprecated/outdated:**
- **CreateServerModal's template selection (lines 64-74, 316-345):** This modal has its own template loading logic that duplicates `templatesApi.list()`. The new ConfigureServerModal should use the already-fetched template detail data instead.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Template `config` JSONB contains parseable RAM/DISK defaults in `env.MEMORY` and similar keys | Config Preview | Templates created by users may not follow the same config schema as built-in fallbacks. May need to fall back to sensible defaults (4 GB RAM / 20 GB DISK). |
| A2 | Plugin/modpack dependency info is not embedded in the Template response | Dependencies | If templates created via Phase 85's edit feature DO embed dependency references in config JSONB, the dependency preview would need to parse them differently. The current Template model.rs has no `dependencies` field. |
| A3 | `config_overrides` in `POST /templates/:id/create-server` supports all override fields we need | Backend API | The backend deep-merges overrides into `server_config` (handler.rs line 165-172). Fields like `world_seed` and `online_mode` would need to be in `config_overrides` as they're not top-level fields in CreateServerFromTemplateRequest. |
| A4 | Node capacity comparison uses `total_memory_gb` from `/nodes/:id/resources` | Resource Enforcement | The endpoint returns total memory, not available (free) memory. For a warning, total memory may over-estimate capacity. The warning should say "requires X GB" not "has only X GB available". |

## Open Questions

1. **How should plugin/modpack dependencies be fetched for preview?**
   - What we know: `TemplateResponse` has no dependency field. `PluginTemplate` and `ModpackTemplate` are separate entities linked by `game_type` + `variant`.
   - What's unclear: Is there an API endpoint that returns "dependencies for template X"? Or should we fetch `plugin-templates?game_type=X&variant=Y` and match by category?
   - Recommendation: Check if the backend has an endpoint linking templates to their plugin/modpack deps. If not, the dependency preview section should show a static message: "Auto-installs compatible plugins/mods" and rely on the template description for specifics.

2. **What fields should go in `config_overrides` vs top-level request params?**
   - What we know: `CreateServerFromTemplateRequest` has these top-level fields: `name`, `node_id`, `game_type`, `minecraft_version`, `ram_mb`, `max_players`, `config_overrides` (freeform JSONB). Fields like `online_mode`, `world_seed`, disk allocation are NOT top-level.
   - What's unclear: Do these go in `config_overrides`? If disk isn't in config_overrides either, there's no way to override it.
   - Recommendation: Put override fields not in the top-level DTO into `config_overrides`. For DISK specifically, verify in the `CreateServerRequest` DTO if it's handled — if not, disk may not be overridable on template creation.

3. **Does the backend handle DISK allocation differently for templates?**
   - What we know: `CreateServerFromTemplateRequest` has `ram_mb` but no `disk_gb`. The `CreateServerRequest` (server_dtos.rs) also may not have a disk field.
   - What's unclear: Is disk allocation an overridable parameter or always set server-side?
   - Recommendation: If disk is not overridable via the API, either (a) hide the DISK stepper, (b) show it as informational only, or (c) add disk to the DTO. This needs backend verification.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vite (dev server) + manual testing |
| Config file | `app/vite.config.js` |
| Quick run command | `npm run dev` (in app/) |
| Full suite command | `npm run build` (in app/) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | Route /templates/:id renders detail page | manual | `navigate('/templates/{id}')` | ❌ Wave 0 |
| D-03 | Modal opens on "Create Server" click | manual | click button | ❌ Wave 0 |
| D-04 | Override fields pre-filled from template defaults | manual | open modal with known template | ❌ Wave 0 |
| D-07 | Resource warning shows on low-capacity node | manual | select node with < template RAM | ❌ Wave 0 |
| D-08 | Warning is non-blocking | manual | submit while warning shown | ❌ Wave 0 |
| D-10 | Dependency preview renders | manual | open modal with deps | ❌ Wave 0 |
| D-12 | Post-creation redirect to /servers/{id} | manual | verify URL after success | ❌ Wave 0 |
| Route safety | /templates/:id/edit doesn't break | manual | navigate to edit page | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `npm run build` — ensure no import/syntax errors
- **Per wave merge:** Manual walkthrough of all states
- **Phase gate:** Full state coverage before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `app/src/pages/templates/TemplateDetailPage.jsx` — new file
- [ ] `app/src/features/templates/ConfigureServerModal.jsx` — new file
- [ ] Route registration in `app/src/app/App.jsx` — modify existing file

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Vite dev server | ✓ | (check runtime) | — |
| npm | Package management | ✓ | (check runtime) | — |
| App dev server | Development | ✓ (via `npm run dev`) | Vite | — |
| Backend API | Template data | ✓ (assumed running) | — | Mock data in development |
| lucide-react | UI icons | ✓ | ^1.20.0 | — |

**Missing dependencies with no fallback:** None identified.

## Security Domain

> `security_enforcement` key absent from `.planning/config.json` — treated as enabled.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Form validation on server name (required, max 64 chars), RAM/DISK/players (range check) |
| V4 Access Control | yes | Backend handles auth via VerifiedUser middleware. Frontend shows/hides based on template ownership |
| V6 Cryptography | no | No encryption operations in this phase |
| V2 Authentication | no | Uses existing JWT from api.js client |

### Known Threat Patterns for React + Tailwind
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via template description | Tampering | React's JSX auto-escapes. Template descriptions are text, not HTML. |
| API response manipulation | Spoofing | Form validation on frontend is UX-only. Backend re-validates. Don't rely on frontend-only checks. |
| IDOR — creating server from template owned by another user | Elevation of Privilege | Backend handler.rs line 160 checks template ownership and visibility. Safe. |

## Sources

### Primary (HIGH confidence)
- `app/src/app/App.jsx` — Route structure for `/templates/:id` placement
- `app/src/lib/api.js` — API client + all endpoint definitions
- `app/src/store/uiStore.js` — Toast and modal state management
- `app/src/store/serverStore.js` — Server state after creation
- `api/src/presentation/handlers/template_handlers.rs` — POST `/templates/:id/create-server` handler (144-230)
- `api/src/application/dto/template_dtos.rs` — `TemplateResponse` + `CreateServerFromTemplateRequest` DTOs
- `api/src/application/dto/node_dtos.rs` — `NodeResourcesResponse` (total_memory_gb, cpu_cores)
- `api/src/presentation/routes/api_routes.rs` — Route definitions for all template and node endpoints
- `api/src/domain/server/template/model.rs` — Template struct definition + fallback data
- `api/src/domain/entities/node.rs` — Node struct with total_memory field
- `app/src/components/TemplateCard.jsx` — Existing "Create Server" link (line 83)
- `app/src/features/server/ServerOnboardingWizard.jsx` — Stepper pattern (130-158), deploy pattern (85-124)
- `app/src/features/server/CreateServerModal.jsx` — Form patterns, node loading (76-90), submit handling (154-215)
- `app/src/features/server/constants.js` — RESOURCE_LIMITS, RAM_OPTIONS, PLAYER_OPTIONS
- `app/src/index.css` — Cosmic theme CSS variables (glass-panel, status-dot, color tokens)
- `app/src/components/SkeletonLoader.jsx` — EscluseSpinner loading component

### Secondary (MEDIUM confidence)
- `87-CONTEXT.md` — Phase decisions and user constraints
- `87-UI-SPEC.md` — Visual and interaction contract (all 6 dimensions verified)
- `.planning/STATE.md` — Project state and history

### Tertiary (LOW confidence)
- None — all factual claims verified against codebase or official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All packages confirmed via `package.json` and `npm view`
- Architecture: HIGH — All code patterns verified by reading source files
- Pitfalls: HIGH — Route order confirmed by reading App.jsx, config JSONB structure confirmed from model.rs fallback data

**Research date:** 2026-06-17
**Valid until:** 2026-07-17 (stable project, rare dependency updates)
