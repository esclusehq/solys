---
phase: 75-update-ui-https-app-esluce-com-servers
plan: 02
subsystem: ui
tags: react, polling, toast, confirmation-modal, restart

requires:
  - phase: 75-update-ui-https-app-esluce-com-servers
    provides: ServerManagerPage.jsx with cards, table view, sort/filter, cosmic theme (Plan 01)
provides:
  - Restart button with confirmation modal (cards + table)
  - 30-second polling with status change detection
  - Toast notifications on restart start/success/error
  - Toast notifications on server status changes between polls
affects: []

tech-stack:
  added: []
  patterns:
    - useRef-based comparison for detecting state changes between polls
    - Inline confirmation modal with backdrop dismiss (stopPropagation)
    - setInterval polling with cleanup via clearInterval in useEffect return

key-files:
  created: []
  modified:
    - app/src/pages/servers/ServerManagerPage.jsx

key-decisions:
  - "Restart triggered via confirmation modal, not inline action (prevents accidental restarts)"
  - "Servers disabled on starting/stopping/crashed status (matching Start/Stop button behavior)"
  - "useServerStore.getState().servers used inside interval (not closure) to always read latest state"
  - "useRef stores previous snapshot to compare status changes without useEffect dependency on servers (avoiding infinite loop)"
  - "Status change fires for ALL transitions including starting→running, not just stopped→running"

requirements-completed: []

duration: 8min
completed: 2026-06-15
---

# Phase 75 Plan 02: Restart buttons with confirmation modal, 30s polling, status change toasts

**Restart buttons wired with confirmation modal (cards + table), 30s polling with `useServerStore.getState().servers` for fresh reads, status diff detection with info toasts, `handleStartStop` upgraded from `console.error` to `addToast`**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-15 (inline)
- **Completed:** 2026-06-15
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Restart buttons on both card view and table view call `setRestartServer(server)` to open confirmation modal
- Confirmation modal with "Cancel" (closes via `setRestartServer(null)`) and "Restart" (calls `serversApi.restart(server.id)`)
- Restart toast sequence: info on start, success on completion, error on failure
- `handleStartStop` catch block uses `addToast` instead of `console.error`
- 30-second polling via `setInterval` in `useEffect` with `clearInterval` cleanup
- Status change detection compares `oldS.status !== newS.status` between polls and fires info toasts (`"{name} is now {status}"`)
- `useRef` stores previous snapshot; `useServerStore.getState().servers` reads latest state inside interval
- Init `useEffect` populates `prevServersRef` when servers first load
- No polling indicator in UI (silent refresh per UI-SPEC)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire Restart buttons + add confirmation modal** - (no git — inline)
2. **Task 2: Add 30s polling with status change detection and toasts** - (no git — inline)

**Plan metadata:** (no git — inline)

_Note: No git operations available per execution context._

## Files Created/Modified

- `app/src/pages/servers/ServerManagerPage.jsx` - Added `useRef` import, `addToast`/`prevServersRef`/`restartServer` state, wired Restart buttons, confirmation modal, polling + status detection useEffects, upgraded handleStartStop toasts

## Decisions Made

- Used `useServerStore.getState().servers` inside interval instead of closure variable to always get latest state (per RESEARCH.md Pattern 1)
- `useRef` for previous snapshot avoids `useEffect` dependency on `servers`, preventing infinite loop
- Restart button disabled when status is `starting`, `stopping`, or `crashed` (matching existing Start/Stop behavior)
- Modal closes on Cancel button click, Restart button click, or backdrop click
- Status change toasts fire for ALL transitions, not just specific ones (per RESEARCH.md Open Q2 recommendation)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## Stub Tracking

No stubs identified. All features fully wired:
- Restart buttons → confirmation modal → API call → toast sequence → refresh
- Polling → ref comparison → status change toasts
- `handleStartStop` → proper error toasts instead of `console.error`

## Threat Flags

No new threat surface introduced beyond what <threat_model> already covers (restart via existing authenticated API, toast content derived from UI-visible server data, polling via existing JWT-authenticated fetch).

## Next Phase Readiness

Ready for next plan in Phase 75 (if any), or Phase 76.

---

*Phase: 75-update-ui-https-app-esluce-com-servers*
*Completed: 2026-06-15*
