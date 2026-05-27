---
phase: 22-fix-polling-logs-untuk-container-yang-tidak-ada
plan: 01
status: complete
completed: 2026-04-20
---

## Summary

Fixed log viewer to show specific error messages when container doesn't exist instead of "Waiting for logs..."

## What Was Fixed

**LogViewer.jsx catch block (lines 93-96):**
```javascript
} catch (err) {
  setHasLogs(true)
  const errMsg = err?.response?.data?.error || err?.message || err?.toString() || 'Container not found'
  setRawLogs(`Error: ${errMsg}`)
}
```

- Sets `hasLogs(true)` so error displays instead of "Waiting for logs..."
- Extracts specific error from API response or falls back to "Container not found"
- Prefixes with "Error: " for clarity

## Verification

- [x] Catch block sets hasLogs(true) - error displays
- [x] Error message is specific (container not found vs generic)
- [x] No more "Waiting for logs..." for non-existent containers