---
phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe
plan: 01
subsystem: api
tags: relay, websocket, node-protocol, relay-config-sync

requires:
  - phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv
    provides: RelayConnect/RelayDisconnect protocol variants, push_all_servers(), ServerRepository subdomain infrastructure

provides:
  - NodeMessage::RelayConfigSync variant on NodeMessage enum (serde rename = relay_config_sync)
  - ServerRelayInfo struct (server_id, subdomain, local_mc_addr, public_port)
  - RelayService.push_relay_config() method — single-message relay config push
  - Wired push_relay_config() call in Register handler, replacing push_all_servers()

affects:
  - 70-02 (agent-side RelayConfigSync consumer)

tech-stack:
  added: []
  patterns:
    - Single RelayConfigSync message replaces N individual RelayConnect messages for relay config delivery
    - Full-qualified paths used inside method body to avoid import churn (ServerRelayInfo used via crate:: path)

key-files:
  created: []
  modified:
    - api/src/presentation/ws/node_protocol.rs
    - api/src/application/services/relay_service.rs
    - api/src/presentation/handlers/node_ws_handler.rs

key-decisions:
  - "Used full paths (crate::presentation::ws::node_protocol::ServerRelayInfo) in push_relay_config() to avoid import changes in relay_service.rs"
  - "Kept push_all_servers() for backward compat during transition (not removed)"
  - "push_relay_config() reads gateway_url and region from env vars with sensible defaults"

requirements-completed: []

duration: 2 min
completed: 2026-06-09
---

# Phase 70: Auto-fetch relay config via WS — Plan 01 Summary

**RelayConfigSync protocol message with ServerRelayInfo struct, push_relay_config() method on RelayService, and wired Register handler replacing per-server RelayConnect push with single-message relay config delivery**

## Performance

- **Duration:** 2 min
- **Started:** 2026-06-09T00:32:32+07:00
- **Completed:** 2026-06-09T00:34:36+07:00
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added `NodeMessage::RelayConfigSync` variant with `relay_token`, `gateway_url`, `region`, and `servers: Vec<ServerRelayInfo>` fields — single message replaces N individual `RelayConnect` messages
- Added `ServerRelayInfo` struct with `server_id` (Uuid), `subdomain` (String), `local_mc_addr` (String), and `public_port` (u16)
- Added `RelayService::push_relay_config()` method — reads node via `node_repository`, reads all servers via `server_repository`, constructs `RelayConfigSync` with relay_token (from node), gateway_url and region (from env vars), and all server relay info
- Wired `push_relay_config()` call in `node_ws_handler.rs` Register handler, replacing the Phase 69 `push_all_servers()` call
- `push_all_servers()` retained for backward compat during transition

## Task Commits

Each task was committed atomically to the `api` sub-repo:

1. **Task 1: Add RelayConfigSync variant to NodeMessage + ServerRelayInfo struct** — `api@f312a3e` (feat)
2. **Task 2: Add push_relay_config() method to RelayService** — `api@e802424` (feat)
3. **Task 3: Wire push_relay_config() in node_ws_handler.rs Register handler** — `api@458dd33` (feat)

## Files Created/Modified

- `api/src/presentation/ws/node_protocol.rs` — Added `RelayConfigSync` variant on `NodeMessage` enum (line 267) and `ServerRelayInfo` pub struct (line 276)
- `api/src/application/services/relay_service.rs` — Added `push_relay_config()` method at line 327, after `push_all_servers()`
- `api/src/presentation/handlers/node_ws_handler.rs` — Replaced `push_all_servers()` call with `push_relay_config()` call (lines 306-314)

## Decisions Made

- **Full paths for ServerRelayInfo:** Used `crate::presentation::ws::node_protocol::ServerRelayInfo` in method body rather than adding to imports — cleaner evolution pattern, avoids import line churn.
- **push_all_servers() retained:** Kept for backward compat during transition. Will become dead code after this plan but remains in case callers not yet switched.
- **Default gateway/region values:** `RELAY_GATEWAY_URL` defaults to `wss://relay.esluce.com/relay/tunnel`, `RELAY_REGION` defaults to `ap-southeast-1`. Backend owns authoritative values.

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

### Task 1 — RelayConfigSync variant + ServerRelayInfo struct

- ✅ `grep -n "RelayConfigSync"` returns line 267 with the variant
- ✅ `grep -n "pub struct ServerRelayInfo"` returns line 276 with the struct
- ✅ `grep -n "relay_config_sync"` returns line 266 with serde rename
- ✅ `cargo check` passes

### Task 2 — push_relay_config() method

- ✅ `grep -n "fn push_relay_config"` returns line 327 with method definition
- ✅ Method reads `node.relay_token` and calls `self.server_repository.find_by_node_id(node_id)`
- ✅ Method constructs `NodeMessage::RelayConfigSync` and calls `self.node_connection_manager.send_to_node()`
- ✅ `cargo check` passes
- ✅ `push_all_servers()` still exists at line 290 (not removed)

### Task 3 — Wire push_relay_config() in handler

- ✅ `grep -n "push_all_servers"` in handler only appears in comment (not in code)
- ✅ `grep -n "push_relay_config"` in handler returns wired call at line 309
- ✅ `grep -n "RelayConfigSync"` in handler returns log message at line 313
- ✅ `cargo check` passes

## Issues Encountered

None.

## Known Stubs

No stubs found — all code is production-ready and connected.

## Threat Surface Scan

No new threat surface beyond plan's threat model (T-70-01 through T-70-03 already addressed: serde strict deserialization, fresh config per WS connect, relay_token disclosure accepted per existing pattern).

## Next Phase Readiness

- Backend protocol complete (RelayConfigSync variant + push implementation)
- Ready for Phase 70-02 (agent-side consumer of RelayConfigSync)

---

*Phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe*
*Plan: 01*
*Completed: 2026-06-09*
