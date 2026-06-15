# Phase 76: update UI https://app.esluce.com/nodes - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Improve the Nodes management page at `/nodes` (Nodes.jsx) with enhanced UI, split-panel layout with table/card toggle, enriched node card content, and visual refresh of health metrics. No new API endpoints — purely frontend changes.

</domain>

<decisions>
## Implementation Decisions

### Layout & Node List
- **D-01:** Keep split-panel layout (node list left, detail panel right).
- **D-02:** Add table/card view toggle for the node list, consistent with Phase 75 pattern.
- **D-03:** View preference persisted to localStorage.

### Node Card Content
- **D-04:** Show: name, IP address, memory, CPU cores, status, **uptime**, and **last seen**.
- **D-05:** Keep current status emoji + text pattern.

### Search & Filtering
- **D-06:** No search or filter. Node count is typically low, not needed.

### Detail Panel
- **D-07:** Keep existing 3 tabs: Overview, API Keys, Tokens.
- **D-08:** No new tabs or sections added.
- **D-09:** Visual refresh for health metrics (same 4 data points: Status, CPU, Memory, Containers — improved presentation with bars/charts).

### the agent's Discretion
- Table view column layout (mirror node card fields)
- Visual style of health metrics refresh (progress bars, mini charts, color coding)
- Exact placement of uptime/last seen in node cards

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Node page
- `app/src/pages/Nodes.jsx` — target page (split-panel layout, node list, detail panel with tabs, modals)
- `app/src/hooks/useNodes.js` — existing node hooks (useNodes, useNodeKeys, useNodeHealth)

### Phase 75 patterns (same "update UI" series)
- `.planning/phases/75-update-ui-https-app-esluce-com-servers/75-CONTEXT.md` — table/card toggle, localStorage persistence patterns

### Existing Components
- `app/src/components/TopBar.jsx` — top bar (renders across all pages)
- `app/src/store/serverStore.js` — server state store pattern

### Styling
- Cosmic theme CSS variables

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 75's table/card toggle pattern — directly applicable to node list
- localStorage preference pattern from Phase 75

### Established Patterns
- Split-panel layout used across dashboard
- Cosmic theme via CSS variables
- Tab navigation in detail panels

### Integration Points
- `/nodes` route in App.jsx

</code_context>

<specifics>
## Specific Ideas

- Table view columns: Name, IP, Memory, CPU, Uptime, Last Seen, Status
- Health metrics visual refresh: consider progress bars for CPU/memory, color-coded status indicator
- Uptime and last seen should use relative time formatting (e.g., "2h 30m", "5m ago")

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 76-update-ui-https-app-esluce-com-nodes*
*Context gathered: 2026-06-14*
