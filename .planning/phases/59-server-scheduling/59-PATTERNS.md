# Phase 59: Server Scheduling - Pattern Map

**Mapped:** 2026-05-31
**Files analyzed:** 12 (7 modified, 3 extended, 2 new)
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `worker/src/cron_eval.rs` (EXTEND) | controller | polling-loop | `worker/src/cron_eval.rs` (existing) | exact |
| `worker/src/queue/mod.rs` (EXTEND) | controller | event-driven | `worker/src/queue/mod.rs` (existing, `process_backup_server`) | exact |
| `api/src/domain/entities/cron_task.rs` (EXTEND) | model | static | `cron_task.rs` (existing entity) | exact |
| `api/src/presentation/handlers/cron_task_handlers.rs` (EXTEND) | controller | CRUD | `cron_task_handlers.rs` (existing handlers) | exact |
| `api/src/presentation/handlers/node_handlers.rs` (EXTEND) | controller | request-response | `node_handlers.rs::poll_node_commands` (existing handler) | exact |
| `api/src/presentation/routes/api_routes.rs` (EXTEND) | route | static | `api_routes.rs` (existing route pattern) | exact |
| `api/src/domain/repositories/cron_task_repository.rs` (EXTEND) | model | static | `cron_task_repository.rs` (existing trait) | exact |
| `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` (EXTEND) | service | CRUD | `postgres_cron_task_repository.rs` (existing) | exact |
| `api/src/migrations/20260531000001_add_cron_task_columns.sql` (NEW) | migration | static | `20260409000006_create_cron_tasks_table.sql` | role-match |
| `app/src/pages/ServerDetails.jsx` (EXTEND) | page | request-response | Restart Policy section (lines 509-635) + Sleep/Wake section (lines 446-507) | exact |
| `app/src/hooks/useScheduledActions.js` (NEW) | hook | CRUD | `app/src/hooks/useBackups.js` | role-match |
| `app/src/lib/api.js` (EXTEND) | utility | request-response | `app/src/lib/api.js` `tasksApi` pattern (lines 159-161) | exact |

## Pattern Assignments

---

### `worker/src/cron_eval.rs` (EXTEND — controller, polling-loop)

**Analog:** Existing `worker/src/cron_eval.rs` (all 101 lines)

**Imports pattern** (lines 1-10):
```rust
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
```

**Core loop pattern** (lines 14-25):
```rust
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
```

**Existing backup-only SQL query** (lines 32-45):
```rust
let rows = sqlx::query(
    r#"SELECT id, server_id, user_id, task_type, schedule_cron, command, enabled,
           last_run, next_run, created_at, updated_at
    FROM cron_tasks
    WHERE enabled = true
      AND task_type = 'backup'
      AND next_run <= NOW()
    ORDER BY next_run ASC
    LIMIT 50"#
)
.fetch_all(pool)
.await?;
```

**Existing Redis job dispatch pattern** (lines 53-83):
```rust
let job_id = Uuid::new_v4();
let job_payload = json!({
    "cron_task_id": cron_task_id,
    "server_id": server_id,
    "user_id": user_id,
});
let job_key = format!("job:{}", job_id);
let queue_key = "queue:jobs:normal";
redis::cmd("HSET")
    .arg(&job_key)
    .arg("data")
    .arg(serde_json::to_string(&serde_json::json!({
        "job_id": job_id,
        "job_type": "backup_server",
        "payload": job_payload,
        "user_id": user_id,
        "priority": 0,
        "created_at": Utc::now().timestamp(),
    }))?)
    .query_async::<_, ()>(redis)
    .await?;
redis::cmd("ZADD")
    .arg(queue_key)
    .arg(Utc::now().timestamp() as f64)
    .arg(job_id.to_string())
    .query_async::<_, ()>(redis)
    .await?;
```

**Existing last_run update** (lines 87-92):
```rust
sqlx::query(
    "UPDATE cron_tasks SET last_run = NOW(), updated_at = NOW() WHERE id = $1"
)
.bind(cron_task_id)
.execute(pool)
.await?;
```

