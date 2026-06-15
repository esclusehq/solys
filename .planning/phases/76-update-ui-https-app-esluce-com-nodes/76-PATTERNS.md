# Phase 76: Update UI https://app.esluce.com/nodes - Pattern Map

**Mapped:** 2026-06-14
**Files analyzed:** 1 new/modified
**Analogs found:** 6 / 6 with matches

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `app/src/pages/Nodes.jsx` | component | CRUD + request-response | `app/src/pages/ServerManager.jsx` | role-match (same page role, split-panel + table pattern) |
| `app/src/pages/Nodes.jsx` (view toggle section) | component | request-response | `75-PATTERNS.md` lines 80-107 | exact (same view toggle pattern from Phase 75) |
| `app/src/pages/Nodes.jsx` (table view section) | component | CRUD | `app/src/pages/ServerManager.jsx` lines 220-370 | role-match (cosmic-themed table markup) |
| `app/src/pages/Nodes.jsx` (delete modal) | component | request-response | `app/src/pages/Nodes.jsx` lines 252-281 (add node modal) | exact (same inline modal pattern in same file) |
| `app/src/pages/Nodes.jsx` (toast error pattern) | component | request-response | `app/src/store/uiStore.js` lines 21-27 | exact (addToast already imported in Nodes.jsx line 11) |
| `app/src/pages/Nodes.jsx` (inline SVG → Trash2) | component | request-response | `app/src/components/TopBar.jsx` line 5 | exact (lucide icon import pattern) |

## Pattern Assignments

### `app/src/pages/Nodes.jsx` (component, CRUD + request-response)

**Target file** (669 lines). Major modifications to add: view toggle (card/table), enriched node cards (uptime + last seen), table view, progress bars for health metrics, relative time formatting, confirmation modals replacing `confirm()`, toast alerts replacing `alert()`, lucide Trash2 replacing inline SVG. File will grow to ~850 lines.

---

### Pattern 1: New Imports

**Analog:** `app/src/components/TopBar.jsx` line 5 (lucide import pattern) + `app/src/pages/Nodes.jsx` lines 1-7 (existing imports)

**Existing imports** (lines 1-7 — keep unchanged):
```javascript
import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useNodes, useNodeKeys, useNodeHealth, createNodeNode, updateNodeNode, deleteNodeNode, generateNodeKeyNode, deleteApiKeyNode, generateRegistrationTokenNode } from '../hooks/useNodes';
import { useUIStore } from '../store/uiStore';
import { api } from '../lib/api';
import { SkeletonText } from '../components/SkeletonLoader';
import { EscluseSpinner } from '../components/SkeletonLoader';
```

**New imports to add** (lucide icons for view toggle + delete button — `app/src/components/TopBar.jsx` line 5 pattern):
```javascript
// Analog: app/src/components/TopBar.jsx line 5 (lucide import pattern)
import { LayoutGrid, List, Trash2 } from 'lucide-react';
```

---

### Pattern 2: View Toggle State + localStorage Persistence

**Analog:** `75-PATTERNS.md` lines 65-78 (useState lazy init + localStorage)

```javascript
// Analog: 75-PATTERNS.md lines 69-77
const [viewMode, setViewMode] = useState(() => {
  return localStorage.getItem('nodeViewMode') || 'card'
})

// Handler — called on toggle button clicks
const handleViewModeChange = (mode) => {
  setViewMode(mode)
  localStorage.setItem('nodeViewMode', mode)
}
```

---

### Pattern 3: View Toggle Button Group (Segmented Control)

**Analog:** `75-PATTERNS.md` lines 80-107 (cosmic-themed segmented control)

