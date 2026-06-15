# Phase 76: Update UI https://app.esluce.com/nodes - Research

**Researched:** 2026-06-14
**Domain:** Frontend UI — React component redesign, split-panel layout, table/card view toggle, health metrics visual refresh
**Confidence:** HIGH

## Summary

This phase redesigns the existing Nodes page (`Nodes.jsx` at `/nodes`) with a split-panel layout (node list left, detail panel right), table/card view toggle for the node list, enriched node card content (adding uptime + last seen), visual refresh for health metrics (progress bars, color coding), and cosmic theme restyling. No new API endpoints — purely frontend changes to a single file.

The primary complexity is the **retrofit of the existing component** (669 lines, single file) which already has a split-panel layout but needs significant restructuring to add the view toggle, enriched card content, relative time formatting, and visual refresh of health metrics. The phase is closely analogous to Phase 75 (servers page), which established the table/card toggle pattern with `useState` + `localStorage` persistence.

**Primary recommendation:** Modify `Nodes.jsx` directly (keep as single file), referencing Phase 75's `75-PATTERNS.md` for the view toggle implementation. Extract the health metric progress bars and relative time utility functions as small inline helpers (not separate files). The existing node data model provides `last_seen` and `first_seen` timestamps — uptime is derivable from `first_seen`, and "last seen" maps directly to `node.last_seen`. [VERIFIED: api/src/application/dto/node_dtos.rs lines 28-46]

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Keep split-panel layout (node list left, detail panel right).
- **D-02:** Add table/card view toggle for the node list, consistent with Phase 75 pattern.
- **D-03:** View preference persisted to localStorage.
- **D-04:** Show: name, IP address, memory, CPU cores, status, **uptime**, and **last seen**.
- **D-05:** Keep current status emoji + text pattern.
- **D-06:** No search or filter. Node count is typically low, not needed.
- **D-07:** Keep existing 3 tabs: Overview, API Keys, Tokens.
- **D-08:** No new tabs or sections added.
- **D-09:** Visual refresh for health metrics (same 4 data points: Status, CPU, Memory, Containers — improved presentation with bars/charts).

### the agent's Discretion
- Table view column layout (mirror node card fields)
- Visual style of health metrics refresh (progress bars, mini charts, color coding)
- Exact placement of uptime/last seen in node cards

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| D-01 | Split-panel layout (node list left, detail panel right) | Existing layout in Nodes.jsx lines 141-250 uses `flex-[4]`/`flex-[6]` pattern — preserve and restyle [VERIFIED: Nodes.jsx] |
| D-02 | Table/card view toggle | Phase 75 established `useState`+`localStorage` pattern with lucide `LayoutGrid`/`List` icons [VERIFIED: 75-PATTERNS.md] |
| D-03 | View preference persisted to localStorage | Use key `'nodeViewMode'` with values `'card'` or `'table'` [VERIFIED: Phase 75 localStorage pattern] |
| D-04 | Enriched card content (name, IP, memory, CPU, status, **uptime**, **last seen**) | Node entity provides `last_seen` and `first_seen` fields — uptime derivable from `first_seen` [VERIFIED: api/src/application/dto/node_dtos.rs:35-36] |
| D-05 | Keep status emoji + text pattern | Current pattern: `🟢`/`🟡`/`🔴` emoji + text — preserve [VERIFIED: Nodes.jsx lines 212-215] |
| D-06 | No search or filter | Not applicable — no research needed |
| D-07 | Keep 3 tabs (Overview, API Keys, Tokens) | Already implemented in NodeDetails component [VERIFIED: Nodes.jsx lines 391-395] |
| D-09 | Health metrics visual refresh | Use progress bars for CPU/memory percentages, color-coded labels, 4-column grid layout [VERIFIED: UI-SPEC.md lines 159-166] |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Split-panel layout | Browser/Client | — | Pure CSS flex layout — no server interaction |
| Table/card view toggle | Browser/Client | — | Purely visual toggle with localStorage persistence — no server interaction |
| Node list rendering | Browser/Client | API/Backend | Data fetched via `useNodes()` hook — renders client-side |
| Node detail panel | Browser/Client | API/Backend | Detail data via `useNodeKeys()`, `useNodeHealth()` hooks |
| Uptime/last seen formatting | Browser/Client | — | Client-side relative time calculation from `node.first_seen`/`node.last_seen` |
| Health metrics visualization | Browser/Client | — | Progress bars + color coding computed from health API response data |
| Modals (add node, keys, tokens) | Browser/Client | — | Pure UI modals — no server state beyond API calls |
| Node CRUD operations | API/Backend | Browser/Client | All mutations go through existing API client functions |
| Node health 30s polling | Browser/Client | API/Backend | `setInterval` in `useNodeHealth()` hook — existing pattern [VERIFIED: useNodes.js lines 126-132] |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.2.4 | UI framework | Already in use project-wide [VERIFIED: app/package.json] |
| zustand | 5.0.12 | State management (uiStore for toasts) | Already in use for `uiStore`, `serverStore` [VERIFIED: app/package.json] |
| lucide-react | 1.18.0 | Icons (LayoutGrid, List, Trash2, etc.) | Already in use project-wide [VERIFIED: npm view lucide-react version] |
| Tailwind v4 | 4.2.0 | CSS utility framework | Already configured via `@tailwindcss/vite` [VERIFIED: app/package.json] |
| react-router-dom | 7.13.0 | Client-side routing | Already in use project-wide [VERIFIED: app/package.json] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `useNodes` from `hooks/useNodes.js` | — | Node data hook (useNodes, useNodeKeys, useNodeHealth) | For all node data fetching — already used [VERIFIED: Nodes.jsx line 3] |
| `useUIStore` from `store/uiStore.js` | — | Toast notifications | For error/success feedback — already used [VERIFIED: Nodes.jsx line 4] |
| `SkeletonText`, `EscluseSpinner` from `components/SkeletonLoader.jsx` | — | Loading states | Already used in Nodes.jsx for loading [VERIFIED: Nodes.jsx lines 6-7] |

