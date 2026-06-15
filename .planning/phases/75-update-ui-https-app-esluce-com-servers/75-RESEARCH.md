# Phase 75: Update UI https://app.esluce.com/servers - Research

**Researched:** 2026-06-14
**Domain:** Frontend UI — React component redesign, card/table view toggle, sorting/filtering, polling with toast notifications
**Confidence:** HIGH

## Summary

This phase redesigns the existing server list page (`ServerManagerPage.jsx` at `/servers`) with an improved card layout, a new table view toggle, sort controls, game-type filtering, restart buttons, 30-second polling with status change toast notifications, and localStorage persistence for view/filter/sort preferences. No new API endpoints are needed — all actions use existing `serversApi` methods (including `serversApi.restart(id)` which is already wired at `app/src/lib/api.js:108`).

The primary complexity is the **90s-style fastball**: adding polling with status-change detection (comparing server arrays between polls) and the table view re-using the column layout from legacy `ServerManager.jsx` but with cosmic theme styling. The implementation should be split into 3 sub-tasks: (1) view toggle + sort/filter + localStorage, (2) restart button + confirmation modal, (3) 30s polling + status change toasts.

**Primary recommendation:** Modify `ServerManagerPage.jsx` directly (keep as single file), adding ~250 lines. Extract only the confirmation modal and status-diff utility if they grow beyond 50 lines. Do not create a separate store — `serverStore.js` already provides `fetchServers`, and `uiStore.js` already provides `addToast`. [VERIFIED: codebase grep]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Card/table view toggle | Browser/Client | — | Purely visual toggle with localStorage persistence — no server interaction |
| Sort controls | Browser/Client | — | Client-side sort of already-loaded in-memory data — no server interaction |
| Game type filter | Browser/Client | — | Client-side filter of in-memory data — same pattern as existing status filter |
| Restart button action | API/Backend | Browser/Client | Calls `serversApi.restart(id)` (existing POST endpoint) — UI triggers, server executes |
| Restart confirmation modal | Browser/Client | — | Pure UI pattern — no server state needed |
| 30s polling | Browser/Client | API/Backend | `setInterval` → `fetchServers()` in component — fetches data, no new endpoints |
| Status change detection | Browser/Client | — | Client-side comparison of server arrays between poll cycles |
| Status change toast | Browser/Client | — | Uses existing `useUIStore().addToast()` — pure UI effect |
| View/filter/sort localStorage persistence | Browser/Client | — | Direct `localStorage.setItem/getItem` — follows existing pattern from `WorkspaceContext.jsx` and `Terminal.jsx` |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.2.4 | UI framework | Already in use project-wide |
| zustand | 5.0.12 | State management | Already in use for `serverStore`, `uiStore` |
| lucide-react | 1.18.0 | Icons (LayoutGrid, List, RotateCcw, ArrowUpDown, Eye, Play, Square) | Already in use project-wide |
| Tailwind v4 | 4.2.0 | CSS utility framework | Already configured via `@tailwindcss/vite` |
| react-router-dom | 7.13.0 | Client-side routing | Already in use project-wide [VERIFIED: package.json] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serversApi` from `lib/api.js` | — | API client with `restart(id)` method | For restart button (already exists, line 108) |

**No new dependencies required.** All functionality uses existing libraries.

## Architecture Patterns

### System Architecture Diagram

```
User Browser (ServerManagerPage.jsx)
    │
    ├── Initial Load
    │   └── useEffect → fetchServers() → zustand serverStore.servers[]
    │
    ├── View Toggle ────────────────────────────────────┐
    │   ├── Card View (default)                         │
    │   │   └── Grid layout: name / game-type / image   │
    │   │       + node / status-dot / actions row       │
    │   ├── Table View                                  │
    │   │   └── Rows: Name, Game, Host:Port, Image,     │
    │   │       Node, StatusBadge, Actions               │
    │   └── localStorage.setItem('serverViewMode')      │
    │                                                    │
    ├── Filter Bar ──────────────────────────────────────┤
    │   ├── Search input (existing)                      │
    │   ├── Status filter select (existing)              │
    │   ├── Game type filter select (NEW)                │
    │   ├── Sort select (NEW) ───────────────────────────┘
    │   └── View toggle buttons (NEW)
    │
    ├── Sort Logic (CLIENT-SIDE)
    │   ├── By Name (default, A-Z)
    │   ├── By Status (running first, then stopped/pending/error)
    │   └── By Last Activity (most recent first)
    │
    ├── 30s Polling (NEW)
    │   ├── setInterval(30000) → fetchServers()
    │   ├── Status diff: compare prevServers.map(s => s.status)
    │   │   with current response
    │   └── For each changed status:
    │       └── useUIStore().addToast({ type: 'info',
    │           message: '{name} is now {status}' })
    │
    ├── Restart Button (NEW)
    │   ├── Click → confirmation modal
    │   ├── Confirm → serversApi.restart(id)
    │   ├── Toast: "Restarting {name}..."
    │   └── On fetchServers() → toast updates implicitly via next poll
    │
    └── localStorage Persistence Pattern
        ├── 'serverViewMode' → 'card' | 'table'
        ├── 'serverSortMode' → 'name' | 'status' | 'activity'
        └── 'serverGameFilter' → 'all' | 'minecraft_java' | ...