```jsx
{/* Analog: 75-PATTERNS.md lines 83-106 — segmented button group */}
<div className="flex gap-1 bg-[var(--color-cosmic-card)] rounded-lg p-1 border border-[var(--color-cosmic-border)]">
  <button
    onClick={() => handleViewModeChange('card')}
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
    onClick={() => handleViewModeChange('table')}
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

**Placement:** Right side of the node list panel header, next to the node count badge (current Nodes.jsx lines 144-155).

---

### Pattern 4: Table View Markup (Nodes Table)

**Analog:** `app/src/pages/ServerManager.jsx` lines 220-370 (cosmic-themed table)

Adapted for node data with columns: Name, IP Address, Memory, CPU, Uptime, Last Seen, Status.

```jsx
{/* Analog: app/src/pages/ServerManager.jsx lines 220-370 — cosmic-themed table */}
<div className="glass-panel overflow-hidden">
  <table className="w-full text-sm">
    <thead>
      <tr className="text-left text-[var(--color-text-muted)] border-b border-[var(--color-cosmic-border)]">
        <th className="px-5 py-3 font-medium">Name</th>
        <th className="px-5 py-3 font-medium">IP Address</th>
        <th className="px-5 py-3 font-medium">Memory</th>
        <th className="px-5 py-3 font-medium">CPU</th>
        <th className="px-5 py-3 font-medium">Uptime</th>
        <th className="px-5 py-3 font-medium">Last Seen</th>
        <th className="px-5 py-3 font-medium">Status</th>
      </tr>
    </thead>
    <tbody>
      {nodes.map(node => (
        <tr
          key={node.id}
          onClick={() => setSelectedNode(node)}
          className={`border-b border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.02)] transition-colors cursor-pointer ${
            selectedNode?.id === node.id ? 'bg-[rgba(13,223,242,0.05)]' : ''
          }`}
        >
          <td className="px-5 py-3 font-medium hover:text-[var(--color-cosmic-cyan)]">
            {node.name}
          </td>
          <td className="px-5 py-3 font-mono text-xs text-[var(--color-text-muted)]">
            {node.ip_address}
          </td>
          <td className="px-5 py-3">
            {node.total_memory
              ? `${(node.total_memory / 1048576).toFixed(1)} GB`
              : '—'}
          </td>
          <td className="px-5 py-3">
            {node.cpu_cores ? `${node.cpu_cores}C` : '—'}
          </td>
          <td className="px-5 py-3 text-xs text-[var(--color-text-muted)]">
            {formatDuration(node.first_seen)}
          </td>
          <td className="px-5 py-3 text-xs text-[var(--color-text-muted)]">
            {formatRelativeTime(node.last_seen)}
          </td>
          <td className="px-5 py-3">
            <span className="text-lg">{statusEmoji[node.status] || '🔴'}</span>
            <span className="text-sm text-[var(--color-text-muted)] ml-1">{node.status}</span>
          </td>
        </tr>
      ))}
    </tbody>
  </table>
</div>
```

**Note:** Use a single `statusEmoji` lookup object shared across card and table views (AVOID duplicating the emoji logic):
```javascript
// Single source of truth for status emoji — used by both card and table views
const statusEmoji = { online: '🟢', warning: '🟡', offline: '🔴' }
```

---

### Pattern 5: Enriched Node Card (Card View — with Uptime + Last Seen)

**Analog:** `app/src/pages/Nodes.jsx` lines 186-219 (existing card, needs enrichment)

Replace the existing node card (lines 186-219) with the enriched version:

```jsx
{/* Analog: Nodes.jsx lines 186-219 (existing card pattern, enriched per UI-SPEC) */}
{nodes.map(node => (
  <div
    key={node.id}
    onClick={() => setSelectedNode(node)}
    className={`p-4 rounded-lg cursor-pointer transition-all ${
      selectedNode?.id === node.id
        ? 'bg-[rgba(13,223,242,0.1)] border border-[var(--color-cosmic-cyan)]'
        : 'bg-[rgba(0,0,0,0.2)] border border-[var(--color-cosmic-border)] hover:border-[var(--color-cosmic-cyan)] hover:bg-[rgba(13,223,242,0.05)]'
    }`}
  >
    {/* Header row: name + status emoji + text */}
    <div className="flex items-center justify-between mb-2">
      <div className="font-medium">{node.name}</div>
      <div className="flex items-center gap-2">
        <span className="text-lg">{statusEmoji[node.status] || '🔴'}</span>
        <span className="text-sm text-[var(--color-text-muted)]">{node.status}</span>
      </div>
    </div>
    {/* Meta row 1: IP, Memory, CPU cores */}
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
    {/* Meta row 2 — NEW: uptime + last seen (UI-SPEC D-04) */}
    <div className="flex items-center gap-3 mt-1">
      <span className="text-xs text-[var(--color-text-muted)]">
        Uptime: {formatDuration(node.first_seen)}
      </span>
      <span className="text-xs text-[var(--color-text-muted)]">
        Last seen: {formatRelativeTime(node.last_seen)}
      </span>
    </div>
  </div>
))}
```

---

### Pattern 6: Relative Time Formatting Utilities

**Analog:** RESEARCH.md lines 194-227 (inline helper functions)

No existing analog in codebase — these are inline utility functions placed at the bottom of Nodes.jsx or before the component:

```javascript
// RESEARCH.md lines 194-226 — relative time helpers
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

