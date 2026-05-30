# Phase 55: Scheduled Backups - Pattern Map

**Mapped:** 2026-05-30
**Files analyzed:** 29 (14 new, 15 modified)
**Analogs found:** 28 / 29

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `worker/src/cron_eval.rs` (NEW) | utility | polling-loop | `api/src/application/services/backup_scheduler.rs` | role-match |
| `worker/src/main.rs` (MODIFY) | config | startup | `worker/src/main.rs` (existing) | exact |
| `worker/src/config.rs` (MODIFY) | config | static | `worker/src/config.rs` (existing) | exact |
| `worker/src/queue/mod.rs` (MODIFY) | controller | event-driven | `worker/src/queue/mod.rs` (existing stubs) | exact |
| `agent/solys/src/handlers/backup.rs` (EXTEND) | controller | transform | `agent/solys/src/handlers/backup.rs::handle_create` | exact |
| `agent/solys/src/handlers/mod.rs` (MODIFY) | controller | request-response | `agent/solys/src/handlers/mod.rs` (existing dispatch) | exact |
| `agent/solys/src/agent_connection.rs` (MODIFY) | controller | event-driven | `agent/solys/src/agent_connection.rs` (existing mapper) | exact |
| `agent/agent-core/crates/agent-backup/src/lib.rs` (EXTEND) | config | static | `agent-backup/src/lib.rs` (existing) | exact |
| `agent/agent-core/crates/agent-backup/src/archive.rs` (NEW) | service | transform | `agent-backup/src/compression.rs` | role-match |
| `agent/agent-core/crates/agent-backup/src/upload.rs` (NEW) | service | file-I/O | `agent/solys/src/handlers/backup.rs::upload_to_s3` | role-match |
| `api/src/presentation/handlers/backup_config_handlers.rs` (NEW) | controller | CRUD | `api/src/presentation/handlers/backup_handlers.rs` OR `cron_task_handlers.rs` | role-match |
| `api/src/presentation/handlers/settings_handlers.rs` (EXTEND) | controller | CRUD | `api/src/presentation/handlers/settings_handlers.rs` (existing) | exact |
| `api/src/domain/entities/backup_config.rs` (NEW) | model | static | `api/src/domain/entities/cron_task.rs` | role-match |
| `api/src/domain/entities/s3_profile.rs` (NEW) | model | static | `api/src/domain/entities/settings.rs::S3Config` | role-match |
| `api/src/domain/repositories/backup_config_repository.rs` (NEW) | model | static | `api/src/domain/repositories/cron_task_repository.rs` | role-match |
| `api/src/domain/repositories/s3_profile_repository.rs` (NEW) | model | static | `api/src/domain/repositories/cron_task_repository.rs` | role-match |
| `api/src/infrastructure/repositories/postgres_backup_config_repository.rs` (NEW) | service | CRUD | `postgres_cron_task_repository.rs` | role-match |
| `api/src/infrastructure/repositories/postgres_s3_profile_repository.rs` (NEW) | service | CRUD | `postgres_cron_task_repository.rs` | role-match |
| `api/src/application/services/backup_config_service.rs` (NEW) | service | CRUD | `api/src/application/services/backup_service.rs` | role-match |
| `api/src/bootstrap/container.rs` (MODIFY) | config | static | `container.rs` (existing) | exact |
| `migration/*.sql` (3 NEW) | migration | static | `migration/20260302000003_add_backup_fields.sql` | role-match |
| `app/src/components/ServerBackupConfig.jsx` (NEW) | component | request-response | `app/src/components/settings/CloudflareSettings.jsx` | role-match |
| `app/src/components/ServerBackups.jsx` (MODIFY) | component | request-response | existing `ServerBackups.jsx` | exact |
| `app/src/hooks/useBackupConfig.js` (NEW) | hook | request-response | `app/src/hooks/useBackups.js` | role-match |
| `app/src/api/backupConfig.js` (NEW) | utility | request-response | `app/src/lib/api.js::cloudflareApi` | role-match |
| `app/src/pages/settings/SettingsPage.jsx` (MODIFY) | page | request-response | existing settings page | exact |