**No new dependencies required.** All functionality uses existing libraries.

## Architecture Patterns

### System Architecture Diagram

```
Nodes.jsx (Split-Panel Page)
│
├── Header Bar
│   ├── "Node Management" heading
│   └── "+ Add Node" button (existing, cosmic-cyan)
│
├── Split Panel (flex gap-6)
│   │
│   ├── Left Panel (flex-[4]) — Node List
│   │   ├── Panel Header
│   │   │   ├── "Nodes" label
│   │   │   ├── Node count badge "X / Y Nodes" (existing)
│   │   │   └── View toggle buttons (NEW)
│   │   │       ├── Card view (LayoutGrid icon) ← default
│   │   │       └── Table view (List icon)
│   │   │
│   │   ├── Loading State
│   │   │   ├── Initial load → EscluseSpinner (size=80)
│   │   │   └── Subsequent load → SkeletonText (5 lines)
│   │   │
│   │   └── Node List Content
│   │       ├── Empty: "No nodes found" + "Add Local Node" button
│   │       └── Card Mode:
│   │           ├── Node Card × N
│   │           │   ├── Header: name (left) + status emoji+text (right)
│   │           │   ├── Meta row 1: IP (mono) | Memory (GB) | CPU cores
│   │           │   ├── Meta row 2: Uptime (relative) | Last seen (relative)
│   │           │   └── Click → select node, open detail panel
│   │           └── Table Mode (NEW):
│   │               └── Table: Name | IP | Memory | CPU | Uptime | Last Seen | Status
│   │
│   └── Right Panel (flex-[6]) — Detail Panel
│       ├── Title bar: node name + IP + status badge + delete button
│       ├── Tab bar: Overview | API Keys | Tokens
│       ├── Overview Tab
│       │   ├── Health Status (4-column grid)
│       │   │   ├── Status: color-coded dot + label (green/orange/red)
│       │   │   ├── CPU: progress bar, % label, green(<70)/orange(70-90)/red(>90)
│       │   │   ├── Memory: progress bar, "X MB / Y MB", same color coding
│       │   │   └── Containers: count + mini bar (active/total)
│       │   └── System Info (2-column grid)
│       │       └── Agent Version, OS, Memory, CPU Cores, Container Runtime
│       ├── API Keys Tab (existing, visual alignment)
│       └── Tokens Tab (existing, visual alignment)
│       └── Loading: skeleton cards (4 items) for health
│
├── Add Node Modal (existing)
├── API Key Generated Modal (existing)
├── Node Created Modal (existing)
├── Token Expiry Modal (existing)
└── Token Display Modal (existing)
```

