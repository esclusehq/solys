---
phase: quick-fix
plan: 01
subsystem: auth, ui
tags: react, react-router-dom, accessibility, import-ordering

requires: []
provides:
  - SPA-friendly auth footer navigation with Link components
  - Clean import ordering in App.tsx
  - Accessible alt text for game card images
affects: []

tech-stack:
  added: []
  patterns:
    - "Internal navigation in auth components uses react-router-dom Link instead of anchor tags"
    - "All imports grouped at top of file, not inline between components"
    - "Alt text uses game name for meaningful accessibility"

key-files:
  created: []
  modified:
    - landing-page-escluse/src/components/auth/Footer.tsx
    - landing-page-escluse/src/App.tsx

key-decisions:
  - "Auth footer links use react-router-dom Link component for SPA navigation without page reloads"
  - "useAuthStore import moved to top of App.tsx with all other imports for clean ordering"
  - "SupportedGames alt text uses game.name (descriptive) instead of game.desc (category label)"

patterns-established:
  - "Internal SPA links use Link from react-router-dom, not <a> tags"
  - "Imports are grouped at the top of the file, never inline"

requirements-completed: []

duration: 3 min
completed: 2026-06-21
---

# Quick Fix 260621-mkp: Code Review Findings Fix Summary

**Replaced `<a>` with `<Link>` in auth footer, moved `useAuthStore` import to top of App.tsx, and fixed SupportedGames alt text from `game.desc` to `game.name`**

## Performance

- **Duration:** 3 min
- **Started:** 2026-06-21T13:45:00Z
- **Completed:** 2026-06-21T13:48:00Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments

- **WR-05 (Auth Footer Links):** Replaced 4 `<a href>` tags with `<Link to>` from react-router-dom in `auth/Footer.tsx`, eliminating full page reloads on internal navigation
- **IN-01 (Import Order):** Moved `useAuthStore` import from inline at line 648 to the top import section of `App.tsx`, consistent with file conventions
- **IN-02 (Alt Text):** Changed `alt={game.desc}` to `alt={game.name}` in the SupportedGames component for meaningful screen reader descriptions

## Task Commits

Each task was committed atomically (in nested `landing-page-escluse` repo):

1. **Task 1: Replace `<a>` with `<Link>` in auth/Footer.tsx** - `5d9e0cd` (fix)
2. **Task 2: Move `useAuthStore` import to top of App.tsx** - `896974b` (fix)
3. **Task 3: Fix alt text in SupportedGames** - `b97923a` (fix)

**Plan metadata:** This file (no separate metadata commit — quick task scope)

## Files Created/Modified

- `landing-page-escluse/src/components/auth/Footer.tsx` - Added `Link` import, replaced 4 `<a>` tags with `<Link>`, `href` with `to`
- `landing-page-escluse/src/App.tsx` - Moved `useAuthStore` import to top; changed SupportedGames `alt={game.desc}` to `alt={game.name}`

## Decisions Made

None - followed plan as specified

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## Next Phase Readiness

All 3 code review findings resolved. Ready for build verification or further code review.

---

*Phase: quick-fix*
*Completed: 2026-06-21*
