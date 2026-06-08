---
phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv
plan: 05
subsystem: relay-gateway
tags: [auth, types, tunnel, 1n-mapping, multi-server]
requires:
  - phase: 69-02
    provides: Per-server tunnel HashMap architecture in agent relay_client.rs
provides:
  - types.rs with ServerMapping struct
  - auth.rs authorize() returns Vec<ServerMapping> (1:N token→server mapping)
  - backend.rs authorize() sends only relay_token, handles backward-compat response shapes
  - tunnel.rs multi-server comments documenting no agent-IP uniqueness enforcement
affects: []
tech-stack:
  added: []
  patterns:
    - "authorize() returns Vec<ServerMapping> for 1:N token→server authorization"
    - "Backward-compatible response handling: accepts JSON array or single object"
    - "Tunnel authorize verifies requested server_id is in authorized mappings list"
key-files:
  created:
    - opt/relay/src/types.rs: ServerMapping struct with server_id and subdomain
  modified:
    - opt/relay/src/auth.rs: authorize() no longer takes server_id param, returns Vec<ServerMapping>
    - opt/relay/src/backend.rs: BackendClient::authorize() sends only relay_token, handles backward-compat response
    - opt/relay/src/tunnel.rs: Call site updated to iterate Vec<ServerMapping>; multi-server comments added
    - opt/relay/src/main.rs: Added mod types;
key-decisions:
  - "authorize() returns all servers for a relay_token (Vec<ServerMapping>), gateway verifies requested server_id is in the list"
  - "Backend response can be JSON array or single object — gateway handles both for backward compatibility"
  - "No agent-IP uniqueness enforcement exists in tunnel.rs; registry enforces one tunnel per server_id"
  - "Per-IP rate limiting in player.rs is separate DoS protection (connection attempts, not tunnel uniqueness)"
duration: "~3 min"
completed: 2026-06-08
---

# Phase 69 Plan 05: Gateway auth/tunnel adaptation for per-server relay tunnels

**Updated gateway authorize() to return Vec<ServerMapping> for 1:N token→server mapping, added ServerMapping struct to types.rs, and documented multi-server agent-IP handling in tunnel.rs**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-06-08T21:53:10Z
- **Completed:** 2026-06-08T21:55:34Z
- **Tasks:** 2
- **Files modified:** 5 (1 new, 4 modified)

## Accomplishments

- **types.rs:** Created `ServerMapping` struct with `server_id: Uuid` and `subdomain: Option<String>` — the return type for the 1:N `authorize()` call
- **auth.rs:** Removed `server_id` parameter from `authorize()`; returns `Vec<ServerMapping>` containing all servers authorized by the relay token
- **backend.rs:** Removed `server_id` from the authorize POST body; response deserialization handles both JSON array `[...]` and single object `{...}` for backward compatibility with old-style tokens
- **tunnel.rs:** Updated `run_tunnel_session` to call `authorize()` without `server_id`, then verify the requested `server_id` is in the returned `Vec<ServerMapping>`. Added clarifying comments that the gateway allows N concurrent WS from the same agent IP (multi-server).
- **main.rs:** Added `mod types;` module declaration

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ServerMapping struct and update authorize() to return Vec<ServerMapping>** — `cac5b00` (feat)
   - types.rs (new), auth.rs, backend.rs, tunnel.rs, main.rs
2. **Task 2: Relax agent IP uniqueness check in tunnel.rs** — `cdb2f7d` (feat)
   - tunnel.rs (comments documenting no agent-IP uniqueness enforcement)

## Files Created/Modified

- `opt/relay/src/types.rs` — **New.** ServerMapping struct (server_id, subdomain) for 1:N return
- `opt/relay/src/auth.rs` — authorize() no longer takes server_id, returns Vec<ServerMapping>
- `opt/relay/src/backend.rs` — Removed AuthorizeResponse/Authorization structs; sends only relay_token; backward-compat response handling (array or object)
- `opt/relay/src/tunnel.rs` — authorize call updated for Vec<ServerMapping> with server_id verification; multi-server comments
- `opt/relay/src/main.rs` — Added `mod types;`

## Decisions Made

- **authorize() returns all servers for a relay_token:** Gateway now calls backend with just the token, receives a list of authorized server mappings, and verifies the requested server_id is in that list. This is a more correct auth flow for the per-server architecture.
- **Backward-compatible response handling:** The backend may return either a JSON array `[...]` or a single object `{...}`. The gateway handles both via `serde_json::Value` type checking.
- **No agent-IP uniqueness enforcement:** The registry has no `by_agent_ip` map — it keys tunnels by `server_id`. The gateway already supports N WS connections from the same agent IP (one per server). No code changes were needed; clarifying comments were added.
- **Per-IP rate limiting preserved:** The `rate_limiter.check(peer.ip())` guard in `player.rs` is DoS protection for player connection attempts, independent of tunnel uniqueness.

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

None — all threat surface is within the scope of the plan's threat model (T-69-13, T-69-14, T-69-15). No new security-relevant endpoints, auth paths, or file access patterns introduced.

## Known Stubs

No stubs found — all changes are production-ready.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Gateway auth layer now supports 1:N relay_token→server_id mapping for per-server tunnels
- Multiple WS from same agent IP are allowed (registry enforces one tunnel per server_id)
- Ready for Phase 69-06 (relay_client.rs connect/disconnect wiring) or Phase 70 (auto-fetch relay_token via WS after agent auth)

## Self-Check: PASSED

- [x] `types.rs` has `ServerMapping` struct with `server_id: Uuid` and `subdomain: Option<String>`
- [x] `auth.rs` `authorize()` signature has `server_id` parameter removed, returns `Vec<ServerMapping>`
- [x] `authorize()` handles backward-compatible response (single object or array) in backend.rs
- [x] All call sites updated — tunnel.rs calls `authorize()` without `server_id`, verifies mappings
- [x] `tunnel.rs` has multi-server comments documenting no agent-IP uniqueness enforcement
- [x] IP-based rate limiting preserved in player.rs
- [x] `cargo check -p relay-gateway` passes
- [x] Both task commits exist in git log

---

*Phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv*
*Completed: 2026-06-08*
