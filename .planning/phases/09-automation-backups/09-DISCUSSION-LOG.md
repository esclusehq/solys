# Phase 9: Automation & Backups - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 9-Automation & Backups
**Areas discussed:** Backup scheduling, Retention policies, Crash recovery, Backup storage

---

## Backup Scheduling

| Option | Description | Selected |
|--------|-------------|----------|
| Cron expression | Cron expression per server, scheduler service runs every minute, UTC timezone | ✓ |
| Preset intervals | Preset intervals: hourly, daily, weekly | |
| UI-based schedule | User selects days/times from dropdown | |

**User's choice:** Cron expression (Recommended)
**Notes:** Flexible, industry standard.

---

## Retention Policies

| Option | Description | Selected |
|--------|-------------|----------|
| Count-based | Keep N most recent backups per server, delete older ones | ✓ |
| Age-based | Keep backups older than X days | |
| Combined approach | Both: keep N recent AND delete older than X days | |

**User's choice:** Count-based (Recommended)
**Notes:** Simple, predictable storage usage.

---

## Crash Recovery

| Option | Description | Selected |
|--------|-------------|----------|
| Container auto-restart | Container auto-restart policy, health check, restart count tracking | ✓ |
| External monitor | Monitor service that detects down servers and restarts them | |
| Manual only | Manual intervention required, no auto-restart | |

**User's choice:** Container auto-restart (Recommended)
**Notes:** Leverages container orchestrator capabilities.

---

## Backup Storage

| Option | Description | Selected |
|--------|-------------|----------|
| Local + S3 | Local disk + S3, configurable per server | ✓ |
| Local only | Local disk only, no cloud backup | |
| S3 only | S3 only, no local storage | |

**User's choice:** Local + S3 (Recommended)
**Notes:** Best of both worlds.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
