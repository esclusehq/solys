# Phase 60: Crash Detection - Research

**Researched:** 2026-05-31
**Domain:** Game server crash forensics, classification, and intelligent recovery
**Confidence:** HIGH

## Summary

This phase delivers intelligent crash diagnosis for game servers — detect WHEN a server crashes, classify WHY (OOM, config error, plugin crash, generic), and trigger the appropriate recovery action per crash type. It builds directly on Phase 57's existing monitoring and auto-restart infrastructure.

The architecture splits responsibilities cleanly: the **Web Agent** captures raw crash data (container exit code + last N log lines) and reports it via a new `CrashReport` WebSocket message. The **backend MonitoringService** receives these reports, classifies the crash type using pattern matching on exit codes and log content, and executes the recovery strategy defined in D-03. Recovery actions range from "notify only" (OOM — restarting without more RAM is futile) to "restart with logging" (plugin crashes may be transient). Every crash triggers multi-channel notifications: WebSocket toast, server event timeline, and Discord webhook.

A new `server_crash_logs` database table stores the forensic data, exposed via REST endpoints for the frontend Crash History UI in the Settings tab.

**Primary recommendation:** Extend the existing `AgentMessage`/`NodeMessage` WebSocket protocol with a `CrashReport` variant. Add crash capture logic to the web-agent's container monitoring. Inject crash classification into the MonitoringService's existing crash detection path (running→stopped status change). Follow the established Phase 56/57 frontend Settings tab section pattern for the Crash History UI.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** The Web Agent reports crashes via a structured WebSocket message containing container exit code + last N lines of server stdout/stderr at crash time. Crash reporter logic lives inside the web-agent binary.
- **D-02:** The Agent sends RAW crash data (exit code + log excerpt) to the backend. The backend MonitoringService classifies the crash type and decides recovery action. The agent is a reporter, not a decision-maker.
- **D-03:** Recovery strategy per crash type:
  - **OOM** (exit 137 + memory patterns) → **Notify only** — do NOT auto-restart
  - **Config error** (3 rapid crashes within 60s of startup) → **Disable auto-restart** after 3 rapid crashes
  - **Plugin/mod crash** (exception in plugin code in logs) → **Restart + log reason** — follow Phase 57 restart policy
  - **Generic crash** (exit code != 0, 137, no specific pattern) → Follow Phase 57's existing auto-restart policy
- **D-04:** Notify user on EVERY crash detection, regardless of recovery outcome: toast notification in web app, server event timeline entry, Discord webhook notification.
- **D-05:** Crash history appears in the ServerDetails **Settings tab** as a "Crash History" section. Each entry shows: timestamp, crash type, exit code, log excerpt, recovery action.
- **D-06:** New `server_crash_logs` table with columns: server_id (UUID FK), crashed_at (TIMESTAMPTZ), exit_code (INTEGER), crash_type (VARCHAR), log_excerpt (TEXT), recovery_action (VARCHAR), resolved_at (TIMESTAMPTZ nullable). Retention: unlimited.

### The Agent's Discretion
- Exact log line capture strategy (how many lines, encoding limits)
- Crash type detection patterns in MonitoringService (regex patterns for OOM, plugin exceptions, config errors)
- Discord webhook message format for crash notifications
- Specific UI layout of Crash History section in Settings tab
- Toast notification design and auto-dismiss duration
- WebSocket message format for crash report from agent to backend
- 60s crash-loop detection window exact value

