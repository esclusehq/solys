---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
verified: 2026-06-07T16:10:00Z
status: gaps_found
score: 22/28 must-haves verified
overrides_applied: 0
overrides: []
re_verification:
  previous_status: null
  previous_score: null
  gaps_closed: []
  gaps_remaining: []
  regressions: []
gaps:
  - truth: "Gateway accepts outbound WSS from agents on wss://relay.esluce.net/relay/tunnel, authenticates via HMAC-signed POST to backend's /internal/relay/authorize, then opens yamux"
    status: failed
    reason: "The gateway's tunnel.rs has explicit TODOs admitting it does NOT open a real yamux server session. It stores `yamux_control: tokio::sync::Mutex::new(None)` and the comment says: 'We don't have a real yamux session because the WebSocket is just a control plane in this MVP; yamux streams come over a side-channel. For now, register a placeholder handle and report lifecycle events to the backend.' Additionally, `auth::authorize` is defined in `opt/relay/src/auth.rs` but is never called from `tunnel.rs` or anywhere in the gateway code (`grep -n auth::authorize opt/relay/src/*.rs` returns only a use-import in backend.rs). The HMAC-signed backend authorize step never runs."
    artifacts:
      - path: "opt/relay/src/tunnel.rs"
        issue: "Stub yamux (Mutex<Option<Control>> = None), no auth::authorize call, no bearer token validation. TODO comment at line 90 admits the gap."
    missing:
      - "Implement tokio_yamux server session over the WebSocket bytes (mirror the agent's `Session::new_client` but use `Session::new_server`)"
      - "Call `auth::authorize(token, server_id)` from tunnel.rs after parsing the TunnelConnect (must extract bearer token from TunnelConnect payload)"
      - "Have the agent add the bearer token to the TunnelConnect JSON (currently sends only subdomain, public_port, agent_public_ip, region; server_id is `Uuid::nil()`)"

  - truth: "Gateway accepts raw TCP on :25565 (NLB passthrough), parses the Minecraft Java Handshake packet to extract the subdomain from <subdomain>.play.esluce.net, looks up server_id by subdomain, and proxies to the matching yamux stream"
    status: failed
    reason: "Handshake parsing and registry.lookup_by_subdomain work correctly. However, the proxy step to the matching yamux stream is broken: the TunnelHandle's `yamux_control: Mutex<Option<Control>>` is always `None` (set in tunnel.rs:110), so `control_lock.as_ref()` matches the `None` arm at player.rs:82 and the player connection is dropped with '[PLAYER] Tunnel handle has no yamux control (stale); closing'. No real yamux stream can be opened against the agent because the gateway never sets up the yamux server session."
    artifacts:
      - path: "opt/relay/src/player.rs"
        issue: "Yamux stream open is unreachable — handle.yamux_control is always None"
      - path: "opt/relay/src/tunnel.rs"
        issue: "Sets yamux_control to None placeholder instead of a real Control handle"
    missing:
      - "Real yamux server session in tunnel.rs that stores its Control handle in TunnelHandle.yamux_control"
      - "Once yamux is wired, player.rs will work end-to-end"

  - truth: "Gateway enforces 100 req/min per source IP rate limit at the player TCP layer (D-20) via in-process token bucket"
    status: failed
    reason: "The token-bucket rate limiter exists in opt/relay/src/ratelimit.rs but is never invoked from the player TCP handler. player.rs accepts every connection without calling check_rate_limit on the source IP. With no real yamux session to protect, the rate limit would be the only defense against a flood, and it is not wired."
    artifacts:
      - path: "opt/relay/src/ratelimit.rs"
        issue: "Token bucket implemented but never called from player.rs"
    missing:
      - "Call `state.rate_limiter.check_rate_limit(&peer_ip)` at the top of player.rs' connection handler before any handshake parsing"

  - truth: "Agent establishes outbound encrypted tunnel to relay.esluce.net. Automatic reconnect with exponential backoff on disconnect. Heartbeat every 10s; tunnel marked stale after 3 missed heartbeats."
    status: partial
    reason: "Agent side is correct: Session::new_client + duplex bridge + ws_bridge + 10s heartbeat + 1s→30s exponential backoff + 24h rekey + cleanup on disconnect. However, the gateway never accepts the binary yamux frames the agent sends (it expects Message::Text JSON), so the connection is rejected at the first frame. End-to-end: agent loop is well-implemented but cannot complete the handshake with the current gateway."
    artifacts:
      - path: "src/handlers/relay_client.rs"
        issue: "Sends Message::Binary yamux frames (line 565, 579); gateway rejects with 'Expected first message Text, got Binary'"
      - path: "opt/relay/src/tunnel.rs"
        issue: "Only matches Message::Text; never reads Message::Binary (line 46, 152)"
    missing:
      - "Either the agent sends raw JSON text frames (and the gateway reads yamux from a side-channel), OR the gateway also runs a `ws_bridge` task that feeds binary frames into a `Session::new_server`"

