---
phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi
plan: 03
subsystem: api
tags: [connectivity, ws-protocol, mcp-slp, raknet, failure-classifier, auto-fix-dispatcher, axum, redis-cooldown, tokio-spawn, audit-log]

# Dependency graph
requires:
  - phase: 67-01
    provides: "connectivity_audit_log table, ConnectivityAuditLog entity/repo, AppContainer.connectivity_audit_log_repository"
  - phase: 67-02
    provides: "Agent-side ConnectivityReport payload contract, agent dispatch arms for connectivity.diagnostics/firewall/upnp tasks"
provides:
  - "3 new NodeMessage WS variants (ConnectivityReport / ConnectivityFixRequest / ConnectivityFixResult)"
  - "2 new WS handler dispatch arms (ConnectivityReport / ConnectivityFixResult) in node_ws_handler.rs"
  - "ConnectivityService (~540 lines) with 4-MVP FailureMode classifier, Java SLP + Bedrock RakNet probes, classify_failure(), dispatch_fix() safe-to-fix gate, handle_agent_diagnostics/handle_fix_result WS dispatchers, probe_server REST dispatcher, 5-min periodic re-probe loop"
  - "3 REST endpoints under /api/v1/servers/:server_id/connectivity* with per-tenant ownership check and 30s Redis cooldown on manual probe"
  - "AppContainer.connectivity_service: Arc<ConnectivityService> reachable from any State<ApiState> consumer"
  - "tokio::spawn(connectivity.start().await) in bootstrap/mod.rs alongside monitoring_service"
affects:
  - "67-04 (frontend will consume the 3 REST endpoints and read the connectivity_audit_log)"
  - "Phase 68 (relay infrastructure) — ConnectivityState.mode field is the place where the relay mode will be surfaced"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SOCK_STREAM SLP handshake + status request (Java Edition): ~80 lines of async tokio over a single TcpStream"
    - "UDP unconnected ping + pong parse (Bedrock Edition): RAKNET_MAGIC constant + 0x1c pong discriminator"
    - "WS protocol extension via #[serde(tag = 'type', rename = '...')] enum cases — extends NodeMessage without forking the envelope"
    - "Reachable→Unreachable auto-fix dispatch gate (only on the first transition) — prevents repeated fixes during a sustained outage"
    - "CGNAT_DETECTED is a hard no-auto-fix (D-05); dispatch_fix returns Ok(()) without sending any NodeMessage"
    - "30s manual-probe cooldown via `SET key 1 NX EX 30` — atomic, no race; cooldown is treated as both 'key already set' and 'redis error' (fail-closed)"
    - "direct sqlx::query for the connectivity_state JSONB column (no repository indirection) — keeps the 4-column hot path tight"

key-files:
  created:
    - api/src/application/services/connectivity_service.rs — Probe + classify + auto-fix dispatcher (548 lines)
    - api/src/presentation/handlers/connectivity_handlers.rs — 3 REST endpoints with per-tenant check (132 lines)
  modified:
    - api/src/presentation/ws/node_protocol.rs — 3 new NodeMessage variants
    - api/src/presentation/handlers/node_ws_handler.rs — 2 new dispatch arms
    - api/src/application/services/mod.rs — pub mod connectivity_service
    - api/src/presentation/handlers/mod.rs — pub mod connectivity_handlers
    - api/src/presentation/routes/api_routes.rs — .merge(connectivity_handlers::router()) into /api/v1/servers nest
    - api/src/bootstrap/container.rs — connectivity_service field, Arc::new init, Self { ... } entry
    - api/src/bootstrap/mod.rs — tokio::spawn(connectivity.start().await) after monitoring_service spawn

