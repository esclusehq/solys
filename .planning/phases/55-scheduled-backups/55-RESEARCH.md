# Phase 55: Scheduled Backups — Research

**Researched:** 2026-05-30
**Domain:** Backup scheduling, cron task evaluation, agent-side backup execution, retention policies, storage management
**Confidence:** HIGH

## Summary

This phase delivers automated scheduled backups for game server data by unifying two parallel scheduling systems (API-side `BackupScheduler` + `cron_tasks` table) into a single canonical path using the Worker service. The backup execution model shifts from API-side `podman exec` + `podman cp` to Worker-orchestrated + Agent-executed backup using the `agent-backup` crate for archive creation and direct upload to S3/local storage.

**Architecture flow:** `cron_tasks` table → Worker cron evaluation loop → Worker dispatches `backup_server` job via Redis → Worker sends `backup.start` WebSocket command to agent → Agent creates archive using `agent-backup` crate → Agent uploads directly to S3/local → Worker updates `backup_history` → Worker independent prune task evaluates label-based retention rules.

The key architectural change is **D-09/D-10**: backup execution moves entirely from the API server to the Agent. The existing `BackupService::trigger_backup` method (which runs `podman exec` + `podman cp` from the API) is replaced by the Agent creating the backup archive locally and uploading directly to storage.

**Primary recommendation:** Implement in 4 parallel tracks — (1) Worker cron evaluation + backup job dispatch, (2) Agent backup.start handler + direct upload, (3) API config CRUD + S3 profile management, (4) Frontend Backup tab config panel with preset dropdown + label-based retention + storage provider selection. Then wire them all together with the Worker-side prune task.

## User Constraints (from CONTEXT.md)

<user_constraints>
### Locked Decisions

#### Scheduling Unification
- **D-01:** Consolidate into cron_tasks as the canonical scheduling system. Deprecate server.backup_cron as source of truth for scheduled backups.
- **D-02:** Cron evaluation lives in the Worker service (Redis job queue), not in-process in the API server. Worker already has job processing infrastructure and a `process_backup_server` stub.
- **D-03:** server.backup_cron and auto_backup_enabled fields stay on the servers table as a shortcut for simple config. UI toggle for "auto backup" writes to both cron_tasks and server.backup_cron. Migration script to copy existing server.backup_cron values into cron_tasks.
- **D-04:** Only backup task type (cron_tasks.task_type = 'backup') is fully automated for now. Restart/stop/command task types remain manual-trigger-only.

#### Backup Config UI Placement
- **D-05:** Backup configuration panel lives in the Backup tab on ServerDetails page, above the existing backup history table (ServerBackups.jsx). All backup settings in one place.
- **D-06:** Schedule input: preset dropdown with common options (Every 6h, 12h, Daily, Weekly, Monthly) + custom cron expression field. UX simpler than the existing ScheduledTasksPage.
- **D-07:** Retention: both max count and label-based time retention (keep 7 daily, 4 weekly, 3 monthly). Implementation handles combined rules — earliest trigger prunes.
- **D-08:** Storage provider: selectable per server — local or S3-compatible (AWS S3, Cloudflare R2, MinIO, DigitalOcean Spaces).

#### Backup Execution Model
- **D-09:** Worker = orchestration layer. Worker evaluates cron_tasks, dispatches backup jobs via Redis queue, coordinates completion.
- **D-10:** Agent = backup implementation. Agent has its own backup logic using the agent-backup crate (zstd/gzip compression). No podman exec commands sent from API to Agent. Clean separation of concerns for future runtime/container migration.
- **D-11:** Agent uploads backup archive directly to storage (S3/local) using existing rusoto_s3 integration. Worker does not proxy archive bytes.
- **D-12:** Current API-side BackupService (podman exec + podman cp) is replaced by agent-side execution. The backup.start command is sent via WebSocket to the agent.

#### Storage & Retention Policy
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

### Deferred Ideas (OUT OF SCOPE)
- Per-server S3 credential override — future phase
- Automated non-backup task types (restart, stop, command) — future phase
- Backup restore automation — restore is partially implemented (delegates to agent). Full restore UX/flow improvements deferred
- Backup notifications (Discord webhook on backup success/failure) — future phase
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cron schedule evaluation | Worker | — | D-02 mandates Worker-owned cron evaluation. Worker queries `cron_tasks` for due tasks and dispatches backup jobs via Redis. |
| Backup archive creation | Agent | — | D-10: Agent has the container running locally, creates archive with `agent-backup` crate (zstd/gzip). No API-side `podman exec`. |
| Backup upload to storage | Agent | — | D-11: Agent uploads directly to S3/local. Agent already has `rusoto_s3` integration. |
| Backup config CRUD (API) | API / Backend | — | New REST endpoints: `GET/PUT /servers/:id/backup-config` for per-server backup settings. |
| S3 profile CRUD (settings) | API / Backend | — | Platform-level S3 credentials management. Extends existing `settings_handlers.rs` pattern (single config → profiles). |
| Retention pruning | Worker | — | D-15: Worker runs decoupled prune task evaluating label-based rules across all servers. |
| Migration (backup_cron → cron_tasks) | API / Backend | — | One-time migration script copying existing `server.backup_cron` values into `cron_tasks` table. |
| Backup config UI | Browser / Client | — | Backup tab on ServerDetails page, above existing `ServerBackups.jsx` table. |
| S3 profile UI (settings) | Browser / Client | — | Platform settings page for admin to pre-configure S3 profiles. |

## Standard Stack

### Core Backend (Rust Axum) — Existing Infrastructure
| Library | Where | Purpose |
|---------|-------|---------|
| `cron` v0.15 | `api/Cargo.toml` | Cron expression parsing. Currently used by `BackupScheduler`. Worker needs same. |
| `sqlx` v0.7 | `api/Cargo.toml`, `worker/Cargo.toml` | PostgreSQL ORM. Worker needs DB access for `cron_tasks` queries. |
| `chrono` v0.4 | All Rust services | DateTime handling for retention rules, timestamps |
| `uuid` v1 | All Rust services | UUID generation for backup records, job IDs |

### Worker Service — Needs Enhancement
| Library | Where | Purpose | Why |
|---------|-------|---------|-----|
| `redis` v0.25 | `worker/Cargo.toml` | Already has Redis client. Needed for cron polling + job dispatch. |
| `sqlx` v0.7 | `worker/Cargo.toml` | Already has sqlx with postgres feature. Needed for cron_tasks queries. |
| `cron` crate | NOT in worker yet | Need to add for cron expression evaluation. Same version as API: `cron = "0.15"`. |
| `reqwest` v0.12 | `worker/Cargo.toml` | Already has reqwest. Needed for WebSocket communication API (or direct WS). |

