---
phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv
plan: 02
subsystem: relay
tags: [refactor, relay-tunnel, per-server, multi-tunnel]
dependency-graph:
  requires: [69-01]
  provides: [69-03, 69-04, 69-05]
  affects: [relay.rs, relay_client.rs, relay_session.rs, state.rs, main.rs]
tech-stack:
  added: [tokio::sync::RwLock, std::sync::OnceLock, PerServerRelayConfig, PerServerRuntime, RelayRuntime]
  patterns:
    - "Per-server state via RwLock<HashMap<ServerId, PerServerRuntime>>"
    - "Parent CancellationToken → child token cascade (D-04)"
    - "Per-server heartbeat + bytes counter (D-18)"
    - "Shared config in OnceCell, per-server config via task payload (D-15)"
key-files:
  created: []
  modified:
    - src/state.rs: Added PerServerRelayConfig struct with dns_record_id field
    - src/handlers/relay_client.rs: Full rewrite — per-server tunnel HashMap
    - src/handlers/relay.rs: Updated dispatch with server_id extraction from payload
    - src/handlers/relay_session.rs: Updated module doc for per-server usage
    - src/main.rs: Added relay_client::shutdown_all() in shutdown sequence
decisions:
  - D-01: Use tokio::sync::RwLock<HashMap<ServerId, PerServerRuntime>> for concurrent reads / exclusive writes
  - D-03: PerServerRuntime owns its own CancellationToken (child of parent), JoinHandle, control_tx, bytes_counter, tunnel_start, config
  - D-04: Parent CancellationToken at RelayRuntime level; parent.cancel() cascades to all children via child_token()
  - D-06: relay.connect() atomically replaces existing tunnel — old child token cancelled, new runtime inserted
  - D-08: relay.disconnect() removes from HashMap, cancels child token, dispatches DNS cleanup
  - D-15: Shared relay config stays in OnceCell; per-server config arrives as PerServerRelayConfig in task payload
  - D-18: Each PerServerRuntime has its own bytes_transferred (Arc<AtomicU64>); drive_inbound_streams looks up per stream
metrics:
  duration: "~6 minutes (Task 1: ~1 min, Task 2: ~4 min, Task 3: ~1 min)"
  completed_date: 2026-06-09

# Phase 69 Plan 02: Refactor relay tunnel to per-server architecture

Refactored the agent's relay tunnel from a single global OnceLock<RelayRuntime> to per-server tunnel instances managed in an RwLock<HashMap<ServerId, PerServerRuntime>>. Shared config (gateway_url, token, etc.) remains in OnceCell; per-server config arrives via task payload.

## Task 1: Split RelayConfig in state.rs

- Removed `server_id`, `subdomain`, `public_port`, `local_mc_addr`, `dns_record_id` from `RelayConfig` struct
- Added new `PerServerRelayConfig` struct with those 5 fields (including `dns_record_id: Option<String>` for DNS cleanup)
- Added `Default` derive to `PerServerRelayConfig`
- Updated `bootstrap_relay_client` in `main.rs` to only set shared fields

**Commit:** `bf26454` — `feat(69-02): split RelayConfig — shared fields in OnceCell, per-server fields in PerServerRelayConfig`

## Task 2: Rewrite relay_client.rs — OnceLock<RelayRuntime> → RwLock<HashMap<ServerId, PerServerRuntime>>

Replaced the entire module:

- **Structs:** `PerServerRuntime` (cancel, join, control_tx, bytes_transferred, tunnel_start, config), `RelayRuntime` (shutdown, tunnels: Arc<RwLock<HashMap<Uuid, PerServerRuntime>>>), global `RELAY_RUNTIME: OnceLock<RelayRuntime>`
- **`shutdown_all()`:** Cancels parent token → all child tokens fire → loops exit → HashMap cleared (D-04)
- **`connect(server_id, config)`:** Creates child CancellationToken, spawns per-server reconnect loop. Atomically replaces existing tunnel (D-06)
- **`disconnect(server_id)`:** Cancels child token, removes from HashMap, dispatches DNS cleanup via `dispatch_remove_cname_record(shared_cfg, per_server_cfg)` (D-08)
- **`send_heartbeat(server_id)`:** Looks up per-server control_tx from HashMap, sends TunnelHeartbeat (D-18)
- **`run_relay_client(config, parent_shutdown)`:** Per-server reconnect loop with child token from parent (D-04)
- **`connect_and_run(shared_cfg, per_server_cfg, shutdown)`:** Full tunnel lifecycle — WS handshake, TunnelConnect with per-server fields, heartbeat task spawn, stream drive, cleanup, DNS removal
- **`drive_inbound_streams(session, local_mc_addr, server_id, shutdown)`:** Looks up per-server `bytes_transferred` from HashMap for each new inbound stream (D-18)
- **`dispatch_remove_cname_record(shared_cfg, per_server_cfg)`:** Uses `per_server_cfg.dns_record_id` and `per_server_cfg.subdomain` for DNS cleanup
- **Removed:** `cancel_token()` public accessor (replaced by `shutdown_all()`)
- **Kept:** `build_ws_request()`, `backoff_with_jitter()`, `ws_bridge()`, `shutdown_control()`, `run_heartbeat_task()` — unchanged

