# Phase 75: Update UI https://app.esluce.com/servers - Pattern Map

**Mapped:** 2026-06-14
**Files analyzed:** 1 new/modified
**Analogs found:** 9 / 9 with matches

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `app/src/pages/servers/ServerManagerPage.jsx` | component | CRUD + request-response | `app/src/pages/ServerManager.jsx` | role-match (same page role, different view style) |
| `app/src/pages/servers/ServerManagerPage.jsx` (table view section) | component | CRUD | `app/src/pages/ServerManager.jsx` lines 220-372 | exact (table view markup) |
| `app/src/pages/servers/ServerManagerPage.jsx` (restart modal) | component | request-response | `app/src/pages/servers/ServerDetailsPage.jsx` lines 257-288 | role-match (inline confirmation modal) |
| `app/src/pages/servers/ServerManagerPage.jsx` (polling) | component | event-driven | `app/src/hooks/useServers.js` lines 27-83 | partial (WebSocket pattern differs from polling) |

## Pattern Assignments

### `app/src/pages/servers/ServerManagerPage.jsx` (component, CRUD + request-response)

The target file. Major modifications to add: table view toggle, sort/filter controls, restart button + modal, 30s polling with status change detection, localStorage persistence. Current file is 199 lines; will grow to ~450 lines.

---

#### Imports pattern

**Analog:** `app/src/pages/servers/ServerManagerPage.jsx` lines 1-6 (existing imports) + `app/src/pages/dashboard/WelcomeModal.jsx` line 5 (lucide pattern)