### Agent (solys) — Needs Enhancement
| Library | Where | Purpose | Why |
|---------|-------|---------|-----|
| `agent-backup` crate | `agent-core/crates/agent-backup` | Archive creation (zstd/gzip). Already has `compression.rs` with `compress_zstd()` and `compress_gzip()`. |
| `rusoto_s3` v0.48 | `agent-core/Cargo.toml` (workspace) | S3 upload. Already used in `handlers/backup.rs` `upload_to_s3()`. |
| `tar` v0.4 + `flate2` v1 | `agent-backup/Cargo.toml` | Archive creation. Already in dependency list. |
| `zstd` v0.13 | `agent-backup/Cargo.toml` | Zstandard compression. Already in dependency list. |
| `sha2` v0.10 | `agent-backup/Cargo.toml` | Checksum calculation for backup verification. Already in dependency list. |
| `tokio` v1 | Workspace | Async runtime for all agent operations. Already present. |

### Frontend (React) — Existing
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| React (state, hooks) | 19.2.4 | UI framework for backup config panel | Existing in ServerBackups.jsx |
| Zustand v5 | 5.0.12 | State management | Not needed for backups specifically (existing pattern) |
| `fetchApi` client | — | API calls | Existing pattern in `app/src/api/client.js` |

### Alternatives Considered — None
All work uses existing stack components. No new libraries needed. The Worker already has Redis + SQLite/Postgres, the Agent already has backup crates.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        BACKEND (Rust / Axum API)                            │
│                                                                             │
│  ┌────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │BackupConfig     │  │S3 Profile CRUD  │  │Migration Script:           │  │
│  │REST endpoints   │  │(Admin Settings) │  │server.backup_cron          │  │
│  │/servers/:id/    │  │/settings/s3     │  │→ cron_tasks                │  │
│  │backup-config   │  │(extends existing)│  │(one-time)                  │  │
│  └───────┬─────────┘  └────────┬────────┘  └─────────────────────────────┘  │
│          │                     │                                             │
│  ┌───────▼─────────────────────▼─────────────────────────────────────────┐  │
│  │                    PostgreSQL                                          │  │
│  │  ┌──────────────┐   ┌────────────────┐   ┌────────────────────┐      │  │
│  │  │cron_tasks    │   │servers         │   │backup_history     │      │  │
│  │  │id            │   │backup_cron*    │   │id, server_id      │      │  │
│  │  │server_id     │   │auto_backup_en* │   │file_name, provider │      │  │
│  │  │task_type     │◄──│backup_provider │   │storage_path       │      │  │
│  │  │schedule_cron │   │max_backups     │   │size_bytes, status  │      │  │
│  │  │enabled       │   │retention_rules │   │created_at          │      │  │
│  │  │last_run      │   └────────────────┘   └────────────────────┘      │  │
│  │  │next_run      │                                                     │  │
│  │  └──────────────┘                                                     │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │    Redis Queue   │
                    │  queue:jobs:high │
                    │  queue:jobs:norm │
                    │  queue:jobs:low  │
                    └────────┬────────┘
                             │
┌────────────────────────────▼────────────────────────────────────────────────┐
│                          WORKER SERVICE (Rust)                               │
│                                                                             │
│  ┌──────────────────────────────┐  ┌──────────────────────────────────┐    │
│  │ Cron Evaluation Loop         │  │ backup_server Job Handler       │    │
│  │ (new)                        │  │ (existing stub, needs fill)     │    │
│  │                              │  │                                  │    │
│  │ while true:                  │  │ process_backup_server(job):     │    │
│  │   query cron_tasks WHERE     │  │   1. Get server info from DB    │    │
│  │     enabled=true AND         │──►   2. Send backup.start via      │    │
│  │     next_run <= NOW()        │  │      WebSocket/nodified client  │    │
│  │   for each due task:         │  │   3. Create backup_history       │    │
│  │     dispatch backup_server   │  │      record (status=in_progress)│    │
│  │     job to redis queue       │  │   4. Wait for agent completion  │    │
│  │   sleep(30s)                 │  │   5. Update backup_history       │    │
│  └──────────────────────────────┘  └──────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────┐                           │
│  │ Prune Task (decoupled from backup jobs)     │                           │
│  │                                             │                           │
│  │ Runs on separate schedule (e.g., every 15m) │                           │
│  │ Evaluates retention rules (count + label)   │                           │
│  │ Deletes old backups from storage + DB       │                           │
│  └─────────────────────────────────────────────┘                           │
└────────────────────────────┬────────────────────────────────────────────────┘
                             │ WebSocket / execute_command
                             │ command: "backup.start"
                             │
┌────────────────────────────▼────────────────────────────────────────────────┐
│                     AGENT (solys — runs on node)                             │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────┐           │
│  │ agent_connection.rs: receives "execute_command" via WS      │           │
│  │   → maps command "backup.start" → task_type "backup.start" │           │
│  │   → constructs Task with payload {server_id, container_id,  │           │
│  │     backup_provider, s3_config, retention_rules}            │           │
│  └──────────────────────┬──────────────────────────────────────┘           │
│                         │                                                  │
│  ┌──────────────────────▼──────────────────────────────────────┐           │
│  │ handlers/mod.rs: execute_task → backup.start               │           │
│  │   → backup::handle_start(task)                              │           │
│  │                                                              │           │
│  │   1. Create archive using agent-backup crate:                │           │
│  │      - tar + zstd or tar + gzip                             │           │
│  │      - Compresses server /data directory                    │           │
│  │   2. Upload directly to storage:                            │           │
│  │      - S3-compatible via rusoto_s3                          │           │
│  │      - Local filesystem to /var/lib/escluse-agent/backups/  │           │
│  │   3. Report result via WebSocket                            │           │
│  └─────────────────────────────────────────────────────────────┘           │
│                                                                             │
│  agent-backup crate (agent-core/crates/agent-backup/):                     │
│    compression.rs: compress_zstd(), compress_gzip(), decompress_*()        │
│    (will be extended with: archive creation + upload logic)                 │
└─────────────────────────────────────────────────────────────────────────────┘

User Flow:
  Settings Tab: Enable Backup → Set Schedule → Set Retention → Pick Storage → Save
       │                                                                        
       ▼                                                                        
  API writes to: cron_tasks (new row) + servers.backup_cron (shortcut)          
       │                                                                        
       ▼                                                                        
  Worker (every 30s): polls cron_tasks WHERE enabled=true AND next_run <= NOW()
       │                                                                        
       ▼                                                                        
  If due: dispatch "backup_server" job to Redis queue → Worker processes it    
       │                                                                        
       ▼                                                                        
  Worker: create backup_history record → send "backup.start" cmd to agent       
       │                                                                        
       ▼                                                                        
  Agent: archive → upload → report result                                      
       │                                                                        
       ▼                                                                        
  Worker: update backup_history → (decoupled) prune task cleans old backups
```

### Recommended Project Structure

```
worker/src/
├── main.rs                          # Add sqlx pool + CronEvalLoop startup
├── config.rs                        # Add DATABASE_URL for cron_tasks queries
├── cron_eval.rs                     # NEW: cron evaluation loop (poll cron_tasks, dispatch)
└── queue/mod.rs                     # Fill process_backup_server() stub

agent/solys/src/
├── handlers/
│   └── backup.rs                    # EXTEND: add handle_start(task) for backup.start
│   └── mod.rs                       # ADD: "backup.start" → backup::handle_start
└── agent_connection.rs              # EXTEND: add "backup.start" → "backup.start" mapping

