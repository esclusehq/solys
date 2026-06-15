# Phase 81: update UI di halaman utama/dashboard app.esluce.com — Research

## 1. Current State Analysis

### Overview
`DashboardPage.jsx` (269 lines) is a functional single-file component rendering a vertical stack: welcome header → 3 summary cards → servers table (or empty state) → nodes table (or empty state). It uses `bg-gray-800`/`bg-gray-700` flat Tailwind colors with no cosmic theme (`glass-panel`, `var(--color-cosmic-*)` variables).

### Data Flow
- **Servers:** `serversApi.list()` → `useServerStore.setServers()` — called once in `useEffect` on mount. Stored in Zustand `servers` state.
- **Nodes:** `useNodes()` hook — fetches on mount, returns `{ nodes, loading, error, refetch }`.
- **Subscription:** `billingApi.getCurrentSubscription()` → local `subscription` state.
- **Auth:** `useAuthStore().user` → used only for welcome greeting via `user.display_name`.
- **Loading:** Single `isLoading` boolean (set `false` after servers fetch completes). Does NOT wait for nodes/subscription.

### Current States Handled
| State | Implementation | Styling |
|---|---|---|
| Loading | `<EscluseSpinner />` centered full screen | `bg-gray-900` |
| Empty (servers) | Centered text + CTA button | `bg-gray-800 rounded-lg p-12` |
| Empty (nodes) | Centered text + CTA button | `bg-gray-800 rounded-lg p-12` |
| Populated | Table rows | `bg-gray-800` container, `border-t border-gray-700` rows |
| Subscription null | Fallback `{ status: 'Free Plan', daysRemaining: '-', renewalDate: '-' }` | Inline in billing card |
| Edge: server disconnected/unknown/null/"" | Mapped to `'Disconnected'` label | Inline ternary status badge colors |

### Current Card Structure (3 `<Link>` components)
- **Servers card:** Blue icon, `activeServers / totalServers running`, links to `/servers`
- **Billing card:** Purple icon, plan status + days remaining, links to `/billing`
- **Agents card:** Green icon, `onlineNodes / totalNodes online`, links to `/nodes`
- All use: `bg-gray-800 rounded-lg p-6 border border-gray-700 hover:border-*-500/50 hover:bg-gray-700/50`

### Current Table Columns
**Servers table (8 cols):** Name, Status, Game, IP:Port, Domain, Node, Created, Actions (View button)
**Nodes table (6 cols):** Name, Status, IP Address, Uptime, Servers, Actions (View button)

---

## 2. Target State

### Visual Target: Match ServerManagerPage polish level (Phase 75)

| Element | Current | Target |
|---|---|---|
| Page container | `p-8` with no background | `p-8` + `stars-bg` overlay on page container |
| Welcome header | Plain `<h1>` + `<p>` text | Simple `<h1>` wrapped in cosmic container |
| Summary cards | `bg-gray-800 rounded-lg p-6 border border-gray-700` | `glass-panel p-6` + cosmic borders |
| Card icons | `bg-blue/purple/green-500/20` with stroke icons | Keep same icons, apply cosmic tint |
| Tables container | `bg-gray-800 rounded-lg overflow-hidden` | `glass-panel` with cosmic borders |
| Table header | `bg-gray-700` | `bg-[rgba(255,255,255,0.02)]` with cosmic bottom border |
| Table rows | `border-t border-gray-700` | `border-b border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.02)]` |
| Status badges | Inline ternary (no StatusBadge component) | Reuse `StatusBadge.jsx` component |
| Loading state | Full-screen `<EscluseSpinner />` | `SkeletonDashboard` component (already exists, matches layout) |
| Empty states | `bg-gray-800 rounded-lg p-12` | `glass-panel p-12` + helpful links below CTA |

### Functional Changes
- **Tables:** Add inline search input, status filter dropdown, and sort control per table (servers + nodes)
- **Servers table search:** Filter by `server.name`
- **Nodes table search:** Filter by `node.name`
- **Status filter:** Dropdown: All, Running/Online, Stopped/Offline
- **Sort:** Name A-Z, Status (running first), Last Activity (servers only) / Uptime (nodes only)
- **View button only:** No start/stop/restart quick actions on dashboard
- **Empty states:** Keep existing CTAs ("Create your first server" / "Add your first node") + add helpful links: docs.esluce.com quick start + feature highlight bullets

### What Stays the Same
- 3 summary cards (no new cards, no sparklines, no MetricsCard)
- Vertical stack layout: Header → Cards → Servers → Nodes
- Column structure for both tables
- Billing card logic: `getBillingInfo()`
- Welcome message logic: `getWelcomeMessage()`
- Route: `/` and `/dashboard` both point to DashboardPage
- `WelcomeModal` import/rendering

---

## 3. Reusable Components & Patterns