## Pattern Assignments

---

### `worker/src/cron_eval.rs` (NEW — utility, polling-loop)

**Analog:** `api/src/application/services/backup_scheduler.rs` (lines 42-151)

**Why:** Existing `BackupScheduler::run()` demonstrates the 60-second tick → query servers → parse cron → fire backup pattern. The new Worker cron evaluation loop does the same but polls `cron_tasks` table and dispatches to Redis instead of calling `BackupService::trigger_backup`.

**Imports pattern** (backup_scheduler.rs lines 1-11):
```rust
use std::collections::HashMap;
use std::str::FromStr;
use chrono::Utc;
use cron::Schedule;
use uuid::Uuid;
```

**Core loop pattern** (backup_scheduler.rs lines 42-55):
```rust
pub async fn run(self: Arc<Self>) {
    tracing::info!("BackupScheduler started — checking every 60 seconds");
    let mut last_triggered: HashMap<Uuid, chrono::DateTime<Utc>> = HashMap::new();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        if let Err(e) = self.tick(&mut last_triggered).await {
            tracing::error!("BackupScheduler tick error: {}", e);
        }
    }
}
```

**Cron evaluation pattern** (backup_scheduler.rs lines 73-95):
```rust
let schedule = match Schedule::from_str(&cron_str) {
    Ok(s) => s,
    Err(e) => {
        tracing::warn!("Invalid cron '{}': {}", cron_str, e);
        continue;
    }
};
let should_fire = schedule
    .after(&reference)
    .take(1)
    .any(|next| next <= now);
```

**Worker analog for main.rs startup pattern** (worker/src/main.rs lines 18-28):
```rust
let config = config::Config::new()?;
let redis = redis::Client::open(config.redis_url.as_str())?
    .get_multiplexed_async_connection()
    .await?;
let mut processor = queue::JobProcessor::new(redis);
processor.run().await;
```

**Redis job dispatch pattern** (worker/src/queue/mod.rs lines 49-68):
```rust
async fn dequeue(&mut self) -> Option<Job> {
    let priorities = ["high", "normal", "low"];
    for priority in priorities {
        let queue_key = format!("{}:{}", QUEUE_PREFIX, priority);
        let result: Option<(String, f64)> = self.redis.zpopmin(&queue_key, 1).await.ok()?;
        if let Some((job_id_str, _)) = result {
            let job_key = format!("job:{}", job_id_str);
            let job_data: Option<String> = self.redis.hget(&job_key, "data").await.ok()?;
            if let Some(data) = job_data {
                if let Ok(job) = serde_json::from_str::<Job>(&data) {
                    return Some(job);
                }
            }
        }
    }
    None
}
```

---

### `worker/src/main.rs` (MODIFY — config, startup)

**Analog:** Existing `worker/src/main.rs` (all 31 lines)

**Imports pattern** (existing lines 1-8):
```rust
use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

mod config;
mod queue;
mod agent;
mod webhook;
```

**Startup pattern** (existing lines 10-28):
```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("RUST_LOG").init();
    info!("Starting worker service...");
    let config = config::Config::new()?;
    let redis = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_async_connection().await?;
    let mut processor = queue::JobProcessor::new(redis);
    processor.run().await;
    Ok(())
}
```

**Add:** sqlx pool creation + `tokio::spawn` for cron evaluation loop (see RESEARCH.md lines 781-813 for exact pattern).

---

### `worker/src/config.rs` (MODIFY — config, static)

**Analog:** Existing `worker/src/config.rs` (all 54 lines)

**Pattern** (existing lines 5-13):
```rust
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub worker_id: String,
    pub worker_poll_interval_ms: u64,
    pub worker_concurrency: u32,
    pub jwt_secret: String,
    pub app_url: String,
}
```

**Note:** `database_url` and `app_url` already exist. Add `api_base_url` field for Worker→API HTTP calls.