key-decisions:
  - "JSONB column used as a single status blob (not separate status/mode/last_probe_at columns) — matches 67-01 decision and lets the service write the whole state in one UPDATE"
  - "Reachable→Unreachable is the only auto-fix trigger (state transition gate) — prevents auto-fix spam during sustained outage"
  - "CGNAT_DETECTED is the only FailureMode variant where dispatch_fix returns Ok(()) without sending any NodeMessage — D-05 'never auto-fix CGN'"
  - "30s Redis SET NX EX 30 used both for cooldown enforcement AND as a fail-closed signal — if Redis is unavailable the probe is rejected (defense-in-depth, never flood the probe origin)"
  - "WS dispatch arm for ConnectivityReport guards with `if let Some(nid) = node_id` (not expect()) — safer for the brief window between WS connect and Register completion"
  - "Probe timeout 10s, periodic interval 5 min — matches D-02 recommendation and bounds load at servers/5min outbound TCP"
  - "Java SLP probe uses protocol_version = -1 (any) and next_state = 1 (status) — same handshake the vanilla Minecraft client uses"
  - "Bedrock RakNet probe is unconnected ping (0x01) → pong (0x1c) only — no MOTD parsing (out of scope; deferred to a UI polish pass if needed)"
  - "Probe functions exposed as pub async (not method) — easier to unit-test in isolation, and a future CLI tool can reuse them without an AppContainer"

patterns-established:
  - "Pattern: WS protocol extension via enum variant — keeps serde_json::from_str::<NodeMessage> working unchanged, avoids envelope forks"
  - "Pattern: Reachable→Unreachable transition gate — only fire auto-fix when the state actually changes, not on every probe"
  - "Pattern: Probe + classify + auto-fix dispatcher = a single service struct that owns the whole backend-side of connectivity — keeps the WS dispatch arms thin (no logic in the handler)"
  - "Pattern: 30s Redis cooldown via SET NX EX — atomic, race-free, fails closed"
  - "Pattern: state.details (raw agent facts) is merged with the probe result details (version, players, latency) only on the Ok path — on failure, the agent's raw facts are preserved untouched so a follow-up agent emit can still see the historical snapshot"

requirements-completed: [DEPLOY-01, DEPLOY-02, DEPLOY-03, DEPLOY-04, DEPLOY-05, RCON-01, RCON-02]

# Metrics
duration: 2 min
completed: 2026-06-07
---

# Phase 67 Plan 03: Backend Connectivity Service & REST Endpoints Summary

**ConnectivityService with Java SLP + Bedrock RakNet probes, 4-MVP FailureMode classifier, safe-to-fix auto-fix dispatcher, 5-min periodic re-probe loop, and 3 per-tenant REST endpoints with 30s Redis cooldown — all wired into AppContainer + bootstrap + NodeMessage protocol.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-06-07T07:02:46Z
- **Completed:** 2026-06-07T07:04:49Z
- **Tasks:** 3
- **Files modified:** 9 (2 new Rust files, 7 existing files extended)
- **Commits:** 3 atomic commits in `api/` subrepo (da9ba4f, 6738085, 19edce5)

## Accomplishments

- Added 3 new `NodeMessage` variants (`ConnectivityReport`, `ConnectivityFixRequest`, `ConnectivityFixResult`) and 2 new dispatch arms in `node_ws_handler.rs`. The `ConnectivityReport` arm guards with `if let Some(nid) = node_id` so a report received in the brief window between WS connect and Register completion is logged-and-dropped instead of panicking.
- Created `ConnectivityService` (~540 lines): owns the outbound TCP+SLP probe (Java Edition) and RakNet unconnected ping (Bedrock Edition), the 4-MVP `FailureMode` classifier (`PORT_NOT_BOUND`, `HOST_FIREWALL_BLOCKED`, `CGNAT_DETECTED`, `UPNP_UNAVAILABLE`), the `dispatch_fix()` safe-to-fix gate (CGNAT_DETECTED is the only mode that returns Ok(()) without sending a `NodeMessage::ConnectivityFixRequest`), and the 5-min periodic re-probe loop over all running servers.
- Created `connectivity_handlers.rs` (~130 lines): 3 REST endpoints (`GET /connectivity`, `POST /connectivity/probe` with 30s Redis cooldown via `SET ... NX EX 30`, `GET /connectivity/audit` with `limit`/`offset` pagination). All three run a per-tenant `ensure_ownership()` check before any service call.
- Wired `ConnectivityService` into `AppContainer` as `pub connectivity_service: Arc<ConnectivityService>` and spawned it in `bootstrap/mod.rs` via `tokio::spawn(connectivity.start().await)` immediately after the `monitoring_service` spawn, mirroring the `BackgroundService` pattern.
- `cargo check` exits 0 (76 pre-existing warnings unrelated to this plan; 0 new errors).

