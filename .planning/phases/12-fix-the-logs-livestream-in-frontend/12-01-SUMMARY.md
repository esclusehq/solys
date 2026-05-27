---
phase: 12
plan: 01
subsystem: frontend-logs-backend-handlers
tags: [logs, livestream, ansi-strip, node-assignment]
dependency_graph:
  requires: []
  provides: []
  affects: [app/src/features/logs/LogViewer.jsx, api/src/presentation/handlers/server_handlers.rs]
tech_stack:
  added: []
  patterns: [ANSI escape code stripping, hasLogs state fix, auto-node-assignment]
key_files:
  created: []
  modified:
    - app/src/features/logs/LogViewer.jsx
    - api/src/presentation/handlers/server_handlers.rs
decisions:
  - Added stripAnsiCodes function to remove ANSI escape sequences from log output
  - Changed hasLogs to set true whenever API call succeeds (not just when logs have content)
  - Added auto-node-assignment in stop_server, restart_server, get_logs, stream_logs, delete_server handlers
metrics:
  duration_minutes: 5
  tasks_completed: 3
  completed_date: 2026-04-10
---

# Phase 12 Plan 01: Fix Logs Livestream in Frontend — Summary

## Overview

One-liner: Fix 3 log viewer issues: ANSI escape codes, first-load display, and stop button on servers without node_id

## Verification Results

### Task 1: ANSI stripping in LogViewer ✓

- `stripAnsiCodes` function defined at lines 6-17 in LogViewer.jsx
- Function removes: `\x1b[`, `\x1b`, and bracket patterns like `[33m`
- Applied to 5 locations:
  - Line 34: WebSocket payload.line
  - Line 65: data.logs from initial fetch
  - Line 75: data.message format
  - Line 79: string format
- **PASS**: 5 stripAnsiCodes usages found

### Task 2: hasLogs fix for first-load ✓

- `setHasLogs(true)` called whenever API returns (even if logs empty)
- Applied at 5 locations:
  - Line 38: WebSocket log line received
  - Line 68: data.logs exists (API succeeded)
  - Line 77: data.message format (API succeeded)
  - Line 81: string format (API succeeded)
  - Line 90: "No logs available" case
- **PASS**: 5 setHasLogs(true) calls found

### Task 3: Auto-node-assignment in stop_server ✓

- Lines 594-603: Uses server.node_id if set and connected
- Lines 606-627: Finds first online node from node_repository if not connected
- Lines 630-635: Saves node_id to database with UPDATE query
- **PASS**: 5 UPDATE queries found across handlers (stop, restart, get_logs, stream_logs, delete)

## Summary

All 3 fixes verified in codebase:
1. **ANSI stripping** — stripAnsiCodes function defined and used on all log output paths
2. **First-load display** — setHasLogs(true) called on any successful API response
3. **Auto-node-assignment** — stop_server (and other handlers) auto-find and persist node_id when missing

## Deviations from Plan

None — all 3 tasks verified exactly as specified in plan.

## Self-Check: PASSED

- [x] app/src/features/logs/LogViewer.jsx exists (201 lines)
- [x] stripAnsiCodes function found at line 6
- [x] 5 setHasLogs(true) calls found
- [x] api/src/presentation/handlers/server_handlers.rs exists (1540+ lines)
- [x] 5 UPDATE servers SET node_id queries found
- [x] auto-node-assignment logic in stop_server verified (lines 594-635)