---

### `worker/src/queue/mod.rs` (MODIFY — controller, event-driven)

**Analog:** Existing `worker/src/queue/mod.rs` (all 120 lines)

**Job dispatch pattern** (existing lines 71-90):
```rust
async fn process_job(&mut self, job: Job) {
    let result = match job.job_type.as_str() {
        "create_server" => self.process_create_server(job).await,
        "delete_server" => self.process_delete_server(job).await,
        "start_server" => self.process_start_server(job).await,
        "stop_server" => self.process_stop_server(job).await,
        "backup_server" => self.process_backup_server(job).await,  // EXISTING STUB
        _ => { tracing::warn!("Unknown job type: {}", job_type); Ok(()) }
    };
}
```

**Existing stub to fill** (existing lines 112-115):
```rust
async fn process_backup_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Processing backup_server job: {}", job.job_id);
    Ok(())
}
```

**Pattern for filling the stub** — use `reqwest` to call API's node commands endpoint:
```rust
let api_url = format!("{}/api/v1/nodes/{}/commands", 
    std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://api:3000".to_string()),
    node_id);
let client = reqwest::Client::new();
let response = client.post(&api_url).json(&body).send().await?;
```

See RESEARCH.md patterns 1-2 (lines 287-449) for detailed implementation.

---

### `agent/solys/src/handlers/backup.rs` (EXTEND — controller, transform)

**Analog:** Existing `handle_create` function (lines 41-173)

**Imports pattern** (existing lines 1-15):
```rust
use std::path::PathBuf;
use std::str::FromStr;
use agent_proto::Task;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{info, warn};
use crate::task_state::TASK_STATE_TRACKER;
```

**Handler pattern** (existing lines 41-48):
```rust
pub async fn handle_create(task: Task) -> Result<serde_json::Value> {
    let payload: BackupCreatePayload = serde_json::from_value(task.payload)?;
    info!(server_id = %payload.server_id, container_id = %payload.container_id, "Creating backup");
    // ... implementation ...
    Ok(serde_json::to_value(output)?)
}
```

**Payload struct pattern** (existing lines 17-23):
```rust
#[derive(Debug, Deserialize)]
pub struct BackupStartPayload {
    pub server_id: uuid::Uuid,
    pub container_id: String,
    pub backup_id: uuid::Uuid,
    pub file_name: String,
    pub provider: String,
    // ... additional fields ...
}
```

**S3 upload pattern** (existing lines 256-280):
```rust
async fn upload_to_s3(bucket: &str, region: &str, key: &str, file_path: &PathBuf) -> Result<String> {
    use rusoto_s3::{S3, S3Client, PutObjectRequest};
    use rusoto_core::Region;
    let region = Region::from_str(region).unwrap_or_else(|_| Region::UsEast1);
    let client = S3Client::new(region);
    let file_data = tokio::fs::read(file_path).await.context("Failed to read backup file")?;
    let request = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(file_data.into()),
        content_type: Some("application/gzip".to_string()),
        ..Default::default()
    };
    client.put_object(request).await.context("Failed to upload to S3")?;
    Ok(format!("s3://{}/{}", bucket, key))
}
```

---

### `agent/solys/src/handlers/mod.rs` (MODIFY — controller, request-response)

**Analog:** Existing dispatch pattern (lines 109-165)

**Dispatch match pattern** (existing lines 117-154):
```rust
let future = async {
    match task_type.as_str() {
        "server.create" => runtime::handle_create(task.clone(), runtime).await,
        "server.start" => runtime::handle_start(task.clone(), runtime).await,
        // ... existing ...
        "backup.create" => backup::handle_create(task.clone()).await,
        "backup.restore" => backup::handle_restore(task.clone()).await,
        // NEW:
        "backup.start" => backup::handle_start(task.clone()).await,
        _ => Err(anyhow::anyhow!("Unknown task type: {}", task_type)),
    }
};
```

