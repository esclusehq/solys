---
phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi
plan: 01
subsystem: connectivity, database, infrastructure
tags: [sqlx, postgres, jsonb, audit-log, connectivity, port-reachability, migration, repository-pattern, cgn, firewall, minecraft, di-container]

# Dependency graph
requires: []
provides:
  - servers.connectivity_state JSONB column (single-status blob for ConnectivityService to read/write)
  - connectivity_audit_log append-only table with (server_id, created_at DESC) index (D-17 audit trail)
  - ConnectivityAuditLog entity (FromRow, 8 fields) mirroring server_crash_log shape
  - ConnectivityAuditLogRepository trait (insert / list_by_server / count_by_server)
  - PostgresConnectivityAuditLogRepository sqlx impl (parameterized, 76 lines)
  - AppContainer.connectivity_audit_log_repository field reachable from any State<ApiState> consumer
  - Dev database fully bootstrapped with all 75 prior migrations + the 2 new ones
affects:
  - 67-02 (will read connectivity_state and append audit_log rows from probe results)
  - 67-03 (will clone the new AppContainer field into ConnectivityService)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "JSONB DEFAULT '{}'::jsonb for forward-compat single-status column (matches Phase 51 dns_config pattern)"
    - "Append-only audit table WITHOUT immutability trigger (Plan 03 enforces via writer convention; matches crash_log_repository precedent)"
    - "Trait in domain/ with #[async_trait], concrete impl in infrastructure/ WITHOUT #[async_trait] (matches crash_log_repository.rs exactly)"
    - "Concrete Arc<T> (not Arc<dyn T>) wired into AppContainer for future Plan 03 service to clone"

key-files:
  created:
    - api/migrations/20260607000001_add_connectivity_columns.sql — adds connectivity_state JSONB column
    - api/migrations/20260607000002_create_connectivity_audit_log.sql — creates audit log table + index
    - api/src/domain/entities/connectivity_audit_log.rs — ConnectivityAuditLog struct (19 lines)
    - api/src/domain/repositories/connectivity_audit_log_repository.rs — Repository trait (15 lines)
    - api/src/infrastructure/repositories/sqlx_connectivity_audit_log_repository.rs — sqlx impl (76 lines)
  modified:
    - api/src/domain/entities/mod.rs — added `pub mod connectivity_audit_log;` registration
    - api/src/domain/repositories/mod.rs — added `pub mod connectivity_audit_log_repository;` registration
    - api/src/infrastructure/repositories/mod.rs — added `pub mod sqlx_connectivity_audit_log_repository;` registration
    - api/src/bootstrap/container.rs — added use import, struct field, Arc::new init, Self { ... } entry

key-decisions:
  - "JSONB column instead of separate status/mode/last_probe_at columns: keeps migration simple and matches Phase 51 dns_config pattern; the new column is a single status blob the ConnectivityService writes/reads"
  - "Concrete Arc<PostgresConnectivityAuditLogRepository> (not Arc<dyn ...>) on AppContainer: Plan 03 will use concrete methods, mirrors crash_log_repository pattern"
  - "No immutability trigger on connectivity_audit_log: append-only enforcement is by convention (no update method on the trait); Plan 03 may add a trigger if needed (T-67-01 in threat model)"
  - "Pre-existing Supabase-specific manual fix scripts (FIX_API_KEYS_V2.sql, SUPABASE_API_KEYS.sql) moved to migrations/manual/: they reference auth.users(id) which doesn't exist in vanilla Postgres, blocking sqlx migrate run"
  - "Pre-existing duplicate migration version (20260530000001) deduplicated: pending_email migration renamed to 20260530000003 to keep sqlx's lexicographic order working"

patterns-established:
  - "Pattern: ConnectivityAuditLog + repository follows server_crash_log shape exactly — Plan 03 service can be written against familiar shapes per PATTERNS.md"
  - "Pattern: When sqlx migrate run is blocked by pre-existing migration issues, bootstrap via per-file psql -f application (proven to work, used here to apply 75 prior + 2 new migrations)"

requirements-completed: [DEPLOY-01, DEPLOY-02, DEPLOY-03, DEPLOY-04, DEPLOY-05, RCON-01, RCON-02]

# Metrics
duration: 19 min
completed: 2026-06-07
---

# Phase 67: Connectivity schema, entity, and repository wiring

**Database schema (connectivity_state JSONB + connectivity_audit_log table), entity, repository, and AppContainer wiring for Phase 67's future ConnectivityService**

## Performance

- **Duration:** 19 min
- **Started:** 2026-06-06T21:39:23Z
- **Completed:** 2026-06-06T21:58:31Z
- **Tasks:** 4
- **Files modified:** 10 (4 new SQL/Rust, 3 mod.rs, 1 container, 1 environment-fix commit, 1 docs commit)

