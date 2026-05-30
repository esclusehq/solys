---
phase: 56-auto-online-sleep-recovery
plan: 04
subsystem: ui
tags: react, tailwind, sleep-wake, status-badge, settings

# Dependency graph
requires:
  - phase: 56-auto-online-sleep-recovery
    provides: Phase context (UI-SPEC.md, CONTEXT.md, sleep/wake API endpoints)
provides:
  - Sleeping status badge (orange dot + "💤 Sleeping") for servers with auto_wake=true
  - Sleep/Wake action button in ServerDetails header with conditional labels
  - Sleep & Wake configuration section in ServerDetails Settings tab
  - autoWake prop wiring across all StatusBadge usages in the app
  - sleepServer() and wakeServer() API functions in useServers + serversApi
  - .status-dot.sleeping CSS class with orange glow effect

affects:
  - 56-05 (any remaining frontend/backend integration)

tech-stack:
  added: []
  patterns:
    - Sleep/wake action button with conditional labels per auto_wake state
    - Settings tab config section following Discord webhook section pattern
    - StatusBadge autoWake prop for sleeping state detection

key-files:
  created: []
  modified:
    - app/src/components/StatusBadge.jsx
    - app/src/index.css
    - app/src/hooks/useServers.js
    - app/src/pages/ServerDetails.jsx
    - app/src/pages/ServerManager.jsx
    - app/src/pages/ServerManagerPage.jsx
    - app/src/pages/Dashboard.jsx
    - app/src/lib/api.js

key-decisions:
  - "auto_wake=false is the default for StatusBadge to avoid breaking existing usage"
  - "Button label matrix: running+auto_wake → 💤 Sleep (red), running+!auto_wake → ■ Stop (red), stopped+auto_wake → 💤 Wake (cyan), stopped+!auto_wake → ▶ Start (cyan)"
  - "Sleep config toggle is optimistic (immediate visual change), persists on Save button click"
  - "ServerManagerPage uses getDisplayStatus helper for sleeping state without modifying actual server status"
  - "Timeout input clamped to 5-240 range on change event per UI-SPEC and threat model T-56-13"

requirements-completed: []

duration: 12 min
completed: 2026-05-30
---

# Phase 56: Auto Online & Sleep Recovery — Plan 04 Summary

**Frontend UI for sleep/wake: sleeping badge state, sleep/wake action button, and Auto Sleep config panel in Settings tab**

## Performance

- **Duration:** 12 min
- **Started:** 2026-05-30T15:20:00Z
- **Completed:** 2026-05-30T15:32:00Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- StatusBadge renders "💤 Sleeping" with orange glowing dot when status=stopped + autoWake=true
- CSS `.status-dot.sleeping` class with orange background and glow
- sleepServer() and wakeServer() API functions in useServers hook and serversApi
- ServerDetails header button shows sleep/wake labels per auto_wake state matrix
- handleToggle dispatches to sleepServer/wakeServer/stopServer/startServer based on context
- Settings tab has Sleep & Wake section with Auto Sleep toggle, timeout input (5-240 min), and Save button
- Toast notifications on config save success/error with 4s auto-dismiss
- ServerManagerPage shows sleeping servers with orange dot and "💤 Wake" button
- ServerManager.jsx table shows sleep/wake-aware toggle button with appropriate labels and colors
- Dashboard StatusBadge passes autoWake prop for sleeping state detection

## Task Commits

Each task was committed atomically in the `app` submodule:

1. **Task 1: Sleeping status badge, CSS, and sleep/wake API calls** - `app@a0f1355` (feat)
2. **Task 2: Sleep/wake toggle and Sleep & Wake settings tab** - `app@ab743c1` (feat)
3. **Task 3: autoWake prop wiring across ServerManager, Dashboard, and ServerManagerPage** - `app@251a697` + `app@4002880` (feat)

## Files Created/Modified

- `app/src/components/StatusBadge.jsx` - Added `autoWake` prop and sleeping state rendering
- `app/src/index.css` - Added `.status-dot.sleeping` with orange glow
- `app/src/hooks/useServers.js` - Added `sleepServer()` and `wakeServer()` exports
- `app/src/pages/ServerDetails.jsx` - Sleep/wake handleToggle, button labels, Settings tab section
- `app/src/pages/ServerManager.jsx` - Sleep/wake-aware toggle, StatusBadge autoWake prop
- `app/src/pages/servers/ServerManagerPage.jsx` - getDisplayStatus, sleep/wake-aware start/stop
- `app/src/pages/Dashboard.jsx` - StatusBadge autoWake prop
- `app/src/lib/api.js` - Added `sleep` and `wake` methods to serversApi

