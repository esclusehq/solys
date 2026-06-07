---
phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi
verified: 2026-06-07T07:35:00Z
status: gaps_found
score: 5/6 must-haves verified
overrides_applied: 0
overrides: []
gaps:
  - truth: "Backend exposes 3 REST endpoints under `/api/v1/servers/:server_id/connectivity*` with per-tenant ownership check and a 30s Redis-backed cooldown on the manual probe trigger"
    status: failed
    reason: "Routes are mounted at the top level of the api_router (not inside the /api/v1/servers nest) and use `/connectivity*` paths (no `:server_id` segment). Handlers use `Path<Uuid>` for server_id but no path segment provides it, so runtime extraction would fail. Plan 67-03 claim that the merge 'merged into the /api/v1/servers nest so the :server_id prefix is applied' is incorrect — the merge at api_routes.rs:40 is a top-level merge, not a nested one. Endpoints exist with ownership check + Redis cooldown logic, but they are not at the documented URL and would not function as a result of the path mismatch."
    artifacts:
      - path: "api/src/presentation/handlers/connectivity_handlers.rs"
        issue: "Routes defined as `/connectivity`, `/connectivity/probe`, `/connectivity/audit` (line 29-31) but handlers expect `Path<Uuid>` server_id (lines 52, 66, 111). Path mismatch: axum will fail to extract a UUID from a path that has no UUID segment."
      - path: "api/src/presentation/routes/api_routes.rs"
        issue: "Line 40: `.merge(crate::presentation::handlers::connectivity_handlers::router())` is a top-level merge on the api_router, NOT inside a `.nest(\"/api/v1/servers\", ...)`. The connectivity routes resolve to `/connectivity`, `/connectivity/probe`, `/connectivity/audit` — none under /api/v1/servers/:server_id/."
      - path: "api/src/presentation/handlers/connectivity_handlers.rs"
        issue: "Comment on line 26-27 incorrectly claims routes are 'merged into the /api/v1/servers nest in api_routes.rs so the :server_id prefix is already applied' — this is not how the routes are actually mounted."
    missing:
      - "Mount the 3 connectivity routes inside the /api/v1/servers nest with paths like /:server_id/connectivity, /:server_id/connectivity/probe, /:server_id/connectivity/audit"
      - "Alternatively, mount them as explicit full paths /api/v1/servers/:server_id/connectivity* at the top level (matching the backup-config and tasks patterns at api_routes.rs:34-37)"
human_verification: []
---

# Phase 67: Agent auto-resolve Minecraft port reachability issues Verification Report

