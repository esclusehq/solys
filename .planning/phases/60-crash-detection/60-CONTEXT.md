# Phase 60: Crash Detection - Context

**Gathered:** 2026-05-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver intelligent crash diagnosis for game servers — detect WHEN a server crashes, classify WHY (OOM, config error, plugin crash, etc.), and trigger the appropriate recovery action per crash type.

Builds on Phase 57 (Auto Restart Policies), which handles reactive restart policy (max attempts, cooldown, RCON health check). Phase 60 adds crash forensics and smart recovery decision-making on top of Phase 57's existing detection infrastructure.

Phase 57's components (MonitoringService crash detection loop, `auto_restart`, `max_restart_attempts`, `restart_cooldown_seconds`, `last_restart_at`, `last_restart_reason`) are all already built and serve as the foundation.

</domain>

<decisions>
## Implementation Decisions

### D-01: Crash Detection Signal — Agent Crash Reporter
**Decision:** The Web Agent reports crashes to the backend via a structured WebSocket message containing:
- Container exit code
- Last N lines of server stdout/stderr at crash time

This data flows from agent to backend as a WebSocket message (fits existing agent → backend WebSocket protocol). The crash reporter logic lives inside the `web-agent` binary (no new agent-core crate).

Rationale: Container exit codes alone (137=OOM, 139=SIGSEGV, 0=clean, 1=error) are insufficient to distinguish crash types. Log context is needed for game-level crash diagnosis. WebSocket delivery keeps the agent's HTTP surface minimal.

### D-02: Crash Classification Location — Backend MonitoringService
**Decision:** The Agent sends RAW crash data (exit code + log excerpt) to the backend. The backend MonitoringService classifies the crash type and decides recovery action. The agent is a reporter, not a decision-maker.

Rationale: Classification logic can evolve without agent updates. Backend has access to full server context (plan, resource allocation, history). Keeps agent simple.

### D-03: Recovery Strategy per Crash Type

| Crash Type | Detection Signal | Recovery Action |
|---|---|---|
| **OOM** (exit 137 + memory-related log patterns) | Exit code 137 or log patterns like "OutOfMemoryError", "Killed", "java.lang.OutOfMemoryError" | **Notify only** — do NOT auto-restart. Restarting without more RAM will crash again. User must allocate more resources. |
| **Config error** (crashes within 60s of startup) | Crash-loop detection: 3 rapid crashes within 60s of startup | **Disable auto-restart** after 3 rapid restarts. Alert user with log excerpt. |
| **Plugin/mod crash** (exception in plugin/mod code) | Crash with plugin exception in logs (e.g., NullPointerException, plugin error stack traces) | **Restart + log reason** — follow Phase 57's existing restart policy (max_attempts, cooldown). Crashed plugin won't always re-crash. |
| **Generic crash** (unexpected container exit) | Any other container exit (exit code != 0, 137) where no specific pattern is detected | Follow Phase 57's existing auto-restart policy. |

### D-04: Crash Notification — Every Crash, Multi-Channel
**Decision:** Notify the user on EVERY crash detection, regardless of recovery outcome:
- **Toast notification** in the web app (follows Phase 57 D-05 pattern)
- **Server event timeline** entry (e.g., server.crash_detected)
- **Discord webhook** notification (Discord webhook URL is already a per-server config field)

Notification content is brief: server name + crash type only (e.g., "Server MyMC crashed — OOM detected"). Full details go in the crash history panel.

### D-05: Crash History UI — Settings Tab Section
**Decision:** Crash history appears in the ServerDetails **Settings tab** as a "Crash History" section, alongside Sleep/Wake (Phase 56), Restart Policy (Phase 57), and Scheduled Actions (Phase 59). Consistent placement for all server automation config.

Each crash entry shows:
- Timestamp of crash
- Crash type (OOM / Config / Plugin / Generic)
- Exit code
- Log excerpt (last line at crash time)
- Recovery action taken (auto-restarted / notified-only / restart-disabled)

### D-06: Crash Storage — New server_crash_logs Table
**Decision:** Dedicated `server_crash_logs` table for crash forensic data:
- `server_id` (UUID, FK to servers)
- `crashed_at` (TIMESTAMPTZ)
- `exit_code` (INTEGER)
- `crash_type` (VARCHAR — OOM / config_error / plugin_crash / generic)
- `log_excerpt` (TEXT — last 1-5 lines)
- `recovery_action` (VARCHAR — auto_restarted / notified_only / restart_disabled)
- `resolved_at` (TIMESTAMPTZ, nullable — marks when user acknowledged/resolved)

Retention: **Unlimited** — user can manually clear. UI shows paginated list, newest first.

### The Agent's Discretion
- Exact log line capture strategy (how many lines, encoding limits)
- Crash type detection patterns in MonitoringService (regex patterns for OOM, plugin exceptions, config errors)
- Discord webhook message format for crash notifications
- Specific UI layout of Crash History section in Settings tab
- Toast notification design and auto-dismiss duration
- WebSocket message format for crash report from agent to backend
- 60s crash-loop detection window exact value

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 57 Auto Restart Policies (Foundation)
- `.planning/phases/57-auto-restart-policies/57-CONTEXT.md` — Auto-restart decisions, RCON health check, restart history, notification patterns
- `.planning/phases/57-auto-restart-policies/57-RESEARCH.md` — Architecture patterns, health check pitfalls
- `.planning/phases/57-auto-restart-policies/57-PATTERNS.md` — File-level analogs