**Existing imports** (keep what's used):
```javascript
import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { useServerStore } from '../../store/serverStore'
import { api, serversApi } from '../../lib/api'
import CreateServerModal from '../../features/server/CreateServerModal'
import { SkeletonCard, EscluseSpinner } from '../../components/SkeletonLoader'
```

**New imports to add** (lucide icons for view toggle + restart):
```javascript
// lucide import pattern from app/src/pages/dashboard/WelcomeModal.jsx line 5:
import { CheckCircle2, Sparkles, X } from 'lucide-react'

// What we need for Phase 75 (lucide-react v1.18.0):
import { LayoutGrid, List, RotateCcw, ArrowUpDown } from 'lucide-react'
```

**StatusBadge import** (for table view):
```javascript
import StatusBadge from '../../components/StatusBadge'
// Pattern from: app/src/pages/ServerManager.jsx line 4
```

**useUIStore import** (for toast notifications):
```javascript
import { useUIStore } from '../../store/uiStore'
// Pattern from: app/src/store/uiStore.js (exported named, used across 20+ files)
```

---

#### Table View Pattern (new table toggle within ServerManagerPage.jsx)

**Analog:** `app/src/pages/ServerManager.jsx` lines 220-372 (cosmic-themed table)

**View toggle state + localStorage persistence** (inspired by WorkspaceContext.jsx lines 7-13):
```javascript
// Analog: app/src/context/WorkspaceContext.jsx lines 6-13
// Use useState lazy init + localStorage for view preference
const [viewMode, setViewMode] = useState(() => {
  return localStorage.getItem('serverViewMode') || 'card'
})

// On change handler
const handleViewModeChange = (mode) => {
  setViewMode(mode)
  localStorage.setItem('serverViewMode', mode)
}
```

**View toggle button group** (from RESEARCH.md Pattern — cosmic-themed segmented control):
```jsx
{/* Analog: RESEARCH.md view toggle pattern - uses lucide LayoutGrid + List */}
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

**Table view markup** (directly adapted from `app/src/pages/ServerManager.jsx` lines 220-372):
```jsx
{/* Analog: app/src/pages/ServerManager.jsx lines 320-369 */}
<table className="w-full text-sm">
  <thead>
    <tr className="text-left text-[var(--color-text-muted)] border-b border-[var(--color-cosmic-border)]">
      <th className="px-5 py-3 font-medium">Name</th>
      <th className="px-5 py-3 font-medium">Game</th>
      <th className="px-5 py-3 font-medium">Host:Port</th>
      <th className="px-5 py-3 font-medium">Image</th>
      <th className="px-5 py-3 font-medium">Node</th>
      <th className="px-5 py-3 font-medium">Status</th>
      <th className="px-5 py-3 font-medium">Actions</th>
    </tr>
  </thead>
  <tbody>
    {filteredSorted.map((server) => (
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
            {/* View, Restart, Start/Stop buttons */}
          </div>
        </td>
      </tr>
    ))}
  </tbody>
</table>
```

#### Sort & Filter Controls

**Analog:** `app/src/pages/servers/ServerManagerPage.jsx` lines 137-146 (existing select pattern) — cosmic restyle from `ServerManager.jsx`

**Sort dropdown** (new — added alongside existing status filter):
```jsx
{/* Follows existing select styling from ServerManagerPage.jsx lines 137-146, 
    ported to cosmic theme from ServerManager.jsx lines 214-215 */}
<select
  value={sortMode}
  onChange={(e) => {
    setSortMode(e.target.value)
    localStorage.setItem('serverSortMode', e.target.value)
  }}
  className="bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] px-4 py-2 rounded-lg"
>
  <option value="name">Sort: Name A-Z</option>
  <option value="status">Sort: Running First</option>
  <option value="activity">Sort: Last Activity</option>
</select>
```

**Game type filter** (new — placed after status filter):
```jsx
<select
  value={gameFilter}
  onChange={(e) => {
    setGameFilter(e.target.value)
    localStorage.setItem('serverGameFilter', e.target.value)
  }}
  className="bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] px-4 py-2 rounded-lg"
>
  <option value="all">All Games</option>
  <option value="minecraft_java">Minecraft Java</option>
  <option value="minecraft_bedrock">Minecraft Bedrock</option>
  <option value="pocketmine_mp">PocketMine-MP</option>
  <option value="nukkit">Nukkit</option>
</select>
```

**Sort utility function** (inline in ServerManagerPage.jsx):
```javascript
// Pattern from RESEARCH.md — pure client-side sort
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

---

#### Restart Button + Confirmation Modal

**Analog for restart action:** `app/src/lib/api.js` line 108 (`serversApi.restart(id)`)

**Analog for modal pattern:** `app/src/pages/servers/ServerDetailsPage.jsx` lines 257-288

**Restart button** (on card and table row):
```jsx
{/* Analog: lucide RotateCcw icon for restart */}
<button
  onClick={() => setRestartServer(server)}
  disabled={['starting', 'stopping', 'crashed'].includes(server.status)}
  aria-label={`Restart ${server.name}`}
  title="Restart"
  className="..."
>
  <RotateCcw size={16} />
</button>
```

**Restart confirmation modal** (adapted from `ServerDetailsPage.jsx` lines 257-288, ported to cosmic theme):
```jsx
{/* Analog: app/src/pages/servers/ServerDetailsPage.jsx lines 257-288 */}
{restartServer && (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div className="bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-lg p-6 max-w-md">
      <h3 className="text-[var(--color-text-main)] text-lg font-semibold mb-4">Restart Server?</h3>
      <p className="text-[var(--color-text-muted)] mb-6">
        Are you sure you want to restart <strong>{restartServer.name}</strong>? This will temporarily disconnect all players.
      </p>
      <div className="flex gap-4 justify-end">
        <button
          onClick={() => setRestartServer(null)}
          className="px-4 py-2 text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]"
        >
          Cancel
        </button>
        <button
          onClick={async () => {
            const server = restartServer
            setRestartServer(null)
            try {
              await serversApi.restart(server.id)
              addToast({ type: 'info', message: `Restarting ${server.name}...` })
              fetchServers()
            } catch (err) {
              addToast({ type: 'error', message: `Failed to restart ${server.name}: ${err.message}` })
            }
          }}
          className="bg-[var(--color-cosmic-cyan)] text-[var(--color-deep-space)] px-4 py-2 rounded-lg font-semibold hover:brightness-110"
        >
          Restart
        </button>
      </div>
    </div>
  </div>
)}
```

#### 30s Polling + Status Change Detection

**Analog for ref-based polling:** RESEARCH.md Pattern 1 (no direct codebase analog — this is new)

**Analog for addToast:** `app/src/store/uiStore.js` lines 21-27 (used across 20+ files)

```javascript
// Pattern: useRef-based polling with cleanup (from RESEARCH.md)
// No existing component does polling — this is novel for the codebase
const prevServersRef = useRef([])
const { addToast } = useUIStore()
const { servers, fetchServers } = useServerStore()

useEffect(() => {
  const interval = setInterval(async () => {
    const oldServers = prevServersRef.current
    await fetchServers()
    const newServers = useServerStore.getState().servers
    // Detect status changes and show toasts
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

// Initialize prevServersRef after first fetch
useEffect(() => {
  if (servers.length > 0) {
    prevServersRef.current = servers
  }
}, [servers])
```

#### Error Handling Pattern (existing, keep as-is)

**Analog:** `app/src/pages/servers/ServerManagerPage.jsx` lines 52-68 (existing start/stop handler pattern)

The existing error handling in ServerManagerPage uses try/catch with `console.error`. The research recommends upgrading to `addToast` for user-facing errors:

```javascript
// Analog: app/src/pages/servers/ServerManagerPage.jsx lines 52-68 (existing pattern)
// UPGRADE: replace console.error with addToast
const handleStartStop = async (server) => {
  try {
    // ... existing logic
    fetchServers()
  } catch (err) {
    addToast({ type: 'error', message: `Failed to toggle server: ${err.message}` })
  }
}
```

---

### `app/src/pages/ServerManager.jsx` (component, CRUD) — REFERENCE ONLY

**Not modified.** Used as the reference for:
- Table markup pattern (lines 220-372)
- Cosmic-themed select/input styling (lines 214-215)
- StatusBadge integration (line 342)
- Table action buttons with cosmic theme colors (lines 344-368)
- `var(--color-cosmic-*)` variable usage throughout

---

## Shared Patterns

### Toast Notifications
**Source:** `app/src/store/uiStore.js` lines 21-27
**Apply to:** All action handlers (restart, start/stop) + polling status change detection
```javascript
// Pattern from app/src/store/uiStore.js lines 21-27
import { useUIStore } from '../../store/uiStore'

const { addToast } = useUIStore()

// Usage:
addToast({ type: 'info', message: 'Server is now running' })   // Status change
addToast({ type: 'success', message: 'Server restarted' })       // Success
addToast({ type: 'error', message: 'Failed to restart server' }) // Error
// Toast auto-dismisses after 5000ms (duration from uiStore.js line 26)
```

### localStorage Persistence
**Source:** `app/src/context/WorkspaceContext.jsx` lines 6-13
**Apply to:** viewMode, sortMode, gameFilter preferences
```javascript
// Pattern from WorkspaceContext.jsx lines 6-13
// On mount — read from localStorage with lazy init
const [viewMode, setViewMode] = useState(() => {
  return localStorage.getItem('serverViewMode') || 'card'
})

// On change — write to localStorage
const handleViewModeChange = (mode) => {
  setViewMode(mode)
  localStorage.setItem('serverViewMode', mode)
}
```

### Cosmic Theme Styling
**Source:** `app/src/index.css` lines 4-19, `app/src/pages/ServerManager.jsx`
**Apply to:** All new UI elements in ServerManagerPage.jsx
```css
/* CSS variables from app/src/index.css */
var(--color-deep-space)      /* #080b15 — page background */
var(--color-cosmic-card)     /* rgba(255,255,255,0.03) — card/surface bg */
var(--color-cosmic-border)   /* rgba(255,255,255,0.08) — borders */
var(--color-text-main)       /* #e2e8f0 — primary text */
var(--color-text-muted)      /* #64748b — secondary text */
var(--color-cosmic-cyan)     /* #0ddff2 — accent, active states */
var(--color-cosmic-green)    /* #10b981 — success, running */
var(--color-cosmic-red)      /* #ef4444 — destructive, error */
var(--color-cosmic-orange)   /* #f59e0b — warning, sleeping */
```

### Server Action via serversApi
**Source:** `app/src/lib/api.js` line 108
**Apply to:** Restart button handler
```javascript
// Pattern from app/src/lib/api.js lines 100-116
import { serversApi } from '../../lib/api'

// Available methods (no need to use raw api.post):
serversApi.restart(id)       // POST /api/v1/servers/:id/restart  (line 108)
serversApi.start(id)         // POST /api/v1/servers/:id/start
serversApi.stop(id)          // POST /api/v1/servers/:id/stop
serversApi.sleep(id)         // POST /api/v1/servers/:id/sleep
serversApi.wake(id)          // POST /api/v1/servers/:id/wake
```

### StatusBadge Component
**Source:** `app/src/components/StatusBadge.jsx` (38 lines)
**Apply to:** Table view — each server row shows StatusBadge with `status` and `autoWake` props
```jsx
// Pattern from app/src/components/StatusBadge.jsx
import StatusBadge from '../../components/StatusBadge'

<StatusBadge status={server.status} autoWake={server.auto_wake} />
// Handles: running (green), crashed (red), degraded (orange),
// starting (cyan pulse), stopped+sleeping (orange/cyan),
// default (gray)
```

### Loading State Pattern
**Source:** `app/src/pages/servers/ServerManagerPage.jsx` lines 71-113 (existing)
**Apply to:** Keep existing skeleton loading pattern. Update skeleton backgrounds from `bg-gray-*` to cosmic variables for consistency if needed, but the existing pattern works.

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns instead):

| File Section | Role | Data Flow | Reason |
|-------------|------|-----------|--------|
| ServerManagerPage.jsx — 30s polling | component | event-driven/polling | No existing polling pattern in codebase; WebSocket (useServers.js) is different. Use RESEARCH.md Pattern 1. |
| ServerManagerPage.jsx — status change detection | component | event-driven | Novel pattern — compare server arrays between polls using useRef. Use RESEARCH.md Pattern 1. |
| ServerManagerPage.jsx — sort utility | utility | transform | Inline sort function; no existing sort utility reference. Use RESEARCH.md sortServers function. |

## Metadata

**Analog search scope:** `app/src/pages/servers/`, `app/src/pages/`, `app/src/store/`, `app/src/components/`, `app/src/hooks/`, `app/src/lib/`, `app/src/context/`
**Files scanned:** 12
**Pattern extraction date:** 2026-06-14