**Task config pattern** (existing lines 219-225):
```rust
"backup.start" => TaskConfig {   // NEW (copy backup.create config)
    timeout: Duration::from_secs(600),
    max_retries: 0,
    retry_delay_ms: 0,
    max_retry_delay_ms: 0,
    backoff_multiplier: 1.0,
},
```

---

### `agent/solys/src/agent_connection.rs` (MODIFY — controller, event-driven)

**Analog:** Existing command mapper (lines 402-411)

**Existing mapper** (lines 402-411):
```rust
let task_type = match command.as_str() {
    "create" => "server.create",
    "start" => "server.start",
    "stop" => "server.stop",
    "restart" => "server.restart",
    "delete" => "server.delete",
    "logs" => "server.logs",
    "command" => "server.command",
    _ => "unknown",
};
```

**Add:** `"backup.start" => "backup.start"` and `"backup.restore" => "backup.restore"` to the match.

---

### `agent/agent-core/crates/agent-backup/src/lib.rs` (EXTEND — config, static)

**Analog:** Existing file (all 5 lines)
```rust
pub mod compression;
pub use compression::*;
```

**Extend with:** `pub mod archive;` and `pub mod upload;`

---

### `agent/agent-core/crates/agent-backup/src/archive.rs` (NEW — service, transform)

**Analog:** `agent-backup/src/compression.rs` (lines 1-28)

**Pattern** (compression.rs lines 3-8):
```rust
use std::io::{Read, Write};

pub fn compress_zstd(input: &[u8], level: i32) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = zstd::Encoder::new(Vec::new(), level)?;
    encoder.write_all(input)?;
    encoder.finish()
}
```

**Follow same pattern** for archive creation — expose `create_tar_archive(source_dir: &Path, dest: &Path, compression: CompressionFormat) -> Result<u64>` using `tar` + `zstd`/`gzip` crates.

---

### `agent/agent-core/crates/agent-backup/src/upload.rs` (NEW — service, file-I/O)

**Analog:** `agent/solys/src/handlers/backup.rs::upload_to_s3` (lines 256-280)

**Pattern** (existing lines 256-280):
```rust
async fn upload_to_s3(bucket: &str, region: &str, key: &str, file_path: &PathBuf) -> Result<String> {
    use rusoto_s3::{S3, S3Client, PutObjectRequest};
    use rusoto_core::Region;
    let region = Region::from_str(region).unwrap_or_else(|_| Region::UsEast1);
    let client = S3Client::new(region);
    // ...
}
```

**Extend with:** configurable endpoint support using `StaticProvider` + `Region::Custom`.

```rust
let credentials = StaticProvider::new(
    access_key.to_string(), secret_key.to_string(), None, None, None,
);
let region = Region::Custom {
    name: region_name.to_string(),
    endpoint: endpoint.to_string(),
};
let client = S3Client::new_with(HttpClient::new()?, credentials, region);
```

---

### `api/src/presentation/handlers/backup_config_handlers.rs` (NEW — controller, CRUD)

**Analog:** `api/src/presentation/handlers/cron_task_handlers.rs` (all 181 lines)

**Import pattern** (cron_task_handlers.rs lines 1-16):
```rust
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post, patch, delete},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::entities::cron_task::{CronTask, CreateCronTaskRequest, UpdateCronTaskRequest};
use crate::domain::auth::middleware::AuthUser;
use crate::domain::repositories::cron_task_repository::CronTaskRepository;
use crate::presentation::routes::api_routes::ApiState;
use crate::presentation::responses::api_response::ApiResponse;
```

**CRUD handler pattern** (cron_task_handlers.rs lines 25-43):
```rust
pub async fn list_tasks(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, String> {
    let server = state.server_repository.find_by_id(&server_id)
        .await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;
    if server.user_id != auth_user.tenant_id {
        return Err("Access denied".to_string());
    }
    let tasks = state.cron_task_repository.find_by_server_id(&server_id)
        .await.map_err(|e| e.to_string())?;
    Ok(Json(ApiResponse::success(tasks)))
}
```