**KEY EXTENSIONS needed:**
1. Add `use chrono_tz::Tz; use std::str::FromStr;` to imports (new dependency)
2. Replace SQL query — remove `AND task_type = 'backup'` filter, add `timezone` column to SELECT
3. Add timezone-aware cron evaluation: convert `Utc::now()` to schedule's timezone before checking cron
4. Map `task_type` to job type: `backup→backup_server`, `start→scheduled_start`, `stop→scheduled_stop`, `restart→scheduled_restart`, `sleep→scheduled_sleep`
5. Include `timezone` and `task_type` in job payload

---

### `worker/src/queue/mod.rs` (EXTEND — controller, event-driven)

**Analog:** Existing `worker/src/queue/mod.rs` (all 217 lines)

**Imports pattern** (lines 1-5):
```rust
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;
```

**Job dispatch match pattern** (lines 75-89):
```rust
async fn process_job(&mut self, job: Job) {
    let job_id = job.job_id.clone();
    let job_type = job.job_type.clone();
    let result = match job_type.as_str() {
        "create_server" => self.process_create_server(job).await,
        "delete_server" => self.process_delete_server(job).await,
        "start_server" => self.process_start_server(job).await,
        "stop_server" => self.process_stop_server(job).await,
        "backup_server" => self.process_backup_server(job).await,
        _ => {
            tracing::warn!("Unknown job type: {}", job_type);
            Ok(())
        }
    };
    if let Err(e) = result {
        tracing::error!("Job {} failed: {}", job_id, e);
    }
}
```

**Existing `process_backup_server` handler pattern** (lines 116-211):
```rust
async fn process_backup_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
    let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

    // 1. Check for active operation (Pitfall prevention)
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM backup_history WHERE server_id = $1 AND status = 'in_progress'"
    )
    .bind(server_id)
    .fetch_one(&self.pool)
    .await?;
    if active > 0 {
        tracing::warn!("Backup already in progress for server {}, skipping", server_id);
        return Ok(());
    }

    // 2. Get server details
    let server_row = sqlx::query(
        r#"SELECT s.id, s.node_id, s.name AS server_name, s.backup_provider,
               c.container_name
        FROM servers s
        LEFT JOIN containers c ON c.server_id = s.id
        WHERE s.id = $1"#
    )
    .bind(server_id)
    .fetch_optional(&self.pool)
    .await?
    .ok_or("Server not found")?;

    let node_id: Uuid = server_row.try_get("node_id")?;

    // 3. Send command to agent via API proxy
    let api_base_url = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://api:3000".to_string());
    let api_url = format!("{}/api/v1/nodes/{}/commands", api_base_url, node_id);

    let body = serde_json::json!({
        "command": "backup.start",
        "server_id": server_id,
        "params": { "container_name": container_name, /* ... */ }
    });

    let client = reqwest::Client::new();
    let response = client.post(&api_url).json(&body).send().await;

    match response {
        Ok(resp) => {
            tracing::info!("Backup dispatched: server={} node={} status={}", server_id, node_id, resp.status());
        }
        Err(e) => {
            tracing::error!("Failed to dispatch for server {}: {}", server_id, e);
        }
    }
    Ok(())
}
```

**KEY EXTENSIONS needed:**
1. Add `"scheduled_start" => self.process_scheduled_start(job).await` etc. to the match in `process_job`
2. Add `process_scheduled_start`, `process_scheduled_stop`, `process_scheduled_restart`, `process_scheduled_sleep` handlers
3. Each handler follows the same pattern as `process_backup_server`: extract payload → check server state → dispatch via API proxy → update cron_tasks
4. Sleep handler (D-07): check `status == 'stopped' && auto_wake == true` to skip if already sleeping
5. Restart handler (D-08): check `restart_count > 0` and wait for auto-restart cooldown
6. Add helper methods: `update_cron_task_result`, `handle_cron_failure`, `is_run_once`, `disable_cron_task`

---

### `api/src/domain/entities/cron_task.rs` (EXTEND — model, static)

**Analog:** Existing `api/src/domain/entities/cron_task.rs` (all 34 lines)

