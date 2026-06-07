---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 02
subsystem: infra
tags: relay, websocket, yamux, tunneling, agent-tunnel-client, cloudflare-dns-cleanup, exponential-backoff, rekey

# Dependency graph
requires:
  - phase: 68-01
    provides: "7 relay columns on nodes/servers, find_by_relay_token repository method, 5 NodeMessage variants (TunnelConnect/Disconnect/Heartbeat inbound, ModeOverrideChange/TunnelCloseAck outbound)"
provides:
  - "Outbound WSS reconnect loop with exponential backoff (1s->30s, +/-20% jitter) and Bearer auth"
  - "yamux-over-WebSocket multiplexed tunnel to local 127.0.0.1:25565 MC server"
  - "D-25 rekeying: 24h uptime OR 100 GB transferred triggers clean WS close for fresh handshake"
  - "D-13 CNAME cleanup: remove_cname_record self-loop on every tunnel disconnect (RESOLVED Q7)"
  - "4 new dispatch arms (relay.connect, relay.disconnect, relay.heartbeat, relay.remove_cname_record) + 4 TaskConfig entries"
  - "Agent bootstrap that auto-starts the relay client if AGENT_RELAY_TOKEN is set"
affects:
  - "68-03 (backend relay service will receive TunnelConnect/Disconnect/Heartbeat from this client)"
  - "68-04 (gateway will accept the WSS, validate Bearer token, open yamux session)"
  - "68-05 (dashboard ModeOverrideChange will trigger relay.connect/disconnect on the agent)"

# Tech tracking
tech-stack:
  added:
    - "tokio-yamux 0.3 (multiplexed streams over a single TCP/WS connection)"
    - "tokio-util 0.7 (CancellationToken for the reconnect loop)"
    - "futures 0.3 (SinkExt/StreamExt for the WS<->duplex bridge)"
    - "base64 0.22 + hmac 0.12 + sha2 0.10 + hex 0.4 (HMAC helpers, used in audit log side-channel)"
  patterns:
    - "WS<->duplex bridge for yamux: WebSocketStream is Stream+Sink<Message>, yamux needs AsyncRead+AsyncWrite; bridge task pumps bytes between WS Binary frames and a tokio::io::duplex pair"
    - "D-25 rekey threshold evaluated by the heartbeat task using shared Arc<AtomicU64> byte counter fed by every yamux session"
    - "Process-global RelayConfig + RelayRuntime (mirrors the existing DOCKER_GLOBAL pattern in state.rs)"
    - "Self-loop cleanup: tunnel disconnect calls dns::handle_remove_record directly with a constructed Task (D-13 / RESOLVED Q7) — the dispatcher arm handles the case when the backend sends the same task type"
    - "Idempotent connect: a second relay.connect call while the reconnect loop is running returns Ok(\"already_running\")"

key-files:
  created:
    - src/handlers/relay_client.rs — Reconnect loop, WS+yamux plumbing, heartbeat with rekey, CNAME cleanup
    - src/handlers/relay_session.rs — Per-stream yamux<->TCP bidirectional copy with byte accounting
    - src/handlers/relay.rs — Task dispatch (handle_relay_task delegates to relay_client)
  modified:
    - Cargo.toml — 6 new deps (tokio-yamux, tokio-util, futures, base64, hmac, sha2, hex)
    - src/audit.rs — log_relay_tunnel_event for tunnel up/down/heartbeat/rekey events
    - src/handlers/dns.rs — handle_remove_record for Cloudflare DELETE with 404-as-success
    - src/handlers/mod.rs — 4 dispatch arms + 4 TaskConfig entries
    - src/main.rs — bootstrap_relay_client reads AGENT_RELAY_* env vars, spawns run_relay_client
    - src/state.rs — RelayConfig struct + process-global setter/getter

