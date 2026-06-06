# Phase 68: Escluse Relay Infrastructure - Context

**Gathered:** 2026-06-07
**Status:** Ready for planning
**Mode:** yolo-autonomous (rich ROADMAP spec + Phase 67 carryforward enabled single-pass decisions; user can override in CONTEXT.md before plan-phase)

<domain>
## Phase Boundary

Implement Esluce Relay as the primary connectivity path for Minecraft servers, providing:

1. **Relay Gateway Service** — Deployed on AWS, accepts persistent outbound tunnel connections from agents, maintains an active tunnel registry indexed by `server_id`, handles Minecraft Java TCP traffic forwarded to the correct tunnel
2. **Agent Tunnel Client** — Outbound encrypted tunnel from agent to `relay.esluce.net`, automatic reconnect with exponential backoff, heartbeat-driven staleness detection
3. **Player Routing Layer** — Player connects to `<server>.play.esluce.net` → gateway parses Host header → looks up active tunnel → forwards TCP. Reject (close socket) when no active tunnel exists
4. **DNS Integration (relay-first)** — New `*.play.esluce.net` wildcard on Route 53 (always-on, stable). Existing `*.play.esluce.com` on Cloudflare stays as conditional Direct-Mode endpoint, emitted only when probe-verified
5. **Connectivity Mode Selection (relay-default)** — Relay is the default and primary path. Direct Mode is a fast-path optimization. User can pin a mode per server. Mode flip on probe + tunnel events
6. **Dashboard Integration** — Extend Phase 67 Connectivity section to show tunnel health, both addresses when applicable, mode (Direct/Relay/Offline)
7. **Security** — Per-agent token auth, server_id ownership validation, TLS 1.3, rate limits, nonce-based replay protection
8. **Monitoring** — Active tunnels, bandwidth, latency p50/p95/p99, error rates, mode distribution

**Out of scope for Phase 68:**
- Bedrock Edition UDP support (architecturally different, deferred)
- Multi-region relay failover (single region + single AZ for initial scope)
- IPv6-only path support
- Custom Esluce relay agent binary distribution (reuses existing agent with new task types)
</domain>

<decisions>
## Implementation Decisions

### Tunnel Protocol & Architecture
- **D-01 (Tunnel transport):** **WebSocket over TLS 1.3 (`wss://`)** — reuses `tokio-tungstenite 0.26` already in the stack (per `.planning/codebase/INTEGRATIONS.md`); follows the existing agent↔backend WS pattern. Avoids raw TCP (cert management complexity) and QUIC (no Rust client in stack, would need new dep). Custom framing layer above WebSocket carries the multiplexed player TCP streams.
- **D-02 (Multiplexing):** **Single WebSocket per agent, multiplexed streams via `yamux`** (Rust-native, used by tonic/gRPC). One persistent connection per agent handles all servers on that node. Each Minecraft player connection is a separate yamux stream within the tunnel. Per-server stream isolation for error handling and bandwidth accounting.
- **D-03 (Encryption):** **TLS 1.3 end-to-end at all hops** — player→relay via `wss://` or Caddy-terminated TLS; relay→agent via `wss://` WebSocket; player TCP traffic inside the tunnel is TLS-encrypted by the WebSocket layer (no double-encryption needed). No additional application-layer crypto.
- **D-04 (Tunnel reconnection):** **Exponential backoff with jitter** — initial 1s, doubles up to 30s cap, ±20% jitter. 10s heartbeat, 30s stale timeout (3 missed = tunnel marked stale). New tunnel replaces old atomically (relay holds the new connection, drops the old). Agent re-registers on every reconnect.

