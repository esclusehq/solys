# Phase 7: Server Status & Metrics - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Real-time server status and resource usage monitoring. This phase establishes status polling, metrics collection, display format, and alert thresholds.

**Requirements:** STATUS-01, STATUS-02

**Success criteria:**
1. User can view current server status (online/offline/starting/stopping)
2. User can view CPU usage percentage
3. User can view RAM usage (used/allocated)
4. User can view disk usage
</domain>

<decisions>
## Implementation Decisions

### Status Polling (D-24)
- **D-24:** Agent pull + Redis cache
- Pull from agent every 10s, cache in Redis
- Stale after 30s (no recent update)
- Reference: `api/src/presentation/ws/node_connection_manager.rs`

### Metrics Collection (D-25)
- **D-25:** 30s interval + 24h retention
- Collect metrics every 30 seconds
- Keep 24 hours in database
- Aggregate hourly/daily for historical views
- Reference: `api/src/infrastructure/executors/podman_server_executor.rs`

### Metrics Display (D-26)
- **D-26:** Current values + sparkline
- Show current CPU, RAM, disk values
- Last 24h sparkline in UI
- Detailed history on click/modal
- Reference: Frontend ServerDetailsPage

### Alert Thresholds (D-27)
- **D-27:** Threshold-based alerts
- CPU > 90% for 5 minutes
- RAM > 85% for 5 minutes
- Disk > 80% for 5 minutes
- Notify via email/Discord webhook

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Metrics Collection
- `api/src/infrastructure/executors/podman_server_executor.rs` — collect_metrics
- `api/src/domain/entities/server_metrics.rs` — ServerMetrics entity

### Status
- `api/src/presentation/ws/node_connection_manager.rs` — Connection state
- `api/src/application/services/monitoring_service.rs` — Monitoring logic

### Usage
- `api/src/domain/usage/service.rs` — Usage tracking

### Frontend
- `app/src/pages/servers/ServerDetailsPage.jsx` — Metrics display

</canonical_refs>

<specifics>
## Specific Ideas

- ServerMetrics entity has: cpu_usage, ram_usage, disk_usage, network_in, network_out
- Resource plans define limits: 2GB→2 cores, 4GB→3 cores, 8GB→4 cores, 16GB→6 cores
- Usage service tracks user consumption against plan limits

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 07-server-status-metrics*
*Context gathered: 2026-04-09*