key-decisions:
  - "Used tokio-yamux 0.3 (latest stable) instead of 0.2 (specified in plan). 0.2 is not readily available in the local cargo registry and the 0.3 API differs significantly (Session-based, concrete StreamHandle type, sync open_stream)."
  - "Implemented a WS<->duplex bridge task instead of trying to make WebSocketStream directly implement AsyncRead+AsyncWrite. This is the standard pattern for running yamux over a WebSocket transport and is the only way to get the byte-stream interface yamux 0.3 requires."
  - "Used `log_relay_tunnel_event` mirroring the existing `log_connectivity_command` pattern (tracing::info! + local audit file + synthetic TaskRejected entry in the global AuditLogger) rather than the plan's `logger.log_event(node_id, category, payload)` call which doesn't exist on the AuditLogger trait."
  - "Stored RelayConfig in a process-global (RelayRuntime) rather than extending agent_config::AgentConfig. This follows the existing DOCKER_GLOBAL pattern and avoids the wide blast radius of modifying the external agent-config crate."
  - "Self-loop implementation: dispatch_remove_cname_record calls dns::handle_remove_record directly with a constructed Task. The dispatcher arm in mod.rs handles the case where the backend sends relay.remove_cname_record; the agent's own teardown path calls the same handler directly without round-tripping through execute_single."

patterns-established:
  - "Pattern 1: WS<->yamux bridge — WebSocketStream<S> is Stream<Msg>+Sink<Msg>; tokio::io::duplex(N) provides AsyncRead+AsyncWrite for yamux; a small bridge task pumps bytes between them via WS Binary messages. The duplex size should match the yamux default window (64 KiB)."
  - "Pattern 2: Rekey threshold watch — heartbeat task checks (tunnel_start.elapsed() >= 24h) || (bytes >= 100 GiB); on hit, calls control.shutdown() which cascades: yamux FIN -> duplex EOF -> bridge sends WS Close -> outer reconnect loop restarts."
  - "Pattern 3: Per-session byte accounting — Arc<AtomicU64> shared between relay_session (writer) and heartbeat task (reader). Relaxed ordering is fine because the rekey threshold is checked periodically; we never need exact values."

requirements-completed:
  - DEPLOY-01
  - DEPLOY-02
  - DEPLOY-03
  - STATUS-01
  - STATUS-02

# Metrics
duration: 31 min
completed: 2026-06-07
---
# Phase 68 Plan 02: Agent Relay Tunnel Client Summary

**Outbound WSS tunnel client with yamux multiplexing, exponential-backoff reconnect, 24h/100GB rekey, and self-cleaning CNAME removal on disconnect (D-13/RESOLVED Q7)**

## Performance

- **Duration:** 31 min
- **Started:** 2026-06-07T07:18:46Z
- **Completed:** 2026-06-07T07:50:00Z
- **Tasks:** 2
- **Files modified:** 9 (3 new, 6 modified)

## Accomplishments

