---
phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe
plan: 03
subsystem: api
tags: relay, websocket, agent-connection, relay-config-sync
requirements: []

# Dependency graph
requires:
  - phase: 70-02
    provides: RelaySessionState, ServerRelayInfo state types, set_relay_session_state(), apply_relay_config()
provides:
  - BackendMessage::RelayConfigSync variant on agent's BackendMessage enum
  - Handler arm in message loop for RelayConfigSync processing
  - ServerRelayInfo deserialization struct in agent_connection.rs
affects: [future relay config plans]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Type conversion pattern: local deserialization struct → state::ServerRelayInfo
    - Non-fatal error handling for WS push processing (warn! level)
    - Legacy RelayConfig bridge for backward compat with run_relay_client()

key-files:
  created: []
  modified:
    - src/agent_connection.rs — RelayConfigSync variant + handler + ServerRelayInfo

key-decisions:
  - "Local ServerRelayInfo struct in agent_connection.rs for serde deserialization, mapped to state::ServerRelayInfo for storage and apply_relay_config()"
  - "All errors in RelayConfigSync handler are non-fatal — existing tunnels continue on failure"
  - "Legacy RelayConfig bridge only activated when relay_config().is_none() (no AGENT_RELAY_TOKEN env var)"

patterns-established: []

# Metrics
duration: 8 min
completed: 2026-06-09
---

# Phase 70: Auto-fetch via WS — Plan 03 Summary

**RelayConfigSync WS push handler: deserialize relay config from backend, store session state, bridge legacy config, and diff-update tunnels via apply_relay_config()**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-09T01:00:00Z
- **Completed:** 2026-06-09T01:08:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `BackendMessage::RelayConfigSync` variant to the agent's `BackendMessage` enum with all 4 fields (relay_token, gateway_url, region, servers)
- Added `ServerRelayInfo` deserialization struct (module-private) for JSON WS parsing
- Added full handler arm in the message loop that: stores `RelaySessionState` for diagnostic access; bridges legacy `RelayConfig` from combined global config + WS push when no env var set; calls `apply_relay_config()` for diff-based tunnel hot-update
- All errors in the handler are non-fatal — logged at `warn!` level, existing tunnels continue

## Task Commits

Each task was committed atomically:

1. **Task 1: Add BackendMessage::RelayConfigSync variant + ServerRelayInfo struct** - `2a1e42d` (feat)
2. **Task 2: Add handler arm for RelayConfigSync in message loop** - `5137215` (feat)

**Plan metadata:** (committed with Task 2)

## Files Created/Modified

- `src/agent_connection.rs` — Added 69 lines: RelayConfigSync variant, ServerRelayInfo struct, full handler arm with state storage, legacy config bridge, and apply_relay_config() call

## Decisions Made

- **Local deserialization struct:** A separate module-private `ServerRelayInfo` struct in `agent_connection.rs` handles serde deserialization of the WS JSON payload, then maps to `state::ServerRelayInfo` for storage and tunnel operations. This keeps serde concerns scoped to the WS module.
- **Non-fatal error handling:** The `apply_relay_config()` call uses `if let Err(e) = ... warn!(...)` pattern — tunnel hot-update failures don't crash the agent, existing tunnels continue running.
- **Legacy RelayConfig bridge:** Only activates when `relay_config().is_none()` (no `AGENT_RELAY_TOKEN` env var set at startup). When the bridge fires, it combines `RelayConfigSync` fields + `GlobalRelayConfig` fields to construct a backward-compatible `RelayConfig` for `run_relay_client()`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Type mismatch: local ServerRelayInfo → state::ServerRelayInfo for apply_relay_config()**
- **Found during:** Task 2 (cargo check)
- **Issue:** The plan's provided handler code passed `servers: Vec<agent_connection::ServerRelayInfo>` directly to `apply_relay_config()`, which expects `Vec<state::ServerRelayInfo>`. Even though the struct fields are identical, Rust treats them as distinct types.
- **Fix:** Added a `.iter().map(...)` conversion to transform each local `ServerRelayInfo` into `state::ServerRelayInfo` before calling `apply_relay_config()`.
- **Files modified:** src/agent_connection.rs
- **Verification:** cargo check passes without type errors
- **Committed in:** 5137215 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required fix — the local deserialization struct and state struct are intentionally separate types; the conversion is the correct bridging pattern.

## Issues Encountered

None.

## Threat Surface Scan

No new threat surface beyond what's modeled in the plan's threat model. The RelayConfigSync handler operates within the established WS message trust boundary, using existing serde `#[serde(tag = "type")]` strict deserialization.

## Known Stubs

No stubs found — all code is fully wired (state storage, legacy bridge, tunnel apply).

## Next Phase Readiness

- RelayConfigSync WS message handling complete
- Agent can now receive and process the complete relay config pushed by backend after RegisterAck
- Ready for verification/testing of the full WS push delivery path

---

*Phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe*
*Completed: 2026-06-09*

## Self-Check: PASSED

- ✅ SUMMARY.md exists at `.planning/phases/70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe/70-03-SUMMARY.md`
- ✅ Commit `2a1e42d` (Task 1: variant + struct)
- ✅ Commit `5137215` (Task 2: handler arm)
- ✅ Commit `cc602f0` (docs: plan metadata)
- ✅ `cargo check -p solys` passes
- ✅ All acceptance criteria verified (5 grep checks + 2 compilation checks)