deferred: []
human_verification: []
---

# Phase 68: Escluse Relay Infrastructure Verification Report

**Phase Goal:** Implement Esluce Relay as the primary connectivity path for Minecraft servers, with relay-backed stable DNS on `*.play.esluce.net` and conditional Direct Mode fast-path on `*.play.esluce.com`.

**Verified:** 2026-06-07T16:10:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

The backend service layer (api subrepo: schema, entities, RelayService, REST handlers, internal HMAC endpoints, monitoring), the agent tunnel client (yamux, backoff, heartbeat, audit), the dashboard UI (ConnectivitySection, TunnelHealthCard, ModeOverrideDropdown, InviteFriendsModal), the containerization layer (Dockerfile, Caddyfile, docker-compose), the operator runbook (DEPLOY.md), and the registry/Handshake-parse routing *design* (by_subdomain + by_server_id, no by_agent_ip) are all in place and verified.

However, the **relay-gateway crate's tunnel control plane is a stub**: it does not open a real yamux server session, it does not invoke `auth::authorize`, and its WS message framing is incompatible with what the agent sends. As a result, **no real player traffic can flow through the relay** — every player TCP connection that reaches the gateway will hit the `[PLAYER] Tunnel handle has no yamux control (stale); closing` branch. The agent's well-built yamux client has nothing on the other end to multiplex with.

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Migration adds 7 relay columns + 3 indexes | VERIFIED | `api/migrations/20260608000001_add_relay_columns.sql` exists with `ALTER TABLE nodes ADD COLUMN relay_token UUID`, `relay_token_issued_at TIMESTAMPTZ`, plus 4 server columns (`relay_status`, `connectivity_mode_override`, `last_tunnel_connected_at`, `last_tunnel_disconnected_at`, `last_tunnel_disconnect_reason`) and 3 indexes |
| 2 | `Node` entity has `relay_token: Option<Uuid>` and `issue_relay_token` | VERIFIED | `api/src/domain/entities/node.rs`: `pub relay_token: Option<Uuid>`, `pub relay_token_issued_at: Option<DateTime<Utc>>`, `pub fn issue_relay_token(&mut self, new_token: Uuid) -> Self` |
| 3 | `Server` entity has relay fields + 3 mutator methods | VERIFIED | `api/src/domain/entities/server.rs`:84-150: `relay_status`, `connectivity_mode_override`, `last_tunnel_connected_at`, `last_tunnel_disconnected_at`, `last_tunnel_disconnect_reason` + `set_relay_status`, `set_mode_override`, `record_tunnel_disconnect` |
| 4 | `find_by_relay_token` exists in trait + impl | VERIFIED | `api/src/domain/repositories/node_repository.rs` (trait) + `api/src/infrastructure/repositories/postgres_node_repository.rs:113` (impl) |
| 5 | WebSocket protocol includes TunnelConnect/Disconnect/Heartbeat | VERIFIED | `api/src/presentation/ws/node_protocol.rs`: `TunnelConnect`, `TunnelDisconnect`, `TunnelHeartbeat`, `ModeOverrideChange`, `TunnelCloseAck` variants present |
| 6 | `RelayService` has 9 methods | VERIFIED | `api/src/application/services/relay_service.rs`: `ensure_relay_token`, `authorize_gateway`, `verify_server_ownership`, `on_tunnel_connect`, `on_tunnel_disconnect`, `on_tunnel_heartbeat`, `set_mode_override`, `get_mode_override`, `get_tunnel_health` |
| 7 | `relay_handlers` exposes GET/PUT mode + GET tunnel-health | VERIFIED | `api/src/presentation/handlers/relay_handlers.rs` |
| 8 | `internal_relay_handlers` has HMAC verify + 2 internal endpoints | VERIFIED | `api/src/presentation/handlers/internal_relay_handlers.rs:100` `verify_hmac` with `Hmac<Sha256>`; `post_authorize` (line 243) and `post_tunnel_event` (line 302) |
| 9 | `node_ws_handler` dispatches 3 relay events | VERIFIED | `api/src/presentation/handlers/node_ws_handler.rs:457-499`: TunnelConnect/Disconnect/Heartbeat dispatch arms |
| 10 | `api_routes` mounts relay router | VERIFIED | `api/src/presentation/routes/api_routes.rs:43,76`: relay + internal_relay routers |
| 11 | Container has `relay_service: Arc<RelayService>` | VERIFIED | `api/src/bootstrap/container.rs:160` |
| 12 | Monitoring service has relay metrics | VERIFIED | `api/src/application/services/monitoring_service.rs`: `scrape_relay_metrics`, `evaluate_relay_alerts`, `emit_relay_alert`, `RelayMetricsHistory` |
| 13 | PUT mode handler normalizes "auto" → None | VERIFIED | `api/src/presentation/handlers/relay_handlers.rs:85-98`: match on `body.mode.as_deref()` produces `Option<String>` with null for "auto" |
| 14 | API subrepo compiles cleanly | VERIFIED | `cargo check` (api/): 77 warnings, no errors |
| 15 | Agent has yamux, backoff, heartbeat, 3 dispatch arms | VERIFIED | `agent/Cargo.toml`: `tokio-yamux = "0.3"`, `tokio-util`, `futures`, `base64`, `hmac`, `sha2`, `hex`; `src/handlers/mod.rs:178-184`: relay.connect/disconnect/heartbeat/remove_cname_record; `src/handlers/relay_client.rs`: `Session::new_client`, `exponential_backoff (1s → 30s)`, 10s heartbeat, 24h rekey threshold |
| 16 | Agent RelayClient bootstrapped only if `AGENT_RELAY_TOKEN` set | VERIFIED | `src/main.rs:327-455`: `bootstrap_relay_client` reads `std::env::var("AGENT_RELAY_TOKEN")`, no-op if missing |
| 17 | Agent compiles cleanly | VERIFIED | `cargo check` (root): 16 warnings, no errors |
| 18 | Gateway has by_subdomain + by_server_id, NO by_agent_ip (BLOCKER 1 fix) | VERIFIED | `opt/relay/src/registry.rs:21-34`: `by_subdomain: DashMap<String, Uuid>`, `by_server_id: DashMap<Uuid, Arc<TunnelHandle>>`; explicit comment "There is intentionally NO `by_agent_ip` map" |
| 19 | Handshake parser extracts subdomain | VERIFIED | `opt/relay/src/player.rs:10-235`: `read_mc_handshake_subdomain` + `try_parse_handshake` + `RELAY_SUFFIX = ".play.esluce.net"` validation; rejects malformed subdomains |
| 20 | Gateway uses 9100 for metrics, not 9090 (D-22) | VERIFIED | `opt/relay/relay-gateway.toml:4`: `metrics_bind = "0.0.0.0:9100"` |
| 21 | Gateway Dockerfile builds relay-gateway crate | VERIFIED | `opt/relay/Dockerfile`: multi-stage `rust:1.81-slim-bookworm`, `cargo build -p relay-gateway --release`, copies binary to `/usr/local/bin/relay-gateway` |
| 22 | Caddy image has caddy-dns/route53 plugin | VERIFIED | `opt/relay/Caddy.Dockerfile`: `caddy build-modules --modules github.com/caddy-dns/route53` |
| 23 | Caddyfile enforces TLS 1.3, routes relay.esluce.net + *.play.esluce.net | VERIFIED | `opt/relay/Caddyfile`: `tls { protocols tls1.3 }`, `reverse_proxy @websocket relay-gateway:8080` |
| 24 | docker-compose orchestrates gateway + caddy on relay-net with 9100:9100 | VERIFIED | `opt/relay/docker-compose.yml`: shared `relay-net` network, `25565:25565` and `9100:9100` ports, `healthcheck: http://localhost:9100/metrics` |
| 25 | DEPLOY.md has AWS Setup, NLB/Route 53/EC2/IAM, static wildcard note, scoped IAM | VERIFIED | `opt/relay/DEPLOY.md`: sections `## AWS Setup`, `## Deploy`, `## Verify`, `## Troubleshooting`; mentions "STATIC", "manual AWS Console", scoped `route53:ChangeResourceRecordSets`, `docker compose up -d --build` |
| 26 | Dashboard has TunnelHealthCard + ModeOverrideDropdown + InviteFriendsModal + useConnectivity | VERIFIED | `app/src/components/{TunnelHealthCard,ModeOverrideDropdown,InviteFriendsModal,ConnectivitySection}.jsx` + `app/src/hooks/useConnectivity.js` (15s polling) |
| 27 | `relayApi` exposes getMode/setMode/getTunnelHealth/joinWaitlist | VERIFIED | `app/src/lib/api.js:219-223`: `relayApi` with all 4 methods; comment at 218 notes `setMode(value === 'auto' ? null : value)` for explicit null |
| 28 | App builds | VERIFIED | `npm run build` (app/): `dist/index.html 0.86 kB`, `dist/assets/index-uAJp7qHM.js 1,404.93 kB`, `built in 7.48s` |

