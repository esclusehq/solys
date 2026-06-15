---
phase: 73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik
plan: 01
subsystem: api
tags: bedrock, udp, port-allocation, relay, loader, protocol
requires: []
provides:
  - UDP port pool seed row (19132-19231) in port_pools table
  - Protocol-aware port allocation (tcp/udp dispatch)
  - loader field pipeline from server entity through RelayConfigSync to agent
affects:
  - 73-02 (Agent UDP relay client)
  - 73-03 (Gateway UDP relay support)
  - 73-04 (UDP relay integration testing)
tech-stack:
  added: []
  patterns:
    - Protocol-aware SQL filtering with AND protocol = $N
    - #[serde(default)] for backward-compatible WS message fields
    - mc_loader "bedrock" → loader: Some("bedrock") pipeline
key-files:
  created:
    - api/migrations/20260613000001_seed_udp_port_pool.sql
  modified:
    - api/src/application/use_cases/port_allocation_use_case.rs
    - api/src/presentation/ws/node_protocol.rs
    - api/src/application/services/relay_service.rs
    - agent/solys/src/agent_connection.rs
    - agent/solys/src/state.rs
key-decisions:
  - "UDP port pool uses global pool (node_id = NULL) matching TCP pool pattern"
  - "Protocol parameter added to both allocate_port and release_port; existing callers pass 'tcp'"
  - "loader field added with #[serde(default)] for backward compat with existing agents"
  - "ServerRelayInfo and RelayConnect both get loader field for complete context"
requirements-completed: []
duration: 6 min
completed: 2026-06-12
---

# Phase 73: Approach 1 Per-Server UDP Port — Plan 01 Summary

**UDP port pool seed migration, protocol-aware port allocation, and loader field pipeline from backend to agent**

## Performance

- **Duration:** 6 min
- **Started:** 2026-06-12T18:12:24Z
- **Completed:** 2026-06-12T18:18:24Z
- **Tasks:** 3
- **Files modified:** 6 (2 created, 4 modified)

## Accomplishments

- Created UDP port pool seed migration for Bedrock (19132-19231, 100 ports, protocol='udp') with idempotent INSERT
- Made `allocate_port` and `release_port` protocol-aware — accept `protocol: &str` and filter SQL queries with `AND protocol = $N`
- Added `loader` field to ServerRelayInfo (backend + agent) with `#[serde(default)]` for backward compatibility
- Added `loader` field to RelayConnect message for the push_all_servers path
- Added `loader` field to RelayServerConfig for agent-side config storage
- Threaded `loader` through `push_relay_config` → `RelayConfigSync` → agent's `RelayServerConfig` pipeline
- Both `api` and `agent/solys` crates pass `cargo check`

## Task Commits

Each task was committed atomically:

1. **Task 1: Create seed migration for UDP port pool** — `api@491a79e` (feat)
2. **Task 2: Protocol-aware port allocation** — `api@685e6cd` (feat)
3. **Task 3: Thread loader field through pipeline** — `api@ce9f724` (feat, backend) + `agent/solys@9f4dd38` (feat, agent-side)

## Files Created/Modified

### Created
- `api/migrations/20260613000001_seed_udp_port_pool.sql` — UDP port pool seed row (19132-19231, 'udp')

### Modified
- `api/src/application/use_cases/port_allocation_use_case.rs` — allocate_port/release_port accept protocol param
- `api/src/presentation/ws/node_protocol.rs` — ServerRelayInfo + RelayConnect with loader field
- `api/src/application/services/relay_service.rs` — push_relay_config + push_all_servers pass loader
- `agent/solys/src/agent_connection.rs` — agent-side ServerRelayInfo with loader, RelayConfigSync mapping
- `agent/solys/src/state.rs` — RelayServerConfig with loader field

## Decisions Made

- **UDP pool pattern:** Uses global pool (node_id = NULL) with `protocol='udp'`, matching the existing TCP pool pattern exactly. This keeps the allocation logic uniform.
- **Protocol filter on both allocate and release:** Both functions filter by protocol on SELECT (node pool and global pool queries) to ensure they operate on the correct pool.
- **Backward-compatible deserialization:** `#[serde(default)]` on agent-side `loader` fields ensures existing agents with older config (no loader field) continue working — missing field defaults to `None` (TCP behavior per T-73-01-01).
- **RelayConnect also gets loader:** The per-server push path (push_all_servers) now also passes loader in the RelayConnect message for complete context.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- No database available in this environment for `sqlx migrate run` verification. The migration SQL is correct (verified against existing seed pattern) and will be applied in deployment environments.

