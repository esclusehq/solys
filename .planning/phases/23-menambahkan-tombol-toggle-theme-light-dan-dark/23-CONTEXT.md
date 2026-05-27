# Phase 23: Tombol toggle theme light/dark - Context

**Gathered:** 2026-04-20

<domain>
## What We're Building

Add a theme toggle button to switch between light and dark themes. Currently there's a theme system in uiStore but no toggle button exists.

**Existing:**
- `uiStore.js` has `theme: 'dark'` and `setTheme()` function
- Uses zustand persist middleware
- Currently hardcoded to 'dark'

</domain>

<decisions>
## Implementation

**Toggle Location:** Header/Navbar (top-right corner)
**Approach:** CSS Variables with light/dark values
- Use CSS custom properties for colors
- Toggle class on root element (e.g., `data-theme="light"` vs `"dark"`)
- Already have CSS variables in index.css

## Files to Modify

1. `app/src/app/App.jsx` or `app/src/components/Layout.jsx` - Add toggle button in header
2. `app/src/store/uiStore.js` - Already has setTheme (no changes needed)
3. `app/src/index.css` - Add light theme CSS variables

## Component

ThemeToggle button:
- Icon: ☀️ for light, 🌙 for dark
- Click calls useUIStore.setTheme('light'/'dark')
- Persists via zustand

</decisions>

<canonical_refs>
## References

- app/src/store/uiStore.js — existing theme state
- app/src/index.css — CSS variables
- Need to find Layout.jsx or App.jsx header

</canonical_refs>

<specifics>
## Implementation Details

1. Add light theme CSS variables to index.css:
```css
[data-theme="light"] {
  --color-bg: #ffffff;
  --color-text: #1a1a1a;
  /* etc */
}
```

2. Add toggle button in header:
```jsx
<button onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
  {theme === 'dark' ? '☀️' : '🌙'}
</button>
```

3. Apply theme to root element in App.jsx:
```jsx
useEffect(() => {
  document.documentElement.setAttribute('data-theme', theme)
}, [theme])
```

</specifics>

<deferred>
## Deferred

- Remember theme preference per-user in backend (future enhancement)
- System preference detection (prefers-color-scheme)

</deferred>

---

## ▶ Next Up

`/clear` then:

/gsd-plan-phase 23 ${GSD_WS} — create plan from this context