- **Outbound WSS reconnect loop** with exponential backoff (1s → 2s → 4s → 8s → 16s → 30s cap, ±20% jitter). Bearer `relay_token` authentication via the `Authorization: Bearer` header on the WS upgrade request. Uses `tokio_tungstenite::connect_async_tls_with_config` to handle TCP+TLS+WS in one call. TLS 1.3 only (D-03 / V6.2).
- **yamux multiplexing over WebSocket** via a small bridge task that pumps bytes between WS Binary frames and a `tokio::io::duplex(64 KiB)`. yamux runs on the duplex side; gateway-side `StreamHandle`s are spawned as `run_relay_session` tasks that open a `TcpStream` to `127.0.0.1:25565` (or whatever `local_mc_addr` is configured) and do bidirectional copy. Byte counts are atomically accumulated into a shared `Arc<AtomicU64>` for the rekey threshold.
- **D-25 rekey (RESOLVED Q4):** 10s heartbeat task evaluates `(tunnel_start.elapsed() >= 24h) || (bytes_transferred >= 100 GiB)` on every tick. On threshold, the control stream is shut down, the yamux session cascades, the duplex EOFs, the bridge sends `Message::Close`, and the outer reconnect loop establishes a fresh handshake (new TLS session, new yamux nonce).
- **D-13 CNAME cleanup (RESOLVED Q7):** on every tunnel disconnect (graceful OR errored), `dispatch_remove_cname_record` invokes `dns::handle_remove_record` with a constructed Task carrying the DNS api_token/zone_id/record_id. The handler calls Cloudflare's `DELETE /zones/{zone_id}/dns_records/{record_id}` and treats 404 as success (record already gone is fine).
- **4 new dispatch arms** in `src/handlers/mod.rs::execute_single` and **4 TaskConfig entries** in `get_task_config`. `relay.connect` is idempotent (returns `{"status":"already_running"}` if a reconnect loop is already in flight).
- **Bootstrap wiring** in `src/main.rs::bootstrap_relay_client` reads `AGENT_RELAY_TOKEN` (required) plus 8 optional env vars, builds a `RelayConfig`, stores it in the process global, and spawns `run_relay_client` in a tokio task. No-op (with a clear `[RELAY] No AGENT_RELAY_TOKEN set; RelayClient not started` info log) if the token isn't set.
- **Audit log** at `audit_data_dir()/relay-tunnel-audit.log` captures every tunnel lifecycle event (connecting/connected/disconnected/heartbeat/rekey) with node_id, server_id, detail, and RFC3339 timestamp. Mirrors to the global `AuditLogger` as a `TaskRejected` entry (no relay-specific `AuditEvent` variant exists; we deliberately do not extend the external `agent-security` crate from this plan).

## Task Commits

Each task was committed atomically:

1. **Task 1: Add relay deps + audit log + relay.rs task entrypoint + dns handle_remove_record** — `334dd06` (feat)
2. **Task 2: Implement relay client + yamux session + bootstrap wiring** — `2aa8b1b` (feat)

## Files Created/Modified

### Created

- `src/handlers/relay.rs` — Task dispatch shim, delegates to `relay_client::{connect, disconnect, send_heartbeat}`.
- `src/handlers/relay_client.rs` — Full tunnel client: WS+yamux reconnect loop, heartbeat with D-25 rekey, D-13 CNAME cleanup self-loop, exponential backoff with ±20% jitter, WS↔duplex bridge task.
- `src/handlers/relay_session.rs` — Per-stream `yamux::StreamHandle` ↔ `tokio::net::TcpStream` bidirectional copy, feeds byte counts into the shared `Arc<AtomicU64>`.

### Modified

