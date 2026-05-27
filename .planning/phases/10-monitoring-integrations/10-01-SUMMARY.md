---
phase: 10-monitoring-integrations
plan: "01"
type: execute
wave: 1
autonomous: true
subsystem: monitoring
tags: [frontend, backend, metrics, recharts]
dependency_graph:
  requires: []
  provides:
    - api: GET /api/v1/metrics/:server_id/history
    - app: ResourceGraph component
    - app: ServerDetailsPage with historical graphs
  affects:
    - api: metrics_handlers.rs
    - app: ServerDetailsPage.jsx
tech_stack:
  added:
    - recharts (React charting library)
  patterns:
    - Recharts LineChart with tooltips
    - Historical metrics fetching
key_files:
  created:
    - api/src/presentation/handlers/metrics_handlers.rs
    - app/src/features/monitoring/ResourceGraph.jsx
  modified:
    - app/package.json
    - app/package-lock.json
    - api/src/presentation/routes/api_routes.rs
    - app/src/pages/servers/ServerDetailsPage.jsx
decisions:
  - "Used Recharts for charting - lightweight, React-native"
  - "Default to 24 hours of data, fetch via query param"
  - "Use cyan/purple/yellow colors for CPU/RAM/Disk respectively"
metrics:
  duration: ~2 min
  completed_date: "2026-04-09"
  tasks: 3
  files: 6
---

# Phase 10 Plan 01: Historical Resource Graphs with Recharts

## Summary

Added historical resource graphs to server details page using Recharts library. Users can now view CPU, RAM, and disk usage trends over the past 24 hours with interactive tooltips.

## Implementation

### Task 1: Install Recharts and Create ResourceGraph Component

- Installed `recharts` library to app
- Created `app/src/features/monitoring/ResourceGraph.jsx` with:
  - Recharts LineChart component
  - Props: data, dataKey, color, title, unit
  - XAxis showing time, YAxis with unit
  - Tooltip and Legend components
  - ResponsiveContainer wrapper

### Task 2: Add Historical Metrics API Endpoint

- Created `api/src/presentation/handlers/metrics_handlers.rs`
- Added `GET /metrics/:server_id/history` endpoint
- Accepts optional `hours` query parameter (default 24)
- Returns Vec<ServerMetrics> ordered by created_at descending

### Task 3: Integrate Historical Graphs into ServerDetailsPage

- Added state for `metricsHistory`
- Added useEffect to fetch historical data on mount
- Integrated three ResourceGraph components:
  - CPU Usage (cyan)
  - Memory Usage (purple)
  - Disk Usage (yellow)

## Verification

- [x] Recharts library installed
- [x] ResourceGraph component created with LineChart
- [x] Historical metrics API endpoint exists
- [x] Server details page shows three historical graphs
- [x] Graphs are interactive with tooltips
- [x] Page renders without errors

## Known Stubs

None - all data wired from API.

## Threat Flags

None - read-only metrics endpoint.

---

**Commit:** f045695

**Verified by:** Self-check passed