### Gateway Architecture & Player Routing
- **D-05 (Gateway deployment):** **Single Rust + Axum service on AWS EC2 (ap-southeast-1, single AZ)** — Caddy terminates TLS for player connections (mirrors Phase 66 analytics deployment pattern); the same EC2 instance runs the relay gateway (Rust) that handles WS tunnels + player TCP forwarding. Behind an ALB (HTTP/HTTPS termination) and an NLB (raw TCP for Minecraft Java player traffic on port 25565). Instance type: `c6i.large` (network-optimized, 2 vCPU, 4 GiB RAM) — agent's discretion to tune.
- **D-06 (Player DNS resolution):** **Wildcard `*.play.esluce.net` → A record → NLB IP** — Player's Minecraft client resolves `<server>.play.esluce.net` to the NLB, opens TCP connection. NLB passes through to the relay gateway with the original destination IP. Gateway uses the **TLS SNI / Host header** (for `wss://` connections) OR a **custom HTTP header in the proxy_protocol** (for raw TCP) to extract the server_id subdomain. Yamux stream is opened on the agent's tunnel for that server_id, and the player's TCP bytes are forwarded transparently.
- **D-07 (DNS hosting):** **`esluce.net` delegated to AWS Route 53** (relay infrastructure owns it). **`esluce.com` stays on Cloudflare** (existing Phase 51 setup; reuses `dns_config` table for API token). Agent owns DNS lifecycle for both zones per current connectivity mode (Phase 67 D-13 + new tunnel-event triggers).
- **D-08 (Direct Mode DNS):** **`<server>.play.esluce.com` A record on Cloudflare, emitted only when Direct Mode is probe-verified** (reuses Phase 51 wildcard pattern). Removed on Direct probe failure or mode flip to Relay. TTL: 60s. Existing Phase 51 records for servers predating Phase 68 are preserved as-is (backward compat, D-24).

### Server ID Authentication at Relay
- **D-09 (Tunnel registration auth):** **Per-agent token issued at agent registration** — new column `relay_token UUID NOT NULL UNIQUE` on `nodes` table; generated by backend on `node.register` event; returned in the agent's registration response. Agent persists the token in its config file. On every tunnel connect, agent sends `Authorization: Bearer <relay_token>` header.
- **D-10 (Server ID ownership validation):** **Backend HTTP introspection** — Relay calls `POST https://api.esluce.com/api/v1/internal/relay/authorize` with `relay_token + server_id` before accepting the tunnel registration. Backend returns `200` if `node.owns(server_id)`, `403` otherwise. Cached in relay memory for the tunnel lifetime. Latency budget: <50ms p99; agent connection fails fast if introspection errors.
- **D-11 (Replay protection):** **Per-connection nonce + timestamp** — agent generates a random 32-byte nonce + current Unix timestamp on each tunnel connect; sent in the WebSocket upgrade request as `X-Relay-Nonce` and `X-Relay-Timestamp` headers. Relay rejects with `401` if `(now - timestamp) > 5 minutes` OR if the nonce was seen in the last 10 minutes (Redis-backed dedup). Prevents replay of captured handshake.

### Mode Selection & User Control
- **D-12 (Mode selection policy):** **Automatic with per-server user override** — agent picks Direct vs Relay based on Phase 67 D-02 probe results + new tunnel health signals. User can pin a mode in the dashboard ("Force Relay" / "Force Direct") per server; pinned mode is stored in `servers.connectivity_mode_override` column (nullable text, default NULL = automatic).
- **D-13 (Mode flip triggers):** **Phase 67 D-02 triggers + tunnel events** — same triggers as Phase 67 (after `server.start`, on IP change, on firewall change, periodic 5 min) + new triggers: `tunnel_disconnect` → flip to Relay immediately, `tunnel_reconnect` → re-probe Direct after 30s of stable tunnel, `direct_probe_latency > 50ms penalty vs relay` → flip to Relay.
- **D-14 (Player address UX):** **Both addresses shown in dashboard** — Server Manager list and Server Details page show BOTH when applicable. `<server>.play.esluce.net` is the primary (always-on, stable) and `<server>.play.esluce.com` is the secondary (Direct only). Default "Copy join address" button copies the relay one. Shareable "Invite friends" UI shows both addresses and a QR code with the relay address.

### Pricing & Access
- **D-15 (Relay access tier):** **Free for all users in initial rollout** — relay bandwidth and EC2 costs absorbed as platform cost. No billing integration in Phase 68. Future pricing (per-server relay minutes) is a follow-up; the auth/billing model is intentionally abstracted enough to add a paywall layer later (D-09/D-10 already gate access by node ownership, which is the prerequisite for tier-based access).

### Scope Confirmation
- **D-16 (Minecraft edition):** **Java TCP only in initial scope** — matches ROADMAP requirement 9. Bedrock UDP deferred (architecturally different: needs UDP tunnel support, yamux is TCP-only, would need a separate `quinn`-based path). Confirmed in scope: `mc-java-tcp`.
- **D-17 (AWS region):** **Single region (ap-southeast-1) + single AZ** — matches existing infra. Multi-region failover deferred; not on the Phase 68 critical path.