- `Cargo.toml` — 7 dep lines: `tokio-util` (compat), `futures`, `tokio-yamux`, `base64`, `hmac`, `sha2`, `hex`. (The plan listed 6; `tokio-util` was added because `CancellationToken` is not in the agent's dep tree yet.)
- `src/audit.rs` — `log_relay_tunnel_event(node_id, server_id, event_type, detail)` mirroring the existing `log_connectivity_command` pattern.
- `src/handlers/dns.rs` — `handle_remove_record` for the Cloudflare DELETE with 404-as-success.
- `src/handlers/mod.rs` — 4 dispatch arms (relay.connect/disconnect/heartbeat/remove_cname_record) + 4 TaskConfig entries.
- `src/main.rs` — `bootstrap_relay_client` reads `AGENT_RELAY_*` env vars, calls `state::set_relay_config`, spawns `run_relay_client`.
- `src/state.rs` — `RelayConfig` struct + `set_relay_config` / `relay_config` accessors (mirrors the existing `DOCKER_GLOBAL` pattern).

## Decisions Made

- **tokio-yamux 0.3 over 0.2:** the plan's `tokio-yamux = "0.2"` is not present in the local cargo registry; the latest stable is 0.3.18. 0.3's API is `Session::new_client(socket, Config::default())` returning a `Stream<Item=Result<StreamHandle>>`, with `StreamHandle` as a concrete (non-generic) `AsyncRead+AsyncWrite` type. 0.3 fits the agent's tokio stack better and is what all the `tokio_yamux::` examples in the registry target.
- **WS↔duplex bridge over a direct `WebSocketStream` adapter:** `WebSocketStream<S>` implements `Stream<Item=Result<Message>>+Sink<Message>` but **not** `AsyncRead+AsyncWrite`. yamux 0.3 requires the latter. The standard pattern is to bridge via `tokio::io::duplex(N)`: yamux gets the duplex side (which is `AsyncRead+AsyncWrite`), the bridge task reads/writes WS Binary messages on the other side. The duplex size (64 KiB) matches the yamux default window so backpressure is sensible.
- **Process-global `RelayConfig` over extending `agent_config::AgentConfig`:** the plan's example used `state.config.relay.token` but neither `state::AgentState` nor `agent_config::AgentConfig` has a relay field. Extending the external `agent-config` crate would have a wide blast radius (loader, validator, default, env-var mapping, all tests). The existing `DOCKER_GLOBAL` pattern in `state.rs` is the obvious local precedent for "static config that the agent bootstraps once". Env vars (`AGENT_RELAY_*`) populate it.
- **Direct `dns::handle_remove_record` call over `task_queue::enqueue_local`:** the plan said "self-loop to its own task queue" but no local task queue exists in the agent (only a `TaskQueue` struct that nothing drains). Calling `dns::handle_remove_record` directly with a constructed `Task` is the same end result (D-13 cleanup on tunnel disconnect) without requiring new infrastructure. The dispatch arm in `mod.rs` handles the orthogonal case where the **backend** itself sends a `relay.remove_cname_record` task.
- **`AuditLogger::log_event` does not exist:** the plan's example used `logger.log_event(node_id, category, payload).await` but the actual trait only has `log`, `log_task_received`, `log_task_completed`, `log_task_failed`, `log_agent_registered`. The `log_connectivity_command` pattern (tracing + local audit file + structured global entry) is the closest existing pattern. Used a synthetic `TaskRejected` entry to surface relay events in the global structured log without modifying the external `agent-security` crate.
- **`TaskConfig` shape mismatch:** the plan's example `TaskConfig { name, timeout_secs, category }` doesn't match the actual struct (`{ timeout, max_retries, retry_delay_ms, max_retry_delay_ms, backoff_multiplier }`). Added entries using the actual struct shape with reasonable retry-disabled defaults (relay.connect = 30 s, relay.disconnect = 10 s, relay.heartbeat = 5 s, relay.remove_cname_record = 15 s).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used tokio-yamux 0.3 instead of 0.2**

- **Found during:** Task 2 (implementing `relay_client.rs`)
- **Issue:** The plan specified `tokio-yamux = "0.2"` in `Cargo.toml`. The local cargo registry only has 0.3.18. The 0.3 API differs significantly: there's no `Client::new(ws_stream, ...)`; the construction is `Session::new_client(socket, Config::default())`; `StreamHandle` is a concrete (non-generic) `AsyncRead+AsyncWrite` type; `open_stream()` is synchronous (not async) and returns `Result<StreamHandle, Error>`. The session is itself a `Stream<Item=Result<StreamHandle>>` for incoming streams.
- **Fix:** Pinned to `tokio-yamux = "0.3"`. Rewrote the `connect_and_run` and `run_relay_session` call sites to use the 0.3 API. The semantic behavior is identical to the plan's intent.
- **Files modified:** `Cargo.toml`, `src/handlers/relay_client.rs`
- **Verification:** `cargo check` exits 0; `grep tokio-yamux Cargo.toml` returns the 0.3 line
- **Committed in:** `2aa8b1b` (part of Task 2 commit)

**2. [Rule 2 - Missing critical] Added `RelayConfig` + process-global (plan referenced a non-existent `state.config.relay.token`)**

- **Found during:** Task 2 (bootstrapping the RelayClient in main.rs)
- **Issue:** The plan's `main.rs` example used `state.config.relay.token.clone()` but neither `state::AgentState` (servers/container_map/metadata) nor `agent_config::AgentConfig` (backend_url/api_key/heartbeat_interval_secs/...) has a `relay` field. There was nowhere to read the per-node bearer token from.
- **Fix:** Added a `RelayConfig` struct + `set_relay_config` / `relay_config` accessors to `state.rs` (mirrors the existing `DOCKER_GLOBAL` pattern). Added a `bootstrap_relay_client` function in `main.rs` that reads `AGENT_RELAY_TOKEN` plus 8 optional env vars (`AGENT_RELAY_GATEWAY_URL`, `AGENT_RELAY_SUBDOMAIN`, `AGENT_RELAY_PUBLIC_PORT`, `AGENT_RELAY_REGION`, `AGENT_RELAY_LOCAL_ADDR`, `AGENT_RELAY_DNS_API_TOKEN`, `AGENT_RELAY_DNS_ZONE_ID`, `AGENT_RELAY_DNS_RECORD_ID`), builds the `RelayConfig`, stores it in the global, and spawns `run_relay_client`.
- **Files modified:** `src/state.rs`, `src/main.rs`
- **Verification:** `cargo check` exits 0; `grep "pub fn relay_config" src/state.rs` returns 1 match
- **Committed in:** `2aa8b1b` (part of Task 2 commit)

**3. [Rule 1 - Bug] `AuditLogger::log_event` method does not exist — used existing `log_connectivity_command` pattern instead**

- **Found during:** Task 1 (implementing `log_relay_tunnel_event` in audit.rs)
- **Issue:** The plan's example used `logger.log_event(node_id, format!("relay.tunnel.{}", event_type), serde_json::json!({...})).await`. The actual `AuditLogger` trait has only `log(AuditEntry)`, `log_task_received`, `log_task_completed`, `log_task_failed`, `log_agent_registered`. There is no `log_event` method.
- **Fix:** Mirrored the existing `log_connectivity_command` pattern: emit a `tracing::info!` line with the structured `[RELAY_TUNNEL_AUDIT]` prefix, write the same line to `audit_data_dir()/relay-tunnel-audit.log`, and (best-effort) build a synthetic `AuditEntry` with `AuditEvent::TaskRejected { reason: ... }` and pass it to `logger.log(entry)`. This surfaces relay events in the global structured log without extending the external `agent-security` crate. The `TaskRejected` reason field carries the relay-tunnel category and detail, so downstream consumers can grep for `relay.tunnel.*`.
- **Files modified:** `src/audit.rs`
- **Verification:** `cargo check` exits 0; `grep "fn log_relay_tunnel_event" src/audit.rs` returns 1 match
- **Committed in:** `334dd06` (part of Task 1 commit)

**4. [Rule 3 - Blocking] Added `tokio-util` to deps (plan didn't list it but the reconnect loop needs `CancellationToken`)**

- **Found during:** Task 2 (writing `relay_client.rs::run_relay_client`)
- **Issue:** The plan's `run_relay_client(state: Arc<AgentState>, shutdown: CancellationToken)` signature uses a `CancellationToken` from the `tokio_util::sync` module. `tokio-util` was not in the agent's `Cargo.toml` prior to this plan.
- **Fix:** Added `tokio-util = { version = "0.7", features = ["compat"] }` to `[dependencies]`. Used the existing `compat` feature (we don't strictly need it for the cancellation token, but it's the most common feature set and keeps the door open for future `StreamCompat`/`Compat` use in the bridge task).
- **Files modified:** `Cargo.toml`
- **Verification:** `cargo check` exits 0; `cargo tree -e normal -i tokio-util` shows it resolving
- **Committed in:** `2aa8b1b` (part of Task 2 commit)

**5. [Rule 1 - Bug] `TaskConfig` shape in plan doesn't match the struct in the codebase**

- **Found during:** Task 2 (adding TaskConfig entries for the 4 new task types in mod.rs)
- **Issue:** The plan's `TaskConfig { name: "...", timeout_secs: 30, category: "..." }` doesn't match the actual struct (`{ timeout: Duration, max_retries: u32, retry_delay_ms: u64, max_retry_delay_ms: u64, backoff_multiplier: f64 }`). The `name` and `category` fields don't exist; `timeout` is a `Duration`, not `u64`.
- **Fix:** Added entries using the actual struct shape with reasonable retry-disabled defaults:
  - `relay.connect` — 30 s timeout (covers the connect+handshake if the gateway is slow to respond)
  - `relay.disconnect` — 10 s timeout (cap on `tokio::time::timeout` for the join handle)
  - `relay.heartbeat` — 5 s timeout (single mpsc send + tunnel_uptime computation)
  - `relay.remove_cname_record` — 15 s timeout (single CF DELETE call)
- **Files modified:** `src/handlers/mod.rs`
- **Verification:** `cargo check` exits 0; grep for the 4 new TaskConfig entries returns 4 matches
- **Committed in:** `2aa8b1b` (part of Task 2 commit)

**6. [Rule 3 - Blocking] Self-loop cleanup implemented as direct call to `dns::handle_remove_record` (no `task_queue::enqueue_local` exists)**

- **Found during:** Task 2 (writing `dispatch_remove_cname_record` in relay_client.rs)
- **Issue:** The plan said "self-loop to its own task queue" via `state.task_queue.enqueue_local(Task { ... })`. The `TaskQueue` struct exists in `task_queue.rs` but is just a `VecDeque<Task>`-backed type with no consumer — nothing in the agent drains a local task queue back through `execute_single`. There is no `enqueue_local` function.
- **Fix:** `dispatch_remove_cname_record` calls `super::dns::handle_remove_record(task)` directly with a constructed `Task`. The end behavior is identical: the Cloudflare DELETE is made, the result is audit-logged. The dispatch arm `"relay.remove_cname_record" => dns::handle_remove_record(...)` in `mod.rs` is the path used when the **backend** itself sends this task type; both paths converge on the same handler.
- **Files modified:** `src/handlers/relay_client.rs`
- **Verification:** `cargo check` exits 0; `grep "relay.remove_cname_record" src/handlers/mod.rs` returns 2 matches (dispatch + TaskConfig)
- **Committed in:** `2aa8b1b` (part of Task 2 commit)

**7. [Rule 1 - Bug] `WebSocketStream<S>` does not implement `AsyncRead+AsyncWrite` — implemented WS↔duplex bridge**

- **Found during:** Task 2 (passing the WebSocketStream to yamux)
- **Issue:** The plan's interface example showed `tokio_yamux::Client::new(ws_stream, ...)` directly. yamux 0.3 requires `T: AsyncRead + AsyncWrite + Unpin` (tokio's traits). `tokio_tungstenite::WebSocketStream<S>` implements `Stream<Item=Result<Message>>+Sink<Message>` but not the byte traits. The plan's API doesn't exist on either side of the 0.2/0.3 boundary.
- **Fix:** Implemented a `ws_bridge` task in `relay_client.rs` that:
  1. Splits the `WebSocketStream` into `(sink, stream)`.
  2. Creates a `tokio::io::duplex(64 * 1024)` pair.
  3. Loops with `tokio::select!` reading bytes from the duplex side and sending them as `Message::Binary` to the WS sink; concurrently reading `Message::Binary` from the WS stream and writing them to the duplex side.
  4. yamux gets the duplex end (which is `AsyncRead+AsyncWrite+Unpin`).
  This is the standard pattern for running yamux over a WebSocket transport. 64 KiB matches the yamux default window so backpressure propagates sensibly.
- **Files modified:** `src/handlers/relay_client.rs`
- **Verification:** `cargo check` exits 0; `grep "ws_bridge" src/handlers/relay_client.rs` returns the bridge function + spawn site
- **Committed in:** `2aa8b1b` (part of Task 2 commit)

---

**Total deviations:** 7 auto-fixed (1 missing critical, 6 blocking/bug)
**Impact:** All 7 auto-fixes were necessary to make the plan execute against the actual codebase. None of them change the plan's intent — the schema, the behavior, the threat model coverage, and the success criteria are all delivered. The deviations are documented in this SUMMARY so future plans (03-05) know that:
- `RelayConfig` is stored in a process-global accessible via `state::relay_config()` (not on `agent_config::AgentConfig`)
- `AuditLogger::log_event` does not exist; relay events use the `log_relay_tunnel_event` pattern in `audit.rs`
- `task_queue::enqueue_local` does not exist; the D-13 self-loop calls `dns::handle_remove_record` directly with a constructed Task
- yamux runs over a `tokio::io::duplex(64 KiB)` bridged to the WebSocket, not directly on the WebSocketStream

## Issues Encountered

- The plan's interface section shows `tokio_yamux::Client::new(ws_stream, yamux_config())` which doesn't exist in either the 0.2 or 0.3 API. yamux 0.3 is `Session::new_client(socket, Config::default())` and the session is `Stream<Item=Result<StreamHandle>>`. The `StreamHandle` is a concrete (non-generic) `AsyncRead+AsyncWrite` type. Worked through this in deviation #1 + #7.
- The plan's `TaskConfig` example shape doesn't match the actual struct in the codebase. Worked through this in deviation #5.
- The plan's `AuditLogger::log_event` method doesn't exist. Worked through this in deviation #3.
- The `state.config.relay.token` field referenced in the plan's bootstrap example doesn't exist on any current struct. Worked through this in deviation #2.

## Known Stubs

None. The plan establishes the agent's outbound tunnel client behavior end-to-end:
- WS handshake with Bearer auth ✓
- yamux session over the WS (via duplex bridge) ✓
- Reconnect loop with exponential backoff and ±20% jitter ✓
- 10s heartbeat with D-25 rekey (24h uptime or 100 GB) ✓
- D-13 CNAME cleanup on every disconnect ✓
- 4 dispatch arms + 4 TaskConfig entries ✓
- main.rs bootstrap with env-var config ✓

End-to-end tunnel behavior is NOT verifiable in this plan (the gateway in Plan 04 and the backend relay service in Plan 03 must exist to accept the WSS, validate the token, and route yamux streams). This plan only verifies the agent's outbound side compiles, the dispatch is wired correctly, and the bootstrap is in place.

## User Setup Required

None — no external service configuration required for this plan. The `AGENT_RELAY_TOKEN` env var is the only required setting; it's set at provisioning time by the deployment tooling (Plan 03 / Plan 05 will set up the provisioning flow). Until then, the agent gracefully no-ops with a clear `[RELAY] No AGENT_RELAY_TOKEN set; RelayClient not started` info log.

## Next Phase Readiness

- The agent's outbound tunnel client is fully implemented. `cargo check` exits 0; all 4 dispatch arms are wired; the main.rs bootstrap is in place.
- Plan 03 (backend relay service) can now define the `find_by_relay_token` HMAC-auth callback that the gateway calls on every WSS upgrade; the agent will send `Authorization: Bearer {relay_token}` and expect 200 + the WSS upgrade.
- Plan 04 (gateway) can be built and tested: the agent will open a WSS to `wss://relay.esluce.net/relay/tunnel`, send a `TunnelConnect` on the yamux control stream, and accept inbound yamux streams that get forwarded to `127.0.0.1:25565`.
- Plan 05 (dashboard) can send `ModeOverrideChange` via the existing backend WS path; the agent's existing inbound WS handler will route it through the regular `execute_single` dispatcher.
- The `relay.remove_cname_record` arm is in place for both directions: backend-initiated (dispatch arm) and self-initiated (tunnel disconnect self-loop).

## Verification Results

### Task 1 — Deps, audit, relay entrypoint, dns handle_remove_record

- ✅ `Cargo.toml` has 6 new deps from the plan: `tokio-yamux` (0.3), `futures` (0.3), `base64` (0.22), `hmac` (0.12), `sha2` (0.10), `hex` (0.4). Plus `tokio-util` (0.7) for the `CancellationToken` used in Task 2.
- ✅ `src/audit.rs` has `pub async fn log_relay_tunnel_event(node_id, server_id, event_type, detail)`. Mirrors the existing `log_connectivity_command` pattern.
- ✅ `src/handlers/relay.rs` exists with `pub async fn handle_relay_task(task: &Task) -> Result<serde_json::Value>` matching on `relay.connect | relay.disconnect | relay.heartbeat` and delegating to `relay_client::{connect, disconnect, send_heartbeat}`.
- ✅ `src/handlers/dns.rs` has `pub async fn handle_remove_record(task: Task)` that calls Cloudflare's `DELETE /zones/{zone_id}/dns_records/{record_id}` and treats 404 as success.
- ✅ `cargo check` exits 0.

### Task 2 — Relay client + session + dispatch + bootstrap

- ✅ `src/handlers/relay_client.rs` exists (25 KB) with `pub async fn connect`, `pub async fn disconnect`, `pub async fn send_heartbeat`, and `pub async fn run_relay_client`.
- ✅ `src/handlers/relay_session.rs` exists (6 KB) with `pub async fn run_relay_session<S>(stream: S, local_addr: String, bytes_counter: Arc<AtomicU64>)` that opens a TcpStream to `127.0.0.1:25565`, bidirectionally copies to a yamux stream, and atomically adds the byte counts to `bytes_counter`.
- ✅ `run_relay_client` tracks `tunnel_start: Instant` and `bytes_transferred: Arc<AtomicU64>`; the heartbeat task checks `tunnel_start.elapsed() >= 24h` OR `bytes_transferred >= 100 GiB`; on threshold, calls `control.shutdown()` which cascades to a WS Close.
- ✅ `run_relay_client`'s cleanup path calls `dispatch_remove_cname_record` which invokes `dns::handle_remove_record` with payload `{api_token, zone_id, record_id, subdomain}`.
- ✅ `src/handlers/mod.rs` has 4 new dispatch arms (`relay.connect`, `relay.disconnect`, `relay.heartbeat`, `relay.remove_cname_record`) and 4 new TaskConfig entries with the correct timeout (30/10/5/15 s) and `max_retries: 0`.
- ✅ `src/main.rs::bootstrap_relay_client` reads `AGENT_RELAY_TOKEN` and 8 optional env vars, calls `state::set_relay_config`, and spawns `run_relay_client` in a tokio task.
- ✅ `cargo check` exits 0.

### Plan-Level Verification

- ✅ `cargo check` exits 0 (exit status 0; 16 pre-existing warnings unrelated to this plan)
- ✅ 3 new modules exist: `src/handlers/relay.rs` (1,946 B), `src/handlers/relay_client.rs` (25,187 B), `src/handlers/relay_session.rs` (6,047 B). All > 50 lines.
- ✅ 8 dispatch-arm matches in `mod.rs` (4 in `execute_single`, 4 in `get_task_config`)
- ✅ 1 match for `relay_client::run_relay_client` in `main.rs` (line 455)
- ✅ 7 dep lines in `Cargo.toml` (6 new + `tokio-util` for the `CancellationToken`)

## Self-Check: PASSED

- ✅ `src/handlers/relay.rs` exists
- ✅ `src/handlers/relay_client.rs` exists (25,187 bytes)
- ✅ `src/handlers/relay_session.rs` exists (6,047 bytes)
- ✅ `src/audit.rs` has `log_relay_tunnel_event`
- ✅ `src/handlers/dns.rs` has `handle_remove_record`
- ✅ `src/handlers/mod.rs` has 4 dispatch arms + 4 TaskConfig entries for the new task types
- ✅ `src/main.rs` has `bootstrap_relay_client` that reads `AGENT_RELAY_TOKEN` and spawns `run_relay_client`
- ✅ `src/state.rs` has `RelayConfig` + `set_relay_config` / `relay_config`
- ✅ `cargo check` exits 0
- ✅ Both commits present in git log: `334dd06` (Task 1) + `2aa8b1b` (Task 2)

---

*Phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela*
*Completed: 2026-06-07*
