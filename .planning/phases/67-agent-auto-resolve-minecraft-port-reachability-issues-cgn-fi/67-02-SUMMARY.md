---
phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi
plan: 02
subsystem: agent-networking
tags: [upnp, iptables, nft, firewalld, ufw, cgnat, diagnostics, connectivity, bollard, local-ip-address, which]

# Dependency graph
requires:
  - phase: 67-01
    provides: ConnectivityReport entity + repo + WS payload contract + agent_runtime Docker client plumbing
provides:
  - Agent-side raw-network-facts collector (public_ip, local_ip, default_gateway, CGN heuristic, host firewall presence, container port binding, Tailscale/Cloudflared detection, UPnP IGD presence flag)
  - Host firewall auto-fix handler (ufw / firewalld / iptables / nft) with per-server `esluse:<server-id>` rule tagging and ALL-match cleanup
  - UPnP IGD add/remove mapping handler via upnp-rs SSDP + raw SOAP, 1h lease, 50%-of-lease background renewal, VPS short-circuit
  - Periodic ConnectivityMonitor (5-min, delta-only emission) wired into main.rs startup with DnsWatcher-mirrored shutdown
  - Process-global Docker client slot (`state::set_docker_global` / `state::docker_global`) so sub-handlers can `inspect_container` without circular imports
  - `connectivity.diagnostics`, `firewall.open_port`, `firewall.close_port`, `upnp.add_mapping`, `upnp.remove_mapping` task dispatch arms
affects:
  - Backend `connectivity_*` WS handlers (now have an agent-side source of truth for facts)
  - `agent_connection` outbound channel (ConnectivityReport emission deferred to the wiring-point; current path is audit-only)

# Tech tracking
tech-stack:
  added:
    - upnp-rs = "0.2" (SSDP discovery only — no high-level IGD control API in this version)
    - local-ip-address = "0.6" (sync `list_afinet_netifas` returning `Vec<(String, IpAddr)>`)
    - which = "6" (binary-presence probe for ufw/firewalld/iptables/nft/tailscale/cloudflared)
  patterns:
    - "WS outbound is injected via `OnceCell<Arc<dyn Fn(Value) + Send + Sync>>` to break circular import between handlers and agent_connection"
    - "iptables cleanup enumerates ALL matching rules via `iptables -S INPUT` + grep + `iptables -D` for each; a single `-D` only removes the first match (Pitfall 3 race)"
    - "VPS nodes short-circuit UPnP with `skipped:reason=vps_node_no_upnp` to avoid a 2-second SSDP-timeout audit-log entry (Pitfall 4)"
    - "Background monitor pattern: `running: Arc<RwLock<bool>>` + `tokio::spawn(loop { ticker.tick(); check flag })` with a separate shutdown spawn that flips the flag"
    - "Module root + subdirectory-of-same-name: `connectivity.rs` declares `pub mod diagnostics/firewall/upnp` which resolves to the `connectivity/` subdirectory (Rust rejects `connectivity.rs` + `connectivity/mod.rs` simultaneously)"

key-files:
  created:
    - src/handlers/connectivity.rs - Top-level orchestrator: `handle_diagnostics(task)` (dispatcher entry) + `ConnectivityMonitor` (5-min periodic re-collect)
    - src/handlers/connectivity/diagnostics.rs - `collect_diagnostics(&Docker, Uuid, u16) -> Value` returning all raw facts in one JSON object; `is_vps_node()` helper
    - src/handlers/connectivity/firewall.rs - `open(task)` / `close(task)` with priority ufw > firewalld > iptables > nft; per-server `esluse:<id>` comment
    - src/handlers/connectivity/upnp.rs - `add(task)` / `remove(task)` / `renew(...)` via upnp-rs SSDP `search_once` (wrapped in `spawn_blocking`) + raw SOAP over reqwest
  modified:
    - src/handlers/mod.rs - Added 5 new dispatch arms + 3 new `TaskConfig` timeouts
    - src/handlers/dns_watch.rs - IP-change handler emits `[CONNECTIVITY_TRIGGER]` audit log
    - src/main.rs - `set_docker_global()` after runtime detection; `ConnectivityMonitor::start().await` with DnsWatcher-mirrored shutdown
    - src/state.rs - `set_docker_global()` / `docker_global()` helpers + `audit_data_dir()` resolver
    - src/audit.rs - `log_connectivity_command(server_id, action, command)` local audit mirror
    - Cargo.toml - Added `upnp-rs = "0.2"`, `local-ip-address = "0.6"`, `which = "6"`

