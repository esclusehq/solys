# Phase 55: Scheduled Backups - Context

**Gathered:** 2026-05-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver automated scheduled backups for game server data — configurable backup intervals, retention policies, and backup storage location. Builds on existing backup infrastructure (BackupScheduler, BackupService, backup_history table, backup columns on servers table) and unifies the two parallel scheduling systems into a single canonical path using cron_tasks table + Worker service.

Key change: restructure backup execution from API-side (podman exec) to Worker-orchestrated + Agent-executed with agent-backup crate.

</domain>

<decisions>
## Implementation Decisions

### Scheduling Unification
- **D-01:** Consolidate into cron_tasks as the canonical scheduling system. Deprecate server.backup_cron as source of truth for scheduled backups.
- **D-02:** Cron evaluation lives in the Worker service (Redis job queue), not in-process in the API server. Worker already has job processing infrastructure and a `process_backup_server` stub.
- **D-03:** server.backup_cron and auto_backup_enabled fields stay on the servers table as a shortcut for simple config. UI toggle for "auto backup" writes to both cron_tasks and server.backup_cron. Migration script to copy existing server.backup_cron values into cron_tasks.
- **D-04:** Only backup task type (cron_tasks.task_type = 'backup') is fully automated for now. Restart/stop/command task types remain manual-trigger-only.

### Backup Config UI Placement
- **D-05:** Backup configuration panel lives in the Backup tab on ServerDetails page, above the existing backup history table (ServerBackups.jsx). All backup settings in one place.
- **D-06:** Schedule input: preset dropdown with common options (Every 6h, 12h, Daily, Weekly, Monthly) + custom cron expression field. UX simpler than the existing ScheduledTasksPage.
- **D-07:** Retention: both max count and label-based time retention (keep 7 daily, 4 weekly, 3 monthly). Implementation handles combined rules — earliest trigger prunes.
- **D-08:** Storage provider: selectable per server — local or S3-compatible (AWS S3, Cloudflare R2, MinIO, DigitalOcean Spaces).

### Backup Execution Model
- **D-09:** Worker = orchestration layer. Worker evaluates cron_tasks, dispatches backup jobs via Redis queue, coordinates completion.
- **D-10:** Agent = backup implementation. Agent has its own backup logic using the agent-backup crate (zstd/gzip compression). No podman exec commands sent from API to Agent. Clean separation of concerns for future runtime/container migration.
- **D-11:** Agent uploads backup archive directly to storage (S3/local) using existing rusoto_s3 integration. Worker does not proxy archive bytes.
- **D-12:** Current API-side BackupService (podman exec + podman cp) is replaced by agent-side execution. The backup.start command is sent via WebSocket to the agent.

### Storage & Retention Policy
- **D-13:** Storage providers: S3-compatible (S3/R2/MinIO/DO Spaces) + local filesystem.
- **D-14:** S3 credentials: reference platform-level config profiles (admin pre-configures S3 profiles in settings). Per-server override support deferred to future phase.
- **D-15:** Retention pruning: Worker runs a separate periodic task (decoupled from backup jobs) that evaluates retention rules across all servers and triggers cleanup.
- **D-16:** Time-based retention uses label-based rules (e.g., keep 7 daily, 4 weekly, 3 monthly) rather than simple max-days.

### the agent's Discretion
- Specific agent-backup crate implementation for archive creation and upload
- Worker cron evaluation loop design (polling interval, error handling, retry)
- Detailed API endpoint design for backup config CRUD
- UI component structure within the Backup tab (form layout, responsive breakpoints, styling)
- S3 profile CRUD implementation in platform settings
- Migration script design for existing server.backup_cron → cron_tasks
- Label-based retention rule parsing and evaluation algorithm
- Prune task scheduling (frequency, batching, error recovery)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scheduling & Backup Infrastructure
- `api/src/application/services/backup_scheduler.rs` — current BackupScheduler (will be replaced by Worker cron evaluation)
- `api/src/application/services/backup_service.rs` — core BackupService (trigger, archive, upload, prune)
- `api/src/presentation/handlers/backup_handlers.rs` — REST handlers for backup operations
- `api/src/domain/entities/cron_task.rs` — CronTask entity (canonical schedule source)
- `api/src/domain/repositories/cron_task_repository.rs` — CronTaskRepository trait
- `api/src/domain/entities/backup.rs` — BackupRecord entity
- `api/src/domain/repositories/backup_repository.rs` — BackupRepository trait
- `api/infrastructure/repositories/postgres_backup_repository.rs` — Postgres backup repo impl

### Database Schema
- `migration/20260302000002_create_backup_history.sql` — backup_history table
- `migration/20260302000003_add_backup_fields.sql` — server backup columns
- `migration/20260409000006_create_cron_tasks_table.sql` — cron_tasks table

