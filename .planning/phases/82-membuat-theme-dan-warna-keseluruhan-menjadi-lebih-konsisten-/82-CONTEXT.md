# Phase 82: Membuat theme dan warna keseluruhan menjadi lebih konsisten, dan membuat toggle light/dark berfungsi dengan benar - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Global theme consistency and functional light/dark toggle across the entire app. Add semantic CSS variables for light mode, normalize hardcoded colors across all components to use CSS variables (so light mode actually works), and improve the toggle to respect system preference. No new features, no page layout changes — purely a foundation/infrastructure phase for theming.

</domain>

<decisions>
## Implementation Decisions

### Scope — Full App Audit
- **D-01:** Fix all components with hardcoded `bg-gray-*` / `text-gray-*` / `border-gray-*` classes — 15+ components identified
- **D-02:** Also audit already-restyled pages (Phases 75-81) — they may still have hardcoded structural color classes that don't respond to theme switch
- **D-03:** Priority-based: visible components first (Sidebar, TopBar, navigation, modals, alerts, toasts), deeper components later (FileManager, MetricsCard, etc.)

### Light Theme Visual Design
- **D-04:** Glass/transparency aesthetic — white panels with subtle `rgba(0,0,0,0.03)` transparency, light shadows, glass effect inverted from dark mode
- **D-05:** No glows, no stars-bg overlay in light mode — remove or zero-opacity
- **D-06:** Keep same cosmic accent colors (cyan, purple, green, red, orange, blue) — they work on light backgrounds as-is
- **D-07:** Light theme base: `--color-deep-space: #f8fafc`, `--color-nebula: #ffffff`, `--color-cosmic-card: rgba(0, 0, 0, 0.03)`, `--color-text-main: #1e293b`, `--color-cosmic-border: rgba(0, 0, 0, 0.08)`

### Color Palette Normalization
- **D-08:** Replace common raw Tailwind colors (gray, blue, red, green, yellow, orange) with CSS variable references — no exceptions for these families
- **D-09:** Add semantic CSS variable tokens with light/dark overrides:
  - `--color-bg-primary` → dark: `var(--color-deep-space)`, light: `#f8fafc`
  - `--color-bg-secondary` → dark: `var(--color-nebula)`, light: `#ffffff`
  - `--color-surface` → dark: `var(--color-cosmic-card)`, light: `rgba(0,0,0,0.03)`
  - `--color-border` → dark: `var(--color-cosmic-border)`, light: `rgba(0,0,0,0.08)`
  - `--color-text-primary` → dark: `var(--color-text-main)`, light: `#1e293b`
  - `--color-text-secondary` → dark: `var(--color-text-muted)`, light: `#64748b`
- **D-10:** Keep existing cosmic-named vars (`--color-cosmic-*`) for backward compatibility — semantic tokens reference them internally

### Toggle Behavior
- **D-11:** Follow system `prefers-color-scheme` media query on first visit (no stored preference yet). Once user manually toggles, persist their choice in localStorage and always respect it
- **D-12:** Smooth CSS transition (300-500ms) on `background-color`, `color`, `border-color` — no page fade
- **D-13:** Manual toggle stays in TopBar (sun/moon icon button) — no UI changes to toggle placement

### Implementation Approach
- **D-14:** Split into 3 sequential plans:
  - **Plan 01:** Add semantic CSS variables with light/dark overrides + toggle improvements (system preference detection, transition CSS)
  - **Plan 02:** Audit & fix visible components (Sidebar, TopBar, navigation, modals, alerts, toasts, overlay components)
  - **Plan 03:** Fix remaining components (FileManager, MetricsCard, NotificationCenter, TunnelHealthCard, ConnectivitySection, etc.)
- **D-15:** Variable naming convention: `--color-bg-*`, `--color-text-*`, `--color-surface`, `--color-border` for semantic tokens

### the agent's Discretion
- Exact transition timing within 300-500ms range
- Specific CSS variable assignments for each component class migration
- Order of component fixes within each plan's scope

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Current Theme Infrastructure
- `app/src/index.css` — All CSS variables (`@theme` block), `[data-theme="light"]` overrides, glass-panel, stars-bg, glow utility classes
- `app/src/store/uiStore.js` — Theme state management (zustand + persist localStorage, `theme: 'dark'` default, `setTheme()`)
- `app/src/app/App.jsx` — `data-theme` attribute set on `<html>` via `useEffect`

### Toggle & Layout
- `app/src/components/TopBar.jsx` — Theme toggle button (sun/moon icon) at line 58-62
- `app/src/components/Sidebar.jsx` — Inline sidebar with hardcoded gray classes

### Components with Hardcoded Colors (full or partial list — scout for full inventory)
- `app/src/components/FileManager.jsx`
- `app/src/components/MetricsCard.jsx`
- `app/src/components/NotificationCenter.jsx`
- `app/src/components/TunnelHealthCard.jsx`
- `app/src/components/ModeOverrideDropdown.jsx`
- `app/src/components/InviteFriendsModal.jsx`
- `app/src/components/ConnectivitySection.jsx`
- `app/src/components/EmailVerificationBanner.jsx`
- `app/src/components/EmailVerificationDialog.jsx`
- `app/src/components/Onboarding.jsx`

### Reference Patterns (already cosmic-themed)
- `app/src/pages/servers/ServerManagerPage.jsx` — Phase 75 cosmic theme reference
- `app/src/pages/billing/BillingPage.jsx` — Phase 79 cosmic theme + glass-panel
- `app/src/pages/settings/SettingsPage.jsx` — Phase 80 file-split + cosmic theme
- `app/src/pages/dashboard/DashboardPage.jsx` — Phase 81 cosmic restyle

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `useUIStore` (zustand + persist) — Theme state management already in place, just needs system-preference enhancement
- 13 existing CSS variables with `[data-theme="light"]` overrides — foundation to extend
- `glass-panel` utility class — ready for light mode with updated var overrides
- TopBar toggle button — already wired to `setTheme()`, no UI changes needed

### Established Patterns
- Cosmic theme: `glass-panel`, `border-[var(--color-cosmic-border)]`, `focus:ring-[var(--color-cosmic-cyan)]`, glow hover effects — established across Phases 75-81
- localStorage storage key: `escluse-ui` for persisted preferences
- `data-theme` attribute on `<html>` for CSS selector targeting

### Integration Points
- `app/src/index.css` — `@theme` block + `[data-theme="light"]` — where new semantic vars go
- `app/src/app/App.jsx` — `useEffect` that sets `data-theme` — where system-preference logic goes
- `app/src/store/uiStore.js` — Where theme state + system preference logic lives
- All components with hardcoded colors need class migration to CSS variable references

</code_context>

<specifics>
## Specific Ideas

- Components recently cosmic-restyled (Phases 75-81) likely used CSS vars for cosmic colors but may still have hardcoded structural classes like `bg-gray-900`, `text-gray-400`, `border-gray-700` — grep for these
- The toggle should feel seamless: no flash of wrong theme on page load (use inline script or `<script>` before React mounts to read localStorage/system preference and set `data-theme` early)
- Light mode glass panels should use white instead of the dark `rgba(255,255,255,0.03)` — so `--color-cosmic-card` light value should be `rgba(0,0,0,0.03)` or solid white with subtle shadow

</specifics>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 82-membuat-theme-dan-warna-keseluruhan-menjadi-lebih-konsisten-*
*Context gathered: 2026-06-15*
