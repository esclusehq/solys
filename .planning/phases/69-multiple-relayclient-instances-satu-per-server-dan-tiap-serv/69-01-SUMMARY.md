---
phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv
plan: 01
subsystem: api
tags: [subdomain, relay, sha256, websocket, postgres]
requires: []
provides:
  - servers.subdomain column with UNIQUE index
  - Server entity subdomain field
  - Subdomain generation (SHA-256 first 5 hex chars)
  - RelayConnect/RelayDisconnect NodeMessage variants
  - push_all_servers on agent reconnect
  - relay.disconnect push on server delete
affects: [69-02, 69-03]

tech-stack:
  added: []
  patterns:
    - Backend pushes relay commands via NodeMessage variant (not generic Task)
    - Subdomain backfill on agent reconnect for pre-Phase-69 servers
    - push_all_servers called after RegisterAck alongside DNS replay

key-files:
  created:
    - api/migrations/20260609000001_add_servers_subdomain.sql
  modified:
    - api/src/domain/entities/server.rs
    - api/src/domain/repositories/server_repository.rs
    - api/src/infrastructure/repositories/postgres_server_repository.rs
    - api/src/application/use_cases/create_server_use_case.rs
    - api/src/application/services/relay_service.rs
    - api/src/presentation/ws/node_protocol.rs
    - api/src/presentation/handlers/node_ws_handler.rs
    - api/src/presentation/handlers/server_handlers.rs

key-decisions:
  - "Represent relay commands as dedicated NodeMessage variants (RelayConnect/RelayDisconnect) instead of generic Task dispatch (D-06)"
  - "Subdomain = first 5 hex chars of SHA-256(server UUID) — ~1M values, 0.5% collision at 1K servers (D-09/D-10)"
  - "RelayConnect payload includes server_id, subdomain, public_port, local_mc_addr for the agent to open tunnel (D-13)"
  - "Backfill subdomain on agent reconnect for servers created before Phase 69 (subdomain = NULL)"
  - "push_all_servers called after RegisterAck + DNS replay, failures are non-fatal (logged only)"

patterns-established: []
requirements-completed: []

duration: 43min
completed: 2026-06-08
---

# Phase 69 Plan 01: Per-server subdomain + relay push on reconnect/delete

**servers.subdomain column with SHA-256 hex hash generation, RelayConnect/RelayDisconnect message variants, push_all_servers on agent WS reconnect, and relay.disconnect on server delete**

## Performance

- **Duration:** 43 min
- **Started:** 2026-06-08T20:54:30Z
- **Completed:** 2026-06-08T21:37:--Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Created migration adding `servers.subdomain TEXT` column with UNIQUE index (WHERE subdomain IS NOT NULL)
- Added `subdomain: Option<String>` field to Server entity with `make_server()` test helper
- Updated all PostgresServerRepository queries (SELECT/INSERT/UPDATE) to include subdomain
- Added `generate_subdomain()` and `set_subdomain()` methods to ServerRepository trait + Postgres impl — uses SHA-256(first 5 hex chars) for deterministic short hash
- Wired subdomain generation in CreateServerUseCase (generated on every server create)
- Added `RelayConnect` and `RelayDisconnect` variants to NodeMessage enum (dedicated variants per D-06)
- Added `push_all_servers()` to RelayService — iterates servers on a node, backfills subdomain for pre-Phase-69 servers, sends RelayConnect for each
- Called `push_all_servers` in node_ws_handler after RegisterAck + DNS replay
- Pushed `RelayDisconnect` in server_handlers delete handler after successful DB delete
- Verified compilation and 42/44 unit tests pass (2 pre-existing failures in node_health.rs)

## Task Commits

Each task was committed atomically to the `api/` sub-repo:

1. **Task 1: Migration + Server entity subdomain field + all repository query updates** — `8c71d09` (feat)
2. **Task 2: Repository trait methods + subdomain generation + wiring in CreateServerUseCase** — `fcd6bc0` (feat)
3. **Task 3: NodeMessage RelayConnect/RelayDisconnect + push_all_servers + reconnect/delete wiring** — `0b8b703` (feat)

