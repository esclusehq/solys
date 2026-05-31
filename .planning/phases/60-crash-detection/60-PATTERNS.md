# Phase 60: Crash Detection - Pattern Map

**Mapped:** 2026-05-31
**Files analyzed:** 14 (4 new, 10 modified)
**Analogs found:** 14 / 14

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `api/src/application/services/crash_classifier.rs` | service | transform | `api/src/application/services/monitoring_service.rs` | role-match |
| `api/migrations/20260531000001_create_server_crash_logs.sql` | migration | SQL | `api/migrations/20260530000002_add_restart_policy.sql` | exact |
| `api/src/domain/entities/server_crash_log.rs` | model | CRUD | `api/src/domain/entities/server.rs` | role-match |
| `api/src/infrastructure/repositories/crash_log_repository.rs` | repository | CRUD | `api/src/infrastructure/repositories/postgres_server_repository.rs` | exact |
| `api/src/presentation/ws/node_protocol.rs` | protocol | request-response | existing file (add variant) | exact |
| `agent/solys/src/agent_connection.rs` | controller | event-driven | existing file (add variant) | exact |
| `api/src/presentation/handlers/node_ws_handler.rs` | controller | event-driven | existing file (add match arm) | exact |
| `api/src/application/services/monitoring_service.rs` | service | CRUD | existing file (add crash classification) | exact |
| `api/src/presentation/handlers/server_handlers.rs` | controller | CRUD | existing file (add endpoints) | exact |
| `api/src/presentation/routes/api_routes.rs` | config | config | existing file (add route) | exact |
| `api/src/shared/events.rs` | utility | event-driven | existing file (add variant) | exact |
| `app/src/pages/ServerDetails.jsx` | component | request-response | existing file (add section) | exact |
| `app/src/api/client.js` | utility | request-response | existing file (add functions) | exact |
| `app/src/hooks/useCrashLogs.js` | hook | request-response | `app/src/hooks/useScheduledActions.js` | exact |

---

## Pattern Assignments

### `api/src/application/services/crash_classifier.rs` (service, transform)

**Analog:** `api/src/application/services/monitoring_service.rs` (pure function pattern from crash detection at lines 139-218)

**Imports pattern** (lines 1-8):
```rust
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use crate::domain::{
    repositories::{server_repository::ServerRepository, metrics_repository::MetricsRepository, node_repository::NodeRepository},
    factories::ExecutorFactory,
};
use crate::infrastructure::events::event_bus::EventBus;
use crate::shared::events::ServerEvent;
```

**Core classification pattern** (pure function — modeled after RESEARCH.md Pattern 2, lines 233-278):

The classifier should be a pure function module — no struct, no async, just a module with public functions:

```rust
// api/src/application/services/crash_classifier.rs
use once_cell::sync::Lazy;
use regex::Regex;

static OOM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(OutOfMemoryError|java\.lang\.OutOfMemoryError|Killed|Cannot allocate memory|java heap space)")
        .expect("Invalid OOM regex")
});

static PLUGIN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(NullPointerException|PluginClassLoader|java\.lang\.reflect\.InvocationTargetException|Caused by:.*Exception)")
        .expect("Invalid plugin exception regex")
});

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CrashType {
    Oom,
    ConfigError,
    PluginCrash,
    Generic,
}

pub fn classify_crash(exit_code: i32, log_excerpt: &str) -> CrashType {
    if exit_code == 137 && OOM_RE.is_match(log_excerpt) {
        CrashType::Oom
    } else if OOM_RE.is_match(log_excerpt) {
        CrashType::Oom
    } else if PLUGIN_RE.is_match(log_excerpt) {
        CrashType::PluginCrash
    } else {
        CrashType::Generic
    }
}
```

**Error handling:** Pure function — no errors possible (regex is compiled at startup). Use `Lazy` for thread-safe lazy init of regex patterns.

---

### `api/migrations/20260531000001_create_server_crash_logs.sql` (migration, SQL)

**Analog:** `api/migrations/20260530000002_add_restart_policy.sql` (lines 1-9)

