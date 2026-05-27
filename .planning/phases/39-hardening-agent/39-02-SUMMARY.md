---
phase: 39-hardening-agent
plan: 02
type: execute
subsystem: web-agent
tags: [logging, file-logging, rotation]
dependency_graph:
  requires: []
  provides: [file-logging]
  affects: [web-agent, agent-config]
tech_stack:
  added:
    - tracing-appender 0.2
  patterns:
    - Log rotation via tracing_appender::rolling::daily
    - Guard lifetime management with std::mem::forget
    - Fallback to stdout for containerized environments
key_files:
  created: []
  modified:
    - agent-core/crates/agent-config/Cargo.toml
    - agent-core/crates/agent-config/src/lib.rs
    - agent-core/crates/agent-config/src/loader.rs
    - web-agent/src/main.rs
decisions:
  - "Use /var/log/escluse-agent/ as primary log path (D-06)"
  - "Fallback to stdout when file paths unavailable (D-08)"
  - "Daily rotation via tracing_appender (D-09)"
metrics:
  duration: ~2 min
  completed: 2026-05-03
---

# Phase 39 Plan 02: File-based Logging with Rotation Summary

## One-liner

Implemented file-based logging with daily rotation for web-agent, with fallback to stdout for containerized environments.

## Completed Tasks

| Task | Name | Status | Commit |
|------|------|--------|--------|
| 1 | Add log directory path resolution | ✅ | 8f6122f |
| 2 | Initialize file logging in main.rs | ✅ | 8f6122f |
| 3 | Configure log rotation (daily) | ✅ | 8f6122f |

## Implementation Details

### Files Modified

1. **agent-core/crates/agent-config/Cargo.toml**
   - Added `tracing-appender = "0.2"` dependency

2. **agent-core/crates/agent-config/src/loader.rs**
   - Added `get_log_dir()` function with fallback paths
   - Added `FileLogGuard` struct for guard lifetime management
   - Added `get_log_writer()` for alternative access

3. **agent-core/crates/agent-config/src/lib.rs**
   - Exported `get_log_dir`, `get_log_writer`, and `FileLogGuard`

4. **web-agent/src/main.rs**
   - Replaced simple tracing_subscriber::fmt() with file logging setup
   - Uses `/var/log/escluse-agent/` as primary path
   - Falls back to stdout for containers
   - Keeps guard alive with std::mem::forget
   - Strips ANSI codes from log files

### Key Behaviors

- **Primary path**: `/var/log/escluse-agent/agent.log` (D-06)
- **Fallback**: stdout for containers (D-08)
- **Rotation**: Daily via tracing_appender (D-09)
- **ANSI stripping**: Disabled for log files

## Deviations from Plan

### Auto-fixed Issues

None - implementation follows plan closely.

### Design Decisions

1. **Inline logging setup in main.rs**: Used inline code instead of `get_log_dir()` helper due to trait bound issues with guard lifetime management when passing through function return values. The helper functions remain available in agent-config for future use.

2. **No explicit size rotation**: The plan mentioned 10MB size rotation (D-09), but tracing-appender 0.2 only supports time-based rotation. Size rotation would require external logrotate config (D-10).

## Verification

- [x] Log directory resolution with fallbacks per D-06, D-07, D-08
- [x] File logging initialized in main.rs
- [x] Rotates daily with files kept by appender
- [x] Stdout fallback for containers
- [x] Build successful

## Known Stubs

None - all functionality implemented.

## Threat Flags

None - logging changes don't introduce new security surface.

---

*Plan: 39-hardening-agent-02*
*Completed: 2026-05-03*