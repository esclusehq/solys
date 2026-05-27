# Phase 22: fix-polling-logs-untuk-container-yang-tidak-ada - Context

**Gathered:** 2026-04-19

<domain>
## What We're Fixing

When user tries to fetch logs for a server/container that doesn't exist, frontend shows misleading "Waiting for logs..." message. Should show explicit error instead.

**Current behavior:**
- LogViewer shows "Waiting for logs..." when API call fails
- This is confusing because it's not "waiting" — the container just doesn't exist

</domain>

<decisions>
## Error Display

**Chosen:** Show explicit error message
- When container doesn't exist or can't be found
- Display "Container not found" or "Unable to fetch logs: [reason]"
- Not "Waiting for logs..."

## Files to Modify

1. `app/src/features/logs/LogViewer.jsx` - Show error when fetch fails
2. Check other log viewers (ServerDetails.jsx, Console.jsx)

</decisions>

<canonical_refs>
## References

- app/src/features/logs/LogViewer.jsx — main log viewer
- app/src/pages/ServerDetails.jsx — server details page
- app/src/pages/Console.jsx — console page

</canonical_refs>

<specifics>
## Implementation

In LogViewer.jsx fetchInitialLogs:
- On catch: setHasLogs(true) and setRawLogs to error message
- Change from "Unable to fetch logs" to more specific message

Example:
```javascript
} catch (err) {
  setHasLogs(true)
  setRawLogs(`Error: ${err.message || 'Container not found'}`)
}
```

</specifics>

<deferred>
## Deferred

- Caching logs for offline viewing (separate phase)
- WebSocket reconnection handling (already works)

</deferred>

---

## ▶ Next Up

`/clear` then:

/gsd-plan-phase 22 ${GSD_WS} — create plan from this context