**BACKUP_CONFIG specific endpoints:**
- `GET /servers/:id/backup-config` — read config (uses `backup_config_repository` or reads from `servers` table directly)
- `PUT /servers/:id/backup-config` — update config (writes to both `cron_tasks` + `servers` table per D-03)

---

### `api/src/presentation/handlers/settings_handlers.rs` (EXTEND — controller, CRUD)

**Analog:** Existing `settings_handlers.rs` — S3 config pattern (lines 13-65)

**Existing GET pattern** (lines 13-34):
```rust
pub async fn get_s3_config(
    State(container): State<ApiState>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match container.settings_repository.get_s3_config().await {
        Ok(config) => Ok((StatusCode::OK, Json(json!({ "success": true, "data": { ... } })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, ... })))),
    }
}
```

**Existing SAVE pattern** (lines 36-65):
```rust
pub async fn save_s3_config(
    State(container): State<ApiState>,
    Json(payload): Json<S3Config>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let config = if payload.secret_key.is_empty() {
        let existing = container.settings_repository.get_s3_config().await.unwrap_or_default();
        S3Config { secret_key: existing.secret_key, ..payload }
    } else { payload };
    match container.settings_repository.save_s3_config(&config).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({ "success": true, "data": null })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ ... })))),
    }
}
```

**Extend with:** `list_s3_profiles`, `create_s3_profile`, `update_s3_profile`, `delete_s3_profile` following the same return type pattern.

---

### `api/src/domain/entities/backup_config.rs` (NEW — model, static)

**Analog:** `api/src/domain/entities/cron_task.rs` (all 34 lines)

**Entity pattern** (cron_task.rs lines 1-18):
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronTask {
    pub id: Uuid,
    pub server_id: Uuid,
    // ... fields ...
}
```

**BackupConfig entity fields:** server_id, auto_backup_enabled, schedule_cron, backup_provider, max_retained_backups, retention_rules (JSON), retention_mode, s3_profile_id

---

### `api/src/domain/entities/s3_profile.rs` (NEW — model, static)

**Analog:** `api/src/domain/entities/settings.rs` — `S3Config` (all 21 lines)

**Entity pattern** (settings.rs lines 3-11):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}
```

**S3Profile entity:** id, name, endpoint, region, bucket, access_key, secret_key, is_default, created_at, updated_at

---

### `api/src/domain/repositories/backup_config_repository.rs` (NEW — model, static)

**Analog:** `api/src/domain/repositories/cron_task_repository.rs` (all 14 lines)

**Trait pattern** (cron_task_repository.rs lines 1-14):
```rust
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::cron_task::CronTask;
use anyhow::Result;

#[async_trait]
pub trait CronTaskRepository: Send + Sync {
    async fn find_by_server_id(&self, server_id: &Uuid) -> Result<Vec<CronTask>>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<CronTask>>;
    async fn create(&self, task: &CronTask) -> Result<()>;
    async fn update(&self, task: &CronTask) -> Result<()>;
    async fn delete(&self, id: &Uuid) -> Result<()>;
    async fn find_due_tasks(&self) -> Result<Vec<CronTask>>;
}
```

**BackupConfigRepository methods:** `find_by_server_id`, `save` (upsert), `delete`

---

### `api/src/domain/repositories/s3_profile_repository.rs` (NEW — model, static)

**Analog:** Same `cron_task_repository.rs` pattern

**Methods:** `list_all`, `find_by_id`, `find_by_name`, `create`, `update`, `delete`

---

### `api/src/infrastructure/repositories/postgres_backup_config_repository.rs` (NEW — service, CRUD)

**Analog:** `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` (all 167 lines)

**Repository impl pattern** (postgres_cron_task_repository.rs lines 10-18):
```rust
pub struct PostgresCronTaskRepository {
    pool: Pool<Postgres>,
}
impl PostgresCronTaskRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}
```