### Directly Reusable
| Component | File | Usage |
|---|---|---|
| `StatusBadge.jsx` | `app/src/components/StatusBadge.jsx:1` | Replace inline status spans in both tables — handles Running, Crashed, Degraded, Starting, Sleeping, Stopped |
| `SkeletonDashboard` | `app/src/components/SkeletonLoader.jsx:125` | Replace full-screen EscluseSpinner loading state — already matches dashboard layout (3 skeleton cards + server table skeleton + nodes table skeleton) |
| `SkeletonCard` | `app/src/components/SkeletonLoader.jsx:1` | Used within SkeletonDashboard |
| `SkeletonServerTable` | `app/src/components/SkeletonLoader.jsx:59` | Used within SkeletonDashboard |
| `SkeletonNodesTable` | `app/src/components/SkeletonLoader.jsx:94` | Used within SkeletonDashboard |
| `EscluseSpinner` | `app/src/components/SkeletonLoader.jsx:158` | Keep for initial full-screen loading if desired |

### Cosmic CSS Classes (in `app/src/index.css`)
| Class/Purpose | Definition |
|---|---|
| `.glass-panel` | `background: var(--color-cosmic-card); border: 1px solid var(--color-cosmic-border); border-radius: 12px; backdrop-filter: blur(12px)` |
| `.stars-bg` | Fixed overlay with radial-gradient star dots + purple/cyan nebula glow |
| `.glow-cyan` | `box-shadow: 0 0 20px rgba(13, 223, 242, 0.1)` |
| `.glow-text` | `text-shadow: 0 0 10px rgba(13, 223, 242, 0.3)` |
| `.status-dot.*` | 8px colored dots with glow for running/sleeping |

### CSS Variable Palette (from `--theme` in index.css)
```
--color-deep-space: #080b15
--color-cosmic-card: rgba(255, 255, 255, 0.03)
--color-cosmic-border: rgba(255, 255, 255, 0.08)
--color-cosmic-cyan: #0ddff2
--color-cosmic-purple: #a855f7
--color-cosmic-green: #10b981
--color-cosmic-red: #ef4444
--color-cosmic-orange: #f59e0b
--color-text-main: #e2e8f0
--color-text-muted: #64748b
```

### Patterns from ServerManagerPage (Phase 75 reference)
- **Search input style:** `bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] px-4 py-2 rounded-lg w-full max-w-md`
- **Select dropdown style:** Same as search but `w-auto`
- **Table container:** `<div className="bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-lg overflow-hidden">`
- **Table header row:** `<tr className="text-left text-[var(--color-text-muted)] border-b border-[var(--color-cosmic-border)] bg-[rgba(255,255,255,0.02)]">`
- **Table data row:** `<tr className="border-b border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.02)] transition-colors">`
- **View button:** `px-3 py-1.5 text-xs rounded border border-[var(--color-cosmic-border)] text-[var(--color-text-muted)] hover:text-[var(--color-cosmic-cyan)] hover:border-[var(--color-cosmic-cyan)] transition-colors`
- **Filter/sort controls in ServerManagerPage:** All at top of section as `select` dropdowns with matching cosmic style
- **localStorage persistence pattern:** `localStorage.getItem('server*')` for view mode, sort mode, game filter

---

## 4. Integration Points

### Routes (`App.jsx:82-83`)
```jsx
<Route path="/" element={<DashboardPage />} />
<Route path="/dashboard" element={<DashboardPage />} />
```
Both routes render the same component. No route changes needed.

### Data Dependencies (all already called in DashboardPage)
| Data Source | Hook/Import | Notes |
|---|---|---|
| Servers | `useServerStore().servers` via `serversApi.list()` | Already fetched on mount; `setServers` available |
| Nodes | `useNodes()` from `app/src/hooks/useNodes.js:4` | Returns `{ nodes, loading, error, refetch }` |
| Subscription | `billingApi.getCurrentSubscription()` | Already fetched on mount |
| Auth/User | `useAuthStore().user` | Already used for welcome message |

### Component Layout
- DashboardPage renders inside `ProtectedRoute` > flex container > main > TopBar > Routes
- `WelcomeModal` is rendered below the main `<div>` in the JSX tree (line 254)
- `ToastContainer` is in `App.jsx:104` (global, no changes needed)

### No New API Endpoints
Per CONTEXT.md boundary: "No new API endpoints — purely frontend changes."

---

## 5. Implementation Considerations

### Gotchas & Edge Cases

1. **Loading state replacement:** Current code uses a single `isLoading` boolean that flips after `serversApi.list()` resolves. Nodes fetch via `useNodes()` has its own `loading` state. The `SkeletonDashboard` component should be shown until both servers AND nodes are resolved. Consider using a combined `pageLoading` state or waiting for `useNodes().loading` to be false.