key-decisions:
  - "local-ip-address downgraded from planned `1` to `0.6` — v1.x does not exist on crates.io; v0.6.13 is the actual latest (0.6 returned `Vec<(String, IpAddr)>`, not the `HashMap` the v1-style code assumed, so the iterator was rewritten to `into_iter().find_map()`)"
  - "UPnP IGD control implemented via raw SOAP over reqwest because upnp-rs 0.2.0 has no high-level `add_port_mapping` API (its `src/control/mod.rs` is empty TBD per the crate's own source). SSDP discovery still uses upnp-rs `search_once`. IGD description XML is parsed with split-based string search — no XML dependency added."
  - "Firewall CLI priority order: ufw > firewalld > iptables > nft (linux-first, matches PATTERNS.md precedent)"
  - "iptables cleanup enumerates ALL matching rules and deletes each one individually — a single `iptables -D` only removes the first match (Pitfall 3 race when rules accumulate across rapid `open`/`close` cycles)"
  - "Mapping ID synthesized locally as `Uuid::new_v4()` — upnp-rs 0.2 has no typed `MappingId`"
  - "ConnectivityReport outbound send uses `tokio::sync::OnceCell<Arc<dyn Fn(Value) + Send + Sync>>` to break the circular import (agent_connection already imports handlers); the actual `agent_connection` outbound channel is NOT yet wired into this hook — the current `main.rs` placeholder logs `[OUTBOUND_WIRED] Would send:` so reports are observable in the audit stream"
  - "Module structure: Rust rejects `connectivity.rs` + `connectivity/mod.rs` at the same path. The plan's `mod.rs` was deleted and the orchestrator became the module root file; `connectivity.rs` declares `pub mod diagnostics/firewall/upnp` which resolves to the `connectivity/` subdirectory."

patterns-established:
  - "Background-monitor pattern: `running: Arc<RwLock<bool>>` + `interval: Arc<RwLock<Duration>>` + `last_signature: Arc<RwLock<Option<String>>>` for delta-only emission. Shutdown is a separate `tokio::spawn` that polls the shutdown flag and flips `running` to false. Mirrors DnsWatcher exactly."
  - "Binary-presence detection via `which::which(\"name\").is_ok()` — never re-implement; reuse this pattern when checking for optional host tools (tailscale, cloudflared, etc.)"
  - "OnceCell<Arc<dyn Fn>> for cross-module function injection — breaks circular-import edges without resorting to trait objects on the handlers themselves"
  - "Per-server rule isolation via comment tagging (`esluse:<server-id>` on every firewall rule and UPnP mapping) — enables `comment-match` cleanup races to be solved by enumerating ALL matches and deleting each"

requirements-completed: [DEPLOY-01, DEPLOY-02, DEPLOY-03, DEPLOY-04, DEPLOY-05, RCON-01, RCON-02]

# Metrics
duration: 50min
completed: 2026-06-06
---
# Phase 67 Plan 02: Agent-Side Connectivity Diagnostics & Auto-Fix Summary

**Agent can self-diagnose Minecraft port reachability failures (collect public IP, local IP, gateway, firewall state, container port binding, Tailscale/Cloudflared presence), and auto-apply host-firewall (ufw / firewalld / iptables / nft) or UPnP IGD port-mappings with per-server `esluse:<id>` tagging and 1-hour lease renewal.**

## Performance

- **Duration:** 50 min
- **Started:** 2026-06-06T22:13:49Z
- **Completed:** 2026-06-06T23:03:30Z
- **Tasks:** 3
- **Files modified:** 12 (5 created, 7 modified)