**SQL query pattern** (postgres_cron_task_repository.rs lines 22-43):
```rust
async fn find_by_server_id(&self, server_id: &Uuid) -> Result<Vec<CronTask>> {
    let rows = sqlx::query(r#"SELECT ... FROM cron_tasks WHERE server_id = $1 ORDER BY created_at DESC"#)
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch cron tasks by server_id")?;
    let mut tasks = Vec::new();
    for row in rows { tasks.push(self.row_to_task(row)?); }
    Ok(tasks)
}
```

**Row mapping pattern** (postgres_cron_task_repository.rs lines 147-166):
```rust
fn row_to_task(&self, row: sqlx::postgres::PgRow) -> Result<CronTask> {
    Ok(CronTask {
        id: row.try_get("id")?,
        server_id: row.try_get("server_id")?,
        // ... field mappings ...
    })
}
```

**CRUD SQL** — same `INSERT`, `UPDATE`, `DELETE`, `SELECT` pattern for the `servers` table columns and `s3_profiles` table.

---

### `api/src/infrastructure/repositories/postgres_s3_profile_repository.rs` (NEW — service, CRUD)

**Analog:** Same `postgres_cron_task_repository.rs` pattern with `s3_profiles` table queries.

---

### `api/src/application/services/backup_config_service.rs` (NEW — service, CRUD)

**Analog:** `api/src/application/services/backup_service.rs` (lines 15-127)

**Service pattern** (backup_service.rs lines 17-48):
```rust
pub struct BackupService<S, B>
where S: ServerRepository + ?Sized, B: BackupRepository + ?Sized,
{
    server_repository: Arc<S>,
    backup_repository: Arc<B>,
    // ... dependencies ...
}

impl<S, B> BackupService<S, B>
where S: ServerRepository + ?Sized + Send + Sync + 'static,
      B: BackupRepository + ?Sized + Send + Sync + 'static,
{
    pub fn new(...) -> Self { Self { ... } }
    pub async fn trigger_backup(&self, server_id: Uuid) -> Result<BackupRecord> { ... }
}
```

**BackupConfigService:** Manages dual-write to `cron_tasks` + `servers` table, validates cron expressions, copies `retention_rules`.

---

### `api/src/bootstrap/container.rs` (MODIFY — config, static)

**Analog:** Existing `container.rs` (lines 69-361)

**Registration pattern** (existing lines 271-292):
```rust
// Backup infrastructure
let backup_repository_concrete = Arc::new(PostgresBackupRepository::new(pool.clone()));
let backup_repo: Arc<dyn BackupRepository> = backup_repository_concrete;
// ...
let backup_service = Arc::new(BackupService::new(repo.clone(), backup_repo.clone(), ...));
Self {
    // ...
    backup_service,
    // ...
}
```

**Add registration for:** `BackupConfigService` (or skip service layer and use repo directly from handler), `S3ProfileRepository`.

---

### `migration/20260530000001_add_retention_rules.sql` (NEW — migration, static)

**Analog:** `migration/20260302000003_add_backup_fields.sql` (lines 1-6)

**Pattern**:
```sql
-- Description of migration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS column_name TYPE DEFAULT default_value;
```

**New columns:** `retention_rules JSONB`, `retention_mode TEXT`, `s3_profile_id UUID REFERENCES s3_profiles(id)`.

---

### `migration/20260530000002_create_s3_profiles.sql` (NEW — migration, static)

**Analog:** `migration/20260302000002_create_backup_history.sql`

**Pattern**:
```sql
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
```

---

### `migration/20260530000003_migrate_backup_cron.sql` (NEW — migration, static)

**One-time data migration**:
```sql
INSERT INTO cron_tasks (id, server_id, user_id, task_type, schedule_cron, command, enabled, last_run, created_at, updated_at)
SELECT gen_random_uuid(), s.id, COALESCE(s.user_id, (SELECT id FROM users LIMIT 1)),
       'backup', s.backup_cron, NULL, s.auto_backup_enabled, NULL, NOW(), NOW()
FROM servers s
WHERE s.backup_cron IS NOT NULL AND s.backup_cron != ''
AND NOT EXISTS (SELECT 1 FROM cron_tasks ct WHERE ct.server_id = s.id AND ct.task_type = 'backup');
```