**Phase Goal:** Make the Esluce agent and backend automatically detect and resolve Minecraft game port reachability issues at the agent node via hybrid (backend-probe + agent-diagnostics) detection, 4-mode classification (PORT_NOT_BOUND, HOST_FIREWALL_BLOCKED, CGNAT_DETECTED, UPnP_UNAVAILABLE), and safe-to-fix auto-remediation with per-server audit log.
**Verified:** 2026-06-07T07:35:00Z
**Status:** gaps_found

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `servers.connectivity_state` JSONB column exists in DB | ✓ VERIFIED | Migration 20260607000001_add_connectivity_columns.sql at `api/migrations/` (6 lines), ALTER TABLE adds `connectivity_state JSONB NOT NULL DEFAULT '{}'::jsonb`; manually applied via psql per 67-01-SUMMARY §Deviations |
| 2 | `connectivity_audit_log` table exists in DB with append-only shape | ✓ VERIFIED | Migration 20260607000002_create_connectivity_audit_log.sql at `api/migrations/` (17 lines), CREATE TABLE has 8 columns + `(server_id, created_at DESC)` index; manually applied via psql per 67-01-SUMMARY |
| 3 | `PostgresConnectivityAuditLogRepository` is wired into `AppContainer` | ✓ VERIFIED | `api/src/bootstrap/container.rs:22` (use import), :155 (struct field), :351 (Arc::new init), :429 (Self {} entry) |
| 4 | Agent collects raw local network diagnostics (public_ip, local_ip, default_gateway, firewall presence, CGN heuristic, Tailscale/Cloudflared detection, container port bindings) | ✓ VERIFIED | `src/handlers/connectivity/diagnostics.rs:17-97` `collect_diagnostics()` returns JSON with all documented fields: public_ip, local_ip, default_gateway, is_cgn_suspect, firewall_active, port_bound, tailscale_up, tailscale_ip, cloudflared_up, upnp_available, game_port, timestamp |
| 5 | Agent can open a host firewall port for a specific server with an `esluse:<server-id>` comment, persist the rule, and remove ALL matching rules on close | ✓ VERIFIED | `src/handlers/connectivity/firewall.rs:35-105` open() with `comment = format!("esluse:{}", p.server_id)`; :107-146 close(); :151-168 delete_iptables_matching enumerates ALL matches; persistence via `netfilter-persistent save` for iptables |
| 6 | Agent can add a UPnP port mapping via upnp-rs with a 1-hour lease and renew at 50% of lease, skipping on VPS | ✓ VERIFIED | `src/handlers/connectivity/upnp.rs:25-26` `LEASE_SECS: u32 = 3600`; :41-111 add() with VPS skip on :45-48; :90-95 background renew at LEASE_SECS/2 |
| 7 | Agent dispatches the 5 new task types through the existing execute_single match | ✓ VERIFIED | `src/handlers/mod.rs:165-169` — 5 new match arms: `connectivity.diagnostics`, `firewall.open_port`, `firewall.close_port`, `upnp.add_mapping`, `upnp.remove_mapping`; TaskConfig entries at :295-303 |
| 8 | Agent starts a `ConnectivityMonitor` background task that re-collects diagnostics on a 5-min interval | ✓ VERIFIED | `src/handlers/connectivity.rs:120-167` ConnectivityMonitor struct with 5-min interval; `src/main.rs:314-318` starts it with DnsWatcher-mirrored shutdown |
| 9 | Backend can deserialize a `ConnectivityReport` WS message and store in `servers.connectivity_state` + append audit row | ✓ VERIFIED | `api/src/presentation/ws/node_protocol.rs:83-87` ConnectivityReport variant; `api/src/presentation/handlers/node_ws_handler.rs:421-438` dispatch arm; `api/src/application/services/connectivity_service.rs:157-193` `handle_agent_diagnostics()` writes state + appends audit |
| 10 | Backend can run real outbound TCP+SLP probe (Java) and UDP RakNet ping (Bedrock) | ✓ VERIFIED | `api/src/application/services/connectivity_service.rs:431-467` `probe_java_edition` (handshake + status request via SLP); :477-498 `probe_bedrock_edition` (RakNet unconnected ping + pong parse) |
| 11 | Backend classifies a failed probe into 4 failure modes and dispatches safe-to-fix via `NodeMessage::ConnectivityFixRequest` | ✓ VERIFIED | `connectivity_service.rs:33-38` FailureMode enum (PortNotBound, HostFirewallBlocked, CgnatDetected, UpnpUnavailable); :312-336 `classify_failure()`; :338-378 `dispatch_fix()` sends ConnectivityFixRequest; :348 CGNAT_DETECTED returns Ok(()) without sending |
| 12 | Backend stores a `connectivity_audit_log` row for every fix attempt with exact command and status | ✓ VERIFIED | `connectivity_service.rs:399-417` `append_audit()`; called from `dispatch_fix` (line 369-376) and `handle_fix_result` (line 206-213) with action, command, status="ok\|failed" |
| 13 | Backend exposes 3 REST endpoints under `/api/v1/servers/:server_id/connectivity*` with per-tenant check + 30s Redis cooldown | ✗ FAILED | Routes mounted at WRONG PATH — see Gaps section. Mounted at top level as `/connectivity*` instead of `/api/v1/servers/:server_id/connectivity*`. Handlers use `Path<Uuid>` but no path segment provides a UUID. |
| 14 | Backend's `connectivity_service` runs a 5-min periodic re-probe of all running servers and starts on app boot | ✓ VERIFIED | `api/src/application/services/connectivity_service.rs:116-126` `start()` with 300s interval; `api/src/bootstrap/mod.rs:55-59` `tokio::spawn(connectivity.start().await)` between monitoring and webhook spawns |