### Deferred Ideas (OUT OF SCOPE)
- **Crash alert escalation** (SMS/pager if server stays down for X minutes) — future phase
- **Automated crash fix suggestions** ("increase RAM by 2GB" for OOM) — post-MVP enhancement
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Crash data capture (exit code + log excerpt) | Web Agent | — | Agent manages containers via Bollard, sees exit codes and can read container logs directly |
| Crash data transport | WebSocket (agent→backend) | — | Existing WS protocol between agent and backend; add CrashReport message variant |
| Crash classification | API / Backend (MonitoringService) | — | Backend has full server context (plan, resources, restart history). Classification logic can evolve without agent updates. |
| Recovery action execution | API / Backend (MonitoringService) | — | Reuses Phase 57 auto-restart infrastructure (backoff, max attempts, cooldown) |
| Crash storage | API / Backend | Database (PostgreSQL) | New `server_crash_logs` table, read/write via repository layer |
| Crash notifications (toast) | API / Backend | Browser / Client | WebSocket ServerEvent → frontend display |
| Crash notifications (event timeline) | API / Backend | — | `emit_server_event("server.crash_detected")` pattern |
| Crash notifications (Discord) | API / Backend | — | Existing `DiscordClient::send_server_event("server.crashed")` pattern |
| Crash History UI | Browser / Client | — | Paginated list in ServerDetails Settings tab, fetch via REST |
| Crash log clear | API / Backend | Browser / Client | DELETE endpoint for clearing crash history |

## Standard Stack

### Core — No new dependencies needed

All work uses the existing stack. No new Rust crates or npm packages required.

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bollard | v0.18 | Docker API client (inspect container exit codes, read logs) | Already used in web-agent for all container ops |
| tokio-tungstenite | v0.26 | WebSocket client for agent communication | Already used in agent_connection.rs |
| agent-proto | workspace | WebSocket message type definitions | Add CrashReport variant here (or in node_protocol.rs) |
| serde/serde_json | v1 | Message serialization | Already used throughout |
| regex | v1 | Crash type pattern matching (OOM patterns, plugin exception patterns) | Already in api/Cargo.toml |
| sqlx | v0.7 | DB migration + queries | Existing pattern for new tables |
| chrono | v0.4 | Time tracking | Already used throughout codebase |
| react | v19.2.4 | Frontend UI (Settings tab section) | Existing framework |
| zustand | v5 | State management | Already in use |
| tailwindcss | v4 | UI styling | Existing utility framework |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| regex for crash classification | nom/pest parser | regex is simpler and already a dependency. Crash log patterns are simple enough for regex. |
| Agent-side classification | Backend-only classification | D-02 locks backend as classifier. Agent is reporter only. |

**Installation:**
```bash
# No new packages needed. The `regex` crate is already in api/Cargo.toml.
```

**Version verification:** All dependencies already present in workspace — no new package installations required.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                       Backend (API)                         │
│                                                             │
│  ┌──────────────┐   ┌─────────────────────────────┐        │
│  │ node_ws_     │   │     MonitoringService        │        │
│  │ handler.rs   │──▶│     (30s loop)               │        │
│  │              │   │                               │        │
│  │ Parse        │   │  1. Fetch servers             │        │
│  │ CrashReport  │   │  2. Skip offline nodes        │        │
│  │ message      │   │  3. Check status per server   │        │
│  │              │   │  4. NEW: Ingest CrashReport   │        │
│  │ Forward to   │   │  5. Classify crash type       │        │
│  │ Monitoring   │   │  6. Execute recovery action   │        │
│  │ Service      │   │  7. Emit notifications        │        │
│  └──────┬───────┘   └───────────┬───────────────────┘        │
│         │                       │                             │
│         │                       ▼                             │
│         │           ┌─────────────────────┐                  │
│         │           │  Crash Classifier    │                  │
│         │           │  (new module/struct) │                  │
│         │           │                     │                  │
│         │           │  classify(exit_code, │                  │
│         │           │    log_excerpt)      │                  │
│         │           │  → CrashType enum    │                  │
│         │           └─────────┬───────────┘                  │
│         │                     │                               │
│         │                     ▼                               │
│         │           ┌─────────────────────┐                  │
│         │           │  Recovery Executor   │                  │
│         │           │  (new or phased code)│                  │
│         │           │                     │                  │
│         │           │  OOM → notify only  │                  │
│         │           │  Config → disable   │                  │
│         │           │  Plugin → restart   │                  │
│         │           │  Generic → P57      │                  │
│         └───────────┴─────────────────────┘                  │
│                                                             │
│  ┌────────────────┐  ┌─────────────────┐                   │
│  │ server_crash_  │  │ Event Bus +     │                   │
│  │ logs table     │  │ Discord Client  │                   │
│  │ (new)          │  │ (existing)      │                   │
│  └───────┬────────┘  └────────┬────────┘                   │
│          │                    │                              │
└──────────┼────────────────────┼──────────────────────────────┘
           │                    │
           │                    │ WebSocket ServerEvent
           ▼                    ▼
