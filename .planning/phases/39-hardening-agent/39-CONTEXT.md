# Phase 39: HARDENING AGENT - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Stabilize the web-agent before distribution to nodes — implement production-grade configuration, logging, error handling, state persistence, and health monitoring systems.

</domain>

<decisions>
## Implementation Decisions

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

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `agent-config` crate in `agent-core/crates/agent-config/` — config loading/validation foundation
- `tracing` crate already used in web-agent/main.rs for logging
- `bollard` crate already imported for Podman/Docker operations

### Established Patterns
- Config validation with error collection (see main.rs line 26-31)
- Tracing subscriber with max_level filter (main.rs line 36-39)
- Graceful shutdown using Arc<AtomicBool> pattern (main.rs lines 82-109)
- Async signal handling with tokio (main.rs lines 86-92)

### Integration Points
- Config loaded early in main() before any other initialization
- Logging initialized after config (line 34-39)
- Audit logger spawned as background task (line 49-52)
- Shutdown handling wraps agent connection run

</code_context>

<specifics>
## Specific Ideas

No specific references — all approaches are standard patterns for production-grade agents.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 39-hardening-agent*
*Context gathered: 2026-05-03*