**Deviation (Rule 3):** `relay.rs` dispatch was also updated in Task 2 as a blocking issue — the new per-server function signatures required it for compilation.

**Commit:** `de88ec1` — `feat(69-02): rewrite relay_client.rs for per-server tunnel architecture`

## Task 3: Update relay.rs + relay_session.rs + main.rs shutdown wiring

- **`relay.rs`:** `handle_relay_task` now extracts `server_id` (Uuid) and per-server config fields from `task.payload` using direct `serde_json::Value` accessors. Calls `connect(server_id, config)`, `disconnect(server_id)`, `send_heartbeat(server_id)` with per-server args.
- **`relay_session.rs`:** Updated module doc to mention per-server `bytes_transferred` lookup (D-18). No signature changes — already takes `bytes_counter` by parameter.
- **`main.rs`:** Added `relay_client::shutdown_all().await` in the agent shutdown sequence, after the agent loop exits and before final state save.

**Commit:** `cd11961` — `feat(69-02): update relay_session docs, add shutdown_all() call in main.rs`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] relay.rs dispatch needed per-server signatures in Task 2**
- **Found during:** Task 2 build check
- **Issue:** New `connect(server_id, config)`, `disconnect(server_id)`, `send_heartbeat(server_id)` signatures broke compilation of `relay.rs` which called the old zero-arg signatures
- **Fix:** Updated `handle_relay_task` inline with direct `task.payload.get()` calls for `server_id`, `subdomain`, `public_port`, `local_mc_addr`, `dns_record_id`. Inlined the extraction logic rather than using separate helper functions for clarity.
- **Files modified:** `src/handlers/relay.rs`
- **Commit:** `de88ec1`

**2. [Rule 3 - Blocking] PerServerRelayConfig missing dns_record_id field**
- **Found during:** Task 2 build check
- **Issue:** `dispatch_remove_cname_record` in `relay_client.rs` references `per_server_cfg.dns_record_id`, but this field wasn't defined
- **Fix:** Added `dns_record_id: Option<String>` to `PerServerRelayConfig` and `Default` derive
- **Files modified:** `src/state.rs`
- **Commit:** `de88ec1`

**3. [Rule 3 - Blocking] `local` variable moved into async closure before `warn!` usage**
- **Found during:** Task 2 build check
- **Issue:** `drive_inbound_streams` moved `local` into spawned async block, then referenced it in `warn!`
- **Fix:** Cloned `local` before the move; used the clone inside the async block
- **Files modified:** `src/handlers/relay_client.rs`
- **Commit:** `de88ec1`

## Known Stubs

None. All per-server fields are wired to task payload extraction. The `Uuid::nil()` values in heartbeat audit calls are intentional (node_id/server_id are not individually meaningful for heartbeat audit events).

## Threat Flags

None. No new threat surface introduced beyond what's modeled in the plan's threat register.

## Self-Check: PASSED

- [x] PerServerRelayConfig struct added to state.rs with 5 fields
- [x] RelayConfig has only shared fields (per-server fields removed)
- [x] relay_client.rs: RwLock<HashMap<Uuid, PerServerRuntime>> (D-01)
- [x] PerServerRuntime has all 6 fields: cancel, join, control_tx, bytes_transferred, tunnel_start, config
- [x] connect(server_id, config) with add-replace semantics (D-06)
- [x] disconnect(server_id) with HashMap remove + token cancel
- [x] send_heartbeat(server_id) with per-server control_tx lookup
- [x] shutdown_all() cancels parent → clear HashMap (D-04)
- [x] run_relay_client takes per-server config + parent CancellationToken
- [x] relay.rs dispatches to per-server relay_client functions
- [x] relay_session.rs docs updated for per-server usage
- [x] main.rs shutdown calls relay_client::shutdown_all()
- [x] cargo check passes
