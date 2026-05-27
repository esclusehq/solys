# Phase 39: HARDENING AGENT - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-03
**Phase:** 39-hardening-agent
**Areas discussed:** Config system, Logging system, Error handling, State persistence, Health system

---

## Config System

| Option | Description | Selected |
|--------|-------------|----------|
| TOML format | Standard Rust config format, well-supported, hierarchical sections | ✓ |
| JSON format | Familiar, but less readable for configs | |
| YAML format | More flexible but requires extra dependency | |

**User's choice:** TOML format (recommended)
**Notes:** Config file structure decided

| Option | Description | Selected |
|--------|-------------|----------|
| ~/.config/escluse-agent/config.toml | Standard XDG config location, per-user home directory | ✓ |
| /etc/escluse-agent/config.toml | System-wide location, requires root | |
| Same directory as binary | Simple, but not ideal for multi-user systems | |

**User's choice:** ~/.config/escluse-agent/config.toml (recommended)
**Notes:** Default config path decided

| Option | Description | Selected |
|--------|-------------|----------|
| Prefix-based (ESCLUSE_AGENT_*) | Clear namespace, easy to document | |
| Exact match | Same name as config key | |
| Both file + env merged | Env takes precedence over file | ✓ |

**User's choice:** Both file + env merged with Prefix-based env
**Notes:** Environment variable override approach decided

---

## Logging System

| Option | Description | Selected |
|--------|-------------|----------|
| /var/log/escluse-agent/agent.log | Standard system log location | ✓ |
| ~/.local/share/escluse-agent/logs/ | User home directory, no permissions needed | ✓ |
| Same directory as binary | Simple but may have permission issues | ✓ |

**User's choice:** Primary: /var/log/escluse-agent/, Fallback: ~/.local/share/escluse-agent/logs/, Last fallback: stdout
**Notes:** Multi-tier logging with fallbacks

| Option | Description | Selected |
|--------|-------------|----------|
| By size + age | Rotate when >10MB or >7 days old, keep 5 files | ✓ |
| By size only | Rotate when file reaches size limit | |
| By time (daily) | New file each day | |
| External tool (logrotate) | Delegate to system logrotate | ✓ |

**User's choice:** Default: size + age (built-in), Advanced: logrotate (optional)
**Notes:** Log rotation approach decided

| Option | Description | Selected |
|--------|-------------|----------|
| Full 5-level (trace, debug, info, warn, error) | Most control | ✓ |
| 4-level (no trace) | Simpler for most users | |
| 3-level (production) | Minimal for production servers | |

**User's choice:** Full 5-level (recommended)
**Notes:** Log level support decided

---

## Error Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Exponential backoff | 2s → 4s → 8s → 16s with max 5 retries | ✓ |
| Linear backoff | 2s → 4s → 6s | |
| Fixed interval | Same delay between retries | |
| Fibonacci backoff | Slower growth, gentler on network | ✓ |

**User's choice:** Exponential backoff + jitter (WAJIB) + max cap + selective retry
**Notes:** Retry strategy with jitter and caps

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable default | Set sensible default (30s), allow override | ✓ |
| Per-operation timeouts | Different timeout for each operation | |
| Infinite timeout | Never timeout — dangerous | |

**User's choice:** Global default (30s) + Per-operation override + Support cancellation
**Notes:** Timeout configuration approach decided

| Option | Description | Selected |
|--------|-------------|----------|
| Controlled shutdown | Log error, cleanup, exit with specific code | ✓ |
| Restart loop | Attempt to restart itself after crash | |
| Panic with message | Standard Rust panic | |

**User's choice:** 1. Controlled shutdown — dengan supervised restart (hybrid production)
**Notes:** Graceful failure handling decided

---

## State Persistence

| Option | Description | Selected |
|--------|-------------|----------|
| Server list + container mapping | Track active servers and their container IDs | ✓ |
| Full state | All operational state | |
| Minimal state | Only server list | |

**User's choice:** Server list + container mapping + Tambah metadata penting (health, restart_count) + Hindari data transient & besar
**Notes:** What state to persist decided

| Option | Description | Selected |
|--------|-------------|----------|
| JSON | Human-readable, easy to debug | ✓ |
| JSON with compression | Smaller file size but harder to inspect | ✓ |
| MessagePack/CBOR | More compact but less readable | |

**User's choice:** 1. JSON — dengan atomic write + optional compression (hybrid ringan)
**Notes:** State file format decided

| Option | Description | Selected |
|--------|-------------|----------|
| Reconnect and verify | Load state → reconnect → verify containers → reconcile | ✓ |
| Fresh start | Clear state on restart | |
| Container-first | Scan existing containers first | |

**User's choice:** Reconnect and verify (recommended)
**Notes:** Auto-recovery approach decided

---

## Health System

| Option | Description | Selected |
|--------|-------------|----------|
| Full status | Component health + agent status + uptime | ✓ |
| Simple OK | Just returns 200 OK | |
| Detailed metrics | Full system metrics + component status | |

**User's choice:** Full status (recommended)
**Notes:** Health endpoint response decided

| Option | Description | Selected |
|--------|-------------|----------|
| Podman + Disk + Memory | Container runtime, disk space, available RAM | ✓ |
| Podman only | Just verify container runtime is available | |
| Full system check | Also check network, backend connectivity | |

**User's choice:** Podman + Disk + Memory (recommended)
**Notes:** Self-checks decided

---

## Deferred Ideas

No deferred ideas — all discussion stayed within phase scope.