**Score:** 22/28 truths verified. The 4 unverified truths are the BLOCKERS in `gaps:` above (each represents a key_link in 04a that the agent + gateway cannot complete end-to-end).

### Required Artifacts (Level 1-2)

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `api/migrations/20260608000001_add_relay_columns.sql` | 7 columns + 3 indexes | VERIFIED | Migration present |
| `api/src/domain/entities/node.rs` | `relay_token`, `issue_relay_token` | VERIFIED | Methods + fields present |
| `api/src/domain/entities/server.rs` | `relay_status`, `set_mode_override`, `record_tunnel_disconnect` | VERIFIED | All 3 methods + 5 fields |
| `api/src/application/services/relay_service.rs` | 9 methods | VERIFIED | All 9 methods present |
| `api/src/presentation/handlers/relay_handlers.rs` | GET/PUT mode, GET tunnel-health | VERIFIED | All 3 endpoints |
| `api/src/presentation/handlers/internal_relay_handlers.rs` | HMAC verify + 2 endpoints | VERIFIED | `verify_hmac` with `Hmac<Sha256>` |
| `api/src/presentation/handlers/node_ws_handler.rs` | 3 dispatch arms | VERIFIED | Connect/Disconnect/Heartbeat |
| `api/src/presentation/routes/api_routes.rs` | relay + internal_relay mounted | VERIFIED | 2 mount points |
| `api/src/bootstrap/container.rs` | `relay_service: Arc<RelayService>` | VERIFIED | Field present |
| `api/src/application/services/monitoring_service.rs` | relay metrics | VERIFIED | 4 methods + RelayMetricsHistory |
| `src/handlers/relay_client.rs` | yamux client + backoff + heartbeat | VERIFIED | Compiles clean |
| `src/handlers/relay_session.rs` | bidi copy + bytes counter | VERIFIED | `run_relay_session`, `copy_bidirectional_with_count` |
| `src/main.rs` | bootstrap_relay_client gated on env | VERIFIED | 327-455 |
| `opt/relay/Cargo.toml` | 19 deps, axum + yamux + prometheus | VERIFIED | Compiles clean (17 warnings) |
| `opt/relay/relay-gateway.toml` | `metrics_bind = "0.0.0.0:9100"` | VERIFIED | D-22 satisfied |
| `opt/relay/src/registry.rs` | by_subdomain + by_server_id, NO by_agent_ip | VERIFIED | BLOCKER 1 fix landed |
| `opt/relay/src/player.rs` | Handshake parser + by_subdomain lookup | VERIFIED | Parser + routing logic |
| `opt/relay/src/backend.rs` | HMAC-SHA256 sign() | VERIFIED | `sign(method, path, body, timestamp, nonce)` |
| `opt/relay/src/ratelimit.rs` | token bucket (D-20) | VERIFIED | Structure present, **but never called** |
| `opt/relay/Dockerfile` | multi-stage Rust build for relay-gateway | VERIFIED | `rust:1.81-slim-bookworm` + `cargo build -p relay-gateway --release` |
| `opt/relay/Caddy.Dockerfile` | caddy-dns/route53 | VERIFIED | Plugin build step |
| `opt/relay/Caddyfile` | TLS 1.3 + WS reverse proxy | VERIFIED | `tls { protocols tls1.3 }` + `reverse_proxy @websocket relay-gateway:8080` |
| `opt/relay/docker-compose.yml` | relay-gateway + caddy on relay-net | VERIFIED | 25565:25565 + 9100:9100 + relay-net |
| `opt/relay/DEPLOY.md` | AWS NLB/Route 53/EC2/IAM runbook | VERIFIED | All sections + static note + scoped IAM |
| `app/src/hooks/useConnectivity.js` | 15s polling | VERIFIED | Default 15000ms |
| `app/src/components/ConnectivitySection.jsx` | wires 3 children + Public Addresses | VERIFIED | Renders TunnelHealthCard + ModeOverrideDropdown + InviteFriendsModal |
| `app/src/components/TunnelHealthCard.jsx` | status card | VERIFIED | Uses useConnectivity hook |
| `app/src/components/ModeOverrideDropdown.jsx` | Auto/Relay/Direct | VERIFIED | Implements null for "auto" |
| `app/src/components/InviteFriendsModal.jsx` | dual-address copy + waitlist | VERIFIED | D-14, D-15 |
| `app/src/lib/api.js` | `relayApi` 4 methods | VERIFIED | Lines 219-223 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `api/src/presentation/handlers/relay_handlers.rs` | `api/src/application/services/relay_service.rs` | service injection | WIRED | `state.relay_service.set_mode_override(...)` (line 98) |
| `api/src/presentation/handlers/internal_relay_handlers.rs` | HMAC verify | `verify_hmac(headers, body, method_and_path, redis)` | WIRED | Called at line 243 and 302 |
| `api/src/application/services/relay_service.rs` | `node_repository.find_by_relay_token` | `self.node_repository.find_by_relay_token(token)` | WIRED | Line 83 |
| `api/src/presentation/handlers/node_ws_handler.rs` | `RelayService` | 3 dispatch arms | WIRED | Lines 457-499 |
| `opt/relay/src/backend.rs` | backend `/internal/relay/authorize` | HMAC-signed POST | WIRED | `state.backend.authorize(*token, *server_id)` |
| `opt/relay/src/registry.rs` | by_subdomain DashMap | `lookup_by_subdomain` | WIRED | Line 64 |
| `opt/relay/src/player.rs` | `opt/relay/src/registry.rs` | `state.registry.lookup_by_subdomain(&subdomain)` | WIRED | Line 60 |
| `opt/relay/src/heartbeat.rs` | `opt/relay/src/registry.rs` | `state.registry.mark_stale(&server_id)` | WIRED | Line 46 |
| `opt/relay/src/tunnel.rs` | `opt/relay/src/registry.rs` | `state.registry.register(handle.clone())` | WIRED | Line 116 |
| `opt/relay/src/tunnel.rs` | `opt/relay/src/backend.rs` | `state.backend.report_tunnel_event(...)` | WIRED | Line 146 (heartbeat) |
| **`opt/relay/src/tunnel.rs`** | **`opt/relay/src/auth.rs`** | **`auth::authorize`** | **NOT_WIRED** | **`auth::authorize` exists in auth.rs but `grep -n auth::authorize opt/relay/src/*.rs` returns zero call sites in tunnel.rs (only `use crate::auth::Authorization;` in backend.rs). The 04a key_link is unmet.** |
| **`opt/relay/src/player.rs`** | **yamux `Control::open_stream`** | **stream multiplexing** | **NOT_WIRED** | **`handle.yamux_control: Mutex<Option<Control>>` is always `None` (set in tunnel.rs:110). The `None` arm at player.rs:82 logs "Tunnel handle has no yamux control (stale); closing" and drops the connection. The 04a truth "forwards TCP to the matching yamux stream" is false.** |
| **`opt/relay/src/player.rs`** | **`opt/relay/src/ratelimit.rs`** | **per-IP rate limit** | **NOT_WIRED** | **Rate limiter is implemented but `state.rate_limiter.check_rate_limit(&peer)` is never called from player.rs. Without it, the player TCP listener is a free flood target.** |
| `src/handlers/relay_client.rs` | `opt/relay/src/tunnel.rs` (gateway WS endpoint) | WSS + TunnelConnect JSON + yamux | **WIRED-ON-AGENT, BROKEN-ON-GATEWAY** | Agent sends `Message::Binary` yamux frames via ws_bridge (line 565, 579); gateway expects `Message::Text` JSON (line 46, 152). The first frame the agent sends is binary yamux data, which the gateway rejects with "Expected first message Text, got Binary" |
| `app/src/components/ConnectivitySection.jsx` | `app/src/hooks/useConnectivity.js` | `useConnectivity(serverId)` | WIRED | Lines 48-49 |
| `app/src/components/TunnelHealthCard.jsx` | `app/src/hooks/useConnectivity.js` | `useConnectivity(serverId)` | WIRED | Line 41 |
| `app/src/components/ModeOverrideDropdown.jsx` | `app/src/hooks/useConnectivity.js` | `useConnectivity(serverId)` | WIRED | Line 16 |
| `app/src/hooks/useConnectivity.js` | `app/src/lib/api.js` | `relayApi.getMode / setMode / getTunnelHealth` | WIRED | Hook calls `relayApi.getMode(serverId)`, etc. |
| `app/src/pages/servers/ServerDetailsPage.jsx` | `app/src/components/ConnectivitySection.jsx` | `<ConnectivitySection server={server} />` | WIRED | Line 387 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `TunnelHealthCard.jsx` | `connectivity` (mode, status, last_connected_at) | `relayApi.getTunnelHealth(serverId)` → `GET /servers/{id}/relay/tunnel-health` → `RelayService::get_tunnel_health` → reads `server.relay_status`, `last_tunnel_connected_at` from DB | YES (DB query) | FLOWING |
| `ModeOverrideDropdown.jsx` | `mode` | `relayApi.getMode / setMode` → `GET/PUT /servers/{id}/relay/mode` → `RelayService::get/set_mode_override` → `servers.connectivity_mode_override` | YES (DB write) | FLOWING |
| `InviteFriendsModal.jsx` | `subdomain`, public addresses | Subdomain derived from `server.subdomain` field; relay address is `${subdomain}.play.esluce.net` | YES (composed from server) | FLOWING |
| `useConnectivity.js` | `connectivity` | 15s polling interval calling `relayApi.getMode` + `getTunnelHealth` | YES (polling loop) | FLOWING |
| `opt/relay/src/registry.rs` | `by_subdomain` map | `tunnel.rs` calls `state.registry.register(handle.clone())` (line 116) | NO (placeholder handle with `yamux_control: None`) | DISCONNECTED — registry is populated, but the handles it stores cannot open yamux streams |
| `opt/relay/src/player.rs` | TCP-forwarded player traffic | `handle.yamux_control.open_stream()` (line 88) | NO (always hits `None` arm) | DISCONNECTED — all player connections are dropped with "Tunnel handle has no yamux control (stale); closing" |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| API subrepo compiles | `cargo check` (in `api/`) | 77 warnings, 0 errors | PASS |
| Agent root compiles | `cargo check` (root) | 16 warnings, 0 errors | PASS |
| Gateway crate compiles | `cargo check -p relay-gateway` | 17 warnings, 0 errors | PASS |
| App builds | `npm run build` (in `app/`) | `dist/index.html 0.86 kB`, `built in 7.48s` | PASS |
| Gateway is reachable for tunnels | WSS handshake with agent | n/a — requires running gateway | SKIP (no service running) |
| Player TCP forwarded to agent | Handshake parse + yamux stream open | n/a — cannot test without real yamux session on gateway | SKIP (BLOCKER) |
| HMAC sign/verify symmetric | unit test or `cargo test` | n/a — only the binary compiles, not executed | SKIP (not run) |

