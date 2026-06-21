---
phase: 84
slug: perbaiki-layout-yang-janggal-ataupun-tidak-bagus-di-app-eslu
status: approved
shadcn_initialized: false
preset: none
created: 2026-06-16
---

# Phase 84 — UI Design Contract

> Visual and interaction contract for Phase 84: comprehensive layout polish across all pages of app.esluce.com.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (Tailwind CSS + custom theme variables) |
| Preset | not applicable |
| Component library | none (custom components) |
| Icon library | lucide-react |
| Font | Inter (body), Fira Code (mono) |

---

## Scope of Work

This phase covers **layout structure only** — no new components, no new pages, no backend changes. Targets:

1. Sidebar refinement (logo size, width, active state, icons, NavLink conversion)
2. Page padding standardization (`p-8` standard)
3. Header hierarchy unification (same heading pattern everywhere)
4. Table pattern consistency (thead, row height, empty states)
5. TopBar/page-header deduplication

---

## Spacing Scale

Existing CSS variables (`--color-*`) + Tailwind spacing. No new spacing tokens needed.

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Icon gaps, inline padding |
| sm | 8px | Close element spacing (mr-1 logo→text gap → mr-2) |
| md | 16px | Default element spacing |
| lg | 24px | Section padding, card gaps |
| xl | 32px | Layout gaps, page padding standard (`p-8`) |
| 2xl | 48px | Major section breaks |
| 3xl | 64px | Page-level spacing |

Exceptions:
- Logo→text gap: currently `mr-1` (4px) — increase to `mr-2` (8px) for tighter visual unit
- Sidebar collapsed `w-16` (64px) — maintain for icon-only nav

---

## Typography

| Role | Size | Weight | Line Height |
|------|------|--------|-------------|
| Body | 14px | normal (400) | 1.5 |
| Label | 12px | medium (500) | 1.4 |
| Heading h1 | 30px (`text-3xl`) | semibold (600) | 1.3 |
| Heading h2 | 24px (`text-2xl`) | semibold (600) | 1.3 |
| Heading h3 | 18px (`text-lg`) | semibold (600) | 1.3 |
| Table header | 12px | medium (500) | 1.4 |

**Standardization rule for this phase:** All pages must use the same h1/h2 size pattern. Current sizes are kept (no size changes), just made consistent across all pages. h1 = `text-3xl font-semibold`, h2 = `text-2xl font-semibold`. Any page using `text-2xl font-bold` for h1 must be updated to match.

---

## Color

No new colors introduced. All existing CSS variables reused.