**Usage:**
- `formatRelativeTime(node.last_seen)` → "5m ago", "2h ago", "3d ago"
- `formatDuration(node.first_seen)` → "2h 30m", "1d 4h", "<1m"

---

### Pattern 7: Progress Bar Component (Health Metrics Refresh)

**Analog:** RESEARCH.md lines 234-271 (inline component for CPU/Memory progress bars)

New inline component in Nodes.jsx to replace the simple text metrics (Nodes.jsx lines 450-482):

```jsx
// RESEARCH.md lines 235-253 — inline ProgressBar component
function ProgressBar({ value, label, suffix = '%', formatValue }) {
  const pct = Math.min(value ?? 0, 100)
  const barColor = pct > 90
    ? 'bg-[var(--color-cosmic-red)]'
    : pct > 70
      ? 'bg-[var(--color-cosmic-orange)]'
      : 'bg-[var(--color-cosmic-green)]'

  return (
    <div>
      <div className="flex justify-between text-xs mb-1">
        <span className="text-[var(--color-text-muted)]">{label}</span>
        <span className="font-medium">
          {formatValue ? formatValue(value) : `${pct.toFixed(1)}${suffix}`}
        </span>
      </div>
      <div className="h-2 bg-[rgba(0,0,0,0.3)] rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all ${barColor}`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  )
}
```

**Usage in health grid** (replacing Nodes.jsx lines 462-477):
```jsx
{/* Health Status section — replaced with progress bars */}
<h3 className="font-medium mb-3">Health Status</h3>
{healthLoading ? (
  <div className="grid grid-cols-4 gap-4">
    {[...Array(4)].map((_, i) => (
      <div key={i} className="bg-[rgba(0,0,0,0.2)] p-3 rounded-lg animate-pulse">
        <div className="h-3 w-16 bg-gray-700 rounded mb-2" />
        <div className="h-5 w-12 bg-gray-700 rounded" />
      </div>
    ))}
  </div>
) : health ? (
  <div className="grid grid-cols-4 gap-4">
    {/* Status */}
    <div className="bg-[rgba(0,0,0,0.2)] p-3 rounded-lg">
      <div className="text-xs text-[var(--color-text-muted)]">Status</div>
      <div className={`font-medium flex items-center gap-2 ${
        health.status === 'healthy' ? 'text-[var(--color-cosmic-green)]'
        : health.status === 'warning' ? 'text-[var(--color-cosmic-orange)]'
        : 'text-[var(--color-cosmic-red)]'
      }`}>
        <span className={`w-2 h-2 rounded-full inline-block ${
          health.status === 'healthy' ? 'bg-[var(--color-cosmic-green)]'
          : health.status === 'warning' ? 'bg-[var(--color-cosmic-orange)]'
          : 'bg-[var(--color-cosmic-red)]'
        }`} />
        {health.status || 'unknown'}
      </div>
    </div>

    {/* CPU — progress bar */}
    <div className="bg-[rgba(0,0,0,0.2)] p-3 rounded-lg">
      <ProgressBar value={health.cpu_usage} label="CPU" />
    </div>

    {/* Memory — progress bar with MB formatting */}
    <div className="bg-[rgba(0,0,0,0.2)] p-3 rounded-lg">
      <ProgressBar
        value={health.memory_used && health.memory_total
          ? (health.memory_used / health.memory_total) * 100
          : 0}
        label="Memory"
        formatValue={() => health.memory_used != null && health.memory_total != null
          ? `${(health.memory_used / 1024 / 1024).toFixed(0)} / ${(health.memory_total / 1024 / 1024).toFixed(0)} MB`
          : '-'}
      />
    </div>

    {/* Containers */}
    <div className="bg-[rgba(0,0,0,0.2)] p-3 rounded-lg">
      <div className="text-xs text-[var(--color-text-muted)] mb-1">Containers</div>
      <div className="font-medium text-lg">{health.container_count ?? 0}</div>
    </div>
  </div>
) : (
  <div className="text-[var(--color-text-muted)]">No health data available</div>
)}
```

---

### Pattern 8: Delete Confirmation Modal (Replacing `confirm()`)

**Analog:** `app/src/pages/Nodes.jsx` lines 252-281 (existing Add Node modal pattern — same modal structure)

Replace the `confirm()` calls (Nodes.jsx lines 64, 90) with state-based confirmation modals:

**State declaration** (add alongside existing state at lines 13-23):
```javascript
// NEW: confirmation modal state (replacing confirm())
const [deleteConfirm, setDeleteConfirm] = useState(null)  // { id, name }
const [deleteKeyConfirm, setDeleteKeyConfirm] = useState(null)  // keyId
```

**Delete node handler** (replace Nodes.jsx lines 63-72):
```javascript
// Analog: Nodes.jsx lines 252-281 (inline modal pattern) + RESEARCH.md lines 438-478
const handleDeleteClick = (node) => {
  setDeleteConfirm(node)  // Opens modal instead of confirm()
}