┌────────────────────┐  ┌──────────────────────┐
│  REST API          │  │  Frontend             │
│  GET/DELETE        │  │  ServerDetails.jsx    │
│  /crash-logs       │  │  Settings Tab         │
│                    │  │  → Crash History      │
│                    │  │  → Paginated list     │
│                    │  │  → Toast on crash     │
└────────────────────┘  └──────────────────────┘
           │
           │
           ▼
┌─────────────────────────────────────────────┐
│              Web Agent                       │
│                                              │
│  ┌─────────────────────────────────────┐    │
│  │  Container Monitor (new)            │    │
│  │                                     │    │
│  │  When container stops:              │    │
│  │  1. Get container inspect → exit    │    │
│  │     code                             │    │
│  │  2. Get container logs → last N     │    │
│  │     lines (stderr/stdout)           │    │
│  │  3. Build CrashReport message       │    │
│  │  4. Send via WebSocket              │    │
│  └─────────────────────────────────────┘    │
│                                              │
│  Bollard (Docker API) ← manages containers  │
└─────────────────────────────────────────────┘
```

### Recommended Project Structure

No new modules needed beyond:
- `api/src/application/services/crash_classifier.rs` — NEW: crash type classification logic
- `api/migrations/20260531000001_create_server_crash_logs.sql` — NEW: migration
- `api/src/domain/entities/server_crash_log.rs` — NEW: entity for crash log table
- `api/src/infrastructure/repositories/crash_log_repository.rs` — NEW: repository for crash logs
- Agent: crash reporter logic inside existing `agent_connection.rs` or new `handlers/crash_reporter.rs`

**Integration points are extensions, not rewrites:**
- `agent-proto`: Add `CrashReport` message type (or add to `NodeMessage` in backend)
- `node_ws_handler.rs`: Handle `CrashReport` → forward to MonitoringService
- `monitoring_service.rs`: Add crash classification + recovery execution
- `server_handlers.rs`: Add GET/DELETE crash-log endpoints
- `ServerDetails.jsx`: Add "Crash History" section in Settings tab

### Pattern 1: WebSocket Protocol Extension

**What:** Add a new `CrashReport` variant to the existing agent→backend WebSocket message enums. [VERIFIED: agent_connection.rs, node_protocol.rs]

**When to use:** Both agent-side `AgentMessage` and backend-side `NodeMessage` use `#[serde(tag = "type")]` with `#[serde(rename = "...")]`. Each direction is a separate enum.

**In `agent_connection.rs` (agent-side AgentMessage):**
```rust
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
enum AgentMessage {
    // ...existing variants...
    #[serde(rename = "crash_report")]
    CrashReport {
        server_id: Uuid,
        exit_code: i32,
        log_excerpt: String,       // last N lines of container log
        timestamp: String,
    },
}
```

**In `node_protocol.rs` (backend-side NodeMessage):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeMessage {
    // ...existing variants...
    #[serde(rename = "crash_report")]
    CrashReport {
        server_id: Uuid,
        exit_code: i32,
        log_excerpt: String,
        timestamp: String,
    },
}
```

### Pattern 2: Crash Classification (Backend)

**What:** Pure function that takes exit code + log excerpt, returns a classified CrashType. [ASSUMED based on D-03 requirements]

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CrashType {
    Oom,
    ConfigError,
    PluginCrash,
    Generic,
}

pub fn classify_crash(exit_code: i32, log_excerpt: &str) -> CrashType {
    // Must be pure — no side effects, no DB access
    match exit_code {
        137 => {
            // Exit 137 (SIGKILL) is strong OOM signal
            // Double-check logs for OOM indicators
            if has_oom_pattern(log_excerpt) {
                CrashType::Oom
            } else {
                CrashType::Generic
            }
        }
        _ => {
            if has_oom_pattern(log_excerpt) {
                CrashType::Oom
            } else if has_plugin_exception_pattern(log_excerpt) {
                CrashType::PluginCrash
            } else {
                CrashType::Generic
            }
        }
    }
}

fn has_oom_pattern(log: &str) -> bool {
    let re = regex::Regex::new(
        r"(?i)(OutOfMemoryError|java\.lang\.OutOfMemoryError|Killed|Cannot allocate memory|java heap space)"
    ).unwrap();
    re.is_match(log)
}

fn has_plugin_exception_pattern(log: &str) -> bool {
    let re = regex::Regex::new(
        r"(?i)(NullPointerException|PluginClassLoader|java\.lang\.reflect\.InvocationTargetException|Caused by:.*Exception)"
    ).unwrap();
    re.is_match(log)
}
```