| Role | Value | Usage |
|------|-------|-------|
| Dominant (60%) | `--color-deep-space` (#080b15) | Page backgrounds |
| Secondary (30%) | `--color-nebula` (#0d0f1a) | Sidebar, cards |
| Accent (10%) | `--color-cosmic-cyan` (#0ddff2) | Active state indicators (sidebar accent bar), focus rings, interactive elements |
| Surface | `--color-cosmic-card` (rgba(255,255,255,0.03)) | Glass-panel containers, table rows |
| Border | `--color-cosmic-border` (rgba(255,255,255,0.08)) | All borders |
| Text main | `--color-text-main` (#e2e8f0) | Primary text (dark mode) |
| Text muted | `--color-text-muted` (#64748b) | Secondary text, labels |
| Destructive | `--color-cosmic-red` (#ef4444) | Destructive actions only |

Accent reserved for: active state left border indicator on sidebar NavLink items, focus ring on inputs, sidebar collapse toggle hover state.

**New additions:**
- **Active nav background:** `rgba(13, 223, 242, 0.08)` — 8% cyan overlay for active nav item background
- **Active nav text:** `--color-cosmic-cyan` — cyan text on active nav item
- **Active nav accent bar:** `3px solid var(--color-cosmic-cyan)` — left border indicator

---

## Sidebar Visual Contract (D-01, D-02, D-03)

### Current → Target

| Element | Current | Target |
|---------|---------|--------|
| Logo size | `w-16 h-16` (64px) | `w-8 h-8` (32px) |
| Logo-text gap | `mr-1` (4px) | `mr-2` (8px) — tighter feel |
| Sidebar open width | `w-64` (256px) | `w-56` (224px) |
| Sidebar collapsed | `w-16` (64px), only hamburger | `w-16` with icon-only nav items visible |
| Nav element | `<a>` tags | React Router `<NavLink>` |
| Active state | None | Left accent bar + background highlight + cyan text |
| Icons | None | lucide-react icons per nav item |
| Collapse button | ☰ text | lucide-react `Menu` icon |

### Nav Item Active State Spec

```css
/* Active nav item */
background: rgba(13, 223, 242, 0.08);
border-left: 3px solid var(--color-cosmic-cyan);
color: var(--color-cosmic-cyan);

/* Hover (non-active) */
background: rgba(255, 255, 255, 0.03);
color: var(--color-text-main);

/* Default */
color: var(--color-text-secondary);
```

### Icon Mapping

| Nav Item | lucide-react Icon |
|----------|-------------------|
| Dashboard | `LayoutDashboard` |
| Servers | `Server` |
| Nodes | `Network` |
| Templates | `LayoutTemplate` |
| Mod Browser | `Puzzle` |
| Billing | `CreditCard` |
| Settings | `Settings` |

### Collapsed State Behavior

When `sidebarOpen === false` (w-16):
- Show icon-only nav items (no text labels)
- Hide "Escluse" brand text and logo
- Hamburger button remains
- On hover over collapsed sidebar, could show tooltip with item name (nice-to-have, agent's discretion)

---

## Page Layout Contract (D-04, D-06, D-07)

### Padding Standard

All pages use `p-8` (32px) as the content area padding, with the following exceptions:

| Page | Current | Target | Notes |
|------|---------|--------|-------|
| DashboardPage | `p-8` | `p-8` | Already correct |
| ServerManagerPage | `p-8` | `p-8` | Already correct |
| Nodes | varies | `p-8` | 900-line file — audit for div structure |
| BillingPage | `p-8` | `p-8` | Already correct |
| SettingsPage | `p-6` | `p-8` | Increase from 24px → 32px |
| Alerts | `px-8 py-6` (header) | Standardize to `p-8` content area |
| Console | flex-1 layout | Standardize to `p-8` content area |
| LoginPage | varies | `p-8` | Auth pages may use centered layout — keep centered but use consistent padding |
| RegisterPage | varies | `p-8` | Same as LoginPage |
| TemplateLibraryPage | varies | `p-8` | |
| TemplateCreatePage | varies | `p-8` | |
| ModBrowserPage | varies | `p-8` | |

### Page Width Strategy (D-07)

- **Dashboard + data-heavy pages** (servers list, nodes, billing tables): Full-width (`w-full` with `p-8`)
- **Other pages** (settings, alerts, templates): Max-width container at `max-w-6xl` centered within the `p-8` area, to maintain readable line lengths

### Header Hierarchy

Standard pattern:

```html
<!-- Page title -->
<h1 class="text-3xl font-semibold text-[var(--color-text-main)]">Page Title</h1>

<!-- Section heading -->
<h2 class="text-2xl font-semibold text-[var(--color-text-main)]">Section Title</h2>

<!-- Sub-section heading -->
<h3 class="text-lg font-semibold text-[var(--color-text-main)]">Sub-section Title</h3>
```

All pages must use this pattern. Pages currently using `text-2xl font-bold` for h1 must be updated.

---

## Table Pattern Contract (D-05)

### Standard Table Structure

```html
<div class="glass-panel overflow-hidden border border-[var(--color-cosmic-border)]">
  <table class="w-full">
    <thead>
      <tr>
        <th class="px-4 py-3 text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider bg-[rgba(255,255,255,0.02)]">Column</th>
      </tr>
    </thead>
    <tbody class="divide-y divide-[var(--color-cosmic-border)]">
      <tr class="hover:bg-[rgba(255,255,255,0.02)] transition-colors">
        <td class="px-4 py-3 text-sm text-[var(--color-text-main)]">Value</td>
      </tr>
    </tbody>
  </table>
</div>
```

### Table Header Spec

- Background: `bg-[rgba(255,255,255,0.02)]`
- Text: `text-xs font-medium uppercase tracking-wider text-[var(--color-text-muted)]`
- Padding: `px-4 py-3`
- Alignment: `text-left`

### Table Row Spec

- Row height: `px-4 py-3` (same as header)
- Hover: `hover:bg-[rgba(255,255,255,0.02)] transition-colors`
- Text: `text-sm text-[var(--color-text-main)]`
- Divider: `divide-y divide-[var(--color-cosmic-border)]`

### Empty State Spec

- Container: `<div class="glass-panel p-12 text-center border border-[var(--color-cosmic-border)]">`
- Heading: `<p class="text-[var(--color-text-muted)] text-lg mb-4">No items message</p>`
- CTA button: `<button class="px-6 py-3 bg-[var(--color-cosmic-cyan)] text-[var(--color-deep-space)] rounded-lg font-semibold hover:brightness-110 transition-all">`
- Messaging: "No {items} match your filters" for filtered states, "No {items} yet" for empty states

---

## TopBar Layout Contract

| Element | Current | Target |
|---------|---------|--------|
| Height | `h-14` | `h-14` (keep) |
| Title | "Escluse Dashboard" | "Escluse Dashboard" (keep) |
| Alpha badge | cyan/orange badge | Keep |
| Theme toggle | Sun/Moon lucide icon | Keep |
| Notifications | Bell lucide icon | Keep |
| User dropdown | Avatar + name + ChevronDown | Keep |

**Issue to fix:** TopBar renders "Escluse Dashboard" heading, while page content also has its own heading ("Welcome back, User!" on dashboard). This is acceptable — TopBar shows app-level context, page heading shows page-level context. But verify no double-h1 issue.

---

## Copywriting Contract

| Element | Copy |
|---------|------|
| Sidebar, collapsed, no active | Plain nav item name |
| Sidebar, active state | Same name, cyan accent |
| Table empty state (filter) | "No {items} match your filters" |
| Table empty state (no data) | "No {items} yet" |
| CTA on empty state | "Create your first {item}" |
| Welcome header | "Welcome back, {name}!" (dynamic) |

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| lucide-react | LayoutDashboard, Server, Network, LayoutTemplate, Puzzle, CreditCard, Settings, Menu, ChevronDown | not required |

---

## Implementation Order (Plans)

Based on dependencies between changes:

### Plan 1: Sidebar Refinement
**Files:** `app/src/app/App.jsx`
- Reduce logo from `w-16 h-16` to `w-8 h-8`
- Reduce sidebar from `w-64` to `w-56`
- Convert `<a>` to `<NavLink>` with active state classes
- Add lucide-react icons per nav item
- Show icon-only nav when collapsed (`w-16`)
- Replace hamburger ☰ with lucide-react `Menu` icon

### Plan 2: Layout Consistency
**Files:** All page components
- Standardize all pages to `p-8` padding
- Fix heading hierarchy (h1 = text-3xl, h2 = text-2xl)
- Apply max-width container pattern (max-w-6xl) to non-data pages

### Plan 3: Table Unification
**Files:** Pages with tables (DashboardPage, Nodes, BillingPage, Alerts)
- Audit all tables against contract
- Fix thead, row height, empty state patterns
- Ensure consistent hover states

---

## Checker Sign-Off

- [x] Dimension 1 Copywriting: PASS — empty states, CTAs, and sidebar labels defined. Error state copy follows existing pattern.
- [x] Dimension 2 Visuals: PASS — sidebar visual contract fully specified (active state CSS, icon mapping, collapsed behavior). Table pattern with exact classes. Padding standard per page.
- [x] Dimension 3 Color: PASS — all colors reference existing CSS variables. New active state colors specified with exact values. Accent usage explicitly scoped to nav active state + focus rings.
- [x] Dimension 4 Typography: PASS — h1/h2/h3/body/label/table-header all specified with exact sizes and weights. Standardization rule enforces `text-3xl font-semibold` / `text-2xl font-semibold` pattern.
- [x] Dimension 5 Spacing: PASS — spacing scale documented. Exceptions listed (logo gap `mr-2`, collapsed sidebar). Page padding standardized to `p-8` with per-page audit table.
- [x] Dimension 6 Registry Safety: PASS — only lucide-react icons used. Complete import list. No third-party registries.

**Approval:** approved 2026-06-16
