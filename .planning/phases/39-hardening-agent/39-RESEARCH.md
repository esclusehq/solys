# Phase 39: HARDENING AGENT - Research

**Researched:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Stabilize the web-agent before distribution to nodes — implement production-grade configuration, logging, error handling, state persistence, and health monitoring systems.

</domain>

<decisions>
## Implementation Decisions (from CONTEXT.md)

### Config System
- **D-01:** Use TOML format for config file
- **D-02:** Default config path: `~/.config/escluse-agent/config.toml` (XDG standard)
- **D-03:** Fallback to `~/.local/share/escluse-agent/` if home-based
- **D-04:** Environment variable overrides with `ESCLUSE_AGENT_` prefix
- **D-05:** Both file + env merged — env takes precedence over file

### Logging System
- **D-06:** Primary log location: `/var/log/escluse-agent/agent.log`
- **D-07:** Fallback: `~/.local/share/escluse-agent/logs/`
- **D-08:** Last fallback: stdout (for containerized environments)
- **D-09:** Log rotation: By size (10MB) + age (7 days), keep 5 files
- **D-10:** Optional: Support external logrotate for advanced users
- **D-11:** Full 5-level log support: trace, debug, info, warn, error

### Error Handling
- **D-12:** Retry strategy: Exponential backoff with jitter + max cap
- **D-13:** Retry on transient failures only (network, timeout) — not on validation/auth errors
- **D-14:** Global default timeout: 30 seconds
- **D-15:** Per-operation timeout overrides supported
- **D-16:** Support cancellation via tokio
- **D-17:** Graceful failure: Controlled shutdown with supervised restart (hybrid production mode)
- **D-18:** No panics in production — log error, cleanup, exit with specific code

### State Persistence
- **D-19:** Persist: Server list + container mapping + important metadata (health, restart_count)
- **D-20:** Exclude: Transient data (connection history, temporary buffers)
- **D-21:** Format: JSON with atomic write (write to temp, then rename)
- **D-22:** Optional compression for large state files (lightweight hybrid)
- **D-23:** Auto-recovery: Load state → reconnect to backend → verify containers → reconcile

### Health System
- **D-24:** `/health` endpoint returns full status (component health + agent status + uptime)
- **D-25:** Self-checks: Podman availability, disk space (for logs/state), available RAM
- **D-26:** Component health: Each check returns status (healthy/degraded/unavailable)

</decisions>

<research>
## Technical Approach

### 1. Config System (TOML + XDG + Env Override)

**Existing Assets:**
- `agent-config` crate in `agent-core/crates/agent-config/` — already handles env var loading
- Current loader reads from `.env` files and env vars with `AGENT_` prefix

**Changes Required:**
- Add `toml` crate for TOML parsing
- Extend `agent-config` to support XDG paths (use `dirs` crate for `config_local_dir()`)
- Change env prefix from `AGENT_` to `ESCLUSE_AGENT_` per D-04
- Support config precedence: env var > file > default (per D-05)

**Implementation Path:**
```rust
// New config loading in loader.rs
fn load_toml_config(config: &mut AgentConfig) {
    let config_path = get_xdg_config_path("config.toml");
    if let Some(path) = config_path {
        let contents = std::fs::read_to_string(path).ok()?;
        let toml: TomlConfig = toml::from_str(&contents).ok()?;
        // Apply to config
    }
}
```

### 2. Logging System (File + Rotation)

**Existing Assets:**
- `tracing-subscriber` already used in web-agent/main.rs
- Current: stdout-only logging via `tracing_subscriber::fmt().init()`

**Changes Required:**
- Add `tracing-appender` for non-blocking file logging
- Implement rotation by size (10MB) + age (7 days) per D-09
- Support 5-level logs: trace, debug, info, warn, error

**Implementation Path:**
```rust
// File appender with rotation
let (non_blocking, _guard) = tracing_appender::non_blocking(
    tracing_appender::rolling::builder()
        .rotation(tracing_appender::rotation::DAILY)
        .max_log_files(5)
        .build("/var/log/escluse-agent")
        .unwrap_or_else(|| tracing_appender::rolling::daily("./logs"))
);

// Keep guard alive for duration of program
std::mem::forget(_guard);
```

### 3. Error Handling (Retry + Timeouts + Graceful Shutdown)

**Existing Assets:**
- `agent-health/retry.rs` — already implements exponential backoff with jitter (D-12 to D-17)
- Config has `task_timeout_default_secs` = 300 (5 min default)
- Graceful shutdown already implemented via `Arc<AtomicBool>` in main.rs lines 82-109