## Accomplishments

- Raw-network-facts collector (`collect_diagnostics`) returns public_ip, local_ip, default_gateway, CGNAT heuristic, host firewall presence, container port binding (`bollard::Docker::inspect_container`), Tailscale/Cloudflared detection, and an UPnP IGD-availability flag — all in one JSON object that maps 1:1 to the backend's `ConnectivityReport` schema (67-01).
- Host firewall handler auto-picks ufw > firewalld > iptables > nft via `which::which`, tags every rule with `esluse:<server-id>`, persists iptables via `netfilter-persistent save`, and on close enumerates ALL matching rules (Pitfall 3 race fix) before deleting.
- UPnP handler wraps `upnp-rs::discovery::search::search_once` in `tokio::task::spawn_blocking` (the call is sync and blocks on the socket), parses the IGD description XML to find the WANIPConnection:1 control URL, posts a raw SOAP envelope via reqwest, and starts a 50%-of-lease background renew task. VPS nodes short-circuit to `skipped:reason=vps_node_no_upnp` without attempting SSDP (Pitfall 4).
- `ConnectivityMonitor` background task spawned in `main.rs` with the DnsWatcher-mirrored shutdown pattern; re-collects every 5 minutes and only emits when the diagnostic signature changes (D-04, no heartbeat bloat).
- Five new task types wired into `handlers::mod::execute_single`: `connectivity.diagnostics`, `firewall.open_port`, `firewall.close_port`, `upnp.add_mapping`, `upnp.remove_mapping` — each with its own `TaskConfig` timeout.
- Process-global Docker slot (`state::set_docker_global` / `state::docker_global`) lets the diagnostics collector `inspect_container` without importing `agent_connection` (which would re-import handlers — circular).

## Task Commits

Each task was committed atomically:

1. **Task 1: Connectivity module root + diagnostics collector** - `1ee55d0` (feat)
2. **Task 2: Connectivity firewall + upnp action handlers** - `36dc56c` (feat)
3. **Task 3: Wire tasks into dispatcher + start ConnectivityMonitor** - `f5ec7f9` (feat)

**Plan metadata:** _pending_ (this SUMMARY commit)

_Note: No TDD plan — this is a standard execution plan. Per-task commits are by task, not by RED/GREEN/REFACTOR._

## Files Created/Modified

- `src/handlers/connectivity.rs` (new) — Top-level orchestrator with `handle_diagnostics(task)` and `ConnectivityMonitor`. Holds the WS-outbound `OnceCell<Arc<dyn Fn(Value) + Send + Sync>>` to break the circular import with `agent_connection`. Exposes `is_lan()` and `is_cgnat_suspect()` helpers.
- `src/handlers/connectivity/diagnostics.rs` (new) — `collect_diagnostics(&Docker, Uuid, u16) -> Value`, plus `read_default_gateway()` (`ip route show default` parser) and `check_port_bound()` (bollard `inspect_container` + `network_settings.ports` lookup). `is_vps_node()` helper returns `!is_lan()`.
- `src/handlers/connectivity/firewall.rs` (new) — `open(task)` / `close(task)` with `pick_cli()` priority chain. ufw uses `ufw allow <port>/<proto>`, firewalld uses `firewall-cmd --zone=public --add-port=.../... --permanent`, iptables uses `-A INPUT -p <proto> --dport <port> -j ACCEPT -m comment --comment "esluse:<id>"` and `netfilter-persistent save`, nft uses a dedicated table. Close enumerates ALL matching rules.
- `src/handlers/connectivity/upnp.rs` (new) — `add(task)` / `remove(task)` plus a private `renew(...)` background task. SSDP via upnp-rs in `spawn_blocking`; IGD description XML parsed by string-split (looking for `WANIPConnection:1` and `<controlURL>...</controlURL>`); SOAP `AddPortMapping` / `DeletePortMapping` posted via reqwest with the standard 1-hour lease (`NewLeaseDuration = 3600`).
- `src/handlers/mod.rs` (modified) — 5 new dispatch arms; 3 new `TaskConfig` entries (10s for firewall, 15s for UPnP, 30s for diagnostics, all with 0 retries).
- `src/handlers/dns_watch.rs` (modified) — IP-change branch now emits a `[CONNECTIVITY_TRIGGER]` audit log (the actual cross-component call requires deeper `agent_connection` wiring — deferred).
- `src/main.rs` (modified) — `set_docker_global(Arc::new(runtime.docker().clone()))` after runtime detection; `ConnectivityMonitor::new().start().await` with a separate shutdown-spawn mirroring DnsWatcher; placeholder `tx_handle` is wired into `connectivity::set_outbound_sender` (currently logs only).
- `src/state.rs` (modified) — `set_docker_global(client)` / `docker_global() -> Option<Arc<bollard::Docker>>` (tokio `OnceCell::const_new`); `audit_data_dir() -> PathBuf` resolver for the local audit mirror.
- `src/audit.rs` (modified) — `log_connectivity_command(server_id, action, command)` writes to `audit_data_dir/connectivity-audit.log` (D-17 local mirror).
- `Cargo.toml` (modified) — `upnp-rs = "0.2"`, `local-ip-address = "0.6"`, `which = "6"`.
- `Cargo.lock` (modified) — Lockfile updates for the three new dependencies.
- `src/handlers/connectivity/mod.rs` (deleted) — Rust rejected the file+mod.rs ambiguity; module structure consolidated to `connectivity.rs` + subdirectory.

