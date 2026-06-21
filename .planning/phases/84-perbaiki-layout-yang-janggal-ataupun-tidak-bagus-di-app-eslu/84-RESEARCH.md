# Phase 84: Layout Fix Research

> **Question answered:** "What do I need to know to PLAN this phase well?"

## 1. Current Layout State

### 1.1 Sidebar (App.jsx lines 56-75)

| Aspect | Current | Issue |
|--------|---------|-------|
| Logo size | `w-16 h-16` (64x64px) | Too large; creates awkward gap with "Escluse" text |
| Logo-text spacing | `mr-1` | Insufficient; with large logo, text looks disconnected |
| Sidebar width | `w-64` (256px) | Makes 8-column server tables cramped |
| Collapsed width | `w-16` (64px) | Only shows hamburger; no icon-only nav |
| Active state | None | No visual indicator of current page |
| Nav items | Plain `<a>` tags | No React Router `NavLink` — no `active` class possible |
| Icons | None | Text-only nav; no visual scanning aid |
| Collapse button | ☰ hamburger | Should use SVG/render icon |

### 1.2 Page Padding Inconsistency

| Page | Padding | Notes |
|------|---------|-------|
| DashboardPage | `p-8` | Content area, not page wrapper |
| SettingsPage | `p-6` | Root div |
| Alerts | `px-8 py-6` | Header uses standalone style |
| Console | flex-1 header `px-8 py-6` | Header + body flex layout |
| BillingPage | `p-8` | Content area |
| ServerManagerPage | `p-8` | Content area |
| Nodes | Unknown | 56KB file |
| auth/LoginPage | Unknown | Different layout entirely |
| auth/RegisterPage | Unknown | Different layout |

### 1.3 Heading Hierarchy

| Page | h1 | h2 |
|------|----|----|
| Dashboard | `text-3xl font-semibold` | `text-2xl font-semibold` |
| Settings | `text-2xl font-semibold` | — |
| Alerts | `text-2xl font-bold` | — |
| Console | `text-2xl font-bold` | — |
| ServerManager | Need to check | Need to check |
| Billing | Need to check | Need to check |

### 1.4 Table Patterns

- **Dashboard**: Wrapped in `glass-panel`, thead uses `bg-[rgba(255,255,255,0.02)]`, rows use `border-b border-[var(--color-cosmic-border)]`
- **ServerManagerPage**: Card/table views with toggle. Table view matches cosmic pattern
- **Nodes**: Large page (900 lines, 56KB) — likely custom table styling
- **Billing**: Payment history table with its own style

### 1.5 TopBar

- Height: `h-14`, border-bottom, flex layout
- Contains: "Escluse Dashboard" h1, alpha badge, theme toggle, bell icon, user dropdown
- Issue: Page-specific headers render below TopBar, creating duplicate headers (Dashboard has both TopBar "Escluse Dashboard" and page content "Welcome back")

## 2. Key Files Inventory

| File | Role | Lines |
|------|------|-------|
| `app/src/app/App.jsx` | Sidebar + layout wrapper | 107 |
| `app/src/components/TopBar.jsx` | Top navigation bar | 120 |
| `app/src/index.css` | CSS variables + utilities | 347 |
| `app/src/pages/dashboard/DashboardPage.jsx` | Dashboard | 488 |
| `app/src/pages/servers/ServerManagerPage.jsx` | Server list | 431 |
| `app/src/pages/Nodes.jsx` | Nodes page | 900 |
| `app/src/pages/billing/BillingPage.jsx` | Billing | 532 |
| `app/src/pages/settings/SettingsPage.jsx` | Settings shell | 63 |
| `app/src/pages/Alerts.jsx` | Alert rules/history | 153 |
| `app/src/pages/Console.jsx` | Terminal console | 78 |
| `app/src/pages/auth/LoginPage.jsx` | Login form | 138 |
| `app/src/pages/auth/RegisterPage.jsx` | Registration | ~140 |
| `app/src/pages/templates/TemplateLibraryPage.jsx` | Template library | ~200 |
| `app/src/pages/templates/TemplateCreatePage.jsx` | Template creation | ~150 |
| `app/src/pages/templates/ModBrowserPage.jsx` | Mod browser | ~200 |

## 3. Recommended Approach

### Wave 1: Sidebar Refinement (D-01, D-02, D-03)
**Files:** `app/src/app/App.jsx`

1. Reduce logo from `w-16 h-16` to `w-10 h-10` or `w-8 h-8` range
2. Adjust sidebar width from `w-64` to `w-56` or `w-48`
3. Convert `<a>` tags to React Router `<NavLink>` for active state
4. Add active state styling (left border accent or background highlight)
5. Add SVG icons next to nav items
6. Improve collapsed state — show icon-only nav instead of just hamburger
7. Replace hamburger with SVG icon

### Wave 2: Layout Consistency Pass (D-04, D-05, D-06)
**Files:** All page components

1. Standardize page padding — adopt `p-8` as the standard (used by Dashboard, Billing, ServerManager)
2. Unify header hierarchy — standardize h1/h2 sizes
3. Create reusable container wrapper component or CSS utility if pattern repeats enough
4. Fix TopBar/page-header duplication

### Wave 3: Table Unification (D-05)
**Files:** Pages with tables (DashboardPage, Nodes, BillingPage, Alerts)

1. Audit all tables for consistent thead style, row height, border pattern
2. Standardize empty state messaging
3. Ensure consistent row hover patterns

## 4. Dependencies

- **No backend changes** — purely frontend CSS/JSX
- Depends on Phase 82 theme variables being correctly applied (semantic colors should be ready)
- Phase 75-81 UI updates should provide context on patterns already established

## 5. Risks

- Nodes.jsx at 900 lines (56KB) may have nested layout complexity — avoid restructuring, just apply surface-level consistency
- Changing sidebar from `<a>` to `<NavLink>` requires careful testing that routing still works (usePathname hook)
- Collapsed state redesign could affect layout calculations — ensure `w-16` still works with new icon nav
- Light mode styles (Phase 82) must be considered — sidebar changes should respect both themes

## 6. Code Architecture Notes

### Reactive sidebar state (App.jsx):
```jsx
const [sidebarOpen, setSidebarOpen] = useState(true)
const toggleSidebar = () => setSidebarOpen(!sidebarOpen)
```
Wired to `localStorage` — maintain this behavior.

### Current padding pattern:
- Most pages use a `<div className="p-8">` or `<div className="p-6">` wrapper
- Some pages (Alerts, Console) use `<header>` + flex layout instead
- Standardizing to `p-8` is the simplest approach — minimal diff per page

### Table pattern (from DashboardPage):
```jsx
<div className="glass-panel overflow-hidden border border-[var(--color-cosmic-border)]">
  <table className="w-full">
    <thead className="bg-[rgba(255,255,255,0.02)]">
      <tr><th className="px-4 py-3 text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider">...</th></tr>
    </thead>
    <tbody className="divide-y divide-[var(--color-cosmic-border)]">
      <tr className="hover:bg-[rgba(255,255,255,0.02)]"><td className="px-4 py-3">...</td></tr>
    </tbody>
  </table>
</div>
```

### NavLink conversion pattern:
```jsx
// Before:
<a href="/servers" className="...">Servers</a>
// After:
import { NavLink } from 'react-router-dom'
<NavLink to="/servers" className={({ isActive }) => `... ${isActive ? 'bg-[var(--color-cosmic-cyan)]/10 border-l-2 border-[var(--color-cosmic-cyan)]' : ''}`}>Servers</NavLink>
```