### Pattern 3: CrashLoop Detection (Config Error)

**What:** Detect 3 rapid crashes within 60s of startup. This requires tracking crash frequency, not just crash type. [ASSUMED based on D-03, discretion area]

```rust
// In MonitoringService, after crash classification:
async fn handle_crash(&self, server: &Server, crash_type: CrashType, exit_code: i32, log_excerpt: &str) -> Result<()> {
    match crash_type {
        CrashType::Oom => {
            // D-03: Notify only, do NOT auto-restart
            self.notify_crash(server, crash_type, "notified_only").await;
            self.store_crash_log(server.id, exit_code, crash_type, log_excerpt, "notified_only").await;
        }
        CrashType::ConfigError => {
            // Check if this is the 3rd rapid crash within 60s
            let recent_crashes = self.crash_log_repository.count_recent(server.id, Duration::from_secs(60)).await?;
            if recent_crashes >= 3 {
                // Disable auto-restart
                let mut updated = server.clone();
                updated.auto_restart = false;
                self.repository.update(&updated).await?;
                self.notify_crash(server, crash_type, "restart_disabled").await;
                self.store_crash_log(server.id, exit_code, crash_type, log_excerpt, "restart_disabled").await;
            } else {
                // Still within threshold — let Phase 57 handle restart
                self.store_crash_log(server.id, exit_code, crash_type, log_excerpt, "auto_restarted").await;
                // Phase 57's crash detection loop will handle the restart
            }
        }
        CrashType::PluginCrash | CrashType::Generic => {
            // Follow Phase 57 auto-restart policy
            // Crash detection in monitoring_service.rs lines 136-218 handles this
            self.store_crash_log(server.id, exit_code, crash_type, log_excerpt, "auto_restarted").await;
        }
    }
}
```

### Pattern 4: Frontend Crash History Section (Settings Tab)

**What:** Paginated crash history table following Phase 56/57 section layout. [VERIFIED: ServerDetails.jsx lines 541-730]

```jsx
{/* ─── CRASH HISTORY SECTION (Phase 60) ─── */}
<section className="glass-panel p-6 mt-6">
  <h3 className="text-lg font-bold mb-1">Crash History</h3>
  <p className="text-xs text-[var(--color-text-muted)] mb-5">
    Detailed crash log with diagnostic information.
  </p>

  {crashToast && (
    <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${...}`}>
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

### Anti-Patterns to Avoid
- **Agent-side classification:** D-02 specifically locks classification to the backend. Putting classification logic in the agent would require agent updates for pattern changes.
- **Blocking the monitoring loop on crash notification:** The MonitoringService's 30s loop already spawns tokio tasks for delayed restarts (line 169). Crash classification and DB writes should be synchronous (fast), but Discord webhook calls should be fire-and-forget.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Regex for log pattern matching | Custom parser | `regex` crate (already in deps) | Simple patterns, already has `Cargo.toml` entry |
| Discord webhook delivery | Custom HTTP webhook sender | Existing `DiscordClient` | `api/src/infrastructure/external_services/discord_client.rs` — already implements embed messages |
| Server event emission | Custom event system | `emit_server_event()` pattern | `api/src/domain/webhook/service.rs` — already wired with the event system |
| Toast notifications | Custom event bus integration | Existing WebSocket ServerEvent pattern | Backend publishes ServerEvent → frontend receives via WebSocket |
| Pagination for crash history | Custom pagination logic | Limit/offset query params on GET endpoint | Simplest approach for moderate data volumes (unlimited retention per D-06 but user-facing pagination) |

