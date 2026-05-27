---
status: awaiting_human_verify
trigger: "pending servers not showing in Running or Stopped filter"
created: 2026-04-21T00:00:00Z
updated: 2026-04-21T00:00:00Z
---

## Current Focus

hypothesis: Filter dropdown missing "pending" option causes pending servers to be hidden
test: Add "pending" to filter dropdown options
expecting: Pending servers will be visible when filtering by "pending"
next_action: Add pending filter option
</status>

## Symptoms

expected: Running (1) + Stopped (2) + Pending (2) = 5 servers visible
actual: Pending servers only visible in "All" filter
errors: None
reproduction: Filter by "Running" or "Stopped" - pending servers missing
started: New issue

## Eliminated

- hypothesis: Filter logic bug - s.status === filter comparison
  evidence: This works correctly - pending has status 'pending', filter has 'running', so no match (expected behavior)
  timestamp: 2026-04-21

## Evidence

- timestamp: 2026-04-21
  checked: ServerManagerPage.jsx filter logic (lines 17-21)
  found: Filter logic correct - compares s.status === filter value
  implication: Issue is in dropdown options, not filter logic

- timestamp: 2026-04-21
  checked: Filter dropdown options (lines 74-77)
  found: Only 'all', 'running', 'stopped' - missing 'pending'
  implication: Pending servers have status='pending' which doesn't match 'running' or 'stopped'

## Resolution

root_cause: Filter dropdown missing 'pending' option - pending servers invisible when filtering by running/stopped
fix: Add `<option value="pending">Pending</option>` to filter dropdown
verification: Select "Pending" filter shows test2, test3 servers
files_changed:
  - app/src/pages/servers/ServerManagerPage.jsx: Added pending filter option