### Recommended Component Structure

Keep everything in `Nodes.jsx`. The single-file approach matches Phase 75's recommendation and the existing codebase pattern. The file is currently 669 lines and will grow to approximately 800-900 lines, which is still manageable for a single page component.

```
app/src/pages/Nodes.jsx    # ~850 lines (was 669 → adds ~180 lines)
```

### Pattern 1: Table/Card View Toggle (copy from Phase 75)
**What:** Segmented button group with lucide icons for switching between card and table views
**When to use:** Any page needing view mode switching
**Source:** [VERIFIED: 75-PATTERNS.md lines 65-107] [VERIFIED: UI-SPEC.md lines 128-134]
```jsx
import { LayoutGrid, List } from 'lucide-react'

// Use state with localStorage persistence (key: 'nodeViewMode')
const [viewMode, setViewMode] = useState(() => {
  return localStorage.getItem('nodeViewMode') || 'card'
})

// Toggle button group
<div className="flex gap-1 bg-[var(--color-cosmic-card)] rounded-lg p-1 border border-[var(--color-cosmic-border)]">
  <button
    onClick={() => { setViewMode('card'); localStorage.setItem('nodeViewMode', 'card') }}
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
    onClick={() => { setViewMode('table'); localStorage.setItem('nodeViewMode', 'table') }}
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

### Pattern 2: Relative Time Formatting Utility
**What:** Small inline function or hook to format ISO timestamps as human-readable relative time
**When to use:** For uptime and last seen display in node cards and table rows
**Source:** [CITED: common JavaScript pattern] [VERIFIED: NodeResponse provides `last_seen` and `first_seen` as RFC3339 strings — node_dtos.rs:35-36, 57-58]
```javascript
// Inline helper in Nodes.jsx
function formatRelativeTime(isoString) {
  if (!isoString) return '—'
  const now = Date.now()
  const then = new Date(isoString).getTime()
  const diffMs = now - then
  const diffSec = Math.floor(diffMs / 1000)
  
  if (diffSec < 60) return 'just now'
  const diffMin = Math.floor(diffSec / 60)
  if (diffMin < 60) return `${diffMin}m ago`
  const diffHr = Math.floor(diffMin / 60)
  if (diffHr < 24) return `${diffHr}h ago`
  const diffDay = Math.floor(diffHr / 24)
  return `${diffDay}d ago`
}

function formatDuration(isoStart) {
  if (!isoStart) return '—'
  const now = Date.now()
  const start = new Date(isoStart).getTime()
  const diffSec = Math.floor((now - start) / 1000)
  
  if (diffSec < 60) return '<1m'
  const diffMin = Math.floor(diffSec / 60)
  if (diffMin < 60) return `${diffMin}m`
  const diffHr = Math.floor(diffMin / 60)
  const remainMin = diffMin % 60
  if (diffHr < 24) return `${diffHr}h ${remainMin}m`
  const diffDay = Math.floor(diffHr / 24)
  return `${diffDay}d ${diffHr % 24}h`
}
```

### Pattern 3: Health Metric Progress Bars
**What:** Colored progress bars for CPU and Memory metrics in the health status grid
**When to use:** In the Overview tab's Health Status section
**Source:** [CITED: UI-SPEC.md lines 159-166] [VERIFIED: NodeHealthResponse provides `cpu_usage`, `memory_used`, `memory_total` fields — node_dtos.rs lines 88-92]
```jsx
// Progress bar component (inline in Nodes.jsx)
function ProgressBar({ value, label, colorClass }) {
  const pct = Math.min(value ?? 0, 100)
  const barColor = pct > 90 ? 'bg-[var(--color-cosmic-red)]'
    : pct > 70 ? 'bg-[var(--color-cosmic-orange)]'
    : 'bg-[var(--color-cosmic-green)]'
  
  return (
    <div>
      <div className="flex justify-between text-xs mb-1">
        <span className="text-[var(--color-text-muted)]">{label}</span>
        <span className="font-medium">{pct.toFixed(1)}%</span>
      </div>
      <div className="h-2 bg-[rgba(0,0,0,0.3)] rounded-full overflow-hidden">
        <div className={`h-full rounded-full transition-all ${barColor}`}
             style={{ width: `${pct}%` }} />
      </div>
    </div>
  )
}

