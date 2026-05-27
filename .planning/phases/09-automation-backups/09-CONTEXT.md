# Phase 9: Automation & Backups - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Automated backup scheduling and crash recovery. This phase establishes backup scheduling, retention policies, crash recovery, and storage options.

**Success criteria:**
1. User can schedule automated backups
2. User can restore from backup
3. Server auto-restarts on crash without user intervention
4. Backup retention policies work correctly
</domain>

<decisions>
## Implementation Decisions

### Backup Scheduling (D-32)
- **D-32:** Cron expression
- Cron expression per server (stored in backup_cron field)
- Scheduler service runs every minute, evaluates cron expressions
- UTC timezone for consistency
- Reference: `api/src/application/services/backup_scheduler.rs`

### Retention Policies (D-33)
- **D-33:** Count-based retention
- Keep N most recent backups per server (max_retained_backups field)
- Delete older backups automatically after new backup completes
- Reference: `api/src/application/services/backup_service.rs`

### Crash Recovery (D-34)
- **D-34:** Container auto-restart
- Docker/Podman auto-restart policy handles crashes
- Health check monitors server health
- restart_count tracked in database
- Reference: `api/src/infrastructure/executors/podman_server_executor.rs`

### Backup Storage (D-35)
- **D-35:** Local + S3
- Local disk storage by default
- S3 configurable per server via backup_provider field
- S3 credentials configured at system level
- Reference: `api/src/infrastructure/backup/`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backup Service
- `api/src/application/services/backup_service.rs` — Backup logic
- `api/src/application/services/backup_scheduler.rs` — Cron scheduler

### Handlers
- `api/src/presentation/handlers/backup_handlers.rs` — Backup endpoints

### Repository
- `api/src/infrastructure/repositories/postgres_backup_repository.rs` — Backup persistence

### Server Model
- `api/src/domain/server/model.rs` — auto_backup_enabled, backup_cron, backup_provider, max_retained_backups, auto_restart

### Frontend
- `app/src/pages/servers/ServerDetailsPage.jsx` — Backup UI

</canonical_refs>

<specifics>
## Specific Ideas

- Existing fields: auto_backup_enabled, backup_cron, backup_provider, backup_path, max_retained_backups, auto_restart, restart_count
- Backup endpoints: POST /:id/backups, GET /:id/backups, DELETE /:id/backups/:backup_id, POST /:id/backups/:backup_id/restore
- BackupService and BackupScheduler already exist in container
- BackupRepository trait exists with create, list, get, delete methods

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 09-automation-backups*
*Context gathered: 2026-04-09*
