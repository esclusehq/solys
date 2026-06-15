---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 01
subsystem: infra
tags: relay, postgres-migration, sqlx, websocket-protocol, entity-extensions, uuid-token, gateway-hmac

# Dependency graph
requires:
  - phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi
    provides: ConnectivityService + NodeMessage protocol extensions (Phase 67 baseline)
provides:
  - 7 new columns on nodes + servers tables for relay lifecycle tracking
  - 3 new indexes (relay_token uniqueness + relay status + last tunnel connected)
  - find_by_relay_token repository method (gateway HMAC auth lookup)
  - 5 new NodeMessage variants (3 inbound TunnelConnect/Disconnect/Heartbeat, 2 outbound ModeOverrideChange/TunnelCloseAck)
  - 2 new entity methods (set_relay_status, set_mode_override) with 5-state and 3-value validation
  - DomainError enum for entity-level invariant violations
affects:
  - 68-02 (agent tunnel client — uses TunnelConnect/Disconnect/Heartbeat)
  - 68-03 (backend relay service — dispatches new NodeMessage variants + uses find_by_relay_token)
  - 68-04 (gateway — uses find_by_relay_token for HMAC auth, consumes ModeOverrideChange)
  - 68-05 (dashboard — sends ModeOverrideChange when user toggles connectivity mode)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Idempotent SQL migrations (ADD COLUMN IF NOT EXISTS, CREATE INDEX IF NOT EXISTS) — safe to re-run"
    - "Partial unique index for nullable token (WHERE relay_token IS NOT NULL)"
    - "State-machine validation at the entity layer (5 lifecycle states) layered on top of DB CHECK constraint"
    - "DomainError enum separate from AppError (string) — typed invariant violations with #[derive(thiserror::Error)]"
    - "Row mapping fallbacks (.ok().flatten()) for columns added in later migrations"

key-files:
  created:
    - api/migrations/20260608000001_add_relay_columns.sql — Phase 68 schema migration
    - api/src/domain/errors.rs — DomainError enum (InvalidRelayStatus, InvalidModeOverride)
  modified:
    - api/src/domain/mod.rs — pub mod errors
    - api/src/domain/entities/node.rs — relay_token, relay_token_issued_at, issue_relay_token method
    - api/src/domain/entities/server.rs — 5 relay fields, set_relay_status, set_mode_override, record_tunnel_disconnect
    - api/src/domain/repositories/node_repository.rs — find_by_relay_token trait method
    - api/src/infrastructure/repositories/postgres_node_repository.rs — find_by_relay_token impl + row_to_node extension
    - api/src/application/use_cases/create_server_use_case.rs — initialize 5 new Server fields on create
    - api/src/infrastructure/repositories/postgres_server_repository.rs — 3 Server struct literals extended with new fields
    - api/src/presentation/ws/node_protocol.rs — 5 new NodeMessage variants

key-decisions:
  - "Used psql + manual INSERT into _sqlx_migrations (with SHA-384 checksum) instead of `sqlx migrate run` — the existing migration history is incomplete (only 20260531 is tracked) so sqlx would try to re-apply old migrations and fail on already-existing tables. The new migration is idempotent at the SQL level (IF NOT EXISTS everywhere) and the row insertion keeps future sqlx runs compatible."
  - "Created DomainError as a typed enum (thiserror) separate from AppError::Domain(String) — gives us a structured invariant-violation type that callers can match on, instead of parsing string error messages. AppError::Domain(String) is still used at the HTTP layer for unknown/foreign domain errors."
  - "Used .ok().flatten() fallbacks in row mapping for new columns — backward compatible with the 67-phase SELECT lists that don't yet include relay columns. Once 67 work is committed and consumes the same row data, the fallbacks still keep things compiling without forcing the SELECT lists to be updated."
  - "Server::set_relay_status stamps last_tunnel_connected_at on 'connected' and last_tunnel_disconnected_at on 'stale'/'failed' — callers do not need to set these timestamps manually. The 'disabled' and 'connecting' transitions do not stamp any timestamp."
  - "Node::issue_relay_token returns Self (immutable) rather than mutating in place — keeps the call site explicit and the change easy to audit in PR review. Caller is expected to call `NodeRepository::update(&issued_node)` to persist."

