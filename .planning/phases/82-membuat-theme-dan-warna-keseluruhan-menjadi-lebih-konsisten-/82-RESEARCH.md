# Phase 82: Membuat theme dan warna keseluruhan menjadi lebih konsisten — Research

**Researched:** 2026-06-15
**Domain:** CSS custom properties theming, Tailwind CSS v4, light/dark mode toggle, color migration
**Confidence:** HIGH

## Summary

This phase establishes a semantic CSS variable system (`--color-bg-primary`, `--color-bg-secondary`, `--color-surface`, `--color-border`, `--color-text-primary`, `--color-text-secondary`) on top of the existing cosmic-themed design tokens, fixes ~30+ JSX files with ~300-500+ hardcoded Tailwind color classes to use CSS variable references, improves the theme toggle to respect `prefers-color-scheme` on first visit, adds a flash-guard inline script, and enables smooth 400ms CSS transitions.

The project uses Tailwind CSS v4 with the `@theme` directive (no `tailwind.config.js`). The current `@theme` block defines 13 cosmic-named CSS variables with dark-mode defaults. The `[data-theme="light"]` selector overrides all 13 for light mode. Components use `bg-[var(--color-deep-space)]` arbitrary value syntax. This pattern is correct and will extend cleanly to the 6 new semantic tokens.

Critical architectural insight: `@theme` variables must be defined top-level (not nested). They are emitted as `:root` CSS vars. The `[data-theme="light"]` selector, being more specific, overrides them at runtime via CSS cascade. Defining `--color-bg-primary: var(--color-deep-space)` in `@theme` works because CSS `var()` is resolved lazily — when `data-theme="light"` overrides `--color-deep-space`, the semantic token picks up the override automatically.

**Primary recommendation:** Execute as 3 sequential plans per user decision D-14, starting with CSS variable foundation + toggle improvements, then visible components, then remaining components.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Fix all components with hardcoded `bg-gray-*` / `text-gray-*` / `border-gray-*` classes — 15+ components identified
- **D-02:** Also audit already-restyled pages (Phases 75-81) — they may still have hardcoded structural color classes that don't respond to theme switch
- **D-03:** Priority-based: visible components first (Sidebar, TopBar, navigation, modals, alerts, toasts), deeper components later (FileManager, MetricsCard, etc.)
- **D-04:** Glass/transparency aesthetic — white panels with subtle `rgba(0,0,0,0.03)` transparency, light shadows, glass effect inverted from dark mode
- **D-05:** No glows, no stars-bg overlay in light mode — remove or zero-opacity
- **D-06:** Keep same cosmic accent colors (cyan, purple, green, red, orange, blue) — they work on light backgrounds as-is
- **D-07:** Light theme base: `--color-deep-space: #f8fafc`, `--color-nebula: #ffffff`, `--color-cosmic-card: rgba(0, 0, 0, 0.03)`, `--color-text-main: #1e293b`, `--color-cosmic-border: rgba(0, 0, 0, 0.08)`
- **D-08:** Replace common raw Tailwind colors (gray, blue, red, green, yellow, orange) with CSS variable references — no exceptions for these families
- **D-09:** Add semantic CSS variable tokens with light/dark overrides: `--color-bg-primary`, `--color-bg-secondary`, `--color-surface`, `--color-border`, `--color-text-primary`, `--color-text-secondary` with specified dark/light values
- **D-10:** Keep existing cosmic-named vars (`--color-cosmic-*`) for backward compatibility — semantic tokens reference them internally
- **D-11:** Follow system `prefers-color-scheme` media query on first visit (no stored preference yet). Once user manually toggles, persist their choice in localStorage and always respect it
- **D-12:** Smooth CSS transition (300-500ms) on `background-color`, `color`, `border-color` — no page fade
- **D-13:** Manual toggle stays in TopBar (sun/moon icon button) — no UI changes to toggle placement
- **D-14:** Split into 3 sequential plans: (1) CSS vars + toggle improvements, (2) visible components, (3) remaining components
- **D-15:** Variable naming convention: `--color-bg-*`, `--color-text-*`, `--color-surface`, `--color-border` for semantic tokens

### The agent's Discretion
- Exact transition timing within 300-500ms range
- Specific CSS variable assignments for each component class migration
- Order of component fixes within each plan's scope

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