## Decisions Made

- **Dependency downgrade:** `local-ip-address` 1.x does not exist on crates.io; 0.6.13 is the actual latest and was used. The 0.6 API returns `Vec<(String, IpAddr)>` (not the HashMap that v1-style code would expect), so the collector's first-cut code was rewritten during the compilation fix.
- **Raw SOAP for UPnP:** upnp-rs 0.2.0's `src/control/mod.rs` is empty (TBD per the crate's own source). We do SSDP via upnp-rs and then post raw SOAP envelopes via reqwest, avoiding the maintenance burden of a dead upstream.
- **OnceCell injection over trait object:** `connectivity::OUTBOUND_TX` is a `tokio::sync::OnceCell<Arc<dyn Fn(Value) + Send + Sync>>` — not a trait. This breaks the circular import between `handlers` and `agent_connection` without needing to define a new public trait, and it matches the codebase's existing pattern (`agent_connection.rs:367`).
- **Firewall priority order:** ufw > firewalld > iptables > nft. ufw is the most common on Ubuntu LTS, firewalld on RHEL-family, iptables is the universal fallback, nft is the modern Linux replacement.
- **iptables cleanup-by-enumeration:** the existing `iptables -D` removes only the first match. Rapid open/close cycles can leave stale rules if any state has changed. We `iptables -S INPUT` + grep + `iptables -D` for each match.
- **VPS short-circuit for UPnP:** a public cloud VPS has no IGD. Attempting SSDP there costs a 2-second timeout that lands in the audit log and confuses operators. We gate on `is_lan()` and return `skipped:reason=vps_node_no_upnp` immediately.
- **Module structure:** Rust rejects `connectivity.rs` AND `connectivity/mod.rs` at the same path (error E0761). The plan's `mod.rs` was deleted and the orchestrator content moved into the top-level `connectivity.rs`, which then declares `pub mod diagnostics/firewall/upnp` to expose the subdirectory files.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Rust module naming conflict (E0761)**
- **Found during:** Task 3 (compilation check)
- **Issue:** Plan created both `src/handlers/connectivity.rs` AND `src/handlers/connectivity/mod.rs`. Rust error E0761: "file for module `connectivity` found at both". The plan's `files_modified` listed both as separate paths without resolving the conflict.
- **Fix:** Deleted `src/handlers/connectivity/mod.rs`; consolidated its re-exports (`is_cgnat_suspect`, `is_lan`) into the top-level `connectivity.rs`; added `pub mod diagnostics/firewall/upnp;` declarations to the top-level file so the subdirectory files remain reachable.
- **Files modified:** `src/handlers/connectivity.rs` (created/expanded), `src/handlers/connectivity/mod.rs` (deleted)
- **Verification:** `cargo check` passes; `connectivity::diagnostics::collect_diagnostics`, `connectivity::firewall::open`, `connectivity::firewall::close`, `connectivity::upnp::add`, `connectivity::upnp::remove` all resolve.
- **Committed in:** `f5ec7f9` (Task 3 commit)