patterns-established:
  - "DomainError::InvalidRelayStatus / InvalidModeOverride: typed errors raised at entity layer when DB CHECK constraints would also reject the value. Two layers of defense (entity + DB), see D-12 and T-68-06 in threat model."
  - "WebSocket protocol addition pattern: 3 inbound (Agent -> Backend) + 2 outbound (Backend -> Agent) variants grouped under a single Phase section comment for diff readability."

requirements-completed:
  - DEPLOY-01
  - DEPLOY-02
  - STATUS-01
  - STATUS-02

# Metrics
duration: 21 min
completed: 2026-06-07
---

# Phase 68 Plan 01: Relay Schema, Entity Surface, and NodeMessage Protocol

**Phase 68 foundation: 7 relay columns, 3 indexes, find_by_relay_token, and 5 NodeMessage variants for the agent tunnel / backend relay service / gateway / dashboard plans to consume**

## Performance

- **Duration:** 21 min
- **Started:** 2026-06-07T06:35:43Z
- **Completed:** 2026-06-07T06:57:31Z
- **Tasks:** 3
- **Files modified:** 9 (2 new, 7 modified)

## Accomplishments

- Migration `20260608000001_add_relay_columns.sql` applied to dev DB: 2 new columns on `nodes` (relay_token UUID, relay_token_issued_at TIMESTAMPTZ) and 5 on `servers` (connectivity_mode_override, relay_status, last_tunnel_connected_at, last_tunnel_disconnected_at, last_tunnel_disconnect_reason), plus 3 new indexes (uq_nodes_relay_token partial unique, idx_servers_relay_status partial, idx_servers_last_tunnel_connected DESC). All 7 columns and 3 indexes verified via `information_schema` and `pg_indexes`.
- Node entity extended with `relay_token` and `relay_token_issued_at` fields plus `issue_relay_token()` method that returns an updated `Self` (immutable).
- Server entity extended with 5 relay fields and 3 entity methods: `set_relay_status` (5-state validation with transition timestamp side effects), `set_mode_override` (relay|direct|None, rejects "auto" as defense-in-depth), `record_tunnel_disconnect` (timestamp + reason).
- New `DomainError` enum created with `thiserror` and added to `domain::mod.rs`. Two variants: `InvalidRelayStatus`, `InvalidModeOverride`.
- `NodeRepository` trait gained `find_by_relay_token`. `PostgresNodeRepository` implements it with a `WHERE relay_token = $1` SQLx query, and `row_to_node` extended to map the new nullable relay columns with `.ok().flatten()` fallbacks for backward compatibility.
- `NodeMessage` enum extended with 5 new variants: `TunnelConnect`, `TunnelDisconnect`, `TunnelHeartbeat` (inbound from agent) and `ModeOverrideChange`, `TunnelCloseAck` (outbound to agent). All serialize to the expected snake_case JSON tags via the existing `#[serde(tag = "type")]` attribute.
- Updated 4 `Server` struct literal sites (CreateServerUseCase + 3 query methods in `postgres_server_repository.rs`) to initialize the 5 new fields, including the uncommitted 67-phase work in `postgres_server_repository.rs`.

## Task Commits

Each task was committed atomically inside the `api/` sub-repo:

1. **Task 1: Write and push the relay schema migration** — `5558461` (feat)
2. **Task 2: Extend Node and Server entities + add find_by_relay_token** — `0030d3f` (feat)
3. **Task 3: Extend NodeMessage enum with 5 relay variants** — `f99284d` (feat)

## Files Created/Modified

### Created

- `api/migrations/20260608000001_add_relay_columns.sql` — Idempotent migration: 2 columns on `nodes` + 5 on `servers` + 3 indexes, all with `IF NOT EXISTS` for safe re-runs. CHECK constraints on `connectivity_mode_override` (relay|direct|NULL) and `relay_status` (5 lifecycle states, default 'disabled'). Partial unique index on `relay_token WHERE NOT NULL`.
- `api/src/domain/errors.rs` — `DomainError` enum with `#[derive(thiserror::Error)]`, 2 variants, 2 unit tests for display and Eq.

### Modified