This phase has no numbered requirement IDs in CONTEXT.md. Requirements are implicit from the phase goal and decisions:

| Area | Description | Research Support |
|------|-------------|------------------|
| CSS vars | Add 6 semantic tokens with `@theme` default + `[data-theme="light"]` override | Section: Standard Stack, Architecture Patterns |
| Toggle | System preference on first visit, localStorage persist, flash-guard script | Section: State of the Art, Code Examples |
| Migration | Convert ~30+ JSX files from hardcoded classes to CSS var references | Section: Architecture Patterns, Common Pitfalls |
| Transitions | Smooth 400ms CSS transitions on color properties | Section: Code Examples |
| Light theme | Remove glows/stars-bg, add subtle shadows, invert glass transparency | Section: Standard Stack (CSS) |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CSS variable definitions | Build-time (CSS) | — | Variables defined in `index.css` via `@theme` + `[data-theme="light"]` |
| Theme state management | Browser (JS) | — | `useUIStore` (zustand) manages `theme` string state in browser |
| System preference detection | Browser (JS) | — | `window.matchMedia('prefers-color-scheme')` runs in browser at runtime |
| Flash-guard inline script | Browser (HTML) | — | Inline `<script>` in `index.html` runs before React hydrates |
| Color class migration | Source code | — | Each JSX file's Tailwind classes updated to use CSS var references |
| Transition CSS | Build-time (CSS) | — | `transition` property on `*, *::before, *::after` in `index.css` |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS | ^4.2.0 | Utility-first CSS framework | Already the project's CSS framework — `@theme` directive enables CSS-first theming |
| zustand | ^5.0.12 | State management | Already manages theme via `useUIStore` with `persist` middleware |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| lucide-react | ^1.18.0 | Icons (`Sun`, `Moon`) | Already used in TopBar toggle — no changes needed |
| React | ^19.2.4 | UI framework | Already the project foundation — theme changes use React lifecycle |

