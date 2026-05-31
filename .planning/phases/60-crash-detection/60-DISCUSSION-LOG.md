# Phase 60: Crash Detection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-31
**Phase:** 60-crash-detection
**Areas discussed:** Crash Classification, Recovery Strategy per Crash Type, Crash Notifications, Crash History UI

---

## Crash Classification

| Option | Description | Selected |
|--------|-------------|----------|
| Container Exit Code | Container runtime reports exit code (137=OOM, 139=SIGSEGV). Simple but limited | |
| Log Scanning | Parse server stdout/stderr for crash patterns. Fragile across game types | |
| Agent Crash Reporter | Agent sends exit code + last N log lines via WebSocket. Most informative | ✓ |

**User's choice:** Agent Crash Reporter
**Notes:** Selected for richest signal. Agent reports raw data, backend classifies.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Exit Code + Last Log Lines | Exit code + last 50 lines of server output. Covers most crash types | ✓ |
| Full Snapshot | Exit code + logs + resource usage + container state. Heavier payload | |

**User's choice:** Exit Code + Last Log Lines
**Notes:** Keeps payload manageable while providing enough context for classification.

---

| Option | Description | Selected |
|--------|-------------|----------|
| WebSocket Message | Agent sends crash report as structured WS message. Real-time, fits existing protocol | ✓ |
| HTTP POST to API | Agent POSTs to /api/v1/servers/{id}/crash-report. Simpler WS protocol | |

**User's choice:** WebSocket Message
**Notes:** Fits existing agent → backend WebSocket communication pattern.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Inside web-agent | Add crash detection logic directly to web-agent binary. Simpler | ✓ |
| New agent-crash crate | New crate in agent-core workspace. Cleaner separation | |

**User's choice:** Inside web-agent
**Notes:** Keeps implementation simpler. No need for separate crate at this point.

---

## Recovery Strategy per Crash Type

| Option | Description | Selected |
|--------|-------------|----------|
| OOM: Notify only — do not auto-restart | Log OOM, notify user, disable auto-restart. Restarting without more RAM will crash again | ✓ |
| OOM: Auto-restart + notify | Restart but flag as OOM. Risk of repeated crashes | |

**User's choice:** Notify only — do not auto-restart
**Notes:** OOM crash = permanent failure until resources increase. No point restarting.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Config error: Detect via log scan + disable restart | Agent detects config error in logs, captures excerpt, disables restart | |
| Config error: Crash-loop detection — disable after 3 rapid restarts | If server crashes within 60s of startup 3 times in a row, disable restart | ✓ |

**User's choice:** Crash-loop detection — disable after 3 rapid restarts within 60s
**Notes:** Crash-loop detection is more robust than log pattern matching for config errors.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Plugin crash: Restart + log + existing restart policy | Restart server, log exception, let Phase 57 handle repeated failures | ✓ |
| Plugin crash: Restart once, disable plugin if re-crashes | Try to identify failing plugin from logs and suggest disabling | |

**User's choice:** Restart + log reason + follow Phase 57 restart policy
**Notes:** Plugin crashes are often transient. No need for special handling beyond Phase 57's existing policy.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Classification in backend MonitoringService | Agent sends raw data, backend classifies and decides recovery | ✓ |
| Classification in Agent | Agent classifies locally, backend acts on classification | |

**User's choice:** Backend MonitoringService
**Notes:** Backend has full context. Classification logic can evolve without agent updates.

---

## Crash Notifications

| Option | Description | Selected |
|--------|-------------|----------|
| On every crash detected | Notification for every detection — even if auto-restart succeeds | ✓ |
| Only when auto-restart fails or non-restartable | Silent recovery if restart succeeds | |

**User's choice:** On every crash detected
**Notes:** User always knows their server went down, regardless of recovery outcome.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Toast + Server Event Timeline | In-app toast + event timeline entry. Consistent with Phase 57 | |
| Toast + Event Timeline + Discord Webhook | Same plus Discord webhook (already configured per server) | ✓ |

**User's choice:** Toast + Event Timeline + Discord Webhook
**Notes:** Full coverage — in-app and external notification.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Server name + crash type only | Brief notification, points to crash history for details | ✓ |
| Server name + crash type + exit code + log excerpt | Self-contained, user knows immediately | |

**User's choice:** Server name + crash type only
**Notes:** Keep notification concise. Full details in crash history panel.

---

## Crash History UI

| Option | Description | Selected |
|--------|-------------|----------|
| Section in ServerDetails Settings tab | Alongside Sleep/Wake, Restart Policy, Scheduled Actions. Consistent placement | ✓ |
| Dedicated tab in ServerDetails | Separate tab. More space for detailed forensics | |

**User's choice:** Section in ServerDetails Settings tab
**Notes:** Follows Phase 56/57/59 pattern — all server automation in one place.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: timestamp + crash type + exit code | Simple table for quick scan | |
| Rich: timestamp + type + exit code + log excerpt + recovery action | Plus 5-line log excerpt and recovery action taken | ✓ |

**User's choice:** Rich display with full details
**Notes:** More informative for debugging. Each entry shows what happened and what was done.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Last 20 crashes (bounded) | Keep 20 most recent. Bounded storage | |
| 30-day rolling window | Keep all crashes for 30 days, auto-prune | |
| Unlimited — let user clear manually | Keep all crash data indefinitely, manual clear with pagination | ✓ |

**User's choice:** Unlimited — let user clear manually
**Notes:** Gives full visibility. User controls cleanup.

---

| Option | Description | Selected |
|--------|-------------|----------|
| New server_crash_logs table | Dedicated table for crash forensic data. Clean separation | ✓ |
| Extend server_events with crash types | Add crash event types to existing server_events. Simpler but mixed concerns | |

**User's choice:** New server_crash_logs table
**Notes:** Clean separation from general server events. Dedicated schema for crash forensics.

---

## The Agent's Discretion

- Exact log line capture strategy (how many lines, encoding limits)
- Crash type detection patterns in MonitoringService (regex patterns for OOM, plugin exceptions, config errors)
- Discord webhook message format for crash notifications
- Specific UI layout of Crash History section in Settings tab
- Toast notification design and auto-dismiss duration
- WebSocket message format for crash report from agent to backend
- 60s crash-loop detection window exact value

## Deferred Ideas

- **Crash alert escalation** (e.g., SMS/pager if server stays down for X minutes) — future phase
- **Automated crash fix suggestions** (e.g., "increase RAM by 2GB" for OOM) — post-MVP enhancement
