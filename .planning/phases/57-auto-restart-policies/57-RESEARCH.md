# Phase 57: Auto Restart Policies - Research

**Researched:** 2026-05-30
**Domain:** Server restart automation (configurable policies, health checking, restart history)
**Confidence:** HIGH

## Summary

This phase delivers configurable auto-restart policies for game servers, building on Phase 56's infrastructure (`max_restart_attempts`, `restart_cooldown_seconds`, `restart_count`, exponential backoff in MonitoringService). Five capabilities are needed: (1) **Restart Policy UI** in the Server Settings tab alongside Sleep & Wake, (2) **Global defaults + per-server override** for `max_restart_attempts` and `restart_cooldown_seconds`, (3) **Restart history tracking** with `last_restart_at` and `last_restart_reason` columns, (4) **RCON health check probe** in the MonitoringService metrics loop for unresponsive detection, and (5) **Restart notifications** via toast + server event log.

The data model changes add `last_restart_at` (TIMESTAMPTZ), `last_restart_reason` (TEXT), and `health_check_timeout_seconds` (INTEGER) to the servers table. A new `server_settings` table or column on the existing settings structure is needed for global default values.

**Primary recommendation:** Extend MonitoringService by injecting RCON health check between metrics collection and sleep detection. Add a new `RestartPolicyService` or extend the existing monitoring logic for configurable policies. Follow Phase 56's exact wiring pattern for per-server fields through DTOs, use cases, handlers, and the Settings tab UI section.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Restart policy configuration lives in the **Server Settings tab** (ServerDetails), as a "Restart Policy" section alongside Phase 56's "Sleep & Wake" config section. Consistent UX — user configures all server automation in one place.
- **D-02:** **Global defaults + per-server override.** Settings page has global default values for `max_restart_attempts` and `restart_cooldown_seconds`. Per-server values in Settings tab override global defaults. New servers inherit global defaults.
- **D-03:** **Detailed display: restart count + last restart time + failure reason.** Add `last_restart_at` (TIMESTAMPTZ) and `last_restart_reason` (TEXT) columns to servers table. Show in a compact Restart History section within Settings tab.
- **D-04:** **Add basic health check probe via RCON.** In MonitoringService metrics collection step, attempt RCON ping for running servers. If unresponsive within configurable timeout, mark as `unresponsive` and trigger restart. Configurable via `health_check_timeout_seconds`.
- **D-05:** **Toast notification + event log entry.** Show toast in UI when restart happens (or max attempts reached). Log restart events (server.restarted, server.restart_limit_reached) in server event timeline.

### The Agent's Discretion
- Specific UI layout of Restart Policy section in Settings tab (form fields, toggle design)
- Global defaults settings page design (existing settings page or new section)
- Default values for global defaults (hardcoded fallbacks: max_attempts=5, cooldown=300s)
- `last_restart_reason` enum values and when each is used
- `health_check_timeout_seconds` default value
- MonitoringService modification details (where to inject RCON health check)
- Toast styling and persistence duration
- Event emission naming convention for restart events
- Global defaults storage mechanism (new DB table vs config file vs existing settings)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| RCON health check probe | API / Backend | — | Runs in MonitoringService 30s tick, pings RCON for running servers |
| Restart policy per-server config | API / Backend | Browser / Client | API CRUD for server restart fields; frontend provides UI in Settings tab |
| Global defaults management | API / Backend | Browser / Client | Settings page for platform-wide restart defaults |
| Restart history display | Browser / Client | — | Read-only display of last_restart_at + last_restart_reason in Settings tab |
| Restart notifications | API / Backend | Browser / Client | Server emits event → frontend shows toast via WebSocket or polling |
| Crash detection enhancement | API / Backend | — | Add unresponsive state detection in MonitoringService alongside existing crash detection |

## Standard Stack

### Core — No new dependencies needed

All work uses the existing stack. No new Rust crates or npm packages required.

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio::time | v1 | Async interval timer | MonitoringService already uses it |
| rcon | v0.6 | RCON health check ping | Already integrated in `rcon_server_executor.rs::collect_metrics` |
| sqlx | v0.7 | DB migration + queries | Existing migration pattern for new columns |
| chrono | v0.4 | Time tracking for cooldown, health check | Already used throughout codebase |
| react | v19.2.4 | Frontend UI (Settings tab section) | Existing framework |
| zustand | v5 | State management for restart config | Already in use |
| tailwindcss | v4 | UI styling | Existing utility framework |