**Migration pattern** (lines 1-9):
```sql
-- Add restart policy and health check fields
-- Phase 57: Auto Restart Policies

-- Restart history tracking
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_restart_at TIMESTAMPTZ;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_restart_reason TEXT;

-- Health check configuration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS health_check_timeout_seconds INTEGER NOT NULL DEFAULT 5;
```

**For the crash_logs table** — use `CREATE TABLE IF NOT EXISTS` pattern:
```sql
-- Phase 60: Crash Detection
CREATE TABLE IF NOT EXISTS server_crash_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    crashed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    exit_code INTEGER NOT NULL,
    crash_type VARCHAR(32) NOT NULL,
    log_excerpt TEXT,
    recovery_action VARCHAR(32) NOT NULL,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_server_crash_logs_server_id ON server_crash_logs(server_id);
CREATE INDEX IF NOT EXISTS idx_server_crash_logs_crashed_at ON server_crash_logs(crashed_at DESC);
```

(Use `IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` idempotency patterns consistent with existing migrations.)

---

### `api/src/domain/entities/server_crash_log.rs` (model, CRUD)

**Analog:** `api/src/domain/entities/server.rs` (lines 1-68)

**Entity pattern** (lines 1-6, 70-78):
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    // ... fields ...
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**For crash_log entity:**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCrashLog {
    pub id: Uuid,
    pub server_id: Uuid,
    pub crashed_at: DateTime<Utc>,
    pub exit_code: i32,
    pub crash_type: String,      // "oom", "config_error", "plugin_crash", "generic"
    pub log_excerpt: Option<String>,
    pub recovery_action: String, // "auto_restarted", "notified_only", "restart_disabled"
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
```

---

### `api/src/infrastructure/repositories/crash_log_repository.rs` (repository, CRUD)

**Analog:** `api/src/infrastructure/repositories/postgres_server_repository.rs` (lines 1-19, 304-319)

**Repository struct + impl pattern** (lines 11-19):
```rust
use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;
use anyhow::{Result, Context};

pub struct PostgresServerRepository {
    pool: PgPool,
}

impl PostgresServerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

**For crash_log repository:**
```rust
use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;
use anyhow::{Result, Context};
use crate::domain::entities::server_crash_log::ServerCrashLog;

pub struct PostgresCrashLogRepository {
    pool: PgPool,
}

impl PostgresCrashLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn insert(&self, log: &ServerCrashLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO server_crash_logs (id, server_id, crashed_at, exit_code, crash_type, log_excerpt, recovery_action, resolved_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(log.id)
        .bind(log.server_id)
        .bind(log.crashed_at.naive_utc())
        .bind(log.exit_code)
        .bind(&log.crash_type)
        .bind(&log.log_excerpt)
        .bind(&log.recovery_action)
        .bind(log.resolved_at)
        .bind(log.created_at.naive_utc())
        .execute(&self.pool)
        .await
        .context("Failed to insert crash log")?;
        Ok(())
    }
    
    pub async fn list_by_server(&self, server_id: Uuid, limit: i64, offset: i64) -> Result<Vec<ServerCrashLog>> {
        let rows = sqlx::query_as::<_, ServerCrashLog>(
            r#"
            SELECT id, server_id, crashed_at, exit_code, crash_type, log_excerpt, recovery_action, resolved_at, created_at
            FROM server_crash_logs
            WHERE server_id = $1
            ORDER BY crashed_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(server_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list crash logs")?;
        Ok(rows)
    }
    
    pub async fn count_by_server(&self, server_id: Uuid) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM server_crash_logs WHERE server_id = $1
            "#,
        )
        .bind(server_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count crash logs")?;
        Ok(row)
    }
    
    pub async fn count_recent(&self, server_id: Uuid, since: DateTime<Utc>) -> Result<i64> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM server_crash_logs
            WHERE server_id = $1 AND crashed_at >= $2
            "#,
        )
        .bind(server_id)
        .bind(since.naive_utc())
        .fetch_one(&self.pool)
        .await
        .context("Failed to count recent crash logs")
    }
    
    pub async fn delete_by_server(&self, server_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM server_crash_logs WHERE server_id = $1")
            .bind(server_id)
            .execute(&self.pool)
            .await
            .context("Failed to clear crash logs")?;
        Ok(())
    }
    
    pub async fn resolve(&self, log_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE server_crash_logs SET resolved_at = $2 WHERE id = $1
            "#,
        )
        .bind(log_id)
        .bind(chrono::Utc::now().naive_utc())
        .execute(&self.pool)
        .await
        .context("Failed to resolve crash log")?;
        Ok(())
    }
}
```

**SQLx query pattern:** Follow `postgres_server_repository.rs` (line 304-319) for simple single-statement queries:
```rust
async fn update_status(&self, id: &Uuid, status: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE servers
        SET status = $2, updated_at = $3
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(chrono::Utc::now().naive_utc())
    .execute(&self.pool)
    .await
    .context("Failed to update server status ")?;
    Ok(())
}
```

---

### `api/src/presentation/ws/node_protocol.rs` (protocol, request-response) — ADD CrashReport variant

**Analog:** Existing file — add to NodeMessage enum (lines 6-120)

**Enum variant pattern** (lines 6-8, 65-75):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeMessage {
    // Agent -> Backend
    #[serde(rename = "register")]
    Register { ... },

    // ...existing variants...

    // Phase 60: Crash Report
    #[serde(rename = "crash_report")]
    CrashReport {
        server_id: Uuid,
        exit_code: i32,
        log_excerpt: String,
        timestamp: String,
    },

    // Backend -> Agent
    #[serde(rename = "execute_command")]
    ExecuteCommand { ... },
}
```

Add the `CrashReport` variant to the existing `NodeMessage` enum following the tagged enum pattern with `#[serde(tag = "type")]`.

---

### `agent/solys/src/agent_connection.rs` (controller, event-driven) — ADD CrashReport sender

**Analog:** Existing file — add to AgentMessage enum (lines 27-80)

**AgentMessage enum pattern** (lines 27-46):
```rust
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
enum AgentMessage {
    #[serde(rename = "register")]
    Register { ... },
    #[serde(rename = "heartbeat")]
    Heartbeat { ... },

    // Phase 60: Crash Report
    #[serde(rename = "crash_report")]
    CrashReport {
        server_id: Uuid,
        exit_code: i32,
        log_excerpt: String,
        timestamp: String,
    },
}
```

**Sending pattern** (follow heartbeat send pattern at lines 371-379):
```rust
let crash_report = AgentMessage::CrashReport {
    server_id,
    exit_code,
    log_excerpt,
    timestamp: chrono::Utc::now().to_rfc3339(),
};
if let Ok(msg) = serde_json::to_string(&crash_report) {
    let _ = ws_sender.send(Message::Text(msg.into())).await;
}
```

The crash reporter logic (container monitor) should be a new module or function that:
1. Uses bollard's `inspect_container()` to get exit code (following runtime.rs patterns)
2. Uses bollard's `LogsOptions` with `tail: 10` to capture log excerpt
3. Sends the `CrashReport` WebSocket message via the existing `ws_sender`

---

### `api/src/presentation/handlers/node_ws_handler.rs` (controller, event-driven) — ADD CrashReport handler

**Analog:** Existing file — add match arm (lines 90-338)

**Match arm pattern** (follow TaskProgress handling at lines 296-323):
```rust
NodeMessage::CrashReport { server_id, exit_code, log_excerpt, timestamp } => {
    tracing::warn!(
        "[WS] CrashReport: server={}, exit_code={}, timestamp={}",
        server_id, exit_code, timestamp
    );
    // Forward to monitoring service via channel
    // Phase 60: container.crash_report_tx.send(CrashReportData { ... }).await;
}
```

**WS message handling pattern** (lines 90-96):
```rust
while let Some(msg) = receiver.next().await {
    match msg {
        Ok(Message::Text(text)) => {
            match serde_json::from_str::<NodeMessage>(&text) {
                Ok(node_msg) => {
                    match node_msg {
                        // ... existing match arms ...
                        NodeMessage::CrashReport { ... } => { /* new handler */ }
                        _ => { tracing::warn!("Unexpected message type"); }
                    }
                }
                Err(e) => { /* parse error handling */ }
            }
        }
        Ok(Message::Close(_)) => { break; }
        Err(e) => { break; }
        _ => {}
    }
}
```

**Channel-based handoff pattern:** Use `tokio::sync::mpsc` sender stored in `ApiState` for crash reports. The WS handler sends crash report data through the channel, and MonitoringService receives it on each loop tick. Follow the RESEARCH.md recommendation (lines 643-646):
```rust
// In node_ws_handler.rs:
NodeMessage::CrashReport { server_id, exit_code, log_excerpt, timestamp } => {
    if let Err(e) = container.crash_report_tx.send(CrashReportData {
        server_id, exit_code, log_excerpt,
    }).await {
        tracing::error!("Failed to enqueue crash report: {}", e);
    }
}
```

---

### `api/src/application/services/monitoring_service.rs` (service, CRUD) — ADD crash ingestion

**Analog:** Existing file — add crash handling after the existing crash detection (lines 136-218)

**Existing crash detection pattern** (lines 136-218) — Phase 57's running→stopped detection already handles auto-restart. Phase 60 adds crash report ingestion + classification + recovery.

**Crash report reception** — add at the top of `check_all_servers()` (before line 78):
```rust
// Phase 60: Drain crash report channel
while let Ok(report) = self.crash_report_rx.try_recv() {
    self.handle_crash_report(report).await;
}
```

**Crash handling method** (modeled on existing backoff restart at lines 156-186):
```rust
async fn handle_crash_report(&self, report: CrashReportData) {
    // 1. Fetch server
    let server = match self.repository.find_by_id(&report.server_id).await {
        Ok(Some(s)) => s,
        _ => {
            tracing::error!("Crash report for unknown server: {}", report.server_id);
            return;
        }
    };
    
    // 2. Classify crash (pure function)
    let crash_type = crash_classifier::classify_crash(report.exit_code, &report.log_excerpt);
    
    // 3. Execute recovery per crash type (D-03)
    match crash_type {
        CrashType::Oom => {
            // Notify only — do NOT auto-restart
            self.notify_and_store(&server, &report, &crash_type, "notified_only").await;
        }
        CrashType::ConfigError => {
            // Check crash-loop: 3 rapid crashes within 60s
            let recent = self.crash_log_repo.count_recent(server.id, Utc::now() - Duration::from_secs(60)).await.unwrap_or(0);
            if recent >= 3 {
                // Disable auto-restart
                let mut updated = server.clone();
                updated.auto_restart = false;
                let _ = self.repository.update(&updated).await;
                self.notify_and_store(&server, &report, &crash_type, "restart_disabled").await;
            } else {
                self.notify_and_store(&server, &report, &crash_type, "auto_restarted").await;
            }
        }
        CrashType::PluginCrash | CrashType::Generic => {
            // Follow Phase 57 auto-restart policy (already handled in detection loop)
            self.notify_and_store(&server, &report, &crash_type, "auto_restarted").await;
        }
    }
    
    // 4. Notifications (D-04): toast + event + Discord
    self.notify_crash(&server, &crash_type, &report).await;
}

async fn notify_and_store(&self, server: &Server, report: &CrashReportData, crash_type: &CrashType, recovery: &str) {
    // Store crash log
    let log = ServerCrashLog {
        id: Uuid::new_v4(),
        server_id: server.id,
        crashed_at: Utc::now(),
        exit_code: report.exit_code,
        crash_type: serde_json::to_value(crash_type).unwrap().to_string().to_lowercase(),
        log_excerpt: Some(report.log_excerpt.clone()),
        recovery_action: recovery.to_string(),
        resolved_at: None,
        created_at: Utc::now(),
    };
    let _ = self.crash_log_repo.insert(&log).await;
}
```

**tokio::spawn pattern** (for non-blocking notification, lines 169-185):
```rust
tokio::spawn(async move {
    // Discord/slow notification work
});
```

---

### `api/src/presentation/handlers/server_handlers.rs` (controller, CRUD) — ADD crash-log endpoints

**Analog:** Existing file — add handler functions (follow pattern at lines 531-548 for get_server)

**Handler pattern** (lines 531-548):
```rust
async fn get_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;
    
    // Check tenant access
    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }
    
    Ok(Json(ApiResponse::success(server)))
}
```

**New handler functions (following pattern above):**
```rust
// List crash logs with pagination
async fn list_crash_logs(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, String> {
    // Tenant check same as get_server
    // ...
    let repo = PostgresCrashLogRepository::new(state.pool.clone());
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let logs = repo.list_by_server(id, limit, offset).await.map_err(|e| e.to_string())?;
    let total = repo.count_by_server(id).await.unwrap_or(0);
    Ok(Json(json!({ "data": logs, "total": total })))
}

// Clear crash logs
async fn clear_crash_logs(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    // Tenant check same as get_server
    // ...
    let repo = PostgresCrashLogRepository::new(state.pool.clone());
    repo.delete_by_server(id).await.map_err(|e| e.to_string())?;
    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "cleared" }))))
}

// Resolve (acknowledge) a crash
async fn resolve_crash_log(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path((server_id, log_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, String> {
    // Tenant check same as get_server
    // ...
    let repo = PostgresCrashLogRepository::new(state.pool.clone());
    repo.resolve(log_id).await.map_err(|e| e.to_string())?;
    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "resolved" }))))
}
```

**Route registration** — add to `ServerHandlers::router()` (lines 305-381):
```rust
// Phase 60: Crash Logs
.route("/:id/crash-logs", get(list_crash_logs).delete(clear_crash_logs))
.route("/:id/crash-logs/:log_id/resolve", post(resolve_crash_log))
```

---

### `api/src/presentation/routes/api_routes.rs` (config, config) — ADD route registration

**Analog:** Existing file — add route line (follow pattern at lines 34-37)

**Route pattern** (lines 34-37):
```rust
.route("/api/v1/servers/:server_id/backup-config", get(...).put(...))
.route("/api/v1/servers/:server_id/tasks", get(...).post(...))
.route("/api/v1/servers/:server_id/tasks/:task_id", patch(...).delete(...))
```

These crash-log routes are already handled inside `ServerHandlers::router()` (nested under `/api/v1/servers` at line 33), so no change needed to `api_routes.rs` unless the crash-log resolvers need separate top-level routes.

**Actually:** No change needed — `ServerHandlers::router()` is mounted at line 33:
```rust
.nest("/api/v1/servers", ServerHandlers::router(state.clone()))
```

All new crash-log routes inside `ServerHandlers::router()` will automatically be under `/api/v1/servers/:id/crash-logs`.

---

### `api/src/shared/events.rs` (utility, event-driven) — ADD CrashDetected variant

**Analog:** Existing file — add variant (lines 1-31)

**ServerEvent enum pattern** (lines 4-9):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerEvent {
    StatusChanged {
        server_id: Uuid,
        status: String,
    },
    MetricsUpdated {
        server_id: Uuid,
        cpu_usage: f32,
        memory_usage_mb: i64,
        disk_usage_mb: i64,
        tps: Option<f32>,
        players: i32,
    },
    AlertTriggered(crate::domain::entities::alert::AlertTriggered),
    LogOutput {
        server_id: Uuid,
        timestamp: String,
        line: String,
        stream: String,
    },
    ScheduleFailed {
        server_id: Uuid,
        command: String,
        reason: String,
    },
    // Phase 60: Crash Detection
    CrashDetected {
        server_id: Uuid,
        crash_type: String,
        exit_code: i32,
        recovery_action: String,
    },
}
```

---

### `app/src/pages/ServerDetails.jsx` (component, request-response) — ADD Crash History section

**Analog:** Existing file — add section in Settings tab (follow Sleep & Wake section lines 541-602, Restart Policy section lines 604-730, Scheduled Actions section lines 732-964)

**Settings tab section pattern** (follow Restart Policy section at lines 604-730):
```jsx
{/* ─── CRASH HISTORY (Phase 60) ─── */}
<section className="glass-panel p-6 mt-6">
    <h3 className="text-lg font-bold mb-1">Crash History</h3>
    <p className="text-xs text-[var(--color-text-muted)] mb-5">
        Detailed crash log with diagnostic information.
    </p>

    {crashToast && (
        <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${crashToast.type === 'success'
            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
            : 'bg-red-500/10 border-red-500/30 text-red-400'
        }`}>
            {crashToast.message}
        </div>
    )}

    {crashLogs.length === 0 ? (
        <div className="p-8 rounded-xl border border-dashed border-[var(--color-cosmic-border)] text-center">
            <p className="text-sm text-[var(--color-text-muted)]">No crash history</p>
        </div>
    ) : (
        <div className="space-y-2">
            {crashLogs.map(log => (
                <div key={log.id} className="p-3 rounded-xl border border-[var(--color-cosmic-border)]">
                    <div className="flex items-center gap-2 mb-1">
                        <CrashTypeBadge type={log.crash_type} />
                        <span className="text-xs text-[var(--color-text-muted)]">
                            {new Date(log.crashed_at).toLocaleString()}
                        </span>
                    </div>
                    <div className="grid grid-cols-2 gap-2 text-xs">
                        <span>Exit code: <strong>{log.exit_code}</strong></span>
                        <span>Action: <strong>{log.recovery_action}</strong></span>
                    </div>
                    {log.log_excerpt && (
                        <pre className="mt-2 p-2 rounded bg-black/40 text-[11px] font-mono
                                       text-[var(--color-text-muted)] overflow-x-auto">
                            {log.log_excerpt}
                        </pre>
                    )}
                    {!log.resolved_at && (
                        <button onClick={() => acknowledgeCrash(log.id)}
                                className="mt-2 text-xs text-[var(--color-cosmic-cyan)] hover:underline">
                            Mark as Resolved
                        </button>
                    )}
                </div>
            ))}
        </div>
    )}

    {/* Pagination */}
    {totalPages > 1 && (
        <div className="flex items-center justify-center gap-2 mt-4">
            {/* page buttons */}
        </div>
    )}

    {/* Clear All */}
    {crashLogs.length > 0 && (
        <button onClick={handleClearCrashLogs}
                className="mt-4 w-full py-2.5 rounded-lg text-sm font-bold
                           bg-red-500/10 text-red-400 border border-red-500/30
                           hover:bg-red-500/20 transition-all">
            Clear Crash History
        </button>
    )}
</section>
```

**State pattern** (follow Restart Policy state at lines 63-69):
```javascript
// Crash History state (near line 69)
const [crashLogs, setCrashLogs] = useState([]);
const [crashLoading, setCrashLoading] = useState(false);
const [crashPage, setCrashPage] = useState(0);
const [crashTotal, setCrashTotal] = useState(0);
const [crashToast, setCrashToast] = useState(null);
const PAGE_SIZE = 10;
```

**Data fetching pattern** (follow initial load at lines 102-120):
```javascript
// Load crash history
useEffect(() => {
    if (activeTab !== 'settings') return;
    setCrashLoading(true);
    fetchApi(`/servers/${id}/crash-logs?limit=${PAGE_SIZE}&offset=${crashPage * PAGE_SIZE}`)
        .then(data => {
            setCrashLogs(data.data || []);
            setCrashTotal(data.total || 0);
        })
        .catch(err => console.error('Failed to load crash logs:', err))
        .finally(() => setCrashLoading(false));
}, [id, activeTab, crashPage]);
```

**Toast saving pattern** (follow webhook save at lines 514-525):
```javascript
const handleClearCrashLogs = async () => {
    try {
        await clearCrashLogs(id);
        setCrashLogs([]);
        setCrashTotal(0);
        setCrashToast({ type: 'success', message: '✅ Crash history cleared' });
    } catch (e) {
        setCrashToast({ type: 'error', message: `❌ ${e.message}` });
    }
    setTimeout(() => setCrashToast(null), 4000);
};
```

**Tab structure note:** The section should be added INSIDE the `activeTab === 'settings'` block (line 484) AFTER the Scheduled Actions section (line 964), following the Settings tab ordering: Webhook → Sleep & Wake → Restart Policy → Scheduled Actions → **Crash History** (new).

---

### `app/src/api/client.js` (utility, request-response) — ADD crash log API functions

**Analog:** Existing file — add functions (follow schedule functions at lines 126-146)

**API function pattern** (lines 126-130):
```javascript
export async function getSchedules(serverId) {
    return fetchApi(`/servers/${serverId}/tasks`);
}

export async function createSchedule(serverId, data) {
    return fetchApi(`/servers/${serverId}/tasks`, {
        method: 'POST',
        body: JSON.stringify(data),
    });
}
```

**New crash log functions:**
```javascript
// ─── Crash History (Phase 60) ───
export async function getCrashLogs(serverId, limit = 20, offset = 0) {
    return fetchApi(`/servers/${serverId}/crash-logs?limit=${limit}&offset=${offset}`);
}

export async function clearCrashLogs(serverId) {
    return fetchApi(`/servers/${serverId}/crash-logs`, { method: 'DELETE' });
}

export async function acknowledgeCrash(serverId, logId) {
    return fetchApi(`/servers/${serverId}/crash-logs/${logId}/resolve`, { method: 'POST' });
}
```

---

### `app/src/hooks/useCrashLogs.js` (hook, request-response) — NEW

**Analog:** `app/src/hooks/useScheduledActions.js` (lines 1-71)

**Hook pattern** (lines 1-71):
```javascript
import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

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
    
    // ... mutation methods ...

    return { schedules, loading, saving, ... };
}
```

**For `useCrashLogs.js`:**
```javascript
import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

const PAGE_SIZE = 10;

export function useCrashLogs(serverId) {
    const [logs, setLogs] = useState([]);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(false);

    const refresh = useCallback(async () => {
        if (!serverId) return;
        setLoading(true);
        try {
            const data = await fetchApi(`/servers/${serverId}/crash-logs?limit=${PAGE_SIZE}&offset=${page * PAGE_SIZE}`);
            setLogs(data.data || []);
            setTotal(data.total || 0);
        } catch (err) {
            console.error('Failed to fetch crash logs:', err);
        } finally {
            setLoading(false);
        }
    }, [serverId, page]);

    useEffect(() => { refresh(); }, [refresh]);

    const totalPages = Math.ceil(total / PAGE_SIZE);

    const clearLogs = useCallback(async () => {
        await fetchApi(`/servers/${serverId}/crash-logs`, { method: 'DELETE' });
        setLogs([]);
        setTotal(0);
        setPage(0);
    }, [serverId]);

    const acknowledge = useCallback(async (logId) => {
        await fetchApi(`/servers/${serverId}/crash-logs/${logId}/resolve`, { method: 'POST' });
        setLogs(prev => prev.map(l => l.id === logId ? { ...l, resolved_at: new Date().toISOString() } : l));
    }, [serverId]);

    return { logs, total, page, totalPages, loading, setPage, clearLogs, acknowledge, refresh };
}
```

---

## Shared Patterns

### Authentication (Tenant Access Check)
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 542-545)
**Apply to:** All new crash-log REST handlers
```rust
// Check tenant access
if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
    return Err("Access denied".to_string());
}
```

### Error Handling (Result<String> pattern)
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 531-548)
**Apply to:** All crash-log handlers
```rust
let repo = SqlxServerRepository::new(state.pool.clone());
let server = repo.find_by_id(id)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Server not found".to_string())?;
```

### Discord Notification
**Source:** `api/src/application/services/server_event_notifier.rs` (lines 42-52)
**Apply to:** Crash notification via Discord
```rust
pub async fn notify_server_crashed(&self, server: &Server) -> Result<()> {
    self.send_event(server, "server.crashed",
        "⚠️ Your game server has crashed! Please check the logs for more details.").await
}
```
The `DiscordClient::send_server_event` already handles `server.crashed` at `discord_client.rs` line 114.

### Server Event Emission (Event Timeline)
**Source:** `api/src/domain/webhook/service.rs` (lines 223-236)
**Apply to:** Crash timeline events
```rust
pub async fn emit_server_event(pool: &PgPool, event_type: &str, user_id: Uuid, server_id: Uuid, server_name: &str) {
    let service = WebhookService::new(pool.clone());
    let data = serde_json::json!({ "server_id": server_id, "server_name": server_name });
    service.emit(event_type, user_id, data).await;
}
```
Use with: `emit_server_event(&pool, "server.crash_detected", ...).await`

### Event Bus Notification (Toast)
**Source:** `api/src/presentation/handlers/node_ws_handler.rs` (lines 283-287)
**Apply to:** Crash toast
```rust
let _ = container.event_bus.publish(ServerEvent::StatusChanged { server_id, status: "running".to_string() });
```
For crash: `container.event_bus.publish(ServerEvent::CrashDetected { server_id, crash_type, exit_code, recovery_action })`

### MonitoringService Crash Detection Loop (Double-Restart Prevention)
**Source:** `api/src/application/services/monitoring_service.rs` (lines 136-218)
**Apply to:** Phase 60 must gate Phase 57's existing crash detection to prevent double-restart (Pitfall 3)

The existing `running → stopped` detection path at lines 139-218 triggers Phase 57's auto-restart. Phase 60's CrashReport WS handler now also triggers recovery. Strategy: in the monitoring loop, check `last_restart_at` — if a restart happened within the last 30s, skip the auto-restart path:
```rust
// Phase 60: Skip auto-restart if crash report was already handled
if let Some(last_restart) = full_server.last_restart_at {
    let elapsed = chrono::Utc::now() - last_restart;
    if elapsed.num_seconds() < 30 {
        tracing::info!("[MONITOR] Skipping auto-restart for {} — crash already handled via CrashReport", server.name);
        continue;
    }
}
```

### WebSocket Tagged Enum Pattern
**Source:** `api/src/presentation/ws/node_protocol.rs` (lines 5-8), `agent/solys/src/agent_connection.rs` (lines 27-29)
**Apply to:** Both `NodeMessage` (backend) and `AgentMessage` (agent) for `CrashReport` variant
```rust
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
enum AgentMessage {
    #[serde(rename = "crash_report")]
    CrashReport { server_id: Uuid, exit_code: i32, log_excerpt: String, timestamp: String },
}
```

### API Response Wrapper Pattern
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 397-398)
**Apply to:** Crash log handler responses
```rust
Ok(Json(ApiResponse::success(created)))
Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "deleted" }))))
```

### Pagination Query Pattern
**Source:** RESEARCH.md Pattern 4 (lines 564-587)
**Apply to:** GET crash-logs endpoint
```rust
#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

---

## No Analog Found

All files have existing analogs in the codebase. No files require RESEARCH.md patterns as primary guide.

| File | Role | Data Flow | Analog Found |
|------|------|-----------|--------------|
| `api/src/application/services/crash_classifier.rs` | service | transform | Partial match — no existing pure classification module; modeled on RESEARCH.md Patterns 2-3 |
| `app/src/hooks/useCrashLogs.js` | hook | request-response | useScheduledActions.js — exact match for hook structure |

---

## Metadata

**Analog search scope:** `api/src/`, `agent/solys/src/`, `agent/agent-core/crates/agent-proto/src/`, `app/src/`
**Files scanned:** 30+ (all services, handlers, repositories, entities, routes, frontend pages, hooks, API client)
**Pattern extraction date:** 2026-05-31
