# Phase 81: update UI di halaman utama/dashboard app.esluce.com - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Redesign the main dashboard page at `/` and `/dashboard` (DashboardPage.jsx) with cosmic theme consistency, inline table enrichment, and improved empty states. Purely frontend changes to the existing page — no new API endpoints.

</domain>

<decisions>
## Implementation Decisions

### Summary Cards (3 cards: Servers, Billing, Agents)
- **D-01:** Minimal cosmic restyle — apply `glass-panel` + cosmic colors to existing 3 cards
- **D-02:** No MetricsCard sparklines, no resource usage cards, no new data
- **D-03:** Keep current card layout and content (icon, label, value, link)

### Cosmic Theme Restyle (whole page)
- **D-04:** Match the polish level of Phase 75's `/servers` page (ServerManagerPage)
- **D-05:** Apply `stars-bg` overlay to dashboard page container
- **D-06:** Welcome header: keep simple `<h1>` text but wrap in cosmic-styled container. No separate stats row (3 cards already show this info)
- **D-07:** Use established cosmic patterns: `glass-panel`, `border border-[var(--color-cosmic-border)]`, `focus:ring-[var(--color-cosmic-cyan)]`, status badge pattern, glow hover effects, cosmic table row transitions

### Servers & Nodes Tables
- **D-08:** Enrich with inline features: search, filter, sort per table
- **D-09:** Keep current column structure for both tables (no simplification)
- **D-10:** View button only per row — no quick start/stop actions on dashboard
- **D-11:** Cosmic restyle: glass-panel container, cosmic borders, hover effects on rows, status badges

### Layout & Content Organization
- **D-12:** Keep vertical stack layout: Header → Summary Cards → Servers table → Nodes table
- **D-13:** No tabs, no 2-column grid, no sidebar conversion
- **D-14:** Section order unchanged from current implementation

### Empty States
- **D-15:** Cosmic restyle of empty state containers (glass-panel)
- **D-16:** Add helpful links below CTA: docs.esluce.com quick start guides + feature highlights (what they can do after setup)
- **D-17:** Keep current CTA buttons ("Create your first server" / "Add your first node")

### the agent's Discretion
- Exact placement of inline search/filter controls within table headers
- Visual styling of empty state helpful links
- Sort direction indicators and default sort column
- Search debounce timing and filter behavior

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Target page
- `app/src/pages/dashboard/DashboardPage.jsx` — main dashboard page (269 lines, target for all changes)
- `app/src/pages/dashboard/WelcomeModal.jsx` — post-checkout success modal (rendered on dashboard)

### Reference pages (cosmic theme patterns)
- `app/src/pages/servers/ServerManagerPage.jsx` — reference for cosmic polish level (D-04)
- `app/src/pages/settings/SettingsPage.jsx` — shell component pattern (Phase 80 reference)
- `app/src/pages/billing/BillingPage.jsx` — cosmic theme usage bars / glass-panel reference

### Reusable components
- `app/src/components/MetricsCard.jsx` — available but NOT used per D-02
- `app/src/components/StatusBadge.jsx` — cosmic status badge pattern
- `app/src/components/SkeletonLoader.jsx` — existing skeleton loaders (SkeletonDashboard, SkeletonCard, SkeletonServerTable, SkeletonNodesTable)
- `app/src/components/TopBar.jsx` — already cosmic themed, rendered above dashboard
- `app/src/components/ToastContainer.jsx` — global toast notifications

### Styling
- `app/src/index.css` — cosmic theme CSS variables and utility classes (glass-panel, glow, status-dot, stars-bg)

### API (no new endpoints needed)
- No new API endpoints. Uses existing `serversApi.list()`, `billingApi.getCurrentSubscription()`, `useNodes()` hook.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `StatusBadge.jsx` — directly reusable for table status columns
- `SkeletonDashboard` / `SkeletonCard` / `SkeletonServerTable` / `SkeletonNodesTable` — already match dashboard layout
- `MetricsCard.jsx` — exists but explicitly NOT used (D-02)
- Cosmic CSS variables (var(--color-cosmic-*)), glass-panel utility class

### Established Patterns
- Cosmic theme: glass-panel containers, cosmic borders, cyan focus rings, glow hovers — established across Phases 75-80
- Table cosmic styling: border-gray-700 → border-cosmic-border, row hover transitions, status dot glow (from Phase 75 /servers)
- localStorage for user preferences (from Phases 74-75)
- Toast notifications via ToastContainer (global)

### Integration Points
- `/` and `/dashboard` routes in `App.jsx:84` — wraps DashboardPage
- Inline sidebar in App.jsx (hardcoded nav links) — NOT Sidebar.jsx component
- TopBar.jsx rendered above nested routes in App.jsx
- Data sources: serversApi.list(), billingApi.getCurrentSubscription(), useNodes() hook — all already called in DashboardPage

</code_context>

<specifics>
## Specific Ideas

- Table inline search/filter should follow pattern similar to Phase 75's filter/sort controls
- Helpful links in empty states: docs links + feature highlights bullet points
- All containers use glass-panel with appropriate padding (matching Phase 79 BillingPage pattern)
- Status badges use the cosmic pattern: `bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]`

</specifics>

<deferred>
## Deferred Ideas

- WebSocket-based real-time dashboard updates — future phase
- Resource utilization gauges (RAM, CPU, disk charts) — future phase
- Recent activity / notification feed on dashboard — future phase
- Sidebar.jsx replacing inline App.jsx sidebar — separate refactor

</deferred>

---

*Phase: 81-update-ui-di-halaman-utama-dashboard-app-esluce-com*
*Context gathered: 2026-06-15*
