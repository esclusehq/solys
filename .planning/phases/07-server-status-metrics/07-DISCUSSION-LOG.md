# Phase 7: Server Status & Metrics - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 7-Server Status & Metrics
**Areas discussed:** Status polling, Metrics collection, Metrics display, Alert thresholds

---

## Status Polling

| Option | Description | Selected |
|--------|-------------|----------|
| Agent pull + Redis cache | Pull from agent every 10s, cache in Redis, stale after 30s | ✓ |
| Direct polling | Poll podman/docker every 5s directly | |
| Agent push only | Agent pushes updates via WebSocket on state change | |

**User's choice:** Agent pull + Redis cache (Recommended)
**Notes:** Reduces load on agent, cached status is fast.

---

## Metrics Collection

| Option | Description | Selected |
|--------|-------------|----------|
| 30s interval + 24h retention | Collect every 30s, keep 24h in DB, aggregate hourly/daily | ✓ |
| 60s + 7 days | Collect every 60s, keep 7 days | |
| On-demand only | Collect on demand, no historical data | |

**User's choice:** 30s interval + 24h retention (Recommended)
**Notes:** Good balance between granularity and storage.

---

## Metrics Display

| Option | Description | Selected |
|--------|-------------|----------|
| Current + sparkline | Current values + last 24h sparkline, detailed history on click | ✓ |
| Full history charts | Full historical charts with zoom, time range selection | |
| Current values only | Just current values, no historical visualization | |

**User's choice:** Current + sparkline (Recommended)
**Notes:** Clean UI, detailed on demand.

---

## Alert Thresholds

| Option | Description | Selected |
|--------|-------------|----------|
| Threshold-based | CPU > 90%, RAM > 85%, Disk > 80% for 5min - notify via email/Discord | ✓ |
| ML-based detection | Any anomaly detection, ML-based | |
| Manual only | No automatic alerts, user can query manually | |

**User's choice:** Threshold-based (Recommended)
**Notes:** Simple, predictable, actionable.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
