# Phase 40: BACKEND ↔ AGENT STABILITY - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-03
**Phase:** 40-backend-agent-stability
**Areas discussed:** Heartbeat system, Node status, Offline handling, Reconnect logic

---

## Heartbeat System

| Option | Description | Selected |
|--------|-------------|----------|
| 10 seconds | Balance between responsiveness and network overhead | |
| 5 seconds | More responsive, more network overhead | |
| 30 seconds | Less overhead, slower to detect issues | |
| Configurable | Allow per-node heartbeat interval setting | ✓ |

**User's choice:** Configurable
**Notes:** Heartbeat interval is configurable per-node

| Option | Description | Selected |
|--------|-------------|----------|
| CPU, RAM, Disk, Uptime | Full system stats as specified | ✓ |
| CPU + RAM only | Minimal for faster processing | |
| Connection status only | Just confirm alive, no metrics | |
| Full metrics | Comprehensive but higher bandwidth | |

**User's choice:** CPU, RAM, Disk, Uptime (recommended)
**Notes:** Heartbeat payload decided

---

## Node Status

| Option | Description | Selected |
|--------|-------------|----------|
| No heartbeat for 3x interval | 3 missed heartbeats = offline | ✓ |
| 1 missed heartbeat | Immediate offline detection | |
| No heartbeat for 60 seconds | 1 minute grace period | |
| WebSocket disconnect only | Only when WebSocket closes | |

**User's choice:** No heartbeat for 3x interval (recommended)
**Notes:** OFFLINE trigger decided

| Option | Description | Selected |
|--------|-------------|----------|
| Heartbeat late or metrics stale | Late heartbeats or stale metrics = degraded | |
| High CPU/RAM usage | Resource exhaustion warning | |
| WebSocket reconnection attempts | Currently reconnecting = degraded | |
| No degraded state | Binary online/offline only | |

**User's choice:** 5. Heartbeat late (>50% interval) OR metrics stale OR high CPU/RAM threshold OR reconnecting attempts
**Notes:** DEGRADED trigger detailed

---

## Offline Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Stop monitoring servers on that node | No point polling unreachable nodes | ✓ |
| Continue monitoring with reduced frequency | Keep trying but less often | |
| Mark servers as unknown, keep polling | Keep trying for status updates | |

**User's choice:** Stop monitoring servers on that node (recommended)
**Notes:** Offline action decided

---

## Reconnect Logic

| Option | Description | Selected |
|--------|-------------|----------|
| Agent initiates | Agent auto-reconnects with backoff | ✓ |
| Backend polls agent | Backend periodically tries to connect | |
| Both | Agent + backend as backup | |

**User's choice:** Agent initiates (recommended)
**Notes:** Reconnection approach decided

| Option | Description | Selected |
|--------|-------------|----------|
| Sync state + resume monitoring | Get latest server list, resume monitoring | ✓ |
| Full re-registration | Treat as new connection | |
| Just resume monitoring | Continue without state sync | |

**User's choice:** Sync state + resume monitoring
**Notes:** What happens on reconnect

---

## Deferred Ideas

No deferred ideas — all discussion stayed within phase scope.