**Existing entity pattern** (lines 1-18):
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronTask {
    pub id: Uuid,
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub task_type: String, // "backup", "restart", "stop", "command"
    pub schedule_cron: String,
    pub command: Option<String>,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Existing DTOs** (lines 20-34):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCronTaskRequest {
    pub task_type: String,
    pub schedule_cron: String,
    pub command: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCronTaskRequest {
    pub task_type: Option<String>,
    pub schedule_cron: Option<String>,
    pub command: Option<String>,
    pub enabled: Option<bool>,
}
```

**EXTEND:**
1. **Entity:** Add `pub timezone: String,` (default `"UTC"`), `pub run_once: bool,` (default `false`), `pub last_result: Option<String>,`, `pub last_error: Option<String>,`
2. **CreateCronTaskRequest:** Add `pub timezone: Option<String>,`, `pub run_once: Option<bool>,`
3. **UpdateCronTaskRequest:** Add `pub timezone: Option<String>,`, `pub run_once: Option<bool>,`
4. Update `task_type` doc comment to include `"start"` and `"sleep"`

---

### `api/src/presentation/handlers/cron_task_handlers.rs` (EXTEND — controller, CRUD)

**Analog:** Existing `api/src/presentation/handlers/cron_task_handlers.rs` (all 181 lines)

**Imports pattern** (lines 1-16):
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

**CRUD ownership check pattern** (lines 25-43):
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

**Create handler task_type validation** (lines 61-67):
```rust
if !["backup", "restart", "stop", "command"].contains(&req.task_type.as_str()) {
    return Err("Invalid task_type. Must be one of: backup, restart, stop, command".to_string());
}
if req.task_type == "command" && req.command.is_none() {
    return Err("Command is required for task_type 'command'".to_string());
}
```

**Existing create_task CronTask construction** (lines 69-81):
```rust
let task = CronTask {
    id: Uuid::new_v4(),
    server_id,
    user_id: auth_user.tenant_id,
    task_type: req.task_type,
    schedule_cron: req.schedule_cron,
    command: req.command,
    enabled: req.enabled.unwrap_or(true),
    last_run: None,
    next_run: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
};
```

**Update handler conditional field pattern** (lines 105-126):
```rust
let mut updated_task = task.clone();
if let Some(task_type) = req.task_type {
    if !["backup", "restart", "stop", "command"].contains(&task_type.as_str()) {
        return Err("Invalid task_type".to_string());
    }
    updated_task.task_type = task_type;
}
if let Some(schedule_cron) = req.schedule_cron {
    updated_task.schedule_cron = schedule_cron;
}
if let Some(command) = req.command {
    updated_task.command = Some(command);
}
if let Some(enabled) = req.enabled {
    updated_task.enabled = enabled;
}
updated_task.updated_at = Utc::now();
```

**KEY EXTENSIONS:**
1. Update task_type validation arrays to include `"start"` and `"sleep"`
2. In create_task: add `timezone: req.timezone.unwrap_or_else(|| "UTC".to_string())` and `run_once: req.run_once.unwrap_or(false)`, `last_result: None`, `last_error: None` to the CronTask construction
3. In update_task: add `if let Some(timezone) = req.timezone { ... }` and `if let Some(run_once) = req.run_once { ... }` conditional field updates
4. Add timezone validation: `if tz.parse::<chrono_tz::Tz>().is_err() { return Err("Invalid timezone".to_string()); }` (or use simple string validation and defer to backend)

---

### `api/src/presentation/handlers/node_handlers.rs` (EXTEND — controller, request-response)

**Analog:** Existing `node_handlers.rs::poll_node_commands` (lines 384-390)

**Existing poll endpoint** (lines 384-390):
```rust
pub async fn poll_node_commands(
    State(state): State<ApiState>,
    Path(node_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let commands = state.node_connection_manager.get_commands(&node_id).await;
    Ok(Json(ApiResponse::success(commands)))
}
```

**Existing handler response pattern** (lines 1-21):
```rust
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use crate::presentation::responses::api_response::ApiResponse;
use crate::shared::errors::app_error::AppError;
// ...other imports...
```

**APPROACH: Create `dispatch_node_command` endpoint**

**New dispatch endpoint structure:**
```rust
#[derive(serde::Deserialize)]
pub struct DispatchCommandRequest {
    pub command: String,         // "start", "stop", "restart", "backup.start", etc.
    pub server_id: Uuid,
    #[serde(default)]
    pub params: Option<DispatchParams>,
}

#[derive(serde::Deserialize, Default)]
pub struct DispatchParams {
    pub container_name: Option<String>,
    pub sleep: Option<bool>,
}

pub async fn dispatch_node_command(
    State(state): State<ApiState>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<DispatchCommandRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Build CommandParams from request
    // Call state.node_client.send_command_with_config(...)
    // This sends ExecuteCommand over Agent's WebSocket
    // Return CommandResponse
}
```

---

### `api/src/presentation/routes/api_routes.rs` (EXTEND — route, static)

**Analog:** Existing `api/src/presentation/routes/api_routes.rs` (all 128 lines)

**Existing node route pattern** (lines 41-60):
```rust
.route("/api/v1/nodes", get(...).post(...))
.route("/api/v1/nodes/:id", get(...).put(...).delete(...))
.route("/api/v1/nodes/online", get(...))
.route("/api/v1/nodes/:id/status/:status", put(...))
.route("/api/v1/nodes/:id/metrics", get(...))
.route("/api/v1/nodes/:id/commands", post(crate::presentation::handlers::node_handlers::poll_node_commands))
.route("/api/v1/nodes/:id/commands/result", post(crate::presentation::handlers::node_handlers::report_command_result))
```

**Add dispatch route** — insert near line 58, after the commands result route:
```rust
.route("/api/v1/nodes/:id/dispatch", post(crate::presentation::handlers::node_handlers::dispatch_node_command))
```

---

### `api/src/domain/repositories/cron_task_repository.rs` (EXTEND — model, static)

**Analog:** Existing `api/src/domain/repositories/cron_task_repository.rs` (all 14 lines)

**Existing trait pattern** (lines 1-14):
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

**NOTE:** The trait itself may not need changes — new columns are part of the `CronTask` entity and handled at the repository implementation level. However, if worker needs a dedicated method like `find_all_due_tasks` (all task types instead of `task_type = 'backup'`), add that here.

---

### `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` (EXTEND — service, CRUD)

**Analog:** Existing `postgres_cron_task_repository.rs` (all 167 lines)

**Existing repository struct and constructor** (lines 10-18):
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

**Existing SQL SELECT columns** (lines 25-26, 48-49, 126-128):
```rust
// In find_by_server_id:
SELECT id, server_id, user_id, task_type, schedule_cron, command, enabled, 
       last_run, next_run, created_at, updated_at
FROM cron_tasks
```

**Existing INSERT columns** (lines 67-69):
```rust
INSERT INTO cron_tasks (id, server_id, user_id, task_type, schedule_cron, command, enabled, 
    last_run, next_run, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
```

**Existing UPDATE columns** (lines 92-95):
```rust
UPDATE cron_tasks 
SET task_type = $2, schedule_cron = $3, command = $4, enabled = $5, 
    last_run = $6, next_run = $7, updated_at = $8
WHERE id = $1
```

**Existing row_to_task mapping** (lines 148-166):
```rust
fn row_to_task(&self, row: sqlx::postgres::PgRow) -> Result<CronTask> {
    Ok(CronTask {
        id: row.try_get("id")?,
        server_id: row.try_get("server_id")?,
        user_id: row.try_get("user_id")?,
        task_type: row.try_get("task_type")?,
        schedule_cron: row.try_get("schedule_cron")?,
        command: row.try_get("command")?,
        enabled: row.try_get("enabled")?,
        last_run: row.try_get::<Option<chrono::NaiveDateTime>, _>("last_run")?
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        next_run: row.try_get::<Option<chrono::NaiveDateTime>, _>("next_run")?
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        created_at: row.try_get::<chrono::NaiveDateTime, _>("created_at")?.and_utc(),
        updated_at: row.try_get::<chrono::NaiveDateTime, _>("updated_at")?.and_utc(),
    })
}
```

**KEY EXTENSIONS:**
1. Add `timezone`, `run_once`, `last_result`, `last_error` to ALL SELECT queries
2. Add `timezone`, `run_once`, `last_result`, `last_error` to INSERT query + bind parameters
3. Add columns to UPDATE SET clause + bind parameters (for last_result/last_error updates)
4. Add mappings in `row_to_task`:
   ```rust
   timezone: row.try_get::<String, _>("timezone")?,
   run_once: row.try_get("run_once")?,
   last_result: row.try_get("last_result")?,
   last_error: row.try_get("last_error")?,
   ```

---

### `api/src/migrations/20260531000001_add_cron_task_columns.sql` (NEW — migration, static)

**Analog:** `20260409000006_create_cron_tasks_table.sql` (all 34 lines) and other ALTER TABLE migrations like `20260530000002_add_restart_policy.sql`

**Migration SQL pattern** (from existing ALTER TABLE migrations):
```sql
ALTER TABLE cron_tasks
  ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
  ADD COLUMN IF NOT EXISTS run_once BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS last_result TEXT,
  ADD COLUMN IF NOT EXISTS last_error TEXT;

-- Update CHECK constraint for task_type to include 'start' and 'sleep'
ALTER TABLE cron_tasks DROP CONSTRAINT IF EXISTS cron_tasks_task_type_check;
ALTER TABLE cron_tasks ADD CONSTRAINT cron_tasks_task_type_check
  CHECK (task_type IN ('backup', 'restart', 'stop', 'command', 'start', 'sleep'));
```

---

### `app/src/pages/ServerDetails.jsx` (EXTEND — page, request-response)

**Analog:** Restart Policy section (lines 509-635) + Sleep/Wake section (lines 446-507)

**Existing Settings tab section pattern** (Restart Policy example, lines 509-635):
```jsx
{/* ─── RESTART POLICY CONFIG (Phase 57) ─── */}
<section className="glass-panel p-6 mt-6">
    <h3 className="text-lg font-bold mb-1">Restart Policy</h3>
    <p className="text-xs text-[var(--color-text-muted)] mb-5">
        Automatically restart server on crash or unresponsive state.
    </p>

    {restartToast && (
        <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${
            restartToast.type === 'success'
                ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
                : 'bg-red-500/10 border-red-500/30 text-red-400'
        }`}>
            {restartToast.message}
        </div>
    )}

    {/* Toggle */}
    <div className="flex items-center gap-3 p-4 rounded-xl border cursor-pointer
                    hover:border-[var(--color-cosmic-cyan)]/50"
         onClick={() => setAutoRestart(!autoRestart)}>
        <div className={`w-12 h-6 rounded-full transition-colors
                        ${autoRestart ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-[var(--color-cosmic-border)]'}`}>
            <div className={`w-5 h-5 rounded-full bg-white transition-transform
                            ${autoRestart ? 'translate-x-6' : 'translate-x-0.5'}`} />
        </div>
        <div className="flex-1">
            <p className="text-sm font-bold">Auto Restart</p>
            <p className="text-xs text-[var(--color-text-muted)]">Restart on crash or unresponsive</p>
        </div>
    </div>

    {/* Conditional fields */}
    {autoRestart && (
        <>
            <div className="mt-4">
                <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">
                    Max Restart Attempts
                </label>
                <input type="number" value={maxRestartAttempts}
                       min={1} max={20}
                       onChange={e => setMaxRestartAttempts(Math.max(1, Math.min(20, parseInt(e.target.value) || 5)))}
                       className="w-full px-4 py-2.5 rounded-lg text-sm
                                  bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)]
                                  text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)]
                                  focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all" />
            </div>
            {/* ... more fields ... */}
        </>
    )}

    {/* Save Button */}
    <button disabled={restartSaving} onClick={handleSaveRestartConfig}
            className="mt-5 w-full py-2.5 rounded-lg text-sm font-bold
                       bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                       hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                       disabled:opacity-50 transition-all">
        {restartSaving ? 'Saving...' : 'Save Changes'}
    </button>
</section>
```

**Existing state pattern for settings sections** (lines 63-68):
```javascript
// Restart Policy state
const [autoRestart, setAutoRestart] = useState(false);
const [maxRestartAttempts, setMaxRestartAttempts] = useState(5);
const [restartCooldown, setRestartCooldown] = useState(300);
const [restartSaving, setRestartSaving] = useState(false);
const [restartToast, setRestartToast] = useState(null);
```

**Existing save handler pattern** (lines 228-251):
```javascript
const handleSaveRestartConfig = async () => {
    try {
        setRestartSaving(true);
        await updateServer(id, { auto_restart: autoRestart, /* ... */ });
        setServer(prev => ({ ...prev, auto_restart: autoRestart }));
        setRestartToast({ type: 'success', message: '✅ Restart policy saved' });
    } catch (e) {
        setRestartToast({ type: 'error', message: `❌ Could not save restart policy. ${e.message}` });
    } finally {
        setRestartSaving(false);
        setTimeout(() => setRestartToast(null), 4000);
    }
};
```

**INSERTION POINT:** After the Restart Policy section's closing `</section>` (line 635), before the closing `</div>` (line 637). The Scheduled Actions section should use the `useScheduledActions` hook for state management and follow the exact same glass-panel pattern with:
- Section heading + description
- Schedule list (or empty state)
- Inline form (expandable on "Add" or "Edit" click)
- Full-width "+ Add Schedule" CTA button

**Imports to add** (near line 1-13):
```javascript
import { useScheduledActions } from '../hooks/useScheduledActions';
```

---

### `app/src/hooks/useScheduledActions.js` (NEW — hook, CRUD)

**Analog:** `app/src/hooks/useBackups.js` (all 59 lines) and `app/src/hooks/useServers.js` (all 121 lines)

**Existing hook pattern** (useBackups.js lines 1-58):
```javascript
import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

export function useBackups(serverId) {
    const [backups, setBackups] = useState([]);
    const [loading, setLoading] = useState(true);
    const [triggering, setTriggering] = useState(false);

    const refresh = useCallback(async () => {
        try {
            const data = await fetchApi(`/servers/${serverId}/backups`);
            setBackups(data || []);
        } catch (err) {
            console.error('Failed to fetch backups:', err);
        } finally {
            setLoading(false);
        }
    }, [serverId]);

    useEffect(() => { refresh(); }, [refresh]);
    // Poll every 10s for in_progress → success transitions
    // useEffect(() => { const interval = setInterval(refresh, 10000); return () => clearInterval(interval); }, [refresh]);

    const triggerBackup = useCallback(async () => {
        try {
            setTriggering(true);
            await fetchApi(`/servers/${serverId}/backups`, { method: 'POST' });
            setTimeout(refresh, 1500);
        } catch (err) { throw err; }
        finally { setTriggering(false); }
    }, [serverId, refresh]);

    return { backups, loading, triggering, triggerBackup, deleteBackup, restoreBackup, refresh };
}
```

**useScheduledActions hook pattern:**
```javascript
import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';
// OR: import { api } from '../lib/api'; — depending on which client ServerDetails uses

export function useScheduledActions(serverId) {
    const [schedules, setSchedules] = useState([]);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);

    const refresh = useCallback(async () => {
        try {
            const data = await fetchApi(`/servers/${serverId}/tasks`);
            setSchedules(data || []);
        } catch (err) {
            console.error('Failed to fetch schedules:', err);
        } finally {
            setLoading(false);
        }
    }, [serverId]);

    useEffect(() => { refresh(); }, [refresh]);

    const createSchedule = useCallback(async (data) => {
        try {
            setSaving(true);
            await fetchApi(`/servers/${serverId}/tasks`, {
                method: 'POST',
                body: JSON.stringify(data),
            });
            await refresh();
        } catch (err) { throw err; }
        finally { setSaving(false); }
    }, [serverId, refresh]);

    const updateSchedule = useCallback(async (taskId, data) => {
        try {
            setSaving(true);
            await fetchApi(`/tasks/${taskId}`, {
                method: 'PATCH',
                body: JSON.stringify(data),
            });
            await refresh();
        } catch (err) { throw err; }
        finally { setSaving(false); }
    }, [refresh]);

    const toggleSchedule = useCallback(async (taskId, enabled) => {
        await fetchApi(`/tasks/${taskId}`, {
            method: 'PATCH',
            body: JSON.stringify({ enabled }),
        });
        await refresh();
    }, [refresh]);

    const deleteSchedule = useCallback(async (taskId) => {
        await fetchApi(`/tasks/${taskId}`, { method: 'DELETE' });
        setSchedules(prev => prev.filter(s => s.id !== taskId));
    }, []);

    return { schedules, loading, saving, createSchedule, updateSchedule, toggleSchedule, deleteSchedule, refresh };
}
```

**API endpoints:**
- `GET /api/v1/servers/{serverId}/tasks` — list schedules for a server
- `POST /api/v1/servers/{serverId}/tasks` — create schedule
- `PATCH /api/v1/tasks/{taskId}` — update schedule (fields: enabled, schedule_cron, timezone, run_once, task_type)
- `DELETE /api/v1/tasks/{taskId}` — delete schedule

**Note:** The frontend uses `fetchApi` from `app/src/api/client.js` (not the `api.js` lib class), as seen in ServerDetails.jsx line 3: `import { fetchApi } from '../api/client';`

---

### `app/src/lib/api.js` (EXTEND — utility, request-response)

**Analog:** Existing `tasksApi` (lines 159-161) and `serversApi` (lines 89-105)

**Existing API client pattern** (lines 87-95):
```javascript
export const serversApi = {
  list: () => api.get('/servers'),
  get: (id) => api.get(`/servers/${id}`),
  create: (data) => api.post('/servers', data),
  update: (id, data) => api.put(`/servers/${id}`, data),
  delete: (id) => api.delete(`/servers/${id}`),
  // ...
}

export const tasksApi = {
  get: (externalId) => api.get(`/jobs/tasks/${externalId}`),
}
```

**Add scheduling API pattern:**
```javascript
export const schedulingApi = {
  list: (serverId) => api.get(`/servers/${serverId}/tasks`),
  create: (serverId, data) => api.post(`/servers/${serverId}/tasks`, data),
  update: (taskId, data) => api.patch(`/tasks/${taskId}`, data),
  delete: (taskId) => api.delete(`/tasks/${taskId}`),
}
```

**NOTE:** ServerDetails.jsx currently imports `fetchApi` from `app/src/api/client.js` (not `app/src/lib/api.js`). If the hook uses the `fetchApi` pattern (raw fetch with JSON body), add scheduling methods there as well:
```javascript
// In app/src/api/client.js:
export async function getSchedules(serverId) {
    return fetchApi(`/servers/${serverId}/tasks`);
}

export async function createSchedule(serverId, data) {
    return fetchApi(`/servers/${serverId}/tasks`, {
        method: 'POST',
        body: JSON.stringify(data),
    });
}

export async function updateSchedule(taskId, data) {
    return fetchApi(`/tasks/${taskId}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
    });
}

export async function deleteSchedule(taskId) {
    return fetchApi(`/tasks/${taskId}`, { method: 'DELETE' });
}
```

---

## Shared Patterns

### Authentication & Ownership (API Handlers)
**Source:** `cron_task_handlers.rs` lines 28-37, `node_handlers.rs` lines 23-30
**Apply to:** `node_handlers.rs` (dispatch endpoint), `cron_task_handlers.rs` (existing pattern)

All handler endpoints use `AuthUser` extractor for ownership checks:
```rust
use crate::domain::auth::middleware::AuthUser;

pub async fn handler(
    State(state): State<ApiState>,
    auth_user: AuthUser,
    // ...
) -> Result<impl IntoResponse, AppError> {
    // Check ownership
    if server.user_id != auth_user.tenant_id {
        return Err(AppError::Forbidden);
    }
    // ...
}
```

For the new `dispatch_node_command` endpoint: the Worker will call this endpoint, so it should accept either:
- An API key / internal auth header (for Worker-to-API communication)
- OR be gated behind internal network access

### Error Handling (API Handlers)
**Source:** `api/src/presentation/responses/api_response.rs` lines 168-188, `cron_task_handlers.rs` lines 29-36
**Apply to:** `node_handlers.rs` (dispatch endpoint), `cron_task_handlers.rs` (existing)

Two patterns coexist:
1. **Legacy string errors** (used in `cron_task_handlers.rs`):
   ```rust
   Result<impl IntoResponse, String>
   Err("Server not found".to_string())
   ```
2. **AppError** (used in `node_handlers.rs`):
   ```rust
   use crate::shared::errors::app_error::AppError;
   Result<impl IntoResponse, AppError>
   Err(AppError::NotFound)
   Err(AppError::InternalError(anyhow::anyhow!(e)))
   ```

The new `dispatch_node_command` endpoint should use AppError (consistent with `node_handlers.rs`).

### Error Handling (Worker)
**Source:** `worker/src/queue/mod.rs` lines 91-93
**Apply to:** New scheduled_* handlers

```rust
if let Err(e) = result {
    tracing::error!("Job {} failed: {}", job_id, e);
}
```

Each new handler should:
1. Log errors via `tracing::error!`
2. Update `cron_tasks.last_result` and `cron_tasks.last_error` with failure details
3. Implement 1 retry after 30s for transient failures (D-05)

### API Response Format
**Source:** `api_response.rs` lines 196-233
**Apply to:** `node_handlers.rs` dispatch endpoint

```rust
Ok(Json(ApiResponse::success(data)))
// Errors through AppError::into_response():
Err(AppError::BadRequest("message".into()))
```

### Worker API Proxy Pattern
**Source:** `worker/src/queue/mod.rs` lines 174-194
**Apply to:** New process_scheduled_start/stop/restart/sleep handlers

```rust
let api_base_url = std::env::var("API_BASE_URL")
    .unwrap_or_else(|_| "http://api:3000".to_string());
let api_url = format!("{}/api/v1/nodes/{}/dispatch", api_base_url, node_id);

let body = serde_json::json!({
    "command": "start",  // or "stop", "restart"
    "server_id": server_id,
    "params": {}
});

let client = reqwest::Client::new();
let response = client.post(&api_url).json(&body).send().await;
```

### Frontend Glass-Panel Section Pattern
**Source:** `ServerDetails.jsx` lines 446-507 (Sleep/Wake), 509-635 (Restart Policy)
**Apply to:** Scheduled Actions section in Settings tab

```jsx
<section className="glass-panel p-6 mt-6">
    <h3 className="text-lg font-bold mb-1">Section Title</h3>
    <p className="text-xs text-[var(--color-text-muted)] mb-5">
        Section description text.
    </p>
    
    {/* Toast notification */}
    {toast && (
        <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${
            toast.type === 'success'
                ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
                : 'bg-red-500/10 border-red-500/30 text-red-400'
        }`}>
            {toast.message}
        </div>
    )}

    {/* Content goes here */}

    {/* Full-width CTA button */}
    <button disabled={saving} onClick={handleSave}
            className="mt-5 w-full py-2.5 rounded-lg text-sm font-bold
                       bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                       hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                       disabled:opacity-50 transition-all">
        {saving ? 'Saving...' : 'Save Changes'}
    </button>
</section>
```

### API Route Registration Pattern
**Source:** `api_routes.rs` lines 41-60
**Apply to:** Adding the dispatch route

```rust
// Internal endpoint for Worker→Agent command dispatch
.route("/api/v1/nodes/:id/dispatch", post(
    crate::presentation::handlers::node_handlers::dispatch_node_command
))
```

### Frontend State Management Pattern
**Source:** `ServerDetails.jsx` lines 63-68 (toast + saving states)
**Apply to:** Scheduled Actions section state

```javascript
// Toast state
const [scheduleToast, setScheduleToast] = useState(null);

// Save pattern
const handleAction = async () => {
    try {
        setSaving(true);
        await apiCall();
        setScheduleToast({ type: 'success', message: '✅ Schedule saved' });
    } catch (e) {
        setScheduleToast({ type: 'error', message: `❌ Could not save schedule. ${e.message}` });
    } finally {
        setSaving(false);
        setTimeout(() => setScheduleToast(null), 4000);
    }
};
```

### Frontend UI Color Semantics (Action Type Badges)
**Source:** UI-SPEC.md — Action Type Badge Colors
**Apply to:** ServerDetails.jsx — Task type badge rendering

```jsx
<span className={`text-xs font-bold px-2 py-1 rounded ${
    schedule.task_type === 'start' ? 'bg-emerald-500/20 text-emerald-400' :
    schedule.task_type === 'stop' ? 'bg-red-500/20 text-red-400' :
    schedule.task_type === 'restart' ? 'bg-amber-500/20 text-amber-400' :
    'bg-purple-500/20 text-purple-400'  // sleep
}`}>
    {schedule.task_type.toUpperCase()}
</span>
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| N/A | — | — | All files have close analogs in the existing codebase |

---

## Metadata

**Analog search scope:** `worker/src/`, `api/src/domain/entities/`, `api/src/presentation/handlers/`, `api/src/presentation/routes/`, `api/src/infrastructure/repositories/`, `api/src/domain/repositories/`, `api/migrations/`, `app/src/pages/`, `app/src/hooks/`, `app/src/lib/`, `app/src/api/`
**Files scanned:** ~30 key source files
**Pattern extraction date:** 2026-05-31
