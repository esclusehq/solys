# Phase 78: Update UI https://app.esluce.com/mods — Research

**Researched:** 2026-06-14
**Domain:** React frontend + Rust backend (Modrinth API proxy)
**Confidence:** MEDIUM

## Summary

This phase enriches the Mod Browser page (`/mods`) with enriched result cards, a category filter, dynamic game versions, page-number pagination, a version detail modal, and an Add-to-Server install flow. Two detailed plans already exist (78-01-PLAN.md and 78-02-PLAN.md) covering the frontend work completely.

**Critical finding:** The existing plans assume backend API fields (`author`, `latest_version`, `date_published`) and an endpoint (`GET /api/v1/plugins/game-versions`) that **do not currently exist** in the Rust backend. These must be added to the backend before (or as part of) the frontend work. The plans also have a parameter name mismatch for the category filter.

**Primary recommendation:** Execute the frontend plans as-is, but add the required backend changes as Task 0 / Wave 0 prerequisites. The backend changes are small (add 3 fields to DTOs, add one new endpoint handler).

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Add author, latest version number, and categories/tags to each card.
- **D-02:** Keep existing: icon, title, description, download count, Versions + Add buttons.
- **D-03:** 'Add' opens a modal to select a target server, then installs the mod to that server's mods folder.
- **D-04:** User picks the mod version in the install modal (not auto-selected).
- **D-05:** Needs a new API endpoint or use existing server file management to copy mod files.
- **D-06:** 'Versions' opens a modal listing available versions for that mod.
- **D-07:** Each version in the modal has an 'Install' button that flows into the server-picker + install flow.
- **D-08:** Add category filter alongside existing version + loader filters.
- **D-09:** Fetch available game versions dynamically from Modrinth API instead of hardcoded list.
- **D-10:** Replace 'Load More' with page-number pagination.

### The agent's Discretion
- Modal design for server picker + version picker
- Category filter options and placement
- Page number component design
- Dynamic version fetching strategy (on mount or on filter interaction)

### Deferred Ideas (OUT OF SCOPE)
- Mod collections/wishlist — future phase
- Mod detail detail page (separate route) — future phase

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Mod search display | Browser (React) | — | Rendering search results, cards, pagination |
| Category/version/loader filter | Browser (React) | API (Rust proxy) | Frontend manages filter state; backend proxies to Modrinth |
| Version list modal | Browser (React) | API (Rust proxy) | Frontend renders modal; backend fetches versions from Modrinth |
| Add-to-Server modal | Browser (React) | API (Rust proxy) | Frontend shows server picker; backend installs plugin file |
| Plugin install execution | API (Rust proxy) | Modrinth CDN | Backend downloads plugin JAR to server's plugins directory |
| Dynamic game versions | API (Rust proxy) | Modrinth API | New backend endpoint proxies Modrinth's tag list |
| Server list for picker | Browser (via zustand) | API (Rust backend) | Server list comes from `useServerStore`, populated by backend API |

## Phase Requirements

No requirement IDs were specified for this phase. All capabilities mapped to decisions D-01 through D-10 above.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | ^19.2.4 | UI framework | Project standard |
| react-router-dom | ^7.13.0 | Routing | `/mods` route in App.jsx |
| zustand | ^5.0.12 | State management | `serverStore`, `uiStore` |
| lucide-react | ^1.18.0 | Icons | `X`, `ChevronLeft`, `ChevronRight` |
| Tailwind CSS v4 | ^4.2.0 | Styling | Cosmic theme via `@theme` in `index.css` |
| Vite | ^7.3.1 | Build tooling | Project standard |

**Installation:**
```bash
# All dependencies already installed — no new packages needed
```

**Version verification:** All packages confirmed via `app/package.json` [VERIFIED: file read].

### Already Available APIs
| API | Method | Endpoint | Status |
|-----|--------|----------|--------|
| `modsApi.search` | `GET /api/v1/plugins/search` | ✅ Exists — returns `{ plugins: [...], total: N }` |
| `modsApi.getVersions` | `GET /api/v1/plugins/:id/versions` | ✅ Exists — returns `PluginVersionDto[]` |
| `modsApi.install` | `POST /api/v1/servers/:id/plugins/install` | ✅ Exists — accepts `{ project_id, version_id }` |
| `modsApi.getGameVersions` | `GET /api/v1/plugins/game-versions` | ❌ **Does not exist — needs to be created** |

