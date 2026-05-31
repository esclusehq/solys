---
phase: 60-crash-detection
plan: 04
subsystem: frontend
tags: ui, crash-history, settings, pagination
requires:
  - phase: 60-crash-detection
    provides: Phase context (CONTEXT.md, RESEARCH.md, PATTERNS.md)
  - phase: 60-crash-detection
    plan: 03
    provides: Backend crash log API endpoints (GET/DELETE /crash-logs, POST /resolve)
provides:
  - useCrashLogs hook with paginated fetch, clear, and acknowledge
  - 3 crash log API functions in client.js
  - Crash History section in ServerDetails Settings tab
affects: []
tech-stack:
  added: []
  patterns:
    - React hook pattern with useState + useEffect + useCallback
    - fetchApi wrapper for all API calls
    - Settings tab section with glass-panel styling
key-files:
  created:
    - app/src/hooks/useCrashLogs.js
  modified:
    - app/src/api/client.js
    - app/src/pages/ServerDetails.jsx
key-decisions:
  - "PAGE_SIZE = 10 for crash log pagination"
  - "Crash History section placed after Scheduled Actions in Settings tab ordering"
  - "4 crash type badges: OOM (red), Config Error (yellow), Plugin Crash (orange), Generic (default)"
  - "Toast auto-dismiss after 4 seconds matching existing pattern"
requirements-completed: []
duration: 3 min
completed: 2026-05-31
---

# Phase 60 Plan 04: Crash History UI Summary

**Added Crash History UI to ServerDetails Settings tab** — paginated crash log list with type badges, log excerpt, clear/resolve actions, and toast notifications.

- `useCrashLogs.js` hook with PAGE_SIZE=10, auto-fetch on serverId/page change, `clearLogs()` (DELETE), `acknowledge(logId)` (POST /resolve)
- 3 API functions in client.js: `getCrashLogs`, `clearCrashLogs`, `acknowledgeCrash`
- Crash History section in Settings tab after Scheduled Actions with: empty state, loading state, paginated crash entries with type badges (OOM/Config/Plugin/Generic), log excerpt in pre block, Mark as Resolved button, Previous/Next pagination, Clear Crash History button, toast messages

## Task Commits

1. **Task 1-3: Hook, API functions, and UI section** — `26a1bc1` in app sub-repo

## Files Created/Modified

### New
- `app/src/hooks/useCrashLogs.js` — 68 lines (useCrashLogs hook with 9 return values)

### Modified
- `app/src/api/client.js` — added getCrashLogs, clearCrashLogs, acknowledgeCrash
- `app/src/pages/ServerDetails.jsx` — added Crash History section (~155 lines of JSX + handlers + helper components)

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- useCrashLogs.js created with all 9 return values (logs, total, page, totalPages, loading, setPage, clearLogs, acknowledge, refresh)
- client.js has all 3 API functions with correct endpoints
- ServerDetails.jsx imports useCrashLogs and renders Crash History section
- Empty state ("No crash history") with dashed border placeholder
- Loading state ("Loading crash history...") centered
- Crash entries show type badge, timestamp, exit code, recovery action, log excerpt (pre), Mark as Resolved button
- Pagination with Previous/Next and page number buttons
- Clear Crash History button with red styling
- Toast messages for success/error
- CrashTypeBadge component with 4 types
- formatRecoveryAction helper for display labels