// Actual deletion in modal confirm button (see JSX below)
```

**Delete Node Confirmation Modal** (new JSX block, placed alongside other modals):
```jsx
{/* Delete Node Confirmation Modal — replaces confirm('Delete this node?') */}
{/* Analog: Nodes.jsx lines 252-281 (modal structure) + RESEARCH.md lines 447-477 */}
{deleteConfirm && (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div className="glass-panel p-6 w-full max-w-md">
      <h3 className="text-lg font-semibold mb-2">Delete Node?</h3>
      <p className="text-sm text-[var(--color-text-muted)] mb-6">
        Are you sure you want to delete <strong>{deleteConfirm.name}</strong>?
        All servers on this node will be affected.
      </p>
      <div className="flex gap-2 justify-end">
        <button
          onClick={() => setDeleteConfirm(null)}
          className="px-4 py-2 rounded-lg text-sm border border-[var(--color-cosmic-border)]"
        >
          Cancel
        </button>
        <button
          onClick={async () => {
            try {
              await deleteNodeNode(deleteConfirm.id)
              if (selectedNode?.id === deleteConfirm.id) setSelectedNode(null)
              refetch()
              addToast({ type: 'success', message: 'Node deleted' })
            } catch (err) {
              addToast({ type: 'error', message: err.message })
            }
            setDeleteConfirm(null)
          }}
          className="bg-[var(--color-cosmic-red)] text-white px-4 py-2 rounded-lg text-sm font-semibold"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
)}