### Existing Monitoring Infrastructure
- `api/src/application/services/monitoring_service.rs` — MonitoringService with 30s crash/health/sleep detection loop (classification logic will be added here)
- `api/src/domain/entities/server.rs` — Server entity with auto_restart, max_restart_attempts, restart_cooldown_seconds, restart_count fields (Phase 57)
- `api/src/presentation/handlers/server_handlers.rs` — REST handlers (update_server for crash history clear, etc.)

### Agent Infrastructure
- `web-agent/src/main.rs` — Web Agent binary (crash reporter logic goes here)
- `agent-core/crates/agent-proto/src/lib.rs` — Task/result protocol definitions (extend for crash report WebSocket message)
- `agent-core/crates/agent-task/src/dispatcher.rs` — Task dispatch for agent-side operations

### Frontend Patterns
- `app/src/pages/ServerDetails.jsx` — Server details page with Settings tab (add Crash History section)
- `app/src/hooks/useServers.js` — Server API hooks (fetch crash history, clear crash history)
- `app/src/api/client.js` — API client with fetch pattern

### Phase 56 & 57 Settings Tab Patterns
- `.planning/phases/56-auto-online-sleep-recovery/56-CONTEXT.md` — Sleep/Wake config section layout
- `.planning/phases/57-auto-restart-policies/57-CONTEXT.md` — Restart Policy section layout, D-01/D-03

### Codebase Maps
- `.planning/codebase/STACK.md` — Tech stack (Rust Axum, React 19, Zustand)
- `.planning/codebase/ARCHITECTURE.md` — Service layer, WebSocket, Agent communication
- `.planning/codebase/INTEGRATIONS.md` — External integrations (Discord webhooks, PostgreSQL, Redis, WebSocket)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `monitoring_service.rs` — Existing 30s inspection loop. Natural injection point for crash classification logic after crash detection.
- `agent-proto/src/lib.rs` — WebSocket protocol definitions. Add `CrashReport` message type.
- `web-agent` — Already manages containers via bollard, knows when they stop. Crash reporter hooks in here.
- `emit_server_event()` — Server event emission pattern. Use for `server.crash_detected`, `server.crash_recovered`, `server.crash_escalated`.
- `Server.auto_restart` / `max_restart_attempts` / `restart_cooldown_seconds` — Already wired through Phase 57.
- Phase 57's `last_restart_at` / `last_restart_reason` fields — Complement crash history data.
- Discord webhook URL per server — Already configured for notification delivery.
- Toast notification + event timeline pattern — Phase 57 D-05. Reuse for crash notifications.

### Established Patterns
- Phase 56/57/59 Settings tab section layout (toggle + inputs + save + toast) — Use for Crash History section
- MonitoringService continuous evaluation pattern (30s loop, skip offline nodes, check conditions)
- WebSocket message protocol between Agent and Backend (agent-proto messages)
- Per-server conditional updates pattern (`if let Some(field) = req.field`)
- Event emission via `emit_server_event()` for lifecycle events
- Toast + event timeline + Discord webhook for notifications

### Integration Points
- **Web Agent (web-agent)**: Add crash reporter that captures exit code + log lines on container stop, sends as WebSocket `CrashReport` message.
- **Agent Proto (agent-core)**: Add `CrashReport` message type to WebSocket protocol.
- **MonitoringService (api)**: Add crash classification logic — receive crash report, classify crash type, execute recovery action per D-03.
- **New DB migration**: Create `server_crash_logs` table (D-06).
- **New API endpoints**: `GET /api/v1/servers/{id}/crash-logs`, `DELETE /api/v1/servers/{id}/crash-logs`.
- **Server entity/repository**: Add read/write for `server_crash_logs`.
- **New API handler**: Crash report ingestion endpoint (or WebSocket handler for CrashReport message).
- **Frontend ServerDetails.jsx Settings tab**: Add "Crash History" section with paginated crash list.
- **Discord notification**: Send crash notification via existing Discord webhook per server.

### Migration Plan
1. `CREATE TABLE server_crash_logs (...)`
2. Add crash report WebSocket message type to agent-proto
3. Add crash reporter to web-agent
4. Add crash report ingestion + classification + recovery in MonitoringService
5. Add API endpoints for crash log read/clear
6. Add Crash History UI section in ServerDetails Settings tab
7. Wire Discord webhook notification for crashes

</code_context>

<specifics>
## Specific Ideas

The agent reports raw data. The backend classifies. Recovery per crash type:
- OOM → notify only, no restart
- Config error → disable restart after 3 rapid crashes
- Plugin crash → restart + log, follow Phase 57 policy
- Every crash notifies user via toast + event + Discord webhook
- Crash history stored in dedicated table, visible in Settings tab

</specifics>

<deferred>
## Deferred Ideas

- **Crash alert escalation** (e.g., SMS/pager if server stays down for X minutes) — future phase
- **Automated crash fix suggestions** (e.g., "increase RAM by 2GB" for OOM) — post-MVP enhancement

</deferred>

---

*Phase: 60-crash-detection*
*Context gathered: 2026-05-31*