agent/agent-core/crates/agent-backup/src/
├── lib.rs                           # EXTEND: add archive + upload modules
├── compression.rs                   # EXISTING: zstd/gzip compress/decompress
├── archive.rs                       # NEW: tar-based archive creation
└── upload.rs                        # NEW: S3 + local upload logic

api/src/
├── presentation/handlers/
│   └── backup_config_handlers.rs    # NEW: backup config CRUD endpoints
│   └── settings_handlers.rs         # EXTEND: add S3 profile CRUD
├── domain/
│   ├── entities/
│   │   └── backup_config.rs         # NEW: BackupConfig entity (retention rules, provider)
│   │   └── s3_profile.rs            # NEW: S3Profile entity (multiple profiles)
│   ├── repositories/
│   │   └── backup_config_repository.rs  # NEW: trait for backup config CRUD
│   │   └── s3_profile_repository.rs     # NEW: trait for S3 profiles
├── infrastructure/repositories/
│   │   └── postgres_backup_config_repository.rs  # NEW
│   │   └── postgres_s3_profile_repository.rs     # NEW
├── application/services/
│   │   └── backup_config_service.rs       # NEW: use case for backup config management
└── migration/
    └── 20260530000001_add_retention_rules.sql  # NEW: retention_rules column on servers
    └── 20260530000002_create_s3_profiles.sql   # NEW: s3_profiles table
    └── 20260530000003_migrate_backup_cron.sql  # NEW: migrate server.backup_cron → cron_tasks

app/src/
├── components/
│   └── ServerBackupConfig.jsx        # NEW: backup config panel (above history table)
│   └── ServerBackups.jsx             # MODIFY: import + render ServerBackupConfig above table
├── hooks/
│   └── useBackupConfig.js            # NEW: API hook for backup config
│   └── useBackups.js                 # EXTEND: add config endpoints
├── api/
│   └── backupConfig.js               # NEW: API client for backup config endpoints
├── pages/
│   └── ServerDetails.jsx             # MODIFY: move backup config panel from Settings tab to Backup tab
└── store/
    └── backupConfigStore.js          # OPTIONAL: Zustand store if needed for complex state
```

### Pattern 1: Worker Cron Evaluation Loop (D-01 to D-04)
**What:** Worker polls `cron_tasks` table for due tasks (enabled=true AND next_run <= NOW()) and dispatches backup jobs via Redis queue. Uses the `cron` crate for next_run calculation on task create/update.

**When to use:** Worker startup, running as a background task alongside the existing job processor. This replaces the API-side `BackupScheduler`.

**Pattern source:** `worker/src/queue/mod.rs` — existing `run()` loop pattern. `api/src/application/services/backup_scheduler.rs` — cron evaluation pattern using `Schedule::from_str` + `after().take(1)`.

```rust
// worker/src/cron_eval.rs (NEW)
// Runs in the Worker alongside the job processor loop
pub async fn run_cron_evaluation_loop(
    pool: PgPool,
    redis: redis::aio::MultiplexedConnection,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = evaluate_and_dispatch(&pool, &redis).await {
            tracing::error!("Cron evaluation error: {}", e);
        }
    }
}