## Files Created/Modified

### Created
- `api/migrations/20260609000001_add_servers_subdomain.sql` — ALTER TABLE servers ADD COLUMN subdomain TEXT with UNIQUE index

### Modified
- `api/src/domain/entities/server.rs` — Added `subdomain: Option<String>` field + test helper
- `api/src/domain/repositories/server_repository.rs` — Added `generate_subdomain()` and `set_subdomain()` trait methods
- `api/src/infrastructure/repositories/postgres_server_repository.rs` — Added subdomain to all SELECT/INSERT/UPDATE queries + method implementations
- `api/src/application/use_cases/create_server_use_case.rs` — Wired `generate_subdomain()` on server create
- `api/src/application/services/relay_service.rs` — Added `push_all_servers()` with backfill logic
- `api/src/presentation/ws/node_protocol.rs` — Added `RelayConnect` and `RelayDisconnect` variants
- `api/src/presentation/handlers/node_ws_handler.rs` — Called `push_all_servers` after RegisterAck + DNS replay
- `api/src/presentation/handlers/server_handlers.rs` — Pushed `RelayDisconnect` after server deletion

## Decisions Made

- **D-06: Dedicated RelayConnect/RelayDisconnect variants** over generic `Task` — clear message intent, compiler-level exhaustiveness checking in agent match blocks
- **D-09: Subdomain = first 5 hex chars of SHA-256(server UUID)** — deterministic, no DB roundtrip for generation, no external dependency
- **D-10: Collision-aware design** — 5 hex chars = 1,048,576 values; ~0.5% collision rate at 1000 servers; extend to 6+ chars if collisions arise
- **D-13: RelayConnect payload** — server_id, subdomain, public_port, local_mc_addr all provided for the agent to open a tunnel without additional lookups

## Deviations from Plan

None — plan executed exactly as written. The plan's Task 3 action pseudo-code showed `NodeMessage::Task(task)` but D-06 (established during planning) directs dedicated variants; implemented per D-06.

## Issues Encountered

- `api/` is a separate git sub-repo (ignored by root `.gitignore`). All commits landed in the `api/` repo's `master` branch.
- 2 pre-existing test failures in `node_health.rs` — unrelated to Phase 69 changes, present before this plan.

## Stub Tracking

None — all wire-up is complete. Subdomain is generated on every server create via `CreateServerUseCase`, backfilled on agent reconnect via `push_all_servers`, and pushed via `RelayDisconnect` on delete.

## Next Phase Readiness

- **69-02:** Ready — entity, repository, migration in place. The agent codegen + crate work can proceed.
- **69-03:** Ready — `RelayConnect`/`RelayDisconnect` variants defined on backend side; agent handler dispatch can be added.
- **69-04/69-05:** Ready — subdomain column exists, subdomain generation wired, subject to no blocker.

## Self-Check

```
=== FILE CHECKS ===
FOUND: api/migrations/20260609000001_add_servers_subdomain.sql
FOUND: api/src/domain/entities/server.rs
FOUND: api/src/domain/repositories/server_repository.rs
FOUND: api/src/infrastructure/repositories/postgres_server_repository.rs
FOUND: api/src/application/use_cases/create_server_use_case.rs
FOUND: api/src/application/services/relay_service.rs
FOUND: api/src/presentation/ws/node_protocol.rs
FOUND: api/src/presentation/handlers/node_ws_handler.rs
FOUND: api/src/presentation/handlers/server_handlers.rs

=== COMMIT CHECKS ===
FOUND: 8c71d09
FOUND: fcd6bc0
FOUND: 0b8b703
```

**PASSED** — All 9 files created/modified, all 3 commits present, all tasks completed.

---

*Phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv*
*Completed: 2026-06-08*