```

### Recommended Component Structure

The implementation stays within `ServerManagerPage.jsx`. No new files required unless:

```
app/src/pages/servers/
├── ServerManagerPage.jsx    # ~450 lines (was ~200 → adds ~250)
├── components/              # Only if extraction is warranted
│   ├── ServerTable.jsx      # Optional: extract table view if file grows >500 lines
│   └── RestartModal.jsx     # Optional: extract if modal logic is complex
```

**Recommendation:** Keep everything in `ServerManagerPage.jsx`. Extract to `components/` only if the file exceeds 500 lines.

### Pattern 1: useRef-based polling with cleanup
**What:** Standard React pattern for interval-based polling with proper cleanup on unmount
**When to use:** Any component needing periodic data refresh
**Source:** [VERIFIED: React docs — useEffect cleanup pattern] [CITED: common React 19 pattern]
```javascript
// Pattern to follow in ServerManagerPage.jsx
const prevServersRef = useRef([])

useEffect(() => {
  const interval = setInterval(async () => {
    const oldServers = prevServersRef.current
    await fetchServers()
    const newServers = useServerStore.getState().servers
    // Detect changes
    newServers.forEach(newS => {
      const oldS = oldServers.find(s => s.id === newS.id)
      if (oldS && oldS.status !== newS.status) {
        addToast({ type: 'info', message: `${newS.name} is now ${newS.status}` })
      }
    })
    prevServersRef.current = newServers
  }, 30000)

  return () => clearInterval(interval)
}, []) // eslint-disable-line react-hooks/exhaustive-deps
```

### Pattern 2: useUIStore().addToast for notifications
**What:** Existing toast notification pattern used across 20+ components
**When to use:** Any user-facing notification
**Source:** [VERIFIED: codebase grep — 121 matches across 15+ files]
```javascript
import { useUIStore } from '../../store/uiStore'

// Inside component:
const { addToast } = useUIStore()

addToast({ type: 'success', message: 'Server restarted successfully' })
addToast({ type: 'error', message: 'Failed to restart server' })
addToast({ type: 'info', message: 'Survival SMP is now running' })
// Toast auto-dismisses after 5000ms (default duration)
```

### Pattern 3: localStorage persistence for user preferences
**What:** Direct localStorage get/set for UI state — no zustand persist middleware needed for transient view state
**When to use:** Simple view preferences that don't need reactive store state
**Source:** [VERIFIED: WorkspaceContext.jsx line 8-12, Terminal.jsx line 29-46, MfaVerifyPage.jsx]
```javascript
// On mount — read from localStorage
const [viewMode, setViewMode] = useState(() => {
  return localStorage.getItem('serverViewMode') || 'card'
})

// On change — save to localStorage
const handleViewModeChange = (mode) => {
  setViewMode(mode)
  localStorage.setItem('serverViewMode', mode)
}
```

### Pattern 4: Select input cosmic theme styling
**What:** Consistent dropdown styling using cosmic CSS variables
**Source:** [VERIFIED: ServerManagerPage.jsx lines 137-146]
```jsx
<select
  value={filter}
  onChange={(e) => setFilter(e.target.value)}
  className="bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] px-4 py-2 rounded-lg"
>
  <option value="all">All Status</option>
  ...