### Requirements Coverage

The PLAN frontmatter for 04a-04c and 03 declares these requirement IDs: `DEPLOY-01`, `DEPLOY-02`, `DEPLOY-03`, `DEPLOY-04`, `DEPLOY-05`, `STATUS-01`, `STATUS-02`. These map onto the existing `REQUIREMENTS.md` IDs.

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| DEPLOY-01 | 04a, 04b, 04c | User can deploy a game server to a selected node | SATISFIED (deploy infrastructure); BLOCKED (relay tunnel itself) | Schema + entities + gateway crate + Docker + runbook all present. **But: deployed servers cannot receive relay traffic** because the gateway cannot establish yamux sessions with agents. |
| DEPLOY-02 | 04a, 04b, 04c | User can start a deployed game server | UNCHANGED (Phase 5 already implements) | Out of Phase 68 scope |
| DEPLOY-03 | 04a, 04b, 04c | User can stop a running game server | UNCHANGED | Out of Phase 68 scope |
| DEPLOY-04 | 04c | User can restart a running game server | UNCHANGED | Out of Phase 68 scope |
| DEPLOY-05 | 04a, 04b | User can delete a game server | UNCHANGED | Out of Phase 68 scope |
| STATUS-01 | 04a | User can view current server status | SATISFIED | `relay_status` field + `/relay/tunnel-health` endpoint + dashboard card |
| STATUS-02 | 04a | User can view server resource usage (CPU, RAM, disk) | UNCHANGED | Out of Phase 68 scope |