### Agent Backup
- `agent/agent-core/crates/agent-backup/src/lib.rs` — agent-backup crate root
- `agent/agent-core/crates/agent-backup/src/compression.rs` — zstd/gzip compressors

### Worker Service
- `worker/src/main.rs` — Worker entry point
- `worker/src/queue/mod.rs` — Job queue processor (has process_backup_server stub)

### Frontend
- `app/src/components/ServerBackups.jsx` — existing backup history UI (will add config panel above)
- `app/src/pages/ServerDetails.jsx` — Server details page with backup tab
- `app/src/hooks/useBackups.js` — backup API hook
- `app/src/features/scheduling/ScheduledTasksPage.jsx` — existing cron task CRUD page

### Storage
- `api/src/infrastructure/storage/mod.rs` — StorageProvider trait
- `api/src/infrastructure/storage/s3_client.rs` — S3-compatible storage client
- `api/src/infrastructure/storage/local_client.rs` — local filesystem storage

### Codebase Maps
- `.planning/codebase/STACK.md` — tech stack (Rust Axum, React 19, Zustand, Tailwind CSS v4)
- `.planning/codebase/ARCHITECTURE.md` — architecture (Clean Architecture, Agent-based node mgmt)
- `.planning/codebase/INTEGRATIONS.md` — external integrations (S3, Redis, WebSocket)

### Roadmap
- `.planning/ROADMAP.md` §Phase 55 — phase goal and dependencies

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ServerBackups.jsx` — existing backup history table with trigger/delete/restore actions. Config panel will be added above this table.
- `useBackups.js` — backup API hook (triggerBackup, deleteBackup, restoreBackup, list). Extend with config endpoints.
- `BackupService` — existing backup logic (trigger, archive, upload, prune). Agent-side implementation will replace the archive/upload portion.
- `BackupScheduler` — existing cron evaluation pattern (60s tick, cron parsing, fire detection). Logic to move to Worker.
- `agent-backup` crate — zstd/gzip compression utilities ready for backup execution on agent side.
- `postgres_backup_repository` — full CRUD for backup_history table.
- Storage abstraction (`StorageProvider` trait + `S3Client` + `LocalClient`) — can be reused by agent for direct upload.

### Established Patterns
- Worker job dispatching via Redis priority queues (create_server, delete_server, start_server, stop_server patterns)
- Agent WebSocket command dispatch (backup.restore already uses this pattern)
- Axum custom extractors for auth (AuthUser pattern)
- Zustand stores with persist middleware for state management
- Tab-based layout on ServerDetails page with component pattern
- Cron evaluation using `cron` crate (Schedule::from_str + schedule.after().take(1))

### Integration Points
- **Worker**: implement cron_tasks evaluation loop + backup_server job handler (replace stub)
- **Agent**: implement backup execution in agent-backup crate + WebSocket handler for backup.start command
- **Frontend**: add backup config panel to ServerBackups.jsx (above existing table)
- **API**: add backup config CRUD endpoints + S3 profile CRUD in platform settings
- **Database**: modify prune logic in backup_service to support time-based retention rules; optionally add retention_rules column to servers table
- **Migration**: copy existing server.backup_cron values into cron_tasks for existing servers

### Creative Options
- Label-based retention (keep 7 daily, 4 weekly) enables sophisticated cleanup without user complexity
- agent-backup crate can be extended as the canonical backup engine supporting multiple runtime types (Docker, Podman, raw filesystem)

</code_context>

<specifics>
## Specific Ideas

Architecture flow for scheduled backup:
```
cron_tasks table → Worker cron evaluation → Worker dispatches backup_server job
  → Agent receives backup.start via WebSocket → Agent archives using agent-backup crate
  → Agent uploads directly to S3/local → Agent reports completion
  → Worker updates backup_history → Worker prune task evaluates retention rules
```

User flow for configuration:
```
Backup tab → Enable auto-backup toggle → Pick schedule preset (or custom cron)
  → Set retention rules (keep 7 daily, 4 weekly, 3 monthly)
  → Pick storage provider (local / S3-compatible) → Save
```

</specifics>

<deferred>
## Deferred Ideas

- **Per-server S3 credential override** — future phase. For now, S3 credentials are platform-level config profiles only
- **Automated non-backup task types** (restart, stop, command) — future phase when cron_tasks is fully mature
- **Backup restore automation** — restore is partially implemented (delegates to agent). Full restore UX/flow improvements deferred
- **Backup notifications** (Discord webhook on backup success/failure) — future phase

</deferred>

---

*Phase: 55-scheduled-backups*
*Context gathered: 2026-05-30*