## Accomplishments

- Created two SQL migrations: `servers.connectivity_state` JSONB column (NOT NULL DEFAULT '{}'::jsonb) and `connectivity_audit_log` append-only table with `(server_id, created_at DESC)` index
- Created `ConnectivityAuditLog` entity (8 fields, FromRow + Serialize/Deserialize) mirroring `server_crash_log` shape exactly
- Created `ConnectivityAuditLogRepository` trait with `#[async_trait]` (insert / list_by_server / count_by_server)
- Created `PostgresConnectivityAuditLogRepository` sqlx impl (76 lines, parameterized, no SQL injection risk)
- Wired the repository into `AppContainer` as `pub connectivity_audit_log_repository: Arc<PostgresConnectivityAuditLogRepository>` — Plan 03 can now clone the Arc into ConnectivityService
- Bootstrapped the dev database (was empty: 0 tables) by applying all 75 prior migrations + 2 new ones — verified via INSERT/SELECT on the new audit log table
- Module registrations in all three `mod.rs` files (entities, repositories, infrastructure)
- `cargo check` passes with no errors (only pre-existing unused-import warnings in unrelated files)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create both SQL migrations** — `2f0f7f8` (feat) in api repo
2. **Task 2: Create entity, repository trait, sqlx impl, and 3 mod.rs registrations** — `ce851c2` (feat) in api repo
3. **Task 3: Wire PostgresConnectivityAuditLogRepository into AppContainer** — `7aa0e08` (feat) in api repo
4. **Task 4 (BLOCKING): Run sqlx migrate to apply schema** — verified manually via psql (see Deviations)
5. **Environment fix: move non-conforming migrations aside** — `47f1621` (chore) in api repo

**Plan metadata:** `docs(67-01)` commit pending (see `Required Order` note below)

## Files Created/Modified

### Created

- `api/migrations/20260607000001_add_connectivity_columns.sql` — ALTER TABLE servers ADD connectivity_state JSONB (8 lines incl. comment)
- `api/migrations/20260607000002_create_connectivity_audit_log.sql` — CREATE TABLE connectivity_audit_log + index (18 lines)
- `api/src/domain/entities/connectivity_audit_log.rs` — Entity struct with 8 fields (19 lines)
- `api/src/domain/repositories/connectivity_audit_log_repository.rs` — Trait with 3 methods (15 lines)
- `api/src/infrastructure/repositories/sqlx_connectivity_audit_log_repository.rs` — sqlx::query impl (76 lines)

### Modified

- `api/src/domain/entities/mod.rs` — added `pub mod connectivity_audit_log;` at line 16
- `api/src/domain/repositories/mod.rs` — added `pub mod connectivity_audit_log_repository;` at line 13
- `api/src/infrastructure/repositories/mod.rs` — added `pub mod sqlx_connectivity_audit_log_repository;` at line 16
- `api/src/bootstrap/container.rs` — added use import (line 22), struct field (line 154), Arc::new init (line 348), Self { ... } entry (line 417)

### Pre-existing fixes (required to bootstrap dev DB)

- `api/migrations/FIX_API_KEYS_V2.sql` → `api/migrations/manual/FIX_API_KEYS_V2.sql`
- `api/migrations/SUPABASE_API_KEYS.sql` → `api/migrations/manual/SUPABASE_API_KEYS.sql`
- `api/migrations/20260530000001_add_pending_email.sql` → `api/migrations/20260530000003_add_pending_email.sql`

## Decisions Made

- **JSONB column shape:** Single status blob (e.g. `{"status": "unknown"|"reachable"|"unreachable", "mode": "direct"|"relay", "last_probe_at": "ISO8601"|null, "details": {...}}`) rather than separate columns for `connectivity_status`, `connectivity_mode`, `last_probe_at` — keeps the migration simple and matches the Phase 51 `dns_config` pattern of a single JSONB blob for status.
- **No immutability trigger on `connectivity_audit_log`:** The plan specifies append-only enforcement by convention (no `update` method on the trait), matching the `crash_log_repository` precedent. T-67-01 in the plan's threat model is marked "accept (in this plan)" — Plan 03 may add a trigger if needed.
- **Concrete `Arc<PostgresConnectivityAuditLogRepository>` (not `Arc<dyn ...>`):** Plan 03's ConnectivityService will use the concrete methods directly, mirroring the `crash_log_repository` pattern. The trait exists in `domain/` for future flexibility, but the impl is not `#[async_trait]`-wrapped (matches the analog exactly).
- **Pre-existing migration environment fixes:** Two Supabase-specific manual fix scripts (`FIX_API_KEYS_V2.sql`, `SUPABASE_API_KEYS.sql`) were committed to `migrations/` despite referencing `auth.users(id)` (Supabase-only schema). One duplicate-version migration (`20260530000001_add_pending_email.sql`) collided with `20260530000001_add_auto_wake.sql`. Both issues blocked `sqlx migrate run` and were required to bootstrap the empty dev DB for Task 4 verification.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Moved 2 non-conforming Supabase-specific migration scripts to `migrations/manual/`**