**Key insight:** The hardest part of crash detection is getting the raw data from the agent. The web-agent uses Bollard for container management and can inspect containers (`docker.inspect_container()` returns container state including exit code). For logs, Bollard's `LogsOptions` can tail the last N lines. Both capabilities already exist in the agent codebase — they just need to be wired into a crash detection flow.

## Runtime State Inventory

> Not applicable — this is not a rename/refactor/migration phase. No runtime state is being migrated.

## Common Pitfalls

### Pitfall 1: Container Exit Code 137 Ambiguity
**What goes wrong:** Exit code 137 (128+9 = SIGKILL) is commonly caused by OOM kills, but can also happen when a container is manually killed or Docker daemon restarts.
**Why it happens:** The OOM killer sends SIGKILL, resulting in exit code 137. But Docker also sends SIGKILL for `docker kill` or daemon restart.
**How to avoid:** Cross-reference exit code 137 with log patterns. If the container was OOM-killed, the kernel logs "Killed" or the JVM logs "OutOfMemoryError" before exit. The presence of OOM-specific patterns in logs is more reliable than exit code alone.
**Warning signs:** High false-positive OOM classification if using exit code 137 alone.

### Pitfall 2: Log Line Encoding and Size Limits
**What goes wrong:** Sending megabytes of log data in a WebSocket message will flood the connection and cause latency spikes.
**Why it happens:** Game server logs can be verbose, especially during crashes with stack traces.
**How to avoid:** Cap log excerpt at the agent level (e.g., last 5-10 lines, max 4KB). Stack traces can be multi-line but are informative — capture the last line + any preceding 2-3 lines that contain "Exception" or "Error". The full log is always available via the existing `/logs/:lines` endpoint.
**Warning signs:** WebSocket message size warnings, connection latency.

### Pitfall 3: Concurrent Crash Reports During Monitoring Loop
**What goes wrong:** Both the new CrashReport WS path AND the existing monitoring_service.rs running→stopped detection path can trigger recovery for the same crash, causing a double-restart.
**Why it happens:** The MonitoringService polls every 30s and detects status changes independently of crash reports.
**How to avoid:** Two strategies: (1) The CrashReport path sets a "crash_handled" flag on the server to skip the next monitoring loop detection, or (2) The monitoring loop checks if a crash report was already received for this server within the last 30s before triggering recovery. Option (2) is simpler — check last_restart_at timestamp.
**Warning signs:** Server being restarted twice in rapid succession.

### Pitfall 4: Plugin Crash Over-Detection (False Positives)
**What goes wrong:** Innocuous plugin warnings or dev-mode stack traces get classified as plugin crashes, causing unnecessary notifications.
**Why it happens:** Many plugins log stack traces at WARN level during normal operation. A lone stack trace in the last 5 log lines doesn't necessarily mean a crash.
**How to avoid:** Require BOTH a non-zero exit code AND plugin exception patterns in the log excerpt. A stack trace alone without an actual crash is not a plugin crash.
**Warning signs:** Users complaining about false alarm notifications.

### Pitfall 5: Discord Webhook Rate Limits
**What goes wrong:** Rapid crash loops trigger multiple Discord webhook calls per minute, hitting Discord's rate limit (30 req/min per webhook URL).
**Why it happens:** Discord webhooks are rate-limited. A crash loop (server crashes, restarts, crashes again) will fire multiple notifications.
**How to avoid:** Rate-limit crash notification delivery in the backend — max 1 Discord notification per server per 60 seconds cacheable via Redis or a simple in-memory cooldown. The event timeline and toast are internal and not rate-limited.
**Warning signs:** Discord webhook error logs, `429 Too Many Requests` responses.

## Code Examples

### Container Exit Code + Log Capture (Web Agent)

