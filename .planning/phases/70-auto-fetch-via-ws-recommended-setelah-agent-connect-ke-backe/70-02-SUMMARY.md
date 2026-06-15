---
phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe
plan: 02
subsystem: agent
tags: relay, ws, config-split, diff-based-hot-update, rust, tokio

# Dependency graph
requires:
  - phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe
    provides: Phase context (CONTEXT.md, RESEARCH.md, PATTERNS.md)
  - phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv
    provides: PerServerRuntime, PerServerRelayConfig, relay_client connect/disconnect
  - phase: 68-escluse-relay-infrastructure
    provides: RelayConfig, gateways, relay_client module

provides:
  - GlobalRelayConfig (OnceCell) for immutable env/TOML-based relay config
  - RelaySessionState (RwLock) for dynamic WS-pushed relay config
  - ServerRelayInfo struct for per-server config from backend
  - apply_relay_config() for diff-based hot update of relay tunnels
  - Modified bootstrap_relay_client() that always loads global config

affects:
  - 70-03 (agent_connection.rs — BackendMessage::RelayConfigSync handler will call apply_relay_config())
  - 70-01 (backend push — relay_service.rs will consume state)

tech-stack:
  added: []
  patterns:
    - OnceCell for immutable config, RwLock for mutable state
    - HashSet difference for tunnel diff computation
    - RwLock write guard scoped to block, dropped before async connect/disconnect calls

key-files:
  created: []
  modified:
    - src/state.rs — Added GlobalRelayConfig, RelaySessionState, ServerRelayInfo, getters/setters
    - src/main.rs — Modified bootstrap_relay_client() to always load GlobalRelayConfig
    - src/handlers/relay_client.rs — Added apply_relay_config(), HashSet import

key-decisions:
  - "GlobalRelayConfig uses OnceCell<Arc<T>> (immutable, set once at startup)"
  - "RelaySessionState uses RwLock<Option<T>> with const_new(None) (dynamically replaced)"
  - "Existing RelayConfig kept unchanged for backward compatibility (AGENT_RELAY_TOKEN fallback)"
  - "apply_relay_config() drops write guard before calling connect() to prevent deadlock"
  - "Config change detection compares subdomain, public_port, local_mc_addr (not dns_record_id)"
  - "All GlobalRelayConfig fields cloned before construction to avoid use-after-move in legacy path"

patterns-established:
  - "Full state replace on RelayConfigSync — agent diffs current tunnels vs desired state"
  - "Write lock scoping pattern: acquire → read → drop before async calls"
  - "Config change detection: compare PerServerRelayConfig fields, restart on mismatch"

requirements-completed: []

# Metrics
duration: 7 min
completed: 2026-06-09
---

# Phase 70: Auto-fetch relay config via WS — Plan 02 Summary

**Split RelayConfig into GlobalRelayConfig (OnceCell) + RelaySessionState (RwLock), modified bootstrap to always load global config, and added apply_relay_config() for diff-based hot tunnel updates**

## Performance

- **Duration:** 7 min
- **Started:** 2026-06-09T00:25:53Z
- **Completed:** 2026-06-09T00:32:53Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added `GlobalRelayConfig` (OnceCell) for immutable env/TOML-based relay config (gateway_url, region, DNS creds, public IP)
- Added `RelaySessionState` (RwLock) for dynamic WS-pushed relay config (relay_token + server list)
- Added `ServerRelayInfo` struct for per-server config from backend RelayConfigSync messages
- Added getter/setter functions: `set_global_relay_config()`, `global_relay_config()`, `set_relay_session_state()`, `relay_session_state()`
- Modified `bootstrap_relay_client()` to always load `GlobalRelayConfig` and conditionally set legacy `RelayConfig` only when `AGENT_RELAY_TOKEN` env var is present
- Implemented `apply_relay_config()` in relay_client.rs with HashSet-based diff: cancel removed tunnels, detect config changes (subdomain/public_port/local_mc_addr), start new/restarted tunnels
- All existing code (RelayConfig, relay_config()) preserved unchanged for backward compatibility