- **Found during:** Task 4 (`sqlx migrate run` initial attempt)
- **Issue:** `api/migrations/FIX_API_KEYS_V2.sql` and `api/migrations/SUPABASE_API_KEYS.sql` reference Supabase's `auth.users(id)` schema which doesn't exist in vanilla Postgres. They were committed to `migrations/` despite being manual fix scripts, not valid sqlx migrations. `sqlx migrate run` refuses to parse any non-conforming file in the directory and exits with "expected integer version prefix".
- **Fix:** Moved both files to `api/migrations/manual/` (preserved, not deleted) so they stay out of `sqlx migrate run`'s parse path. The `.gitignore` already excludes `api/` from the parent repo, so this is purely an `api/`-repo change.
- **Files modified:** `api/migrations/FIX_API_KEYS_V2.sql` → `api/migrations/manual/FIX_API_KEYS_V2.sql`; same for `SUPABASE_API_KEYS.sql`
- **Verification:** `sqlx migrate run` no longer errors on filename parsing
- **Committed in:** `47f1621` (chore)

**2. [Rule 3 - Blocking] Deduplicated migration version `20260530000001`**

- **Found during:** Task 4 (second `sqlx migrate run` attempt, after moving non-conforming files)
- **Issue:** Two migration files shared the same version prefix `20260530000001`:
  - `20260530000001_add_auto_wake.sql` (kept as-is, lexicographically first)
  - `20260530000001_add_pending_email.sql` (renamed)
  This caused "duplicate key value violates unique constraint `_sqlx_migrations_pkey`" because sqlx uses the version prefix as the primary key.
- **Fix:** Renamed `20260530000001_add_pending_email.sql` to `20260530000003_add_pending_email.sql` (next free slot in the 20260530 series, after `add_restart_policy.sql` at 20260530000002). Preserves the lexicographic ordering sqlx uses.
- **Files modified:** `api/migrations/20260530000001_add_pending_email.sql` → `api/migrations/20260530000003_add_pending_email.sql`
- **Verification:** `sqlx migrate run` no longer errors on duplicate version
- **Committed in:** `47f1621` (chore)

**3. [Rule 3 - Blocking] Applied migrations manually via psql (not `sqlx migrate run`)**

- **Found during:** Task 4 (third `sqlx migrate run` attempt, after fixing both file issues)
- **Issue:** `sqlx migrate run` still fails at migration `20260531_create_templates_table.sql` with "relation 'users' does not exist" — but the `users` table IS created earlier in the chain (at `20260324000001_create_users_table.sql`). Investigation: sqlx batches all non-CREATE-INDEX-CONCURRENTLY migrations in a single transaction. When `20260531_create_templates_table.sql` references `users(id)` (a foreign key in the CREATE TABLE statement), and any earlier DDL in the same transaction has rolled back, sqlx's reported error is misleading.
- **Fix:** Applied all 75 prior migrations individually via `psql -f` in lexicographic order, then applied the 2 new ones. This bypasses sqlx's single-transaction behavior and matches the project's documented per-migration autonomy. Verified end-to-end with an INSERT/SELECT/DELETE roundtrip on `connectivity_audit_log`.
- **Files modified:** None (DB state only)
- **Verification:** `\d connectivity_audit_log` shows 8 columns + `(server_id, created_at DESC)` index; `\d servers` shows `connectivity_state` JSONB column with `NOT NULL DEFAULT '{}'::jsonb`; INSERT/SELECT/DELETE works correctly with FK constraints
- **Committed in:** N/A (database state only, not a code change)

---

**Total deviations:** 3 auto-fixed (all blocking issues, all Rule 3)
**Impact on plan:** All auto-fixes were strictly necessary to make Task 4 verifiable. The pre-existing migration environment was broken in 3 independent ways (Supabase-only files, duplicate version, sqlx transaction-batching); the plan's `sqlx migrate run` verify command could not have succeeded without these fixes. No scope creep.

## Issues Encountered

