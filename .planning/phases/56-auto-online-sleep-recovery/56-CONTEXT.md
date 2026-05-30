# Phase 56: Auto Online & Sleep Recovery - Context

**Gathered:** 2026-05-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver automatic online/sleep recovery for game servers — detect when servers go idle or crash and automatically restore them. Covers sleep mode detection (player inactivity), manual sleep action, automatic wake-up mechanisms, and refinement of existing auto-restart logic.

Builds on existing infrastructure: MonitoringService (30s polling loop, already handles crash→restart), auto_restart + auto_pause fields on Server entity, NodeHealthService for node-level heartbeat.

</domain>

<decisions>
## Implementation Decisions

### Server State Model
- **D-01:** **No new 'sleeping' status.** Keep existing server status values ('running', 'starting', 'stopped', 'container_running'). Add an `auto_wake` boolean field to indicate a server that is stopped but will auto-recover. UI shows a different badge for servers with `auto_wake=true` + status=`stopped`.
- **D-02:** **Sleep triggers — both manual and automatic.** User can click 'Sleep' in the UI (same as stop but sets `auto_wake=true`), AND servers auto-sleep after player inactivity timeout. Both paths produce the same state: status=`stopped`, `auto_wake=true`.
- **D-03:** **Inactivity detection via existing monitoring loop.** MonitoringService (30s tick) checks player count via executor (RCON). If 0 players for >X configurable minutes, triggers sleep. No agent-side changes needed.
- **D-04:** **auto_pause kept separate.** The existing `auto_pause` field is unrelated to sleep behavior and preserved for future use (pause = freeze in memory, sleep = stop + auto-restore).

### the agent's Discretion
- Specific inactivity timeout duration and configuration (default, min/max)
- Wake-up trigger implementation (player connection attempt detection, API-based wake)
- Auto-restart refinement: max restart attempts, cooldown periods, exponential backoff, failure alerts
- UI placement of sleep/wake configuration (server settings tab vs inline controls)
- Database migration design for `auto_wake` column
- MonitoringService integration details (where in the 30s loop to inject sleep detection)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Server & Monitoring Infrastructure
- `api/src/domain/entities/server.rs` — Server entity with auto_restart, auto_pause, status fields. Target for auto_wake addition.
- `api/src/application/services/monitoring_service.rs` — 30s monitoring loop. Already handles crash→auto-restart (lines 137-189). Sleep detection will be added here.
- `api/src/application/services/node_health_service.rs` — Node-level heartbeat health. Reference pattern for health evaluation.
- `api/src/application/services/scheduler_service.rs` — Cron task execution. Reference for how the Worker processes scheduled operations.
- `api/src/domain/entities/node_health.rs` — NodeHealthStatus enum (Online, Offline, Degraded). Reference for health state pattern.
- `api/src/application/dto/server_dtos.rs` — Server DTOs with existing auto_pause field.
- `api/src/application/use_cases/create_server_use_case.rs` — Server creation with auto_pause wiring.
- `api/src/application/use_cases/update_server_use_case.rs` — Server update with auto_pause wiring.
- `api/src/presentation/handlers/settings_handlers.rs` — Existing settings endpoints (reference for sleep config endpoints).

### Prior Phase Context
- `.planning/phases/55-scheduled-backups/55-CONTEXT.md` — D-04: Worker-only task automation pattern. Monitoring loop patterns.

### Codebase Maps
- `.planning/codebase/ARCHITECTURE.md` — Service layer architecture, MonitoringService placement
- `.planning/codebase/STACK.md` — Technology stack (Axum, Tokio, sqlx, cron)
- `.planning/codebase/INTEGRATIONS.md` — External integrations (PostgreSQL, Redis, WebSocket)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `MonitoringService` (`monitoring_service.rs`) — Existing 30s polling loop with crash detection. Natural injection point for sleep detection & debounced auto-restart cooldown.
- `NodeHealthService` (`node_health_service.rs`) — Heartbeat-based health evaluation pattern. Use similar configurable-interval pattern for sleep timeout.
- `Server.auto_restart` — Already wired through create/update use cases and DTOs. Follow same pattern for `auto_wake`.
- `Server.auto_pause` — Field exists on entity and DTOs. Keep as-is, not repurposed.

### Established Patterns
- Background services spawned in `AppContainer` bootstrap with tokio::spawn
- 30-second monitoring interval with immediate first tick
- Status field as plain String (not enum) — new values don't need enum expansion
- EventBus for status change events (published in monitoring_service)

### Integration Points
- Monitoring loop in `monitoring_service.rs:53-69` — add sleep detection check after existing crash detection
- Server entity `server.rs` — add `auto_wake: bool` field
- Server DTOs in `server_dtos.rs` — add auto_wake to CreateServerRequest and UpdateServerRequest
- Server create/update use cases — wire auto_wake persistence
- Frontend server detail / settings UI — add sleep/wake configuration controls

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The existing crash→restart pattern in monitoring_service should be extended rather than replaced.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 56-auto-online-sleep-recovery*
*Context gathered: 2026-05-30*