// Usage in health grid (replacing the current simple metric display):
// CPU metric:
<ProgressBar value={health.cpu_usage} label="CPU" />

// Memory metric:
<div>
  <div className="flex justify-between text-xs mb-1">
    <span className="text-[var(--color-text-muted)]">Memory</span>
    <span className="font-medium">
      {(health.memory_used / 1024 / 1024).toFixed(0)} / {(health.memory_total / 1024 / 1024).toFixed(0)} MB
    </span>
  </div>
  <div className="h-2 bg-[rgba(0,0,0,0.3)] rounded-full overflow-hidden">
    <div className={`h-full rounded-full ${memoryBarColor}`}
         style={{ width: `${(health.memory_used / health.memory_total) * 100}%` }} />
  </div>
</div>
```

### Pattern 4: Enriched Node Card (Card View)
**What:** Restyled node card with uptime and last seen
**When to use:** When rendering the node list in card view mode
**Source:** [VERIFIED: UI-SPEC.md lines 142-148] [VERIFIED: NodeResponse fields in node_dtos.rs lines 28-46]
```jsx
// Card view node card (enriched with uptime + last seen)
<div
  key={node.id}
  onClick={() => setSelectedNode(node)}
  className={`p-4 rounded-lg cursor-pointer transition-all ${
    selectedNode?.id === node.id
      ? 'bg-[rgba(13,223,242,0.1)] border border-[var(--color-cosmic-cyan)]'
      : 'bg-[rgba(0,0,0,0.2)] border border-[var(--color-cosmic-border)] hover:border-[var(--color-cosmic-cyan)]'
  }`}
>
  {/* Header row */}
  <div className="flex items-center justify-between mb-2">
    <div className="font-medium">{node.name}</div>
    <div className="flex items-center gap-2">
      <span className="text-lg">{node.status === 'online' ? '🟢' : node.status === 'warning' ? '🟡' : '🔴'}</span>
      <span className="text-sm text-[var(--color-text-muted)]">{node.status}</span>
    </div>
  </div>
  {/* Meta row 1 */}
  <div className="flex items-center gap-3 text-sm">
    <span className="font-mono text-xs text-[var(--color-text-muted)]">{node.ip_address}</span>
    {node.total_memory && (
      <span className="text-xs text-[var(--color-text-muted)]">
        {(node.total_memory / 1048576).toFixed(1)} GB
      </span>
    )}
    {node.cpu_cores && (
      <span className="text-xs text-[var(--color-text-muted)]">{node.cpu_cores}C</span>
    )}
  </div>
  {/* Meta row 2 - NEW: uptime + last seen */}
  <div className="flex items-center gap-3 mt-1">
    <span className="text-xs text-[var(--color-text-muted)]">
      Uptime: {formatDuration(node.first_seen)}
    </span>
    <span className="text-xs text-[var(--color-text-muted)]">
      Last seen: {formatRelativeTime(node.last_seen)}
    </span>
  </div>