{/* Delete API Key Confirmation Modal — replaces confirm('Delete this API key?') */}
{deleteKeyConfirm && (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div className="glass-panel p-6 w-full max-w-md">
      <h3 className="text-lg font-semibold mb-2">Delete API Key?</h3>
      <p className="text-sm text-[var(--color-text-muted)] mb-6">
        Are you sure you want to delete this API key? This action cannot be undone.
      </p>
      <div className="flex gap-2 justify-end">
        <button
          onClick={() => setDeleteKeyConfirm(null)}
          className="px-4 py-2 rounded-lg text-sm border border-[var(--color-cosmic-border)]"
        >
          Cancel
        </button>
        <button
          onClick={async () => {
            try {
              if (!selectedNode) return
              await deleteApiKeyNode(selectedNode.id, deleteKeyConfirm)
              setSelectedNode({ ...selectedNode })
              addToast({ type: 'success', message: 'API key deleted' })
            } catch (err) {
              addToast({ type: 'error', message: err.message })
            }
            setDeleteKeyConfirm(null)
          }}
          className="bg-[var(--color-cosmic-red)] text-white px-4 py-2 rounded-lg text-sm font-semibold"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
)}
```

---

### Pattern 9: Toast Notifications (Replacing `alert()`)

**Analog:** `app/src/store/uiStore.js` lines 21-27 + `app/src/pages/Nodes.jsx` line 11 (already imported)

`useUIStore` with `addToast` is already imported and used in Nodes.jsx line 11:
```javascript
const { addToast } = useUIStore();  // Already exists at line 11
```

**Replace all `alert()` calls:**

| Line | Current Code | Replacement |
|------|-------------|-------------|
| 59 | `alert('Error: ' + err.message)` | `addToast({ type: 'error', message: \`Failed to create node: ${err.message}\` })` |
| 70 | `alert(err.message)` | `addToast({ type: 'error', message: \`Failed to delete node: ${err.message}\` })` |
| 85 | `alert(err.message)` | `addToast({ type: 'error', message: \`Failed to generate key: ${err.message}\` })` |
| 97 | `alert(err.message)` | `addToast({ type: 'error', message: \`Failed to delete API key: ${err.message}\` })` |
| 109 | `alert(err.message)` | `addToast({ type: 'error', message: \`Failed to generate token: ${err.message}\` })` |
| 176 | `alert('Error: ' + err.message)` | `addToast({ type: 'error', message: \`Failed to create local node: ${err.message}\` })` |

**Also replace `confirm()` with toast for non-destructive:**
- Nodes.jsx line 128: `if (confirm('Node limit reached. Upgrade your plan to add more nodes?'))` → `addToast({ type: 'info', message: 'Node limit reached. Upgrade plan to add more nodes.' })` then `navigate('/billing')`

---

### Pattern 10: Trash2 Icon (Replacing Inline SVG)

**Analog:** `app/src/components/TopBar.jsx` line 5 (lucide import + usage pattern)

Replace the inline SVG delete icon (Nodes.jsx lines 410-413 and 562-564) with the lucide `Trash2` component:

**Import** (added in Pattern 1):
```javascript
import { Trash2 } from 'lucide-react';
```

**Replace in NodeDetails header** (Nodes.jsx lines 408-413):
```jsx
{/* REPLACEMENT: lucide Trash2 instead of inline SVG */}
<button onClick={() => onDelete(node.id)}
  className="p-2 rounded hover:bg-[var(--color-cosmic-red)]/20 text-[var(--color-cosmic-red)]">
  <Trash2 size={16} />
</button>
```

**Replace in API Keys tab** (Nodes.jsx lines 560-565):
```jsx
<button onClick={() => onDeleteApiKey(key.id)}
  className="p-1.5 rounded hover:bg-[var(--color-cosmic-red)]/20 text-[var(--color-cosmic-red)]">
  <Trash2 size={16} />
</button>
```

---

### Pattern 11: Node Limit Check — Toast Instead of confirm()

**Analog:** Nodes.jsx lines 127-131 (existing limit check pattern — replace `confirm` with appropriate UX)

The + Add Node button already shows "Limit Reached" at line 137. The `confirm()` at line 128 should be replaced with navigation to `/billing` without a confirm dialog, or use `addToast`:

```javascript
// Nodes.jsx lines 126-134 — REPLACEMENT: toast instead of confirm()
<button onClick={() => {
  if (maxNodes !== -1 && nodes.length >= maxNodes) {
    addToast({ type: 'info', message: 'Node limit reached. Upgrade plan to add more nodes.' })
    navigate('/billing')
    return;
  }
  setShowForm(true);
}}
```

---

### Pattern 12: Empty State — Error Handler with Toast

**Analog:** Nodes.jsx lines 171-178 (existing empty state "Add Local Node" button)

Replace the `alert()` at line 176 with `addToast()`:
```javascript
{/* Nodes.jsx lines 170-179 — REPLACEMENT: toast instead of alert */}
<button
  onClick={async () => {
    try {
      await createNodeNode({ name: 'Local', ip_address: '127.0.0.1' });
      refetch();
      addToast({ type: 'success', message: 'Local node created' });
    } catch (err) {
      addToast({ type: 'error', message: `Failed to create local node: ${err.message}` });
    }
  }}
  className="px-4 py-2 bg-[var(--color-cosmic-cyan)] text-black rounded-lg hover:opacity-90"
>
  Add Local Node
</button>
```

---

## Shared Patterns

### View Toggle + localStorage Persistence
**Source:** `75-PATTERNS.md` lines 65-107
**Apply to:** Nodes.jsx — node list view toggle (card/table)
```javascript
// UseState lazy init with localStorage
const [viewMode, setViewMode] = useState(() => {
  return localStorage.getItem('nodeViewMode') || 'card'
})
const handleViewModeChange = (mode) => {
  setViewMode(mode)
  localStorage.setItem('nodeViewMode', mode)
}
```

### Toast Notifications (replacing alert)
**Source:** `app/src/store/uiStore.js` lines 21-27
**Apply to:** All action handlers (create, delete, generate key, generate token)
```javascript
// Pattern from app/src/store/uiStore.js lines 21-27
import { useUIStore } from '../store/uiStore'
const { addToast } = useUIStore()

// Usage:
addToast({ type: 'error', message: 'Failed to create node: ...' })  // Error
addToast({ type: 'success', message: 'Node deleted' })               // Success
addToast({ type: 'info', message: 'Node limit reached...' })          // Info
// Toast auto-dismisses after 5000ms (duration from uiStore.js line 26)
```

### Confirmation Modals
**Source:** `app/src/pages/Nodes.jsx` lines 252-281 (existing modal pattern)
**Apply to:** Delete node confirmation, Delete API key confirmation
```jsx
// Modal structure (use glass-panel + cosmic variables)
<div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
  <div className="glass-panel p-6 w-full max-w-md">
    <h3 className="text-lg font-semibold mb-2">Title</h3>
    <p className="text-sm text-[var(--color-text-muted)] mb-6">Body text</p>
    <div className="flex gap-2 justify-end">
      <button onClick={() => setState(null)}
        className="px-4 py-2 rounded-lg text-sm border border-[var(--color-cosmic-border)]">
        Cancel
      </button>
      <button onClick={...}
        className="bg-[var(--color-cosmic-red)] text-white px-4 py-2 rounded-lg text-sm font-semibold">
        Delete
      </button>
    </div>
  </div>
</div>
```

### Cosmic Theme CSS Variables
**Source:** `app/src/index.css` lines 4-19
**Apply to:** All new UI elements in Nodes.jsx
```css
/* CSS variables from app/src/index.css lines 4-19 */
var(--color-deep-space)      /* #080b15 — page background */
var(--color-cosmic-card)     /* rgba(255,255,255,0.03) — card/surface bg */
var(--color-cosmic-border)   /* rgba(255,255,255,0.08) — borders */
var(--color-text-main)       /* #e2e8f0 — primary text */
var(--color-text-muted)      /* #64748b — secondary text */
var(--color-cosmic-cyan)     /* #0ddff2 — accent, active states */
var(--color-cosmic-green)    /* #10b981 — success, online status */
var(--color-cosmic-red)      /* #ef4444 — destructive, error */
var(--color-cosmic-orange)   /* #f59e0b — warning */
var(--color-cosmic-purple)   /* #a855f7 — token modals, purple accent */
```

### Lucide Icon Import Pattern
**Source:** `app/src/components/TopBar.jsx` line 5
**Apply to:** All icon usage in Nodes.jsx
```javascript
// import pattern from TopBar.jsx line 5
import { LayoutGrid, List, Trash2 } from 'lucide-react'
// lucide-react v1.18.0 — LayoutGrid, List, Trash2 all available
```

### Skeleton Loading States (keep existing pattern)
**Source:** `app/src/components/SkeletonLoader.jsx` lines 45-57, 158-164
**Apply to:** Loading states in Nodes.jsx (lines 156-164 already correct)
- Initial load: `<EscluseSpinner size={80} color="#06b6d4" />` — keep as-is
- Subsequent: `<SkeletonText lines={5} />` — keep as-is
- Health loading: 4 skeleton cards in grid — keep as-is

### Table Row Pattern (cosmic theme)
**Source:** `app/src/pages/ServerManager.jsx` lines 220-370
**Apply to:** Node table view rows
```jsx
// Table row class for cosmic theme
<tr className="border-b border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.02)] transition-colors">
  <td className="px-5 py-3 font-medium">...</td>
  <td className="px-5 py-3 font-mono text-xs">...</td>
  <td className="px-5 py-3">...</td>
  ...
</tr>
```

## Metadata

**Analog search scope:** `app/src/pages/`, `app/src/pages/servers/`, `app/src/hooks/`, `app/src/store/`, `app/src/components/`, `app/src/`
**Files scanned:** 8
**Pattern extraction date:** 2026-06-14