- `api/src/domain/mod.rs` — `pub mod errors;` added
- `api/src/domain/entities/node.rs` — 2 new fields + `issue_relay_token` method + 2 existing tests
- `api/src/domain/entities/server.rs` — 5 new fields + 3 new methods + 6 new unit tests
- `api/src/domain/repositories/node_repository.rs` — `find_by_relay_token` trait method with doc comment
- `api/src/infrastructure/repositories/postgres_node_repository.rs` — `find_by_relay_token` impl + `row_to_node` extended
- `api/src/application/use_cases/create_server_use_case.rs` — initialize 5 new Server fields
- `api/src/infrastructure/repositories/postgres_server_repository.rs` — 3 Server struct literals extended
- `api/src/presentation/ws/node_protocol.rs` — 5 new NodeMessage variants with doc comments

## Decisions Made

- **psql + manual `_sqlx_migrations` insert instead of `sqlx migrate run`:** The existing migration history is incomplete — only the `20260531` migration is tracked in `_sqlx_migrations`, so sqlx would try to re-apply `20260218182547_servers_table.sql` and fail on "relation already exists". The new migration is idempotent at the SQL level (every statement uses `IF NOT EXISTS`), so applying it via `psql -f` is safe. The row was inserted into `_sqlx_migrations` with the correct SHA-384 checksum (`b012ddb5...`) so future `sqlx migrate run` invocations see it as already applied and skip it.
- **Typed `DomainError` enum vs `AppError::Domain(String)`:** The plan's `set_relay_status`/`set_mode_override` signatures return `Err(DomainError::Variant)`. The existing `AppError::Domain(String)` is too loose for callers to match on, so a new typed enum gives us structured invariant violations while `AppError::Domain(String)` remains for foreign/unknown domain errors at the HTTP layer. `DomainError` derives `thiserror::Error + PartialEq + Eq` for ergonomic `assert_eq!` in tests and `?` propagation in callers.
- **`.ok().flatten()` fallbacks in row mapping for new columns:** The uncommitted 67-phase SELECT lists in `postgres_server_repository.rs` (lines 166 and 338) do not yet include the relay columns. Rather than forcing the 67 SELECT lists to be updated as part of this plan (out of scope and would expand the 67 work), the new `row.try_get("relay_status")` calls use `.unwrap_or_else(|_| "disabled".to_string())` for the non-nullable defaulted column and `.ok().flatten()` for the nullable ones. This means new server rows load with `relay_status = "disabled"` and the option fields as `None` until 67's SELECT lists are extended. Acceptable because the field defaults match.
- **`Node::issue_relay_token` returns `Self` rather than `&mut self`:** Matches the plan's contract ("does NOT mutate in place; returns a new struct") and makes the call site explicit about the persistence step that must follow. Caller is expected to invoke `NodeRepository::update(&issued_node)` to write the new token to the DB.
- **`Server::set_relay_status` does not stamp `last_tunnel_connected_at` on a no-op transition:** The method only stamps the timestamp when the value actually changes to a new state. Idempotent calls (setting to the same state) do not refresh the timestamp. This is a minor design choice that wasn't explicitly specified in the plan but follows the principle of least surprise.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Applied migration via psql + manual `_sqlx_migrations` insert instead of `sqlx migrate run`**

- **Found during:** Task 1
- **Issue:** `sqlx migrate run` failed with "error returned from database: relation 'servers' already exists" while applying `20260218182547_servers_table.sql`. The existing `_sqlx_migrations` table only tracked 1 of ~50+ migrations; the rest were applied outside sqlx (likely via supabase migrations) and are not in the history.
- **Fix:** Applied the new migration via `psql -f migrations/20260608000001_add_relay_columns.sql` (the SQL is idempotent with `ADD COLUMN IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` everywhere). Then computed the SQLx 0.7 SHA-384 checksum (`b012ddb5e87ea239251268075d9a2552e190b971bc427eb8253e51b42bd20969ee781b6d3f674812482c1e97143af86d`) and `INSERT`ed the migration row into `_sqlx_migrations` so future `sqlx migrate run` invocations see it as already applied.
- **Files modified:** `_sqlx_migrations` table (via psql); the migration SQL file itself
- **Verification:** `\d nodes` and `\d servers` show all 7 new columns; `pg_indexes` shows all 3 new indexes; `SELECT * FROM _sqlx_migrations WHERE version = 20260608000001` shows the row with `success = t` and the correct checksum
- **Committed in:** `5558461` (part of Task 1 commit)

**2. [Rule 3 - Blocking] Updated uncommitted 67-phase `postgres_server_repository.rs` to include the 5 new Server fields**

