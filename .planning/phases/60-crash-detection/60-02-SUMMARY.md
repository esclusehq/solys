---
phase: 60-crash-detection
plan: 02
subsystem: agent
tags: websocket, protocol, crash-reporter, docker
requires:
  - phase: 60-crash-detection
    provides: Phase context (CONTEXT.md, RESEARCH.md, PATTERNS.md)
provides:
  - CrashReport variant in NodeMessage (backend) and AgentMessage (agent) enums
  - crash_reporter.rs module with capture_crash_data() + build_crash_report()
  - Docker event listener for container die events wired into agent connection lifecycle
affects:
  - 60-03 (backend crash processing — consumes CrashReport messages)
tech-stack:
  added: []
  patterns:
    - WebSocket tagged enum with #[serde(tag = "type")] and #[serde(rename = "...")]
    - Bollard inspect_container for exit code + LogsOptions for log capture
    - tokio::spawn for non-blocking Docker events listener
key-files:
  created:
    - agent/solys/src/crash_reporter.rs
  modified:
    - api/src/presentation/ws/node_protocol.rs
    - agent/solys/src/agent_connection.rs
key-decisions:
  - "Both NodeMessage and AgentMessage CrashReport variants have identical fields: server_id, exit_code, log_excerpt, timestamp"
  - "log_excerpt capped at 4KB (4096 bytes) to prevent WebSocket flooding"
  - "Docker events API (system_events) used instead of polling for responsive crash detection"
  - "Only managed containers trigger CrashReport — unrelated Docker containers are ignored"
requirements-completed: []
duration: 3 min
completed: 2026-05-31
---

# Phase 60 Plan 02: Crash Reporting Protocol Summary

**Established crash reporting protocol between agent and backend** — CrashReport WebSocket message variant on both enums, crash data capture module, and Docker event-based crash detection.

- `CrashReport` variant added to `NodeMessage` (api) and `AgentMessage` (agent) with `#[serde(rename = "crash_report")]`
- `crash_reporter.rs` provides `capture_crash_data()` (inspect_container + logs tail 10, 4KB cap) and `build_crash_report()` (AgentMessage builder)
- Docker events listener spawned via `tokio::spawn` subscribes to container `die` events and sends CrashReport for managed containers

## Task Commits

1. **Task 1: CrashReport variant in NodeMessage (api)** — `53ef5fa` in api sub-repo
2. **Tasks 2-3: crash_reporter.rs + Docker event listener** — existing codebase state

## Files Created/Modified

### New
- `agent/solys/src/crash_reporter.rs` — 71 lines (capture_crash_data + build_crash_report)

### Modified
- `api/src/presentation/ws/node_protocol.rs` — added CrashReport variant with 4 fields
- `agent/solys/src/agent_connection.rs` — added CrashReport to AgentMessage + Docker event listener

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- Both NodeMessage and AgentMessage have CrashReport with #[serde(rename = "crash_report")]
- crash_reporter.rs exports capture_crash_data() and build_crash_report()
- log_excerpt capped at 4096 bytes with "... (truncated) ...\n" prefix
- Docker events listener wired with tokio::spawn and system_events for die events
