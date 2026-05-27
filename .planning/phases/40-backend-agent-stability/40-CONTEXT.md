# Phase 40: BACKEND ↔ AGENT STABILITY - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Backend handling of WebSocket agent connections to eliminate "node not connected" issues — heartbeat monitoring, node status tracking, offline handling, and reconnection logic.

</domain>

<decisions>
## Implementation Decisions

### Heartbeat System
- **D-01:** Heartbeat interval is configurable (per-node setting)
- **D-02:** Default heartbeat: 10 seconds
- **D-03:** Payload includes: CPU, RAM, Disk, Uptime

### Node Status
- **D-04:** OFFLINE trigger: No heartbeat for 3x the configured interval (30s for 10s default)
- **D-05:** DEGRADED trigger: Heartbeat late (>50% interval) OR metrics stale OR high CPU/RAM threshold OR reconnecting attempts
- **D-06:** States: ONLINE, OFFLINE, DEGRADED

### Offline Handling
- **D-07:** When agent goes OFFLINE: Stop monitoring servers on that node
- **D-08:** Do not spam retry — wait for agent to reconnect

### Reconnect Logic
- **D-09:** Reconnection initiated by agent (Phase 39 already has retry/backoff)
- **D-10:** When agent reconnects: Sync state + resume monitoring

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `api/src/presentation/ws/node_connection_manager.rs` — existing connection manager
- `api/src/application/services/node_health_service.rs` — existing health service
- `api/src/domain/entities/node_health.rs` — existing health entity
- `agent-core/crates/agent-config/src/schema.rs` — agent config (Phase 39)

</canonical_refs>

## Existing Code Insights

### Reusable Assets
- `NodeConnectionManager` in `api/src/presentation/ws/node_connection_manager.rs` — manages WebSocket connections
- `NodeHealthService` in `api/src/application/services/node_health_service.rs` — checks node health
- `NodeHealth` entity in `api/src/domain/entities/node_health.rs` — health status types

### Established Patterns
- Uses `Arc<RwLock<HashMap<Uuid, NodeSender>>>` for connection storage
- `is_connected()` method already exists in NodeConnectionManager
- Health status evaluation uses `NodeHealthStatus` enum

### Integration Points
- NodeConnectionManager is injected into ApiState and used across handlers
- NodeHealthService uses connection_manager to check `is_connected()`
- Server handlers check connection before sending commands

</code_context>

<specifics>
## Specific Ideas

No specific references — standard patterns for node connection stability.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 40-backend-agent-stability*
*Context gathered: 2026-05-03*