## Decisions Made

- autoWake defaults to `false` in StatusBadge to maintain backward compatibility with existing usage
- Button labels follow the UI-SPEC matrix: running+auto_wake=💤 Sleep, running+!auto_wake=■ Stop, stopped+auto_wake=💤 Wake, stopped+!auto_wake=▶ Start
- Sleep config toggle is optimistic (immediate visual toggle), persisted only on Save button click
- ServerManagerPage.jsx uses a helper function `getDisplayStatus` that maps `stopped+auto_wake` → "sleeping" for display without changing the actual status field
- Timeout input is clamped to 5-240 range on every change event (not just on save) to prevent out-of-range values per T-56-13

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Wired autoWake prop in ServerManager.jsx and Dashboard.jsx**
- **Found during:** Task 3 (autoWake prop wiring)
- **Issue:** Plan's read_first only mentioned ServerManagerPage.jsx, but two other files (ServerManager.jsx and Dashboard.jsx) also render StatusBadge without the autoWake prop, which would show sleeping servers as plain "Stopped"
- **Fix:** Updated StatusBadge usage in ServerManager.jsx table view and Dashboard.jsx server list to pass `autoWake={s.auto_wake}`. Also updated ServerManager.jsx handleToggle to be sleep/wake aware and added sleep/wake button labels.
- **Files modified:** app/src/pages/ServerManager.jsx, app/src/pages/Dashboard.jsx
- **Verification:** grep confirms autoWake prop passed in all 4 StatusBadge usage locations
- **Committed in:** `app@4002880` (Task 3 commit)

**2. [Rule 2 - Missing Critical] Added sleep/wake methods to serversApi**
- **Found during:** Task 3 (ServerManagerPage implementation)
- **Issue:** ServerManagerPage.jsx uses `serversApi` (from `../../lib/api`) for API calls, not the `useServers` hook. Without sleep/wake methods on serversApi, the sleep/wake dispatch in handleStartStop would fail.
- **Fix:** Added `sleep: (id) => api.post(/servers/:id/sleep)` and `wake: (id) => api.post(/servers/:id/wake)` to serversApi in api.js
- **Files modified:** app/src/lib/api.js
- **Verification:** grep confirms sleep/wake methods exist in api.js
- **Committed in:** `app@251a697` (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (2 missing critical)
**Impact on plan:** Both auto-fixes essential for correct functionality — without them, sleeping servers would appear as plain "Stopped" in some views and the sleep/wake API would be unavailable to ServerManagerPage.

## Issues Encountered

None.

## Known Stubs

No stubs found — all UI elements are wired to their data sources through existing server API responses.

## Threat Flags

None — all files modified are UI components with no new network endpoints, auth paths, or schema changes at trust boundaries.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Frontend sleep/wake UI complete for all server views (details, manager, dashboard)
- Ready for backend integration (sleep/wake API endpoints, monitoring service sleep detection)
- When `POST /servers/:id/sleep` and `POST /servers/:id/wake` endpoints are implemented, the full sleep/wake flow will be end-to-end functional

---

### Self-Check: PASSED

- ✅ `app/src/components/StatusBadge.jsx` - autoWake prop + sleeping state present
- ✅ `app/src/index.css` - `.status-dot.sleeping` rule present with orange glow
- ✅ `app/src/hooks/useServers.js` - sleepServer/wakeServer exported
- ✅ `app/src/pages/ServerDetails.jsx` - sleep/wake toggle + settings tab section present
- ✅ `app/src/pages/ServerManager.jsx` - sleep/wake aware toggle + autoWake prop
- ✅ `app/src/pages/servers/ServerManagerPage.jsx` - getDisplayStatus + autoWake-aware start/stop
- ✅ `app/src/pages/Dashboard.jsx` - autoWake prop passed to StatusBadge
- ✅ `app/src/lib/api.js` - sleep/wake methods on serversApi
- ✅ `app` git log shows 4 feat(56-04) commits

*Phase: 56-auto-online-sleep-recovery*
*Completed: 2026-05-30*