</select>
```

### Anti-Patterns to Avoid
- **Creating a new zustand store for sort/filter preferences:** These are transient UI state, not shared state. Use `useState` + `localStorage`, same as the existing pattern. [VERIFIED: codebase localStorage usage]
- **Using WebSocket for real-time updates:** Explicitly deferred to future phase (D-11). Use `setInterval` polling only.
- **Batch operations:** Deferred (D-08). Single-server actions only.
- **Adding server-side sort/filter parameters to API calls:** All sort/filter is client-side on already-loaded data. No API changes needed.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Toast notifications | Custom toast container | `useUIStore().addToast()` | Already exists with auto-dismiss, error/success/info types |
| Loading states | Custom spinner | `<EscluseSpinner />` from SkeletonLoader | Already exists in ServerManagerPage |
| Status badges | Custom status indicator | `<StatusBadge />` component | Already exists with running/stopped/crashed/starting/sleeping states |
| Confirmation modal | Custom modal overlay | Build inline modal (follow pattern from ServerDetailsPage delete modal) | No existing generic confirm component, but pattern is simple enough to inline |

**Key insight:** The codebase already has all the infrastructure needed. This phase adds UI only — no new stores, no new API calls, no new services.

## Runtime State Inventory

> Not applicable — this is a greenfield UI redesign phase, not a rename/refactor/migration phase.

## Common Pitfalls

### Pitfall 1: Poll interval not cleaned up on unmount
**What goes wrong:** Server list keeps polling after navigating away — causes memory leaks and unnecessary API calls.
**Why it happens:** Missing `clearInterval(interval)` in the `useEffect` cleanup function.
**How to avoid:** Always return a cleanup function from the polling `useEffect` that calls `clearInterval`.
**Warning signs:** API calls appearing in dev tools network tab after navigating away from `/servers`.

### Pitfall 2: Status change detection causing infinite loop
**What goes wrong:** Comparing `servers` array in `useEffect` dependency triggers re-render → re-fetch → status "change" → toast → re-render → infinite loop.
**Why it happens:** Using `useEffect` with `servers` as dependency to detect changes, then calling `fetchServers()` which updates `servers`.
**How to avoid:** Use `useRef` to store the previous servers snapshot (see Pattern 1). Compare ref value with store value after each poll — don't use `useEffect` on `servers` for change detection.
**Warning signs:** Toast notifications appearing on every poll cycle even when no status changed.

### Pitfall 3: Restart button accidentally doing a stop/start sequence
**What goes wrong:** The restart button calls `stop()` then `start()` instead of the dedicated restart endpoint.
**Why it happens:** Developer not aware that `serversApi.restart(id)` already exists as a single POST endpoint.
**How to avoid:** Use `serversApi.restart(id)` (line 108 of `api.js`) — it's a single API call with proper server-side restart handling. [VERIFIED: app/src/lib/api.js:108]

### Pitfall 4: localStorage key conflicts
**What goes wrong:** View mode and sort mode localStorage keys collide with other features.
**Why it happens:** Using generic keys like `viewMode` or `sortMode` without scoping.
**How to avoid:** Use namespaced keys: `'serverViewMode'`, `'serverSortMode'`, `'serverGameFilter'`. Check existing localStorage keys in the codebase — patterns include `escluse-*`, `devnode_*`.

### Pitfall 5: Table view columns don't match legacy ServerManager.jsx
**What goes wrong:** The table view in the new design has different columns than specified in UI-SPEC.
**Why it happens:** Developer derives columns from memory instead of the UI-SPEC contract.
**How to avoid:** Follow UI-SPEC.md Interaction Contract > Table View Columns section exactly. Columns in order: Name, Game, Host:Port, Image, Node, Status (StatusBadge), Actions.

## Code Examples

### Restart action using existing serversApi
```javascript
// ServerManagerPage.jsx — restart handler
import { serversApi } from '../../lib/api'
import { useServerStore } from '../../store/serverStore'
import { useUIStore } from '../../store/uiStore'