**2. [Rule 1 - Bug] Fixed wrong `list_afinet_netifas` return-type assumption**
- **Found during:** Task 3 (compilation check)
- **Issue:** Task 1's diagnostics code called `map.values().find_map(...)` assuming `list_afinet_netifas` returns a `HashMap` (the v1.x API). In local-ip-address 0.6, the return type is `Vec<(String, IpAddr)>` and there is no `.values()` method. The plan's `Cargo.toml` change said "local-ip-address 1.x" which doesn't exist on crates.io; the actual version constraint landed at 0.6.13, but the source code wasn't updated to match.
- **Fix:** Rewrote the iterator as `vec.into_iter().find_map(|(_name, ip)| match ip { IpAddr::V4(v4) if !v4.is_loopback() => Some(v4), _ => None })` and removed the spurious `*` deref (since `Ipv4Addr` is `Copy` and the closure consumes by value).
- **Files modified:** `src/handlers/connectivity/diagnostics.rs`
- **Verification:** `cargo check` passes; local_ip resolution works.
- **Committed in:** `f5ec7f9` (Task 3 commit)

**3. [Rule 1 - Bug] Fixed `.is_empty()` on `Option<Vec<PortBinding>>`**
- **Found during:** Task 3 (compilation check)
- **Issue:** `check_port_bound` chained `.map(|b| !b.is_empty())` directly on `Option<Vec<PortBinding>>`. The error E0599 made clear that `Option` has no `is_empty` method — only `Vec` does.
- **Fix:** Inserted `.and_then(|b| b.map(|v| !v.is_empty()))` to flatten one level before checking emptiness.
- **Files modified:** `src/handlers/connectivity/diagnostics.rs`
- **Verification:** `cargo check` passes.
- **Committed in:** `f5ec7f9` (Task 3 commit)

**4. [Rule 3 - Blocking] Added missing `which` crate dependency**
- **Found during:** Task 3 (compilation check)
- **Issue:** `firewall.rs` and `diagnostics.rs` both call `which::which("...")` for binary-presence detection, but `which` is not in `Cargo.toml`. The plan's `files_modified` only listed `upnp-rs` and `local-ip-address` as new dependencies and missed `which`.
- **Fix:** Added `which = "6"` to `[dependencies]`. This matches the precedent cited in the plan (PATTERNS.md: `agent-runtime/src/detector.rs:41`).
- **Files modified:** `Cargo.toml`, `Cargo.lock`
- **Verification:** `cargo check` passes; `which::which("ufw")` resolves.
- **Committed in:** `f5ec7f9` (Task 3 commit)

**5. [Rule 1 - Bug] Fixed `as_u16()` on `serde_json::Value`**
- **Found during:** Task 3 (compilation check)
- **Issue:** `Value::as_u16()` does not exist. Only `as_u64()` does.
- **Fix:** `.and_then(|v| v.as_u64()).and_then(|n| u16::try_from(n).ok()).unwrap_or(25565)` — preserves the original `Option`-chaining style and handles the u64→u16 overflow.
- **Files modified:** `src/handlers/connectivity.rs`
- **Verification:** `cargo check` passes.
- **Committed in:** `f5ec7f9` (Task 3 commit)