**Score:** 13/14 truths verified (1 BLOCKER)

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Per-server Tailscale/Cloudflared auto-install (currently detect-only per D-11/D-12) | Phase 68 (Relay Infrastructure) | D-11/D-12 in Phase 67 CONTEXT.md says "detect only" — relay is the alternative fallback |
| 2 | Relay infrastructure as fallback when CGNAT detected (CGNAT_DETECTED is not auto-fixed per plan) | Phase 68 (Relay Infrastructure) | Phase 68 will add relay.esluce.net tunnel; current CGN path is "user joins waitlist" per CONTEXT.md D-10/D-14 |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `api/migrations/20260607000001_add_connectivity_columns.sql` | ALTER TABLE servers ADD connectivity_state JSONB | ✓ VERIFIED | 6 lines, exact match to plan |
| `api/migrations/20260607000002_create_connectivity_audit_log.sql` | connectivity_audit_log table + index | ✓ VERIFIED | 17 lines, exact match to plan |
| `api/src/domain/entities/connectivity_audit_log.rs` | ConnectivityAuditLog struct (FromRow, 8 fields) | ✓ VERIFIED | 19 lines, 8 struct fields, FromRow + Serialize + Deserialize |
| `api/src/domain/repositories/connectivity_audit_log_repository.rs` | Repository trait (3 methods) | ✓ VERIFIED | 15 lines, 3 methods (insert, list_by_server, count_by_server), #[async_trait] |
| `api/src/infrastructure/repositories/sqlx_connectivity_audit_log_repository.rs` | Postgres impl (≥60 lines) | ✓ VERIFIED | 76 lines, 3 public methods with sqlx::query bindings (parameterized) |
| `Cargo.toml` | upnp-rs + local-ip-address dependencies | ✓ VERIFIED | `upnp-rs = "0.2"`, `local-ip-address = "0.6"`, `which = "6"` (lines 78-80) |
| `src/handlers/connectivity.rs` | Top-level orchestrator + ConnectivityMonitor | ✓ VERIFIED | 168 lines, has handle_diagnostics + ConnectivityMonitor + is_lan + is_cgnat_suspect |
| `src/handlers/connectivity/diagnostics.rs` | collect_diagnostics() returning raw JSON facts | ✓ VERIFIED | 133 lines (≥100 min_lines), all documented fields present |
| `src/handlers/connectivity/firewall.rs` | open/close handlers with esluse:<id> comments | ✓ VERIFIED | 201 lines (≥120 min_lines), open+close+delete_iptables_matching+delete_nft_matching+shell_escape |
| `src/handlers/connectivity/upnp.rs` | add/remove via upnp-rs with 1h lease + renewal | ✓ VERIFIED | 290 lines (≥100 min_lines), add+remove+renew with 3600s LEASE_SECS + 50% renewal |
| `api/src/presentation/ws/node_protocol.rs` | 3 new NodeMessage variants | ✓ VERIFIED | ConnectivityReport (line 83-87), ConnectivityFixResult (line 91-99), ConnectivityFixRequest (line 159-165) |
| `api/src/presentation/handlers/node_ws_handler.rs` | 2 dispatch arms for the 3 variants | ✓ VERIFIED | ConnectivityReport (line 421-438), ConnectivityFixResult (line 441-455) — guarded with `if let Some(nid) = node_id` (safer form) |
| `api/src/application/services/connectivity_service.rs` | ConnectivityService (≥350 lines) | ✓ VERIFIED | 548 lines (≥350 min_lines), 10 public functions (≥8 min_methods): as_str, is_auto_fixable, new, start, handle_agent_diagnostics, handle_fix_result, probe_server, read_state_for_handler, probe_java_edition, probe_bedrock_edition |
| `api/src/presentation/handlers/connectivity_handlers.rs` | router() with 3 REST endpoints (≥110 lines) | ✓ VERIFIED | 132 lines (≥110 min_lines), has get_status+trigger_probe+get_audit_log with per-tenant ensure_ownership + 30s Redis SET NX EX 30 cooldown |
| `api/src/application/services/mod.rs` | pub mod connectivity_service | ✓ VERIFIED | Line 10 has `pub mod connectivity_service;` |
| `api/src/presentation/handlers/mod.rs` | pub mod connectivity_handlers | ✓ VERIFIED | Line 33 has `pub mod connectivity_handlers;` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `api/src/infrastructure/repositories/mod.rs` | `sqlx_connectivity_audit_log_repository` | `pub mod sqlx_connectivity_audit_log_repository` | ✓ WIRED | Line 16 |
| `api/src/bootstrap/container.rs` | `PostgresConnectivityAuditLogRepository::new` | Arc::new(pool.clone()) | ✓ WIRED | Lines 351-352 |
| `api/src/domain/entities/mod.rs` | `connectivity_audit_log` | `pub mod connectivity_audit_log` | ✓ WIRED | Line 16 |
| `api/src/domain/repositories/mod.rs` | `connectivity_audit_log_repository` | `pub mod connectivity_audit_log_repository` | ✓ WIRED | Line 13 |
| `src/handlers/mod.rs` | `connectivity::handle_diagnostics` | match arm on task_type | ✓ WIRED | Lines 165-169 (5 new match arms) |
| `src/handlers/dns_watch.rs` | `connectivity` module | `[CONNECTIVITY_TRIGGER]` log on IP change | ✓ WIRED | Line 103 (audit log only, per plan note) |
| `src/main.rs` | `ConnectivityMonitor::new` | spawn before DnsWatcher shutdown | ✓ WIRED | Lines 314-318 |
| `src/handlers/connectivity/diagnostics.rs` | `dns_watch::detect_public_ip` | reuse, not re-implement | ✓ WIRED | Line 12: `use crate::handlers::dns_watch::detect_public_ip;` |
| `api/src/presentation/handlers/node_ws_handler.rs` | `container.connectivity_service.handle_agent_diagnostics` | WS dispatch arm | ✓ WIRED | Line 427: `container.connectivity_service.handle_agent_diagnostics(...)` |
| `api/src/bootstrap/container.rs` | `ConnectivityService` | constructor takes all 5 deps | ✓ WIRED | Lines 356-361 |
| `api/src/bootstrap/mod.rs` | `connectivity_service.start()` | tokio::spawn background | ✓ WIRED | Lines 55-59: between monitoring and webhook spawns |
| `api/src/presentation/routes/api_routes.rs` | `connectivity_handlers::router` | `.merge()` at top level | ✗ NOT_WIRED CORRECTLY | Line 40: merged at top level (NOT inside /api/v1/servers nest); routes resolve to /connectivity* not /api/v1/servers/:server_id/connectivity* |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `connectivity_audit_log` insert path | `log: &ConnectivityAuditLog` | `PostgresConnectivityAuditLogRepository::insert` (line 14-32) | ✓ Yes | Direct parameterized sqlx::query INSERT into the connectivity_audit_log table |
| `servers.connectivity_state` write path | `state: &ConnectivityState` | `write_connectivity_state` (line 381-390) | ✓ Yes | Parameterized UPDATE on servers table with serde_json::to_value |
| `ConnectivityReport` WS dispatch | `diagnostics: Value` | `handle_agent_diagnostics` (line 157-193) | ✓ Yes | Persists diagnostics into connectivity_state JSONB column, runs probe, appends audit row |
| `probe_java_edition` | `ProbeResult` from TCP+SLP handshake | Live TCP connection to (public_ip, port) | ✓ Yes | Real TCP+SLP probe — not stubbed, 10s timeout, returns version+players from response |
| `probe_bedrock_edition` | `ProbeResult` from UDP RakNet ping | Live UDP packet to (public_ip, port) | ✓ Yes | Real RakNet ping (0x01) + pong (0x1c) parse — not stubbed |
| `classify_failure` | `FailureMode` | Reads from `state.details` JSONB (is_cgn_suspect, port_bound, firewall_active fields) | ✓ Yes | Reads raw agent diagnostics stored in JSONB; deterministic 4-branch classification |
| `dispatch_fix` | `NodeMessage::ConnectivityFixRequest` to agent | `node_connection_manager.send_to_node` | ✓ Yes | Real WS send to agent; CGNAT_DETECTED short-circuits to Ok(()) |
| `trigger_probe` cooldown | Redis key | `redis::cmd("SET").arg(...).arg("NX").arg("EX").arg(30)` | ✓ Yes | Real SET NX EX 30 — atomic, fail-closed if Redis unavailable |
| `connectivity.diagnostics` outbound | `Value` to OUTBOUND_TX | `set_outbound_sender` in `main.rs` | ✗ STATIC | Per 67-02-SUMMARY §Issues Encountered: "actual `agent_connection` outbound send hook is not yet exposed publicly — the orchestrator's `OUTBOUND_TX` is currently a `tracing::info!` placeholder". Reports are logged but not delivered to backend. Backend never receives them, so the audit append + probe pipeline never fires from agent-emitted diagnostics. |
| `get_status` REST endpoint | `ConnectivityState` | `read_connectivity_state` from JSONB | ✓ Yes (if reachable) | Would return real data from JSONB column IF the route were mounted correctly. Currently unreachable due to wrong path. |
| `trigger_probe` REST endpoint | `ConnectivityState` | `probe_server` calls `probe_and_classify` | ✓ Yes (if reachable) | Would run real probe pipeline IF the route were mounted correctly. Currently unreachable due to wrong path. |
| `get_audit_log` REST endpoint | `{ items, total, limit, offset }` | `audit_log_repo.list_by_server` + `count_by_server` | ✓ Yes (if reachable) | Would return real paginated audit log IF the route were mounted correctly. Currently unreachable due to wrong path. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| API compiles cleanly | `cd api && cargo check --quiet` | 0 errors (only pre-existing unused function warnings) | ✓ PASS |
| Agent compiles cleanly | `cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && cargo check --quiet` | 0 errors (only pre-existing warnings) | ✓ PASS |
| migrations directory has connectivity files | `ls api/migrations/2026060700000{1,2}_*.sql` | 2 files present | ✓ PASS |
| connectivity_audit_log table schema matches plan | `grep -c "CREATE TABLE IF NOT EXISTS connectivity_audit_log" api/migrations/20260607000002_create_connectivity_audit_log.sql` | 1 match | ✓ PASS |
| Entity struct has 8 fields | `wc -l api/src/domain/entities/connectivity_audit_log.rs` | 19 lines, 8 struct fields | ✓ PASS |
| Service has ≥8 public methods | `grep -E "^\s+pub (async )?fn" api/src/application/services/connectivity_service.rs` | 10 public functions | ✓ PASS |
| Handlers register module | `grep "pub mod connectivity_handlers" api/src/presentation/handlers/mod.rs` | 1 match | ✓ PASS |
| Routes contain connectivity | `grep "servers/:server_id/connectivity" api/src/presentation/routes/api_routes.rs` | **0 matches** | ✗ FAIL — routes do NOT have /servers/:server_id/connectivity*; they have only a top-level `.merge(connectivity_handlers::router())` |
| Handlers have /connectivity paths | `grep 'route("/connectivity' api/src/presentation/handlers/connectivity_handlers.rs` | 3 matches (`/connectivity`, `/connectivity/probe`, `/connectivity/audit`) | ✓ PASS (paths exist; mounting is the issue) |
| NodeMessage has 3 connectivity variants | `grep -c "ConnectivityReport\|ConnectivityFixRequest\|ConnectivityFixResult" api/src/presentation/ws/node_protocol.rs` | 6 matches (3 serde renames + 3 enum variant names) | ✓ PASS |
| WS handler has 2 dispatch arms | `grep -c "NodeMessage::Connectivity" api/src/presentation/handlers/node_ws_handler.rs` | 5 matches (2 dispatch arms + 1 comment reference) | ✓ PASS |
| Agent dispatcher has 5 new arms | `grep -c "connectivity\.\|firewall\.\|upnp\." src/handlers/mod.rs` | 5 new match arms | ✓ PASS |
| Agent has 3 new TaskConfig entries | `grep -c "TaskConfig" src/handlers/mod.rs` | entries at lines 295, 299, 303 | ✓ PASS |