const handleRestart = async (server) => {
  try {
    await serversApi.restart(server.id)
    addToast({ type: 'info', message: `Restarting ${server.name}...` })
    fetchServers()
  } catch (err) {
    addToast({ type: 'error', message: `Failed to restart ${server.name}: ${err.message}` })
  }
}
```
[VERIFIED: serversApi.restart(id) exists at app/src/lib/api.js:108]

### Sort utility function
```javascript
const sortServers = (servers, sortMode) => {
  const sorted = [...servers]
  switch (sortMode) {
    case 'name':
      sorted.sort((a, b) => (a.name || '').localeCompare(b.name || ''))
      break
    case 'status':
      const statusOrder = { running: 0, starting: 1, stopping: 2, stopped: 3, sleeping: 4, pending: 5, error: 6, crashed: 7 }
      sorted.sort((a, b) => (statusOrder[a.status] ?? 99) - (statusOrder[b.status] ?? 99))
      break
    case 'activity':
      sorted.sort((a, b) => new Date(b.updated_at || 0) - new Date(a.updated_at || 0))
      break
  }
  return sorted
}
```

### View toggle with lucide icons
```jsx
import { LayoutGrid, List } from 'lucide-react'

// In component:
<div className="flex gap-1 bg-[var(--color-cosmic-card)] rounded-lg p-1 border border-[var(--color-cosmic-border)]">
  <button
    onClick={() => setViewMode('card')}
    aria-label="Card view"
    className={`p-2 rounded-md transition-colors ${
      viewMode === 'card'
        ? 'bg-[rgba(13,223,242,0.15)] text-[var(--color-cosmic-cyan)]'
        : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'
    }`}
  >
    <LayoutGrid size={18} />
  </button>
  <button
    onClick={() => setViewMode('table')}
    aria-label="Table view"
    className={`p-2 rounded-md transition-colors ${
      viewMode === 'table'
        ? 'bg-[rgba(13,223,242,0.15)] text-[var(--color-cosmic-cyan)]'
        : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'
    }`}
  >
    <List size={18} />
  </button>
</div>
```
[VERIFIED: LayoutGrid and List icons available in lucide-react v1.18.0 — node check passed]

### Table view row (cosmic-themed, following legacy ServerManager.jsx)
```jsx
// Table view row — follows ServerManager.jsx line 320-369 pattern
<tr key={server.id} className="border-b border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.02)] transition-colors">
  <td className="px-5 py-3">
    <Link to={`/servers/${server.id}`} className="font-medium hover:text-[var(--color-cosmic-cyan)]">
      {server.name}
    </Link>
  </td>
  <td className="px-5 py-3 capitalize">{server.config?.game_type || server.game || '—'}</td>
  <td className="px-5 py-3 font-mono text-xs">
    {server.public_host || 'localhost'}:{server.port}
  </td>
  <td className="px-5 py-3 font-mono text-xs truncate max-w-[150px]">{server.image || '—'}</td>
  <td className="px-5 py-3 font-mono text-xs">
    {server.node_id ? `${server.node_id.slice(0, 8)}...` : 'Auto-assigned'}
  </td>
  <td className="px-5 py-3"><StatusBadge status={server.status} autoWake={server.auto_wake} /></td>
  <td className="px-5 py-3">
    <div className="flex gap-2">
      {/* Actions: View, Restart, Start/Stop/Sleep/Wake */}
    </div>
  </td>
</tr>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw Tailwind gray classes (`bg-gray-800`, `border-gray-700`) | Cosmic CSS variables (`var(--color-cosmic-card)`, `var(--color-cosmic-border)`) | Phase 23+ | New ServerManagerPage code MUST use cosmic variables for consistency with TopBar, legacy ServerManager, and upcoming UI phases |
| Server action API via `api.post()` directly | `serversApi.*` methods in `lib/api.js` | Phase ~46+ | Use `serversApi.restart(id)` not raw `api.post('/servers/${id}/restart')` |
| Raw `alert()` for errors | `useUIStore().addToast()` | Phase ~44+ | Toast pattern is standard across all recent UI |

**Deprecated/outdated:**
- Direct `api.post('/servers/${id}/restart')` calls — use `serversApi.restart(id)` from `lib/api.js`
- Gray-900 backgrounds (`bg-gray-900`) — use `var(--color-deep-space)` or `#080b15`
- Gray-800 card backgrounds (`bg-gray-800`) — use `var(--color-cosmic-card)` or `rgba(255,255,255,0.03)`

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `serversApi.restart(id)` triggers a server restart via `POST /api/v1/servers/:id/restart` and returns without error | Code Examples (Restart) | LOW — endpoint exists in api.js and is used by ServerDetailsPage; restart is a standard server operation |
| A2 | The server response from `fetchServers()` includes `updated_at` timestamps for sort-by-activity | Code Examples (Sort) | MEDIUM — if `updated_at` isn't in the response, need fallback sort or field confirmation |
| A3 | Server objects include `config.game_type` field for game type filtering | Code Examples (Filter) | MEDIUM — current ServerManagerPage uses `server.config?.game_type` (line 163) as fallback from `server.image`, so filter logic should also handle both |

