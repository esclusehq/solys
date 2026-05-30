# Phase 57: Auto Restart Policies - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-30
**Phase:** 57-auto-restart-policies
**Areas discussed:** Restart Policy UI Placement, Default Policies vs Per-Server Override, Restart History & Visibility, Crash Detection Enhancement, Restart Notifications

---

## Restart Policy UI Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Server Settings tab (alongside Sleep & Wake) | Add 'Restart Policy' section to existing Settings tab near Phase 56's Sleep & Wake config | ✓ |
| New 'Restart' tab in ServerDetails | Dedicated tab alongside Overview, Settings, Backups | |

**User's choice:** Server Settings tab (alongside Sleep & Wake)
**Notes:** Consistent UX — user configures all automation in one place.

---

## Default Policies vs Per-Server Override

| Option | Description | Selected |
|--------|-------------|----------|
| Per-server only, no global defaults | Each server has own config, hardcoded defaults | |
| Global defaults + per-server override | Settings page for global values, per-server overrides | ✓ |

**User's choice:** Global defaults + per-server override
**Notes:** Phase 56's hardcoded-only approach won't scale. Need platform-level defaults with per-server overrides.

---

## Restart History & Visibility

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal — only current restart count | Badge on server card | |
| Detailed — count + last restart time + failure reason | Compact info section in Settings tab | ✓ |
| Full history — timeline of all restart events | Needs new storage | |

**User's choice:** Detailed — count + last restart time + failure reason
**Notes:** Add `last_restart_at` and `last_restart_reason` columns to servers table.

---

## Crash Detection Enhancement

| Option | Description | Selected |
|--------|-------------|----------|
| No — container status is sufficient | Current detection is enough | |
| Yes — basic health check probe | RCON ping for unresponsive detection | ✓ |
| Yes — but deferred to future phase | Keep health check for later | |

**User's choice:** Yes — basic health check probe
**Notes:** Add RCON health check in monitoring loop metrics collection. Configurable timeout.

---

## Restart Notifications

| Option | Description | Selected |
|--------|-------------|----------|
| Status badge only | Passive indication | |
| Toast notification on dashboard | Brief alert | |
| Toast + event log entry | Toast in UI + event timeline | ✓ |

**User's choice:** Toast + event log entry
**Notes:** Emit server.restarted and server.restart_limit_reached events.

---

## the agent's Discretion

- Specific UI layout of Restart Policy section
- Global defaults settings page design
- Default values for global defaults
- `last_restart_reason` enum values
- `health_check_timeout_seconds` default value
- MonitoringService modification details
- Toast styling and persistence duration

## Deferred Ideas

None — discussion stayed within phase scope.