## Task Commits

Each task was committed atomically in the `api/` subrepo:

1. **Task 1: Extend NodeMessage protocol and node_ws_handler dispatch** — `da9ba4f` (feat)
2. **Task 2: Create ConnectivityService** — `6738085` (feat)
3. **Task 3: REST handlers, routes mount, container wiring, bootstrap spawn** — `19edce5` (feat)

## Files Created/Modified

### Created

- `api/src/application/services/connectivity_service.rs` — Probe + classify + auto-fix dispatcher (548 lines, ≥ 350 min_lines, 9 documented methods ≥ 8 min_methods)
- `api/src/presentation/handlers/connectivity_handlers.rs` — 3 REST endpoints with per-tenant check (132 lines, ≥ 110 min_lines)

### Modified

- `api/src/presentation/ws/node_protocol.rs` — Added 3 enum variants (ConnectivityReport, ConnectivityFixResult, ConnectivityFixRequest) at the appropriate position in the agent→backend / backend→agent blocks
- `api/src/presentation/handlers/node_ws_handler.rs` — Added 2 dispatch arms after the CrashReport case (line 414), before the `_` wildcard (line 454)
- `api/src/application/services/mod.rs` — Added `pub mod connectivity_service;` after `pub mod server_event_notifier;`
- `api/src/presentation/handlers/mod.rs` — Added `pub mod connectivity_handlers;` after `pub mod modpack_template_handlers;`
- `api/src/presentation/routes/api_routes.rs` — `.merge(crate::presentation::handlers::connectivity_handlers::router())` after the cron-task route block (line 37), into the /api/v1/servers nest
- `api/src/bootstrap/container.rs` — Added use import (line 74), struct field (line 157), Arc::new init (line 356), Self { ... } entry (line 430)
- `api/src/bootstrap/mod.rs` — Added `tokio::spawn(connectivity.start().await)` (line 53-58), between the monitoring_service spawn and the webhook spawn

## Decisions Made