</div>
```

### Anti-Patterns to Avoid
- **Creating a new zustand store for node state:** Nodes already use a custom hook pattern (`useNodes()`) — not a zustand store. Keep this pattern; don't port to serverStore-like zustand store. [VERIFIED: useNodes.js]
- **Using `confirm()` and `alert()` for destructive actions:** Current code uses `confirm('Delete this node?')` (line 64) and `alert(err.message)` (lines 59, 70, 85, 97, 109). Replace with toast notifications (`addToast`) and confirmation modals matching Phase 75 pattern. [VERIFIED: Nodes.jsx lines 59-109]
- **Using gray-800/gray-700 background classes:** Legacy classes in `SkeletonNodesTable.jsx` — use cosmic CSS variables. [VERIFIED: 75-RESEARCH.md line 345]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Toast notifications | Custom toast container | `useUIStore().addToast()` | Already exists with auto-dismiss, error/success/info types [VERIFIED: uiStore.js lines 21-27] |
| Loading states | Custom spinner | `<EscluseSpinner />` from SkeletonLoader | Already exists and used in Nodes.jsx [VERIFIED: Nodes.jsx lines 157-158] |
| Skeleton loading | Custom skeleton | `<SkeletonText lines={5} />` from SkeletonLoader | Already exists and used in Nodes.jsx [VERIFIED: Nodes.jsx lines 162-163] |
| View toggle | Custom toggle UI | lucide `LayoutGrid` + `List` icons | Already established as the project pattern from Phase 75 [VERIFIED: 75-PATTERNS.md] |
| Confirmation modal | Custom modal | Inline modal (follow Nodes.jsx existing pattern) | Existing add node modal pattern is sufficient for delete/key confirmations |

**Key insight:** The codebase already has all the infrastructure needed. This phase adds UI layout and visual polish only — no new stores, API calls, or services.

## Runtime State Inventory

> Not applicable — this is a pure UI restyle phase, not a rename/refactor/migration phase. No runtime state changes.

## Common Pitfalls

### Pitfall 1: Uptime calculation using wrong timestamp field
**What goes wrong:** Uptime shows incorrect duration because the wrong timestamp field is used for calculation.
**Why it happens:** The Node entity has `first_seen` (node creation time), `last_seen` (last heartbeat), `created_at`, and `updated_at`. Uptime should be calculated from `first_seen` (when node first registered) or from health heartbeat timestamps, not from `created_at`.
**How to avoid:** Use `node.first_seen` for uptime (time since node was first registered). Use `node.last_seen` for "last seen" display. Both are available as RFC3339 strings. [VERIFIED: node_dtos.rs lines 35-36, 57-58]
**Warning signs:** Uptime showing "1d ago" on a node known to be online for weeks.

### Pitfall 2: Table view column order inconsistent with UI-SPEC
**What goes wrong:** Table columns are out of order or missing fields specified in UI-SPEC.
**Why it happens:** Developer creates columns from memory instead of following the spec.
**How to avoid:** Follow UI-SPEC.md Interaction Contract exactly: Name, IP Address, Memory, CPU, Uptime, Last Seen, Status. [VERIFIED: UI-SPEC.md lines 149-157]
**Warning signs:** Missing "Uptime" or "Last Seen" columns in table view.

### Pitfall 3: Status emoji mismatch between card view and table view
**What goes wrong:** Emoji status indicators differ between card and table modes.
**Why it happens:** Separate rendering paths in the component duplicate the emoji logic.
**How to avoid:** Use a single inline function or variable for status emoji that both views reference:
```jsx
const statusEmoji = { online: '🟢', warning: '🟡', offline: '🔴' }
```
[VERIFIED: UI-SPEC.md D-05 — keep emoji + text pattern]

### Pitfall 4: Modal content not visually updated to cosmic theme
**What goes wrong:** Existing modals (Add Node, API Key Generated, Token Display) use `bg-[rgba(0,0,0,0.3)]`/`bg-[rgba(0,0,0,0.2)]` which is slightly different from `var(--color-cosmic-card)` / `var(--color-cosmic-border)`.
**How to avoid:** Ensure all modals use cosmic CSS variables (`var(--color-cosmic-card)`, `var(--color-cosmic-border)`, etc.) for consistency with the rest of the page.

### Pitfall 5: `alert()`/`confirm()` calls not replaced
**What goes wrong:** Nodes.jsx currently uses `alert()` for errors (lines 59, 70, 85, 97, 109) and `confirm()` for delete confirmation (lines 64, 90). The UI-SPEC specifies toast-based error handling and confirmation modals.
**How to avoid:** Replace `alert()` calls with `useUIStore().addToast({ type: 'error', message: '...' })`. Replace `confirm()` for deletes with inline confirmation modals (following Phase 75's delete modal pattern). [VERIFIED: UI-SPEC.md lines 98-105]

## Code Examples

### Import Changes (Nodes.jsx)
```javascript
// Existing imports (keep)
import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useNodes, useNodeKeys, useNodeHealth, createNodeNode, updateNodeNode, deleteNodeNode, generateNodeKeyNode, deleteApiKeyNode, generateRegistrationTokenNode } from '../hooks/useNodes';
import { useUIStore } from '../store/uiStore';
import { api } from '../lib/api';
import { SkeletonText } from '../components/SkeletonLoader';
import { EscluseSpinner } from '../components/SkeletonLoader';