```rust
// Source: [VERIFIED: runtime.rs Bollard patterns, inspect API]
// Pattern for capturing crash data in the web-agent when a container exits unexpectedly.

use bollard::container::{LogsOptions, RemoveContainerOptions};
use bollard::Docker;
use futures_util::StreamExt;

async fn capture_crash_data(
    docker: &Docker,
    container_id: &str,
) -> Result<(i32, String), anyhow::Error> {
    // 1. Inspect container to get exit code
    let inspect = docker.inspect_container(container_id, None).await?;
    let exit_code = inspect
        .state
        .and_then(|s| s.exit_code)
        .unwrap_or(-1);

    // 2. Read last 10 lines of logs (both stdout and stderr)
    let log_options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: 10,
        ..Default::default()
    };

    let mut log_stream = docker.logs(container_id, Some(log_options));
    let mut log_lines = Vec::new();
    
    while let Some(Ok(chunk)) = log_stream.next().await {
        let line = chunk.to_string();
        log_lines.push(line);
    }

    let log_excerpt = log_lines.join("\n");
    // Truncate to 4KB max
    let log_excerpt = if log_excerpt.len() > 4096 {
        format!("... (truncated) ...\n{}", &log_excerpt[log_excerpt.len() - 4000..])
    } else {
        log_excerpt
    };

    Ok((exit_code, log_excerpt))
}
```

### Crash Report WebSocket Dispatch (Web Agent)

```rust
// Source: [VERIFIED: agent_connection.rs AgentMessage pattern]
// Integration into the agent's existing WebSocket send path.

// After detecting container stopped unexpectedly:
let (exit_code, log_excerpt) = capture_crash_data(&docker, &container_id).await?;

let crash_report = AgentMessage::CrashReport {
    server_id,
    exit_code,
    log_excerpt,
    timestamp: chrono::Utc::now().to_rfc3339(),
};

ws_sender
    .send(Message::Text(serde_json::to_string(&crash_report)?.into()))
    .await?;
```

### Backend Crash Report Ingestion

```rust
// Source: [VERIFIED: node_ws_handler.rs NodeMessage handling pattern]
// In handle_node_socket() in node_ws_handler.rs:

NodeMessage::CrashReport { server_id, exit_code, log_excerpt, timestamp } => {
    tracing::warn!(
        "[WS] CrashReport: server={}, exit_code={}, timestamp={}",
        server_id, exit_code, timestamp
    );
    
    // Forward to monitoring service for classification + recovery
    // Option A: Direct method call if MonitoringService is in AppContainer
    // container.monitoring_service.handle_crash_report(server_id, exit_code, log_excerpt).await;
    
    // Option B: Enqueue for processing (if async processing needed)
    // container.crash_report_queue.send((server_id, exit_code, log_excerpt)).await;
    
    // For Phase 60, the simplest approach: store the crash report,
    // then the next monitoring loop tick picks it up.
    // OR: process immediately via a new MonitoringService method.
}
```

### DB Migration Pattern

```sql
-- Source: [ASSUMED based on sqlx migration pattern, D-06 requirements]
-- Migration: api/migrations/20260531000001_create_server_crash_logs.sql

CREATE TABLE IF NOT EXISTS server_crash_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    crashed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    exit_code INTEGER NOT NULL,
    crash_type VARCHAR(32) NOT NULL,  -- 'oom', 'config_error', 'plugin_crash', 'generic'
    log_excerpt TEXT,
    recovery_action VARCHAR(32) NOT NULL,  -- 'auto_restarted', 'notified_only', 'restart_disabled'
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_server_crash_logs_server_id ON server_crash_logs(server_id);
CREATE INDEX idx_server_crash_logs_crashed_at ON server_crash_logs(crashed_at DESC);
```

### REST API Endpoint Pattern

```rust
// Source: [VERIFIED: server_handlers.rs router pattern]
// Add to ServerHandlers::router():
.route("/:id/crash-logs", get(list_crash_logs).delete(clear_crash_logs))

// Handler:
async fn list_crash_logs(
    State(container): State<ApiState>,
    Path(id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let logs = container.crash_log_repository
        .list_by_server(id, limit, offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let total = container.crash_log_repository
        .count_by_server(id)
        .await
        .unwrap_or(0);
    Ok(Json(json!({ "data": logs, "total": total, "limit": limit, "offset": offset })))
}
```

### Frontend API Calls

