---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 08
type: execute
gap_closure: true
wave: 1
subsystem: relay-gateway
tags: [yamux, ws-bridge, auth, relay-control-plane, gap-closure]
depends_on:
  - 68-04a
  - 68-02
files_modified:
  - src/state.rs
  - src/main.rs
  - src/handlers/relay_client.rs
  - opt/relay/src/tunnel.rs
autonomous: true
requirements:
  - DEPLOY-01
  - STATUS-01
user_setup: []
source_verification: .planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-VERIFICATION.md
source_plan: .planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04a-PLAN.md

# Out of scope (explicit non-gaps, NOT addressed by this plan)
out_of_scope:
  - "BLOCKER #4 (rate limiter NOT wired) is a VERIFIER FALSE POSITIVE — `state.rate_limiter.check(peer.ip())` is ALREADY wired at opt/relay/src/player.rs:37 (method name is `check`, not `check_rate_limit` as the verifier expected). Per the gap-closure directive, do NOT add a task to address this. The 04a plan at line 438 also uses `check(peer.ip())`. The rate limiter is fully implemented AND wired."
  - "The 6 verified PASS BLOCKERs at 04a-26 (handshake parser, by_subdomain lookup, by_server_id index, etc.) are not touched. Verifier scored them PASS at 22/28 — they're already done."
  - "Phase 68 plans 68-01, 68-02, 68-03, 68-04b, 68-04c, 68-05 are NOT modified (only 04a's territory + a small agent-side token addition)."

must_haves:
  truths:
    - "Agent opens outbound WSS to wss://relay.esluce.net/relay/tunnel and the first yamux stream it opens contains a TunnelConnect JSON with `relay_token` (string UUID) AND `server_id` (UUID) — the gateway now has the (relay_token, server_id) pair it needs to call auth::authorize"
    - "Gateway's tunnel.rs spawns a `ws_bridge` task that pumps bytes between WS Binary messages and a `tokio::io::duplex(64KB)`, and creates a real yamux server session via `Session::new_server(duplex_side, YamuxConfig::default())` — exactly mirroring the agent's pattern at src/handlers/relay_client.rs:331-355"
    - "Gateway reads the TunnelConnect JSON from the first inbound yamux stream (`session.next().await`), then calls `state.backend.authorize(relay_token, server_id).await` (the HMAC-signed POST to /internal/relay/authorize at opt/relay/src/backend.rs:69), and only registers the tunnel on 2xx response — closes the WS on 401/403/502"
    - "Gateway stores the yamux `Control` handle from the server-side session in `TunnelHandle.yamux_control` (the field is already typed `Mutex<Option<Control>>` in opt/relay/src/registry.rs:12), so player.rs's `control_lock.as_ref().open_stream()` no longer hits the `None` arm and drops the connection with 'Tunnel handle has no yamux control (stale)'"
    - "No more 'Expected first message Text, got Binary' warnings: the gateway no longer matches `Message::Text` for the first frame — yamux handles all WS framing, the gateway only reads from the duplex side"
    - "WS frame type compatibility: gateway reads WS Binary (via ws_bridge task), agent writes WS Binary (via ws_bridge task at relay_client.rs:565, 579)"
  artifacts:
    - path: "src/state.rs"
      provides: "RelayConfig.server_id: Uuid field (new), populated from AGENT_RELAY_SERVER_ID env var"
      contains: "pub server_id: Uuid"
    - path: "src/main.rs"
      provides: "bootstrap_relay_client reads AGENT_RELAY_SERVER_ID and populates RelayConfig.server_id"
      contains: "AGENT_RELAY_SERVER_ID"
    - path: "src/handlers/relay_client.rs"
      provides: "TunnelConnect JSON includes `relay_token: cfg.token.clone()` and `server_id: cfg.server_id` — gateway can now call auth::authorize"
      contains: "relay_token.*cfg.token"
    - path: "opt/relay/src/tunnel.rs"
      provides: "Real yamux server session: ws_bridge task + Session::new_server + read TunnelConnect from first inbound yamux stream + auth::authorize call + store Control in TunnelHandle + drive session loop"
      contains: "Session::new_server"
    - path: "opt/relay/src/tunnel.rs"
      provides: "TunnelConnect struct now has `relay_token: Uuid` field (new), parsed from agent's TunnelConnect JSON"
      contains: "pub relay_token: Uuid"
  key_links:
    - from: "opt/relay/src/tunnel.rs"
      to: "opt/relay/src/backend.rs"
      via: "state.backend.authorize(relay_token, server_id) after reading TunnelConnect — HMAC-signed POST to /internal/relay/authorize"
      pattern: "state\\.backend\\.authorize"
    - from: "opt/relay/src/tunnel.rs"
      to: "opt/relay/src/registry.rs"
      via: "TunnelHandle.yamux_control: Mutex<Option<Control>> is now Some(session.control()) — player.rs's open_stream() is reachable"
      pattern: "session\\.control\\(\\)"
    - from: "opt/relay/src/tunnel.rs"
      to: "tokio_yamux::Session"
      via: "Session::new_server over a tokio::io::duplex(64KB) — mirror of agent's Session::new_client at relay_client.rs:337"
      pattern: "Session::new_server"
    - from: "opt/relay/src/tunnel.rs"
      to: "WebSocket"
      via: "ws_bridge task: tokio::io::duplex ↔ WebSocket Binary frames — mirror of agent's ws_bridge at relay_client.rs:545-600"
      pattern: "Message::Binary"
    - from: "src/handlers/relay_client.rs"
      to: "opt/relay/src/tunnel.rs"
      via: "TunnelConnect JSON shape: agent writes `relay_token` + `server_id` fields, gateway parses them into the TunnelConnect struct"
      pattern: "relay_token.*cfg\\.token"