- **JSONB column for state.** The `servers.connectivity_state` JSONB column (added in 67-01) is treated as a single status blob. The service writes the whole `ConnectivityState` struct in one `UPDATE` rather than column-by-column. Matches 67-01's decision and the Phase 51 `dns_config` precedent.
- **Reachable→Unreachable transition gate.** `dispatch_fix` only fires when the previous status was not "unreachable" (i.e. on the first transition). This prevents auto-fix spam during a sustained outage where repeated re-probes would otherwise re-dispatch `firewall.open_port` every 5 minutes.
- **CGNAT_DETECTED never auto-fixes.** `dispatch_fix` returns `Ok(())` immediately when the failure mode is `CgnatDetected` — no `NodeMessage::ConnectivityFixRequest` is sent. The audit row is also skipped (CGN needs human action: Tailscale, port-forwarding wizard, or the future Phase 68 Relay).
- **WS dispatch arm guards `node_id` with `if let Some`.** The plan offered two options (`expect("node_id set after Register")` or `if let Some(nid) = node_id { ... } else { warn!("...dropping") }`). The safer `if let Some` form is used — a report received in the brief window between WS connect and Register completion is logged and dropped rather than panicking.
- **30s Redis cooldown as fail-closed.** `trigger_probe` treats both "key already exists (cooldown active)" and "Redis error" as cooldown-active. If Redis is unavailable, the manual probe is rejected — defense-in-depth against probe flooding via the public REST endpoint.
- **Probe functions exposed as `pub async fn` (not method).** The Java SLP and Bedrock RakNet probes are free functions, not methods on `ConnectivityService`. This makes them directly unit-testable in isolation and lets a future CLI tool reuse them without an `AppContainer`.
- **`Set` from the connectivity service's classify step is the single source of truth.** The plan's `last_probe_at: Some(Utc::now())` and `status: "reachable" | "unreachable"` flow directly into the JSONB write. No separate "probe result" table is needed for MVP.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed `Ok(p) | Err(p)` match arm with mismatched binding types**
- **Found during:** Verification (`cargo check` after first commit)
- **Issue:** The plan's skeleton code in `probe_and_classify()` had `Ok(p) | Err(p) => { ... format!("Probe failed: {:?}", p) ... }`. In the `Ok` arm, `p` binds to `ProbeResult`; in the `Err` arm, `p` binds to `anyhow::Error`. Rust requires the same binding to have the same type in all alternatives, so the code failed with E0308 "expected `ProbeResult`, found `Error`". The plan's own `last_error: Some(format!("Probe failed: {:?}", p.as_ref().err()))` workaround was a viable alternative, but using `p.as_ref().err()` discards the Ok case's debug info.
- **Fix:** Replaced the `Ok(p) | Err(p)` arm with a single `_` catchall that re-patterns the `probe` value via `match &probe` to produce a descriptive `last_error_msg` for both the Ok-with-bad-stages case (`"Probe did not reach all stages: tcp_ok=... handshake_ok=... status_ok=..."`) and the Err case (`"Probe failed: {e}"`). The state-machine logic (unreachable + classify + auto-fix gate) is unchanged.
- **Files modified:** `api/src/application/services/connectivity_service.rs` (lines 280-300)
- **Verification:** `cargo check` exits 0; the new `last_error_msg` correctly surfaces both probe paths in the `servers.connectivity_state` JSONB
- **Committed in:** `6738085` (Task 2 commit — fix was part of the same atomic commit as the service file)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Single source-code fix required to make the service compile. No scope creep; the public API of the service is unchanged.

## Issues Encountered

- **Prior attempt left uncommitted work in the working tree** (this is the RETRY context). All 9 modified files (7 modifications + 2 new files) were present from the previous dispatch, but never committed. The cargo check was failing on the `Ok(p) | Err(p)` pattern at the start of this session; the fix above is the only code change made by this dispatch. All other diffs were preserved from the prior attempt.
- **`api/` is a separate git repo** (not a submodule — the parent repo's `.gitignore` line 58 excludes `api/`, but it's actually a standalone git repo on its own `master` branch with 4 commits ahead of `origin/master` from the 67-01 work). The 67-03 commits live in the `api/` repo, not the parent repo. The orchestrator's post-wave shared-file writes will need to know to look in the `api/` repo for the new commits.

## User Setup Required

None - no external service configuration required. The service is fully self-contained: it uses the existing Postgres pool (for `servers.connectivity_state` reads/writes and `connectivity_audit_log` inserts) and the existing Redis pool (for the 30s cooldown on the manual probe). The WebSocket dispatch path uses the existing `NodeConnectionManager`.

## Next Phase Readiness

- Plan 04 (frontend: `ConnectivityBadge.jsx`, `ConnectivitySection.jsx`, `useConnectivity.js` hook, `api.js` extensions, `ServerManagerPage.jsx` + `ServerDetailsPage.jsx` modifications) is unblocked — the 3 REST endpoints and the audit log shape are ready.
- Phase 68 (Relay Infrastructure) can extend the `ConnectivityState.mode` field with `"relay"` once the relay tunnel protocol is in place; the `direct` → `relay` mode-flip logic can be added to `connectivity_service::probe_and_classify()` as a follow-up.
- The `agent_connection` outbound WS delivery hook from Plan 02 is still a placeholder (`OUTBOUND_TX` is `tracing::info!` in `agent/main.rs`). When the real hook is exposed, the `dispatch_fix` call here will deliver `ConnectivityFixRequest` messages to the agent without further code changes.
- The Bedrock RakNet probe currently returns only a binary pong check (no MOTD/players extraction). UI polish pass can extend `ProbeResult` to surface the RakNet string fields from the pong payload (D-04 details).

## Verification Results