2. **Status edge cases in DashboardPage (lines 166-169):** The current inline status badge handles `disconnected`, `Unknown`, `null`, `""` by mapping to "Disconnected". The `StatusBadge` component handles `crashed`, `degraded`, `starting`, `sleeping` (autoWake), and defaults everything else to "Stopped". Need to ensure the node status display also uses the same `StatusBadge` pattern.

3. **Node status display:** Current inline node status uses simple `online`/`offline` green/red ternary. The `StatusBadge` component doesn't handle node-specific statuses (it's server-centric). Two options:
   - Extend `StatusBadge` with a `type="node"` prop
   - Keep inline badge for nodes but restyle with cosmic colors

4. **Search/filter/sort state isolation:** Both tables need independent:
   - `searchText` (string)
   - `statusFilter` ('all' | 'online'/'running' | 'offline'/'stopped')
   - `sortMode` ('name' | 'status' | 'activity'/'uptime')
   - These should NOT share state between tables (servers filter ≠ nodes filter)

5. **Filter options differ per table:**
   - **Servers:** status values = running, stopped, starting, sleeping, pending, crashed, disconnected
   - **Nodes:** status values = online, offline
   - Sort options: servers have `name`, `status`, `activity`; nodes have `name`, `status`, `uptime`

6. **localStorage persistence** (from Phases 74-75 pattern): Optionally persist filter/sort preferences per table. The ServerManagerPage persists `serverViewMode`, `serverSortMode`, `serverGameFilter`. For dashboard tables, consider per-table keys like `dashboardServersSort`, `dashboardNodesSort`.

7. **Billing card with null subscription:** Current `getBillingInfo()` returns `{ status: 'Free Plan', ... }` when subscription is null. No change needed but the card should handle the Free Plan display gracefully in the cosmic style.

8. **Empty state helpful links:** Need to generate helpful links for:
   - Servers empty: "Create your first server" CTA + docs.esluce.com quick start guide + feature highlights (what they can do after setup)
   - Nodes empty: "Add your first node" CTA + docs.esluce.com node setup guide + feature highlights
   - These should be styled with cosmic theme (glass-panel container)

9. **`stars-bg` overlay placement:** The `stars-bg` is a `position: fixed` element with `z-index: 0` and `pointer-events: none`. It should be placed as the first child of the dashboard page container (or wrapped in a container with `relative z-10` for content). Check how ServerManagerPage handles this — it doesn't use stars-bg directly (page uses cosmic card background instead). The dashboard is the landing page, so stars-bg makes sense here to match the cosmic landing feel.

10. **Filter controls at table section level vs table header:** D-08 says "inline search, filter, sort per table". The CONTEXT.md gives agent discretion on placement. The cleanest approach (matching ServerManagerPage) is to have a filter bar above each table section heading, similar to the search/filter/sort row in ServerManagerPage lines 194-261.

11. **ViewMode (card/table toggle) specifically excluded:** D-09 says "keep current view" — no card/table toggle on dashboard tables. Always table view.

12. **No quick start/stop actions:** D-10 explicitly says "View button only per row — no quick start/stop actions on dashboard." Keep only the View button, unlike ServerManagerPage which has View + Restart + Start/Stop.

13. **`useNodes()` re-fetch:** UseNodes doesn't auto-refetch unless the component re-mounts. For the dashboard, this is acceptable since it's a summary overview (no polling). The `refetch` function is available if needed.

14. **SkeletonDashboard still uses flat `bg-gray-*` colors:** The existing `SkeletonLoader.jsx` components use `bg-gray-800`, `bg-gray-700`, `bg-gray-600` instead of cosmic CSS variables. These should be updated to use `var(--color-cosmic-card)`, `var(--color-nebula)` etc. for consistency, OR the skeleton can be kept simple since it's a transient loading state.

---

## 6. File Change List

### Files to Modify

| File | Changes |
|---|---|
| `app/src/pages/dashboard/DashboardPage.jsx` | **Primary target.** Complete rewrite of the JSX while preserving data logic. Apply: cosmic `glass-panel` to all containers, `stars-bg` overlay, `StatusBadge` component for server statuses, inline search/filter/sort per table, `SkeletonDashboard` for loading state, enriched empty states with helpful links. |
| `app/src/components/SkeletonLoader.jsx` | Minor: Update `SkeletonCard`, `SkeletonDashboard`, `SkeletonServerTable`, `SkeletonNodesTable` to use cosmic CSS variables (`var(--color-cosmic-card)`, `var(--color-nebula)`) instead of flat `bg-gray-*` classes for consistency with the restyled dashboard. |

### Files to Create

| File | Purpose |
|---|---|
| None | All changes are contained within existing files. No new component files needed — the dashboard is a single page with no extracted sub-components (unlike Phase 80 SettingsPage which was split). |