async fn evaluate_and_dispatch(
    pool: &PgPool,
    redis: &redis::aio::MultiplexedConnection,
) -> anyhow::Result<()> {
    // Query due backup tasks only (D-04: only backup is automated)
    // The next_run column is updated by the API when creating/updating cron_tasks
    let due_tasks = sqlx::query_as::<_, CronTask>(r#"
        SELECT id, server_id, user_id, task_type, schedule_cron, command, enabled,
               last_run, next_run, created_at, updated_at
        FROM cron_tasks
        WHERE enabled = true 
          AND task_type = 'backup'
          AND next_run <= NOW()
        ORDER BY next_run ASC
        LIMIT 50
    "#)
    .fetch_all(pool)
    .await?;

    for task in due_tasks {
        // Enqueue backup job via Redis priority queue
        let job = Job {
            job_id: Uuid::new_v4(),
            job_type: "backup_server".to_string(),
            payload: serde_json::json!({
                "cron_task_id": task.id,
                "server_id": task.server_id,
                "user_id": task.user_id,
            }),
            user_id: task.user_id,
            priority: 0, // normal
            created_at: Utc::now().timestamp(),
        };
        
        let job_key = format!("job:{}", job.job_id);
        let queue_key = "queue:jobs:normal";
        
        redis::cmd("HSET")
            .arg(&job_key)
            .arg("data")
            .arg(serde_json::to_string(&job)?)
            .query_async::<_, ()>(redis)
            .await?;
        
        redis::cmd("ZADD")
            .arg(queue_key)
            .arg(job.created_at as f64)
            .arg(job.job_id.to_string())
            .query_async::<_, ()>(redis)
            .await?;
        
        // Update last_run + calculate next_run
        // (next_run should be calculated at CREATE time, but update here as safeguard)
        sqlx::query(
            "UPDATE cron_tasks SET last_run = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(task.id)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}
```

### Pattern 2: Worker `backup_server` Job Handler (D-09, D-10, D-12)
**What:** The Worker processes the `backup_server` job type — sends a `backup.start` command to the Agent via the existing WebSocket `execute_command` path, creates a `backup_history` record, waits for agent completion, and updates the record.

**Where:** `worker/src/queue/mod.rs` — extend the existing `process_backup_server` stub.

```rust
// Existing stub in worker/src/queue/mod.rs, needs implementation:
async fn process_backup_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
    let user_id: Uuid = serde_json::from_value(job.payload["user_id"].clone())?;
    
    // 1. Get server details (node_id, container_name, backup config)
    let server = sqlx::query_as::<_, Server>(r#"
        SELECT id, node_id, name, container_name, backup_provider, 
               auto_backup_enabled, retention_rules, max_retained_backups
        FROM servers WHERE id = $1
    "#)
    .bind(server_id)
    .fetch_optional(&self.pool)
    .await?
    .ok_or("Server not found")?;
    
    let node_id = server.node_id.ok_or("Server has no node assigned")?;
    
    // 2. Create backup_history record (in_progress)
    let backup_id = Uuid::new_v4();
    let file_name = format!("backup_{}_{}.tar.zst", 
        server.name.replace(|c: char| !c.is_alphanumeric() && c != '-', "_"),
        Utc::now().format("%Y%m%dT%H%M%S")
    );
    
    sqlx::query(r#"
        INSERT INTO backup_history (id, server_id, file_name, provider, status, created_at)
        VALUES ($1, $2, $3, $4, 'in_progress', NOW())
    "#)
    .bind(backup_id)
    .bind(server_id)
    .bind(&file_name)
    .bind(&server.backup_provider)
    .execute(&self.pool)
    .await?;
    
    // 3. Send backup.start command to agent via WebSocket
    // Worker needs access to the agent's WebSocket connection.
    // Option A: Worker sends command through API's node_client endpoint (HTTP call to API)
    // Option B: Worker has its own WebSocket connection pool (more complex)
    // Option C: Worker sends command via HTTP to an API endpoint that proxies to agent
    
    // Using Option A (recommended — simplest, worker already has reqwest):
    let api_url = format!("{}/api/v1/nodes/{}/commands", 
        std::env::var("API_BASE_URL").unwrap_or("http://api:3000".to_string()),
        node_id
    );
    
    let payload = serde_json::json!({
        "command": "backup.start",
        "params": {
            "server_id": server_id,
            "container_name": server.container_name,
            "backup_id": backup_id,
            "file_name": file_name,
            "provider": server.backup_provider,
        }
    });
    
    let client = reqwest::Client::new();
    let response = client.post(&api_url)
        .json(&payload)
        .send()
        .await?;
    
    tracing::info!("Backup started for server {}: {}", server_id, file_name);
    Ok(())
}
```

**Critical note on WebSocket access:** The Worker currently has no WebSocket connection to the Agent. Three approaches:
- **(A) Via API HTTP endpoint** — simplest: Worker calls `POST /api/v1/nodes/:id/commands` on the API server, which proxies the command to the agent via its existing WebSocket connection. Minimal structural change.
- **(B) Worker-to-Agent direct WS** — Worker maintains its own WebSocket pool. More performant but duplicates the API's connection management.
- **(C) Worker->API HTTP->Agent WS** — Worker sends job data to API via HTTP, API dispatches to agent.

**Recommendation:** Approach A (Worker calls API endpoint). The node client infrastructure already exists in the API. The Worker already has `reqwest`. This keeps the Worker stateless regarding WebSocket connections.

### Pattern 3: Agent `backup.start` Handler (D-10, D-11)
**What:** New handler in `agent/solys/src/handlers/backup.rs` that creates the backup archive using the `agent-backup` crate and uploads directly to storage. Receives payload with `server_id`, `container_id`, `backup_id`, `file_name`, `provider`. Reports result back via WebSocket `TaskResult`.

**Pattern source:** `agent/solys/src/handlers/backup.rs` — existing `handle_create()` and `handle_restore()` patterns.

```rust
// In agent/solys/src/handlers/backup.rs — NEW handler
use agent_backup::{compress_zstd, create_tar_archive};

#[derive(Debug, Deserialize)]
pub struct BackupStartPayload {
    pub server_id: Uuid,
    pub container_id: String,
    pub backup_id: Uuid,
    pub file_name: String,
    pub provider: String, // "local" or "s3"
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BackupStartOutput {
    pub backup_id: Uuid,
    pub size_bytes: u64,
    pub checksum: String,
    pub storage_path: String,
}

pub async fn handle_start(task: Task) -> Result<serde_json::Value> {
    let payload: BackupStartPayload = serde_json::from_value(task.payload)?;
    let started_at = std::time::Instant::now();
    
    tracing::info!(
        server_id = %payload.server_id,
        backup_id = %payload.backup_id,
        "Starting agent-side backup"
    );
    
    // 1. Create archive from container volume using agent-backup crate
    let backup_dir = PathBuf::from("/var/lib/escluse-agent/backups")
        .join(payload.server_id.to_string());
    tokio::fs::create_dir_all(&backup_dir).await?;
    
    let archive_path = backup_dir.join(&payload.file_name);
    
    // Use agent-backup crate to create tar + compress
    // (agent-backup/archive.rs will handle this)
    let archive_size = agent_backup::create_container_backup(
        &payload.container_id,
        "/data", // server data volume
        &archive_path,
        agent_backup::CompressionFormat::Zstd(3), // zstd level 3
    ).await?;
    
    // 2. Calculate checksum
    let checksum = agent_backup::calculate_checksum(&archive_path).await?;
    
    // 3. Upload to storage directly (D-11)
    let storage_path = match payload.provider.as_str() {
        "s3" => {
            // Validate S3 config
            let endpoint = payload.s3_endpoint.ok_or("S3 endpoint required")?;
            let bucket = payload.s3_bucket.ok_or("S3 bucket required")?;
            let access_key = payload.s3_access_key.ok_or("S3 access key required")?;
            let secret_key = payload.s3_secret_key.ok_or("S3 secret key required")?;
            
            upload_to_s3_with_config(
                &endpoint, &bucket, &payload.s3_region.unwrap_or_default(),
                &access_key, &secret_key,
                &payload.server_id.to_string(),
                &payload.file_name,
                &archive_path,
            ).await?
        }
        _ => {
            // Local storage — file already exists at archive_path
            // Return the path as storage location
            archive_path.to_string_lossy().to_string()
        }
    };
    
    // 4. Report completion
    let output = BackupStartOutput {
        backup_id: payload.backup_id,
        size_bytes: archive_size,
        checksum,
        storage_path,
    };
    
    tracing::info!(
        backup_id = %output.backup_id,
        size_bytes = output.size_bytes,
        duration_ms = %started_at.elapsed().as_millis(),
        "Backup completed successfully"
    );
    
    Ok(serde_json::to_value(output)?)
}

// S3 upload function (extends existing upload_to_s3 with configurable endpoint)
async fn upload_to_s3_with_config(
    endpoint: &str,
    bucket: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
    server_id: &str,
    file_name: &str,
    file_path: &PathBuf,
) -> Result<String> {
    use rusoto_core::{Region, credential::StaticProvider, HttpClient};
    use rusoto_s3::{S3, S3Client, PutObjectRequest};
    use rusoto_sts::WebIdentityProvider;
    use std::str::FromStr;
    
    let credentials = StaticProvider::new(
        access_key.to_string(),
        secret_key.to_string(),
        None, None, None,
    );
    
    let region = if region.is_empty() {
        Region::Custom {
            name: "auto".to_string(),
            endpoint: endpoint.to_string(),
        }
    } else {
        Region::Custom {
            name: region.to_string(),
            endpoint: endpoint.to_string(),
        }
    };
    
    let client = S3Client::new_with(HttpClient::new()?, credentials, region);
    
    let file_data = tokio::fs::read(file_path).await?;
    let key = format!("{}/{}", server_id, file_name);
    
    let request = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.clone(),
        body: Some(file_data.into()),
        content_type: Some("application/zstd".to_string()),
        ..Default::default()
    };
    
    client.put_object(request).await?;
    
    Ok(format!("s3://{}/{}", bucket, key))
}
```

### Pattern 4: Frontend Backup Config Panel (D-05 to D-08)
**What:** React component added above the existing backup history table in `ServerBackups.jsx`. Contains: auto-backup toggle, schedule preset dropdown + custom cron input, retention rules inputs (count + label-based), storage provider selector, save button.

**Pattern source:** `app/src/pages/ServerDetails.jsx` lines 386-525 — existing backup config form in Settings tab (to be moved to Backup tab). `ScheduledTasksPage.jsx` — existing cron task CRUD pattern.

```jsx
// NEW: app/src/components/ServerBackupConfig.jsx
// Rendered ABOVE the existing backup history table in ServerBackups.jsx

import { useState, useEffect } from 'react';
import { fetchApi } from '../api/client';

const SCHEDULE_PRESETS = [
  { label: 'Every 6 hours', cron: '0 */6 * * *' },
  { label: 'Every 12 hours', cron: '0 */12 * * *' },
  { label: 'Daily (midnight)', cron: '0 0 * * *' },
  { label: 'Weekly (Sunday midnight)', cron: '0 0 * * 0' },
  { label: 'Monthly (1st midnight)', cron: '0 0 1 * *' },
  { label: 'Custom...', cron: '__custom__' },
];

const LABEL_RETENTION_OPTIONS = [
  { label: 'None (count only)', daily: 0, weekly: 0, monthly: 0 },
  { label: 'Keep 7 daily, 4 weekly, 3 monthly', daily: 7, weekly: 4, monthly: 3 },
  { label: 'Keep 14 daily, 6 weekly, 6 monthly', daily: 14, weekly: 6, monthly: 6 },
  { label: 'Keep 30 daily, 8 weekly, 12 monthly', daily: 30, weekly: 8, monthly: 12 },
];

export default function ServerBackupConfig({ serverId }) {
  const [config, setConfig] = useState({
    auto_backup_enabled: false,
    schedule_cron: '',
    backup_provider: 'local',
    max_retained_backups: 10,
    retention_daily: 7,
    retention_weekly: 4,
    retention_monthly: 3,
    selected_preset: null,
  });
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchApi(`/servers/${serverId}/backup-config`)
      .then(data => { if (data) setConfig(prev => ({ ...prev, ...data })); })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [serverId]);

  const handleSave = async () => {
    setSaving(true);
    try {
      await fetchApi(`/servers/${serverId}/backup-config`, {
        method: 'PUT',
        body: JSON.stringify(config),
      });
      // Show success toast
    } catch (err) {
      // Show error toast
    } finally {
      setSaving(false);
    }
  };

  // Label-based retention rules follow D-07 format (keep N daily, N weekly, N monthly)
  // These get stored as JSON in the retention_rules column on servers table
}
```

### Anti-Patterns to Avoid

- **Worker calling `podman exec`:** D-10 mandates that the Agent handles all container operations. The Worker must never directly exec into containers — it orchestrates via the Agent.
- **Worker proxying archive bytes:** D-11 mandates direct agent-to-storage upload. The Worker must never receive or forward backup archive data.
- **Retention pruning inline with backup:** D-15 decouples pruning from backup jobs. A separate task prevents backup failures from blocking retention cleanup.
- **In-memory cron evaluation state:** The old `BackupScheduler` used `HashMap<Uuid, DateTime>` for last-triggered tracking. The new system uses `cron_tasks.next_run` in the database — it's persistent and survives restarts.
- **Frontend-only validation:** D-10/D-12 means the Agent-side backup must be the canonical source of truth for backup success/failure. The frontend polls `backup_history` status — never assume completion.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Backup archive creation | Shell out to `tar`/`docker cp` from API | `agent-backup` crate (tar + zstd/gzip) | D-10: Agent-side execution. Existing crate has `compress_zstd()`/`compress_gzip()`. Extend with tar archive creation. |
| S3 upload from Agent | Implement new S3 client | Existing `rusoto_s3` in agent | Agent's `handlers/backup.rs` already has `upload_to_s3()` using `rusoto_s3`. Extend with configurable endpoint. |
| Cron expression parsing | Write custom parser | `cron` v0.15 crate | Already used by API `BackupScheduler`. Same version for Worker. |
| Job queue management | Build custom Redis queue | Existing priority queue pattern in `worker/src/queue/mod.rs` | Worker already has `zpopmin` + enqueue/dequeue pattern. Reuse for backup jobs. |
| WebSocket communication | Worker-to-Agent direct WS | API proxied via existing node_client | Worker calls `POST /api/v1/nodes/:id/commands` on API. API already has WebSocket pool. |
| Retention rule parsing | Complex UI for rule building | Preset dropdown (keep N daily, N weekly, N monthly) | D-07 specifies label-based retention. Simpler UX, straightforward algorithm. |
| S3 profile management | Full IAM/credentials system | Platform-level S3 config profiles (admin settings) | D-14: Single config per profile, no per-server override yet. |

**Key insight:** This phase is about **orchestrating existing infrastructure** more than building new components. The Worker, Agent backup handler, storage clients, and database schema all exist. The work is wiring them together with the right data flow.

## Common Pitfalls

### Pitfall 1: Worker has no direct WebSocket connection to Agent
**What goes wrong:** The Worker cannot directly send `backup.start` commands to the Agent because it has no WebSocket connection pool. The API server is the one connected to agents.
**Why it happens:** The Worker currently processes Redis jobs and calls APIs via HTTP. It doesn't maintain WebSocket connections to agents.
**How to avoid:** Worker calls `POST /api/v1/nodes/:id/commands` on the API server via HTTP. The API already has `NodeClient` with WebSocket connection management. Alternatively, the Worker can make DB updates and let the API's existing monitor loop pick up changes. Best: Worker calls API endpoint that proxies to agent WS.
**Warning signs:** Timeout waiting for backup to start, "Node not connected" errors from Worker.

### Pitfall 2: Agent `execute_command` handler doesn't map `backup.start`
**What goes wrong:** Agent receives `execute_command` with `command: "backup.start"` but the match at `agent_connection.rs:402-411` doesn't map it — falls through to `task_type = "unknown"` and fails.
**Why it happens:** The existing command-to-task_type mapping only handles server commands (create/start/stop/restart/delete/logs/command). Backup commands (`backup.create`, `backup.restore`) are routed through the agent's Task assignment system, not execute_command.
**How to avoid:** Add `"backup.start" | "backup.restore"` to the command mapping:
```rust
// In agent_connection.rs, add to the match block:
let task_type = match command.as_str() {
    "create" => "server.create",
    "start" => "server.start",
    "backup.start" => "backup.start", // NEW
    "backup.restore" => "backup.restore", // NEW (already exists but not mapped)
    _ => "unknown",
};
```
**Warning signs:** Agent logs show `task_type = "unknown"` when backup is triggered.

### Pitfall 3: S3 credentials not propagated to Agent
**What goes wrong:** The Agent receives a `backup.start` command with `provider: "s3"` but no S3 credentials. The upload fails.
**Why it happens:** S3 credentials are managed in the API's `settings_handlers.rs` as platform-level config. The Agent doesn't have access to the `app_settings` table. Credentials must be sent as part of the backup command payload.
**How to avoid:** When the API dispatches the backup command (via Worker → API → Agent), include S3 credentials in the payload. The Worker fetches the S3 config from `settings_repository` and includes it in the command params sent to the Agent.
**Warning signs:** Agent upload fails with "S3 endpoint required" or auth errors.

### Pitfall 4: Concurrent backup for the same server
**What goes wrong:** A scheduled backup fires while a manual backup is in progress. Two backup processes run on the same server simultaneously, potentially corrupting data.
**Why it happens:** The old `BackupScheduler` checked `has_active_backup()`. The new Worker cron evaluation loop must do the same before dispatching.
**How to avoid:** In the `process_backup_server` handler, check for active backup first:
```rust
let active = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(1) FROM backup_history WHERE server_id = $1 AND status = 'in_progress'"
)
.bind(server_id)
.fetch_one(&self.pool)
.await?;

if active > 0 {
    tracing::warn!("Backup already in progress for server {}, skipping", server_id);
    return Ok(());
}
```
**Warning signs:** Two `in_progress` records for the same server in `backup_history`.

### Pitfall 5: Retention rules interaction — count vs. label-based
**What goes wrong:** Both `max_retained_backups` (count-based) and retention rules (label-based) are active simultaneously. The wrong backups get pruned first.
**Why it happens:** D-07 says "earliest trigger prunes." But both rules independently could trigger pruning. If label-based retention says "keep 7 daily" and the count-based says "keep 50 max," the count-based rule may delete backups that label-based rules wanted to keep.
**How to avoid:** Implement combined evaluation:
1. First evaluate label-based retention: flag backups that should be deleted based on time labels
2. Then evaluate count-based: if after label pruning there are still more than `max_retained_backups`, delete oldest
3. The prune task selects all backups that are "candidates for deletion" from both rules and deletes the union (with overlap dedup)
**Warning signs:** Label-based retention rules never trigger because count-based threshold is reached first.

### Pitfall 6: Migration script + D-03 dual-write
**What goes wrong:** After the migration copies `server.backup_cron` → `cron_tasks`, the existing `BackupScheduler` (which reads `server.backup_cron`) and the new Worker (which reads `cron_tasks`) both fire backups for the same servers — double backups.
**Why it happens:** The old `BackupScheduler` is still running in the API server. D-03 keeps `server.backup_cron` as a shortcut field.
**How to avoid:** **Stop the old `BackupScheduler`** in the API server after migration. The API's `AppContainer` creates `BackupScheduler` at bootstrap and runs it in a background task. Remove or disable that startup. The new Worker's cron evaluation loop replaces it entirely.
**Warning signs:** Double backup records in `backup_history` for the same server at near-identical timestamps.

## Code Examples

### Verified Patterns from Codebase

### Worker: Adding sqlx + cron to Worker Cargo.toml
```toml
# worker/Cargo.toml — add cron dependency
[dependencies]
# ... existing deps ...
cron = "0.15"  # NEW: for cron expression evaluation
```

### Worker: Main.rs with CronEvalLoop
```rust
// worker/src/main.rs — extend with cron evaluation
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("RUST_LOG")
        .init();

    let config = config::Config::new()?;
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let redis = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_async_connection()
        .await?;

    // Start cron evaluation loop in background
    let cron_pool = pool.clone();
    let cron_redis = redis.clone();
    tokio::spawn(async move {
        cron_eval::run_cron_evaluation_loop(cron_pool, cron_redis).await;
    });

    // Start job processor (existing)
    let mut processor = queue::JobProcessor::new(pool, redis);
    processor.run().await;

    Ok(())
}
```

### Worker: process_backup_server with API proxy
```rust
// worker/src/queue/mod.rs — extended process_backup_server
async fn process_backup_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
    let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;
    
    // Get server info including node_id
    let row = sqlx::query(r#"
        SELECT s.id, s.node_id, s.name, s.backup_provider, s.max_retained_backups,
               s.retention_rules, c.container_name
        FROM servers s
        LEFT JOIN containers c ON c.server_id = s.id
        WHERE s.id = $1
    "#)
    .bind(server_id)
    .fetch_optional(&self.pool)
    .await?
    .ok_or("Server not found")?;
    
    let node_id: Uuid = row.try_get("node_id")?;
    
    // Create backup_history record
    let backup_id = Uuid::new_v4();
    let file_name = format!("backup_{}_{}.tar.zst", 
        row.try_get::<String, _>("name")?.replace(' ', "_"),
        Utc::now().format("%Y%m%dT%H%M%S")
    );
    
    sqlx::query(r#"
        INSERT INTO backup_history (id, server_id, file_name, provider, status, created_at)
        VALUES ($1, $2, $3, $4, 'in_progress', NOW())
    "#)
    .bind(backup_id)
    .bind(server_id)
    .bind(&file_name)
    .bind(row.try_get::<String, _>("backup_provider")?)
    .execute(&self.pool)
    .await?;
    
    // Dispatch command to agent via API (worker->API->agent)
    let api_url = format!("{}/api/v1/nodes/{}/commands", 
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://api:3000".to_string()),
        node_id
    );
    
    let body = serde_json::json!({
        "command": "backup.start",
        "server_id": server_id,
        "params": {
            "container_name": row.try_get::<Option<String>, _>("container_name")?,
            "backup_id": backup_id,
            "file_name": file_name,
            "provider": row.try_get::<String, _>("backup_provider")?,
        }
    });
    
    let client = reqwest::Client::new();
    let response = client.post(&api_url)
        .json(&body)
        .send()
        .await?;
    
    tracing::info!("Backup dispatched: server={} backup={} node={}", server_id, backup_id, node_id);
    Ok(())
}
```

### API: S3 Profile Management (extends settings_handlers.rs)
```rust
// Extending the existing S3 config pattern to support named profiles

// POST /api/v1/settings/s3/profiles — Create S3 profile
pub async fn create_s3_profile(
    State(container): State<ApiState>,
    auth_user: AuthUser,
    Json(payload): Json<S3Profile>,
) -> Result<impl IntoResponse, AppError> {
    if !auth_user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    
    let mut profiles = container.settings_repository.get_s3_profiles().await?;
    
    // Validate uniqueness
    if profiles.iter().any(|p| p.name == payload.name) {
        return Err(AppError::BadRequest("Profile name already exists".into()));
    }
    
    profiles.push(payload);
    container.settings_repository.save_s3_profiles(&profiles).await?;
    
    Ok(Json(ApiResponse::success(profiles)))
}
```

### Database: Retention Rules Migration
```sql
-- NEW: 20260530000001_add_retention_rules.sql
-- Add column for label-based retention rules
ALTER TABLE servers 
  ADD COLUMN IF NOT EXISTS retention_rules JSONB DEFAULT '{"daily": 7, "weekly": 4, "monthly": 3}',
  ADD COLUMN IF NOT EXISTS retention_mode TEXT DEFAULT 'hybrid' CHECK (retention_mode IN ('count', 'label', 'hybrid'));

-- NEW: 20260530000002_create_s3_profiles.sql
CREATE TABLE IF NOT EXISTS s3_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    endpoint TEXT NOT NULL,
    region TEXT NOT NULL DEFAULT '',
    bucket TEXT NOT NULL,
    access_key TEXT NOT NULL,
    secret_key TEXT NOT NULL,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- NEW: 20260530000003_migrate_backup_cron.sql
-- One-time migration: copy existing server.backup_cron into cron_tasks
INSERT INTO cron_tasks (id, server_id, user_id, task_type, schedule_cron, command, enabled, last_run, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    s.id,
    COALESCE(s.user_id, (SELECT id FROM users LIMIT 1)),
    'backup',
    s.backup_cron,
    NULL,
    s.auto_backup_enabled,
    NULL,
    NOW(),
    NOW()
FROM servers s
WHERE s.backup_cron IS NOT NULL AND s.backup_cron != ''
AND NOT EXISTS (
    SELECT 1 FROM cron_tasks ct 
    WHERE ct.server_id = s.id AND ct.task_type = 'backup'
);
```

### Frontend: Backup Config Hook
```javascript
// NEW: app/src/hooks/useBackupConfig.js
import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

export function useBackupConfig(serverId) {
  const [config, setConfig] = useState(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const data = await fetchApi(`/servers/${serverId}/backup-config`);
      setConfig(data);
    } catch (err) {
      console.error('Failed to fetch backup config:', err);
    } finally {
      setLoading(false);
    }
  }, [serverId]);

  useEffect(() => { refresh(); }, [refresh]);

  const saveConfig = useCallback(async (newConfig) => {
    try {
      setSaving(true);
      await fetchApi(`/servers/${serverId}/backup-config`, {
        method: 'PUT',
        body: JSON.stringify(newConfig),
      });
      setConfig(newConfig);
    } catch (err) {
      throw err;
    } finally {
      setSaving(false);
    }
  }, [serverId]);

  return { config, loading, saving, saveConfig, refresh };
}
```

### Agent: Adding backup.start to agent_connection.rs mapper
```rust
// In agent/solys/src/agent_connection.rs line ~402
// Extend the command mapper to include backup commands:

let task_type = match command.as_str() {
    "create" => "server.create",
    "start" => "server.start",
    "stop" => "server.stop",
    "restart" => "server.restart",
    "delete" => "server.delete",
    "logs" => "server.logs",
    "command" => "server.command",
    "backup.start" => "backup.start",   // NEW: not yet mapped
    "backup.restore" => "backup.restore", // NEW: already exists but not mapped
    _ => "unknown",
};
```

### Agent: Adding backup.start to handlers/mod.rs
```rust
// In agent/solys/src/handlers/mod.rs line ~127
// Add "backup.start" to the execute_single match:

async fn execute_single(task: &Task, ...) -> Result<serde_json::Value, HandlerError> {
    let task_type = task.task_type.clone();
    let future = async {
        match task_type.as_str() {
            // ... existing matches ...
            "backup.create" => backup::handle_create(task.clone()).await,
            "backup.restore" => backup::handle_restore(task.clone()).await,
            "backup.start" => backup::handle_start(task.clone()).await,  // NEW
            // ...
        }
    };
    // ...
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| API-side `BackupScheduler` (60s tick, HashMap last-triggered) | Worker-side cron evaluation (poll `cron_tasks`, Redis dispatch) | This phase | Persistent state (`cron_tasks.next_run`), survives restarts, Worker has dedicated job queue |
| API-side `BackupService::trigger_backup` (podman exec + podman cp) | Agent-side `backup.start` handler (agent-backup crate) | This phase | Clean separation of concerns. Agent has direct filesystem access. No API-to-container traffic. |
| `server.backup_cron` as source of truth | `cron_tasks` as canonical scheduling. `server.backup_cron` as shortcut | This phase | Single scheduling path. Prepares for future automated task types. |
| Simple count-based retention (`max_retained_backups`) | Label-based retention (keep N daily, N weekly, N monthly) + count | This phase | More sophisticated retention. Users keep granular historical backups. |
| Prune inline with backup (`BackupService::prune_old_backups`) | Worker decoupled prune task | This phase | Pruning doesn't block backups. Runs on its own schedule. |
| Single S3 config (global settings) | S3 profiles (named, selectable per server) | This phase | Multiple storage destinations. Each server can target different storage. |

### Deprecated/outdated:
- `BackupScheduler` struct in `api/src/application/services/backup_scheduler.rs` — will be disabled after migration. The Worker replaces this entirely.
- `BackupService::trigger_backup` — the API-side podman exec implementation. The endpoint handler at `backup_handlers.rs::trigger_backup` may remain as a manual trigger that dispatches via Worker, but the old `create_archive()` method with `podman exec` + `podman cp` is replaced.
- `server.backup_cron` as source of truth — D-01 deps it. It becomes a mirror of `cron_tasks` for simple reads.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Worker can communicate with Agent via HTTP call to API's `POST /api/v1/nodes/:id/commands` endpoint | Pattern 2 | If this endpoint doesn't proxy to WebSocket correctly, Agent never receives the command. Mitigation: verify the node commands endpoint works end-to-end before building Worker integration. |
| A2 | The `cron` crate's `Schedule::from_str` + `after().take(1)` pattern works identically in the Worker | Pattern 1 | Same crate version (0.15) as API. Low risk. |
| A3 | The Agent's existing `rusoto_s3` import can create a client with custom endpoint/credentials | Pattern 3, upload_to_s3_with_config | `rusoto_s3` supports `StaticProvider` and `Region::Custom`. Verified from `Cargo.toml` having `rusoto_s3 = "0.48"`. Medium confidence on the exact API — may need minor adaptation. |
| A4 | The existing `execute_command` WebSocket path in `agent_connection.rs` will route `backup.start` to the agent | Pattern 3 | Currently only server commands are mapped. D-10 requires adding `backup.start` mapping. Verified by code inspection — the match block is straightforward to extend. |
| A5 | S3 credentials can be passed as command params without security risk | Pitfall 3 | Credentials transit through Redis (Worker) + HTTP (Worker→API) + WebSocket (API→Agent). Redis is internal. HTTP/WS is internal Docker network. Acceptable for alpha. Future: Agent fetches credentials directly from DB with scoped access. |
| A6 | Label-based retention rules can be stored as a JSONB column on `servers` | Code Examples — Migration | JSONB is flexible for `{"daily": 7, "weekly": 4, "monthly": 3}`. The prune task reads this column and evaluates SQL date arithmetic. Low risk — standard PostgreSQL pattern. |

## Open Questions

1. **(RESOLVED) Worker-to-Agent communication path** — Worker has no WebSocket pool.
   - **What we know:** Worker has `reqwest` v0.12 for HTTP calls. API has `NodeClient` with WebSocket management.
   - **Resolution:** Worker calls `POST /api/v1/nodes/:id/commands` on the API server, which proxies to the agent's WebSocket. This is the existing `send_command_with_config` pattern used by `backup_handlers.rs` for `backup.restore`.
   - **Recommendation:** Implement a new API endpoint (or use existing `POST /api/v1/nodes/:node_id/commands`) that accepts `{ command: "backup.start", params: { ... } }` and proxies to the agent.

2. **(RESOLVED) Where does `next_run` get calculated?**
   - **What we know:** The `cron_tasks` table has a `next_run` column. The `find_due_tasks()` query filters `WHERE next_run <= NOW()`.
   - **Resolution:** `next_run` is set when the cron_task is **created or updated** (in the API handler). The Worker's cron evaluation loop only checks for due tasks — it doesn't calculate next_run. The Worker updates `last_run` after dispatching.

3. **(RESOLVED) What about the existing `BackupScheduler` — when does it stop?**
   - **What we know:** `AppContainer::new()` creates `BackupScheduler` and it runs as a background task in the API server.
   - **Resolution:** Remove/disable the `BackupScheduler` startup in the API server. The migration script is the cutover point — after migration runs, only the Worker evaluates cron schedules.

4. **(RESOLVED) Does the Agent's existing `handlers/backup.rs::handle_create` need modification?**
   - **What we know:** The existing `handle_create` uses `docker cp` + `tar` to create backups. D-10 specifies using `agent-backup` crate for archiving.
   - **Resolution:** Create a **new** handler `handle_start` that uses the `agent-backup` crate for archive creation + rusoto_s3 for direct upload. The old `handle_create` remains for backward compatibility but is no longer the canonical backup path.

5. **(RESOLVED) How does the Worker know when the Agent backup completes?**
   - **What we know:** The Agent reports `TaskResult` via WebSocket after backup completes. The API's `node_client` receives this. The Worker doesn't have direct WS access.
   - **Resolution:** After sending the `backup.start` command, the Worker records the backup as `in_progress` and returns success. The Agent's result updates `backup_history` via the API. The frontend (and prune task) poll `backup_history` to detect completion. **Async fire-and-forget** — the Worker does not block waiting for agent completion.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| PostgreSQL | Worker (cron_tasks queries) | ✓ | 16 | — |
| Redis | Worker (job queue) | ✓ | 7 | — |
| Rust/Cargo | Worker build | ✓ | 1.70+ | — |
| `cron` crate | Worker | need to add | 0.15 | — |
| Node.js | Frontend build | ✓ | v20 | — |
| Docker/Podman | Agent (container archive) | ✓ (on agent nodes) | — | — |
| S3-compatible storage | Agent upload | ✓ (configurable) | — | Local storage fallback |

**Missing dependencies with no fallback:**
- None — all required infrastructure exists. Worker needs the `cron` crate added to `Cargo.toml`.

**Missing dependencies with fallback:**
- None.

## Validation Architecture

> Skipped — `workflow.nyquist_validation` is explicitly `false` in `.planning/config.json`.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | JWT auth on all API endpoints (existing `AuthUser` extractor) |
| V4 Access Control | yes | AuthUser checks on backup config CRUD (server ownership), S3 profile CRUD (admin only) |
| V5 Input Validation | yes | Cron expression validation, retention rule bounds, S3 config validation |
| V6 Cryptography | no | Agent uses `sha2` crate for checksums (not cryptographic security). S3 credentials use TLS in transit. |
| V9 Communication | yes | All inter-service communication via internal Docker network (Worker→API→Agent). S3 uploads use HTTPS. |

### Known Threat Patterns for Scheduled Backups

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Backup command injection | Tampering | Command params are serialized JSON, not string-interpolated. Task type is validated at dispatch (must be "backup.start"). |
| Unauthorized backup access | Information Disclosure | `AuthUser` extractor on all backup config endpoints verifies server ownership. S3 profiles require admin role. |
| S3 credential exposure | Information Disclosure | Credentials stored in DB (hashed or plaintext, existing pattern). Passed through internal Docker network only. Future: Agent fetches from DB directly. |
| Backup storage exhaustion | Denial of Service | Retention rules + `max_retained_backups` limit total storage. Prune task runs periodically. |
| Concurrent backup data corruption | Tampering | `has_active_backup()` check before dispatching. Prevents overlapping backups on same server. |
| Stale cron tasks (zombie schedules) | Repudiation | `cron_tasks.enabled` flag prevents execution. Next_run is recalculated on update. Prune task cleans tasks for deleted servers (cascade delete). |

## Sources

### Primary (HIGH confidence)
- [CODE] `api/src/application/services/backup_scheduler.rs` — Existing scheduler pattern (60s tick, cron eval, HashMap)
- [CODE] `api/src/application/services/backup_service.rs` — Existing API-side backup (podman exec + cp, to be replaced)
- [CODE] `api/src/domain/entities/cron_task.rs` — CronTask entity (task_type, schedule_cron, enabled, last_run, next_run)
- [CODE] `api/src/domain/repositories/cron_task_repository.rs` — CronTaskRepository trait (find_due_tasks for Worker)
- [CODE] `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` — Postgres impl (find_due_tasks SQL)
- [CODE] `api/src/bootstrap/container.rs` — AppContainer with BackupService, BackupScheduler, cron_task_repository
- [CODE] `worker/src/main.rs` — Worker entry point (starts Redis connection + JobProcessor)
- [CODE] `worker/src/queue/mod.rs` — JobProcessor (Redis priority queues, process_backup_server stub)
- [CODE] `worker/src/config.rs` — Worker config (REDIS_URL, DATABASE_URL, poll interval)
- [CODE] `agent/solys/src/handlers/backup.rs` — Agent backup handler (handle_create, handle_restore, upload_to_s3)
- [CODE] `agent/solys/src/handlers/mod.rs` — Agent task dispatcher (execute_task, execute_single with backup.create/restore)
- [CODE] `agent/solys/src/agent_connection.rs` — WebSocket connection handler (execute_command → task_type mapping)
- [CODE] `agent/agent-core/crates/agent-backup/src/lib.rs` — Agent backup crate root
- [CODE] `agent/agent-core/crates/agent-backup/src/compression.rs` — zstd/gzip compression utilities
- [CODE] `agent/agent-core/crates/agent-proto/src/task.rs` — Task/TaskResult types for agent communication
- [CODE] `agent/agent-core/crates/agent-proto/src/messages.rs` — WebSocket message types (BackendToAgent, AgentToBackend)
- [CODE] `agent/agent-core/crates/agent-task/src/dispatcher.rs` — Task dispatcher with timeout/retry
- [CODE] `api/src/presentation/handlers/backup_handlers.rs` — Existing backup REST handlers
- [CODE] `api/src/infrastructure/storage/mod.rs` — StorageProvider trait
- [CODE] `api/src/infrastructure/storage/s3_client.rs` — S3 compatible storage client
- [CODE] `api/src/presentation/handlers/settings_handlers.rs` — Existing S3 config settings pattern
- [CODE] `api/src/domain/entities/settings.rs` — S3Config entity
- [CODE] `migration/20260409000006_create_cron_tasks_table.sql` — cron_tasks table schema
- [CODE] `migration/20260302000002_create_backup_history.sql` — backup_history table schema
- [CODE] `migration/20260302000003_add_backup_fields.sql` — server backup columns (backup_cron, auto_backup_enabled, etc.)
- [CODE] `app/src/components/ServerBackups.jsx` — Existing backup history UI
- [CODE] `app/src/hooks/useBackups.js` — Backup API hook
- [CODE] `app/src/pages/ServerDetails.jsx` — Server details page with backup tab + existing config form
- [CODE] `app/src/features/scheduling/ScheduledTasksPage.jsx` — Existing cron task CRUD pattern
- [CODE] `.planning/phases/55-scheduled-backups/55-CONTEXT.md` — All locked decisions

### Secondary (MEDIUM confidence)
- [CODE] `api/src/presentation/handlers/cron_task_handlers.rs` — Existing cron task CRUD handler pattern
- [CODE] `agent/solys/src/agent_connection.rs` — execute_command mapping (lines 402-411, needs backup.start addition)
- [CODE] `agent/solys/src/startup.rs` — Capability registration (BackupCreate, BackupRestore already registered)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all verified from codebase inspection. No new libraries needed beyond adding `cron` crate to Worker.
- Architecture: HIGH — patterns established from existing Worker (job processor), Agent (WebSocket handler, backup.create), and API (BackupScheduler, settings CRUD).
- Pitfalls: HIGH — six specific failure modes identified, each with mitigation. Worker WS access and agent command mapping confirmed by code inspection.

**Research date:** 2026-05-30
**Valid until:** 2026-06-30 (stable codebase, no fast-moving dependencies)
