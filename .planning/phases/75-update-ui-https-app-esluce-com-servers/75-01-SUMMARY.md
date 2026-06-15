---
phase: 75-update-ui-https-app-esluce-com-servers
plan: 01
subsystem: ui
tags: react, tailwind, cosmic-theme, lucide-react, localStorage

requires: []
provides:
  - View toggle (card/table) with segmented button group
  - Sort by Name/Status/Activity with client-side sort utility
  - Game type filter (Minecraft Java/Bedrock/PocketMine-MP/Nukkit)
  - localStorage persistence for viewMode, sortMode, gameFilter
  - Table view with 7-column layout matching legacy ServerManager.jsx
  - Cosmic theme restyle of all existing elements
affects: [75-update-ui-https-app-esluce-com-servers]

tech-stack:
  added: []
  patterns:
    - localStorage lazy init for user preference persistence
    - Client-side sort utility with status order mapping
    - Conditional table/card rendering with viewMode switching
    - Cosmic theme CSS variables for all UI elements (var(--color-cosmic-*))

key-files:
  created: []
  modified:
    - app/src/pages/servers/ServerManagerPage.jsx

key-decisions:
  - "sortServers function placed before filtered computation to avoid temporal dead zone with const"
  - "Restart buttons use placeholder onClick handlers for Plan 02 wiring"
  - "StatusBadge imported for table view rows"
  - "Empty state shows contextual message based on active filter state"

requirements-completed: []

duration: 12 min
completed: 2026-06-15
---

# Phase 75 Plan 01: View Toggle, Sort, Filter, and Cosmic Theme Restyle

**View toggle (card/table), sort dropdown (Name/Status/Last Activity), game type filter (4 types), table view with 7 columns, and cosmic theme restyle for ServerManagerPage.jsx**

## Performance

- **Duration:** 12 min
- **Started:** 2026-06-15
- **Completed:** 2026-06-15
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Segmented button group with LayoutGrid/List lucide icons for card/table view toggle, persisted to localStorage
- Sort dropdown with 3 modes: Name A-Z, Running First (ordered status priority), Last Activity (by updated_at)
- Game type filter with 5 options: All Games, Minecraft Java, Minecraft Bedrock, PocketMine-MP, Nukkit
- Filter bar reordered: search input → status filter → sort dropdown → game filter → view toggle
- `sortServers()` utility function implementing all 3 sort modes with immutable copy pattern
- `filteredSorted` variable chains filter + sort before render
- Table view with cosmic theme: 7 columns (Name, Game, Host:Port, Image, Node, Status via StatusBadge, Actions with View/Restart/StartStop)
- Card view preserved as default with cosmic theme restyling — all `bg-gray-800`/`text-white` replaced with `var(--color-cosmic-card)`/`var(--color-text-main)`
- Page title and + Add Server button ported to cosmic theme variables
- Empty state shows contextual message: "No servers match your filters." when filters active, original message when no filters
- Restart buttons present in both card and table views with `/* restart - Plan 02 */` placeholder onClick
- Build verified: `npm run build` passes with no errors

## Task Commits

No git operations performed — orchestrator instructed to skip git. Changes applied directly to file.

## Files Created/Modified

- `app/src/pages/servers/ServerManagerPage.jsx` — 365 lines (was 199) with view toggle, sort, game filter, table view, cosmic theme

## Decisions Made

- **sortServers placement:** Moved before filtered/filteredSorted computation to avoid `const` temporal dead zone — the plan's original placement after handleStartStop would cause a ReferenceError
- **Restart placeholder buttons:** Both card and table view include restart buttons with `onClick={() => {/* restart - Plan 02 */}}` — intentionally disabled/wired in next plan
- **StatusBadge component:** Used in table view instead of inline status dot — follows legacy ServerManager.jsx pattern
- **Contextual empty state:** Shows different message when filters are active vs. no servers at all
- **Skeleton loading preserved:** Loading skeleton backgrounds remain as-is (pre-existing, out of scope)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Moved sortServers before filteredSorted to avoid ReferenceError**
- **Found during:** Task 2 (Add sort dropdown, sort utility, and game type filter)
- **Issue:** Plan placed `const sortServers = (servers, sortMode) => {...}` after `handleStartStop` (~line 69), but `const filteredSorted = sortServers(filtered, sortMode)` is computed earlier in the function body (~line 59). Since `const` declarations are NOT hoisted (temporal dead zone), calling `sortServers` before its declaration would throw `ReferenceError: Cannot access 'sortServers' before initialization`.
- **Fix:** Moved the `sortServers` function definition before the `filtered`/`filteredSorted` computation — between the initial `useEffect` and the filter logic. Execution order is now correct.
- **Files modified:** app/src/pages/servers/ServerManagerPage.jsx
- **Verification:** Build passes, all acceptance criteria pass

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor — function reordering has zero behavioral impact. All planned functionality delivered as specified.

## Stub Tracking

| Location | Stub | Reason |
|----------|------|--------|
| ServerManagerPage.jsx:281, 337 | `onClick={() => {/* restart - Plan 02 */}}` | Intentional placeholder — restart handler with confirmation modal wired in Plan 75-02 |

## Issues Encountered

- **`sortServers` temporal dead zone:** Plan placed `const sortServers` after `filteredSorted` usage. Fixed by reordering. No runtime impact.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. localStorage stores only non-sensitive preference keys (`serverViewMode`, `serverSortMode`, `serverGameFilter`). Table view displays same server data already exposed in card view.

## Next Phase Readiness

- Ready for Plan 75-02: Restart button handler with confirmation modal, 30s polling with status change toasts
- Restart placeholder buttons (`onClick={() => {/* restart - Plan 02 */}}`) in both card and table views — search for these comments to wire handlers
- `filteredSorted` variable used in both views — polling should use same sort/filter pipeline

---

*Phase: 75-update-ui-https-app-esluce-com-servers*
*Completed: 2026-06-15*