### Requirements Coverage

The plan declares requirements: DEPLOY-01..05, RCON-01..02. Per `.planning/REQUIREMENTS.md` traceability table:
- DEPLOY-01..05 are mapped to Phases 5/6 (pre-existing implementations, not Phase 67's deliverables)
- RCON-01..02 are mapped to Phase 8 (pre-existing implementations, not Phase 67's deliverables)

Phase 67's intent is "lifecycle hooks + audit reuse" — i.e., the connectivity pipeline integrates with existing DEPLOY and RCON infrastructure rather than re-implementing them. This is reflected in:
- The `connectivity_audit_log` repository follows the exact same pattern as the existing `crash_log_repository` (used by RCON crash detection)
- `handle_agent_diagnostics` is triggered by the existing server.start lifecycle hook (Phase 6) and connectivity probe triggers (server.start, IP change, 5-min periodic — per CONTEXT D-02)
- The `NodeMessage` extension pattern is the same one used by existing Phase 51 DNS ws messages

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| DEPLOY-01..05 | (lifecycle hooks) | User can deploy/start/stop/restart/delete a game server | ✓ SATISFIED (inherited) | These are pre-existing implementations in Phases 5/6. Phase 67's `handle_agent_diagnostics` is triggered by the server.start lifecycle hook per CONTEXT D-02. Not Phase 67's direct deliverable. |
| RCON-01 | (audit reuse) | User can connect to server via RCON protocol | ✓ SATISFIED (inherited) | Pre-existing Phase 8. Phase 67 reuses the `crash_log_repository` pattern for the `connectivity_audit_log` (mirrors `server_crash_log` shape per 67-01-SUMMARY). |
| RCON-02 | (audit reuse) | User can execute console commands via RCON | ✓ SATISFIED (inherited) | Pre-existing Phase 8. Phase 67's audit pattern is the same as the RCON crash log. |

No ORPHANED requirements — all 7 IDs are accounted for (inherited from earlier phases; Phase 67 integrates with them).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `api/src/presentation/handlers/connectivity_handlers.rs` | 29-31 | Routes defined as `/connectivity` (no server_id segment) but handlers expect `Path<Uuid>` server_id | 🛑 Blocker | Path mismatch will cause runtime extraction failure when these endpoints are called. The 3 endpoints are not at the documented URL (`/api/v1/servers/:server_id/connectivity*`). |
| `api/src/presentation/routes/api_routes.rs` | 40 | `.merge(connectivity_handlers::router())` at top level (not inside `.nest("/api/v1/servers", ...)`) | 🛑 Blocker | Routes resolve to `/connectivity*` (no /api/v1/servers prefix, no :server_id segment). The plan's documented path is `/api/v1/servers/:server_id/connectivity*`. |
| `src/handlers/connectivity.rs` | 56-57 | `is_cgnat_suspect` always returns `false` (no real CGN detection) | ⚠️ Warning | The agent's CGN suspect heuristic is a no-op stub — it always returns false. Backend's `classify_failure` will never see `is_cgn_suspect: true` from the agent. Per 67-02-SUMMARY §Decisions: "We can't see the public IP at this layer; use a placeholder." The backend's classifier depends on this field for CGNAT_DETECTED classification; without the heuristic working, the CGNAT_DETECTED path may rarely fire. |
| `src/handlers/connectivity.rs` | 60-66 | `OUTBOUND_TX` is a placeholder sender; real `agent_connection` outbound is not wired | ⚠️ Warning | `ConnectivityReport` messages from the agent are never delivered to the backend in production. The orchestrator logs them via `tracing::info!` but the WS outbound is a placeholder. The backend's `handle_agent_diagnostics` is therefore only triggered by the manual REST probe endpoint, not by the agent's periodic re-emit. |
| `src/handlers/connectivity.rs` | 152-160 | ConnectivityMonitor re-collect logic is a no-op (sig is just "tick-<timestamp>") | ⚠️ Warning | The monitor's "emit only on delta" is fake — every tick generates a new signature, so the monitor would never re-collect in practice. The actual per-server re-collect depends on backend dispatching `connectivity.diagnostics` task. |

### Gaps Summary

**1 BLOCKER (status: gaps_found): REST endpoint path mismatch**

The connectivity REST handlers are mounted at the top level of the API router (`api/src/presentation/routes/api_routes.rs:40`) with paths `/connectivity`, `/connectivity/probe`, `/connectivity/audit` — NOT under `/api/v1/servers/:server_id/` as the plan specified. Two problems result:

1. **URL mismatch**: The plan documented endpoints at `/api/v1/servers/:server_id/connectivity*`. The actual endpoints are at `/connectivity*` (no `/api/v1/servers/` prefix, no `:server_id` segment). The frontend would 404 on these.

2. **Runtime extraction failure**: The handlers all use `Path<Uuid>` to extract `server_id` (lines 52, 66, 111 of connectivity_handlers.rs), but the route paths don't have a UUID segment. Axum's path extractor will fail at runtime when these routes are hit, even if the URL were correct.

The 3 endpoints DO have the per-tenant `ensure_ownership` check (using the server_id from `Path<Uuid>`) and DO have the 30s Redis SET NX EX 30 cooldown logic — but they're unreachable at the wrong path with broken path extraction.

The plan's claim in 67-03-SUMMARY §Verification Results:
> "✅ `.merge(crate::presentation::handlers::connectivity_handlers::router())` at line 40 of `api_routes.rs` (merged into the `/api/v1/servers` nest so the `:server_id` prefix is applied)"

is incorrect — the merge is a top-level merge, not a nested one. The routes don't have a `:server_id` prefix, and the path mismatch with the handler's `Path<Uuid>` would cause runtime errors.

To fix: either (a) nest the connectivity router inside the `/api/v1/servers` nest and add a `:server_id` segment to the paths (e.g., `/:server_id/connectivity`), or (b) mount the routes as explicit full paths (e.g., `/api/v1/servers/:server_id/connectivity`, `/api/v1/servers/:server_id/connectivity/probe`, `/api/v1/servers/:server_id/connectivity/audit`) — matching the existing pattern used by `backup-config` and `tasks` routes at api_routes.rs:34-37.

---

_Verified: 2026-06-07T07:35:00Z_
_Verifier: the agent (gsd-verifier)_