**Changes Required:**
- Extend retry logic to filter transient vs permanent errors (D-13)
- Add per-operation timeout overrides (D-15)
- Ensure `panic = "abort"` is NOT set in Cargo.toml (currently it's set for release, need to remove)
- Implement proper error propagation instead of `.unwrap()`

**Implementation Path:**
```rust
// Retry on transient errors only
async fn with_retry<T, E>(
    op: impl Fn() -> impl Future<Output = Result<T, E>>
) -> Result<T, E> {
    // Use agent_health::retry_with_backoff but filter errors
    // NetworkError, TimeoutError -> retry
    // ValidationError, AuthError -> fail immediately
}
```

### 4. State Persistence (JSON + Atomic Write)

**Existing Assets:**
- None currently — this is net new

**Changes Required:**
- Create state directory in XDG data dir
- JSON format for server list + container mapping (D-19 to D-20)
- Atomic write via temp file + rename pattern (D-21)
- Auto-recovery on startup (D-23)

**Implementation Path:**
```rust
// Atomic write pattern
async fn save_state(state: &AgentState) -> Result<(), Error> {
    let temp_path = state_path.with_extension("tmp");
    let json = serde_json::to_string_pretty(state)?;
    tokio::fs::write(&temp_path, json).await?;
    tokio::fs::rename(&temp_path, &state_path).await?;  // Atomic on POSIX
    Ok(())
}
```

### 5. Health System (Enhanced)

**Existing Assets:**
- `/health` endpoint already exists in `api/routes.rs`
- Current: returns hardcoded "ok" + runtime + connected
- `sysinfo` crate already imported

**Changes Required:**
- Add self-checks: Podman availability, disk space, RAM per D-25
- Component health per D-26: healthy/degraded/unavailable
- Extend to show uptime, restart count, last error

**Implementation Path:**
```rust
#[derive(serde::Serialize)]
pub struct HealthResponse {
    status: String,  // "healthy" | "degraded" | "unavailable"
    components: Vec<ComponentHealth>,
    agent_status: AgentStatus,
    uptime_secs: u64,
}

#[derive(serde::Serialize)]
pub struct ComponentHealth {
    name: String,
    status: String,  // "healthy" | "degraded" | "unavailable"
    message: Option<String>,
}
```

</research>

<validation_architecture>
## Validation Strategy

### Plan Structure

This phase is suitable for 2-3 parallel plans by feature area:

**Plan 1: Config + Logging System**
- TOML config file support
- XDG path resolution
- File logging with rotation
- Env var override prefix change

**Plan 2: Error Handling + Retry**  
- Extend retry logic for transient errors
- Timeout configuration
- Proper error handling (no unwrap)
- Graceful shutdown enhancement

**Plan 3: State Persistence**
- JSON state file with atomic write
- Auto-recovery on startup
- Server list + container mapping persistence

**Plan 4: Health Monitoring (if needed)**
- Enhanced /health endpoint
- Self-checks (disk, RAM, runtime)
- Component health reporting

### Verification Criteria

Each plan should verify:
1. Config loads correctly from TOML + env
2. Logs appear in correct location with rotation
3. Retry logic works for transient errors
4. State persists across restart
5. Health endpoint returns meaningful data

</validation_architecture>

<implementation_notes>
## Key Implementation Details

### Use Existing Crates

| Decision | Implementation |
|----------|----------------|
| D-12 to D-17 | Extend `agent-health/src/retry.rs` — already has `retry_with_backoff` |
| Logging | Use `tracing-appender` (new dependency) |
| TOML | Add `toml` crate to web-agent |
| State write | Use `tokio::fs` with rename (atomic) |
| Health checks | Use `sysinfo` crate (already included) |

### File Modifications

Primary files to modify:
1. `agent-core/crates/agent-config/src/loader.rs` — TOML + XDG support
2. `agent-core/crates/agent-config/src/schema.rs` — Add log config
3. `web-agent/src/main.rs` — File logging setup
4. `web-agent/src/api/routes.rs` — Enhanced health
5. New file: `web-agent/src/state.rs` — State persistence
6. `web-agent/Cargo.toml` — Add `tracing-appender`

### Test Strategy

Verify by:
1. Checking config file loads from XDG path
2. Checking logs appear in `/var/log/escluse-agent/`
3. Simulating transient failures and verifying retry
4. Killing agent and verifying state recovery
5. Checking `/health` returns all components

</implementation_notes>

---

*Phase: 39-hardening-agent*
*Researched: 2026-05-03*