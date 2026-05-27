# Phase 10: Monitoring & Integrations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 10-Monitoring & Integrations
**Areas discussed:** Historical resource graphs, Discord notifications, Cron task scheduling, Notification events

---

## Historical Resource Graphs

| Option | Description | Selected |
|--------|-------------|----------|
| Recharts | Recharts line charts in React, store 24h data points | ✓ |
| Chart.js | Chart.js for simple visualizations | |
| Current values only | No historical charts, just current values | |

**User's choice:** Recharts (Recommended)
**Notes:** Rich visualizations, good React integration.

---

## Discord Notifications

| Option | Description | Selected |
|--------|-------------|----------|
| Per-server webhooks | Store per-server, use DiscordClient for sending, embed format | ✓ |
| Global webhook | One global Discord webhook for all notifications | |
| None | No Discord integration yet | |

**User's choice:** Per-server webhooks (Recommended)
**Notes:** Allows per-server notification settings.

---

## Cron Task Scheduling

| Option | Description | Selected |
|--------|-------------|----------|
| UI-based scheduling | UI time picker, stored in DB, scheduler service runs tasks | ✓ |
| Cron expression input | Cron expression input field similar to backups | |
| None | No custom cron tasks, only backup scheduling | |

**User's choice:** UI-based scheduling (Recommended)
**Notes:** User-friendly, no cron syntax knowledge needed.

---

## Notification Events

| Option | Description | Selected |
|--------|-------------|----------|
| Event-based | start, stop, crash, backup complete, restore complete | ✓ |
| Full event system | All server events, including status changes, metrics thresholds | |
| Manual trigger only | No automatic notifications | |

**User's choice:** Event-based (Recommended)
**Notes:** Key events only, not overwhelming.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