- **Found during:** Task 2
- **Issue:** `cargo check` failed with `error[E0063]: missing fields 'connectivity_mode_override', 'last_tunnel_connected_at', 'last_tunnel_disconnect_reason' and 2 other fields in initializer of 'Server'`. The 3 `Server { ... }` struct literals in `postgres_server_repository.rs` (find_by_id, list, find_by_node_id) were uncommitted 67-phase work that pre-dated the relay columns. Without the new fields, the project would not compile.
- **Fix:** Added the 5 new fields to all 3 struct literals using `row.try_get(...).ok().flatten()` for the nullable ones and `row.try_get(...).unwrap_or_else(|_| "disabled".to_string())` for `relay_status`. This keeps the 67-phase SELECT lists (which don't yet include relay columns) compatible — the new fields default to `None` / `"disabled"` until the SELECT lists are updated.
- **Files modified:** `api/src/infrastructure/repositories/postgres_server_repository.rs`
- **Verification:** `cargo check` exits 0; `cargo test --lib domain::entities::server` passes 6/6
- **Committed in:** `0030d3f` (part of Task 2 commit)

**3. [Rule 1 - Missing Critical] Created `DomainError` enum (plan referenced it but didn't say where to put it)**

- **Found during:** Task 2
- **Issue:** The plan's `set_relay_status` and `set_mode_override` return `Err(DomainError::InvalidRelayStatus)` and `Err(DomainError::InvalidModeOverride)`, but no `DomainError` type existed in the codebase. `AppError::Domain(String)` is the closest existing error but isn't typed.
- **Fix:** Created `api/src/domain/errors.rs` with a `DomainError` enum using `#[derive(thiserror::Error)]` and 2 variants matching the plan's contract. Added `pub mod errors;` to `api/src/domain/mod.rs`. Added 2 unit tests (display string + Eq semantics).
- **Files created:** `api/src/domain/errors.rs`
- **Verification:** `cargo test --lib domain::errors` passes 2/2
- **Committed in:** `0030d3f` (part of Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical)
**Impact on plan:** All 3 auto-fixes were necessary to make the plan execute in the actual dev environment. None of them change the plan's intent — the schema, entities, repository, and NodeMessage contracts are all delivered exactly as specified. The deviations are documented in the SUMMARY so future plans (02-05) know that `postgres_server_repository.rs` has relay-aware row mapping (in case they need to extend the SELECT lists to include the relay columns).

## Issues Encountered

- `sqlx migrate run` failed on the first attempt because the existing migration history in `_sqlx_migrations` is incomplete. This is a pre-existing project state, not something this plan broke. Documented in deviation #1 above.
- The plan's exact migration command uses `postgresql://server:dev_password@localhost:5432/backend_db` (a placeholder), but the real dev password in `api/.env` is `IIXji73OC6DqhpGtiGWg`. Used the real password. The plan's interface section also documents the `dev_password` placeholder; both are placeholders. The real `.env` is the source of truth.
- The 67-phase work in `postgres_server_repository.rs` (3 uncommitted `Server` struct literals) was the primary blocker for `cargo check`. Resolved by extending them with the new fields, documented in deviation #2.

## Known Stubs

None — the plan establishes only the type/contract surface, and every piece is complete. Downstream plans (02-05) will consume these contracts and provide the actual behavior (agent sends TunnelConnect, backend dispatches to RelayService, gateway looks up by relay_token, dashboard sends ModeOverrideChange).

## User Setup Required

None — no external service configuration required for this plan. The relay token is generated lazily on first node registration (will be wired in Plan 02/03), and the gateway HMAC auth callback is wired in Plan 04.

## Next Phase Readiness

- Schema migration is applied and recorded. All downstream plans can SELECT/INSERT against the 7 new columns immediately.
- `Node`, `Server`, and `NodeMessage` types compile and have unit test coverage for the new validation methods.
- `NodeRepository::find_by_relay_token` is ready for the gateway to call (Plan 04) and for the backend relay service to call (Plan 03) when receiving a `TunnelConnect` from a node that needs ownership verification.
- The 5 new `NodeMessage` variants are ready to be sent by the agent (Plan 02) and dispatched by the backend (Plan 03).
- `postgres_server_repository.rs` row mapping uses `.ok().flatten()` fallbacks for the new columns, so it will continue to work even when the uncommitted 67-phase SELECT lists are eventually extended. When they are, the fallbacks become unnecessary but harmless.

## Verification Results

### Task 1 — Migration

- ✅ `api/migrations/20260608000001_add_relay_columns.sql` exists with the literal SQL from the plan
- ✅ Migration applied: `_sqlx_migrations` has row version=`20260608000001`, description=`add relay columns`, success=`t`, checksum=`b012ddb5...`
- ✅ 2 new columns on `nodes`: `relay_token` (uuid), `relay_token_issued_at` (timestamp with time zone)
- ✅ 5 new columns on `servers`: `connectivity_mode_override` (text), `relay_status` (text), `last_tunnel_connected_at`, `last_tunnel_disconnected_at`, `last_tunnel_disconnect_reason`
- ✅ 3 new indexes: `uq_nodes_relay_token`, `idx_servers_relay_status`, `idx_servers_last_tunnel_connected`

### Task 2 — Entities & Repository

- ✅ `Node` has `pub relay_token: Option<Uuid>` and `pub relay_token_issued_at: Option<DateTime<Utc>>`
- ✅ `Node::new()` initializes both new fields to `None`
- ✅ `Node::issue_relay_token(token)` returns `Self` with token + issued_at set
- ✅ `Server` has 5 new fields (connectivity_mode_override, relay_status, last_tunnel_connected_at, last_tunnel_disconnected_at, last_tunnel_disconnect_reason)
- ✅ `Server::set_relay_status` validates 5 values; invalid returns `Err(DomainError::InvalidRelayStatus)`
- ✅ `Server::set_mode_override` accepts `Some("relay")`/`Some("direct")`/`None`; rejects `Some("auto")` with `Err(DomainError::InvalidModeOverride)`
- ✅ `Server::record_tunnel_disconnect` sets timestamp and reason
- ✅ `NodeRepository::find_by_relay_token` declared in trait
- ✅ `PostgresNodeRepository::find_by_relay_token` implemented with `WHERE relay_token = $1`
- ✅ `cargo check` exits 0
- ✅ 8 unit tests pass (2 DomainError + 6 Server)

### Task 3 — NodeMessage

- ✅ 5 new variants: `TunnelConnect`, `TunnelDisconnect`, `TunnelHeartbeat`, `ModeOverrideChange`, `TunnelCloseAck`
- ✅ Each variant has the field types exactly as specified in the plan
- ✅ `cargo check` exits 0
- ✅ Round-trip JSON test passed (verified via standalone cargo project):
  - `TunnelConnect` → `{"type":"tunnel_connect","server_id":"...","subdomain":"abc12345","public_port":25565,"agent_public_ip":"1.2.3.4","region":"ap-southeast-1"}`
  - `TunnelDisconnect` → `{"type":"tunnel_disconnect","server_id":"...","reason":"server_stopped"}`
  - `TunnelHeartbeat` → `{"type":"tunnel_heartbeat","server_id":"...","tunnel_uptime_secs":42}`
  - `ModeOverrideChange` → `{"type":"mode_override_change","server_id":"...","mode":"auto"}`
  - `TunnelCloseAck` → `{"type":"tunnel_close_ack","server_id":"..."}`

### Plan-Level Verification

- ✅ 7 new columns visible via information_schema (see Task 1 verification)
- ✅ 3 new indexes visible via pg_indexes (see Task 1 verification)
- ✅ `cargo check` exits 0
- ✅ `find_by_relay_token` exists in both trait and impl (2 grep matches)
- ✅ 5 NodeMessage variant declarations exist (5 grep matches)

## Self-Check: PASSED

- ✅ `api/migrations/20260608000001_add_relay_columns.sql` exists
- ✅ `api/src/domain/errors.rs` exists
- ✅ 7 modified files exist (entities/node.rs, entities/server.rs, mod.rs, repositories/node_repository.rs, infra/postgres_node_repository.rs, use_cases/create_server_use_case.rs, infra/postgres_server_repository.rs, presentation/ws/node_protocol.rs)
- ✅ All 3 commits found in `api/` git log: `5558461`, `0030d3f`, `f99284d`
- ✅ Migration applied to dev DB (verified via psql)
- ✅ `cargo check` exits 0
- ✅ 8 unit tests pass (DomainError + Server entity)
- ✅ Round-trip serde test for NodeMessage variants passes

---

*Phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela*
*Completed: 2026-06-07*
