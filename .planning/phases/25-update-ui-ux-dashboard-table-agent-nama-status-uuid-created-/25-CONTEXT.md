# Phase 25: update UI/UX dashboard - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Redesign dashboard UI/UX with new card layout, enhanced tables for agents and servers, personalized welcome message, and search/pagination.

</domain>

<decisions>
## Implementation Decisions

### Welcome Message
- **D-01:** If account created <= 2 days ago → "Welcome, {user}!" (new user greeting)
- **D-02:** If account created > 2 days ago → "Welcome back, {user}!"
- **Decision:** Account age logic for greeting personalization

### Dashboard Cards
- **D-03:** Exactly 3 cards fixed: Servers, Billing, Agents
- **D-04:** Server card shows: total servers, running servers count
- **D-05:** Billing card shows: subscription status, days remaining, renewal date
- **D-06:** Agent card shows: total agents, online agents count

### Agent Table (Nodes page)
- **D-07:** Columns: Name, Status (with colored dots), UUID, Created (date), Uptime (how long alive), Connected servers count, Actions (3-dot menu)
- **D-08:** Use existing NodeDetails component pattern for detail view

### Server Table (Servers page)
- **D-09:** Add columns: Game type, IP address, Port, Domain
- **D-10:** Keep existing columns (Name, Status, Actions)
- **D-11:** 3-dot menu for actions dropdown

### Search & Pagination
- **D-12:** Inline search input in list header (not full-width top bar)
- **D-13:** Pagination for both agents list and servers list
- **D-14:** Use existing pagination pattern from other pages

### Agent's Discretion
- Exact implementation of 3-dot menu actions (reuse existing patterns)
- Specific pagination UI (page numbers vs "Load more" button)
- Exact uptime calculation format

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing UI Pattern Files
- `app/src/pages/Nodes.jsx` — Node list with inline search, card selection pattern
- `app/src/pages/servers/ServerManagerPage.jsx` — Server list with search/filter
- `app/src/components/Sidebar.jsx` — Navigation structure
- `app/src/pages/dashboard/DashboardPage.jsx` — Current dashboard (to redesign)
- `app/src/hooks/useNodes.js` — Node data fetching
- `app/src/hooks/useServers.js` — Server data fetching

### UI Components to Reuse
- `app/src/components/Sidebar.jsx` — Navigation with ops/dev mode toggle
- StatusBadge component for status indicators
- MetricsCard component for dashboard cards
- Existing search/filter patterns in ServerManagerPage

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- StatusBadge component: Used for node/server status display
- MetricsCard component: Can be reused for dashboard cards
- Nodes.jsx: Node list pattern with detail panel
- ServerManagerPage.jsx: Search + filter + grid layout

### Established Patterns
- Ops/Dev mode toggle in Sidebar (use as-is)
- Search input with filter dropdown in ServerManagerPage
- Card grid layout for servers (3 columns responsive)
- 3-dot menu for actions (existing pattern or new)

### Integration Points
- DashboardPage.jsx: Main entry point for redesign
- Nodes.jsx: Agent table modifications
- ServerManagerPage.jsx: Server table additions
- Sidebar already has navigation structure

</code_context>

<specifics>
## Specific Ideas

- Welcome greeting: Uses user creation date to determine "new user" vs "welcome back"
- Card layout: 3 cards in a row (responsive: 1 on mobile, 2 on tablet, 3 on desktop)
- Agent 3-dot menu: View details, Generate key, Delete
- Server 3-dot menu: View, Start/Stop, Delete (reuse existing)
- Uptime: "2 hours", "3 days", "2 weeks" format (human-readable)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 25-update-ui-ux-dashboard*
*Context gathered: 2026-04-16*