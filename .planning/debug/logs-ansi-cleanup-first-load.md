---
status: awaiting_human_verify
trigger: "Fix two issues with log display: 1) Logs show 'Waiting for logs...' on first start instead of showing initial logs immediately. 2) Logs contain raw ANSI escape codes like `[33m`, `[0;39m`, `[1;31m` that are not being cleaned/stripped."
created: 2026-04-10T00:00:00Z
updated: 2026-04-10T00:00:00Z
---

## Current Focus

hypothesis: "Issue 1 fixed: hasLogs only set when data.logs.trim() has content - so empty container logs (but running server) shows 'Waiting for logs...'. Issue 2 fixed: No ANSI stripping was done on log data."
test: "Applied fixes to LogViewer.jsx"
expecting: "Both issues resolved"
next_action: "Verify the log viewer displays correctly"

## Symptoms

expected: "1. When server is running, logs should immediately show existing container logs on page load. 2. ANSI escape codes should be stripped so logs look clean"
actual: "1. First load shows 'Waiting for logs... Start the server to see live logs' - but server is already running. 2. Raw ANSI codes visible in log output: `[33m[mc-image-helper]`, `[0;39m[1;31m[mc-image-helper]`, etc."
errors: []
reproduction: "1. Navigate to server logs page with server already running - see 'Waiting for logs...' instead of existing logs. 2. View any logs and see raw ANSI codes"
started: "Unknown - reported issue"

## Eliminated

## Evidence

## Resolution

root_cause: "Issue 1: fetchInitialLogs only set hasLogs=true when data.logs.trim() returned content. If container was running but had no new logs, it showed 'Waiting for logs...' even though server was running. Issue 2: No ANSI escape code stripping was performed on log data from Docker."
fix: "1) Changed fetchInitialLogs to set hasLogs=true whenever API call succeeds (even if logs are empty), using the `fetchSucceeded` flag. 2) Added stripAnsiCodes() function to remove ANSI escape sequences like [33m, [0;39m, [1;31m from log text."
verification: "Requires testing in browser with running server - logs should show immediately on page load, and ANSI codes should not appear in log output"
files_changed: ["app/src/features/logs/LogViewer.jsx"]