## Open Questions (RESOLVED)

1. **Game type filter options source**
   - What we know: UI-SPEC lists Minecraft Java, Minecraft Bedrock, PocketMine-MP, Nukkit as filter options.
   - What's unclear: Should filter options be hardcoded or derived from available game types in the data?
    - RESOLVED: Hardcode the 4 options as specified in UI-SPEC (D-05). Deriving from data adds complexity with no benefit since game types are fixed.

2. **Status change detection threshold**
    - What we know: D-10 says "show toast notification when a server changes status".
    - What's unclear: Should the toast fire for all status transitions (including starting→running, which could happen every poll during a restart)?
    - RESOLVED: Fire toast for ALL status transitions. The initial "starting" → "running" is genuinely useful feedback. If too noisy, it can be filtered later.

3. **Restart confirmation modal design**
    - What we know: UI-SPEC defines the copy and basic interaction.
    - What's unclear: Should the modal be an inline component in ServerManagerPage or a shared `ConfirmModal` component?
    - RESOLVED: Build inline in ServerManagerPage (~30 lines). If another phase needs confirmation, extract later (YAGNI).

## Environment Availability

> Step 2.6: SKIPPED (no external dependencies identified — this phase is purely frontend code changes; Node.js/npm availability confirmed at v22.22.2/10.9.7)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None detected |
| Config file | Not found |
| Quick run command | `npm run build` |
| Full suite command | `npm run build` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | Card shows minimal content | manual-only | — | ❌ Wave 0 |
| D-02 | View toggle switches between card and table | manual-only | — | ❌ Wave 0 |
| D-03 | View preference persisted to localStorage | manual-only | — | ❌ Wave 0 |
| D-04 | Sort by name/status/activity | manual-only | — | ❌ Wave 0 |
| D-05 | Game type filter works | manual-only | — | ❌ Wave 0 |
| D-06 | Filter/sort preferences persisted to localStorage | manual-only | — | ❌ Wave 0 |
| D-07 | Restart button shows confirmation then calls API | manual-only | — | ❌ Wave 0 |
| D-08 | No batch operations | manual-only | — | ❌ Wave 0 |
| D-09 | 30s polling via setInterval | manual-only | — | ❌ Wave 0 |
| D-10 | Status change shows toast notification | manual-only | — | ❌ Wave 0 |

### Wave 0 Gaps
- No test infrastructure exists in the project. All phase requirements are manual-only verification.

## Security Domain

> Omitted — `security_enforcement` is not configured. This phase is purely frontend UI changes with no new API endpoints, no data persistence changes, and no authentication/authorization logic. All existing security controls (JWT auth on API calls via `ApiClient.getToken()`) remain unchanged.

## Sources

### Primary (HIGH confidence)
- Codebase grep — serverStore.js (zustand, fetchServers, 90 lines), uiStore.js (addToast pattern, 66 lines), api.js (serversApi.restart at line 108), ServerManagerPage.jsx (199 lines target file), ServerManager.jsx (legacy table pattern), StatusBadge.jsx (38 lines), ToastContainer.jsx (30 lines), App.jsx (routing at line 84), TopBar.jsx (localStorage pattern)
- Package.json — versions confirmed: react 19.2.4, zustand 5.0.12, lucide-react 1.18.0, react-router-dom 7.13.0, tailwindcss 4.2.0, vite 7.3.1
- UI-SPEC.md — full interaction contract for view toggle, sort, filters, restart, polling, table columns
- CONTEXT.md — locked decisions D-01 through D-11

### Secondary (MEDIUM confidence)
- Node.js runtime check: v22.22.2, npm 10.9.7
- lucide-react icon availability verified via require() check: LayoutGrid, List, RotateCcw, ArrowUpDown, Eye, Play, Square, Trash2 all available

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified from package.json and node_modules
- Architecture: HIGH — patterns verified by codebase grep across 20+ files
- Pitfalls: HIGH — all identified from actual React pattern mistakes (useRef cleanup, infinite loops) and project-specific knowledge (existing restart endpoint)

**Research date:** 2026-06-14
**Valid until:** 2026-07-14 (stable dependencies, no fast-moving packages in this phase)
