# Phase 26: Menambahkan table informasi node di dashboard - Research

**Researched:** 2026-05-04
**Domain:** React Frontend UI - Dashboard Table Component
**Confidence:** HIGH

## Summary

This is a frontend UI task to add a node information table to the existing dashboard. The implementation pattern is already established in the codebase via the servers table. The dashboard already has the nodes table implemented (lines 182-255 in DashboardPage.jsx) using the same pattern as the servers table.

**Primary recommendation:** The implementation is already complete. Verify acceptance criteria are met and ensure the calculateUptime function handles edge cases properly.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Table columns: Name, Status (online/offline), IP Address, Uptime, Server Count
- Table location: Below servers table in dashboard
- Empty state: "No nodes found" message

### the agent's Discretion
- Implementation details (table styling, badge colors)
- Uptime calculation approach

### Deferred Ideas (OUT OF SCOPE)
- Node detail actions (future)
- Node health metrics in table
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-01 | User can see nodes table below servers table in dashboard | Verified - table exists at lines 182-240 in DashboardPage.jsx |
| REQ-02 | User can see node status (online/offline) with color-coded badges | Verified - Green for online, red for offline (lines 216-220) |
| REQ-03 | User can see node IP address | Verified - Displayed at line 222 |
| REQ-04 | User can see node uptime | Verified - calculateUptime function exists (lines 245-255) |
| REQ-05 | User can see server count per node | Verified - nodeServers filter at line 210 |
</phase_requirements>

---

## Standard Stack

### Core (Already Used)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.2.4 | UI Framework | Project standard |
| Zustand | 5.0.12 | State management | Already used for serverStore and nodes data |
| Tailwind CSS | 4.2.0 | Styling | Project standard |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| useNodes hook | Existing | Data fetching | Get nodes from API |
| serversApi | Existing | Server data | Get servers for count calculation |

**Installation:** No new packages needed - all dependencies already installed.

---

## Architecture Patterns

### Recommended Project Structure

```
app/src/
├── pages/dashboard/
│   └── DashboardPage.jsx      # Contains both tables
├── hooks/
│   └── useNodes.js            # Nodes data hook
└── store/
    └── serverStore.js         # Servers data (via useServerStore)
```

### Pattern: Table with Conditional Empty State

The codebase follows this established pattern (servers table lines 133-180, nodes table lines 182-240):

```jsx
// Empty state first
{totalNodes === 0 ? (
  <div className="bg-gray-800 rounded-lg p-12 text-center">
    <p className="text-gray-400 text-lg mb-4">No nodes found</p>
    <button onClick={() => navigate('/nodes')}>
      Add your first node
    </button>
  </div>
) : (
  // Table rendering
  <table>...</table>
)}
```

**Source:** DashboardPage.jsx lines 185-195 (empty state) and 196-238 (table)

### Pattern: Uptime Calculation

```jsx
function calculateUptime(firstSeen) {
  if (!firstSeen) return '-'
  const start = new Date(firstSeen)
  const now = new Date()
  const diffMs = now - start
  const days = Math.floor(diffMs / (1000 * 60 * 60 * 24))
  const hours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h`
  return '<1h'
}
```

**Source:** DashboardPage.jsx lines 245-255

### Pattern: Status Badge with Conditional Classes

```jsx
<span className={`px-2 py-1 rounded text-sm ${
  node.status === 'online' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'
}`}>
  {node.status || 'offline'}
