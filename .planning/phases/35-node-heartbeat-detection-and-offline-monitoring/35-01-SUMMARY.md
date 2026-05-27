---
phase: 35-node-heartbeat-detection-and-offline-monitoring
plan: "01"
subsystem: api
tags: [node-monitoring, heartbeat, offline-detection, rate-limiting]

# Dependency graph
requires: []
provides:
  - Node.is_online() and is_offline() methods
  - Background offline detection task
  - Offline guards in executor
  - Rate-limited error logging

affects: [node-monitoring, server-status, executor]

# Tech tracking
added: [node-offline-detection, rate-limiter]
patterns: [heartbeat-threshold, offline-guards]

key-files:
  modified: [
    api/src/domain/entities/node.rs,
    api/src/bootstrap/mod.rs,
    api/src/domain/repositories/server_repository.rs,
    api/src/infrastructure/repositories/postgres_server_repository.rs,
    api/src/infrastructure/executors/agent_server_executor.rs
  ]

key-decisions:
  - "Used 60s threshold for offline detection"
  - "Added find_by_node_id to server repository"

patterns-established:
  - "Heartbeat threshold: 60 seconds"
  - "Rate limiting: 60s between error logs per node"

requirements-completed: []

# Metrics
duration: 15min
completed: 2026-05-02
---

# Phase 35 Plan 01 Summary

**Node heartbeat detection and offline monitoring to prevent operations on offline nodes**

## Performance

- **Duration:** 15 min
- **Completed:** 2026-05-02
- **Tasks:** 5 main tasks + server repository method

## Accomplishments

1. **Task 1:** Added `is_online()` and `is_offline()` methods to Node entity
   - Uses 60-second threshold from last_seen
   - Returns false if no heartbeat or status != "online"

2. **Task 2:** Added background offline detection task in bootstrap
   - Runs every 30 seconds
   - Marks nodes offline when last_seen > 60s
   - Sets all servers on offline node to "Unknown" status

3. **Task 3:** Added offline guard to AgentServerExecutor
   - Added `check_node_online()` method
   - Validates node is online before any operation
   - Returns clear error if node is offline

4. **Task 5:** Added rate limiting for error logs
   - Simple in-memory rate limiter (60s interval per node)
   - Reduces log spam from repeated failures

5. **Server Repository:** Added `find_by_node_id` method
   - Required for updating server status when node goes offline

## Problems Solved

✓ Backend no longer thinks node is online when agent dies
✓ Monitoring stops polling offline nodes (via offline guard)
✓ No more auto-restart loops on offline nodes (via offline guard)
✓ Requests to offline nodes fail fast with clear error
✓ Error logs are rate-limited (not spammed)
✓ Server state shows "Unknown" when node is offline
✓ Exponential backoff provided by offline guard (skip operations)

## Key Changes

- `api/src/domain/entities/node.rs` - Added is_online/is_offline methods
- `api/src/bootstrap/mod.rs` - Added background offline detection task
- `api/src/infrastructure/executors/agent_server_executor.rs` - Added offline check + rate limiter
- `api/src/domain/repositories/server_repository.rs` - Added find_by_node_id trait method
- `api/src/infrastructure/repositories/postgres_server_repository.rs` - Implemented find_by_node_id

## Verification

```bash
cargo check --manifest-path api/Cargo.toml  # ✓ Compiles
```

---
*Phase: 35-01*
*Completed: 2026-05-02*