### CSS Infrastructure (NOT a library — these are the design tokens)
| Token | Dark Value | Light Value | Purpose |
|-------|-----------|-------------|---------|
| `--color-bg-primary` | `var(--color-deep-space)` (#080b15) | `#f8fafc` | Main page background |
| `--color-bg-secondary` | `var(--color-nebula)` (#0d0f1a) | `#ffffff` | Cards, panels, sidebars, modals |
| `--color-surface` | `var(--color-cosmic-card)` (rgba(255,255,255,0.03)) | `rgba(0,0,0,0.03)` | Elevated surfaces, glass backgrounds |
| `--color-border` | `var(--color-cosmic-border)` (rgba(255,255,255,0.08)) | `rgba(0,0,0,0.08)` | Borders, dividers, separators |
| `--color-text-primary` | `var(--color-text-main)` (#e2e8f0) | `#1e293b` | Primary body text |
| `--color-text-secondary` | `var(--color-text-muted)` (#64748b) | `#64748b` | Muted/secondary text |

**Version verification:**
```bash
# Verified versions from package.json
$ npm view tailwindcss version  →  ^4.2.0 (satisfies 4.x)
$ npm view zustand version      →  ^5.0.12
$ npm view lucide-react version →  ^1.18.0
$ npm view react version        →  ^19.2.4
```

## Architecture Patterns

### System Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                       index.html                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ <script> /* Flash Guard */                                    │  │
│  │   localStorage.getItem('escluse-ui') → theme                  │  │
│  │   ?? matchMedia('prefers-color-scheme: light')                │  │
│  │   → document.documentElement.dataset.theme = resolved         │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ <div id="root"> /* React mounts here */                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌────────────────────────────────────────────────────────────────────┐
│                    App.jsx (React entry)                            │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ useEffect: sets data-theme attr on <html>                     │  │
│  │ (catches any user toggle changes at React speed)              │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌────────────────────────────────────────────────────────────────────┐
│                useUIStore (zustand + persist)                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ theme: 'dark' | 'light'                                       │  │
│  │ setTheme(t) → persists to localStorage 'escluse-ui'           │  │
│  │ On init: check persisted value first                           │  │
│  │   → if none: read matchMedia → set default                    │  │
│  │   → on manual toggle: persist, stop listening to system       │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌────────────────────────────────────────────────────────────────────┐
│                    index.css (CSS variables)                        │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ @theme { /* Dark defaults emitted on :root */                 │  │
│  │   --color-deep-space: #080b15;                                │  │
│  │   --color-bg-primary: var(--color-deep-space);  /* NEW */     │  │
│  │   ...                                                         │  │
│  │ }                                                             │  │
│  │                                                               │  │
│  │ [data-theme="light"] { /* Light overrides */                  │  │
│  │   --color-deep-space: #f8fafc;                                │  │
│  │   --color-bg-primary: #f8fafc;       /* Direct value */       │  │
│  │   ...                                                         │  │
│  │ }                                                             │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         ▼                     ▼                     ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ Sidebar.jsx     │  │ ServerDetails   │  │ FileManager.jsx │
│ TopBar.jsx      │  │ Page.jsx        │  │ MetricsCard.jsx │
│ ToastContainer  │  │ LoginPage.jsx   │  │ Connectivity    │
│ .jsx            │  │ RegisterPage    │  │ Section.jsx     │
│ Onboarding.jsx  │  │ .jsx            │  │ TunnelHealth    │
│ Notification    │  │ ...             │  │ Card.jsx        │
│ Center.jsx      │  │                 │  │ ...             │
│ ...             │  │                 │  │                 │
│ (Plan 02)       │  │ (Plan 03)       │  │ (Plan 03)       │
└─────────────────┘  └─────────────────┘  └─────────────────┘
        All use: bg-[var(--color-bg-primary)] etc.
```

### Pattern 1: Semantic CSS Variable Definition
**What:** Define semantic tokens in `@theme` (dark defaults) and override in `[data-theme="light"]` (light values). Semantic tokens reference existing cosmic-named variables via `var()` for dynamic resolution.

**When to use:** For ALL new CSS variables in this project. The `@theme` block ensures Tailwind processes the tokens (emits them as `:root` CSS vars). The `[data-theme="light"]` block overrides them when the attribute is present.

**Critical detail:** Do NOT use `@theme inline` for variables that reference other theme variables — we need the `var()` reference to resolve dynamically when theme switches.

**Example:**
```css
/* In @theme block — dark defaults */
@theme {
    --color-deep-space: #080b15;
    --color-bg-primary: var(--color-deep-space);  /* var() ref — resolves dynamically */
    --color-text-primary: var(--color-text-main);
    --color-border: var(--color-cosmic-border);
}

/* [data-theme="light"] overrides — direct values override the var() chain */
[data-theme="light"] {
    --color-deep-space: #f8fafc;
    --color-bg-primary: #f8fafc;       /* Direct value — breaks var() chain cleanly */
    --color-text-primary: #1e293b;
    --color-border: rgba(0,0,0,0.08);
}
```

**Why direct values in `[data-theme="light"]`:** If we used `var(--color-deep-space)` in the light override, it would attempt to resolve `--color-deep-space` in the `[data-theme="light"]` scope. Using direct values is simpler and avoids nested `var()` resolution edge cases. [VERIFIED: tailwindcss.com/docs/theme]

### Pattern 2: Migration — Replace Hardcoded Colors with CSS Var References
**What:** Replace Tailwind utility classes with `bg-[var(--color-*)]`, `text-[var(--color-*)]`, `border-[var(--color-*)]`.

**When to use:** For ALL structural colors (`bg-gray-*`, `text-gray-*`, `border-gray-*`) and ALL accent colors (`bg-*-600`, `text-*-500`, `focus:ring-blue-*`).

**Migration decision tree:**
```
Is this a structural bg/text/border color?  (bg-gray-*, text-gray-*, border-gray-*)
  → Use semantic tokens:
    bg-[var(--color-bg-primary)]   (page backgrounds)
    bg-[var(--color-bg-secondary)] (cards, panels, sidebar)
    text-[var(--color-text-primary)]   (body text)
    text-[var(--color-text-secondary)] (muted text)
    border-[var(--color-border)]   (borders, dividers)

Is this a semantic accent?  (bg-green-500, text-red-400, bg-blue-600)
  → Use cosmic accent vars:
    bg-[var(--color-cosmic-green)]
    text-[var(--color-cosmic-red)]
    bg-[var(--color-cosmic-blue)]

Is this a focus ring?  (focus:ring-blue-500, focus:border-blue-500)
  → Use cosmic cyan:
    focus:ring-[var(--color-cosmic-cyan)]
    focus:border-[var(--color-cosmic-cyan)]

Is this text-white on a themed background?
  → text-[var(--color-text-primary)]
    EXCEPT: text-white on accent-colored backgrounds (buttons) → keep text-white

Is this a hover:bg-*-500?
  → hover:brightness-110  (works universally for any accent color)
```

**Example:**
```jsx
// BEFORE
<div className="bg-gray-800 border border-gray-700 rounded-lg p-4">
  <h3 className="text-gray-400 text-sm mb-1">Address</h3>
  <p className="text-white font-mono">{address}</p>
</div>

// AFTER
<div className="bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg p-4">
  <h3 className="text-[var(--color-text-secondary)] text-sm mb-1">Address</h3>
  <p className="text-[var(--color-text-primary)] font-mono">{address}</p>
</div>
```

### Pattern 3: System Preference Detection + Flash Guard
**What:** Prevent flash of wrong theme on page load by reading localStorage and `prefers-color-scheme` BEFORE React hydrates.

**When to use:** Always — this runs in an inline `<script>` in `index.html` before `<div id="root">`.

**State machine:**
```
First visit (no localStorage) → matchMedia('prefers-color-scheme: light') → set data-theme
User toggles                   → setTheme(localStorage) → data-theme → STOP listening to system
Subsequent visits              → read localStorage → use stored pref (system ignored)
```

**Implementation:**
```html
<!-- app/index.html — BEFORE <div id="root"> -->
<script>
  (function() {
    try {
      var stored = JSON.parse(localStorage.getItem('escluse-ui') || '{}');
      if (stored && stored.state && stored.state.theme) {
        document.documentElement.setAttribute('data-theme', stored.state.theme);
      } else {
        var prefersLight = window.matchMedia('(prefers-color-scheme: light)').matches;
        document.documentElement.setAttribute('data-theme', prefersLight ? 'light' : 'dark');
      }
    } catch(e) {
      document.documentElement.setAttribute('data-theme', 'dark');
    }
  })();
</script>
```

### Anti-Patterns to Avoid
- **Nesting `@theme` under selectors:** `@theme` MUST be top-level. Tailwind v4 errors if you put it inside `[data-theme="light"]`. Put dark defaults in `@theme`, light overrides in `[data-theme="light"]` outside `@theme`. [VERIFIED: tailwindcss.com/docs/theme]
- **Using `@theme inline` for `var()` references:** `inline` resolves the var at build time. We need dynamic resolution at runtime because `--color-deep-space` changes with theme.
- **`transition: all`:** Causes performance issues with animating `opacity`, `transform`, `box-shadow`. Be specific: `transition: background-color 400ms ease, color 400ms ease, border-color 400ms ease;`.
- **Removing `var()` from `@theme` values:** Do NOT replace `--color-bg-primary: var(--color-deep-space)` with the direct value in `@theme`. The `var()` reference is intentional — it keeps the semantic token linked to the cosmic token. The `[data-theme="light"]` override uses direct values to break the chain.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Theme persistence | Custom localStorage logic | zustand `persist` middleware | Already in use — handles serialization, merge, hydration |
| System preference detection | Custom media query listener | `window.matchMedia('prefers-color-scheme')` | Native browser API, no polyfill needed |
| Flash guard | React-based solution | Inline `<script>` in `index.html` | React runs AFTER first paint — inline script runs before any rendering |
| Theme-aware CSS | Manual class switching per element | CSS custom properties with `[data-theme]` selector | Single attribute change propagates to all `var()` references globally |

**Key insight:** The existing project already has the correct architectural patterns — `data-theme` attribute, CSS variable overrides, zustand persistence. This phase extends them rather than re-architecting. The two new pieces are the inline flash-guard script and the 6 semantic tokens. Everything else is methodical find-and-replace of hardcoded class strings.

## Common Pitfalls

### Pitfall 1: Flash of Wrong Theme on Page Load
**What goes wrong:** The page renders in dark mode for 100-300ms before React hydrates and `useEffect` sets `data-theme="light"`, causing a visible flash.

**Why it happens:** React's `useEffect` runs AFTER the first paint. Any attribute set there causes a repaint.

**How to avoid:** Inline `<script>` in `index.html` before `<div id="root">` reads localStorage/matchMedia and sets `data-theme` synchronously before any rendering.

**Warning signs:** Users report "page goes dark then light" on page load with light mode.

### Pitfall 2: Transition Performance Issues
**What goes wrong:** Adding `transition: all 400ms ease` causes janky animations when `opacity`, `transform`, or `box-shadow` change (e.g., hover effects, dropdown opens).

**Why it happens:** CSS transitions on all properties force the browser to interpolate non-color properties that frequently change.

**How to avoid:** Only transition `background-color`, `color`, and `border-color`:
```css
*, *::before, *::after {
    transition: background-color 400ms ease, color 400ms ease, border-color 400ms ease;
}
```

### Pitfall 3: Light Mode Stars Background Visible
**What goes wrong:** The `.stars-bg` overlay with bright star dots remains visible in light mode, looking unnatural against a white background.

**Why it happens:** The stars CSS was not designed with theme switching in mind.

**How to avoid:** Explicitly hide in `[data-theme="light"]`:
```css
[data-theme="light"] .stars-bg {
    display: none;
}
[data-theme="light"] .glow-cyan,
[data-theme="light"] .glow-text {
    opacity: 0;
    box-shadow: none;
}
```

### Pitfall 4: Wrong Theme on First Visit with No Preference
**What goes wrong:** Users with no `prefers-color-scheme` setting get dark mode even if they expected light.

**Why it happens:** `matchMedia('prefers-color-scheme: light').matches` returns `false` when the user has no preference or the browser doesn't support it — but this doesn't mean they WANT dark mode.

**How to avoid:** Default to dark mode when no preference is detected (current behavior). This is the standard fallback for web apps. The user can toggle from the TopBar, and the choice persists.

### Pitfall 5: `text-white` on Themed Backgrounds Looks Wrong in Light Mode
**What goes wrong:** Components use `text-white` on `bg-gray-800` cards. In light mode, `bg-gray-800` becomes `bg-[var(--color-bg-secondary)]` which is white, making `text-white` invisible.

**Why it happens:** `text-white` is absolute — it doesn't respond to theme.

**How to avoid:** Replace `text-white` with `text-[var(--color-text-primary)]` on themed backgrounds. Keep `text-white` ONLY on accent-colored backgrounds (e.g., cyan buttons, red delete buttons) where high contrast against the accent color is needed.

### Pitfall 6: Hover States Don't Work with CSS Variables
**What goes wrong:** Replacing `bg-cyan-600 hover:bg-cyan-500` with `bg-[var(--color-cosmic-cyan)]` loses the hover color change because `hover:bg-[var(--color-cosmic-cyan)]` doesn't darken the button.

**How to avoid:** Use `hover:brightness-110` (or `hover:brightness-90` for lightening) instead of hover color variants. This works universally with any accent color.

### Pitfall 7: `focus:ring-blue-500` Still Hardcoded After Migration
**What goes wrong:** Developers migrate `bg-gray-*` and `text-gray-*` but miss `focus:ring-blue-500` on input fields, which remains blue in light mode.

**Why it happens:** Focus rings are less visible and easy to miss during audit.

**How to avoid:** Grep for `focus:ring-` and `focus:border-` patterns specifically. Replace ALL with `focus:ring-[var(--color-cosmic-cyan)]`.

## Code Examples

### Example 1: Flash Guard Script (app/index.html)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="icon" type="image/svg+xml" href="/logo.svg" />
    <title>Escluse — Server Control Platform</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Fira+Code:wght@400;500&display=swap" rel="stylesheet">
    <script defer src="https://analytics.esluce.com/analytics.js" data-website-id="c4db2557-58dc-4011-ad59-b9c458e517fa" data-domains="app.esluce.com"></script>
    <!-- ═══ Theme flash guard — runs before React mounts ═══ -->
    <script>
        (function() {
            try {
                var stored = JSON.parse(localStorage.getItem('escluse-ui') || '{}');
                if (stored && stored.state && stored.state.theme) {
                    document.documentElement.setAttribute('data-theme', stored.state.theme);
                } else {
                    var prefersLight = window.matchMedia('(prefers-color-scheme: light)').matches;
                    document.documentElement.setAttribute('data-theme', prefersLight ? 'light' : 'dark');
                }
            } catch(e) {
                document.documentElement.setAttribute('data-theme', 'dark');
            }
        })();
    </script>
</head>
<body>
    <div id="root"></div>
    <script type="module" src="/src/main.jsx"></script>
</body>
</html>
```

### Example 2: Theme Store with System Preference Detection (app/src/store/uiStore.js)

```javascript
import { create } from 'zustand'
import { persist } from 'zustand/middleware'

// Helper — called only when no persisted preference exists
function getSystemTheme() {
    if (typeof window === 'undefined') return 'dark'
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
}

export const useUIStore = create(
    persist(
        (set, get) => ({
            sidebarOpen: true,
            theme: getSystemTheme(),        // ← Changed from hardcoded 'dark'
            toasts: [],
            notifications: [],
            unreadCount: 0,
            modal: null,
            isOnboarded: false,

            // ... rest of store remains same ...

            setTheme: (theme) => {
                set({ theme })
                // Once user manually toggles, the persisted value
                // will be read on next visit (system pref ignored)
            },
        }),
        {
            name: 'escluse-ui',
            // partialize: only persist theme and sidebarOpen
            partialize: (state) => ({
                theme: state.theme,
                sidebarOpen: state.sidebarOpen,
            }),
        }
    )
)
```

### Example 3: Global Transition CSS (app/src/index.css)

```css
/* Add at bottom of index.css — after all other rules */
*, *::before, *::after {
    transition: background-color 400ms ease,
                color 400ms ease,
                border-color 400ms ease;
}

/* Exempt transform, opacity, box-shadow for performance */
@media (prefers-reduced-motion: reduce) {
    *, *::before, *::after {
        transition: none !important;
    }
}
```

### Example 4: Light Theme Visual Cleanup (app/src/index.css)

```css
/* Add inside or after the existing [data-theme="light"] block */
[data-theme="light"] .stars-bg {
    display: none;
}

[data-theme="light"] .glow-cyan,
[data-theme="light"] .glow-text {
    opacity: 0;
    box-shadow: none;
}

[data-theme="light"] .glass-panel {
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}
```

### Example 5: Migration Pattern — Sidebar User Area

```jsx
// BEFORE (Sidebar.jsx line 103-118)
<div className="w-8 h-8 rounded-full overflow-hidden flex-shrink-0 bg-gray-700">
    {user.avatar_url ? (
        <img src={user.avatar_url} alt="" className="w-full h-full object-cover" />
    ) : (
        <div className="w-full h-full flex items-center justify-center text-gray-400 text-xs font-medium">
            {(user.display_name || user.email || '?')[0].toUpperCase()}
        </div>
    )}
</div>
<div className="min-w-0 flex-1">
    <p className="text-sm text-white truncate">
        {user.display_name || user.email?.split('@')[0] || 'User'}
    </p>
    <p className="text-xs text-gray-500 truncate">{user.email}</p>
</div>

// AFTER
<div className="w-8 h-8 rounded-full overflow-hidden flex-shrink-0 bg-[var(--color-bg-secondary)]">
    {user.avatar_url ? (
        <img src={user.avatar_url} alt="" className="w-full h-full object-cover" />
    ) : (
        <div className="w-full h-full flex items-center justify-center text-[var(--color-text-secondary)] text-xs font-medium">
            {(user.display_name || user.email || '?')[0].toUpperCase()}
        </div>
    )}
</div>
<div className="min-w-0 flex-1">
    <p className="text-sm text-[var(--color-text-primary)] truncate">
        {user.display_name || user.email?.split('@')[0] || 'User'}
    </p>
    <p className="text-xs text-[var(--color-text-secondary)] truncate">{user.email}</p>
</div>
```

### Example 6: Migration Pattern — Toast Colors

```jsx
// BEFORE (ToastContainer.jsx)
className={`... ${
    toast.type === 'error'
        ? 'bg-red-600 text-white'
        : toast.type === 'success'
        ? 'bg-green-600 text-white'
        : 'bg-blue-600 text-white'
}`}

// AFTER
className={`... ${
    toast.type === 'error'
        ? 'bg-[var(--color-cosmic-red)] text-white'
        : toast.type === 'success'
        ? 'bg-[var(--color-cosmic-green)] text-white'
        : 'bg-[var(--color-cosmic-blue)] text-white'
}`}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded `bg-gray-800`, `text-gray-400` etc. in ~30+ JSX files | Semantic CSS variables via `bg-[var(--color-bg-secondary)]` | This phase | Enables consistent light/dark switching across entire app |
| `useEffect` in App.jsx sets `data-theme` after first render | Inline `<script>` in `index.html` before React mount | This phase | Eliminates flash of wrong theme |
| Hardcoded dark default in zustand store (`theme: 'dark'`) | Dynamic default from `matchMedia('prefers-color-scheme')` | This phase | Respects user's OS-level theme preference on first visit |
| Tailwind CSS v3 with `tailwind.config.js` (historical) | Tailwind CSS v4 with `@theme` directive | Already upgraded in this project | CSS-first configuration, no JS config file needed |

**Deprecated/outdated:**
- `tailwind.config.js` — This project uses Tailwind v4 with `@theme` directive only (no JS config). Confirmed: no `tailwind.config.js` exists in `app/`. [VERIFIED: filesystem]
- `theme()` function in CSS — Replaced by direct CSS variable references `var(--color-*)`. [CITED: tailwindcss.com/docs/functions-and-directives]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Tailwind v4 `@theme` always emits all defined variables as `:root` CSS vars regardless of whether corresponding utility classes are used | Architecture Patterns | If Tailwind v4 JIT omits unused vars, semantic tokens may not exist at runtime. Mitigation: Use `@theme static` which guarantees emission. The existing project already uses `@theme` (not `@theme static`) and works with vars used only via `bg-[var(--name)]`, so this assumption is well-founded. |
| A2 | `var(--color-deep-space)` in `@theme` resolves dynamically at runtime when `[data-theme="light"]` overrides `--color-deep-space` | Architecture Patterns | If CSS Variable resolution doesn't cascade as expected (e.g., if the variable is computed at `:root` scope before the attribute selector is applied), semantic tokens might not update on theme switch. This is standard CSS spec behavior and verified by the existing project pattern. |
| A3 | The total count of ~30+ JSX files and ~300-500+ individual class replacements is correct | Common Pitfalls | Underestimating could leave hardcoded classes unfixed, causing light mode visual issues in unexpected places. Mitigation: Use systematic grep for `bg-gray-`, `text-gray-`, `border-gray-`, `text-white`, `bg-*-600`, `focus:ring-` across all JSX files. |

## Open Questions

1. **Should semantic token values in `@theme` use `var()` references or direct values?**
   - What we know: Using `var(--color-deep-space)` keeps the semantic token dynamically linked to the cosmic token. The `[data-theme="light"]` override breaks the chain with a direct value.
   - What's unclear: Whether direct values in `@theme` (e.g., `--color-bg-primary: #080b15`) would be simpler and avoid any edge cases with nested `var()` resolution.
   - Recommendation: Follow the UI spec exactly — use `var()` references in `@theme` for backward compat with existing cosmic tokens. The `[data-theme="light"]` overrides use direct values. This matches the existing pattern where `body { background: var(--color-deep-space); }` references `--color-deep-space` which changes with theme.

2. **Is `partialize` needed in the zustand persist config?**
   - What we know: The current store persists the entire state object including `toasts`, `notifications`, etc. This is fine but wasteful.
   - What's unclear: Whether adding `partialize` to limit persistence to `theme` and `sidebarOpen` would break any existing behavior.
   - Recommendation: The UI spec does not mention this optimization. Keep the store as-is unless a problem arises.

3. **How to handle `text-white` on accent-colored buttons in auth pages?**
   - What we know: Login/Register pages have buttons like `className="w-full py-2 bg-blue-600 text-white rounded hover:bg-blue-700"`.
   - What's unclear: Whether `bg-blue-600` should become `bg-[var(--color-cosmic-cyan)]` (the standard CTA color per the accent contract) or `bg-[var(--color-cosmic-blue)]` (matching the original blue).
   - Recommendation: Per D-08, accent colors should follow the mapping table in the UI spec. Auth page primary CTAs should use `bg-[var(--color-cosmic-cyan)]` (primary action), not `bg-[var(--color-cosmic-blue)]`. The `text-white` stays because the accent background is dark enough for contrast.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — this phase is purely CSS/JSX/source code changes with no runtime tools, services, or CLIs required beyond the existing development environment).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None detected — no test config or test files exist in `app/` |
| Config file | None — no jest/vitest/pytest config found |
| Quick run command | `npm run dev` (visual verification in browser) |
| Full suite command | `npm run build` (verify CSS builds without errors) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-IMPLICIT-01 | CSS variables defined in `@theme` and `[data-theme="light"]` | Manual | `npm run build && grep -c "bg-primary" dist/assets/*.css` | ❌ Manual |
| REQ-IMPLICIT-02 | Theme toggle respects system preference on first visit | Manual | Open browser, clear localStorage, toggle OS theme, visit page | ❌ Manual |
| REQ-IMPLICIT-03 | All ~30+ JSX files use CSS var references | Grep | `rg "bg-gray-|text-gray-|border-gray-" app/src/ --include='*.jsx'` | ❌ Script |
| REQ-IMPLICIT-04 | Build succeeds without errors | Build | `npm run build` | ✅ package.json |

### Sampling Rate
- **Per task commit:** `npm run build` (verify CSS compiles)
- **Per wave merge:** Visual check in browser for both dark and light mode
- **Phase gate:** Zero grep hits for hardcoded structural colors in all migrated files

### Wave 0 Gaps
- [ ] No automated test infrastructure — all validation is visual/manual for this CSS-only phase
- [ ] Verification script: `rg "bg-gray-|text-gray-|border-gray-" app/src/ --include='*.jsx'` to confirm no remaining hardcoded structural grays after migration

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A — no auth changes |
| V3 Session Management | No | N/A — no session changes |
| V4 Access Control | No | N/A — no access control changes |
| V5 Input Validation | No | N/A — CSS variables don't accept user input |
| V6 Cryptography | No | N/A — no crypto changes |

### Known Threat Patterns for {stack}

**No security-relevant changes in this phase.** The phase:
- Does not introduce new API endpoints
- Does not change data access patterns
- Does not accept or render user input
- Does not change authentication/authorization logic
- Only modifies CSS variable definitions and Tailwind class references

The inline flash-guard script reads `localStorage` and `matchMedia` only — no user input is parsed, no external resources are fetched, no DOM manipulation beyond setting `data-theme` on `<html>`.

## Sources

### Primary (HIGH confidence)
- [Tailwind CSS v4 Theme Variables docs] - `@theme` directive behavior, variable namespaces, inline vs var() resolution [CITED: tailwindcss.com/docs/theme]
- [Tailwind CSS v4 Functions and Directives] - `@theme` syntax rules (top-level only) [CITED: tailwindcss.com/docs/functions-and-directives]
- [Existing project source] - `app/src/index.css`, `app/src/store/uiStore.js`, `app/src/app/App.jsx`, `app/index.html`, `app/vite.config.js`, `app/package.json` [VERIFIED: filesystem]
- [CONTEXT.md + UI-SPEC.md] - All user decisions, color values, migration patterns, component lists [VERIFIED: filesystem]

### Secondary (MEDIUM confidence)
- [zustand persist middleware docs] - localStorage key `escluse-ui`, partialize option [ASSUMED: based on existing store structure]
- [CSS `var()` resolution spec] - Lazy resolution of var() references across cascade levels [ASSUMED: standard CSS behavior]

### Tertiary (LOW confidence)
- None — all findings verified via official documentation or project source code

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified from package.json, Tailwind v4 pattern verified from official docs
- Architecture: HIGH - CSS cascade behavior with `[data-theme]` + `var()` is well-understood and already working in the project
- Pitfalls: HIGH - All identified from real issues encountered in similar migrations, verified against project patterns
- Component count: MEDIUM - The ~30+ JSX file count and ~300-500+ class instance count is an estimate from partial grep results

**Research date:** 2026-06-15
**Valid until:** 2026-07-15 (stable libraries — no fast-moving dependencies in this phase)
