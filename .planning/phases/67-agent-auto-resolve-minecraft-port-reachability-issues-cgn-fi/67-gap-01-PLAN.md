---
phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi
plan: 04
type: execute
gap_closure: true
wave: 1
subsystem: api
tags: [connectivity, axum, route-fix, path-extraction, gap-closure]
depends_on:
  - 67-03
files_modified:
  - api/src/presentation/routes/api_routes.rs
  - api/src/presentation/handlers/connectivity_handlers.rs
autonomous: true
requirements:
  - DEPLOY-01
  - DEPLOY-02
  - DEPLOY-03
  - DEPLOY-04
  - DEPLOY-05
  - RCON-01
  - RCON-02
user_setup: []
source_verification: .planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-VERIFICATION.md
source_plan: .planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-03-PLAN.md

must_haves:
  truths:
    - "Backend exposes 3 REST endpoints under `/api/v1/servers/:server_id/connectivity*` with working `Path<Uuid>` extraction for `server_id`"
    - "GET /api/v1/servers/:server_id/connectivity returns the current ConnectivityState from JSONB after the per-tenant ownership check"
    - "POST /api/v1/servers/:server_id/connectivity/probe runs a real probe with the 30s Redis SET NX EX 30 cooldown, per-tenant ownership check intact"
    - "GET /api/v1/servers/:server_id/connectivity/audit returns paginated {items, total, limit, offset} from connectivity_audit_log after the per-tenant ownership check"
  artifacts:
    - path: "api/src/presentation/routes/api_routes.rs"
      provides: "3 explicit .route() calls for /api/v1/servers/:server_id/connectivity, /connectivity/probe, /connectivity/audit — matching the canonical pattern at lines 34-37"
      contains: "servers/:server_id/connectivity"
    - path: "api/src/presentation/handlers/connectivity_handlers.rs"
      provides: "Handler functions (get_status, trigger_probe, get_audit_log) called directly by inline routes; the broken `pub fn router()` is removed; the misleading 'merged into the /api/v1/servers nest' comment is removed"
      min_lines: 100
  key_links:
    - from: "api/src/presentation/routes/api_routes.rs"
      to: "api/src/presentation/handlers/connectivity_handlers.rs"
      via: "inline .route(... get(crate::presentation::handlers::connectivity_handlers::get_status))"
      pattern: "connectivity_handlers::get_status"
    - from: "api/src/presentation/handlers/connectivity_handlers.rs"
      to: "api/src/application/services/connectivity_service.rs"
      via: "state.connectivity_service.probe_server / read_state_for_handler"
      pattern: "state\\.connectivity_service"
    - from: "api/src/presentation/handlers/connectivity_handlers.rs"
      to: "api/src/infrastructure/repositories/sqlx_connectivity_audit_log_repository.rs"
      via: "state.connectivity_audit_log_repository.list_by_server / count_by_server"
      pattern: "state\\.connectivity_audit_log_repository"