### Player Behavior
- **D-18 (Player connection failure):** **Clean socket close + DNS fallback by client** — when player connection arrives but no active tunnel exists for the requested `server_id`, relay closes the socket cleanly (no RST, no application-level error). Player's Minecraft client shows "Connection refused" and the user can try the other address (relay or direct) from the dashboard. No application-level "server offline" message at the relay layer.
- **D-19 (Connection keepalive):** **Transparent TCP forwarding, no special keepalive** — Minecraft client sends keepalive packets; yamux + TCP handle the rest. Relay does not inject keepalives or modify TCP options on forwarded streams. Idle connection timeout: 5 minutes (matches typical Minecraft idle).

### Security Hardening
- **D-20 (Rate limiting):** **100 connection attempts per source IP per minute** — implemented at the gateway as an in-process token bucket (per-IP, refilled at 100/60 per second). Applies to both player TCP connections and tunnel connect attempts. **Phase 68 scope is single-instance (D-05) so an in-process bucket is correct**; a future horizontal-scale phase must migrate this to a Redis-backed Lua counter for atomicity across instances.
- **D-21 (Tunnel rate limit):** **1 active tunnel per server_id** — when a new tunnel connect comes in for a server_id that already has an active tunnel, relay atomically replaces it. Race condition: the old tunnel's first missed heartbeat causes it to be marked stale; the new one is registered in parallel; no duplicate forwarding.

### Monitoring & Integration
- **D-22 (Metrics):** **Prometheus endpoint at `relay.esluce.net:9100/metrics`** — exposed metrics: `relay_active_tunnels_total`, `relay_bandwidth_in_bytes`, `relay_bandwidth_out_bytes`, `relay_players_per_tunnel`, `relay_reconnect_rate_5m`, `relay_latency_seconds{quantile="0.5|0.95|0.99"}`, `relay_errors_total{kind="handshake|timeout|rejected_lookup"}`, `relay_mode_distribution{kind="direct|relay|offline"}`. Backend monitoring service scrapes every 15s. Dashboard shows latency, bandwidth, mode distribution.
- **D-23 (Alerts):** **Reuse existing alert infrastructure** — new alert types: `tunnel_down` (sustained 1m), `bandwidth_spike` (>2x rolling 5m avg), `mode_flip_spike` (>10% servers flipped in 5m). Per-server Discord webhook reuse. Email alerts reuse Phase 25 transport.

### Backward Compatibility
- **D-24 (Existing `esluce.com` records):** **No migration** — all existing `<server>.play.esluce.com` A records (from Phase 51) continue to work. Agent updates them only when Direct Mode is active; otherwise they remain as the last known good value. Servers not yet on Phase 68 see no behavior change. Phase 68's relay path is purely additive (new zone + new gateway); it does not touch Phase 51's Cloudflare automation.

### the agent's Discretion
- Exact Caddy config for player TLS termination (cert paths, HSTS policy, cipher suites)
- Specific AWS instance type for the gateway (recommend c6i.large, scale up if >5k concurrent tunnels)
- EFS/FSx for relay state (not needed initially; all state in PostgreSQL + Redis)
- WebSocket frame max size and yamux window size (defaults: 16 KiB frames, 256 KiB window per stream)
- Heartbeat payload contents (recommend: timestamp + tunnel uptime + bytes transferred + active stream count)
- Exact reconnect jitter formula (recommend: `base * 2^attempt * (0.8 + rand() * 0.4)`)
- Specific Prometheus metric label values (already specified in D-22; agent chooses string IDs)
- ALB vs NLB decision for player traffic (recommend: NLB for raw TCP Minecraft Java; ALB only if we add HTTPS-based player endpoints later)
- Tunnel session rekeying cadence (recommend: every 24h or 100 GB transferred, whichever first)
- Whether to add HTTP/3 (QUIC) for the player→relay path in a follow-up (out of Phase 68 scope, but the architecture should not preclude it)

### Folded Todos
None — no todos matched Phase 68 in `todo.match-phase`.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and prior roadmap context
- `.planning/ROADMAP.md` § Phase 68 — Goal statement: "Implement Esluce Relay as the primary connectivity path for Minecraft servers, with relay-backed stable DNS on *.play.esluce.net and conditional Direct Mode fast-path on *.play.esluce.com"
- `.planning/PROJECT.md` — Project overview, core value, multi-node agent model. Phase 68's relay is the "always-on, stable" answer to the project's "users deploy on their own infra" constraint
- `.planning/REQUIREMENTS.md` — No direct Phase 68 requirements; relates to DEPLOY-01..05 (server lifecycle), STATUS-01..02, and the broader "no port forwarding needed" promise from STRATEGI.md
- `.planning/STATE.md` — Phase 67 planned 2026-06-07, Phase 68 follows