## Detailed Research

### 1. RCON Health Check Integration

Phase 56 already has a `collect_metrics()` call in the monitoring loop that pings RCON for player count. The health check probe can reuse the same RCON connection mechanism:

```
MonitoringService 30s loop:
  1. Fetch all running servers (existing)
  2. Skip offline nodes (existing)
  3. For each running server:
     a. executor.collect_metrics() → player count, resource usage (existing)
     b. NEW: RCON ping health check → if fails within timeout → mark unresponsive
     c. Crash detection → check container status (existing, lines 143-189)
     d. Sleep detection → check player inactivity (Phase 56)
     e. Auto-restart backoff → check restart_count vs max_attempts (existing)
```

Key considerations:
- RCON health check should have its own timeout (`health_check_timeout_seconds` defaulting to 5s)
- A failed RCON ping doesn't necessarily mean the server is dead — it could be a temporary network blip. Consider requiring 2 consecutive failures before marking `unresponsive`.
- The `last_restart_reason` should distinguish between `crash_detected` (container exited) and `unresponsive` (RCON not responding but container running).
- Follow the existing `if let Some(field) = req.field` pattern for conditional updates.

### 2. Global Defaults Storage

Two approaches for storing global default restart policies:

**Option A: Platform settings table** (RECOMMENDED)
- Add `max_restart_attempts_default` and `restart_cooldown_seconds_default` to an existing platform settings store or create a new `platform_settings` table.
- Simpler to implement, fewer DB changes.
- Follows pattern of other platform-level settings.

**Option B: Config file / environment variables**
- Harder for users to change, less discoverable.
- Not recommended for a user-facing feature.

**Recommendation:** Option A — use an existing settings mechanism or a lightweight KV store in the database.

### 3. Restart History Display

The restart history section in the Settings tab needs:
- Current restart count (from `restart_count` on server entity, already exists from Phase 56)
- Last restart time (`last_restart_at` — new column)
- Last restart reason (`last_restart_reason` — new column, values: `crash_detected`, `unresponsive`, `max_attempts_reached`)
- Compact display following the same visual pattern as Phase 56's Sleep & Wake config

### 4. Backend Wiring Pattern

Follow Phase 56's exact pattern for each new field:
1. **Migration:** Add columns to servers table via sqlx migration
2. **Entity:** Add fields to Server entity (both models if dual-model pattern)
3. **Repository:** Add read/write in PostgresServerRepository + sqlx repository
4. **DTOs:** Add fields to UpdateServerRequest and ServerResponse
5. **Use case:** Add conditional blocks in update_server_use_case.rs
6. **Handlers:** Wire fields in update_server handler
7. **Frontend:** Add form fields in Settings tab section

### 5. MonitoringService Integration Points

The monitoring_service.rs file structure (from Phase 56 context):
- Lines 143-189: Crash detection logic
- Lines 189-207: Between crash detection and metrics
- Lines 207-234: Metrics collection

The RCON health check should be injected AFTER metrics collection but BEFORE sleep detection. This ensures:
1. Player count is collected first (for sleep detection)
2. RCON health check runs (for unresponsive detection)
3. Sleep detection runs (for auto-sleep)
4. Auto-restart backoff evaluates (for restart decisions)

### 6. Event Emission Pattern

Phase 56 already has `emit_server_event()` for server lifecycle events. New event types:
- `server.restarted` — Emitted when server is successfully restarted
- `server.restart_limit_reached` — Emitted when max restart attempts reached without success

Both should carry:
- server_id, reason, timestamp
- For `server.restart_limit_reached`: also attempt_count, max_attempts

### 7. Frontend Pattern

Follow Phase 56's Sleep & Wake section exactly:
- Section title: "Restart Policy"
- Toggle for enable/disable auto-restart (uses existing `auto_restart` field)
- Number inputs for `max_restart_attempts` and `restart_cooldown_seconds`
- Compact restart history display (read-only)
- Save button
- Toast on success

Phase 56 reference file: `app/src/pages/ServerDetails.jsx` — Sleep & Wake section pattern.