No orphaned requirements (the RELAY-01/02/03 IDs in the 04a SUMMARY's `requirements-completed` line are a Phase 68 internal labeling — not in `REQUIREMENTS.md`).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `opt/relay/src/tunnel.rs` | 96-110 | Explicit TODO: "We don't have a real yamux session because the WebSocket is just a control plane in this MVP; yamux streams come over a side-channel. For now, register a placeholder handle..." | **BLOCKER** | No player traffic can be proxied. The core value of Phase 68 (relay forwarding) is non-functional. |
| `opt/relay/src/tunnel.rs` | 87-90 | Explicit TODO: "TODO: when the agent adds the bearer token to TunnelConnect, verify here." | **BLOCKER** | No authentication of agent → gateway at tunnel-establishment time. Backend `/internal/relay/authorize` HMAC check exists but is never invoked. |
| `opt/relay/src/player.rs` | 79-86 | `control_lock.as_ref()` matches `None` arm | **BLOCKER** | Every player connection that reaches the gateway is dropped with "Tunnel handle has no yamux control (stale); closing". |
| `opt/relay/src/player.rs` | (whole file) | `state.rate_limiter` never called | **WARNING** | Without rate limiting, a single attacker can flood the gateway with TCP connections and exhaust file descriptors. |
| `opt/relay/src/tunnel.rs` | 46 | `Message::Text` only match | **BLOCKER** | Incompatible with agent's `Message::Binary` yamux frames. The agent's `ws_bridge` (relay_client.rs:545-580) explicitly skips text frames, so the gateway would never see one. |

No other stubs found — the schema, entities, RelayService, REST handlers, internal HMAC handlers, monitoring, agent tunnel client, dashboard UI, Dockerfile, Caddyfile, docker-compose, and DEPLOY.md are all substantively implemented and correctly wired.

### Human Verification Required

(none — the core value proposition is broken at the code level, not at a level requiring human testing)

### Gaps Summary

The phase 68 backend, agent, dashboard, containerization, and runbook layers are well-implemented. The **gateway's tunnel control plane is the bottleneck**: it was scoped down to a "control plane MVP" where it reads a JSON text frame as the first WS message and registers a placeholder handle with `yamux_control: None`. This worked as a registry for the heartbeat watcher, but the player-to-tunnel forwarder in `opt/relay/src/player.rs` cannot open a yamux stream against `None` — so every player TCP connection is dropped at `player.rs:82` with "Tunnel handle has no yamux control (stale); closing".

The agent side was built to a fuller specification (real `Session::new_client` over a `tokio::io::duplex` + `ws_bridge` task that translates yamux frames to WS binary frames), but the gateway does not mirror that: there is no `ws_bridge` task on the gateway, and no `Session::new_server` invocation. The two components are also message-frame-incompatible: agent sends `Message::Binary`, gateway reads `Message::Text`.

Additionally, the rate limiter (`opt/relay/src/ratelimit.rs`) and the HMAC `auth::authorize` function are implemented but never wired into the player or tunnel control paths.

To close the gaps, the 04a plan needs a follow-up that:

1. **Decide on a single architecture** — either:
   - (a) The agent sends raw JSON over WS and a yamux session is created over a separate side-channel (as the gateway's TODO comment suggests but no such side-channel exists in the codebase), OR
   - (b) The gateway mirrors the agent: split the WS into a yamux server session via `tokio::io::duplex` + `ws_bridge`, read TunnelConnect JSON from a yamux stream, and store the resulting `Control` handle in `TunnelHandle.yamux_control`.
2. **Wire `auth::authorize` into `tunnel.rs`** — after parsing the TunnelConnect, call `state.backend.authorize(token, server_id)`. Add the bearer token to the agent's TunnelConnect payload (currently sends `server_id: Uuid::nil()`).
3. **Wire `state.rate_limiter.check_rate_limit(&peer)`** at the top of player.rs' connection handler.

The backend, agent, dashboard, Docker, and runbook layers do not need re-work — only the gateway's tunnel control plane + player forwarder.

---

_Verified: 2026-06-07T16:10:00Z_
_Verifier: the agent (gsd-verifier)_
