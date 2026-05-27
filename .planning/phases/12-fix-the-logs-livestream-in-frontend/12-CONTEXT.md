# Phase 12: fix-the-logs-livestream-in-frontend - Context

**Gathered:** 2026-04-10
**Status:** Ready for verification

<domain>
## Phase Boundary

Fix issues with the log viewer in the frontend:
1. Logs show "Waiting for logs..." on first load instead of showing existing container logs immediately
2. Logs contain raw ANSI escape codes like `[33m`, `[0;39m`, `[1;31m` that are not cleaned/stripped
3. Stop button doesn't work on servers without node_id assigned

</domain>

<decisions>
## Implementation Decisions

### Log Display Issues
- **D-01:** Added `stripAnsiCodes()` function to remove ANSI escape sequences from log output - implemented in `app/src/features/logs/LogViewer.jsx`
- **D-02:** Changed `fetchInitialLogs` to set `hasLogs=true` whenever API call succeeds (not just when logs have content) - so "Waiting for logs..." doesn't show when server is running but has no new output

### Node Assignment
- **D-03:** Added auto-node-assignment logic to `get_logs`, `stop_server`, `restart_server`, `stream_logs`, and `delete_server` handlers in `api/src/presentation/handlers/server_handlers.rs` - finds connected node and persists node_id when missing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend
- `app/src/features/logs/LogViewer.jsx` — Main log viewer component with ANSI stripping and hasLogs fix
- `app/src/pages/servers/ServerDetailsPage.jsx` — Server details page that includes LogViewer

### Backend
- `api/src/presentation/handlers/server_handlers.rs` — Server handlers with auto-node-assignment logic

### Debug Sessions (for reference)
- `.planning/debug/logs-ansi-cleanup-first-load.md` — ANSI cleanup and first-load fix
- `.planning/debug/new-server-logs-and-stop.md` — Node assignment fix

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `LogViewer.jsx` - Already handles WebSocket connection via `useWebSocket` hook
- `stripAnsiCodes()` function - Now strips ANSI codes from log lines
- `lastLogLinesRef` Set - Tracks unique log lines to prevent duplicates

### Established Patterns
- Frontend uses Zustand for state management
- API calls use `api.get()` from `lib/api.js`
- WebSocket via custom `useWebSocket` hook

### Integration Points
- Logs fetched from `/api/v1/servers/:id/logs/:lines` endpoint
- Livestream via `/api/v1/servers/:id/logs/stream` endpoint
- WebSocket messages on `event` type with `LogOutput` payload

</code_context>

<specifics>
## Specific Ideas

No additional specific requirements — fixes are complete and in codebase.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 12-fix-the-logs-livestream-in-frontend*
*Context gathered: 2026-04-10*