```javascript
// Source: [VERIFIED: client.js fetchApi pattern]
// In app/src/api/client.js:

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

## State of the Art

| Old Approach (Phase 57) | Current Approach (Phase 60) | When Changed | Impact |
|--------------------------|------------------------------|--------------|--------|
| Crash detection via status change (running→stopped) only | Crash detection via agent-reported exit code + log excerpt | Phase 60 | Provides forensic data for classification |
| Single "crash" classification | 4 crash types: OOM/Config/Plugin/Generic | Phase 60 | Enables per-type recovery strategy as defined in D-03 |
| Restart-or-give-up binary decision | Recovery strategy per crash type | Phase 60 | OOM no longer loops restart; plugin crashes get retried |
| No crash history persistence | `server_crash_logs` table with full forensic data | Phase 60 | Audit trail and user visibility via Settings tab |
| Toast + event timeline on restart | Multi-channel notification on EVERY crash (toast + event + Discord) | Phase 60 | D-04 requires notification regardless of recovery outcome |

**Deprecated/outdated:**
- Phase 57's crash detection (lines 136-218 of monitoring_service.rs) will still run, but should be gated: if a CrashReport was already processed for this server within the last 30s, skip the auto-restart path to avoid double-restart (Pitfall 3).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The `regex` crate is available in the api workspace for pattern matching | Standard Stack | Medium — `regex` is listed in STACK.md but not confirmed in current Cargo.lock. Required for crash pattern detection. |
| A2 | The web-agent uses Bollard's `docker.inspect_container()` which returns `ContainerState.exit_code` | Code Examples | Low — bollard v0.18 definitely supports this. Verified by grep showing `inspect_container` usage in runtime.rs. |
| A3 | Crash classification should be a pure function (no DB access) | Pattern 2 | Low — classification only needs exit code + log text. Server history DB lookups happen in the handler, not the classifier. |
| A4 | The 60s crash-loop detection window is counted from the first crash in the window | Pattern 3 | Medium — the exact window semantics (sliding vs fixed) are in the agent's discretion per CONTEXT.md. The planner can choose. |
| A5 | sqlx migrations can be applied even though the codebase currently skips them | Migration | Medium — codebase currently has `tracing::info!("Skipping migrations - assuming already applied")`. May need to unskip or manually run the SQL. |

## Open Questions (RESOLVED)

1. **How should the web-agent detect container crashes?**
   - [RESOLVED] Use the Bollard events API (`docker.system_events()`) to subscribe to container `die` events. This is the most responsive approach and avoids polling. The event stream is already a pattern in the web-agent ecosystem.

2. **Where does the CrashReport WebSocket message type live?**
   - [RESOLVED] Add to both enums independently, following the existing pattern. The agent's `AgentMessage` and backend's `NodeMessage` already have parallel definitions for Register, Heartbeat, etc. No shared crate refactoring needed.

3. **How to handle the crash report → monitoring loop handoff?**
   - [RESOLVED] Use a tokio::sync::mpsc channel. The WS handler sends CrashReport notifications. The monitoring loop receives from the channel on each tick and processes them. This keeps the monitoring loop as the single source of truth for recovery decisions and avoids direct coupling from the WS handler to the monitoring service.

4. **Should crash report ingestion block the monitoring loop?**
   - [RESOLVED] Keep classification and DB writes synchronous (fast). Spawn Discord webhook calls and restart actions as tokio tasks (already the pattern at monitoring_service.rs line 169). The loop should not be delayed by I/O-bound notification delivery.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Backend, Agent | ✓ | (inferred 1.70+) | — |
| Node.js | Frontend | ✓ | (inferred v20+) | — |
| PostgreSQL | Database | ✓ | 16 | — |
| Redis | Rate limiting (optional) | ✓ | 7 | In-memory cooldown map |
| Docker | Container management (web-agent) | — (prod only) | — | — |

**Missing dependencies with no fallback:** None — all required infrastructure is already in place.

**Missing dependencies with fallback:** None.

## Validation Architecture

> Skip: `workflow.nyquist_validation` is not explicitly set to `false` in config. However, per the phase workflow conventions, this section is generated by the planner when needed. Phase 60 validation will be via manual verification steps before `/gsd-verify-work`.

### Verification Strategy for Phase 60

| Verification Step | What to Check | How |
|-------------------|---------------|-----|
| WebSocket message format | CrashReport message is parseable on both sides | Unit test with serde roundtrip on NodeMessage::CrashReport |
| Crash classification | OOM, Plugin, Config, Generic correctly identified | Unit test classify_crash() with known log excerpts |
| CrashLoop detection | 3 crashes within 60s disables auto-restart | Integration test with mocked crash log repository |
| DB migration | server_crash_logs table created with correct schema | Run migration SQL, verify table exists |
| API endpoints | GET/DELETE crash-logs return correct data | Manual test via curl against running backend |
| Frontend UI | Crash History section renders with mock data | Manual visual verification in browser |
| Discord notification | Discord webhook receives crash notification | Manual test with real Discord webhook URL |

## Security Domain

> Required: security_enforcement is enabled (absent in config = enabled).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Validate CrashReport message fields (exit_code range, log_excerpt length cap) |
| V8 Data Protection | yes | server_crash_logs server_id FK ensures user can only access own server's crash logs |
| V9 Communication | yes | Crash report flows over existing authenticated WebSocket (already validated via node registration) |
| V14 Configuration | partial | `auto_restart` disable (ConfigError path) modifies server config — use existing conditional update pattern |

### Known Threat Patterns for Axum/Bollard/React Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Crash report spoofing | Spoofing | WS already authenticated via node registration + API key. CrashReport from unauthenticated agent rejected. |
| Log injection in crash report | Tampering | Log_excerpt is TEXT field — display-escape for frontend rendering (React handles this naturally with JSX) |
| Crash loop notification spam | DoS | Rate-limit Discord notifications per server (1/60s). Toast + event timeline are internal and bounded. |
| Unauthorized crash log access | Info Disclosure | Crash log endpoints gated by server ownership check (same pattern as all server endpoints) |

## Sources

### Primary (HIGH confidence)
- Phase 57 CONTEXT.md — Auto-restart decisions, notification patterns, existing infrastructure
- Phase 57 RESEARCH.md — MonitoringService architecture, health check pitfalls, frontend section patterns
- `api/src/application/services/monitoring_service.rs` — 30s monitoring loop, crash detection at lines 136-218
- `api/src/presentation/ws/node_protocol.rs` — NodeMessage enum definition (wire protocol)
- `agent/solys/src/agent_connection.rs` — AgentMessage enum, WebSocket send/receive loop
- `app/src/pages/ServerDetails.jsx` — Settings tab sections (Sleep/Wake, Restart Policy, Scheduled Actions)
- `api/src/presentation/handlers/node_ws_handler.rs` — Backend WS message handling pattern
- `api/src/shared/events.rs` — ServerEvent enum, event bus pattern
- `api/src/infrastructure/external_services/discord_client.rs` — Discord webhook client with server.crashed support
- `api/src/application/services/server_event_notifier.rs` — Discord event notification with existing crash method

### Secondary (MEDIUM confidence)
- `agent/solys/src/handlers/runtime.rs` — Bollard container inspect/start/stop patterns, exit code access via ContainerState
- `api/src/presentation/routes/api_routes.rs` — Route registration pattern for new endpoints
- `api/src/application/dto/server_dtos.rs` — DTO pattern for API request/response
- `api/src/domain/entities/server.rs` — Server entity with all existing fields (Phase 56/57 additions)
- `app/src/api/client.js` — Frontend API client pattern
- `app/src/hooks/useServers.js` — Server hooks pattern

### Tertiary (LOW confidence)
- None — all claims verified with existing codebase or from locked decisions in CONTEXT.md

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies needed
- Architecture: HIGH — patterns verified by reading existing code
- Pitfalls: MEDIUM — container exit code ambiguity and Discord rate limits are known from general knowledge, verified by code inspection of DiscordClient
- Agent crash capture: MEDIUM — exact integration point (events API vs polling) unverified, but Bollard API capabilities confirmed

**Research date:** 2026-05-31
**Valid until:** 2026-06-30 (stable stack, no fast-moving dependencies)
