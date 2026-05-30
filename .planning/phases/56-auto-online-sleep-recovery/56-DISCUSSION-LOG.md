# Phase 56: Auto Online & Sleep Recovery - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-30
**Phase:** 56-auto-online-sleep-recovery
**Areas discussed:** Server State Model

---

## Server State Model

| Option | Description | Selected |
|--------|-------------|----------|
| Distinct 'sleeping' status | New status value visibly different from 'stopped' | |
| Status + auto-wake flag | Keep 'stopped' status, add auto_wake boolean | ✓ |

**User's choice:** Status + auto-wake flag
**Notes:** User prefers keeping the state machine simple. 'stopped' + auto_wake=true badge in UI.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Player inactivity timeout | No players for X min → auto-stop with auto_wake=true | |
| Manual sleep action | User clicks 'Sleep' in UI | |
| Both | Manual + inactivity, configurable timeout | ✓ |

**User's choice:** Both
**Notes:** Both triggers produce same state: stopped + auto_wake=true.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Via existing monitoring loop | MonitoringService 30s tick, check player count | ✓ |
| Agent-side tracking | Agent monitors locally, reports via WebSocket | |

**User's choice:** Via existing monitoring loop
**Notes:** No new infrastructure needed. MonitoringService already polls every 30s.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Map to sleep behavior | Rename auto_pause to auto_sleep | |
| Keep separate | Pause and sleep are different concepts | ✓ |

**User's choice:** Keep separate
**Notes:** auto_pause = freeze in memory (future). Sleep = stop + auto-recover.

---

## the agent's Discretion

- Specific inactivity timeout duration and configuration
- Wake-up trigger implementation details
- Auto-restart refinement (max attempts, cooldown, backoff)
- UI placement of sleep/wake configuration
- Migration design for auto_wake column

## Deferred Ideas

None.
