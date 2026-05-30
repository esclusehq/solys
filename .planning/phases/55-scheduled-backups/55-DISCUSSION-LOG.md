# Phase 55: Scheduled Backups - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-30
**Phase:** 55-scheduled-backups
**Areas discussed:** Scheduling Unification, Backup config UI placement, Backup execution model, Storage & retention policy

---

## Scheduling Unification

| Option | Description | Selected |
|--------|-------------|----------|
| Consolidate into one | Pick one canonical scheduler and migrate the other | ✓ (direction like option 3) |
| Keep separate with bridge | Both systems have different scopes, bridge them | |
| Use cron_tasks exclusively | Deprecate server-level backup_cron in favor of cron_tasks | |

**User's choice:** Consolidate into one — use cron_tasks as canonical scheduling system, deprecate server.backup_cron as source of truth
**Notes:** User chose consolidate but direction matches option 3 (use cron_tasks exclusively)

| Option | Description | Selected |
|--------|-------------|----------|
| In-process (API server) | Current pattern — backup_scheduler as tokio background task | |
| Worker service (Redis queue) | Cron evaluation lives in the Worker | ✓ |

**User's choice:** Worker service (Redis queue)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as shortcut for simple config | UI toggle writes to cron_tasks, columns stay for backward compat | ✓ |
| Deprecate completely | Remove columns from servers table | |
| Keep as read-only cache | Mirror active schedule for quick reads | |

**User's choice:** Keep as shortcut for simple config

| Option | Description | Selected |
|--------|-------------|----------|
| All task types | Worker evaluates cron for backup, restart, stop, and command | |
| Backup only for now | Focus cron evaluation on backup tasks first | ✓ |

**User's choice:** Backup only for now

---

## Backup Config UI Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Backup tab in Server Details | Config panel above existing backup history table | ✓ |
| Server settings tab | Backup config in Settings tab | |
| Dedicated Scheduled Tasks page | Split between two pages | |

**User's choice:** Backup tab in Server Details

| Option | Description | Selected |
|--------|-------------|----------|
| Preset dropdown with custom cron | Common presets + custom field | ✓ (simpler UX) |
| Simple interval picker | Pick interval in hours/days | |
| Visual cron builder | Day/time selectors | |

**User's choice:** Preset dropdown with custom cron — simpler UX than ScheduledTasksPage

| Option | Description | Selected |
|--------|-------------|----------|
| Max backup count only | Simple number input | |
| Count + time-based retention | Both max count AND max age | ✓ |

**User's choice:** Count + time-based retention

| Option | Description | Selected |
|--------|-------------|----------|
| Selectable per server | Dropdown to choose storage per server | ✓ |
| Display-only (global config) | Storage shown in badges only | |

**User's choice:** Selectable per server (local / S3 / R2 / MinIO)

---

## Backup Execution Model

| Option | Description | Selected |
|--------|-------------|----------|
| Agent-side (WebSocket command) | API sends backup.start to agent | |
| Keep API-side (current) | API creates archive via podman exec | |
| Worker dispatches to agent | Worker picks up backup_server job, sends to agent | ✓ |

**User's choice:** Worker dispatches to agent — Worker = orchestration, Agent = backup implementation

| Option | Description | Selected |
|--------|-------------|----------|
| Use existing agent-backup crate | Agent creates archive using own filesystem access | ✓ |
| Send podman exec command | API tells agent which podman exec to run | |

**User's choice:** agent-backup crate — agent has own backup logic. No podman exec from API. Clean separation of concerns for future scalability.

| Option | Description | Selected |
|--------|-------------|----------|
| Agent uploads directly | Agent archives + uploads to S3/local directly | ✓ |
| Agent sends to Worker (proxy) | Agent sends bytes to Worker, Worker uploads | |

**User's choice:** Agent uploads directly

---

## Storage & Retention Policy

| Option | Description | Selected |
|--------|-------------|----------|
| S3-compatible only (S3/R2/MinIO/DO Spaces) | Single UI with endpoint, bucket, region, credentials | |
| S3-compatible + local | Keep local filesystem option | ✓ |

**User's choice:** S3-compatible + local

| Option | Description | Selected |
|--------|-------------|----------|
| Per-server credentials | Each server has own S3 creds | |
| Reference platform-level configs | Admin pre-configures S3 profiles | ✓ |

**User's choice:** Platform-level configs with future per-server override support

| Option | Description | Selected |
|--------|-------------|----------|
| Simple: max days | Single input: delete older than N days | |
| Label-based: keep rules | Keep 7 daily, 4 weekly, 3 monthly | ✓ |

**User's choice:** Label-based: keep rules

| Option | Description | Selected |
|--------|-------------|----------|
| Worker (as part of job completion) | Inline with the backup job | |
| Worker (separate periodic task) | Decoupled cleanup loop | ✓ |
| Agent prunes during backup | Agent handles retention | |

**User's choice:** Worker (separate periodic task)

---

## the agent's Discretion

- Specific agent-backup crate implementation for archive creation and upload
- Worker cron evaluation loop design (polling interval, error handling, retry)
- Detailed API endpoint design for backup config CRUD
- UI component structure within the Backup tab (form layout, responsive breakpoints, styling)
- S3 profile CRUD implementation in platform settings
- Migration script design for existing server.backup_cron → cron_tasks
- Label-based retention rule parsing and evaluation algorithm
- Prune task scheduling (frequency, batching, error recovery)

## Deferred Ideas

- **Per-server S3 credential override** — future phase
- **Automated non-backup task types** (restart, stop, command) — future phase
- **Backup restore automation** — partially implemented, full UX deferred
- **Backup notifications** (Discord webhook) — future phase
