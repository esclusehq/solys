---
status: testing
phase: 12-fix-the-logs-livestream-in-frontend
source: 12-01-SUMMARY.md
started: 2026-04-11T00:00:00Z
updated: 2026-04-11T00:03:00Z
---

## Current Test

number: 4
name: Restart server without node_id
expected: |
  Restart a server that has no node_id assigned. The handler should automatically find an online node and assign it to the server before performing the restart operation.
awaiting: user response

## Tests

### 3. Stop server without node_id
expected: Stop a server that has no node_id assigned. The handler should automatically find an online node and assign it to the server before performing the stop operation.
result: pass

### 4. Restart server without node_id
expected: Restart a server that has no node_id assigned. The handler should automatically find an online node and assign it to the server before performing the restart operation.
result: [pending]

## Summary

total: 4
passed: 3
issues: 0
pending: 1
skipped: 0
blocked: 0
skipped: 0
blocked: 0

## Gaps