## Threat Flags

None — all changes are within the scope of the plan's threat model. ServerRelayInfo.loader was explicitly modeled (T-73-01-01, mitigated by `#[serde(default)]`).

## User Setup Required

None — no external service configuration required. The migration will run automatically via `sqlx migrate run` during deployment.

## Next Phase Readiness

- Backend can allocate UDP ports from the Bedrock pool (19132-19231)
- Backend includes `loader` field in `RelayConfigSync` for Bedrock servers
- Agent deserializes `loader` from `RelayConfigSync` and stores it in `RelayServerConfig`
- Ready for Plan 73-02 (Agent UDP relay client using the loader field)
- Ready for Plan 73-03 (Gateway UDP relay support)
- Ready for Plan 73-04 (UDP relay integration testing)

## Verification Results

### Task 1 — Seed Migration
- ✅ `api/migrations/20260613000001_seed_udp_port_pool.sql` exists with INSERT INTO port_pools
- ✅ File contains `protocol = 'udp'` and `19132, 19231, 19132`
- ⚠️ `sqlx migrate run`: No DB connection in this environment (SQL syntax verified against schema)

### Task 2 — Protocol-aware Port Allocation
- ✅ `allocate_port` signature: `pub async fn allocate_port(pool: &PgPool, node_id: Option<Uuid>, protocol: &str)`
- ✅ `release_port` signature: `pub async fn release_port(pool: &PgPool, port: i32, node_id: Option<Uuid>, protocol: &str)`
- ✅ Both global pool queries contain `AND protocol = $1`
- ✅ Both node pool queries contain `AND protocol = $2`
- ✅ `cargo check` passes for api crate (warnings only)

### Task 3 — Loader Field Pipeline
- ✅ `node_protocol.rs` ServerRelayInfo has `pub loader: Option<String>` with `#[serde(default)]`
- ✅ `node_protocol.rs` RelayConnect has `loader: Option<String>` with `#[serde(default)]`
- ✅ `relay_service.rs` push_relay_config passes `loader: Some("bedrock")` when mc_loader is "bedrock"
- ✅ `relay_service.rs` push_all_servers passes `loader` on RelayConnect
- ✅ `agent_connection.rs` ServerRelayInfo has `loader: Option<String>` with `#[serde(default)]`
- ✅ `agent_connection.rs` RelayConfigSync maps `loader: s.loader.clone()`
- ✅ `state.rs` RelayServerConfig has `pub loader: Option<String>`
- ✅ Both `api` and `agent/solys` crates pass `cargo check`

## Self-Check: PASSED

- ✅ `api/migrations/20260613000001_seed_udp_port_pool.sql` exists with INSERT, 'udp', 19132-19231
- ✅ `allocate_port` signature has `protocol: &str`
- ✅ `release_port` signature has `protocol: &str`
- ✅ `api@491a79e` (Task 1) — seed migration commit found
- ✅ `api@685e6cd` (Task 2) — protocol-aware port allocation commit found
- ✅ `api@ce9f724` (Task 3) — backend loader pipeline commit found
- ✅ `agent/solys@9f4dd38` (Task 3) — agent-side loader pipeline commit found
- ✅ `parent@7715457` (Summary) — summary commit found
- ✅ `node_protocol.rs` ServerRelayInfo has `pub loader: Option<String>` with `#[serde(default)]`
- ✅ `relay_service.rs` push_relay_config maps bedrock loader correctly
- ✅ `agent_connection.rs` ServerRelayInfo has `loader` with `#[serde(default)]`
- ✅ `agent_connection.rs` RelayConfigSync maps `loader: s.loader.clone()`
- ✅ `state.rs` RelayServerConfig has `pub loader: Option<String>`
- ✅ Both `api` and `agent/solys` crates pass `cargo check`

---

*Phase: 73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik*
*Completed: 2026-06-12*