</span>
```

**Source:** DashboardPage.jsx lines 215-220

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| State management | Custom useState for nodes | useNodes hook | Already exists, handles loading/error states |
| Data fetching | Manual fetch calls | getNodes from API client | Consistent with project patterns |
| Date formatting | Custom Date parsers | JavaScript Date object | Native API sufficient for simple calculations |

**Key insight:** The codebase already has all necessary abstractions. The nodes table implementation reuses existing patterns from the servers table.

---

## Common Pitfalls

### Pitfall 1: Empty Array Causes Rerenders
**What goes wrong:** Passing empty array directly to render can cause performance issues in some table libraries
**Why it happens:** `[]` creates a new array reference on every render
**How to avoid:** Check `length === 0` before rendering table, not just `data || []`
**Warning signs:** This project uses conditional rendering with `totalNodes === 0 ?` which is the correct pattern

### Pitfall 2: Null/Undefined Node Data
**What goes wrong:** Table crashes when node has null fields
**Why it happens:** API might return incomplete node objects
**How to avoid:** Use optional chaining (`node?.name`) and default values
**Warning signs:** Current implementation at line 214 uses `node?.name` - good protection

### Pitfall 3: Uptime Calculation with Invalid Dates
**What goes wrong:** "Invalid Date" displayed when first_seen is malformed
**Why it happens:** API returns unexpected date format
**How to avoid:** Check `if (!firstSeen) return '-'` before processing
**Warning signs:** Current implementation checks `if (!firstSeen)` at line 246 - good

### Pitfall 4: Server Count Calculation on Every Render
**What goes wrong:** Filtering servers for each node on every render is inefficient
**Why it happens:** `(servers || []).filter(s => s.node_id === node.id).length` runs for each node, each render
**How to avoid:** Memoize server count per node using useMemo or pre-calculate
**Warning signs:** Current implementation recalculates on every render (line 210)

---

## Code Examples

Verified patterns from codebase:

### Nodes Table Structure
```jsx
// Source: DashboardPage.jsx lines 196-238
<div className="bg-gray-800 rounded-lg overflow-hidden">
  <table className="w-full">
    <thead className="bg-gray-700">
      <tr>
        <th className="px-6 py-3 text-left text-gray-400 text-sm">Name</th>
        <th className="px-6 py-3 text-left text-gray-400 text-sm">Status</th>
        <th className="px-6 py-3 text-left text-gray-400 text-sm">IP Address</th>
        <th className="px-6 py-3 text-left text-gray-400 text-sm">Uptime</th>
        <th className="px-6 py-3 text-left text-gray-400 text-sm">Servers</th>
        <th className="px-6 py-3 text-left text-gray-400 text-sm">Actions</th>
      </tr>
    </thead>
    <tbody>
      {nodes.map(node => {
        const nodeServers = (servers || []).filter(s => s.node_id === node.id).length
        const uptime = node.first_seen ? calculateUptime(node.first_seen) : '-'
        return (
          <tr key={node.id} className="border-t border-gray-700">
            <td className="px-6 py-4 text-white">{node?.name}</td>
            ...
          </tr>
        )
      })}
    </tbody>
  </table>
</div>
```

### Empty State Pattern
```jsx
// Source: DashboardPage.jsx lines 185-195
{totalNodes === 0 ? (
  <div className="bg-gray-800 rounded-lg p-12 text-center">
    <p className="text-gray-400 text-lg mb-4">No nodes found</p>
    <button
      onClick={() => navigate('/nodes')}
      className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
    >
      Add your first node
    </button>
  </div>
) : ( ... )}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No nodes display | Table with status, uptime, server count | Pre-existing | Users can see node overview |
| Only node count in card | Full table below servers | Pre-existing | More detailed node information |

**Deprecated/outdated:**
- None - this feature was not previously implemented

---

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Node API returns first_seen and last_seen for uptime calculation | Uptime column | LOW - confirmed by CONTEXT.md mentioning these fields |
| A2 | Node status is either 'online' or 'offline' | Status badge | LOW - confirmed by CONTEXT.md |
| A3 | Server data includes node_id field for filtering | Server count | MEDIUM - not explicitly confirmed but used in existing servers table (line 163) |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions

1. **Should server count be memoized?**
   - What we know: Currently calculates on every render (line 210)
   - What's unclear: Performance impact with many nodes
   - Recommendation: Acceptable for small node counts; optimize if nodes > 20

2. **Should nodes table show loading state?**
   - What we know: useNodes has loading state but dashboard doesn't display it
   - What's unclear: User expectation for loading UX
   - Recommendation: Add loading skeleton if UX requires it

---

## Environment Availability

> Step 2.6: SKIPPED (no external dependencies identified)

This is a frontend-only UI task. No external tools, services, or runtimes beyond the existing project dependencies are required.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None detected |
| Config file | N/A |
| Quick run command | N/A |
| Full suite command | N/A |

**Note:** Project has no test infrastructure (no test/ spec files found). Manual verification required.

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-01 | Nodes table visible below servers table | Manual | Open /dashboard in browser | N/A |
| REQ-02 | Status badges show online/offline | Manual | Check node row display | N/A |
| REQ-03 | IP address displayed | Manual | Check IP column | N/A |
| REQ-04 | Uptime calculated and shown | Manual | Check uptime column | N/A |
| REQ-05 | Server count per node | Manual | Check servers column | N/A |

### Wave 0 Gaps
- [ ] No test infrastructure detected - verification must be manual

---

## Security Domain

> Skip this section - this is a frontend display-only task with no security implications. No authentication, authorization, or data validation changes involved.

---

## Sources

### Primary (HIGH confidence)
- DashboardPage.jsx - Current implementation of nodes table
- useNodes.js - Nodes data hook structure
- serverStore.js - Server state management pattern
- CONTEXT.md - Phase requirements and constraints

### Secondary (MEDIUM confidence)
- Web search: React table best practices 2025

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already in project
- Architecture: HIGH - Pattern already established in codebase
- Pitfalls: HIGH - Implementation follows correct patterns

**Research date:** 2026-05-04
**Valid until:** 90 days (stable - no library changes needed)