### Phase 67 carryforward (CRITICAL — phase 68 implements deferred Phase 67 decisions)
- `.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-CONTEXT.md` — D-10: relay explicitly deferred to "follow-up phase"; D-14: fallback order ends with Esluce Relay; D-13: failure report includes "Join Esluce Relay Waitlist"; D-15: Connectivity section UI; D-17: per-server audit log
- `.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-RESEARCH.md` — Reachability probe architecture (hybrid backend+agent) Phase 68's Direct Mode probe reuses
- `.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-PATTERNS.md` — Pattern map for Phase 67's WebSocket protocol extensions; Phase 68 reuses the same `NodeMessage` extension pattern for tunnel control messages
- `.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-UI-SPEC.md` — Per-server connectivity badge + Connectivity section UI; Phase 68 extends it with tunnel health + both addresses

### Phase 51 (Cloudflare DNS automation)
- `.planning/phases/51-automasi-dns-cloudflare/51-CONTEXT.md` — `*.esluce.com` wildcard, DDNS-like auto-refresh, agent config + dashboard hybrid flow, API token in DB
- `.planning/phases/51-automasi-dns-cloudflare/51-01-PLAN.md` — Per-server A record provisioning
- `.planning/phases/51-automasi-dns-cloudflare/51-02-PLAN.md` — DNS API client integration
- `.planning/phases/51-automasi-dns-cloudflare/51-03-PLAN.md` — Auto-refresh on IP change

### Phase 66 (AWS deployment pattern — same shape as Phase 68 gateway)
- `.planning/phases/66-integrasikan-umami-analitycs-dashboard-dengan-rds-di-project/66-CONTEXT.md` — EC2 + Docker + Caddy + RDS deployment pattern; same ops profile as the Phase 68 relay gateway

### Phase 65 (Installer pattern — opt-in consent)
- `.planning/phases/65-buat-installer-script-auto-install-docker-sebelum-install-so/65-CONTEXT.md` — D-13..D-16: install-time consent + config generation. Phase 68's relay_token persistence follows the same opt-in config pattern

### Existing agent networking code (integration points)
- `src/handlers/dns_watch.rs:18-80, 132-155` — `DnsWatcher` background task pattern + `detect_public_ip()` function. Phase 68 reuses the same `tokio::spawn` + interval pattern for the tunnel client
- `src/handlers/mod.rs:118-294` — `execute_single` task dispatch + `get_task_config`. New task types for tunnel control hook in here: `relay.connect`, `relay.disconnect`, `relay.heartbeat`
- `agent/agent-core/crates/agent-proto/src/task.rs` — `Task` / `TaskResult` types. Tunnel control tasks serialize as `serde_json::Value`
- `api/src/presentation/ws/node_protocol.rs:7-80` — Existing `NodeMessage` enum (Register, Heartbeat, CommandResponse, CrashReport). Phase 68 adds new variants: `TunnelConnect`, `TunnelDisconnect`, `TunnelHeartbeat`, `ModeOverrideChange`

### Backend WebSocket node protocol and persistence
- `api/src/presentation/handlers/node_ws_handler.rs:237-298` — Heartbeat handler + CommandResponse handler. New tunnel-related message types follow this pattern
- `api/migrations/` — Existing migration files. New migrations for Phase 68: `nodes.relay_token`, `servers.connectivity_mode_override`, `servers.relay_status`, `servers.last_tunnel_connected_at`, `relay_sessions` (active tunnel registry, ephemeral; Redis-backed instead)
- `api/src/domain/entities/server.rs:8-75` — New `Server` struct. Phase 68 adds fields for tunnel health + mode override
- `api/src/domain/entities/node.rs` (assumed to exist) — New `relay_token` column on `nodes` table
- `api/src/bootstrap/container.rs` — AppContainer DI. Add `RelayAuthClient` for the gateway's introspection calls

### Agent binary + build system
- `Cargo.toml` workspace — New deps: `yamux = "0.13"` (multiplexing), `tokio-tungstenite = "0.26"` (already in stack per INTEGRATIONS.md)
- `agent/solys/install.sh` — Installer. Phase 68 does not change install.sh; the relay token is generated at first agent connect, not at install