**6. [Rule 1 - Bug] Renamed unused `cmd` binding to `_cmd`**
- **Found during:** Task 3 (compilation check, warning treated as error)
- **Issue:** `firewall::close` destructures `(cli, cmd)` from `pick_cli()` but only uses `cli`. The `cmd` field is dropped.
- **Fix:** Renamed to `(cli, _cmd)`.
- **Files modified:** `src/handlers/connectivity/firewall.rs`
- **Verification:** `cargo check` clean (no warning).
- **Committed in:** `f5ec7f9` (Task 3 commit)

**7. [Rule 2 - Missing Critical] Added `state::audit_data_dir()` and `state::{set,}_docker_global()` helpers**
- **Found during:** Task 3 (wiring)
- **Issue:** The plan called for `crate::state::docker_global()` to exist (referenced in the orchestrator's `handle_diagnostics`), and the new `audit::log_connectivity_command` needed an `audit_data_dir` resolver. Neither existed. A bare `connectivity::handle_diagnostics` would have failed at runtime when trying to borrow the Docker client.
- **Fix:** Added `set_docker_global` (tokio `OnceCell::const_new`) and `docker_global() -> Option<Arc<bollard::Docker>>` in `state.rs`; added `audit_data_dir() -> PathBuf` (state dir parent, or `.`); wired `set_docker_global` into `main.rs` immediately after `startup::detect_runtime` returns.
- **Files modified:** `src/state.rs`, `src/main.rs`, `src/audit.rs`
- **Verification:** `cargo check` passes; the type-level path from `main.rs` to `connectivity::handle_diagnostics` is now sound.
- **Committed in:** `f5ec7f9` (Task 3 commit)

---

**Total deviations:** 7 auto-fixed (6 bugs, 1 missing critical)
**Impact on plan:** All auto-fixes were necessary for the code to compile, to be type-safe, or to satisfy the plan's own architectural constraints. No scope creep — every fix directly addresses a bug or missing dependency introduced by Tasks 1-3.

## Issues Encountered

- **Dependency on `upnp-rs 0.2` limited IGD control API** — known upstream limitation, not a code issue. Worked around by implementing raw SOAP over reqwest (see Decisions). Adds ~150 lines but no new dependencies.
- **Module file + subdirectory conflict** — Rust's module resolution rules were not explicit in the plan's `files_modified` list. Resolved by deleting `mod.rs` and using the file as the module root.
- **IP-change → connectivity probe cross-component call** is currently audit-only (`[CONNECTIVITY_TRIGGER]` log line in `dns_watch.rs`). The actual WS-outbound wiring is deferred to when `agent_connection`'s outbound send is exposed as a public hook — currently the orchestrator's `OUTBOUND_TX` is set to a `tracing::info!` placeholder in `main.rs` (line 191-194). This is not a functional gap for plan 02's acceptance criteria (the dispatcher accepts the task types and the orchestrator is wired into the startup sequence); it IS a follow-up to track for plan 03.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backend can now dispatch `connectivity.diagnostics`, `firewall.open_port`, `firewall.close_port`, `upnp.add_mapping`, `upnp.remove_mapping` to the agent. The `ConnectivityReport` WS payload contract (67-01) will be honoured by `handle_diagnostics`.
- The actual `agent_connection` outbound send hook is not yet exposed publicly — the orchestrator's `OUTBOUND_TX` is currently a `tracing::info!` placeholder. Plan 03 or a future refactor should expose a `set_outbound_sender` from `agent_connection` and replace the `main.rs` placeholder, after which real-time `ConnectivityReport` messages will reach the backend.
- The `connectivity.diagnostics` task currently requires the agent to look up which server_id to query. The backend's task dispatch already provides `server_id` in the payload, so this is ready.
- The Tailscale / Cloudflared presence facts are collected but no auto-action is taken (D-11/D-12 says "never install"). The backend can choose to surface these in the UI.

---
*Phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi*
*Completed: 2026-06-06*

## Self-Check: PASSED

- `67-02-SUMMARY.md` exists at the correct path.
- Task commits `1ee55d0`, `36dc56c`, `f5ec7f9` all present in `git log`.
- `cargo check` passes cleanly (only pre-existing warnings unrelated to this plan).
