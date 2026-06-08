---
phase: 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv
plan: 04
subsystem: relay
tags: [heartbeat, jitter, bandwidth, state-logging, per-server]
requires:
  - phase: 69-02
    provides: PerServerRuntime with bytes_transferred, control_tx, tunnel_start
provides:
  - Heartbeat staggering with 0-10s random jitter to prevent thundering herd
  - active_servers_with_bandwidth() function for per-server bandwidth metrics
  - Structured state transition logging at all 6 lifecycle points
affects: [69-05]

tech-stack:
  added: [rand = "0.8"]
  patterns:
    - Heartbeat staggering via rand::thread_rng().gen_range(0..=10_000) before heartbeat task init
    - State transition logs with server_id = %server_id for per-server observability
    - Bandwidth counter access via active_servers_with_bandwidth() returning (Uuid, u64, Duration)

key-files:
  created: []
  modified:
    - Cargo.toml — added rand = "0.8" dependency
    - Cargo.lock — updated with explicit rand dependency
    - src/handlers/relay_client.rs — jitter, bandwidth accessor, state logging

key-decisions:
  - "Jitter placed in connect_and_run BEFORE heartbeat task spawn (not inside run_heartbeat_task), so tunnel is fully established before the one-time delay"
  - "server_id added as parameter to run_heartbeat_task to enable structured per-server logging in heartbeat/rekey events"
  - "active_servers_with_bandwidth() collects server IDs first, then queries per-server state to avoid holding RwLock across .await points"

requirements-completed: []

duration: 8min
completed: 2026-06-09
---

# Phase 69 Plan 04: Heartbeat staggering, per-server bandwidth accounting, and state transition logging

**0-10s random heartbeat jitter to prevent thundering herd, active_servers_with_bandwidth() metrics accessor, and structured tracing logs at all 6 tunnel lifecycle states**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-09T04:45:00Z
- **Completed:** 2026-06-09T04:50:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- **Heartbeat staggering:** Added `rand::thread_rng().gen_range(0..=10_000)` ms sleep before the heartbeat task's ticker init, preventing N simultaneous heartbeats when N per-server tunnels restart concurrently (e.g. after agent WS reconnect). `tracing::debug!` log confirms stagger with server_id and jitter_ms.
- **Bandwidth accessor:** `active_servers_with_bandwidth()` returns `Vec<(Uuid, u64, Duration)>` — a snapshot of all active per-server byte counts and tunnel uptimes. Designed to avoid holding `RwLock` across `.await` points by collecting server IDs first.
- **State transition logging:** Structured `tracing::info!` / `tracing::warn!` / `tracing::debug!` at all 6 lifecycle points with `server_id = %server_id` context:
  1. **Connecting** — before WS upgrade to gateway
  2. **Connected** — after TunnelConnect sent, control stream established
  3. **Heartbeat** — per-tick on the 10s heartbeat loop
  4. **Reconnecting** — on connect_and_run failure with delay in seconds
  5. **Rekeying** — when 24h uptime or 100 GiB threshold is hit
  6. **Disconnected permanently** — when loop exits via shutdown

## Task Commits

Each task was committed atomically:

1. **Task 1: Add heartbeat staggering with 0-10s random jitter** — `5bf56b3` (feat)
2. **Task 2: Add per-server bandwidth accessor and state transition logging** — `d017797` (feat)

## Files Created/Modified

- `Cargo.toml` — Added `rand = "0.8"` dependency
- `Cargo.lock` — Updated with explicit rand dependency for solys package
- `src/handlers/relay_client.rs` — Three enhancement groups:
  - Heartbeat jitter code (lines 463-471 in final file)
  - `active_servers_with_bandwidth()` function (lines 272-302)
  - 6 state transition log points scattered across run_relay_client, connect_and_run, run_heartbeat_task

## Decisions Made

- **Jitter placement:** Placed in `connect_and_run` BEFORE the heartbeat task spawn (`tokio::time::sleep(Duration::from_millis(jitter_ms))`) rather than inside the heartbeat task. This ensures the yamux session, control stream, and control_tx channel are fully established before the one-time delay. Subsequent heartbeats run at the standard 10s interval.
- **server_id param for heartbeat task:** Added `server_id: Uuid` to `run_heartbeat_task`'s parameter list. This is necessary for structured logging in the rekey and heartbeat events. The call site in `connect_and_run` passes `per_server_cfg.server_id` via a captured variable.
- **Bandwidth accessor pattern:** Uses a two-phase approach — collect server IDs under the read lock, then iterate with individual lock acquisitions — to avoid holding the `RwLock` across `.await` points inside closures.
- **Log message format:** Follows structured tracing conventions: `info!(server_id = %id, ...)` for state transitions, `warn!(server_id = %id, error = %e, ...)` for reconnects, `debug!(server_id = %id, ...)` for periodic heartbeats.

## Deviations from Plan

None — plan executed as written. The plan's action pseudo-code showed a `ticker.tick().await` first-tick pattern, but the actual 69-02 codebase uses a `tokio::time::interval` without an explicit first tick. The jitter was placed at the semantically correct location (before heartbeat task spawn in `connect_and_run`) rather than inline in `run_relay_client`.

## Issues Encountered

None.

## Threat Surface

No new threat surface introduced beyond what's modeled in the plan's threat register (T-69-10 accept, T-69-11 accept, T-69-12 accept). All changes are internal to the agent process:
- Heartbeat staggering is pure timing optimization — no security impact
- bytes_transferred counter always uses `Ordering::Relaxed` (monotonic increment, no ordering dependency)
- State logs contain only `server_id` (UUID), `subdomain`, and error messages — no secrets

## Next Phase Readiness

- Heartbeat staggering complete — ready for gateway changes in 69-05
- Bandwidth metrics accessible via `active_servers_with_bandwidth()` for future monitoring/observability plans
- Per-server state transitions visible in agent logs — assists debugging gateway issues in 69-05
- Ready for Plan 69-05 (Gateway: auth.rs 1:N relay_token→server_id mapping, tunnel.rs N concurrent WS from same agent IP)

## Self-Check: PASSED

- ✅ All 3 files exist (Cargo.toml, relay_client.rs, SUMMARY.md)
- ✅ All 3 commits found in git log (5bf56b3, d017797, f12a260)
- ✅ `rand::thread_rng().gen_range(0..=10_000)` used for heartbeat jitter
- ✅ `tokio::time::sleep(Duration::from_millis(jitter_ms))` before heartbeat ticker init
- ✅ `active_servers_with_bandwidth()` function present with correct signature
- ✅ All 6 state transition logs present (connecting, connected, heartbeat, reconnecting, rekeying, disconnected)
- ✅ All state log lines include `server_id = %server_id`
- ✅ `rand = "0.8"` added to Cargo.toml
- ✅ `Ordering::Relaxed` import present
- ✅ `cargo check` passes (only pre-existing warnings)
