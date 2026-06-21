# Phase 84: perbaiki layout yang janggal ataupun tidak bagus di app.esluce.com secara menyeluruh - Context

**Gathered:** 2026-06-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Comprehensive UI layout polish across all pages of app.esluce.com. Fix spacing, alignment, consistency issues — sidebar refinement, standardized page layouts, unified table patterns, and consistent header hierarchy. No new features, no new pages, no backend changes.

</domain>

<decisions>
## Implementation Decisions

### Sidebar Refinement
- **D-01:** Smaller logo + closer text spacing — reduce the sidebar logo (`w-16 h-16`) so the icon and "Escluse" text feel like one cohesive unit, not two separate elements
- **D-02:** Narrower sidebar width — reduce `w-64` to a more compact width (e.g., `w-56` or `w-48`) so content (especially dashboard tables) has more room
- **D-03:** Add active state highlighting — sidebar nav items must show which page is currently active (e.g., highlighted background, colored accent indicator)

### Layout Consistency (comprehensive pass)
- **D-04:** Standardize page padding — all pages use the same padding value
- **D-05:** Unify table patterns — consistent header style, row height, border pattern, and empty state across all pages that use tables
- **D-06:** Standardize header hierarchy — same heading size pattern used across all pages (keep current sizes, just make them consistent)
- **D-07:** Page width — mixed approach: dashboard and data-heavy pages keep full-width layout (tables need space), other pages use a constrained max-width container

### the agent's Discretion
- Exact logo size reduction amount
- Exact sidebar width value and collapsed state adjustments
- Active state visual style (background color, left border indicator, text color)
- Max-width value for container
- Padding standardization value (p-6, p-8, or other)
- Table header visual details and row hover patterns
- Implementation order (which pages to fix first)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Target Files
- `app/src/app/App.jsx` — Sidebar implementation (inline aside, lines 55-75), main layout wrapper
- `app/src/components/TopBar.jsx` — TopBar component rendered above page content
- `app/src/index.css` — Cosmic theme CSS variables, utility classes

### Pages to Audit
- `app/src/pages/dashboard/DashboardPage.jsx` — Dashboard layout (tables, cards, empty states)
- `app/src/pages/servers/ServerManagerPage.jsx` — Server list (card/list views, filter bar, table)
- `app/src/pages/Nodes.jsx` — Nodes page (largest page, 56KB)
- `app/src/pages/billing/BillingPage.jsx` — Billing page (plan cards, usage bars, payment table)
- `app/src/pages/settings/SystemSettings.jsx` — Settings page
- `app/src/pages/settings/SettingsPage.jsx` — Settings shell component
- `app/src/pages/Alerts.jsx` — Alerts page
- `app/src/pages/Console.jsx` — Console page
- `app/src/pages/auth/LoginPage.jsx` — Auth forms
- `app/src/pages/auth/RegisterPage.jsx` — Registration form
- `app/src/pages/templates/TemplateLibraryPage.jsx` — Template library
- `app/src/pages/templates/TemplateCreatePage.jsx` — Template creation form
- `app/src/pages/templates/ModBrowserPage.jsx` — Mod browser

### Theme Reference
- `app/src/index.css` — CSS variables for colors, glass-panel, glow utilities
- `app/src/store/uiStore.js` — Theme state management

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `StatusBadge.jsx` — Reusable status badge component (used across tables)
- `SkeletonLoader.jsx` — Skeleton loaders for loading states
- `glass-panel` utility class — Reusable container pattern from cosmic theme
- Semantic CSS variables (`--color-bg-*`, `--color-text-*`, `--color-cosmic-*`) — Consistent theming foundation

### Established Patterns
- Cosmic theme: glass-panel containers, cosmic borders, cyan accent buttons
- Page layout: each page is a self-contained component with its own padding/container
- Table pattern: glass-panel wrapper, thead with `bg-[rgba(255,255,255,0.02)]`, border-b rows
- Empty states: glass-panel centered content with CTA button and helpful links

### Integration Points
- `App.jsx` sidebar — inline hardcoded `<a>` tags, no icons, no active state, manual `w-64` / `w-16` collapse
- All page components — need consistent padding, header hierarchy, and container patterns
- Tables across DashboardPage, ServerManagerPage, Nodes, BillingPage — need unified styling

</code_context>

<specifics>
## Specific Ideas

- Sidebar: logo currently `w-16 h-16` with `mr-1` spacing — reduce to `w-8 h-8` or `w-10 h-10` range so it sits naturally next to the brand text
- Active state: use left border accent (3px `var(--color-cosmic-cyan)` border-left) or subtle background highlight like the existing card hover patterns
- Dashboard tables: the server table has 8 columns which can feel cramped with a 256px sidebar — narrower sidebar directly helps this
- Page padding: `p-8` seems to be the emerging standard (dashboard, billing) — align other pages to match
- Table empty states: ensure all tables show consistent "No items match your filters" messaging

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 84-perbaiki-layout-yang-janggal-ataupun-tidak-bagus-di-app-eslu*
*Context gathered: 2026-06-16*