// NEW imports for Phase 76:
import { LayoutGrid, List, Trash2 } from 'lucide-react';
// NOTE: Trash2 is the lucide icon for the delete button (replacing inline SVG)
```
[VERIFIED: lucide-react v1.18.0 includes LayoutGrid, List, Trash2]

### Node Card Data Fields (from API response)
The `NodeResponse` from `GET /api/v1/nodes` includes these relevant fields for card/table rendering:
```json
{
  "id": "uuid",
  "name": "string",
  "ip_address": "string",
  "status": "online|offline|unhealthy|warning",
  "last_seen": "2026-06-14T12:00:00Z",
  "first_seen": "2026-06-01T08:00:00Z",
  "total_memory": 8589934592,      // bytes → / 1048576 for MB, / 1073741824 for GB
  "cpu_cores": 4,
  "os_info": "Linux x86_64",
  "agent_version": "1.0.0",
  "container_runtime": "docker",
  "podman_version": null,
  "created_at": "2026-06-01T08:00:00Z",
  "updated_at": "2026-06-14T12:00:00Z"
}
```
[VERIFIED: api/src/application/dto/node_dtos.rs lines 28-46]

### Health Metric Data Fields (from useNodeHealth)
The response from `GET /api/v1/nodes/:id/health` includes:
```json
{
  "status": "healthy|warning|unhealthy",
  "cpu_usage": 45.2,
  "memory_used": 2147483648,
  "memory_total": 8589934592,
  "container_count": 3,
  "last_heartbeat": "2026-06-14T12:00:00Z"
}
```
[VERIFIED: api/src/domain/entities/node_health.rs lines 134-166]

### Toast Notification Pattern (replacing alert())
```javascript
// Current (replace):
alert('Error: ' + err.message);

// New:
addToast({ type: 'error', message: `Failed to create node: ${err.message}` });
```
[VERIFIED: uiStore.js lines 21-27]

### Delete Confirmation Modal Pattern (replacing confirm())
```javascript
// Current (replace):
if (!confirm('Delete this node?')) return;

// New: Use inline state-based modal
const [deleteConfirm, setDeleteConfirm] = useState(null);