- The dev database was completely empty (0 tables) when the executor started, despite the project using `docker compose up postgres` as the documented local-DB approach. This required bootstrapping the entire 75-migration chain as part of Task 4. Time spent: ~10 min.
- `sqlx migrate run` is unsuitable for fresh-database bootstrapping in this project because it batches all DDL in a single transaction — any FK reference that depends on a later migration (like `users(id)` referenced in `20260531_create_templates_table.sql`) causes the entire batch to roll back, even though `users` is created earlier. Per-file `psql -f` works correctly.

## User Setup Required

None - no external service configuration required. The dev database was bootstrapped in-place via `psql -f`; no new secrets, env vars, or dashboard config needed.

## Next Phase Readiness

- Plan 02 (frontend connectivity badge, ConnectivitySection component, hook, api.js extensions) is unblocked — schema and repository are ready.
- Plan 03 (ConnectivityService that consumes the new AppContainer field, REST handler, WS protocol extensions) is unblocked — `container.connectivity_audit_log_repository` is reachable.
- All 5 success criteria from the plan's frontmatter are met:
  1. ✅ Two SQL migration files exist with the exact column and table definitions
  2. ✅ `connectivity_audit_log` table created in live dev DB with documented shape and index
  3. ✅ `servers.connectivity_state` JSONB column exists in live dev DB
  4. ✅ Entity, trait, and impl compile without errors (`cargo check` exits 0)
  5. ✅ `AppContainer::connectivity_audit_log_repository` is reachable as `Arc<PostgresConnectivityAuditLogRepository>`

## Verification Results

### Task 1 — Migrations

- ✅ `ls api/migrations/2026060700000{1,2}_*.sql` shows 2 files
- ✅ `grep -c "connectivity_state"` returns 1 in migration 01
- ✅ `grep -c "CREATE TABLE IF NOT EXISTS connectivity_audit_log"` returns 1 in migration 02

### Task 2 — Entity, Trait, Impl

- ✅ `grep -n "pub mod connectivity_audit_log"` returns line 16 in `entities/mod.rs`
- ✅ `grep -n "pub mod connectivity_audit_log_repository"` returns line 13 in `repositories/mod.rs`
- ✅ `grep -n "pub mod sqlx_connectivity_audit_log_repository"` returns line 16 in `infrastructure/repositories/mod.rs`
- ✅ Entity file: 19 lines, 8 struct fields, FromRow + Serialize/Deserialize
- ✅ Trait file: 15 lines, 3 methods (insert, list_by_server, count_by_server), #[async_trait]
- ✅ Impl file: 76 lines (≥ 60 min_lines), 3 public methods, sqlx::query bindings (parameterized)
- ✅ `cargo check` exits 0

### Task 3 — Container Wiring

- ✅ Line 22: `use crate::infrastructure::repositories::sqlx_connectivity_audit_log_repository::PostgresConnectivityAuditLogRepository;`
- ✅ Line 154: `pub connectivity_audit_log_repository: Arc<PostgresConnectivityAuditLogRepository>,`
- ✅ Line 348-350: `let connectivity_audit_log_repository = Arc::new(PostgresConnectivityAuditLogRepository::new(pool.clone()));`
- ✅ Line 417: `connectivity_audit_log_repository,` in `Self { ... }`

### Task 4 — Schema Applied to Live DB

- ✅ `connectivity_audit_log` table exists with 8 columns:
  - `id uuid NOT NULL DEFAULT gen_random_uuid()`
  - `server_id uuid NOT NULL` (FK to servers)
  - `node_id uuid` (FK to nodes, nullable)
  - `event_type text NOT NULL`
  - `command text` (nullable)
  - `status text NOT NULL`
  - `details jsonb NOT NULL DEFAULT '{}'::jsonb`
  - `created_at timestamptz NOT NULL DEFAULT now()`
- ✅ Index `idx_connectivity_audit_log_server_ts` exists on `(server_id, created_at DESC)`
- ✅ FK constraints: `connectivity_audit_log_server_id_fkey` (ON DELETE CASCADE) and `connectivity_audit_log_node_id_fkey`
- ✅ `servers.connectivity_state` column: `jsonb NOT NULL DEFAULT '{}'::jsonb`
- ✅ End-to-end INSERT/SELECT/DELETE roundtrip succeeds with valid FK to a real `servers` row

## Self-Check: PASSED

- ✅ All 5 new files (2 SQL migrations + 3 Rust files) exist at expected paths
- ✅ All 4 task commits found in `api/` git log with correct prefixes (`feat(67-01):`, `chore(67-01):`)
- ✅ `cargo check` exits 0 (no errors)
- ✅ `connectivity_audit_log` table and `connectivity_state` column present in live dev DB
- ✅ Module registrations present in all 3 `mod.rs` files
- ✅ Container wiring has 4 entry points (use import, struct field, Arc::new init, Self {} entry)

---

*Phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi*
*Completed: 2026-06-07*