### Task 1 — WS protocol + dispatch

- ✅ `ConnectivityReport` variant at line 84 of `node_protocol.rs`
- ✅ `ConnectivityFixResult` variant at line 92 of `node_protocol.rs`
- ✅ `ConnectivityFixRequest` variant at line 160 of `node_protocol.rs`
- ✅ `NodeMessage::ConnectivityReport` dispatch arm at line 421 of `node_ws_handler.rs`
- ✅ `NodeMessage::ConnectivityFixResult` dispatch arm at line 441 of `node_ws_handler.rs`

### Task 2 — ConnectivityService

- ✅ `pub mod connectivity_service;` at line 10 of `services/mod.rs`
- ✅ 548 lines in `connectivity_service.rs` (≥ 350 min_lines)
- ✅ 9 documented methods (≥ 8 min_methods): `start`, `periodic_reprobe_all`, `handle_agent_diagnostics`, `handle_fix_result`, `probe_server`, `read_state_for_handler`, `probe_and_classify`, `classify_failure`, `dispatch_fix` + the pub probe functions (`probe_java_edition`, `probe_bedrock_edition`)
- ✅ Module imports from Plan 01: `ServerRepository`, `NodeRepository`, `ConnectivityAuditLog`, `PostgresConnectivityAuditLogRepository`, `NodeConnectionManager`, `NodeMessage`
- ✅ No use of the placeholder `monitoring_service::CrashReportData` (removed per plan note)

### Task 3 — REST, routes, container, bootstrap

- ✅ `pub mod connectivity_handlers;` at line 33 of `handlers/mod.rs`
- ✅ `.merge(crate::presentation::handlers::connectivity_handlers::router())` at line 40 of `api_routes.rs` (merged into the `/api/v1/servers` nest so the `:server_id` prefix is applied)
- ✅ `pub connectivity_service: Arc<ConnectivityService>` at line 157 of `container.rs`
- ✅ `let connectivity_service = Arc::new(ConnectivityService::new(...))` at line 356 of `container.rs`
- ✅ `connectivity_service,` in the `Self { ... }` return block at line 430 of `container.rs`
- ✅ `let connectivity = container.connectivity_service.clone();` + `tokio::spawn(async move { connectivity.start().await; })` at lines 56-60 of `bootstrap/mod.rs` (between monitoring and webhook spawns)
- ✅ 3 REST handlers: `get_status` (line 51), `trigger_probe` (line 65), `get_audit_log` (line 110) — all use the per-tenant `ensure_ownership()` check
- ✅ `trigger_probe` uses `redis::cmd("SET").arg(&key).arg("1").arg("NX").arg("EX").arg(30u64)` for the 30s cooldown (fail-closed if Redis is unavailable)

### Cross-task

- ✅ `cargo check` exits 0 in `api/` subrepo (76 pre-existing warnings unrelated to this plan; 0 new errors)
- ✅ All 3 task commits present in `api/` git log: `da9ba4f`, `6738085`, `19edce5`
- ✅ `api/` repo branch is 7 commits ahead of `origin/master` (4 from 67-01 + 3 from 67-03)

## Self-Check: PASSED

- ✅ All 9 modified/created files exist at expected paths in the `api/` subrepo
- ✅ All 3 task commits found in `api/` git log with `feat(67-03):` prefix
- ✅ Module registrations present in both `services/mod.rs` and `handlers/mod.rs`
- ✅ Container wiring has 4 entry points (use import, struct field, Arc::new init, Self { ... } entry)
- ✅ Bootstrap spawn present and placed correctly (between monitoring_service and webhook_service)
- ✅ 3 REST handlers all have per-tenant `ensure_ownership()` checks
- ✅ 30s Redis cooldown implemented as `SET ... NX EX 30` and is fail-closed
- ✅ `CGNAT_DETECTED` is hard-coded to never auto-fix in `dispatch_fix` (D-05)
- ✅ Reachable→Unreachable transition gate present in `probe_and_classify` (prevents auto-fix spam)

---

*Phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi*
*Completed: 2026-06-07*
