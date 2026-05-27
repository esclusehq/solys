---
phase: 40-backend-agent-stability
plan: 01
subsystem: api
tags: [rust, axum, websocket, heartbeat, node-health, monitoring]

# Dependency graph
requires:
  - phase: 39-hardening-agent
    provides: WebSocket agent hardening and reconnection logic
provides:
  - NodeHealthStatus with ONLINE, OFFLINE, DEGRADED states
  - Configurable heartbeat interval from node.metadata
  - Interval-based offline/degraded detection (3x and >50%)
  - MonitoringService skip for offline nodes
affects: [monitoring, node-connection, agent-reconnect]

# Tech tracking
tech-stack:
  added: []
  patterns: [heartbeat-interval-based health evaluation]

key-files:
  modified:
    - api/src/domain/entities/node_health.rs
    - api/src/application/services/node_health_service.rs
    - api/src/application/services/monitoring_service.rs

key-decisions:
  - "Default heartbeat interval: 10 seconds (D-02)"
  - "OFFLINE trigger: 3x interval (30s for 10s default)"
  - "DEGRADED trigger: >50% interval late (D-05)"
  - "Monitoring skipping offline nodes without errors (D-07, D-08)"

patterns-established:
  - "Heartbeat interval configurable per node via node.metadata"
  - "Status evaluation uses is_online + heartbeat_age_seconds"
  - "MonitoringService gets node_repository for status checks"

requirements-completed: []

# Metrics
duration: 5min
completed: 2026-05-03
---

# Phase 40-01 Plan: Backend Agent Connection Stability Summary

**Heartbeat-based node status tracking with ONLINE/OFFLINE/DEGRADED states and configurable intervals**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-03
- **Completed:** 2026-05-03
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Extended NodeHealthStatus enum with Online, Offline, Degraded variants
- Added configurable heartbeat interval from node.metadata (default 10s)
- Updated monitoring to skip servers on offline nodes

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend NodeHealthStatus with ONLINE, OFFLINE, DEGRADED states** - `5f75d2c` (feat)
2. **Task 2: Add configurable heartbeat interval and degraded evaluation** - `dc0abfe` (feat)
3. **Task 3: Skip server monitoring for offline nodes** - `e91cf2e` (feat)

**Plan metadata:** N/A (docs per-task only)

## Files Created/Modified
- `api/src/domain/entities/node_health.rs` - Added Online/Offline/Degraded, evaluate_with_interval()
- `api/src/application/services/node_health_service.rs` - Added get_heartbeat_interval(), interval-based evaluation
- `api/src/application/services/monitoring_service.rs` - Added node status check before server monitoring

## Decisions Made

- Default heartbeat interval: 10 seconds (D-02)
- OFFLINE trigger: 3x interval (30s for default 10s)
- DEGRADED trigger: >50% interval late OR stale metrics
- Monitoring skips offline nodes without errors (D-07, D-08)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- MonitoringService needed node_repository injected (bootstrap/container.rs update) - fixed as blocking issue

## Next Phase Readiness

- Node health status states ready
- Interval-based evaluation complete
- Monitoring skip for offline nodes ready
- Build passes with warnings only

---
*Phase: 40-backend-agent-stability*
*Completed: 2026-05-03*