### Existing server model and DNS scaffolding
- `api/src/domain/server/model.rs:8-31` — OLD `Server` struct. Phase 68 uses the NEW `Server` struct (`api/src/domain/entities/server.rs:8-75`); fix the diverging models is a separate concern (already in Phase 67 deferred)
- `migration/20260307000001_add_enhanced_server_features.sql:9-10` — `enable_tailscale` + `tailscale_auth_key`. Phase 68 does NOT touch Tailscale (Phase 67 only)
- `api/src/handlers/dns.rs` — Existing Cloudflare DNS automation. Phase 68 extends to manage `esluce.com` (Direct Mode) but not `esluce.net` (Route 53)

### Existing alert infrastructure
- `api/src/domain/billing/webhooks.rs` — Webhook emission pattern. Phase 68 alerts reuse this
- `api/migrations/` (search for `discord_webhook_url`) — Per-server Discord webhook. Reused for relay alerts (D-23)
- `api/src/domain/usage/service.rs` — Existing metrics/usage tracking. Phase 68 Prometheus metrics follow the same emission pattern

### Strategy and debugging context
- `STRATEGI.md:23, 47, 143` — "No port forwarding" is a Tier 1 differentiator. Phase 68's relay IS the no-port-forwarding infrastructure — the ultimate solution to the strategy
- `.planning/debug/server-details-wrong-address-version-status.md` — Documents the two competing `Server` models and the missing `endpoints` column. Already deferred from Phase 67; Phase 68 does not depend on resolving this

### Codebase maps (tech context)
- `.planning/codebase/STACK.md` — Tech stack versions: Rust 2021, Axum 0.7, tokio 1, sqlx 0.7, PostgreSQL 16, Redis 7, tokio-tungstenite 0.26
- `.planning/codebase/STRUCTURE.md` — Directory layout: `src/handlers/`, `api/src/presentation/`, `api/migrations/`, `agent-core/crates/`
- `.planning/codebase/INTEGRATIONS.md` — WebSocket, PostgreSQL, Redis, Cloudflare DNS, AWS S3. Phase 68 adds: AWS Route 53, AWS ALB, AWS NLB, AWS EC2, yamux
- `.planning/codebase/ARCHITECTURE.md` — Microservices with node agents; agent → backend WebSocket outbound. Phase 68 adds a NEW component: relay gateway (separate service, not part of the existing API/Worker/Agent trio)
- `.planning/codebase/CONCERNS.md` — Known fragile areas: WebSocket connection management, server executor trait implementations. Phase 68's tunnel gateway is a new WebSocket-heavy service; same fragility applies. The yamux + tunnel reconnection logic is a new surface for this concern

### External service docs (for new AWS / yamux / tokio-tungstenite integrations)
- AWS Route 53 — DNS hosting for `esluce.net` zone
- AWS ALB / NLB — load balancing for player TCP traffic
- yamux 0.13 docs (https://github.com/hashicorp/yamux/blob/master/spec.md) — multiplexing protocol spec
- tokio-tungstenite 0.26 — WebSocket client/server (already used in the project)
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`DnsWatcher` (`src/handlers/dns_watch.rs:18-80`)** — Background watcher pattern (tokio::spawn with interval ticker). Phase 68's tunnel client reuses the same pattern with `tokio::time::interval(10s)` for heartbeat.
- **`detect_public_ip()` (`src/handlers/dns_watch.rs:132-155`)** — Fetches public IP from 4 fallback providers. NOT used in Phase 68 (the relay abstracts the public IP away), but available if needed for Direct Mode probes.
- **`CloudflareDnsConfig` + Cloudflare API client (`src/handlers/dns.rs`)** — Existing Cloudflare client. Phase 68 reuses for `esluce.com` Direct-Mode DNS lifecycle (NOT for `esluce.net`, which goes through Route 53).
- **`execute_single` task dispatch (`src/handlers/mod.rs:118-166`)** — Central task dispatcher. New tunnel control task types hook in here.
- **`audit::log_task_received/completed/failed` (`src/audit.rs`)** — Existing per-task audit pattern. Phase 68's per-tunnel session log mirrors this (but stored in Redis as ephemeral, not in DB).
- **`NodeMessage` WS enum (`api/src/presentation/ws/node_protocol.rs:7-80`)** — Extensible message protocol. New tunnel control variants follow the same `serde` derive pattern.
- **`AppContainer` DI (`api/src/bootstrap/container.rs`)** — Phase 68 adds `RelayAuthClient` (HTTP client to relay gateway) and `RelayMetricsCollector` (Prometheus scrape).
- **`Phase 67 NodeMessage extensions`** — Phase 67 already adds `ConnectivityReport`, `ConnectivityProbeRequest`, `ConnectivityProbeResult`. Phase 68 extends further with `TunnelConnect`, `TunnelDisconnect`, etc. Same pattern, same handler dispatch in `node_ws_handler.rs`.

### Established Patterns
- **Agent → backend via WebSocket (outbound only)** — Phase 68 follows the same pattern: agent opens outbound WebSocket to `relay.esluce.net` (different from the existing backend WS at `api.esluce.com`). Two WebSockets per agent: one to backend (control plane), one to relay (data plane).
- **Per-server / per-task task config** — `get_task_config` in `src/handlers/mod.rs:186` maps task_type → timeout/retries. New tunnel control tasks get entries here.
- **TLS 1.3 everywhere** — Already standard in the project (STRATEGI.md, all existing WS). Phase 68 reuses Caddy for relay gateway TLS termination.
- **AWS EC2 + Docker + Caddy deployment** — Phase 66's analytics dashboard on EC2. Phase 68's relay gateway follows the same deployment shape (different instance, different subdomain, different Caddy config).
- **Per-tenant ownership check at API boundary** — Phase 68's `RelayAuthClient` follows the same pattern: every call checks `node.owns(server_id)` via the backend before relay action.
- **Background tokio tasks with `tokio::spawn` + interval** — Standard pattern. Phase 68's heartbeat task, tunnel monitor, and reconnection loop all use this.
- **Redis as ephemeral state** — Already used for sessions, rate limit counters, etc. Phase 68's active tunnel registry, nonce dedup, and per-source-IP rate limit counters live in Redis (not DB).
- **Caddy for TLS termination** — Standard. Phase 68's Caddy handles `*.play.esluce.net` wildcard cert (Let's Encrypt DNS-01 challenge via Route 53 plugin).