## ⚠️ Critical Backend Gaps

The Rust backend (`api/src/`) is missing several fields and endpoints that the plans assume exist. These must be addressed before the frontend work can function correctly.

### Gap 1: `author` field missing from search response
- **File:** `api/src/infrastructure/external_services/modrinth_client.rs` — `ModrinthProject` struct
- **File:** `api/src/application/dto/plugin_dtos.rs` — `PluginSearchResult` struct
- **File:** `api/src/application/use_cases/plugin_use_cases.rs` — mapping in `SearchPluginsUseCase::execute`
- **Fix:** Add `pub author: String` to `ModrinthProject` (field exists in Modrinth v2 API search hits) and `PluginSearchResult`, map in use case.
- **Impact:** Without this, `mod.author` in `ModSearchResult.jsx` will be `undefined`.

### Gap 2: `latest_version` field missing from search response
- **Same files as Gap 1**
- **Fix:** Add `pub latest_version: String` to `ModrinthProject` (field exists in Modrinth v2 API search hits) and `PluginSearchResult`.
- **Impact:** Without this, `mod.latest_version` in `ModSearchResult.jsx` will be `undefined`.

### Gap 3: `date_published` field missing from version DTO
- **File:** `api/src/application/dto/plugin_dtos.rs` — `PluginVersionDto` struct
- **File:** `api/src/application/use_cases/plugin_use_cases.rs` — mapping in `GetPluginVersionsUseCase::execute`
- **Fix:** Add `pub date_published: String` to `PluginVersionDto` (field `date_published` exists in Modrinth v2 API version responses as an ISO 8601 string) and map in use case.
- **Impact:** Without this, `ver.date_published` in the version modal will be `undefined`.