gap_addressed: |
  VERIFICATION.md reports 4 BLOCKERs in opt/relay/src/tunnel.rs and player.rs.
  This plan closes 3 of the 4 (BLOCKER #1 yamux session, BLOCKER #2 WS frame type,
  BLOCKER #3 auth::authorize never called). BLOCKER #4 (rate limiter "not wired")
  is a verifier false positive — rate_limiter.check(peer.ip()) IS already wired at
  opt/relay/src/player.rs:37 and is explicitly out of scope per the gap-closure
  directive.

  BLOCKER #1 (yamux): opt/relay/src/tunnel.rs:100-114 stores `yamux_control:
  tokio::sync::Mutex::new(None)` and the comment admits "We don't have a real
  yamux session because the WebSocket is just a control plane in this MVP; yamux
  streams come over a side-channel." Fix: implement real `Session::new_server`
  over `tokio::io::duplex(64KB)` + ws_bridge task, mirroring the agent's
  pattern at src/handlers/relay_client.rs:331-355. Store `session.control()` in
  the TunnelHandle.

  BLOCKER #2 (WS frame type): opt/relay/src/tunnel.rs:46 and :152 match only
  `Message::Text` for the first frame and for subsequent messages. The agent's
  ws_bridge (relay_client.rs:565, 579) sends `Message::Binary` only (it skips
  Text frames at line 590). The gateway would never see the TunnelConnect JSON.
  Fix: replace the direct `Message::Text` matching with a `ws_bridge` task that
  pumps WS Binary frames into a duplex, and read TunnelConnect from the yamux
  stream (not from a WS Text frame).

  BLOCKER #3 (auth::authorize): opt/relay/src/auth.rs:17-23 implements
  `pub async fn authorize(state, token, server_id)` (HMAC-signed POST to
  backend), but it is never called from tunnel.rs. The comment at tunnel.rs:90
  says "TODO: when the agent adds the bearer token to TunnelConnect, verify
  here." Fix: (a) agent adds `relay_token` and `server_id` to the TunnelConnect
  JSON (relay_client.rs:343-349); (b) gateway parses these from the TunnelConnect
  JSON and calls `state.backend.authorize(relay_token, server_id).await` before
  registering the tunnel.
---

<objective>
Close 3 of the 4 BLOCKERs in opt/relay/src/tunnel.rs and player.rs that the Phase 68 verifier identified (BLOCKER #4 is a verifier false positive — see `out_of_scope` frontmatter). The fix mirrors the agent's existing architecture on the gateway side: real yamux server session over a `tokio::io::duplex(64KB)` with a `ws_bridge` task, plus a small agent-side addition to put `relay_token` and `server_id` in the TunnelConnect JSON so the gateway can call the already-implemented `state.backend.authorize()` (HMAC-signed POST to /internal/relay/authorize) before registering the tunnel.

Purpose: The 22/28 score reflects that the backend, agent, dashboard, Docker, and runbook layers are all verified PASS. The remaining 4 BLOCKERs are concentrated in the gateway's tunnel control plane: the agent and the gateway are using incompatible WS frame types (Binary vs Text), the gateway's yamux server session is a `Mutex::new(None)` stub, and the HMAC-signed `auth::authorize` call is never invoked. The 3 real BLOCKERs collapse into a single 4-file, ~100-line refactor: the gateway adopts the agent's exact `ws_bridge` + `Session::new_*` pattern, and the agent adds 2 fields to its TunnelConnect JSON. After the fix, the player TCP path can actually open yamux streams against the agent (player.rs:88 `control.open_stream()` becomes reachable), and the backend's per-(relay_token, server_id) ownership check actually runs.

Output:
- `src/state.rs` — `RelayConfig` gains `pub server_id: Uuid` field (new)
- `src/main.rs` — `bootstrap_relay_client` reads `AGENT_RELAY_SERVER_ID` env var and populates `RelayConfig.server_id` (defaults to `Uuid::nil()` with a warn log if missing)
- `src/handlers/relay_client.rs` — `connect_msg` JSON includes `relay_token: cfg.token.clone()` and `server_id: cfg.server_id`
- `opt/relay/src/tunnel.rs` — major refactor: spawn `ws_bridge` task, `Session::new_server(duplex_side, yamux_cfg)`, wait for first inbound yamux stream, read TunnelConnect JSON from it, call `state.backend.authorize(relay_token, server_id).await`, store `session.control()` in `TunnelHandle.yamux_control`, drive session loop on the bridge task's lifetime + 10s ticker

Scope: This plan only touches the gateway's tunnel control plane and a small agent-side addition to put the bearer token in the JSON. The handshake parser, by_subdomain/by_server_id routing, rate limiter, metrics, Docker, Caddyfile, dashboard, and the entire backend HMAC stack are NOT modified — they were verified PASS at 22/28 and stay byte-identical.
</objective>

<execution_context>
@/home/rhnbztnl/.config/opencode/get-shit-done/workflows/execute-plan.md
@/home/rhnbztnl/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-VERIFICATION.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04a-PLAN.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04a-SUMMARY.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-02-SUMMARY.md

# Canonical analogs (read these before writing)
@opt/relay/src/tunnel.rs (the file this plan rewrites — 237 lines)
@opt/relay/src/player.rs (lines 78-95 for the broken `control_lock.as_ref()` reachability check)
@opt/relay/src/registry.rs (line 12: `yamux_control: Mutex<Option<Control>>` — the field shape stays; only the inner value changes)
@opt/relay/src/auth.rs (lines 17-23: `pub async fn authorize(state, token, server_id) -> Result<Authorization, GatewayError>`)
@opt/relay/src/backend.rs (lines 69-118: `BackendClient::authorize(relay_token, server_id) -> Result<Authorization, GatewayError>` — the HMAC-signed POST)
@src/handlers/relay_client.rs (lines 331-355: the agent's `connect_and_run` setup with `tokio::io::duplex(64KB)` + `Session::new_client`; lines 545-600: `ws_bridge` function; lines 343-349: the `connect_msg` JSON the gateway now needs to mirror)
@src/state.rs (lines 136-160: `RelayConfig` struct — the field to add is `pub server_id: Uuid` between `pub token: String` and `pub subdomain: String`)
@src/main.rs (lines 393-455: `bootstrap_relay_client` — add the `AGENT_RELAY_SERVER_ID` env read + populate `RelayConfig.server_id`)
</context>

<dependency_graph>
## requires
- **68-04a**: provides `opt/relay/` crate with registry (yamux_control field shape: `Mutex<Option<Control>>` at registry.rs:12), player (Handshake parser + rate_limiter.check wired at line 37), backend (HMAC-signed `BackendClient::authorize` at backend.rs:69-118), auth (the `pub async fn authorize` wrapper at auth.rs:17-23), and the 04a tunnel.rs STUB (Message::Text only, Mutex::new(None) yamux_control). This plan REWRITES tunnel.rs and ADDS the bearer token to the agent's TunnelConnect JSON so the gateway can call the existing `auth::authorize` (which was correctly implemented but never invoked).
- **68-02**: provides the agent's `Session::new_client` + `ws_bridge` pattern (relay_client.rs:331-355, 545-600) that the gateway now mirrors on the server side. Also provides `RelayConfig` (state.rs:136-160) which gains one new field.

## provides
- Real yamux server session in the gateway: `Session::new_server(duplex_side, YamuxConfig::default())` at opt/relay/src/tunnel.rs, with `session.control()` stored in `TunnelHandle.yamux_control`. This makes player.rs's `control.open_stream()` (player.rs:88) reachable — the player TCP forwarder can finally open yamux streams.
- Working `auth::authorize` flow: gateway reads `relay_token` and `server_id` from the agent's TunnelConnect JSON, calls `state.backend.authorize(relay_token, server_id).await` (HMAC-signed POST to backend's `/internal/relay/authorize`), and only registers the tunnel on 2xx response. On 401/403/502, the WS is closed and the tunnel is not registered.
- WS message type compatibility: gateway uses `ws_bridge` task to pump `Message::Binary` frames from the agent into a `tokio::io::duplex`, exactly mirroring the agent's pattern. The gateway's `run_tunnel_session` no longer matches `Message::Text` directly (the `Message::Text` arms at tunnel.rs:46 and :152 are removed).
- No more 'Tunnel handle has no yamux control (stale); closing' player drops: the `None` arm at player.rs:82 becomes unreachable because `TunnelHandle.yamux_control` is always `Some(control)` after this fix.
- Agent side: `RelayConfig.server_id: Uuid` field (read from `AGENT_RELAY_SERVER_ID` env var, defaults to `Uuid::nil()` with a warn log) + `relay_token` and `server_id` fields in the TunnelConnect JSON (so the gateway can authorize).

## consumed_by
- Future phases that deploy the gateway + agent end-to-end: the relay forwarding path is now actually functional.
- The backend's `/internal/relay/authorize` HMAC endpoint (68-03): now actually called on every WSS connect (was implemented but never invoked before this fix).
- The backend's `nodes.relay_token` column (68-01): now used as the bearer token in the TunnelConnect JSON (was just sent in the WS upgrade header before — still sent there too for double-coverage).

## wave
- Wave 1 (no other gap-closure plans; this is the only gap-closure for Phase 68).
</dependency_graph>

<tech_tracking>
- No new dependencies: `tokio`, `tokio-yamux`, `tokio-tungstenite`, `futures`, `serde`, `serde_json`, `uuid`, `tracing` are all already in `opt/relay/Cargo.toml` and the root `Cargo.toml`.
- No migration changes. No container or bootstrap changes (the Dockerfile, Caddyfile, and docker-compose at opt/relay/{Dockerfile,Caddy.Dockerfile,Caddyfile,docker-compose.yml} are untouched — they were verified PASS at 22/28).
- No new tests added (this is a control-plane wire-up; the existing 04a behavioral spot-checks at VERIFICATION.md:181-191 already cover the `cargo check` invariant, and the existing 22/28 score covers the static-analysis checks).
- Touches 4 files: `src/state.rs` (+1 field, ~3 lines), `src/main.rs` (+5 lines env read + populate), `src/handlers/relay_client.rs` (+2 fields in JSON, ~3 lines), `opt/relay/src/tunnel.rs` (major refactor, ~80 lines rewritten).
- New helper function in tunnel.rs: `ws_bridge` (mirror of agent's at relay_client.rs:545-600, ~55 lines), `read_json_message` (helper to read variable-length JSON from a yamux StreamHandle, ~20 lines).
- The `TunnelHandle` struct in `opt/relay/src/registry.rs` is NOT touched — the `yamux_control: Mutex<Option<Control>>` field is already correctly typed.
- The `TunnelConnect` struct in `opt/relay/src/tunnel.rs` gains 1 new field: `pub relay_token: Uuid` (parsed from the agent's TunnelConnect JSON, passed to `state.backend.authorize`).
</tech_tracking>

<tasks>

<task type="auto">
  <name>Task 1: Agent adds `relay_token` and `server_id` to TunnelConnect JSON</name>
  <files>src/state.rs, src/main.rs, src/handlers/relay_client.rs</files>
  <read_first>
    - src/state.rs (lines 136-160: the `RelayConfig` struct — add `pub server_id: Uuid` between `pub token: String` and `pub subdomain: String`)
    - src/main.rs (lines 393-455: `bootstrap_relay_client` — add the `AGENT_RELAY_SERVER_ID` env read after the `AGENT_RELAY_REGION` block at line 414)
    - src/handlers/relay_client.rs (lines 343-349: the `connect_msg` JSON the gateway now needs to mirror on the server side)
  </read_first>
  <action>
  **Part A — `src/state.rs`**: add `server_id: Uuid` field to `RelayConfig` (between `pub token: String` and `pub subdomain: String` at line 142-144). The field is `pub server_id: Uuid` with a doc comment: "Per-server UUID the gateway uses for the auth::authorize (relay_token, server_id) HMAC pair. Read from `AGENT_RELAY_SERVER_ID` env var at bootstrap; defaults to `Uuid::nil()` with a warn log if missing (the gateway's authorize call will then 403, which is the correct fail-closed behavior)."

  **Part B — `src/main.rs`**: in `bootstrap_relay_client` (lines 393-455), after the `AGENT_RELAY_REGION` block at lines 413-414 and BEFORE the `AGENT_RELAY_LOCAL_ADDR` block at line 415, add:
  ```rust
  let server_id = std::env::var("AGENT_RELAY_SERVER_ID")
      .ok()
      .and_then(|s| Uuid::parse_str(&s).ok())
      .unwrap_or_else(|| {
          tracing::warn!(
              "[RELAY] AGENT_RELAY_SERVER_ID not set or invalid; using Uuid::nil(). \
               Gateway's auth::authorize will return 403 until this is set."
          );
          Uuid::nil()
      });
  ```
  Then in the `RelayConfig { ... }` construction at line 430-442, add the field between `token: token.clone()` and `subdomain`:
  ```rust
  let relay_cfg = state::RelayConfig {
      gateway_url,
      token: token.clone(),
      server_id,
      subdomain,
      public_port,
      agent_public_ip,
      region,
      local_mc_addr,
      dns_api_token,
      dns_zone_id,
      dns_record_id,
  };
  ```

  **Part C — `src/handlers/relay_client.rs`**: in `connect_and_run` at lines 343-354, replace the `connect_msg` JSON with one that includes `relay_token` and `server_id`:
  ```rust
  let connect_msg = json!({
      "type": "tunnel_connect",
      "relay_token": cfg.token,
      "server_id": cfg.server_id,
      "subdomain": cfg.subdomain,
      "public_port": cfg.public_port,
      "agent_public_ip": cfg.agent_public_ip,
      "region": cfg.region,
  });
  ```
  Note: `cfg.token: String` is the string form of the relay_token UUID. The gateway's `TunnelConnect.relay_token: Uuid` field (added in Task 2) parses the string back to a UUID. The `Uuid::nil()` default in Part B means a missing env var results in a JSON `server_id: "00000000-0000-0000-0000-000000000000"` — the backend's `find_by_relay_token` will then return 403, which is the correct fail-closed behavior.

  Do NOT change `cfg.token.clone()` semantics: the existing `cfg.token` is moved into the `Bearer` header at `build_ws_request(uri, &cfg.token)` (line 315). Since `json!` consumes `cfg.token` by value, you may need `cfg.token.clone()` instead of `cfg.token`. Inspect the call site at line 315 and clone if needed.

  </action>
  <verify>
    <automated>grep -n "pub server_id: Uuid" src/state.rs && echo "---" && grep -n "AGENT_RELAY_SERVER_ID" src/main.rs && echo "---" && grep -n "relay_token.*cfg\.token\|relay_token: cfg" src/handlers/relay_client.rs && echo "---" && grep -n '"server_id": cfg.server_id' src/handlers/relay_client.rs && echo "---" && cargo check 2>&1 | tail -10</automated>
  </verify>
  <done>
  - `src/state.rs` has `pub server_id: Uuid` field in `RelayConfig` with the doc comment above
  - `src/main.rs` reads `AGENT_RELAY_SERVER_ID`, parses as Uuid, defaults to `Uuid::nil()` with warn log if missing
  - `src/handlers/relay_client.rs:343-349` TunnelConnect JSON includes `relay_token: cfg.token.clone()` and `server_id: cfg.server_id`
  - `cargo check` exits 0 with no new errors and no new warnings in the 3 modified files
  </done>
</task>

<task type="auto">
  <name>Task 2: Gateway implements real yamux server session in tunnel.rs (ws_bridge + Session::new_server + read TunnelConnect from yamux stream + store Control)</name>
  <files>opt/relay/src/tunnel.rs</files>
  <read_first>
    - opt/relay/src/tunnel.rs (lines 1-188: the function to rewrite — current `Message::Text` matching at :46 and :152, `Mutex::new(None)` yamux_control at :110)
    - opt/relay/src/registry.rs (line 12: `yamux_control: tokio::sync::Mutex<Option<Control>>` — the field shape stays; only the inner value changes from `None` to `Some(control)`)
    - src/handlers/relay_client.rs (lines 331-355: the agent's `connect_and_run` setup with `tokio::io::duplex(BRIDGE_BUFFER_BYTES)` + `Session::new_client` — the pattern to mirror)
    - src/handlers/relay_client.rs (lines 545-600: the agent's `ws_bridge` function — copy this exact pattern into tunnel.rs as a private helper)
  </read_first>
  <action>
  Rewrite `run_tunnel_session` in `opt/relay/src/tunnel.rs` to mirror the agent's architecture. The new flow:

  **Imports** (replace the existing import block at lines 1-11):
  ```rust
  use std::sync::Arc;
  use std::time::Duration;

  use axum::extract::ws::{Message, WebSocket};
  use futures::{SinkExt, StreamExt};
  use serde::{Deserialize, Serialize};
  use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
  use tokio_yamux::{Config as YamuxConfig, Session, StreamHandle};
  use tracing::{error, info, warn};
  use uuid::Uuid;

  use crate::registry::TunnelHandle;
  use crate::state::AppState;

  /// Buffer size for the WS ↔ duplex bridge. 64 KiB matches yamux's default window size
  /// (mirror of BRIDGE_BUFFER_BYTES in src/handlers/relay_client.rs:102).
  const BRIDGE_BUFFER_BYTES: usize = 64 * 1024;
  ```

  **TunnelConnect struct** (replace lines 14-23) — add the `relay_token: Uuid` field:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TunnelConnect {
      #[serde(rename = "type")]
      pub msg_type: String,
      pub relay_token: Uuid,             // NEW: bearer token for auth::authorize
      pub server_id: Uuid,               // already existed; now actually populated by the agent
      pub subdomain: String,
      pub public_port: u16,
      pub agent_public_ip: String,
      pub region: String,
  }
  ```

  **`run_tunnel_session`** (rewrite lines 41-188). The new function:

  1. **Set up duplex + ws_bridge** (mirror relay_client.rs:331-333):
     ```rust
     let (yamux_side, ws_byte_side) = tokio::io::duplex(BRIDGE_BUFFER_BYTES);
     let bridge_handle = tokio::spawn(ws_bridge(socket, ws_byte_side));
     ```

  2. **Create yamux server session** (mirror relay_client.rs:336-337):
     ```rust
     let yamux_cfg = YamuxConfig::default();
     let mut session = Session::new_server(yamux_side, yamux_cfg);
     ```

  3. **Wait for the first inbound yamux stream** (the agent's control stream — relay_client.rs:421 `session.next()`):
     ```rust
     let mut control_stream: StreamHandle = match session.next().await {
         Some(Ok(s)) => s,
         Some(Err(e)) => { warn!("[TUNNEL] yamux session error: {}", e); return; }
         None => { warn!("[TUNNEL] yamux session ended before first stream"); return; }
     };
     ```

  4. **Read TunnelConnect JSON from the control stream** (use a small helper, see below):
     ```rust
     let connect: TunnelConnect = match read_json_message(&mut control_stream).await {
         Ok(bytes) => match serde_json::from_slice(&bytes) {
             Ok(c) => c,
             Err(e) => { warn!("[TUNNEL] Invalid TunnelConnect JSON: {}", e); return; }
         },
         Err(e) => { warn!("[TUNNEL] Failed to read TunnelConnect: {}", e); return; }
     };
     if connect.msg_type != "tunnel_connect" {
         warn!("[TUNNEL] First message was not tunnel_connect: {}", connect.msg_type);
         return;
     }
     info!(
         "[TUNNEL] TunnelConnect: server_id={}, subdomain={}, agent_ip={}",
         connect.server_id, connect.subdomain, connect.agent_public_ip
     );
     ```

  5. **Validate subdomain** (the existing `validate_subdomain` at lines 226-236):
     ```rust
     if let Err(e) = validate_subdomain(&connect.subdomain) {
         warn!("[TUNNEL] Invalid subdomain '{}': {}", connect.subdomain, e);
         return;
     }
     ```

  6. **Task 3 handles this step — leave a `// (auth::authorize call added in Task 3)` marker here for now.** The plan can be implemented as a single atomic commit, but the structure is clearer with the marker.

  7. **Build the TunnelHandle with the real `Control`** (replace the `Mutex::new(None)` at line 110):
     ```rust
     let control = session.control();  // tokio_yamux::Control: Clone + Send + Sync
     let handle = Arc::new(TunnelHandle {
         server_id: connect.server_id,
         subdomain: connect.subdomain.clone(),
         agent_public_ip: connect.agent_public_ip.clone(),
         last_heartbeat: std::sync::atomic::AtomicU64::new(
             std::time::SystemTime::now()
                 .duration_since(std::time::UNIX_EPOCH)
                 .unwrap_or_default()
                 .as_secs(),
         ),
         yamux_control: tokio::sync::Mutex::new(Some(control)),  // CHANGED from None
         started_at: std::time::Instant::now(),
         bytes_in: std::sync::atomic::AtomicU64::new(0),
         bytes_out: std::sync::atomic::AtomicU64::new(0),
     });
     ```

  8. **Register and report connected** (unchanged from current lines 116-132):
     ```rust
     if let Err(e) = state.registry.register(handle.clone()) {
         warn!("[TUNNEL] Registry::register failed: {}", e);
         return;
     }
     if let Err(e) = state.backend.report_tunnel_event(handle.server_id, "connected", "tunnel_established").await {
         warn!("[TUNNEL] Failed to report connected event: {}", e);
     }
     crate::metrics::ACTIVE_TUNNELS.inc();
     crate::metrics::TUNNEL_EVENTS_TOTAL.with_label_values(&["connected"]).inc();
     ```

  9. **Spawn a task to read heartbeats/disconnects from the control stream** (NEW — the agent's heartbeat design is write-only on its end, so the gateway needs a reader to update `last_heartbeat` and detect `TunnelDisconnect`):
     ```rust
     let hb_state = state.clone();
     let hb_handle = handle.clone();
     let mut control_for_reader = ...; // can't move control_stream while we hold it; use a different approach
     // Actually, we need to KEEP control_stream alive but don't actively use it here.
     // Spawn a task that owns control_stream and reads from it.
     let hb_task = tokio::spawn(async move {
         read_control_stream(hb_state, hb_handle, control_stream).await;
     });
     ```
     Wait — we need to split or move the `control_stream`. The cleanest approach: move the entire `StreamHandle` into the spawned task. The main loop just relies on the 10s ticker and the bridge task for liveness. The `last_heartbeat` field gets updated by the read task.

     Revised (correct ownership):
     ```rust
     // Spawn the control-stream reader task that owns the StreamHandle.
     let hb_state = state.clone();
     let hb_handle = handle.clone();
     let hb_task = tokio::spawn(async move {
         read_control_stream(hb_state, hb_handle, control_stream).await;
     });
     ```

  10. **10s ticker for backend liveness reports + bridge-end detection** (the loop is now smaller because heartbeats are read in a separate task):
      ```rust
      let mut ticker = tokio::time::interval(Duration::from_secs(state.config.tunnel.heartbeat_interval_secs));
      ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
      loop {
          tokio::select! {
              _ = ticker.tick() => {
                  if let Err(e) = state.backend.report_tunnel_event(handle.server_id, "heartbeat", "ok").await {
                      warn!("[TUNNEL] Heartbeat backend report failed: {}", e);
                  }
              }
              _ = &mut bridge_handle => {
                  // The WS bridge returned (agent disconnected or session died).
                  info!("[TUNNEL] WS bridge ended: server_id={}", handle.server_id);
                  break;
              }
          }
      }
      ```

  11. **Cleanup** (unchanged from current lines 174-187, plus abort the hb_task):
      ```rust
      hb_task.abort();
      let _ = hb_task.await;
      bridge_handle.abort();
      let _ = bridge_handle.await;
      state.registry.unregister(&handle.server_id);
      crate::metrics::ACTIVE_TUNNELS.dec();
      crate::metrics::TUNNEL_EVENTS_TOTAL.with_label_values(&["disconnected"]).inc();
      if let Err(e) = state.backend.report_tunnel_event(handle.server_id, "disconnected", "ws_closed").await {
          warn!("[TUNNEL] Failed to report disconnected event: {}", e);
      }
      crate::session_log::log_session_end(handle.server_id, 0, 0);
      ```

  **New private helpers** (added to the end of tunnel.rs):

  - `async fn ws_bridge<S>(ws: WebSocket, mut yamux_side: DuplexStream)`: copy of the agent's `ws_bridge` at relay_client.rs:545-600, with these adjustments:
    - The `ws` parameter type is `axum::extract::ws::WebSocket` (not `tokio_tungstenite::WebSocketStream<S>`) — use `ws.split()` to get `(ws_sink, ws_stream)`.
    - Same `BRIDGE_BUFFER_BYTES` buffer, same `Message::Binary` write, same `Message::Text / Ping / Pong` skip.
    - The agent's version takes the WS as `tokio_tungstenite::WebSocketStream<S>`; the gateway's axum WebSocket has a different `split()` signature. Use `futures::StreamExt::split` on the axum WebSocket.
    - The body is ~50 lines.

  - `async fn read_control_stream(state: Arc<AppState>, handle: Arc<TunnelHandle>, mut stream: StreamHandle)`: reads JSON messages from the agent's control stream in a loop, parses as `TunnelMessage`, dispatches to the existing `handle_tunnel_message` helper at lines 190-224 (which is unchanged), and breaks on `TunnelDisconnect` or stream EOF. Body is ~20 lines.

  - `async fn read_json_message(stream: &mut StreamHandle) -> Result<Vec<u8>, std::io::Error>`: reads a single JSON message from a yamux stream. yamux framing is by stream, not by message, so a "message" is whatever the agent wrote in one `write_all` call. Since the agent writes the full JSON in one `control.write_all(&connect_bytes)` (relay_client.rs:351-353), the gateway can read up to a reasonable max (e.g., 64 KiB) and parse it. If the buffer fills before the JSON ends, treat that as an error. Body is ~15 lines.

  **Preserve unchanged**:
  - `TunnelMessage` enum (lines 33-39) — keep as-is.
  - `TunnelHeartbeat` struct (lines 25-31) — keep as-is.
  - `handle_tunnel_message` (lines 190-224) — keep as-is. The control-stream reader task in the new design calls this on each parsed `TunnelMessage`.
  - `validate_subdomain` (lines 226-236) — keep as-is.
  - The `Message::Text` arms at lines 46 and 152 — DELETE them; the gateway no longer matches `Message::Text` directly. yamux framing handles all bytes.

  </action>
  <verify>
    <automated>grep -n "Session::new_server" opt/relay/src/tunnel.rs && echo "---" && grep -n "Message::Binary" opt/relay/src/tunnel.rs && echo "---" && grep -n "session.control()" opt/relay/src/tunnel.rs && echo "---" && grep -n "Mutex::new(None)" opt/relay/src/tunnel.rs && echo "(should be 0)" && echo "---" && grep -n "Message::Text" opt/relay/src/tunnel.rs && echo "(should be 0)" && echo "---" && cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay && cargo check 2>&1 | tail -15</automated>
  </verify>
  <done>
  - `opt/relay/src/tunnel.rs` has exactly 1 match for `Session::new_server` (line ~70 of the rewritten file)
  - `opt/relay/src/tunnel.rs` has ≥1 match for `Message::Binary` (inside the new `ws_bridge` helper that writes to the WS)
  - `opt/relay/src/tunnel.rs` has 1 match for `session.control()` (the Control is stored in TunnelHandle)
  - `opt/relay/src/tunnel.rs` has 0 matches for `Mutex::new(None)` (the yamux_control stub is replaced with `Some(control)`)
  - `opt/relay/src/tunnel.rs` has 0 matches for `Message::Text` (the gateway no longer reads Text frames directly; yamux handles all framing)
  - `TunnelConnect` struct has the new `pub relay_token: Uuid` field
  - 3 new private helpers exist: `ws_bridge`, `read_control_stream`, `read_json_message`
  - `cd opt/relay && cargo check` exits 0 with no new errors and no new warnings in tunnel.rs
  - `handle_tunnel_message` (lines 190-224) and `validate_subdomain` (lines 226-236) are byte-identical to their pre-rewrite state
  </done>
</task>

<task type="auto">
  <name>Task 3: Gateway calls `state.backend.authorize` after reading TunnelConnect</name>
  <files>opt/relay/src/tunnel.rs</files>
  <read_first>
    - opt/relay/src/tunnel.rs (the rewritten file from Task 2; find the marker at step 6)
    - opt/relay/src/backend.rs (lines 69-118: the `BackendClient::authorize(relay_token, server_id) -> Result<Authorization, GatewayError>` signature)
    - opt/relay/src/auth.rs (lines 17-23: the `pub async fn authorize(state, token, server_id) -> Result<Authorization, GatewayError>` wrapper — same signature)
  </read_first>
  <action>
  In the rewritten `opt/relay/src/tunnel.rs`, at the marker left by Task 2 (step 6, between subdomain validation and TunnelHandle construction), insert the `auth::authorize` call. This is the BLOCKER #3 fix — the HMAC-signed POST to the backend's `/internal/relay/authorize` endpoint.

  Replace the `// (auth::authorize call added in Task 3)` marker with:
  ```rust
  // Authorize the (relay_token, server_id) pair against the backend.
  // The backend's /internal/relay/authorize endpoint verifies ownership
  // (the node that owns this relay_token also owns this server_id) and
  // returns 200 on success, 401/403 on auth failure, 502 on backend
  // unreachable. The HMAC is signed by `state.backend` (T-68-17).
  if let Err(e) = crate::auth::authorize(&state, &connect.relay_token, &connect.server_id).await {
      warn!(
          "[TUNNEL] auth::authorize failed for server_id={}, token={}: {}; closing WS",
          connect.server_id, connect.relay_token, e
      );
      // No tunnel is registered on auth failure — clean close.
      // The bridge task is aborted implicitly when this function returns
      // and `socket` is dropped (axum sends a WS Close frame on drop).
      return;
  }
  ```

  This:
  - Calls `crate::auth::authorize(&state, &connect.relay_token, &connect.server_id)` (the wrapper at `opt/relay/src/auth.rs:17-23` that calls `state.backend.authorize(...)`)
  - On `Err(GatewayError::Auth)` (401/403 from backend): logs and returns, no tunnel registered, no metrics incremented
  - On `Err(GatewayError::BackendUnreachable(_))` (502 from backend or network error): logs and returns, same fail-closed behavior
  - On `Ok(Authorization { node_id, user_id })`: proceeds to construct the TunnelHandle (Task 2's step 7) and register
  - Does NOT log the `Authorization { node_id, user_id }` contents (the `Authorization` struct at auth.rs:7-11 contains `Uuid` values; logging them is fine for audit but the existing code doesn't)

  Note: the original 04a plan (line 670) said "calls `auth::authorize` with the bearer token" — it implied the token comes from a field. The current design puts both `relay_token` and `server_id` in the TunnelConnect JSON (added in Task 1), so we read them from there. The Bearer token in the WS upgrade header is redundant (sent by the agent at relay_client.rs:609-615) and is not used by the gateway (it was a defense-in-depth measure in 04a that's now subsumed by the JSON path).

  No other changes — the rest of `run_tunnel_session` (handle construction, registration, metrics, session drive loop, cleanup) is unchanged from Task 2.
  </action>
  <verify>
    <automated>grep -n "auth::authorize\|crate::auth::authorize\|state\.backend\.authorize" opt/relay/src/tunnel.rs && echo "---" && grep -n "connect.relay_token" opt/relay/src/tunnel.rs && echo "---" && grep -n "connect.server_id" opt/relay/src/tunnel.rs && echo "---" && cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay && cargo check 2>&1 | tail -10</automated>
  </verify>
  <done>
  - `opt/relay/src/tunnel.rs` has exactly 1 call to `crate::auth::authorize` (or `state.backend.authorize`) between the subdomain validation and the TunnelHandle construction
  - The call passes `&connect.relay_token` and `&connect.server_id` (read from the TunnelConnect JSON in Task 1)
  - On `Err(...)` (auth failure or backend unreachable), the function returns early without registering the tunnel or incrementing metrics
  - On `Ok(...)`, the function proceeds to build the TunnelHandle and register (Task 2's step 7+)
  - `cd opt/relay && cargo check` exits 0 with no new errors and no new warnings
  - The `Authorization { node_id, user_id }` returned by `auth::authorize` is NOT used in this version (the 04a plan mentioned recording it for audit, but the existing code in 04a didn't either, and the gap-closure scope is minimal)
  </done>
</task>

<task type="auto">
  <name>Task 4: Verification — verifier re-check should show 28/28 (was 22/28)</name>
  <files>(verification only — no source file modifications)</files>
  <read_first>
    - opt/relay/src/tunnel.rs (the rewritten file from Tasks 2-3)
    - src/handlers/relay_client.rs (the modified connect_msg JSON from Task 1)
    - .planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-VERIFICATION.md (the gap source of truth; the checks below mirror the verifier's grep/cargo check pattern)
  </read_first>
  <action>
  Run the full set of automated checks below. These are the same checks the verifier uses to score the must-haves; if all 13 checks pass, the 3 BLOCKERs are closed and the verifier's next run should score 28/28 (instead of the current 22/28; BLOCKER #4 was a false positive so it never counted as a real gap).

  **BLOCKER #1 closure — real yamux server session**:
  - `grep -n "Session::new_server" opt/relay/src/tunnel.rs` returns ≥1 match (the new `Session::new_server(duplex_side, yamux_cfg)` call)
  - `grep -n "session.control()" opt/relay/src/tunnel.rs` returns ≥1 match (the Control is stored in TunnelHandle)
  - `grep -n "Mutex::new(None)" opt/relay/src/tunnel.rs` returns 0 matches (the stub is gone)

  **BLOCKER #2 closure — WS message type compatibility**:
  - `grep -n "Message::Binary" opt/relay/src/tunnel.rs` returns ≥1 match (the new `ws_bridge` helper writes Binary frames)
  - `grep -n "Message::Text" opt/relay/src/tunnel.rs` returns 0 matches (the gateway no longer reads Text frames directly; yamux handles all framing)
  - `grep -n "ws_bridge" opt/relay/src/tunnel.rs` returns ≥1 match (the new private helper exists)

  **BLOCKER #3 closure — auth::authorize is called**:
  - `grep -nE "auth::authorize|crate::auth::authorize|state\.backend\.authorize" opt/relay/src/tunnel.rs` returns ≥1 match (the new call after reading TunnelConnect)
  - `grep -n "relay_token" src/handlers/relay_client.rs` returns ≥1 match (the new field in the TunnelConnect JSON, between `type` and `server_id`)
  - `grep -n "pub server_id: Uuid" src/state.rs` returns 1 match (the new RelayConfig field)
  - `grep -n "AGENT_RELAY_SERVER_ID" src/main.rs` returns 1 match (the new env var read)

  **Compile checks**:
  - `cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay && cargo check 2>&1 | tail -10` exits 0 with no errors (warnings are OK; the existing 17 warnings stay)
  - `cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && cargo check 2>&1 | tail -10` exits 0 with no errors (the parent agent + workspace compiles; warnings are OK; the existing 16 warnings stay)

  **Out-of-scope confirmation (BLOCKER #4 is a verifier false positive, NOT a real gap)**:
  - `grep -n "rate_limiter.check" opt/relay/src/player.rs` returns 1 match (line 37 — the rate limiter IS already wired). This is the verifier's BLOCKER #4; per the gap-closure directive it is out of scope and should NOT be re-addressed.

  If any of the 13 checks fail, the executor must fix the issue and re-run before marking Task 4 complete. The fix should be surgical (don't expand the scope).
  </action>
  <verify>
    <automated>echo "=== BLOCKER #1 (yamux session) ===" && grep -n "Session::new_server" opt/relay/src/tunnel.rs && grep -n "session.control()" opt/relay/src/tunnel.rs && (test $(grep -c "Mutex::new(None)" opt/relay/src/tunnel.rs) -eq 0 && echo "OK: no Mutex::new(None)") && echo "=== BLOCKER #2 (WS frame type) ===" && grep -n "Message::Binary" opt/relay/src/tunnel.rs && (test $(grep -c "Message::Text" opt/relay/src/tunnel.rs) -eq 0 && echo "OK: no Message::Text") && grep -n "ws_bridge" opt/relay/src/tunnel.rs && echo "=== BLOCKER #3 (auth::authorize) ===" && (grep -nE "auth::authorize|crate::auth::authorize|state\.backend\.authorize" opt/relay/src/tunnel.rs | head -3) && grep -n "relay_token" src/handlers/relay_client.rs && grep -n "pub server_id: Uuid" src/state.rs && grep -n "AGENT_RELAY_SERVER_ID" src/main.rs && echo "=== Compile checks ===" && cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay && cargo check 2>&1 | tail -5 && cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && cargo check 2>&1 | tail -5 && echo "=== BLOCKER #4 (false positive) ===" && grep -n "rate_limiter.check" opt/relay/src/player.rs && echo "(rate limiter IS wired at player.rs:37 — false positive, out of scope)"</automated>
  </verify>
  <done>
  - All 13 automated checks pass:
    1. `Session::new_server` in tunnel.rs ≥1 match
    2. `session.control()` in tunnel.rs ≥1 match
    3. `Mutex::new(None)` in tunnel.rs = 0 matches
    4. `Message::Binary` in tunnel.rs ≥1 match
    5. `Message::Text` in tunnel.rs = 0 matches
    6. `ws_bridge` in tunnel.rs ≥1 match
    7. `auth::authorize` (or `state.backend.authorize`) in tunnel.rs ≥1 match
    8. `relay_token` in src/handlers/relay_client.rs ≥1 match
    9. `pub server_id: Uuid` in src/state.rs = 1 match
    10. `AGENT_RELAY_SERVER_ID` in src/main.rs = 1 match
    11. `cd opt/relay && cargo check` exits 0
    12. `cd <root> && cargo check` exits 0
    13. `rate_limiter.check` in opt/relay/src/player.rs ≥1 match (verifying BLOCKER #4 false positive — method name is `check`, not `check_rate_limit` as the verifier expected)
  - 3 BLOCKERs closed: BLOCKER #1 (real yamux session), BLOCKER #2 (WS frame type), BLOCKER #3 (auth::authorize called)
  - BLOCKER #4 explicitly noted as false positive (rate limiter already wired at player.rs:37)
  - Verifier re-check should score 28/28 (was 22/28)
  - No new dependencies added; no registry shape change; no tests added
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Agent → Gateway WSS | Agent opens outbound WSS to `relay.esluce.net:443` (Caddy TLS 1.3 terminated, then proxied to gateway :8080). The agent sends `Authorization: Bearer {relay_token}` in the WS upgrade header AND `relay_token` + `server_id` fields in the TunnelConnect JSON. The gateway now reads the JSON path (the bearer header is redundant defense-in-depth) and calls `state.backend.authorize(relay_token, server_id)` — a HMAC-signed POST to backend's `/internal/relay/authorize`. |
| Player → Gateway TCP | Player opens raw TCP to `<subdomain>.play.esluce.net:25565` (NLB passthrough to gateway). Gateway parses the Minecraft Handshake to extract the subdomain and looks up `server_id` in the `by_subdomain` map. The player's source IP is NOT used for routing (BLOCKER 1 from 04a). |
| Gateway → Backend | Internal HMAC-signed POSTs to `/internal/relay/authorize` (now actually called on every tunnel establish) and `/internal/relay/tunnel-event` (already wired). |
| Gateway → Redis | Nonce dedup (NOT rate-limit coordination; rate-limit is in-process per D-20 RESOLVED). |

## STRIDE Threat Register

The 11 threats from 68-04a-PLAN.md (T-68-17 through T-68-27) are unchanged. The gap-closure STRENGTHENS the existing mitigations:

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-68-17 | Spoofing | Bearer token on WSS | **mitigate (STRENGTHENED)** | Token is a 122-bit UUIDv4 verified by HMAC-signed backend callback. The callback (`state.backend.authorize`) was implemented but never called; after this fix it runs on every tunnel establish. The token now flows through TWO independent channels: (a) the WS upgrade `Authorization: Bearer` header (defense-in-depth, not validated by gateway code), and (b) the TunnelConnect JSON's `relay_token` field (the path `auth::authorize` reads). |
| T-68-04 | Replay | Tunnel establish | **mitigate (NEW)** | `state.backend.authorize` signs the request with `Hmac<Sha256>` over `(method, path, body, timestamp, nonce)`. The backend's `verify_hmac` (68-03, internal_relay_handlers.rs:100) checks the timestamp + nonce; replays older than the nonce TTL are rejected. With this fix, the nonce + timestamp check now actually runs on every tunnel establish (was dormant before). |
| T-68-05 | Spoofing | TunnelConnect JSON | mitigate | The agent signs the WS upgrade with `Authorization: Bearer {relay_token}` and the same token appears in the TunnelConnect JSON. A malicious actor that could read the WS body but not the upgrade headers (i.e., a MITM with TLS 1.3 termination capability) would have to forge the HMAC, which requires the GATEWAY_HMAC_SECRET. |
| T-68-19 | Tampering | WSS + yamux bytes | mitigate | All agent ↔ gateway bytes are now inside TLS 1.3 (Caddy) + yamux session (server side). The `ws_bridge` task preserves the byte ordering of `Message::Binary` frames and feeds them into the yamux session; yamux's stream multiplexing ensures ordering per-stream. |
| T-68-G01-01 | Information Disclosure | Tunnel handle reuse | mitigate | A 2nd `TunnelConnect` for the same `server_id` triggers `Registry::register`'s D-21 enforcement (drops the older tunnel). The older tunnel's `ws_bridge` and `read_control_stream` tasks are aborted during the `run_tunnel_session` cleanup. |
| T-68-G01-02 | Denial of Service | Agent-side auth failure storm | mitigate | On `auth::authorize` 401/403, the gateway closes the WS without registering; the agent's `connect_async_tls_with_config` returns a TLS handshake error, the reconnect loop's exponential backoff (1s → 30s) kicks in. Persistent auth failures don't cause an outbound flood. |
| T-68-G01-03 | Tampering | TunnelConnect JSON field types | mitigate | `serde_json::from_slice` rejects malformed JSON; `Uuid::parse_str` is not needed (the gateway's `relay_token: Uuid` field uses serde's UUID deserializer which validates format). Invalid UUIDs produce a deserialization error → `Err` → function returns → WS closes. |

No new trust boundary is introduced. The existing 04a STRIDE mitigations (T-68-17 through T-68-27) are byte-identical. The gap-closure only changes WHO reads what, not the security model.

## ASVS L1 Mappings (Phase 68 gateway tier only)

- **V1.4 Access Control:** Bearer token required; HMAC-signed backend authorization now actually runs on every connect.
- **V2.1 Authentication:** HMAC-SHA256 with 32-byte secret from AWS Secrets Manager.
- **V3.7 Session Management:** yamux sessions are per-connection; tunnel is single-use per `server_id`.
- **V4.1 Input Validation:** TunnelConnect JSON is validated by serde; UUID fields are validated by the UUID deserializer; subdomain is validated by the existing `validate_subdomain`.
- **V6.2 Cryptographic Practices:** TLS 1.3 enforced at Caddy; yamux over WSS.
- **V6.4 Secret Management:** `GATEWAY_HMAC_SECRET` injected from Secrets Manager, never logged.
- **V9.1 Rate Limiting:** 100 req/min per source IP at the player TCP layer (in-process; already wired at player.rs:37).
- **V11.1 Data Classification:** No PII processed; only UUIDs, subdomains, and tunnel lifecycle events.
</threat_model>

<verification>
After all 4 tasks complete, run the consolidated check (same as Task 4's verify block):

```bash
# BLOCKER #1 closure
grep -n "Session::new_server" opt/relay/src/tunnel.rs       # ≥1
grep -n "session.control()" opt/relay/src/tunnel.rs          # ≥1
test $(grep -c "Mutex::new(None)" opt/relay/src/tunnel.rs) -eq 0   # 0

# BLOCKER #2 closure
grep -n "Message::Binary" opt/relay/src/tunnel.rs           # ≥1
test $(grep -c "Message::Text" opt/relay/src/tunnel.rs) -eq 0      # 0
grep -n "ws_bridge" opt/relay/src/tunnel.rs                  # ≥1

# BLOCKER #3 closure
grep -nE "auth::authorize|crate::auth::authorize|state\.backend\.authorize" opt/relay/src/tunnel.rs   # ≥1
grep -n "relay_token" src/handlers/relay_client.rs           # ≥1 (in connect_msg JSON)
grep -n "pub server_id: Uuid" src/state.rs                   # 1 (new RelayConfig field)
grep -n "AGENT_RELAY_SERVER_ID" src/main.rs                  # 1 (new env var)

# Compile checks
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay && cargo check 2>&1 | tail -5   # exit 0
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && cargo check 2>&1 | tail -5             # exit 0

# BLOCKER #4 (false positive confirmation)
grep -n "rate_limiter.check" opt/relay/src/player.rs          # ≥1 (line 37 — already wired)
```

End-to-end behavior (real WS handshake + yamux session + auth::authorize) is NOT verifiable in this plan — it requires the agent + backend + a real Minecraft client. This plan verifies the artifacts compile, the architectural pieces are in place, and the static-analysis checks pass.
</verification>

<success_criteria>
1. `src/state.rs` has `pub server_id: Uuid` field in `RelayConfig` (between `pub token: String` and `pub subdomain: String`).
2. `src/main.rs` reads `AGENT_RELAY_SERVER_ID` env var, parses as `Uuid`, defaults to `Uuid::nil()` with a warn log if missing.
3. `src/handlers/relay_client.rs:343-349` TunnelConnect JSON includes `relay_token: cfg.token.clone()` and `server_id: cfg.server_id`.
4. `opt/relay/src/tunnel.rs` has 0 matches for `Mutex::new(None)` (the yamux_control stub is replaced with `Some(session.control())`).
5. `opt/relay/src/tunnel.rs` has 0 matches for `Message::Text` (the gateway no longer reads Text frames directly).
6. `opt/relay/src/tunnel.rs` has ≥1 match for `Session::new_server`, `Message::Binary`, `session.control()`, `ws_bridge`, and `auth::authorize` (or `state.backend.authorize`).
7. `opt/relay/src/tunnel.rs` has 3 new private helpers: `ws_bridge`, `read_control_stream`, `read_json_message`.
8. `opt/relay/src/tunnel.rs` `TunnelConnect` struct has the new `pub relay_token: Uuid` field.
9. `cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay && cargo check` exits 0 with no new errors and no new warnings.
10. `cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && cargo check` exits 0 with no new errors and no new warnings.
11. The 3 BLOCKERs (yamux session, WS frame type, auth::authorize called) are closed; BLOCKER #4 (rate limiter) is explicitly noted as a verifier false positive in the plan's `out_of_scope` frontmatter and is NOT addressed.
12. The `TunnelHandle` shape in `opt/relay/src/registry.rs` is byte-identical (no field changes).
13. The `Registry::register` 1-tunnel-per-server_id (D-21) enforcement in `opt/relay/src/registry.rs:42-61` is byte-identical.
14. The handshake parser + by_subdomain routing + 100 req/min rate limit (already wired at player.rs:37) are byte-identical.
15. No new dependencies added; `tokio-yamux 0.3`, `tokio-tungstenite 0.26`, `tokio 1`, `futures 0.3`, `serde`, `serde_json`, `uuid`, `tracing` are all already in `opt/relay/Cargo.toml` and the root `Cargo.toml`.
16. No other file is touched (the 4 modified files are `src/state.rs`, `src/main.rs`, `src/handlers/relay_client.rs`, `opt/relay/src/tunnel.rs`).
17. Verifier re-check should show 28/28 must-haves verified (was 22/28; the missing 6 = 3 BLOCKERs × 2 must-haves each + the 0 expected from BLOCKER #4's false-positive status).
</success_criteria>

<output>
After completion, create `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-gap-01-SUMMARY.md` with a single-section summary covering: (a) the 4-file diff summary (lines added/removed in tunnel.rs, relay_client.rs, state.rs, main.rs); (b) the verifier re-check result table mirroring VERIFICATION.md:79-108 (all 22/28 must-haves from the original run should now be 28/28); (c) `cargo check` exit codes for both opt/relay and parent agent; (d) explicit confirmation that BLOCKER #4 (rate limiter) is a verifier false positive and was NOT addressed; (e) the commit message following the project's `fix(68-gap-01): <description>` style.
</output>