### Integration Points
- **New WebSocket message types** in `api/src/presentation/ws/node_protocol.rs`: `TunnelConnect` (agent → backend, signals tunnel established), `TunnelDisconnect` (agent → backend, signals tunnel lost), `TunnelHeartbeat` (agent → backend, periodic), `ModeOverrideChange` (backend → agent, user pinned a mode). Dispatched in `api/src/presentation/handlers/node_ws_handler.rs` alongside the existing `Heartbeat` at lines 237-298.
- **New task types** dispatched in `src/handlers/mod.rs:118-166`: `relay.connect` (initiate tunnel), `relay.disconnect` (tear down tunnel), `relay.heartbeat` (send heartbeat), `relay.refresh_token` (rotate expired token). Each gets a `get_task_config` entry in `src/handlers/mod.rs:186-294`.
- **New migration** for `nodes.relay_token UUID NOT NULL UNIQUE`, generated on node registration.
- **New migration** to add `connectivity_mode_override` (nullable text), `relay_status` (text: connected/connecting/disconnected), `last_tunnel_connected_at` (timestamptz) to `servers` table.
- **New endpoint** `POST /api/v1/internal/relay/authorize` — internal-only, called by relay gateway. Validates `(relay_token, server_id)` and returns 200/403.
- **New endpoint** `POST /api/v1/servers/:id/connectivity/mode-override` — user-facing, sets `servers.connectivity_mode_override`. Auth required.
- **New service** `api/src/application/services/relay_service.rs` — proxy for relay gateway operations (mode overrides, tunnel health queries).
- **New service** in agent: `src/handlers/relay_client.rs` — manages the WebSocket tunnel connection, yamux stream multiplexing, heartbeat loop, reconnection.
- **New AWS infrastructure** (declarative, manual setup per Phase 66 D-06):
  - EC2 instance `c6i.large` running relay gateway
  - NLB in front of EC2 for raw TCP Minecraft Java traffic
  - ALB in front of EC2 for HTTPS WebSocket tunnel traffic
  - Route 53 hosted zone for `esluce.net`
  - Security group allowing inbound 443 (ALB), 25565 (NLB, for Minecraft Java), 22 (admin SSH via bastion)
  - IAM role for EC2 to call Route 53 (for the wildcard cert DNS-01 challenge)
- **Frontend additions** in `app/src/pages/servers/`:
  - Extend `ConnectivitySection.jsx` with tunnel health (latency, uptime, last heartbeat)
  - Add "Mode override" dropdown per server (Auto / Force Direct / Force Relay)
  - Add "Copy join address" buttons (relay first, direct second)
  - Add "Invite friends" share UI with QR code