---

### `app/src/components/ServerBackupConfig.jsx` (NEW — component, request-response)

**Analog:** `app/src/components/settings/CloudflareSettings.jsx` (lines 1-60)

**Component pattern** (CloudflareSettings.jsx lines 5-44):
```jsx
import { useState, useEffect } from 'react'
import { useUIStore } from '../../store/uiStore'

export default function CloudflareSettings() {
  const { addToast } = useUIStore()
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)

  useEffect(() => { loadConfig() }, [])

  const loadConfig = async () => {
    setLoading(true)
    try {
      const data = await api.getConfig()
      if (data) { /* set state */ }
    } catch (err) { console.error(err) }
    finally { setLoading(false) }
  }

  const handleSave = async (e) => {
    e.preventDefault()
    setSaving(true)
    try {
      await api.saveConfig(data)
      addToast({ type: 'success', message: 'Saved!' })
    } catch (err) { addToast({ type: 'error', message: err.message }) }
    finally { setSaving(false) }
  }
  // ... JSX ...
}
```

**ServerBackupConfig specifics:** Renders inside Backup tab ABOVE history table. Contains:
- Auto-backup toggle
- Schedule preset dropdown + custom cron input
- Retention rules (presets for daily/weekly/monthly counts)
- Storage provider selector (local / S3 profile)
- Save button

**Existing backup config form in ServerDetails.jsx** (lines 384-531) — move/refactor the JSX from the Settings tab into the new component.

---

### `app/src/components/ServerBackups.jsx` (MODIFY — component, request-response)

**Analog:** Existing file (all 207 lines)

**Modification:** Add `import ServerBackupConfig from './ServerBackupConfig';` at top, then render `<ServerBackupConfig serverId={serverId} />` above the existing history table content (above line 100).

---

### `app/src/hooks/useBackupConfig.js` (NEW — hook, request-response)

**Analog:** `app/src/hooks/useBackups.js` (all 59 lines)

**Hook pattern** (useBackups.js lines 1-58):
```javascript
import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

export function useBackups(serverId) {
    const [backups, setBackups] = useState([]);
    const [loading, setLoading] = useState(true);
    // ...
    const refresh = useCallback(async () => {
        try { const data = await fetchApi(`/servers/${serverId}/backups`); setBackups(data || []); }
        catch (err) { console.error(err); }
        finally { setLoading(false); }
    }, [serverId]);
    useEffect(() => { refresh(); }, [refresh]);

    const triggerBackup = useCallback(async () => { ... }, [serverId, refresh]);
    return { backups, loading, triggerBackup, /* ... */, refresh };
}
```

**useBackupConfig hook:** `config`, `loading`, `saving`, `saveConfig`, `refresh` — GET/PUT to `/servers/:id/backup-config`.

---

### `app/src/api/backupConfig.js` (NEW — utility, request-response)

**Analog:** `app/src/lib/api.js` `cloudflareApi` (lines 165-168):
```javascript
export const cloudflareApi = {
  getConfig: () => api.get('/settings/cloudflare'),
  saveConfig: (data) => api.put('/settings/cloudflare', data),
  testConnection: () => api.post('/settings/cloudflare/test'),
}
```

**backupConfig API**:
```javascript
export const backupConfigApi = {
  get: (serverId) => fetchApi(`/servers/${serverId}/backup-config`),
  save: (serverId, data) => fetchApi(`/servers/${serverId}/backup-config`, { method: 'PUT', body: JSON.stringify(data) }),
}
```

---

### `app/src/pages/settings/SettingsPage.jsx` (MODIFY — page, request-response)

**Analog:** Existing settings page with CloudflareSettings section pattern.

**Tab navigation pattern** (SettingsPage.jsx e.g., lines 15-16):
```javascript
const [activeTab, setActiveTab] = useState('profile');
```

