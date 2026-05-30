# Phase 57: Auto Restart Policies - Context

**Gathered:** 2026-05-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver configurable auto-restart policies for game servers — let users enable/disable auto-restart per server, configure max restart attempts and cooldown periods, see restart history, and get notified of restart events.

Builds on Phase 56 infrastructure: `max_restart_attempts`, `restart_cooldown_seconds` fields on Server entity, exponential backoff in MonitoringService, restart_count tracking/reset.

</domain>

<decisions>
## Implementation Decisions

### Restart Policy UI Placement
- **D-01:** Restart policy configuration lives in the **Server Settings tab** (ServerDetails), as a "Restart Policy" section alongside Phase 56's "Sleep & Wake" config section. Consistent UX — user configures all server automation in one place.

### Default Policies vs Per-Server Override
- **D-02:** **Global defaults + per-server override.** Settings page (or platform settings section) has global default values for `max_restart_attempts` and `restart_cooldown_seconds`. Per-server values in the Settings tab override global defaults when set. New servers inherit global defaults initially.

### Restart History & Visibility
- **D-03:** **Detailed display: restart count + last restart time + failure reason.** Add `last_restart_at` (TIMESTAMPTZ) and `last_restart_reason` (TEXT — e.g., `crash_detected`, `unresponsive`) columns to servers table. Show in a compact Restart History section within the Settings tab.

### Crash Detection Enhancement
- **D-04:** **Add basic health check probe via RCON.** In the MonitoringService metrics collection step, attempt an RCON ping for running servers. If RCON doesn't respond within a configurable timeout, mark as `unresponsive` and trigger restart. Configurable health check timeout via `health_check_timeout_seconds` on the servers table.

### Restart Notifications
- **D-05:** **Toast notification + event log entry.** Show a toast in the UI when a restart happens (or max attempts reached). Log restart events (server.restarted, server.restart_limit_reached) in the server event timeline so users can review later.

### the agent's Discretion
- Specific UI layout of Restart Policy section in Settings tab (form fields, toggle design)
- Global defaults settings page design (existing settings page or new section)
- Default values for global defaults (hardcoded fallbacks: max_attempts=5, cooldown=300s if no global defaults set)
- `last_restart_reason` enum values and when each is used
- `health_check_timeout_seconds` default value
- MonitoringService modification details (where in the loop to inject RCON health check)
- Toast styling and persistence duration
- Event emission naming convention for restart events

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 56 Infrastructure (Already Built)
- `api/migrations/20260530000001_add_auto_wake.sql` — Adds auto_wake, max_restart_attempts, restart_cooldown_seconds, last_player_activity columns to servers table
- `api/src/domain/entities/server.rs` — Server entity with auto_restart, max_restart_attempts, restart_cooldown_seconds, restart_count fields
- `api/src/domain/server/model.rs` — New Server model with auto_wake, sleep_timeout_minutes
- `api/src/application/services/monitoring_service.rs` — Monitoring loop with crash detection, sleep detection, exponential restart backoff. Target for RCON health check injection
- `api/src/application/dto/server_dtos.rs` — Server DTOs with auto_restart field (follow pattern for restart policy fields)
- `api/src/application/use_cases/update_server_use_case.rs` — Conditional update pattern for server fields
- `api/src/presentation/handlers/server_handlers.rs` — REST handlers including update_server

### Existing Server Infrastructure
- `api/src/domain/entities/server.rs` — Server entity (source of truth for field additions)
- `api/src/infrastructure/repositories/postgres_server_repository.rs` — Old Server repository (INSERT/UPDATE/SELECT patterns)
- `api/src/domain/server/sqlx_repository.rs` — New Server repository with sqlx::FromRow
- `api/src/application/services/monitoring_service.rs` — 30s monitoring loop (crash detection at lines 143-189, metrics at 207-234, sleep detection after Phase 56)

### Frontend Patterns (Phase 56 UI pattern)
- `app/src/pages/ServerDetails.jsx` — Server details page with Settings tab (Phase 56's Sleep & Wake section)
- `app/src/components/StatusBadge.jsx` — Status badge component
- `app/src/hooks/useServers.js` — Server API hooks (updateServer pattern)
- `app/src/pages/servers/ServerManagerPage.jsx` — Server list with status dots

### Codebase Maps
- `.planning/codebase/STACK.md` — Technology stack (Rust Axum, React 19, Zustand, Tailwind CSS v4)
- `.planning/codebase/ARCHITECTURE.md` — Service layer architecture, monitoring service placement
- `.planning/codebase/INTEGRATIONS.md` — External integrations (PostgreSQL, Redis, WebSocket)

### Prior Phase Context
- `.planning/phases/56-auto-online-sleep-recovery/56-CONTEXT.md` — Sleep/wake decisions, auto-restart backoff context

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `monitoring_service.rs` — Existing 30s inspection loop with crash detection, sleep detection, auto-restart backoff. Natural injection point for RCON health check probe.
- `Server.auto_restart` — Already wired through create/update use cases and DTOs. Follow same pattern for restart policy fields.
- `PostgresServerRepository` — Has full INSERT/UPDATE/SELECT for all server fields. Add new columns here.
- Phase 56 Sleep & Wake section in ServerDetails.jsx Settings tab — Follow same UI pattern for Restart Policy section.
- `emit_server_event()` — Server event emission pattern (server.restarted, server.restart_limit_reached).
- `executor.collect_metrics()` — Already used in monitoring_service for player count. Can be extended to include RCON health check.

### Established Patterns
- Phase 56's Sleep & Wake config section layout (toggle, number input, save button, toast)
- Per-server conditional updates in update_server_use_case.rs (`if let Some(field) = req.field`)
- Event emission via `emit_server_event()` for server lifecycle events
- MonitoringService continuous evaluation pattern (skip offline nodes, check auto_wake/auto_restart before acting)

### Integration Points
- **MonitoringService**: Add RCON health check in the metrics collection block (after player count check), before sleep detection. Add `unresponsive` detection path.
- **Server entity**: Add `last_restart_at`, `last_restart_reason`, `health_check_timeout_seconds` fields to both Server models.
- **PostgresServerRepository + SqlxServerRepository**: Add read/write for new fields.
- **Server DTOs**: Add restart policy fields to UpdateServerRequest and ServerResponse.
- **Update use case**: Add conditional blocks for new fields.
- **Server handlers**: Wire restart policy fields in update_server handler.
- **Frontend Settings tab**: Add "Restart Policy" section after Sleep & Wake section.
- **Settings page**: Add global defaults for restart policies (if using platform settings).
- **Server event timeline**: Emit server.restarted and server.restart_limit_reached events.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Follow Phase 56's pattern for Settings tab UI section placement and design.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 57-auto-restart-policies*
*Context gathered: 2026-05-30*