- **Existing alert webhooks** (`discord_webhook_url` + email) — Phase 68's tunnel alerts reuse the same dispatch path.
- **Bollard client re-usage** — Phase 68 does NOT use bollard directly. The relay gateway is a network service, not a Docker orchestrator.
- **Caddy + Route 53 plugin** — for the `*.play.esluce.net` wildcard cert via Let's Encrypt DNS-01.
</code_context>

<specifics>
## Specific Ideas

### Long-term vision (from STRATEGI.md:23, 47, 143)
Phase 68 IS the "no port forwarding needed" promise made real. The user has talked about this in many prior phases; Phase 68 is where the infrastructure lands. Relay becomes the default; Direct Mode is an optimization for users who happen to have working port forwarding.

### From the user (per Phase 67 D-10, D-13, D-14 carryforward)
- Relay is the **last resort** in Phase 67's fallback chain (after Direct/UPnP/Firewall/Tailscale/Cloudflare)
- Phase 67's "hybrid failure report" includes "Join Esluce Relay Waitlist" as a CGNAT user's option — Phase 68 turns the waitlist into a real product
- Phase 67's mode selection placeholder ("Direct/Relay/Offline") becomes functional in Phase 68

### Direct Mode vs Relay Mode flow (Phase 68 implements this end-to-end)
```
Server Start
  ↓
Phase 67: Reachability Check (Direct probe from backend)
  ↓
Direct probe passes + low latency?
├─ Yes → Direct Mode
│         • Emit <server>.play.esluce.com A record (Cloudflare)
│         • Mode = Direct
│         • Agent opens tunnel to relay.esluce.net (for fallback)
│         • Player can use either address
└─ No  → Relay Mode
          • Don't emit <server>.play.esluce.com (or remove if present)
          • Mode = Relay
          • Agent opens tunnel to relay.esluce.net
          • Player must use <server>.play.esluce.net
```

### Agent tunnel lifecycle (illustrative)
```
1. Agent receives `relay.connect` task from backend
2. Agent dials wss://relay.esluce.net/tunnel
3. WebSocket upgrade headers: Authorization: Bearer <relay_token>, X-Relay-Nonce: <32 bytes>, X-Relay-Timestamp: <unix>, X-Server-Id: <uuid>
4. Relay validates (D-09, D-10, D-11)
5. On accept: yamux session starts. Agent registers all servers it owns (one yamux stream per server ready to receive player connections)
6. Heartbeat every 10s: timestamp, uptime, bytes_in, bytes_out, active_streams
7. On player TCP connection arriving at relay for <server>.play.esluce.net:
   a. Relay looks up server_id from Host header
   b. Relay opens new yamux stream to that server's agent
   c. Bidirectional copy between player TCP and yamux stream
8. On agent disconnect: yamux session ends, all streams close, player connections see "Connection reset"
9. Agent reconnects with backoff (D-04); relay re-validates; new session replaces old atomically
```

### Per-tunnel session log shape (Redis-backed, ephemeral)
```
[sess-a1b2c3] 2026-06-07T10:00:00Z connected: agent=node-uuid, server=server-uuid, public_ip=47.129.171.64
[sess-a1b2c3] 2026-06-07T10:00:05Z heartbeat: uptime=5s, bytes_in=0, bytes_out=0, active_streams=0
[sess-a1b2c3] 2026-06-07T10:00:12Z player_connected: player_ip=203.0.113.42, stream_id=1
[sess-a1b2c3] 2026-06-07T10:00:18Z player_disconnected: stream_id=1, duration=6s, bytes_in=2048, bytes_out=1024
[sess-a1b2c3] 2026-06-07T10:30:00Z heartbeat_timeout: 3 missed
[sess-a1b2c3] 2026-06-07T10:30:00Z disconnected: reason=heartbeat_timeout, duration=30m
```

### User-facing "Invite friends" UI (Phase 68 extends Phase 67's Connectivity section)
```
┌─────────────────────────────────────────────────────────────┐
│ Invite friends to "My Minecraft"                            │
│                                                             │
│ Always-on address (works everywhere):                       │
│   mc.play.esluce.net    [Copy]  [QR]                        │
│                                                             │
│ Direct address (only when port forwarding works):           │
│   mc.play.esluce.com    [Copy]  [QR]                        │
│                                                             │
│ Mode: Relay (default)                                       │
│   ○ Auto (let Esluce pick the best)                         │
│   ● Force Relay (always use relay)                          │
│   ○ Force Direct (only use direct, fail if unavailable)     │
│                                                             │
│ Tunnel health:                                              │
│   Latency: 23ms (relay round-trip)                          │
│   Last heartbeat: 4 seconds ago                             │
│   Connection duration: 2h 14m                               │
└─────────────────────────────────────────────────────────────┘
```