// In JSX:
{deleteConfirm && (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div className="glass-panel p-6 w-full max-w-md">
      <h3 className="text-lg font-semibold mb-2">Delete Node?</h3>
      <p className="text-sm text-[var(--color-text-muted)] mb-6">
        Are you sure you want to delete <strong>{deleteConfirm.name}</strong>?
        All servers on this node will be affected.
      </p>
      <div className="flex gap-2 justify-end">
        <button onClick={() => setDeleteConfirm(null)}
          className="px-4 py-2 rounded-lg text-sm border border-[var(--color-cosmic-border)]">
          Cancel
        </button>
        <button onClick={async () => {
          try {
            await deleteNodeNode(deleteConfirm.id);
            if (selectedNode?.id === deleteConfirm.id) setSelectedNode(null);
            refetch();
            addToast({ type: 'success', message: 'Node deleted' });
          } catch (err) {
            addToast({ type: 'error', message: err.message });
          }
          setDeleteConfirm(null);
        }}
          className="bg-[var(--color-cosmic-red)] text-white px-4 py-2 rounded-lg text-sm font-semibold">
          Delete
        </button>
      </div>
    </div>
  </div>
)}
```
[VERIFIED: UI-SPEC.md lines 100-105 for copy text]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw Tailwind gray classes (`bg-gray-800`, `border-gray-700`) | Cosmic CSS variables (`var(--color-cosmic-card)`, `var(--color-cosmic-border)`) | Phase 23+ | New/updated Nodes.jsx code MUST use cosmic variables for consistency |
| `alert()` / `confirm()` for errors and confirmations | `useUIStore().addToast()` / inline confirmation modals | Phase ~44+ | Replace all `alert()`/`confirm()` calls in Nodes.jsx |
| Existing NodeDetails inline in Nodes.jsx | Same pattern, with visual refresh to health metrics | This phase | Keep NodeDetails as a co-located function component in Nodes.jsx |
| Simple metric text display (e.g., "45.2%") | Progress bars with color coding (green/orange/red thresholds) | This phase | Add `<ProgressBar>` component for CPU/memory health metrics |

**Deprecated/outdated:**
- `confirm()` and `alert()` calls — use `addToast()` and inline confirmation modals
- Inline SVG for delete icon (Nodes.jsx lines 410-413) — use `Trash2` from lucide-react
- Gray-900/800/700 background classes — use cosmic CSS variables

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `node.first_seen` is a reliable proxy for uptime (time since first registration) | Pattern 4 | LOW — if first_seen is reset on certain events, uptime might show incorrectly. The alternative is using health's `last_heartbeat` field but that requires the health API call. |
| A2 | The `useNodeHealth()` hook is always called when a node is selected, and health data is available within the polling interval | Code Examples | LOW — the hook already has a 30s polling interval and error handling that preserves last known data |
| A3 | Nodes.jsx stays at ~800-900 lines without needing extraction | Component Structure | MEDIUM — if the file grows beyond 1000 lines, extract the NodeDetails component and/or the modal components to separate files |

## Open Questions (RESOLVED)

1. **Uptime calculation source when node is offline**
   - What we know: `node.first_seen` is always available; `node.last_seen` indicates when node was last seen.
   - What's unclear: Should uptime show the last connected duration (time between connected/last_seen) or the total time since first_seen?
   - Recommendation: Use `node.first_seen` for uptime as the simplest approach. When a node is offline, uptime shows "—" since the node isn't actively running. Handle offline display by checking `node.status === 'online'` before showing uptime.

2. **Health metric loading state for progress bars**
   - What we know: `useNodeHealth()` already has a `loading` state and error handling.
   - What's unclear: Should the progress bars show skeleton/animated placeholders during loading, or default to "—"?
   - Recommendation: Keep the existing skeleton pulse pattern (4 animated skeleton cards in grid) during loading, already implemented in Nodes.jsx lines 442-449.

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
| D-01 | Split-panel layout renders correctly | manual-only | — | ❌ Wave 0 |
| D-02 | Table/card view toggle switches between views | manual-only | — | ❌ Wave 0 |
| D-03 | View preference persisted to localStorage ('nodeViewMode') | manual-only | — | ❌ Wave 0 |
| D-04 | Node cards show name, IP, memory, CPU, status, uptime, last seen | manual-only | — | ❌ Wave 0 |
| D-05 | Status emoji + text pattern preserved | manual-only | — | ❌ Wave 0 |
| D-06 | No search/filter visible | manual-only | — | ❌ Wave 0 |
| D-07 | Detail panel has 3 tabs (Overview, API Keys, Tokens) | manual-only | — | ❌ Wave 0 |
| D-08 | No new tabs added | manual-only | — | ❌ Wave 0 |
| D-09 | Health metrics show progress bars with color coding | manual-only | — | ❌ Wave 0 |

### Wave 0 Gaps
- No test infrastructure exists in the project. All phase requirements are manual-only verification via `npm run build` and visual inspection.

## Security Domain

> Omitted — `security_enforcement` is not configured in `.planning/config.json`. This phase is purely frontend UI changes with no new API endpoints, no data persistence changes, and no authentication/authorization logic. All existing security controls (JWT auth on API calls via `ApiClient.getToken()`) remain unchanged.

## Sources

### Primary (HIGH confidence)
- Codebase grep — Nodes.jsx (669 lines), useNodes.js (node hooks), node_dtos.rs (NodeResponse fields), uiStore.js (addToast), SkeletonLoader.jsx (spinner/skeleton)
- Package.json — versions confirmed: react 19.2.4, zustand 5.0.12, lucide-react 1.18.0, react-router-dom 7.13.0, tailwindcss 4.2.0
- 75-PATTERNS.md — table/card toggle pattern, localStorage persistence, lucide icons
- 75-RESEARCH.md — view toggle code, loading/error patterns, toast usage
- UI-SPEC.md (Phase 76) — full interaction contract for split-panel, view toggle, card/table content, health metrics

### Secondary (MEDIUM confidence)
- Node.js runtime check: v22.22.2, npm 10.9.7
- lucide-react icon availability: LayoutGrid, List, Trash2 verified via npm view

### Tertiary (LOW confidence)
- None — all findings verified against codebase or official spec files

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified from package.json and node_modules
- Architecture: HIGH — patterns verified by codebase grep of Nodes.jsx, useNodes.js, node_dtos.rs, and analogous Phase 75 patterns
- Pitfalls: HIGH — all identified from actual React pattern mistakes (uptime field confusion, stale data) and project-specific knowledge (existing alert/confirm calls)

**Research date:** 2026-06-14
**Valid until:** 2026-07-14 (stable dependencies, no fast-moving packages in this phase)