## Task Commits

Each task was committed atomically:

1. **Task 1: Add GlobalRelayConfig, RelaySessionState, ServerRelayInfo to state.rs** - `ed92565` (feat)
2. **Task 2: Modify bootstrap_relay_client() to always load GlobalRelayConfig** - `4cf6696` (feat)
3. **Task 3: Add apply_relay_config() for diff-based hot update** - `a1d6923` (feat)

## Files Created/Modified

- `src/state.rs` — Added 3 structs, 2 statics, and 4 getter/setter functions (61 lines added)
- `src/main.rs` — Rewrote bootstrap_relay_client() to always load GlobalRelayConfig, conditional legacy path (43 insertions, 26 deletions)
- `src/handlers/relay_client.rs` — Added apply_relay_config() with HashSet diff logic, config change detection (110 insertions, 1 deletion)

## Decisions Made

- **Config storage split:** `GlobalRelayConfig` uses `OnceCell<Arc<T>>` for immutable config set once at startup. `RelaySessionState` uses `RwLock<Option<T>>` for state dynamically replaced on WS push. This matches established patterns in the codebase (DOCKER_GLOBAL OnceCell, tunnels RwLock).
- **Backward compat:** Existing `RelayConfig` + `RELAY_CONFIG` OnceCell kept unchanged. Old deployments with `AGENT_RELAY_TOKEN` continue working via the conditional legacy path in `bootstrap_relay_client()`.
- **Clone safety:** All `GlobalRelayConfig` fields are cloned before construction to avoid use-after-move errors in the legacy `RelayConfig` path that reuses the same variables.
- **Config change detection:** `apply_relay_config()` compares `subdomain`, `public_port`, `local_mc_addr` between old and new config. `dns_record_id` is intentionally excluded — it's agent-internal state, not part of the backend config sync.
- **Deadlock prevention:** The RwLock write guard is scoped to an inner block that completes before any async `connect()`/`disconnect()` call.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Agent-side config storage split complete (GlobalRelayConfig + RelaySessionState)
- `apply_relay_config()` ready to be called from WS message handler (Plan 70-03)
- `bootstrap_relay_client()` loads global config and conditionally sets legacy config
- Ready for Plan 70-03: Add BackendMessage::RelayConfigSync handler to agent_connection.rs
- The `RELAY_SESSION_STATE` and `apply_relay_config()` function signatures provide clean integration points for 70-03

## Self-Check: PASSED

- ✅ `grep -n "pub struct GlobalRelayConfig" src/state.rs` returns line 197
- ✅ `grep -n "pub struct RelaySessionState" src/state.rs` returns line 208
- ✅ `grep -n "pub struct ServerRelayInfo" src/state.rs` returns line 215
- ✅ `grep -n "pub fn set_global_relay_config" src/state.rs` returns line 226
- ✅ `grep -n "pub fn global_relay_config" src/state.rs` returns line 231
- ✅ `grep -n "pub async fn set_relay_session_state" src/state.rs` returns line 239
- ✅ `grep -n "pub async fn relay_session_state" src/state.rs` returns line 245
- ✅ Existing `RelayConfig` at line 141 and `relay_config()` at line 184 still present
- ✅ `grep -n "set_global_relay_config" src/main.rs` called at line 428
- ✅ `grep -n "AGENT_RELAY_TOKEN.*legacy bootstrap" src/main.rs` at line 436
- ✅ `grep -n "waiting for WS push" src/main.rs` at line 456
- ✅ `grep -n "fn apply_relay_config" src/handlers/relay_client.rs` at line 813
- ✅ `grep -n "HashSet" src/handlers/relay_client.rs` — imported at line 58, used at 817, 825
- ✅ `config_changed` / `to_restart` found at multiple lines (config-change detection per D-04 step 4)
- ✅ `cargo check -p solys` passes (all 3 tasks verified)

---

*Phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe*
*Completed: 2026-06-09*
