# Phase 75: update UI https://app.esluce.com/servers - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Redesign the server list page at `/servers` (ServerManagerPage) with improved layout, sorting/filtering, quick actions, and auto-refresh. Keep the card-based UI but add a table view toggle. No new API endpoints — purely frontend changes to the existing page.

</domain>

<decisions>
## Implementation Decisions

### Card Content & Layout
- **D-01:** Keep minimal card content — name, game type, status dot, image name, node ID. No additional data on cards.
- **D-02:** Add a list/table view toggle alongside the existing card grid. Users can switch between card layout and compact table.
- **D-03:** View preference persisted to localStorage.

### Sorting & Filtering
- **D-04:** Sort by: name (default), status (running first), or last activity.
- **D-05:** Add game type filter next to the existing status filter.
- **D-06:** Filter/sort preferences persisted to localStorage.

### Quick Actions
- **D-07:** Add a Restart button alongside View and Start/Stop on each card/row.
- **D-08:** No batch/multi-select operations. Single-server actions only.

### Auto-Refresh & Real-Time
- **D-09:** Poll server list every 30s (setInterval-based refetch).
- **D-10:** Show toast notification when a server changes status (e.g., "Survival SMP is now running").
- **D-11:** WebSocket-based real-time updates deferred to a future phase.

### the agent's Discretion
- Visual styling of the toggle (icon/tab/button), exact placement of game type filter, toast placement and duration.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Server page
- `app/src/pages/servers/ServerManagerPage.jsx` — target page for this phase (card grid, search, filter, actions)
- `app/src/pages/servers/ServerDetailsPage.jsx` — server detail page (linked from cards)
- `app/src/pages/ServerManager.jsx` — legacy table-based server list (reference for table view pattern and cosmic theme styling)

### Existing Components & Patterns
- `app/src/components/TopBar.jsx` — top bar added in Phase 74 (renders across all pages)
- `app/src/store/serverStore.js` — server state store (current fetchServers/isLoading pattern)
- `app/src/hooks/useServers.js` — legacy server hooks (reference for action patterns)
- `app/src/components/StatusBadge.jsx` — status badge component used in legacy table view

### API
- No new API endpoints needed. Uses existing `useServerStore().fetchServers` for polling.

### Styling
- Cosmic theme CSS variables (var(--color-cosmic-*), var(--color-text-main), etc.)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ServerManager.jsx` table markup — directly reusable for the table view toggle (cosmic-themed table with columns: Name, Game, Host:Port, Environment, Executor, Status, Actions)
- `StatusBadge.jsx` — reusable status indicator component
- `serverStore.js` fetchServers — existing refresh method for polling

### Established Patterns
- localStorage for user preferences (used in Phase 74 for theme)
- Cosmic theme via CSS variables throughout
- Lucide icons via import pattern

### Integration Points
- `/servers` route in `App.jsx:84` — wraps `<ServerManagerPage />`
- Server cards link to `/servers/:id` for detail page

</code_context>

<specifics>
## Specific Ideas

- Table view should mirror the legacy `ServerManager.jsx` column layout (Name, Game, Host:Port, Environment, Executor, Status, Actions) with cosmic styling.
- Game type filter options: Minecraft Java, Minecraft Bedrock, PocketMine-MP, Nukkit (from available game types in the system).
- Toast notifications should use existing pattern (check if any toast component exists in codebase).

</specifics>

<deferred>
## Deferred Ideas

- WebSocket-based real-time server status updates — future phase
- Batch/multi-select server operations — future phase
- More detailed card content (resource usage, player count) — future phase

</deferred>

---

*Phase: 75-update-ui-https-app-esluce-com-servers*
*Context gathered: 2026-06-14*