**Add S3 Profile management section** — renders list of profiles with add/edit/delete functionality. Use the same section/card design as CloudflareSettings.

---

## Shared Patterns

### Authentication
**API Handlers:** Use `AuthUser` extractor (from `crate::domain::auth::middleware::AuthUser`) for ownership checks:
```rust
auth_user: AuthUser,
// then:
if server.user_id != auth_user.tenant_id { return Err("Access denied".to_string()); }
```
**Source:** `cron_task_handlers.rs` lines 28-37.

**Settings (admin-only):** Use `auth_user.is_admin()` check:
```rust
if !user.is_admin() { return Err((StatusCode::FORBIDDEN, ...)); }
```
**Source:** `settings_handlers.rs` lines 71-77.

### Error Handling
**API Handlers (AppError):**
```rust
use crate::shared::errors::app_error::AppError;
// Return Result<impl IntoResponse, AppError>
Err(AppError::NotFound)
Err(AppError::BadRequest("msg".into()))
Err(AppError::InternalError(anyhow::anyhow!(e)))
```
**Source:** `backup_handlers.rs` lines 33-36, `app_error.rs` lines 9-65.

**API Handlers (legacy string errors):**
```rust
// Alternative: return Result<impl IntoResponse, String>
Err("Server not found".to_string())
```
**Source:** `cron_task_handlers.rs` lines 25-44.

**Agent handlers (anyhow::Result):**
```rust
use anyhow::{Result, Context};
// Return Result<serde_json::Value>
```
**Source:** `handlers/backup.rs` lines 9, 41.

### Worker Patterns
**Job dispatch via Redis:** `zpopmin` + `hget` pattern in `queue/mod.rs` lines 49-68.
**Job enqueue:** `HSET job:{id} data ...` + `ZADD queue:jobs:normal ...` pattern from RESEARCH.md lines 344-359.

### API Response Format
```rust
Ok(Json(ApiResponse::success(data)))
// or for errors:
ApiResponse::<()>::error("ERROR_CODE", "message")
```
**Source:** `api_response.rs` lines 196-213.

### Frontend Component Styling
**Glass panel pattern** (ServerBackups.jsx lines 99-204, ServerDetails.jsx lines 328-531):
- Container: `max-w-5xl` or `max-w-2xl` with `glass-panel p-6`
- Inputs: `w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border ...`
- Buttons: `px-5 py-2.5 rounded-xl text-sm font-bold ... bg-[var(--color-cosmic-cyan)]/10 ...`
- Toggle: `w-12 h-6 rounded-full` with cyan/border backgrounds

### API Route Registration
Routes are registered in `api/src/presentation/routes/api_routes.rs` using either:
- `.nest("/api/v1/servers", ServerHandlers::router(state.clone()))` for handler groups
- `.route("/api/v1/settings/s3", get(...).put(...))` for individual handlers

**For backup_config_handlers:** Add `.route("/api/v1/servers/:server_id/backup-config", get(...).put(...))` to `api_routes.rs`.

**For S3 profiles:** Add `.route("/api/v1/settings/s3/profiles", get(list_s3_profiles).post(create_s3_profile))` and `.route("/api/v1/settings/s3/profiles/:id", put(...).delete(...))`.

### Agent Task Dispatch
Add `"backup.start" => "backup.start"` to both:
1. `agent_connection.rs` command mapper (line ~410)
2. `handlers/mod.rs` `execute_single` match (lines 117-154) + `get_task_config` (lines 175-276)

### Cargo Dependency Addition
**Worker Cargo.toml:** Add `cron = "0.15"` and `sqlx` (postgres feature — verify if already present).

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| N/A | — | — | All files have a close analog in the existing codebase |

## Metadata

**Analog search scope:** `worker/src/`, `agent/solys/src/`, `agent/agent-core/crates/agent-backup/src/`, `api/src/`, `app/src/`, `migration/`
**Files scanned:** ~40 key source files
**Pattern extraction date:** 2026-05-30