### AWS deployment shape (declarative, manual setup per Phase 66 D-06)
- 1× EC2 `c6i.large` (ap-southeast-1a), 100 GB gp3 EBS, IAM role for Route 53
- 1× NLB (TCP 25565 → EC2:25565) for Minecraft Java player traffic
- 1× ALB (HTTPS 443 → EC2:8443) for WebSocket tunnel traffic
- 1× Route 53 hosted zone for `esluce.net`, wildcard `*.play.esluce.net` → NLB
- 1× Caddy on EC2, terminates TLS for ALB → Caddy → relay-gateway (Rust)
- Security group: 443 (ALB), 25565 (NLB), 22 (SSH via bastion)
- No DB on the relay gateway; all state in PostgreSQL (backend) + Redis (shared)

### Connection flow (relay-primary)
```
Player's Minecraft Client
  ↓ TCP to mc.play.esluce.net:25565
DNS: *.play.esluce.net → NLB IP
  ↓
NLB → EC2:25565
  ↓
relay-gateway (Rust) receives raw TCP
  ↓ looks up server_id from SNI/Host header (or proxy_protocol)
  ↓ queries Redis for active tunnel
  ↓ opens yamux stream to that server's agent
  ↓
bidirectional copy_bidi between player TCP and yamux stream
  ↓
agent's tunnel client receives yamux stream
  ↓ opens TCP connection to localhost:25565 (Minecraft server)
  ↓
bidirectional copy_bidi between yamux stream and local TCP
  ↓
Minecraft server processes player packets
```
</specifics>

<deferred>
## Deferred Ideas

### Bedrock Edition UDP support
Phase 68 is Java TCP only. Bedrock uses UDP (RakNet protocol) and needs a separate path: yamux is TCP-only, would need to add `quinn` (Rust QUIC) or a UDP-aware multiplexer. Architecturally different. Belongs in a follow-up phase ("Phase 69: Bedrock Relay Support" or similar).

### Multi-region relay failover
Phase 68 is single-region (ap-southeast-1, single AZ). For global latency and HA, multi-region with geo-DNS is a follow-up. Requires: per-region relay gateways, geo-routing in Route 53, cross-region tunnel registry replication, mode selection that considers closest region.

### IPv6 dual-stack relay
Phase 68 is IPv4 only (matches existing `0.0.0.0` binding in agent). IPv6 path requires IPv6 DNS records, IPv6-capable ALB/NLB, IPv6 yamux listeners. Defer until IPv6 reachability is a user-reported issue.

### HTTP/3 (QUIC) player transport
Minecraft Java is TCP-only; HTTP/3 (QUIC) does not apply to player traffic. But the agent→relay tunnel COULD use QUIC for better mobile-node performance. Defer; the current WebSocket-over-TLS path is already performant.

### Per-server relay pricing tier
Phase 68 is free for all users. Future paywall (per-server relay minutes, free tier of N hours, etc.) requires: per-server usage metering, billing integration, paywall enforcement at `RelayAuthClient`. The architecture in D-09/D-10 is intentionally abstracted enough to add a tier check later.

### Custom Esluce Relay agent binary
The relay tunnel logic is added to the existing agent binary as new task types. A separate "relay agent" binary is not needed. If we ever want a lightweight relay-only agent for headless nodes, that's a follow-up.

### IPv6-only nodes
Currently the agent uses `local-ip-address` which prefers IPv4. Dual-stack / IPv6-only node support is a separate concern.

### Migrating Phase 51's `esluce.com` zone to Route 53
Phase 68 ADDS `esluce.net` (new zone on Route 53) but keeps `esluce.com` on Cloudflare. Migrating `esluce.com` to Route 53 is a separate decision (tradeoffs: unified DNS vs Cloudflare's DDoS protection + CDN). Out of scope.

### Re-architecting the two competing Server models
Already deferred from Phase 67. Phase 68 uses the NEW `Server` struct from `api/src/domain/entities/server.rs`. The OLD `Server` in `model.rs` is still used by the dashboard endpoint; this divergence is pre-existing tech debt unrelated to Phase 68.
</deferred>

---

*Phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela*
*Context gathered: 2026-06-07 via yolo-autonomous mode (rich ROADMAP + Phase 67 carryforward enabled single-pass; user can edit CONTEXT.md before /gsd-plan-phase 68)*