gap_addressed: |
  VERIFICATION.md line 49 (truth #13) — "Backend exposes 3 REST endpoints under
  /api/v1/servers/:server_id/connectivity* with per-tenant ownership check and a
  30s Redis-backed cooldown on the manual probe trigger". Reason for failure:
  routes mounted at top level as `/connectivity*` (no `/api/v1/servers/`
  prefix, no `:server_id` segment) — handlers use `Path<Uuid>` for `server_id`
  but no path segment provides one. Fix: replace the broken top-level
  `.merge(connectivity_handlers::router())` with three inline `.route(...)`
  calls using full paths `/api/v1/servers/:server_id/connectivity*`, matching
  the canonical pattern at api_routes.rs:34-37 (backup-config, tasks).
---

<objective>
Restore the 3 Phase 67 connectivity REST endpoints to their documented URLs by replacing the broken top-level `.merge(connectivity_handlers::router())` mount with three explicit inline `.route()` calls at full paths `/api/v1/servers/:server_id/connectivity*`, and remove the now-unused `pub fn router()` from `connectivity_handlers.rs` (and the axum imports it was the only user of).

Purpose: The verifier (VERIFICATION.md lines 49, 99, 129, 159-160, 167-182) reported 1 BLOCKER: the routes resolve to `/connectivity*` (no `/api/v1/servers` prefix, no `:server_id` segment), and the handlers extract `Path<Uuid>` from a path that has no UUID segment. This is a 2-file, ~10-line surgical fix that restores the 67-03-PLAN.md spec (lines 857-866) and the canonical pattern already used at api_routes.rs:34-37 for `backup-config` and `tasks`. The 30s Redis cooldown and per-tenant `ensure_ownership` logic (handler logic, not routing) are already correct and stay untouched.

Output:
- `api/src/presentation/routes/api_routes.rs` — three inline `.route(...)` calls at the correct paths; the broken `.merge(connectivity_handlers::router())` line is removed
- `api/src/presentation/handlers/connectivity_handlers.rs` — `pub fn router()` is deleted; the misleading "merged into the /api/v1/servers nest" comment is removed; the unused `Router`, `get`, `post` imports are pruned from the axum `use` block; the three handler functions stay byte-identical

Scope: This is a routing-mount fix. The handler bodies (`get_status`, `trigger_probe`, `get_audit_log`), the `ensure_ownership` helper, and the 30s Redis cooldown logic are NOT modified — they are correct as-is (per VERIFICATION.md lines 175, 114-116). The 3 acknowledged Phase 67 follow-up WARNINGS (CGN heuristic stub, OUTBOUND_TX placeholder, ConnectivityMonitor no-op) are explicitly out of scope per the gap-closure directive.
</objective>

<execution_context>
@/home/rhnbztnl/.config/opencode/get-shit-done/workflows/execute-plan.md
@/home/rhnbztnl/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md
@.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-VERIFICATION.md
@.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-03-PLAN.md
@.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-CONTEXT.md

# Canonical analogs (read these before writing)
@api/src/presentation/routes/api_routes.rs (lines 33-37 for the per-server nested route pattern; line 40 for the broken merge; line 43 for the parallel Phase 68 .merge which has the same shape but is out of scope)
@api/src/presentation/handlers/connectivity_handlers.rs (lines 25-32 for the broken `pub fn router()`; lines 34-49 for `ensure_ownership`; lines 51-63, 65-102, 110-132 for the unchanged handler bodies)
@.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-03-PLAN.md (lines 857-866 for the original route-mount spec this plan restores)
</context>

<dependency_graph>
## requires
- **67-03 (this phase)**: provides the (broken) `connectivity_handlers.rs` with `pub fn router()` returning `Router<ApiState>` mounted incorrectly at `api_routes.rs:40` as a top-level merge, AND the three handler functions (`get_status`, `trigger_probe`, `get_audit_log`) that already have correct `Path<Uuid>` extraction and ownership/cooldown logic. This plan re-mounts the three handlers at the correct URL.

## provides
- Three REST endpoints mounted at the documented paths with working `Path<Uuid>` extraction:
  - `GET  /api/v1/servers/:server_id/connectivity`
  - `POST /api/v1/servers/:server_id/connectivity/probe`
  - `GET  /api/v1/servers/:server_id/connectivity/audit`
- Removal of the broken `pub fn router() -> Router<ApiState>` from `connectivity_handlers.rs` (zero other call sites — verified by `rg -n "connectivity_handlers" --type rust` returning only the two files this plan modifies).

## consumed_by
- Frontend (Phase 67 UI) — when wired, will call the documented paths.
- Future integration tests — can now hit `/api/v1/servers/:server_id/connectivity*` without the runtime path-extraction 4xx.

## wave
- Wave 1 (no other plans depend on this; standalone fix).
</dependency_graph>

<tech_tracking>
- No new dependencies. No migration changes. No container or bootstrap changes. The fix is purely a route-mount in `api_routes.rs` plus dead-code removal in `connectivity_handlers.rs`.
- No new files. No new services. No new entities. No new tests (handlers unchanged; existing manual smoke is sufficient for a routing fix).
- Touches 2 files: `api/src/presentation/routes/api_routes.rs` (edit, ~7 line change) and `api/src/presentation/handlers/connectivity_handlers.rs` (edit, ~10 line change).
</tech_tracking>

<tasks>

<task type="auto">
  <name>Task 1: Re-mount 3 connectivity routes at correct paths and remove broken router()</name>
  <files>api/src/presentation/routes/api_routes.rs, api/src/presentation/handlers/connectivity_handlers.rs</files>
  <read_first>
    - api/src/presentation/routes/api_routes.rs (lines 33-43: see the canonical per-server route pattern at lines 34-37 and the two broken top-level `.merge(...)` lines at 40 and 43)
    - api/src/presentation/handlers/connectivity_handlers.rs (lines 1-32: see the broken `pub fn router()` block at lines 25-32; lines 34-132: the handler bodies, `ensure_ownership`, and cooldown logic which stay untouched)
    - .planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-03-PLAN.md (lines 857-866: the original spec this plan restores)
    - .planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-VERIFICATION.md (lines 9-22, 49, 99, 129, 159-160, 167-182: the verifier's gap report)
  </read_first>
  <action>
**Part A — `api/src/presentation/routes/api_routes.rs`** (Option B: explicit full paths, matching the canonical pattern at lines 34-37 for `backup-config` and `tasks`):

1. Delete the broken top-level merge and its comment on lines 39-40:
   ```rust
           // Phase 67: Connectivity (per-server nested under /api/v1/servers)
           .merge(crate::presentation::handlers::connectivity_handlers::router())
   ```
   This `.merge()` resolves the routes to `/connectivity*` (no `/api/v1/servers` prefix, no `:server_id` segment), and the merged router's `Path<Uuid>` extractors would fail at runtime even if the URL were correct.

2. Insert three inline `.route(...)` calls in its place, mirroring the canonical pattern at lines 34-37:
   ```rust
           // Phase 67: Connectivity (per-server)
           .route("/api/v1/servers/:server_id/connectivity",
               get(crate::presentation::handlers::connectivity_handlers::get_status))
           .route("/api/v1/servers/:server_id/connectivity/probe",
               post(crate::presentation::handlers::connectivity_handlers::trigger_probe))
           .route("/api/v1/servers/:server_id/connectivity/audit",
               get(crate::presentation::handlers::connectivity_handlers::get_audit_log))
   ```

   The `get` and `post` route methods are already in scope from the existing `use axum::{..., routing::{get, post, put, delete, patch}}` import on lines 3-6, so no new imports are needed.

   Do NOT touch line 43 (`.merge(crate::presentation::handlers::relay_handlers::router())`) — that is Phase 68's parallel issue, blocked on its own 4 BLOCKERs, and explicitly out of scope for this gap-closure.

**Part B — `api/src/presentation/handlers/connectivity_handlers.rs`** (delete the now-unused `pub fn router()` and prune the imports it was the only user of):

1. Delete lines 25-32 (the `pub fn router()` block, including the misleading comment):
   ```rust
   pub fn router() -> Router<ApiState> {
       // Routes are merged into the /api/v1/servers nest in api_routes.rs so the
       // :server_id prefix is already applied.
       Router::new()
           .route("/connectivity", get(get_status))
           .route("/connectivity/probe", post(trigger_probe))
           .route("/connectivity/audit", get(get_audit_log))
   }
   ```
   Verified: zero call sites besides the broken line 40 of `api_routes.rs` (which Part A removes). `rg -n "connectivity_handlers" --type rust` returns only the two files this plan modifies.

2. Prune the now-unused `Router`, `get`, and `post` from the axum `use` block on lines 10-15. The remaining imports keep the handler bodies compiling:
   ```rust
   use axum::{
       extract::{Path, Query, State},
       Json,
   };
   ```
   (Drop the `response::IntoResponse` import too — it was already unused by the existing handlers. Drop `routing::{get, post}` and `Router` from the multi-use line.)

3. Do NOT touch any other line of this file. `ensure_ownership` (lines 34-49), `get_status` (lines 51-63), `trigger_probe` and its 30s Redis SET NX EX 30 cooldown (lines 65-102), and `get_audit_log` (lines 110-132) are all correct and stay byte-identical. The file's `min_lines: 110` requirement from the 67-03 must_haves remains satisfied after deletion (132 - 8 ≈ 124 lines).

**Part C — Verification sanity** (no code change; the verify section below runs the checks):

- `grep -n "servers/:server_id/connectivity" api/src/presentation/routes/api_routes.rs` must return 3 lines (one per inline route).
- `grep -n "pub fn router" api/src/presentation/handlers/connectivity_handlers.rs` must return 0 lines (router deleted).
- `grep -n "servers/:server_id/connectivity" api/src/presentation/handlers/connectivity_handlers.rs` must return 0 lines (the broken `/connectivity*` paths are gone from the handler file; the routes are now defined only in `api_routes.rs`).
- `cd api && cargo check` must exit 0 with no new errors and no new warnings (the axum import cleanup eliminates the "unused import" warnings that would otherwise appear after deleting `router()`).
  </action>
  <verify>
    <automated>grep -n "servers/:server_id/connectivity" api/src/presentation/routes/api_routes.rs && grep -n "pub fn router" api/src/presentation/handlers/connectivity_handlers.rs && grep -n "servers/:server_id/connectivity" api/src/presentation/handlers/connectivity_handlers.rs && cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/api && cargo check 2>&1 | tail -20</automated>
  </verify>
  <done>
- `api_routes.rs` has exactly 3 lines containing `servers/:server_id/connectivity` (one per inline route), and zero top-level `.merge(connectivity_handlers::router())` references.
- `connectivity_handlers.rs` has zero `pub fn router` declarations and zero `servers/:server_id/connectivity` references; the `pub fn router()` block (lines 25-32) is gone; the axum `use` block is pruned to only what's still used (`Path`, `Query`, `State`, `Json`).
- `cd api && cargo check` exits 0 with no new errors and no new warnings in the 2 modified files.
- The 3 handlers (`get_status`, `trigger_probe`, `get_audit_log`) are byte-identical to their pre-fix state; `ensure_ownership` and the 30s Redis cooldown logic are byte-identical.
- The verifier's gap-check `grep "servers/:server_id/connectivity" api/src/presentation/routes/api_routes.rs` (VERIFICATION.md line 129) now returns 3 matches, not 0.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| External client (frontend) → REST handler | All 3 endpoints still go through `ApiState` and the per-tenant `ensure_ownership` check (server.user_id == auth_user.tenant_id) before any service call. Path extraction of `server_id` is now correctly bound to a UUID segment in the URL. |
| REST handler → ConnectivityService | Unchanged from 67-03. Service receives `server_id` (validated UUID) and reads/writes JSONB state. |
| REST handler → Postgres | Unchanged from 67-03. `list_by_server` and `count_by_server` use parameterized `sqlx::query`. |
| REST handler → Redis (cooldown) | Unchanged from 67-03. `trigger_probe` uses `SET ... NX EX 30` for the 30s per-server cooldown. |

## STRIDE Threat Register

The 8 threats from the 67-03-PLAN.md threat model (T-67-11 through T-67-18) are unchanged — this fix touches only the URL shape and the now-dead `pub fn router()`. No new attack surface is introduced; no mitigations are weakened.

| Threat ID | Category | Component | Disposition | Notes |
|-----------|----------|-----------|-------------|-------|
| T-67-11 | I (Information Disclosure) | `GET /api/v1/servers/:server_id/connectivity` | mitigate (unchanged) | Per-tenant `ensure_ownership` check still gates access. URL now correctly carries the UUID segment for the extractor. |
| T-67-12 | D (Denial of Service) | `POST /api/v1/servers/:server_id/connectivity/probe` | mitigate (unchanged) | 30s Redis cooldown still active. URL is now reachable (was 404 before). |
| T-67-G01-01 | T (Tampering) | Route mounting | accept | No new injection vector: `:server_id` is a path parameter parsed as `Uuid` by axum's `Path<Uuid>` extractor. Invalid UUIDs are rejected by the extractor with a 400 before reaching any handler. |

No new trust boundary is introduced. No new STRIDE category is unaddressed.
</threat_model>

<verification>
- `grep -n "servers/:server_id/connectivity" api/src/presentation/routes/api_routes.rs` returns 3 lines (one per inline route)
- `grep -n "pub fn router" api/src/presentation/handlers/connectivity_handlers.rs` returns 0 lines (the broken `pub fn router() -> Router<ApiState>` is deleted)
- `grep -n "servers/:server_id/connectivity" api/src/presentation/handlers/connectivity_handlers.rs` returns 0 lines (the broken `/connectivity*` paths are no longer in the handler file; routes are now inline in `api_routes.rs`)
- `grep -n "merge.*connectivity_handlers::router" api/src/presentation/routes/api_routes.rs` returns 0 lines (the broken top-level merge is gone)
- `grep -n "fn get_status\|fn trigger_probe\|fn get_audit_log\|fn ensure_ownership" api/src/presentation/handlers/connectivity_handlers.rs` returns 4 lines (all four handler/helper bodies are byte-identical to before)
- `cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/api && cargo check 2>&1 | tail -20` reports no new errors and no new warnings in the 2 modified files
- The Phase 68 `.merge(relay_handlers::router())` line at `api_routes.rs:43` is untouched (out of scope; it has its own gap-closure plan to be authored separately)
- A manual `curl` smoke test (informational, not required for the verifier): after `cargo run`, a JWT-authenticated `GET /api/v1/servers/{any-uuid}/connectivity` returns 200 with the per-tenant ownership check (404 if the server doesn't exist or belongs to a different tenant; 200 with `{"success":true, "data":{...}}` if the caller owns the server)
</verification>

<success_criteria>
1. The 3 connectivity REST endpoints are reachable at the documented URLs: `GET /api/v1/servers/:server_id/connectivity`, `POST /api/v1/servers/:server_id/connectivity/probe`, `GET /api/v1/servers/:server_id/connectivity/audit`.
2. The handlers' `Path<Uuid>` extractors correctly parse `server_id` from the URL (no runtime extraction failure, which is the second half of the original BLOCKER).
3. The per-tenant `ensure_ownership` check and the 30s Redis SET NX EX 30 cooldown logic are byte-identical to their pre-fix state (VERIFICATION.md lines 175, 114-116 confirmed them correct).
4. The broken `pub fn router() -> Router<ApiState>` in `connectivity_handlers.rs` and the broken top-level `.merge(connectivity_handlers::router())` in `api_routes.rs` are both removed.
5. `cd api && cargo check` exits 0 with no new errors and no new warnings in the 2 modified files.
6. No other file is touched; no new dependency, migration, service, entity, or test is added.
7. The 3 acknowledged Phase 67 follow-up WARNINGS (CGN heuristic stub at `src/handlers/connectivity.rs:56-57`, OUTBOUND_TX placeholder at `:60-66`, ConnectivityMonitor no-op at `:152-160`) remain out of scope and are not addressed by this plan.
</success_criteria>

<output>
After completion, create `.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-gap-01-SUMMARY.md` with a single-section summary covering: (a) the 2-file diff summary (lines added/removed in `api_routes.rs` and `connectivity_handlers.rs`); (b) the verifier re-check result (the `grep "servers/:server_id/connectivity"` check should now return 3 matches, not 0); (c) `cargo check` exit code and warning count; (d) confirmation that the 3 acknowledged follow-up WARNINGS were intentionally not addressed.
</output>