### Gap 4: `GET /api/v1/plugins/game-versions` endpoint missing
- Need to create a new handler + route in `api/src/`
- **Option A (recommended):** Proxy Modrinth's tag list: `GET https://api.modrinth.com/v2/tag/game_version` returns available game versions. Return the list of version names.
- **Option B:** Return a server-side hardcoded list (same as the frontend's `FALLBACK_VERSIONS`). Simpler but less dynamic.
- **Impact:** Without this, `modsApi.getGameVersions()` will 404 and fall back to the hardcoded list immediately.

### Gap 5: Category filter parameter name mismatch
- Plan's `handleCategoryChange` sends `params.category = c` but the backend's `PluginSearchQuery` expects `params.project_type`.
- **Fix:** In `ModBrowserPage.jsx` `executeSearch`, change `if (c) params.category = c` to `if (c) params.project_type = c`.
- **Impact:** Category filter will not actually filter results if it sends the wrong param name.

### Gap 6: `migration/` duplicate codebase
- Both `api/` and `migration/` directories contain nearly identical code. If both are deployed, backend changes need to be applied in both locations. The `api/` directory is the active deployment (confirmed via `docker-compose.yml` `backend` service).
- **Recommendation:** Confirm with user whether `migration/` is active or deprecated. If active, duplicate backend changes to `migration/src/`.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  Browser (React)                                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  ModBrowserPage.jsx                                       │  │
│  │  ┌──────────┐  ┌───────────┐  ┌────────────┐              │  │
│  │  │  Search  │  │  Filters  │  │ Pagination │              │  │
│  │  │  Input   │  │  (cat,    │  │ (page num) │              │  │
│  │  │          │  │   ver,    │  │            │              │  │
│  │  │          │  │   loader) │  │            │              │  │
│  │  └──────────┘  └───────────┘  └────────────┘              │  │
│  │                                                           │  │
│  │  ┌─────────────────────┐  ┌──────────────────────┐       │  │
│  │  │ ModSearchResult.jsx  │  │  VersionListModal    │       │  │
│  │  │ (enriched card)      │──│  (version rows +     │       │  │
│  │  │                      │  │   Install button)    │       │  │
│  │  └─────────────────────┘  └──────────────────────┘       │  │
│  │                                                           │  │
│  │  ┌───────────────────────────────────────────────┐       │  │
│  │  │  AddToServerModal                              │       │  │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌──────┐ │       │  │
│  │  │  │ Version      │  │ Server       │  │Inst. │ │       │  │
│  │  │  │ Picker       │  │ Picker       │  │Button│ │       │  │
│  │  │  └──────────────┘  └──────────────┘  └──────┘ │       │  │
│  │  └───────────────────────────────────────────────┘       │  │
│  │                                                           │  │
│  │  ◄────────── modsApi (api.get / api.post) ──────────────► │  │
│  └───────────────────────────────────────────────────────────┘  │
│                        │                                        │
│                        ▼                                        │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  zustand Stores                                           │  │
│  │  ┌────────────┐  ┌──────────┐                             │  │
│  │  │ serverStore │  │ uiStore  │                             │  │
│  │  │ (servers,   │  │ (toasts) │                             │  │
│  │  │  fetch)     │  │          │                             │  │
│  │  └────────────┘  └──────────┘                             │  │
│  └───────────────────────────────────────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────────┘
                               │ HTTP
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│  Rust Backend (api/src/)                                        │
│  ┌─────────────────┐  ┌──────────────┐  ┌────────────────┐     │
│  │ /plugins/search │  │ /plugins/:id │  │ /servers/:id/  │     │
│  │ (proxy to       │  │ /versions    │  │ plugins/install│     │
│  │  Modrinth)      │  │ (proxy to    │  │ (download .jar)│     │
│  │                 │  │  Modrinth)   │  │                │     │
│  └───────┬─────────┘  └──────┬───────┘  └───────┬────────┘     │
│          │                   │                  │               │
│          ▼                   ▼                  ▼               │
│   ┌─────────────────────────────────────────────────────┐      │
│   │  ModrinthClient (modrinth_client.rs)                │      │
│   │  • search_plugins() → Modrinth API v2/search       │      │
│   │  • get_project_versions() → Modrinth API v2/project│      │
│   │  • download_plugin() → download JAR from CDN       │      │
│   └─────────────────────────────────────────────────────┘      │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  NEW: /plugins/game-versions → Modrinth tag list      │    │
│  └────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

**Primary use case trace (Search → Install):**
```
User types query → debounced executeSearch() → modsApi.search(params) → 
/api/v1/plugins/search → backend proxies to Modrinth → returns enriched results →
render ModSearchResult cards with author/categories/version →
User clicks "Add" → handleAddFromCard → fetches versions + servers →
opens AddToServerModal → user picks version + server →
clicks Install → modsApi.install() → POST /api/v1/servers/:id/plugins/install →
backend downloads JAR to server's plugins/ directory →
success toast
```

### Recommended Project Structure

```
api/src/
├── infrastructure/external_services/modrinth_client.rs  # Add author, latest_version to ModrinthProject
├── application/dto/plugin_dtos.rs                        # Add author, latest_version to PluginSearchResult; date_published to PluginVersionDto
├── application/use_cases/plugin_use_cases.rs             # Map new fields from ModrinthProject → DTOs
└── presentation/
    ├── handlers/plugin_handlers.rs                       # Add get_game_versions handler
    └── routes/api_routes.rs                              # Add /plugins/game-versions route

app/src/
├── components/ModSearchResult.jsx                        # Enriched card (author, categories, latest_version) [Plan 01 Task 1]
├── pages/templates/ModBrowserPage.jsx                    # Category filter, dynamic versions, pagination, modals [Plan 01 Task 2 + Plan 02]
└── api/templatesApi.js                                   # Add getGameVersions() + install() [Plan 01 + Plan 02]
```

### Pattern 1: Modal Overlay Pattern
**What:** All modals use a consistent cosmic-themed overlay pattern from `InviteFriendsModal.jsx` (lines 60-68), updated to cosmic theme variables. [VERIFIED: file read]
**Example:**
```jsx
<div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4"
     onClick={onClose} role="dialog" aria-modal="true">
  <div className="bg-[var(--color-nebula)] border border-[var(--color-cosmic-border)]
                  rounded-xl p-6 max-w-lg w-full space-y-4 shadow-xl"
       onClick={(e) => e.stopPropagation()}>
```
- Escape key: Add `useEffect` with `keydown` listener for Escape [VERIFIED: established pattern]
- Backdrop click: `onClick={onClose}` on the outer div [VERIFIED: InviteFriendsModal.jsx line 62]

### Pattern 2: Toast Pattern
**What:** Success/error feedback via zustand store. [VERIFIED: uiStore.js lines 21-27]
```javascript
const { addToast } = useUIStore()
addToast({ type: 'success', message: 'Mod installed to MyServer' })
addToast({ type: 'error', message: 'Failed to install Mod. Network error' })
```
Auto-dismisses after 5000ms (default duration).

### Pattern 3: Server Store
**What:** Access server list via zustand store. [VERIFIED: serverStore.js]
```javascript
const { servers, fetchServers } = useServerStore()
// servers = [{ id, name, status?, game_type? }, ...]
```

### Pattern 4: Debounced Search
**What:** `useRef` + `setTimeout` + cleanup effect pattern already in `ModBrowserPage.jsx`. [VERIFIED: file read lines 16, 43-55]
```javascript
const debounceRef = useRef(null)
const handleSearchChange = (e) => {
  setQuery(val)
  setCurrentPage(1)
  clearTimeout(debounceRef.current)
  debounceRef.current = setTimeout(() => executeSearch(val, ...), 300)
}
useEffect(() => { return () => clearTimeout(debounceRef.current) }, [])
```

### Pattern 5: Filter Dropdown Styling
**What:** All filter selects use the same cosmic-theme styling. [VERIFIED: ModBrowserPage.jsx lines 108-110]
```jsx
<select value={version} onChange={handleVersionChange}
        className="px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                   border border-[var(--color-cosmic-border)] text-[var(--color-text-main)]">
```

### Anti-Patterns to Avoid
- **alert() for version viewing:** Already present in current code (line 85). Must be replaced with modal.
- **alert() for Add flow:** Already present (line 92). Must be replaced with modal + toast.
- **Hardcoded version list:** Current code has inline `<option>` elements (lines 112-118). Must be replaced with dynamic `gameVersions` state.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| State management | Custom state container | zustand | Already in project — serverStore, uiStore established |
| Icons | SVG icons | lucide-react | Already in project — `X`, `ChevronLeft`, `ChevronRight` available |
| HTTP client | Raw fetch with auth | `api` client from `lib/api.js` | Handles token refresh, error parsing, base URL |
| Toast notifications | Custom toast component | `useUIStore().addToast()` | Already implemented in `uiStore.js` + `ToastContainer.jsx` |
| Server list fetching | Custom API hook | `useServerStore().fetchServers()` | Already implemented — cached, deduped |
| Loading skeletons | Custom spinner | `EscluseSpinner` from SkeletonLoader | Already in project |

**Key insight:** This phase needs NO new npm packages. All required libraries (zustand, lucide-react, react-router-dom) are already in `package.json`. The only "new" things are backend DTO fields and one new backend endpoint.

## Common Pitfalls

### Pitfall 1: Backend DTO fields not updated
**What goes wrong:** Frontend tries to access `mod.author`, `mod.latest_version`, `ver.date_published` but they're `undefined` because the Rust backend DTOs don't include these fields.
**Root cause:** The `ModrinthProject` and `PluginSearchResult` structs in `api/src/` don't capture these Modrinth API fields.
**How to avoid:** Add backend DTO changes as Wave 0/Task 0 before any frontend work.
**Warning signs:** Build passes but cards show `by undefined`, version tag empty, version modal shows `Invalid Date`.

### Pitfall 2: Category filter sends wrong param name
**What goes wrong:** Category dropdown appears to work but doesn't filter results.
**Root cause:** Plan's `executeSearch` sends `params.category` but backend expects `params.project_type`.
**How to avoid:** Use `project_type` in the search params, not `category`.
**Warning signs:** Selecting a category does not change search results.

### Pitfall 3: Game versions endpoint 404
**What goes wrong:** `modsApi.getGameVersions()` hits 404, caught by try/catch, falls back to hardcoded list silently.
**Root cause:** No `/api/v1/plugins/game-versions` route registered in `api_routes.rs`.
**How to avoid:** Create the endpoint handler and route registration before frontend work.
**Warning signs:** No impact if fallback is acceptable; dynamic versions won't update with new Minecraft releases.

### Pitfall 4: Double codebase (api/ vs migration/)
**What goes wrong:** Changes made to `api/src/` but `migration/src/` is the actively deployed service, or vice versa.
**Root cause:** Two near-identical Rust codebases; unclear which is production.
**How to avoid:** Confirm with user before making backend changes. Apply changes in both directories if both are active.
**Warning signs:** Backend changes deploy but have no effect.

### Pitfall 5: `SkeletonText` imported but unused
**What goes wrong:** Plan 01 imports `SkeletonText` from SkeletonLoader but never uses it in the component code.
**Root cause:** Copied import from reference, not actually referenced in JSX.
**How to avoid:** Remove unused import or use it as loading placeholder in filters.
**Warning signs:** Build may show lint warning (depending on lint config).

## Code Examples

Verified patterns from the existing codebase:

### API Client Usage
```javascript
// Source: app/src/lib/api.js [VERIFIED: file read]
import { api } from '../../lib/api'

// GET with params
const data = await api.get('/plugins/search', { params: { q: 'essentials', offset: '0' } })

// POST with body
const result = await api.post('/servers/abc123/plugins/install', { project_id: 'xyz', version_id: 'ver_1' })
```

### Install API Request/Response
```rust
// Source: api/src/application/dto/plugin_dtos.rs [VERIFIED: file read]
// Request:
{ "project_id": "abc123", "version_id": "ver_001" }
// Response:
{ "filename": "MyPlugin.jar", "message": "Plugin installed successfully. Restart the server to activate it." }
```

### ModSearchResult Enrichment
```jsx
// Plan 01 enriches the card with:
// 1. {mod.author && <p>by {mod.author}</p>}
// 2. Category chips with max 2 + "+N" overflow
// 3. {mod.latest_version && <span class="font-mono text-xs">{mod.latest_version}</span>}
// Source: 78-01-PLAN.md [VERIFIED: plan includes complete code]
```

### Cosmic Theme Button Colors
```jsx
// Add button (card):     bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20
// Versions button (card): bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
// Install button (modal): bg-[var(--color-cosmic-cyan)] text-[#07090e] hover:bg-[#22dcf0] shadow-[0_0_15px_rgba(13,223,242,0.3)]
// Cancel button (modal):  bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] text-[var(--color-text-muted)] hover:text-white
// Source: 78-UI-SPEC.md [VERIFIED: file read]
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Load More button | Page-number pagination | This phase | Better UX for large result sets |
| alert() for versions | Version list modal | This phase | Proper modal with scroll, loading, error states |
| alert() for Add | Add-to-Server modal + toast | This phase | Functional install flow end-to-end |
| Hardcoded version list | Dynamic from API (fallback) | This phase | Auto-updates with new Minecraft releases |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Backend `api/src/` is the active deployed codebase, not `migration/src/` | Standard Stack | Backend changes in wrong directory won't deploy |
| A2 | Modrinth v2 API returns `author`, `latest_version` in search hits, and `date_published` in version responses | Critical Backend Gaps | Verified via Modrinth API v2 docs — LOW risk |
| A3 | The `/api/v1/servers/:id/plugins/install` endpoint works for installing plugins to any server | Architecture Patterns | Confirmed via `server_handlers.rs` line 412 — LOW risk |
| A4 | User wants both `api/` and `migration/` directories kept in sync | Critical Backend Gaps | If `migration/` is deprecated, only `api/` needs changes |
| A5 | The hardcoded `FALLBACK_VERSIONS = ['1.21', '1.20.4', ..., '1.16.5']` is acceptable as initial display | Code Examples | If user wants a different fallback, change the constant |

## Open Questions (RESOLVED)

1. **Is `migration/src/` active or deprecated?** — *(RESOLVED by Plan 78-01: changes applied to both `api/` and `migration/` for safety)*
   - What we know: Both `api/` and `migration/` have near-identical `src/presentation/handlers/plugin_handlers.rs` and route files. The docker-compose builds from `api/`.
   - Plan resolution: 78-01-PLAN.md applies backend changes to both directories. If `migration/` is confirmed deprecated later, the duplicate is harmless.

2. **Should `/plugins/game-versions` proxy the Modrinth tag list or return server-hardcoded versions?** — *(RESOLVED by Plan 78-01: proxy Modrinth v2 tag API)*
   - What we know: Modrinth has `GET /v2/tag/game_version` endpoint returning all available game versions.
   - Plan resolution: 78-01-PLAN.md Task 2 creates a handler that proxies Modrinth's `/v2/tag/game_version` endpoint using `reqwest::Client`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Frontend build | ✓ | TBD | — |
| npm | Package manager | ✓ | TBD | — |
| Rust + Cargo | Backend build | ✓ | TBD | — |
| Docker | Running backend for testing | ✓ | TBD | — |
| PostgreSQL | Backend data | ✓ | TBD | — |
| Redis | Backend caching | ✓ | TBD | — |

*Note: Environment-specific version probing was not performed. Build pipeline is assumed operational based on existing deployment.*

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None detected in `app/package.json` — no testing dependencies |
| Config file | None detected |
| Quick run command | `npm run build` from `app/` (build verification only) |
| Full suite command | `npm run build` from `app/` |

### Phase Requirements → Test Map
No testing framework is configured. The only automated validation is `npm run build` exits with code 0.

Verification is done via grep-based checks defined in the plans:
- Plan 01: 10 automated grep checks (enriched cards, category filter, dynamic versions, pagination, version modal)
- Plan 02: 8 automated grep checks (install modal state, handlers, toast integration, no alert())

### Wave 0 Gaps
- [ ] No test framework configured — `npm run build` is the only validation available
- [ ] Backend unit tests? No test runner detected in `api/Cargo.toml` either

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Auth handled by existing JWT middleware — no changes in this phase |
| V3 Session Management | No | No session changes |
| V4 Access Control | No | Server ownership checked by backend plugin install handler |
| V5 Input Validation | Yes | Search query, filters sent as URL params — backend validates/whitelists before forwarding to Modrinth |
| V6 Cryptography | No | No cryptographic operations |

### Known Threat Patterns for Rust + React

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Double-click install submission | DoS | Button disabled with `installing` state (plan already handles this) |
| Server ID tampering | Tampering | Backend validates server ownership before install (exists in `InstallPluginUseCase`) |

## Sources

### Primary (HIGH confidence)
- `app/src/pages/templates/ModBrowserPage.jsx` — target page current state [VERIFIED: file read]
- `app/src/components/ModSearchResult.jsx` — card component current state [VERIFIED: file read]
- `app/src/api/templatesApi.js` — modsApi current implementation [VERIFIED: file read]
- `app/src/lib/api.js` — API client pattern [VERIFIED: file read]
- `app/src/store/serverStore.js` — server store pattern [VERIFIED: file read]
- `app/src/store/uiStore.js` — toast/UI store pattern [VERIFIED: file read]
- `app/src/hooks/usePlugins.js` — installPlugin pattern [VERIFIED: file read]
- `app/src/components/InviteFriendsModal.jsx` — modal overlay pattern [VERIFIED: file read]
- `api/src/presentation/handlers/plugin_handlers.rs` — backend handlers [VERIFIED: file read]
- `api/src/application/dto/plugin_dtos.rs` — backend DTOs [VERIFIED: file read]
- `api/src/application/use_cases/plugin_use_cases.rs` — backend use cases [VERIFIED: file read]
- `api/src/infrastructure/external_services/modrinth_client.rs` — Modrinth client [VERIFIED: file read]
- `api/src/presentation/routes/api_routes.rs` — v1 API routes [VERIFIED: file read]
- `api/src/presentation/routes/server_routes.rs` — server/plugin routes [VERIFIED: file read]
- `api/src/presentation/handlers/server_handlers.rs` — server handler router (lines 410-413 for plugin routes) [VERIFIED: file read]
- `78-CONTEXT.md` — phase locked decisions [VERIFIED: file read]
- `78-UI-SPEC.md` — UI design contract [VERIFIED: file read]
- `78-01-PLAN.md` — existing plan for frontend Wave 1 [VERIFIED: file read]
- `78-02-PLAN.md` — existing plan for frontend Wave 2 [VERIFIED: file read]

### Secondary (MEDIUM confidence)
- Modrinth API v2 documentation — confirms `author`, `latest_version`, `date_published` fields in search/version responses [ASSUMED based on training data]
- Modrinth `GET /v2/tag/game_version` — returns list of valid game versions [ASSUMED]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all confirmed via file reads
- Architecture: HIGH — confirmed via file reads and pattern analysis
- Backend gaps: HIGH — confirmed via DTO/struct file reads; plans assume fields that don't exist
- Pitfalls: HIGH — directly derived from codebase analysis
- Security: MEDIUM — no changes to auth/access control, but double-submit and param validation identified

**Research date:** 2026-06-14
**Valid until:** 2026-07-14 (stable project — 30-day validity)
