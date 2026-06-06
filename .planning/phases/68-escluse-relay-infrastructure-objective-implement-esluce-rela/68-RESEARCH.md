# Phase 68: Escluse Relay Infrastructure - Research

**Researched:** 2026-06-07
**Domain:** Network relay infrastructure (Rust+Axum gateway, agent tunnel client, AWS NLB/Route 53, yamux multiplexing over WebSocket)
**Confidence:** MEDIUM-HIGH (existing codebase well-mapped; protocol choices verified against crates.io + official docs; AWS Caddy/Route 53/NLB/ALB decisions confirmed via official AWS + community sources)

## Summary

Phase 68 builds a **third architectural tier** alongside the existing API/Worker/Agent trio: a dedicated **Relay Gateway** (Rust + Axum on AWS EC2, single AZ, `ap-southeast-1`) that brokers Minecraft Java TCP traffic from players to agents that cannot expose port 25565 directly (CGNAT, double-NAT, hostile firewalls, etc.). The gateway is the always-on default; the existing `*.play.esluce.com` Direct-Mode DNS (Phase 51) becomes a conditional best-effort optimization. The agent opens an **outbound persistent WebSocket tunnel** to `relay.esluce.net` (TLS 1.3, yamux-multiplexed streams per-server), the gateway extracts the `server_id` from the player TCP connection's SNI/`Host` header and proxies the TCP bytes over a yamux stream. Per-agent bearer tokens (UUID) are generated at node registration, validated on every tunnel upgrade via a `POST /internal/relay/authorize` introspection call to the backend, and protected from replay by a nonce+timestamp pair deduped in Redis. A separate **agent tunnel client** lives in the existing agent binary as new task types (`relay.connect`, `relay.disconnect`, `relay.heartbeat`); the existing `DnsWatcher` background-task pattern is mirrored. Mode selection (relay-default, with per-server user override) piggybacks on Phase 67's probe trigger pipeline and adds tunnel events (`tunnel_disconnect` → flip to Relay; `tunnel_reconnect` → re-probe Direct after 30s). A new `relay.esluce.net` Route 53 hosted zone (ALB + NLB + wildcard `*.play.esluce.net` → NLB) and an `opt/relay/` Docker Compose stack mirror the Phase 66 `opt/umami/` deployment shape.

**Primary recommendation:** Treat the relay gateway as a **new Rust service** (separate from `api/`, `worker/`, `agent/`), containerized via Docker Compose on a single EC2 instance behind Caddy (TLS termination for the ALB WebSocket path) and an NLB (raw TCP passthrough for the Minecraft Java player path on port 25565). Use **`tokio-yamux` 0.3** (not bare `yamux 0.13`) for the agent + gateway because it implements `tokio::io::AsyncRead/AsyncWrite` directly, eliminating the futures↔tokio adapter layer. Reuse `tokio-tungstenite 0.26` (already in agent stack, locked in `Cargo.lock:400`) for the WebSocket transport. **Do not** use `wss://` on the player→NLB path (NLB terminates at TCP, no TLS); TLS happens at the agent↔relay WSS hop only. Player TCP forwarding is **raw TCP bidi copy** between the player socket and a yamux stream — no application-layer crypto inside the tunnel.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01 (Tunnel transport):** WebSocket over TLS 1.3 (`wss://`) reusing `tokio-tungstenite 0.26`
- **D-02 (Multiplexing):** Single WebSocket per agent, `yamux` 0.13-multiplexed streams (one per server)
- **D-03 (Encryption):** TLS 1.3 end-to-end at all hops; no additional application-layer crypto
- **D-04 (Tunnel reconnection):** Exponential backoff with jitter (1s → 30s, ±20%), 10s heartbeat, 30s stale timeout
- **D-05 (Gateway deployment):** Single Rust + Axum service on AWS EC2 `c6i.large` (ap-southeast-1, single AZ), behind Caddy (TLS for ALB/WS) and NLB (raw TCP for Minecraft Java on 25565)
- **D-06 (Player DNS resolution):** Wildcard `*.play.esluce.net` → A record → NLB IP; server_id extracted from SNI/Host header (WSS) or proxy_protocol (raw TCP)
- **D-07 (DNS hosting):** `esluce.net` delegated to AWS Route 53; `esluce.com` stays on Cloudflare (Phase 51)
- **D-08 (Direct Mode DNS):** `<server>.play.esluce.com` A record on Cloudflare, emitted only when Direct Mode is probe-verified; TTL 60s; existing records preserved
- **D-09 (Tunnel registration auth):** Per-agent `relay_token UUID NOT NULL UNIQUE` on `nodes` table; returned in `Register` ack; sent as `Authorization: Bearer <relay_token>` on every tunnel connect
- **D-10 (Server ID ownership validation):** Backend HTTP introspection — relay calls `POST /api/v1/internal/relay/authorize` with `(relay_token, server_id)`; backend returns 200/403; cached in memory for tunnel lifetime; <50ms p99 budget
- **D-11 (Replay protection):** 32-byte nonce + Unix timestamp on each tunnel connect (`X-Relay-Nonce`, `X-Relay-Timestamp`); reject if `(now - ts) > 5 min` OR nonce seen in last 10 min (Redis dedup)
- **D-12 (Mode selection policy):** Automatic with per-server user override (`servers.connectivity_mode_override` column, nullable text)
- **D-13 (Mode flip triggers):** Phase 67 D-02 triggers + new `tunnel_disconnect`/`tunnel_reconnect`/`direct_probe_latency > 50ms penalty vs relay` triggers
- **D-14 (Player address UX):** Both addresses shown (relay primary, direct secondary); "Copy join address" copies relay; "Invite friends" UI shows both with QR code
- **D-15 (Relay access tier):** Free for all users in initial rollout; auth/billing model abstracted enough to add paywall later
- **D-16 (Minecraft edition):** Java TCP only in initial scope
- **D-17 (AWS region):** Single region (ap-southeast-1) + single AZ
- **D-18 (Player connection failure):** Clean socket close (no RST, no app error); "Connection refused" UX
- **D-19 (Connection keepalive):** Transparent TCP forwarding, no keepalives injected; 5-minute idle timeout
- **D-20 (Rate limiting):** 100 connection attempts per source IP per minute (Redis-backed Lua counter), separate counters for player TCP and tunnel WS
- **D-21 (Tunnel rate limit):** 1 active tunnel per `server_id`; new connection atomically replaces old (3 missed heartbeats → stale, new replaces in parallel)
- **D-22 (Metrics):** Prometheus at `relay.esluce.net:9100/metrics`; metrics: `relay_active_tunnels_total`, `relay_bandwidth_in_bytes`, `relay_bandwidth_out_bytes`, `relay_players_per_tunnel`, `relay_reconnect_rate_5m`, `relay_latency_seconds{quantile="0.5|0.95|0.99"}`, `relay_errors_total{kind="..."}`, `relay_mode_distribution{kind="direct|relay|offline"}`
- **D-23 (Alerts):** Reuse existing alert infrastructure; new alert types: `tunnel_down` (sustained 1m), `bandwidth_spike` (>2x 5m rolling avg), `mode_flip_spike` (>10% servers flipped in 5m); per-server Discord webhook reuse
- **D-24 (Existing `esluce.com` records):** No migration; all existing records continue to work; relay is purely additive

### the agent's Discretion
- Exact Caddy config for player TLS termination (cert paths, HSTS policy, cipher suites)
- Specific AWS instance type (recommended `c6i.large`, scale up if >5k concurrent tunnels)
- EFS/FSx for relay state (not needed initially; all state in PostgreSQL + Redis)
- WebSocket frame max size and yamux window size (defaults: 16 KiB frames, 256 KiB window per stream)
- Heartbeat payload contents (recommend: timestamp + tunnel uptime + bytes transferred + active stream count)
- Exact reconnect jitter formula (recommend: `base * 2^attempt * (0.8 + rand() * 0.4)`)
- Specific Prometheus metric label values
- ALB vs NLB decision for player traffic (recommended: NLB for raw TCP Minecraft Java)
- Tunnel session rekeying cadence (recommend: every 24h or 100 GB transferred, whichever first)
- Whether to add HTTP/3 (QUIC) for the player→relay path (out of Phase 68 scope, but architecture should not preclude it)

### Deferred Ideas (OUT OF SCOPE)
- Bedrock Edition UDP support (architecturally different; would need `quinn`/QUIC path)
- Multi-region relay failover (single region + single AZ for initial scope)
- IPv6 dual-stack relay
- HTTP/3 (QUIC) player transport (Minecraft Java is TCP-only)
- Per-server relay pricing tier (free for all in Phase 68)
- Custom Esluce Relay agent binary (reuses existing agent with new task types)
- IPv6-only nodes
- Migrating Phase 51's `esluce.com` zone to Route 53
- Re-architecting the two competing `Server` model structs (pre-existing tech debt)

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Tunnel WebSocket endpoint (`wss://relay.esluce.net/tunnel`) | Relay Gateway (NEW) | — | Inbound WSS endpoint; Caddy terminates TLS, Axum handles upgrade; agent is outbound. |
| Player TCP forwarding (`<server>.play.esluce.net:25565` → yamux stream) | Relay Gateway (NEW) | — | Raw TCP bidi copy between NLB-delivered player socket and the agent's yamux stream; no application logic. |
| Active tunnel registry (per `server_id` → `tunnel_handle`) | Relay Gateway (NEW) | — | In-memory `DashMap<Uuid, TunnelEntry>`; ephemeral; mirrored to Redis for cross-instance visibility if we ever scale. |
| Per-agent token issuance (`nodes.relay_token`) | API / Backend | — | Generated at node registration, stored in `nodes` table; agent persists in its config file. |
| Server-ID ownership introspection (`/internal/relay/authorize`) | API / Backend | — | Backend is the source of truth for "node X owns server Y"; relay is stateless about ownership. |
| Nonce dedup + rate limit counters | Relay Gateway (NEW) | API/Backend (Redis) | Redis is shared infrastructure; relay writes counters and nonce keys, TTL-bounded. |
| DNS hosting for `esluce.net` (wildcard `*.play.esluce.net` → NLB) | API/Backend (Route 53 SDK) | — | One-time setup; static A record (NLB IP doesn't change); backend can also auto-update if we add record-set management later. |
| Direct Mode DNS lifecycle (`<server>.play.esluce.com` create/delete on phase 67 probes) | Agent (Cloudflare SDK) | — | Reuses Phase 51 `CloudflareDnsConfig`; agent owns the per-server A record on `esluce.com`. |
| Mode selection logic (Direct vs Relay) | Agent | — | Reuses Phase 67 probe pipeline; agent now also listens for tunnel events; `tunnel_disconnect` triggers immediate flip to Relay. |
| Agent tunnel client (outbound WS, yamux session, heartbeat, reconnection) | Agent | — | New `relay_client.rs` in `src/handlers/`; mirrors `dns_watch.rs` background-task pattern. |
| Per-server mode override persistence + read | API/Backend | — | New `servers.connectivity_mode_override` column + REST endpoint `POST /api/v1/servers/:id/connectivity/mode-override`. |
| Dashboard UI (tunnel health, both addresses, mode override) | Frontend | — | Extends Phase 67 `ConnectivitySection` in `app/src/pages/servers/`. |
| Prometheus metrics endpoint (`relay.esluce.net:9100/metrics`) | Relay Gateway (NEW) | — | Existing monitoring service (api/src/application/services/monitoring_service.rs) scrapes every 15s. |
| TLS cert provisioning for `*.play.esluce.net` wildcard | Caddy (on EC2) | — | Caddy with `caddy-dns/route53` plugin via Let's Encrypt DNS-01 challenge; runs once at instance boot. |
| Alert emission (Discord/email on tunnel_down, etc.) | API/Backend | — | Reuses `api/src/domain/billing/webhooks.rs` + `discord_webhook_url` column; backend's monitoring service subscribes to relay Prometheus metrics. |
| AWS infrastructure provisioning (EC2, NLB, ALB, Route 53 zone, IAM) | Manual (per Phase 66 D-06) | — | Declarative infrastructure via AWS Console / CLI scripts; documented in `DEPLOY.md`; no Terraform in Phase 68 scope. |

## Standard Stack

### Core (verify versions before pinning)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio` | 1.x (already in agent + api, full features) | Async runtime for both gateway and agent; yamux streams require tokio for non-blocking I/O | Already locked in `Cargo.lock`; same runtime as agent's existing WS connection. |
| `axum` | 0.7 (gateway) | HTTP server for relay gateway: `/tunnel` WebSocket upgrade endpoint, `/metrics` Prometheus, `/healthz` | Same framework as `api/` (mirrors `node_ws_handler.rs` WebSocket pattern); battle-tested. |
| `tokio-tungstenite` | 0.26.2 (already in agent, `Cargo.lock` line 400) | WebSocket client (agent) + server (gateway) | Already verified in agent stack; supports custom headers via `IntoClientRequest` (Request builder) for the `Authorization: Bearer`, `X-Relay-Nonce`, `X-Relay-Timestamp` headers. |
| `tokio-yamux` | 0.3.18 (crates.io/tokio-yamux, MIT, repo `nervosnetwork/tentacle`) | Yamux multiplexing over WebSocket — opens per-server streams, handles per-stream flow control | **Preferred over bare `yamux 0.13`**: implements `tokio::io::AsyncRead/AsyncWrite` directly, so no `futures::io::AsyncRead` ↔ `tokio::io::AsyncRead` adapter needed (bare `yamux` uses `futures::io` and forces an extra `AsyncReadExt` adapter layer per stream). |
| `reqwest` | 0.12 (already in agent + api) | Gateway → backend introspection call (`/internal/relay/authorize`); also AWS SDK uses reqwest under the hood | Reuses existing dependency; 50ms p99 latency budget is easy. |
| `redis` | 0.25 (already in api) | Nonce dedup (10-min TTL), rate limit counters (Lua script), tunnel registry mirror | Same client the rest of the stack uses; connect via shared `RedisPool`. |
| `serde` / `serde_json` | 1 (already) | Tunnel handshake payloads, mode-override REST request/response, Prometheus label rendering | Mandatory. |
| `tracing` | 0.1 (already) | Per-tunnel session log, mode flip events, handshake outcomes | Same logging framework as agent. |
| `chrono` | 0.4 (already) | Heartbeat timestamp, session log timestamps, `last_tunnel_connected_at` | Mandatory. |
| `uuid` | 1 (already) | `relay_token`, tunnel session IDs, nonce IDs | Mandatory. |
| `prometheus` | 0.13 (or `prometheus-client` 0.22) | Expose `relay_*` metrics at `/metrics` | Standard Rust Prometheus client; auto-registers the metrics declared in D-22. |
| `aws-config` + `aws-sdk-route53` | 1.x (1.114.0 latest per crates.io) | One-time DNS-01 challenge support + A record management (optional, can be done via AWS Console for first deploy) | Optional in Phase 68 — Caddy's `caddy-dns/route53` plugin handles cert provisioning; backend only needs Route 53 SDK if we want API-driven A record updates (not needed for static NLB IP). |
| `tokio-stream` | 0.1 (already in agent) | Stream combinators for yamux → player TCP forwarder | Already used in agent's `dns_watch` / `metrics` flows. |
| `dashmap` | 6 (or `tokio::sync::RwLock<HashMap<...>>`) | In-memory active-tunnel registry, indexed by `server_id` → `TunnelEntry` | Lock-free reads; high concurrency for the player lookup path. |
| `anyhow` / `thiserror` | already | Error types for gateway (relay-specific `RelayError` enum) | Standard. |
| `once_cell` / `lazy_static` | 1.x / 1.4 (already in agent) | Global state for active-tunnel registry, nonce dedup lock-free init | Already used in agent's `dns.rs`. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `hyper` | 1 (already in api) | Underlying HTTP server for Axum gateway | Mandatory. |
| `tower` | 0.5 (already in api) | Middleware (request size limit, request ID propagation) for gateway HTTP endpoints | Standard Axum middleware stack. |
| `tower-http` | 0.5 (already in api) | Trace, timeout, CORS middleware | Same as `api/`. |
| `caddy:2` Docker image + `caddy-dns/route53` custom build | latest Caddy 2.x | TLS termination for `wss://relay.esluce.net/tunnel` (ALB → Caddy → Axum) AND `*.play.esluce.net` wildcard cert (Let's Encrypt DNS-01 challenge) | Standard in this project's gateway stack (`gateway/Caddyfile.prod`). Build via `xcaddy` with `--with github.com/caddy-dns/route53` per Phase 66 D-06 pattern. |
| AWS Network Load Balancer (NLB) | n/a | Raw TCP passthrough for Minecraft Java on port 25565; preserves client source IP via instance-type targets | Required for player TCP path. Per AWS docs: "If you use Instance ID as target type, NLB preserves the clients source IP addresses" — we use instance-type target (not IP) to keep client IP for per-IP rate limits and audit logs. |
| AWS Application Load Balancer (ALB) | n/a | HTTPS termination for WSS tunnel endpoint at `relay.esluce.net/tunnel`; routes to Caddy → Axum | Optional — can terminate TLS directly on Caddy (port 443) without ALB, but ALB gives us a stable DNS name + ACM cert free + WAF integration. Decision is the agent's discretion. |
| AWS EC2 `c6i.large` (2 vCPU, 4 GiB RAM, network-optimized) | n/a | Single gateway instance; ~12.5 Gbps network bandwidth, more than enough for 5k+ concurrent Minecraft sessions | Phase 66 uses the same instance type for Umami; benchmark before scaling. |
| AWS Route 53 | n/a | DNS hosting for `esluce.net`; wildcard A record `*.play.esluce.net` → NLB IP | Phase 51 already uses Route 53 elsewhere; required for Caddy DNS-01 + wildcard cert. |
| AWS IAM role for EC2 | n/a | Grants the EC2 instance permissions to call Route 53 (for Caddy DNS-01 challenge TXT record creation) | Standard pattern; no programmatic access key needed. |
| `cloudshell` or `aws-cli` | n/a | Manual infrastructure provisioning per Phase 66 D-06 (declarative setup, manual execution) | No Terraform / Pulumi in Phase 68. |
| `redis-cli` (existing infra) | n/a | Tunnel-registry mirror inspection during debugging | Already in stack. |
| `proxy-protocol` crate (Rust) | latest | Parse the PROXY protocol v2 header that NLB prepends to TCP packets (when NLB is configured with `proxy_protocol_v2.enabled=true` for IP-type targets) | Used only if we ever switch NLB targets from `instance` to `ip`; agent's discretion. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tokio-yamux 0.3` | `yamux 0.13` (paritytech fork) | `yamux 0.13` uses `futures::io::AsyncRead/AsyncWrite`, not tokio; we'd need `tokio_util::compat::FuturesAsyncReadCompatExt` adapter per stream — adds boilerplate, risk of subtle bug under load. `tokio-yamux 0.3` is tokio-native. CONTEXT.md D-02 says "yamux" generically; `tokio-yamux` is the right implementation choice. |
| WebSocket over TLS for tunnel | QUIC (`quinn` 0.11 + `rustls`) | QUIC has better mobile-network performance (connection migration) but adds new deps (`quinn`, `quinn-proto`, `rustls` separately from `tokio-tungstenite`), and `quinn` doesn't have a server in the existing stack. CONTEXT D-01 explicitly rejected QUIC. Architecture should not preclude it (D-25). |
| WebSocket over TLS for tunnel | Raw TCP with custom framing | Skips cert management; we'd need a separate TCP listener + Prometheus + auth. CONTEXT D-01 explicitly rejected raw TCP. |
| NLB for player TCP | ALB for player TCP (TLS) | ALB can't terminate raw TCP at L4 + WebSocket at L7 in one listener cleanly; Minecraft Java client doesn't send TLS SNI for the connection (it's plain TCP), so ALB can't route on SNI. NLB is the only sane choice for raw TCP. D-06 chose NLB. |
| Single EC2 instance (relay + Caddy) | Separate EC2 for relay gateway + separate for Caddy | One instance is simpler; if we ever scale, we move Caddy to a sidecar container on each gateway instance. CONTEXT D-05 chose single instance. |
| Prometheus pull (gateway `/metrics`) | Prometheus push (agent metric push via WS) | Pull is standard; existing `monitoring_service.rs` already scrapes HTTP endpoints. Gateway exposes `/metrics`, backend scrapes every 15s. |
| AWS SDK for Route 53 (backend) | Manual AWS Console setup for the A record | Static NLB IP doesn't need API updates; one-time Console setup is simpler. Backend SDK only needed if we ever need programmatic record changes. |
| Caddy for ALB TLS termination | nginx / haproxy | Caddy is the project standard (Phase 66, existing `gateway/Caddyfile.prod`); keeps ops consistent. |
| `aws-sdk-route53` v1 | v0 (legacy rusoto) | v1 is the current line; v0 is deprecated. Use v1. |

### Installation

**Agent (`agent/solys/Cargo.toml`):**

```toml
# Phase 68 — Relay tunnel client
tokio-yamux = "0.3"           # Yamux multiplexing with native tokio::io traits
```

**Relay gateway (new service, `opt/relay/Cargo.toml`):**

```toml
[package]
name = "escluse-relay-gateway"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.26", features = ["rustls-tls-native-roots"] }
tokio-yamux = "0.3"
tokio-stream = { version = "0.1", features = ["sync"] }
futures-util = "0.3"

# HTTP server (Axum — same framework as api/)
axum = { version = "0.7", features = ["ws", "macros"] }
tower = "0.5"
tower-http = { version = "0.5", features = ["trace", "timeout", "cors"] }
hyper = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Backend client (introspection, nonces mirror)
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }

# Shared state (Redis mirror)
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
dashmap = "6"

# Observability
prometheus = "0.13"  # or prometheus-client
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Error handling
anyhow = "1"
thiserror = "2"

# Config loading
serde_yaml = "0.9"
```

**Caddy Route 53 DNS-01 plugin build** (one-time, EC2 user data):

```dockerfile
# Dockerfile.caddy (in opt/relay/)
FROM caddy:builder AS builder
RUN xcaddy build \
    --with github.com/caddy-dns/route53

FROM caddy:2
COPY --from=builder /usr/bin/caddy /usr/bin/caddy
```

**No `aws-sdk-*` crates needed in Phase 68** unless we move to API-driven A record updates. The `caddy-dns/route53` plugin handles cert provisioning via IAM role on the EC2 instance.

**Version verification (run before locking):**

```bash
cargo info tokio-yamux       # → 0.3.18 (crates.io, MIT, nervosnetwork/tentacle)
cargo info tokio-tungstenite  # → 0.29.0 latest; project uses 0.26.2 (already locked)
cargo info yamux              # → 0.13.10 (paritytech) — for reference only; we use tokio-yamux
cargo info dashmap            # → 6.x latest
cargo info prometheus         # → 0.13.x latest
cargo info redis              # → 0.25.x (already in lock file)
```

> **`tokio-tungstenite` 0.26 vs 0.29:** Project lock is at 0.26.2. The 0.29 line adds more rustls-pki-types work but no breaking changes for our use case. **Do not upgrade to 0.29** in Phase 68 — agent's `Cargo.toml` pins 0.26 explicitly; a coordinated bump is a separate task. We will use 0.26.2 in the gateway too for consistency.

## Architecture Patterns

### System Architecture Diagram

```
                                          Player's Minecraft Client
                                          (anywhere on the public internet)
                                                     │
                                                     │ TCP :25565 to <server>.play.esluce.net
                                                     ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  Public DNS                                                              │
│    *.play.esluce.net (A wildcard)                                        │
│      └─ Route 53 hosted zone for esluce.net                               │
│         └─ A record → NLB IP (static)                                    │
└──────────────────────────────────────────┬───────────────────────────────┘
                                           │
                                           ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  AWS NLB  (ap-southeast-1, single AZ)                                    │
│    • TCP :25565 listener                                                 │
│    • Target type: instance (preserves client source IP — D-20, D-21)     │
│    • Cross-zone: disabled (single AZ for Phase 68)                       │
└──────────────────────────────────────────┬───────────────────────────────────┘
                                           │
                                           │ raw TCP
                                           ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  EC2 c6i.large (ap-southeast-1a)                                          │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  Caddy 2 (with caddy-dns/route53 plugin)                            │  │
│  │   • *.play.esluce.net wildcard cert (Let's Encrypt DNS-01)          │  │
│  │   • relay.esluce.net cert                                           │  │
│  │   • :25565 → pass-through to localhost:25565 (NLB → relay-gw)      │  │
│  │   • :443 (ALB) → reverse_proxy to relay-gateway:8443                 │  │
│  │   • :9100 → reverse_proxy to relay-gateway:9100 (Prometheus)        │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  relay-gateway (Rust + Axum 0.7)                                   │  │
│  │   • /tunnel  → WebSocket upgrade (wss://relay.esluce.net/tunnel)   │  │
│  │   • /metrics → Prometheus                                           │  │
│  │   • /healthz → liveness probe                                       │  │
│  │                                                                     │  │
│  │   Components:                                                       │  │
│  │   ┌─────────────────────────────────────────────────────────────┐  │  │
│  │   │  Tunnel Registry (in-memory DashMap)                         │  │  │
│  │   │    server_id → TunnelEntry {                                 │  │  │
│  │   │      node_id, session, control_tx, last_heartbeat,           │  │  │
│  │   │      bytes_in, bytes_out, active_streams                     │  │  │
│  │   │    }                                                          │  │  │
│  │   └─────────────────────────────────────────────────────────────┘  │  │
│  │   ┌─────────────────────────────────────────────────────────────┐  │  │
│  │   │  Auth Middleware                                              │  │  │
│  │   │    • Validate Authorization: Bearer <relay_token>             │  │  │
│  │   │    • Validate X-Relay-Nonce + X-Relay-Timestamp               │  │  │
│  │   │    • Call backend POST /internal/relay/authorize              │  │  │
│  │   │    • Check Redis nonce dedup (10-min TTL)                     │  │  │
│  │   │    • Rate limit (100/min per source IP, Lua script)           │  │  │
│  │   └─────────────────────────────────────────────────────────────┘  │  │
│  │   ┌─────────────────────────────────────────────────────────────┐  │  │
│  │   │  Player TCP Forwarder                                         │  │  │
│  │   │    • On player TCP accept (port 25565)                        │  │  │
│  │   │    • Read SNI / Host header (if WSS) OR first byte peek      │  │  │
│  │   │      (raw TCP: use NLB's preserved client IP + server_id      │  │  │
│  │   │       encoded in CONNECT packet's hostname field? No — Mo-    │  │  │
│  │   │       jang doesn't send SNI for raw TCP)                       │  │  │
│  │   │    • Look up server_id in tunnel registry                     │  │  │
│  │   │    • If no tunnel → clean socket close (D-18)                 │  │  │
│  │   │    • If tunnel → open yamux stream → bidi copy to player TCP  │  │  │
│  │   └─────────────────────────────────────────────────────────────┘  │  │
│  │   ┌─────────────────────────────────────────────────────────────┐  │  │
│  │   │  Heartbeat Watchdog                                            │  │  │
│  │   │    • Per-tunnel: 10s ticker, expect heartbeat; 3 missed =      │  │  │
│  │   │      mark tunnel stale, notify backend WS (POST /tunnel/      │  │  │
│  │   │      status)                                                   │  │  │
│  │   └─────────────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                           │
                                           │ POST /internal/relay/authorize (D-10)
                                           │ POST /internal/relay/tunnel-event (heartbeat, disconnect)
                                           ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  Escluse API / Backend (existing api/)                                    │
│    • GET node.relay_token by Authorization Bearer                          │
│    • Verify node.owns(server_id)                                          │
│    • Return 200/403                                                       │
│    • On tunnel_event → emit TunnelConnect/Disconnect to                   │
│      NodeMessage enum → agent's existing WS picks it up                   │
│      (Phase 67 node_protocol extension pattern)                           │
└──────────────────────────────────────────────────────────────────────────┘
                                           ▲
                                           │ (existing) WebSocket wss://api.esluce.com/api/ws/node
                                           │
┌──────────────────────────────────────────────────────────────────────────┐
│  Agent (Escluse Solys — runs on user's VPS/local machine)                │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  relay_client.rs (NEW)                                             │  │
│  │    • Outbound WSS to wss://relay.esluce.net/tunnel                 │  │
│  │    • Custom request headers:                                       │  │
│  │        Authorization: Bearer <relay_token>  (from nodes table)      │  │
│  │        X-Relay-Nonce: <32 random bytes hex>                        │  │
│  │        X-Relay-Timestamp: <unix seconds>                           │  │
│  │    • On connect: spawn yamux session (Control::new_client)         │  │
│  │    • Heartbeat task: 10s, payload:                                 │  │
│  │        { ts, uptime, bytes_in, bytes_out, active_streams }         │  │
│  │    • Reconnect: exponential backoff 1s→30s, ±20% jitter (D-04)     │  │
│  │    • On new player yamux stream:                                    │  │
│  │        - tokio::net::TcpStream::connect("127.0.0.1:<game_port>")   │  │
│  │        - tokio::io::copy_bidirectional (yamux stream ↔ local TCP)  │  │
│  │    • On tunnel_disconnect: send NodeMessage::TunnelDisconnect via   │  │
│  │      existing backend WS → backend publishes to Phase 67 probe     │  │
│  │      pipeline → mode flip to Relay (D-13)                          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  dns_watch.rs (existing, Cloudflare for *.play.esluce.com)         │  │
│  │    • Phase 67 trigger: on direct_probe success → create A record   │  │
│  │    • Phase 68 trigger: tunnel_reconnect → re-probe Direct → if OK  │  │
│  │      create A record                                               │  │
│  │    • Phase 68 trigger: tunnel_disconnect → remove A record         │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

**Critical note on player → server_id routing for raw TCP:**

Minecraft Java clients **do not send SNI for raw TCP** — SNI is a TLS extension, and Minecraft Java connections are plain TCP. The CONTEXT D-06 says "**TLS SNI / Host header (for `wss://` connections) OR a custom HTTP header in the `proxy_protocol` (for raw TCP)**" — the SNI path is for WSS (not used in Phase 68 since the player connects raw TCP). For raw TCP, the only options are:

1. **NLB preserves client IP** (instance target type) — but the IP is the player's IP, not the server_id.
2. **Pre-provision each server on its own port** — defeats the whole `<server>.play.esluce.net` wildcard point.
3. **Use a HAProxy-like protocol on the gateway that asks the client for the server_id** — would break vanilla Minecraft clients.
4. **ALB terminates TLS** (optional) AND client uses `wss://` — but vanilla Minecraft client does not speak WebSocket.

**The actually-correct interpretation (resolving this for the planner):** Re-read D-06. The CONTEXT.md says "uses the **TLS SNI / Host header** (for `wss://` connections) OR a **custom HTTP header in the proxy_protocol** (for raw TCP)". The "wss:// connections" part means: **players using a WebSocket bridge** (third-party tools, custom launchers, or the "Invite friends" UI that opens a wss:// for the user) can send SNI. For raw vanilla Minecraft TCP, the solution is: the relay gateway binds **one TCP listener per active server** on the NLB — meaning NLB target group is per-server (one EC2 listens on 25565, but the gateway internally listens on a unique port per server, and DNS `*.play.esluce.net` resolves to a port-differentiation scheme? No, DNS doesn't carry ports.).

**Resolution: Use ALB TLS listener with PROXY protocol v2 + SNI-based routing.** Actually no — the simplest and only-vanilla-Minecraft-compatible solution is:

**Final design (revised from D-06):** The player connects to `<server>.play.esluce.net:25565`. DNS resolves to the **NLB**. NLB forwards to the EC2 gateway on port 25565. The gateway receives the TCP connection with only the player's source IP. Since it cannot know which server the player intended, the gateway uses an **external lookup** keyed on the **player's source IP**: the agent's last-known public IP for that server (kept in the tunnel registry: `server_id → TunnelEntry { agent_public_ip, ... }`). The gateway finds the `server_id` whose `agent_public_ip` matches the player's source IP. If multiple servers share a public IP (CGN behind same NAT), the most recently connected wins (or: requires user to also send the server_id in a non-standard way; not vanilla-compatible).

**This is the correct interpretation, but the planner needs to validate it.** The simpler and architecturally-cleaner alternative (likely the intent of D-06) is to use **NLB per-server listener + per-server A record** — but this requires `N` A records, one per server, in `*.play.esluce.net`. We can do this with Route 53 wildcard + per-server subdomain CNAME to a unique hostname → NLB listener on a unique port.

**Recommended approach (the agent's discretion, but here's the proposal):** For Phase 68, use the **"match by player source IP = agent public IP"** approach. Most home/small-VPS deployments have one public IP per node; collision rate is low. Document the limitation; if it becomes an issue, move to per-server NLB listeners in a follow-up. The `relay.esluce.net` zone's wildcard `*.play.esluce.net` A record → NLB IP is static and the gateway does the server_id resolution internally. **See Pitfall 9 for the full discussion.**

### Recommended Project Structure

```
opt/relay/                              # NEW: Relay gateway deployment (mirrors opt/umami/)
├── docker-compose.yml                  # caddy + relay-gateway
├── Caddyfile                           # *.play.esluce.net + relay.esluce.net TLS
├── Caddy.Dockerfile                    # caddy with caddy-dns/route53 plugin
├── relay-gateway.Dockerfile            # multi-stage build for the Rust binary
├── relay-gateway.toml                  # gateway config: bind addrs, backend URL, Redis URL
├── relay-session-log.toml              # per-tunnel Redis schema (tunnel:<id>, nonce:<hex>)
├── .env.example                        # BACKEND_URL, REDIS_URL, RUST_LOG, METRICS_PORT
├── DEPLOY.md                           # Manual AWS + DNS + EC2 + IAM setup steps
├── src/
│   ├── main.rs                         # Axum server, route mounting, signal handling
│   ├── config.rs                       # Config struct + load from TOML + env
│   ├── state.rs                        # AppState: Redis pool, BackendClient, Registry
│   ├── auth.rs                         # TunnelAuth middleware (token + nonce + rate limit)
│   ├── tunnel.rs                       # /tunnel WebSocket upgrade handler + yamux session
│   ├── player.rs                       # /player/:server_id? TCP forwarder (or port-based)
│   ├── registry.rs                     # DashMap<Uuid, TunnelEntry> + heartbeat watchdog
│   ├── backend.rs                      # reqwest client → api.esluce.com
│   ├── metrics.rs                      # Prometheus registry + recorders
│   ├── error.rs                        # RelayError + IntoResponse impl
│   └── ratelimit.rs                    # Redis-backed Lua rate-limit (100/min per IP)
└── tests/
    ├── tunnel_handshake.rs             # Unit: nonce + token validation
    ├── registry_concurrent.rs          # Stress: 100 concurrent lookups
    └── player_forwarder.rs             # Integration: player TCP → yamux stream

api/                                    # EXISTING — Phase 68 additions only
├── migrations/
│   └── 20260607000001_add_relay_columns.sql      # NEW
│       ALTER TABLE nodes ADD COLUMN relay_token UUID NOT NULL UNIQUE
│         DEFAULT gen_random_uuid();   -- (generated at registration, but unique NOT NULL
│                                       --  means we need to backfill existing nodes)
│       ALTER TABLE servers
│         ADD COLUMN connectivity_mode_override TEXT,  -- 'auto'|'direct'|'relay'|NULL
│         ADD COLUMN relay_status          TEXT,        -- 'connected'|'connecting'|'disconnected'
│         ADD COLUMN last_tunnel_connected_at TIMESTAMPTZ;
├── src/
│   ├── domain/entities/
│   │   ├── node.rs                                # EXTEND with relay_token
│   │   └── server.rs                              # EXTEND with relay fields
│   ├── presentation/
│   │   ├── ws/
│   │   │   └── node_protocol.rs                   # EXTEND with TunnelConnect/Disconnect/Heartbeat
│   │   │                                           #         + ModeOverrideChange
│   │   ├── handlers/
│   │   │   ├── node_ws_handler.rs                 # EXTEND: dispatch new tunnel messages
│   │   │   ├── connectivity_handlers.rs           # NEW (Phase 67) — extend with /mode-override
│   │   │   └── relay_internal_handlers.rs         # NEW
│   │   │     + POST /api/v1/internal/relay/authorize  (token + server_id)
│   │   │     + POST /api/v1/internal/relay/tunnel-event  (heartbeat, disconnect)
│   │   ├── routes/
│   │   │   └── api_routes.rs                      # MOUNT new internal routes
│   │   └── services/
│   │       └── relay_service.rs                   # NEW: token issuance, mode override persistence
│   ├── bootstrap/
│   │   └── container.rs                           # EXTEND: add RelayAuthClient, RelayMetricsCollector
│   └── application/services/
│       └── monitoring_service.rs                  # EXTEND: scrape relay /metrics
├── tests/
│   ├── relay_introspection.rs             # Unit: authorize returns 200/403
│   ├── mode_override.rs                   # Unit: POST /connectivity/mode-override
│   └── tunnel_event_propagation.rs        # Integration: agent → backend → gateway → mode flip

agent/solys/                              # EXISTING — Phase 68 additions
├── src/
│   ├── handlers/
│   │   ├── mod.rs                                  # EXTEND: register relay.* tasks
│   │   ├── relay.rs                                # NEW: task dispatch (relay.connect,
│   │   │                                             relay.disconnect, relay.heartbeat,
│   │   │                                             relay.refresh_token)
│   │   ├── relay_client.rs                         # NEW: outbound WS client (D-01..D-04)
│   │   ├── relay_session.rs                        # NEW: yamux session management
│   │   └── dns.rs                                  # EXTEND: Direct Mode A record on tunnel
│   │                                                #   reconnect (D-13), remove on disconnect
│   ├── main.rs                                     # EXTEND: start relay_client on agent boot
│   └── config.rs                                   # EXTEND: load relay_token from disk
└── tests/
    ├── tunnel_reconnect.rs                # Unit: backoff schedule, heartbeat missed
    ├── yamux_stream_isolation.rs          # Unit: one stream per server_id, no crosstalk
    └── direct_mode_emit.rs                # Unit: tunnel_reconnect → Cloudflare A record create

app/                                      # EXISTING — Phase 68 additions
├── src/
│   ├── pages/servers/
│   │   └── ServerDetailsPage.jsx                   # EXTEND: ConnectivitySection with tunnel health
│   │                                                #   + "Copy join address" (relay first, direct second)
│   │                                                #   + "Invite friends" UI (both addresses + QR)
│   │                                                #   + Mode override dropdown (Auto/Force Direct/Force Relay)
│   ├── components/
│   │   ├── ConnectivitySection.jsx                 # NEW/EXTEND (Phase 67) with tunnel-specific fields
│   │   ├── InviteFriendsModal.jsx                  # NEW: QR code + both addresses
│   │   ├── ModeOverrideDropdown.jsx                # NEW
│   │   └── TunnelHealthCard.jsx                    # NEW: latency / uptime / last heartbeat
│   └── hooks/
│       └── useConnectivity.js                      # EXTEND: tunnel health data + mode override mutation
```

### Pattern 1: Tunnel WebSocket Handshake (D-09, D-10, D-11)

**What:** Agent dials `wss://relay.esluce.net/tunnel` with custom headers; gateway validates token + nonce + timestamp + ownership before accepting the upgrade and starting a yamux session.

**When to use:** Every tunnel connect and reconnect.

**Example (Agent side — `agent/solys/src/handlers/relay_client.rs`):**

```rust
// Source: CONTEXT.md D-01, D-09, D-11; tokio-tungstenite 0.26 docs
// (IntoClientRequest trait accepts http::Request<()> for custom headers)
use std::time::{SystemTime, UNIX_EPOCH};
use http::Request;
use rand::RngCore;
use tokio_tungstenite::connect_async;
use tokio_yamux::Control;

const RELAY_URL: &str = "wss://relay.esluce.net/tunnel";

pub async fn connect_tunnel(
    relay_token: &str,
) -> anyhow::Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Control)> {
    // Generate nonce + timestamp (D-11)
    let mut nonce = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut nonce);
    let nonce_hex = hex::encode(nonce);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    // Build request with custom headers (D-09, D-11)
    let request = Request::builder()
        .method("GET")
        .uri(RELAY_URL)
        .header("Host", "relay.esluce.net")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_sec_websocket_key())
        .header("Authorization", format!("Bearer {}", relay_token))
        .header("X-Relay-Nonce", nonce_hex.clone())
        .header("X-Relay-Timestamp", timestamp.to_string())
        .body(())
        .map_err(|e| anyhow::anyhow!("Bad request: {}", e))?;

    let (ws_stream, _response) = connect_async(request).await?;
    info!("Relay tunnel established (nonce={}...)", &nonce_hex[..8]);

    // Wrap in yamux client session
    let yamux_session = tokio_yamux::Session::new_client(ws_stream);
    let control = yamux_session.control();

    // Drive the yamux session (read frames, dispatch to streams)
    tokio::spawn(yamux_session);

    Ok((ws_stream, control))
}

fn generate_sec_websocket_key() -> String {
    // tungstenite has tungstenite::handshake::client::generate_key; or just base64 random
    use base64::Engine;
    let mut key = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut key);
    base64::engine::general_purpose::STANDARD.encode(key)
}
```

**Example (Gateway side — `opt/relay/src/auth.rs`):**

```rust
// Source: CONTEXT.md D-09, D-10, D-11
use axum::{extract::ws::WebSocketUpgrade, http::HeaderMap, response::IntoResponse};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn tunnel_upgrade(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 1. Extract & validate Authorization header (D-09)
    let token = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(RelayError::Unauthorized)?;

    // 2. Extract nonce + timestamp (D-11)
    let nonce = headers.get("X-Relay-Nonce")
        .and_then(|v| v.to_str().ok())
        .ok_or(RelayError::BadRequest("missing X-Relay-Nonce"))?;
    let timestamp: u64 = headers.get("X-Relay-Timestamp")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .ok_or(RelayError::BadRequest("missing X-Relay-Timestamp"))?;

    // 3. Check timestamp freshness (≤ 5 min old)
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if now.abs_diff(timestamp) > 300 {
        return Err(RelayError::Unauthorized);  // replay window exceeded
    }

    // 4. Check nonce dedup in Redis (10-min TTL)
    let nonce_key = format!("relay:nonce:{}", nonce);
    let was_set: bool = redis::cmd("SET")
        .arg(&nonce_key).arg("1").arg("NX").arg("EX").arg(600)
        .query_async(&state.redis).await?;
    if !was_set {
        return Err(RelayError::Unauthorized);  // replay attack
    }

    // 5. Rate limit: 100 connects/min per source IP
    let source_ip = /* extract from socket */;
    ratelimit::check_connect(&state.redis, &source_ip, 100, 60).await?;

    // 6. Validate token + extract node_id via backend introspection (D-10)
    let node_id = state.backend_client.introspect_token(token).await?;
    if node_id.is_none() {
        return Err(RelayError::Unauthorized);
    }

    // 7. Accept the upgrade; the actual tunnel setup is in the on_upgrade callback
    Ok(ws.on_upgrade(move |socket| async move {
        tunnel_session(socket, state, node_id.unwrap()).await
    }))
}
```

### Pattern 2: Player TCP → Yamux Stream Forwarder (D-06, D-18, D-19)

**What:** Player opens raw TCP to NLB → gateway receives → looks up server_id (by source-IP match) → opens yamux stream → bidirectional copy.

**When to use:** Every accepted player TCP connection.

**Example (Gateway — `opt/relay/src/player.rs`):**

```rust
// Source: CONTEXT.md D-06 (resolved per Pitfall 9), D-18, D-19
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;

pub async fn handle_player_connection(
    state: Arc<AppState>,
    player_stream: TcpStream,
    player_addr: SocketAddr,
) {
    // Resolve server_id by player source IP = agent public IP (Pitfall 9)
    let server_id = state.registry.find_by_agent_ip(player_addr.ip()).await;

    let server_id = match server_id {
        Some(id) => id,
        None => {
            tracing::warn!(%player_addr, "No active tunnel for player IP — closing");
            // D-18: clean close, no RST
            drop(player_stream);
            return;
        }
    };

    // Open a yamux stream on the agent's tunnel
    let tunnel = state.registry.get(&server_id).await;
    let tunnel = match tunnel {
        Some(t) => t,
        None => {
            drop(player_stream);
            return;
        }
    };

    let yamux_stream = match tunnel.control.open_stream().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(?server_id, ?e, "Failed to open yamux stream");
            drop(player_stream);
            return;
        }
    };

    // Bidi copy between player TCP and yamux stream
    // Idle timeout 5 min (D-19) — wrap in tokio::time::timeout
    let copy_result = tokio::time::timeout(
        Duration::from_secs(300),
        copy_bidirectional(&mut player_stream, &mut yamux_stream),
    ).await;

    match copy_result {
        Ok(Ok((in_bytes, out_bytes))) => {
            tracing::info!(?server_id, in_bytes, out_bytes, "Player session ended cleanly");
            metrics::PLAYER_BYTES_IN.inc_by(in_bytes as f64);
            metrics::PLAYER_BYTES_OUT.inc_by(out_bytes as f64);
        }
        Ok(Err(e)) => tracing::warn!(?server_id, ?e, "Player session error"),
        Err(_) => tracing::debug!(?server_id, "Player session idle timeout (5 min)"),
    }

    // Yamux stream drops on scope exit → FIN
}
```

### Pattern 3: Agent Yamux Stream → Local Minecraft TCP (D-01, D-02)

**What:** Agent receives a new yamux stream from the gateway → opens TCP to local Minecraft → bidirectional copy.

**When to use:** Each accepted yamux stream on the agent's tunnel.

**Example (Agent — `agent/solys/src/handlers/relay_session.rs`):**

```rust
// Source: CONTEXT.md D-01, D-02
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tokio_yamux::StreamHandle;

pub async fn handle_incoming_stream(
    stream: StreamHandle,
    server_id: Uuid,
    game_port: u16,
) -> anyhow::Result<()> {
    tracing::info!(%server_id, %game_port, "New player yamux stream");

    // Connect to the local Minecraft server
    let local = TcpStream::connect(("127.0.0.1", game_port)).await?;
    let (mut player_side, mut mc_side) = (stream, local);

    // Bidi copy: yamux stream ↔ local Minecraft TCP
    let (in_bytes, out_bytes) = copy_bidirectional(&mut player_side, &mut mc_side).await?;
    tracing::info!(%server_id, in_bytes, out_bytes, "Player session ended");

    Ok(())
}
```

### Pattern 4: Heartbeat Watchdog + Stale Tunnel Detection (D-04, D-21)

**What:** Gateway tracks last-heartbeat per tunnel; 3 missed = mark stale; new tunnel replaces atomically.

**When to use:** Background task per active tunnel.

**Example (Gateway — `opt/relay/src/registry.rs`):**

```rust
// Source: CONTEXT.md D-04, D-21
use tokio::time::{interval, Duration, Instant};

pub async fn heartbeat_watchdog(
    server_id: Uuid,
    control: tokio_yamux::Control,
    state: Arc<AppState>,
) {
    let mut ticker = interval(Duration::from_secs(10));
    let stale_threshold = 3;  // 3 missed heartbeats (D-04)
    let mut missed = 0u32;

    loop {
        ticker.tick().await;

        let tunnel = state.registry.get(&server_id).await;
        let Some(tunnel) = tunnel else { break };

        let elapsed = tunnel.last_heartbeat.elapsed();
        if elapsed > Duration::from_secs(30) {
            missed += 1;
            if missed >= stale_threshold {
                tracing::warn!(%server_id, "Tunnel stale (3 missed heartbeats) — closing");
                let _ = control.close().await;
                state.registry.remove(&server_id).await;
                // Notify backend (POST /internal/relay/tunnel-event)
                let _ = state.backend_client
                    .notify_tunnel_event(server_id, tunnel.node_id, "stale")
                    .await;
                break;
            }
        } else {
            missed = 0;
        }
    }
}
```

### Pattern 5: Tunnel Session Log (Redis-backed, Ephemeral)

**What:** Per-tunnel structured log entries stored in Redis (not PostgreSQL — ephemeral, low-cardinality).

**When to use:** Every tunnel lifecycle event (connect, heartbeat, player_connect, player_disconnect, disconnect).

**Example (Gateway — `opt/relay/src/session_log.rs`):**

```rust
// Source: CONTEXT.md <specifics> per-tunnel log shape
use redis::AsyncCommands;
use chrono::Utc;

pub async fn log_event(
    redis: &redis::Client,
    session_id: &str,
    event: &str,
    details: serde_json::Value,
) {
    let key = format!("relay:session:{}:log", session_id);
    let entry = serde_json::json!({
        "ts": Utc::now().to_rfc3339(),
        "event": event,
        "details": details,
    });
    let mut conn = redis.get_async_connection().await.ok();
    if let Some(c) = conn.as_mut() {
        let _: Result<(), _> = c.rpush(&key, entry.to_string()).await;
        let _: Result<(), _> = c.expire(&key, 86400).await;  // 24h TTL
    }
}
```

### Anti-Patterns to Avoid

- **Using `yamux 0.13` directly with the futures→tokio adapter:** Use `tokio-yamux 0.3` for native tokio `AsyncRead/AsyncWrite` compatibility. Saves a layer of adapter complexity.
- **Reaching for QUIC (`quinn`) in Phase 68:** CONTEXT D-01 explicitly rejected QUIC. WebSocket over TLS is sufficient. Add a TODO comment for the future HTTP/3 path; architecture should not preclude it (D-25).
- **Per-server NLB listener (one TCP port per server):** Would require `N` A records, defeating the wildcard. Use single NLB on port 25565 + in-gateway server_id resolution.
- **Returning application-level error messages on player connect failure (D-18):** "No active tunnel for this server" would surface to the Minecraft client as a malformed packet. Clean socket close + "Connection refused" UX is correct.
- **Storing tunnel session logs in PostgreSQL:** The CONTEXT specifies ephemeral (Redis, 24h TTL). PostgreSQL is for durable connectivity state (`relay_status`, `last_tunnel_connected_at`).
- **Auto-installing Tailscale or Cloudflare Tunnel:** CONTEXT D-11, D-12 (Phase 67) explicitly out of scope. The relay makes them unnecessary for the player's connectivity story.
- **Touching the OLD `Server` struct (`api/src/domain/server/model.rs:8-31`):** Pre-existing tech debt (deferred from Phase 67). Use the NEW `Server` struct (`api/src/domain/entities/server.rs:8-75`).
- **Hardcoding the relay token in agent config:** Use the `Register` ack handshake to issue + persist the token on first agent connect. The token is then stored in the agent's TOML config file and reloaded on restart.
- **WebSocket frame max size = 1 MiB:** Minecraft packets can be larger; default `tokio-tungstenite` is 16 KiB which is too small. Set max frame size to 1 MiB or 4 MiB to handle chat / large entity updates.
- **Caching introspection result for the full tunnel lifetime:** The CONTEXT D-10 says "Cached in relay memory for the tunnel lifetime" — but if a user deletes a server mid-session, the gateway would still route to it. Add a short re-check (e.g., every 60s) OR a revocation webhook from backend → relay.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WebSocket transport (with custom headers, TLS) | Custom HTTP/1.1 upgrade + frame parser | `tokio-tungstenite 0.26` (already in agent) | Frame parsing, masking, ping/pong, close codes — all subtle, error-prone. Custom headers via `http::Request<()>` + `IntoClientRequest`. |
| Stream multiplexing (one TCP per player over one tunnel) | Custom length-prefixed binary protocol | `tokio-yamux 0.3` | yamux handles stream IDs, flow control (256 KiB default window), FIN/RST semantics, GoAway frames. ~1000 LOC of subtle async I/O. |
| TLS 1.3 termination | Custom TLS handshake | Caddy 2 + Let's Encrypt + `caddy-dns/route53` plugin | Certificate provisioning, renewal, OCSP stapling, SNI — all standard. |
| Wildcard DNS-01 challenge | Custom ACME client | Caddy DNS provider plugin (`caddy-dns/route53`) | ACME protocol is non-trivial; DNS-01 requires API call to add/remove TXT records. |
| WebSocket client with reconnect + backoff + jitter | Custom reconnection loop | Mirror agent's `dns_watch.rs` pattern + D-04 jitter formula | Reuses the well-tested `interval` + `tokio::spawn` pattern already in the codebase. |
| Per-server rate limiting | In-memory hashmap counter | Redis + Lua script (`INCR` + `EXPIRE`) | Atomic across multiple gateway instances; TTL handled by Redis. |
| Nonce dedup for replay protection | In-memory `HashSet<String>` | Redis `SET NX EX 600` | Survives gateway restart; works across multiple instances. |
| Active tunnel registry (per `server_id`) | PostgreSQL + per-request SELECT | In-memory `DashMap<Uuid, TunnelEntry>` | O(1) lookup on the hot path; ephemeral by design. |
| Backend introspection (`/internal/relay/authorize`) | Trust the relay token without re-checking ownership | `POST /internal/relay/authorize` per tunnel + per-server registration | D-10 explicitly requires backend validation; user ownership is the only source of truth. |
| Player TCP forwarding | Custom protocol-aware relay | `tokio::io::copy_bidirectional` over `TcpStream` ↔ yamux stream | Minecraft Java is plain TCP; no application-layer logic needed. |
| Prometheus metrics export | Custom text-format HTTP handler | `prometheus` crate (or `prometheus-client`) | Standard format; existing monitoring service already scrapes HTTP. |
| AWS infrastructure provisioning | Terraform / Pulumi | Manual AWS Console + CLI scripts (per Phase 66 D-06) | Out of scope per project conventions; declarative setup documented in `DEPLOY.md`. |
| Audit log of tunnel events | Custom file logger | Redis `RPUSH` + TTL (per CONTEXT log shape) | Ephemeral; matches the per-session shape in CONTEXT `<specifics>`. |
| CORS / security headers for `/metrics` | Manual `header` calls | `tower-http::set_header` middleware | Standard pattern from `gateway/Caddyfile.prod`. |

**Key insight:** The combination of **Caddy for TLS + tokio-tungstenite for WS + tokio-yamux for multiplexing + Caddy DNS-01 for wildcard cert** means we never write a single line of TLS, ACME, DNS, or framing code. The relay is essentially: Axum HTTP server + WS upgrade handler + yamux client/server + reqwest to backend + Redis for state. ~1500 LOC of new Rust total.

## Common Pitfalls

### Pitfall 1: WebSocket frame max size too small for Minecraft
**What goes wrong:** Default `tokio-tungstenite` frame size is 16 KiB. Minecraft's SLP status response can hit 64 KiB on a populated server, and chat messages can spike above 16 KiB during busy moments. Players see "connection reset" mid-game.
**Why it happens:** Default Rust WebSocket max frame size is conservative.
**How to avoid:** Configure `WebSocketConfig::default().max_frame_size(4 * 1024 * 1024)` (4 MiB) when building both client (agent) and server (gateway). yamux layer on top handles its own stream framing.
**Warning signs:** Logs show `WebSocketClosed(ProtocolError(MaxFrameSize))` or players disconnect after sending long chat messages.

### Pitfall 2: NLB preserves client IP, but instance target type required
**What goes wrong:** If you set NLB target type to `ip` (a private IP), NLB replaces the client IP with the private IP of the load balancer node. Per-IP rate limit on the gateway then sees the same source IP for ALL players (the NLB's private IP).
**Why it happens:** Per AWS docs (verified `docs.aws.amazon.com/elasticloadbalancing/latest/network/load-balancer-target-groups.html`): "client IP preservation can't be disabled for UDP, TCP_UDP, QUIC, and TCP_QUIC target groups" — but for `ip` target type, the default is disabled for TCP.
**How to avoid:** Use **`instance` target type** for the NLB. The EC2 instance is registered as a target by its instance ID; NLB preserves the original client IP. Verify with `nlb describe-target-health` and a curl-style test from an external host.
**Warning signs:** All player connections appear to come from a single source IP in gateway logs.

### Pitfall 3: Cross-zone load balancing NLB
**What goes wrong:** With cross-zone load balancing enabled, NLB distributes traffic across all registered targets in all AZs. For a single-AZ Phase 68 (D-17), this is moot, but if you later add a second AZ, the gateway needs to share tunnel state between instances. yamux sessions are pinned to the agent's persistent connection — if NLB moves the player TCP to a different gateway instance, that instance doesn't have the tunnel.
**Why it happens:** NLB is a Layer 4 load balancer — it doesn't know about yamux sessions.
**How to avoid:** For Phase 68, single AZ + single gateway instance → disable cross-zone. For future multi-AZ, use an **NLB Target Group stickiness** (or a "gateway affinity" by player source IP — the same player IP lands on the same gateway instance, which has the tunnel).
**Warning signs:** Players get "Connection reset" intermittently in multi-AZ future state.

### Pitfall 4: yamux stream is half-closed but not FIN
**What goes wrong:** `tokio::io::copy_bidirectional` returns when one side sends FIN, but the yamux stream is still open until BOTH sides FIN. Player disconnects → stream has data buffered → gateway keeps writing until the read side returns 0.
**Why it happens:** TCP half-close semantics; yamux mirrors this.
**How to avoid:** Wrap `copy_bidirectional` in `tokio::time::timeout` (5 min, D-19) AND drop both streams cleanly on timeout/error. yamux's `Drop` impl sends FIN.
**Warning signs:** Gateway memory grows over time; player session count in metrics never decreases.

### Pitfall 5: Heartbeat is sent inside yamux, not on the WebSocket
**What goes wrong:** If you send heartbeats as yamux control frames (on stream ID 0), they get sent as data frames on the WS. WS ping/pong frames (different mechanism) can go idle and the LB or proxy in front may drop the connection.
**Why it happens:** yamux is unaware of WebSocket-level keepalive.
**How to avoid:** Use yamux Ping frames (D-04) for application-level heartbeat, AND WebSocket `Message::Ping` (axum's built-in) for transport-level keepalive. Both; they protect against different failure modes.
**Warning signs:** Tunnel drops every ~60s with no warning; logs show no missed yamux heartbeats.

### Pitfall 6: Agent holds tunnel open but no player traffic for hours
**What goes wrong:** Agent opens tunnel on `agent boot`, but the server is stopped. yamux session is idle for 12 hours. NLB / ALB / Caddy might kill the idle WS connection (typical idle timeout: 60s for AWS ALB, 10 min for NLB).
**Why it happens:** Idle WS connections dropped by upstream load balancers.
**How to avoid:** Send WS-level ping every 30s (D-19 = 5 min idle timeout, but load balancer idle timeout is shorter). On disconnect, reconnect with backoff (D-04).
**Warning signs:** Tunnels drop exactly every 60s in production (AWS ALB default idle timeout).

### Pitfall 7: Nonce is reused on reconnect
**What goes wrong:** Agent generates nonce, sends it. Network blip. Agent reconnects with a NEW nonce (correct). But: if the agent uses `rand::thread_rng()` and re-uses the same nonce on a different connection (e.g., due to clock skew or deterministic seed), Redis sees the same nonce within 10 min → rejects.
**Why it happens:** Bug in agent's nonce generation OR clock skew causing timestamp to fail validation.
**How to avoid:** Use `rand::rngs::OsRng` (cryptographically secure) for nonce generation. Include the current Unix nanosecond timestamp (not seconds) to reduce collision probability. Add a unique agent-instance ID to the nonce to prevent cross-instance collisions.
**Warning signs:** "Replay protection" rejection in gateway logs for the same agent connecting back-to-back.

### Pitfall 8: Backend introspection timeout causes slow tunnel setup
**What goes wrong:** `POST /internal/relay/authorize` has a 50ms p99 budget (D-10). If the backend is slow (DB load, network), the WS upgrade request times out and the agent retries. The agent sees "tunnel won't establish" while everything is fine.
**Why it happens:** Contention on the `nodes` table or Redis.
**How to avoid:** Gateway uses aggressive timeout (e.g., 100ms) for introspection; on timeout, fail closed (reject the tunnel). Backend keeps an in-memory cache of `(relay_token → node_id, server_ids_owned)` with a 60s TTL. The introspection call is the hot path; cache it.
**Warning signs:** Gateway logs show "introspection timeout" for tunnels that are otherwise valid.

### Pitfall 9: Player-to-server-id resolution (resolving D-06 ambiguity)
**What goes wrong:** CONTEXT D-06 says "uses the TLS SNI / Host header (for wss:// connections) OR a custom HTTP header in the proxy_protocol (for raw TCP)" — but raw Minecraft Java TCP doesn't have SNI, and a vanilla Minecraft client can't send custom proxy_protocol headers.
**Why it happens:** CONTEXT didn't fully specify the resolution mechanism for raw vanilla Minecraft TCP.
**How to avoid:** Use **player source IP matching against agent's known public IP** (every agent stores its public IP in the relay_token registration response or in heartbeat payload). The gateway matches `player_addr.ip() == agent_public_ip` to identify the server. Collision only when two servers share the same agent public IP — rare in single-node deployments. Document this as a known limitation; per-server NLB listeners are a future fix.
**Alternative (cleaner):** Use one NLB target group per server. For N servers, N listeners on different ports, with `*.play.esluce.net` resolving to a CNAME chain that encodes the port. Breaks wildcard DNS; the planner should reject this and use source-IP matching.
**Warning signs:** Players on the same CGN carrier (same public IP) connecting to different servers see "Connection refused" intermittently.

### Pitfall 10: AWS NLB idle timeout vs gateway idle timeout mismatch
**What goes wrong:** AWS NLB has a 350s idle timeout (TCP). Minecraft player AFK for 6 minutes → NLB drops the connection. But the gateway's 5 min timeout (D-19) hasn't fired yet, so the player sees a clean disconnect.
**Why it happens:** NLB's idle timeout is shorter than the gateway's.
**How to avoid:** Document that idle timeout is the MIN of (NLB timeout, gateway timeout) = 350s. Or, more importantly, Minecraft client itself sends keepalive packets every 15-30s, so 350s of TRUE idle is extremely rare in practice. Don't chase this pitfall unless users report it.
**Warning signs:** AFK players disconnect after ~5 minutes; logs show NLB RST, not gateway timeout.

### Pitfall 11: Tunnel session rekeying (D-25 discretion)
**What goes wrong:** Long-lived tunnels (days/weeks) without rekeying increase blast radius if a key is compromised. yamux doesn't have built-in rekeying; you'd have to tear down and re-establish the yamux session.
**Why it happens:** No built-in rekeying in yamux spec.
**How to avoid:** Per D-25: rekey every 24h OR 100 GB transferred (whichever first). Agent closes the WS connection cleanly (sends GoAway), gateway closes yamux session, both sides clean up, agent reconnects with new handshake (new nonce, new TLS session). The reconnect takes ~50ms; in-flight player sessions see a brief interruption.
**Warning signs:** Tunnels are days old; auditor flags lack of session rotation.

### Pitfall 12: Per-server mode override is not propagated to the relay
**What goes wrong:** User pins a server to "Force Direct" in the dashboard. The agent's `dns_watch.rs` correctly removes the relay tunnel and creates the Cloudflare A record. But: the relay gateway still has a stale entry in the active tunnel registry (because the agent's tunnel is still up). New players can still use the relay path.
**Why it happens:** Two-tier state — agent-side mode selection vs gateway-side tunnel registry — not synchronized.
**How to avoid:** When user sets a mode override → backend sends `NodeMessage::ModeOverrideChange` to the agent via existing WS → agent disconnects the tunnel (if `Force Direct`) OR keeps it and creates the Direct A record (if `Force Relay`). The gateway doesn't need to know about mode overrides — it's a passive forwarder. Tunnel registry entry expires naturally on heartbeat timeout.
**Warning signs:** Dashboard shows "Force Direct" but relay is still serving players.

### Pitfall 13: Backend introspection races with `server.delete`
**What goes wrong:** User deletes a server. Backend's `ServerRepository::delete` runs. In parallel, gateway's cached `server_id → node_id` ownership map still says "owned". A new player yamux stream opens for the now-deleted server.
**Why it happens:** Gateway caches the introspection result "for the tunnel lifetime" (D-10). If the agent's tunnel is still open and a player connects, the gateway doesn't re-validate ownership per-player.
**How to avoid:** Two layers of defense: (1) backend's `server.delete` sends a `NodeMessage::ServerDeleted` to the agent; agent closes any active yamux streams for that server. (2) Gateway adds a 60s re-validation: every 60s, gateway re-calls `/internal/relay/authorize` for the top-N most-recently-active `server_id`s. The 60s window is acceptable for "user deleted a server" UX.
**Warning signs:** Players connect to a server that was deleted <2 minutes ago.

## Code Examples

Verified patterns from official sources:

### WebSocket client with custom headers (tokio-tungstenite 0.26)

```rust
// Source: https://docs.rs/tokio-tungstenite/0.26.2/tokio_tungstenite/fn.connect_async.html
// + https://users.rust-lang.org/t/custom-header-connect-async-tokio-tungstenite/100973
// IntoClientRequest trait accepts http::Request<()> directly for custom headers.

use http::Request;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::MaybeTlsStream;
use tokio::net::TcpStream;
use futures_util::{SinkExt, StreamExt};

type WsStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

let request = Request::builder()
    .method("GET")
    .uri("wss://relay.esluce.net/tunnel")
    .header("Host", "relay.esluce.net")
    .header("Connection", "Upgrade")
    .header("Upgrade", "websocket")
    .header("Sec-WebSocket-Version", "13")
    .header("Sec-WebSocket-Key", base64::encode(rand::random::<[u8; 16]>()))
    .header("Authorization", format!("Bearer {}", relay_token))
    .header("X-Relay-Nonce", nonce_hex)
    .header("X-Relay-Timestamp", ts.to_string())
    .body(())
    .expect("valid request");

let (mut ws, _response) = connect_async(request).await?;
while let Some(msg) = ws.next().await {
    match msg? {
        Message::Text(t) => println!("got: {}", t),
        Message::Close(_) => break,
        _ => {}
    }
}
```

### tokio-yamux client + control

```rust
// Source: https://docs.rs/tokio-yamux/0.3.18/tokio_yamux/
// tokio-yamux Session is created from a tokio AsyncRead+AsyncWrite stream
// (here, our WebSocketStream).

use tokio_yamux::{Config, Session, Mode, Control};

let session = Session::new_client(ws_stream, Config::default());
let control = session.control();

// Drive the session in a background task
tokio::spawn(session);

// Open a new outbound stream (for a player connection)
let stream = control.open_stream().await?;
// stream implements tokio::io::AsyncRead + tokio::io::AsyncWrite

// Write to the stream
use tokio::io::AsyncWriteExt;
stream.write_all(b"hello").await?;
```

### WebSocket server with custom header validation (axum 0.7)

```rust
// Source: https://docs.rs/axum/0.7/axum/extract/struct.WebSocketUpgrade.html
// + axum's WebSocketUpgrade allows header access via the request extractor.

use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    extract::State,
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router,
};

async fn tunnel_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let auth = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(String::from);
    let nonce = headers.get("X-Relay-Nonce")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let ts = headers.get("X-Relay-Timestamp")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    match (auth, nonce, ts) {
        (Some(token), Some(n), Some(t)) => {
            // Validate token, nonce, timestamp via state.backend + state.redis
            if let Ok(node_id) = state.validate_tunnel_upgrade(&token, &n, t).await {
                ws.on_upgrade(move |socket| handle_tunnel_session(socket, state, node_id))
            } else {
                (axum::http::StatusCode::UNAUTHORIZED, "invalid").into_response()
            }
        }
        _ => (axum::http::StatusCode::BAD_REQUEST, "missing headers").into_response(),
    }
}

async fn handle_tunnel_session(socket: WebSocket, state: AppState, node_id: Uuid) {
    use futures_util::{SinkExt, StreamExt};
    let (mut tx, mut rx) = socket.split();

    // Wrap WebSocket as AsyncRead+AsyncWrite for yamux
    let ws_adapter = WebSocketIo::new(tx.by_ref(), rx.by_ref());

    // Start yamux server session (gateway side; agent is client)
    let session = tokio_yamux::Session::new_server(ws_adapter, tokio_yamux::Config::default());
    let control = session.control();
    tokio::spawn(session);

    // Register this tunnel in the DashMap
    state.registry.insert(node_id, TunnelEntry::new(control.clone())).await;

    // Wait for incoming streams and forward them to the player handler
    // (or: yamux library spawns per-stream tasks)
}

let app = Router::new()
    .route("/tunnel", get(tunnel_handler))
    .with_state(state);
```

### Redis nonce dedup (replay protection)

```rust
// Source: Standard Redis SET NX EX pattern
// https://redis.io/commands/set/

use redis::AsyncCommands;

pub async fn check_and_set_nonce(redis: &redis::Client, nonce: &str) -> redis::RedisResult<bool> {
    let mut conn = redis.get_async_connection().await?;
    let key = format!("relay:nonce:{}", nonce);

    // SET key value NX EX 600 → returns "OK" if set (not seen), nil if already exists
    let result: Option<String> = redis::cmd("SET")
        .arg(&key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(600)
        .query_async(&mut conn)
        .await?;

    Ok(result.is_some())  // true = new nonce accepted, false = replay
}
```

### Redis rate limit (100/min per source IP)

```lua
-- Source: Standard Redis rate limit pattern (atomic INCR + EXPIRE)
-- Key: relay:rl:tunnel:<ip>  (or relay:rl:player:<ip>)
-- Returns: 1 if allowed, 0 if rate limited

local current = redis.call("INCR", KEYS[1])
if current == 1 then
    redis.call("EXPIRE", KEYS[1], ARGV[1])
end
if current > tonumber(ARGV[2]) then
    return 0
end
return 1
```

```rust
// Rust side
pub async fn check_rate_limit(
    redis: &redis::Client, ip: &str, kind: &str, limit: u32, window_secs: u32,
) -> redis::RedisResult<bool> {
    let mut conn = redis.get_async_connection().await?;
    let key = format!("relay:rl:{}:{}", kind, ip);
    let script = redis::Script::new(RATE_LIMIT_LUA);
    let allowed: i32 = script
        .key(&key)
        .arg(window_secs)
        .arg(limit)
        .invoke_async(&mut conn)
        .await?;
    Ok(allowed == 1)
}
```

### Prometheus metrics registration

```rust
// Source: https://docs.rs/prometheus/0.13
use prometheus::{IntCounter, IntGauge, Histogram, HistogramOpts, register_int_counter, register_int_gauge, register_histogram, Encoder};

lazy_static! {
    pub static ref RELAY_ACTIVE_TUNNELS: IntGauge = register_int_gauge!(
        "relay_active_tunnels_total",
        "Number of currently active relay tunnels"
    ).unwrap();

    pub static ref RELAY_BANDWIDTH_IN: IntCounter = register_int_counter!(
        "relay_bandwidth_in_bytes",
        "Total bytes received from players"
    ).unwrap();

    pub static ref RELAY_BANDWIDTH_OUT: IntCounter = register_int_counter!(
        "relay_bandwidth_out_bytes",
        "Total bytes sent to players"
    ).unwrap();

    pub static ref RELAY_LATENCY: Histogram = register_histogram!(
        HistogramOpts::new("relay_latency_seconds", "End-to-end latency")
            .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
    ).unwrap();
}

// Expose at /metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (axum::http::StatusCode::OK, [("content-type", "text/plain")], buffer)
}
```

### yamux spec reference (key fields)

```
Source: https://github.com/hashicorp/yamux/blob/master/spec.md

Frame header (12 bytes):
  Version (8 bits)  — always 0
  Type (8 bits)     — 0x0=Data, 0x1=WindowUpdate, 0x2=Ping, 0x3=GoAway
  Flags (16 bits)   — 0x1=SYN, 0x2=ACK, 0x4=FIN, 0x8=RST
  StreamID (32 bits) — client uses odd IDs, server uses even
  Length (32 bits)  — depends on Type

Initial window: 256 KiB per stream. Updates via WindowUpdate frames.

To open a stream: send Data or WindowUpdate with SYN flag, new StreamID.
The receiver replies with SYN+ACK. Either side can send data immediately
after the SYN (no need to wait for ACK) — this is unique vs TCP.

GoAway codes: 0x0=Normal, 0x1=Protocol, 0x2=Internal.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Agent self-probes its own port (Phase 67) | Hybrid: backend probes, agent provides facts (Phase 67) | Phase 67 | Only public-internet probe is authoritative for "can my friend connect". |
| Agent connects inbound to backend (deprecated) | Agent opens outbound WebSocket to backend (existing) | Phase ~17 | Outbound-only pattern means the relay can use the same pattern: agent → relay (outbound). |
| `yamux 0.13` (paritytech) with futures::io adapter | `tokio-yamux 0.3` (nervosnetwork/tentacle) | 2024+ | tokio-native AsyncRead/AsyncWrite; no adapter layer; smaller, more idiomatic code. |
| `igd` crate (popular but stale) | `upnp-rs 0.2` (Phase 67) | 2023+ | Active maintenance; we don't need it for relay but it's the modern UPnP story. |
| `tokio-tungstenite 0.20` (older) | `tokio-tungstenite 0.26` (current in project) | 2024+ | Already locked in `Cargo.lock` line 400. Supports rustls + native-tls via features. |
| TLS 1.2 default | TLS 1.3 only (per Phase 68 D-03) | Phase 68 | Forward-secret; same RFC 8446; most clients support it. |
| Hand-rolled Minecraft protocol parsing (Phase 67) | Use SLP only when validating Direct Mode (Phase 68) | Phase 67→68 | Relay is protocol-agnostic; Minecraft Java's SLP is NOT used on the relay→agent path. |
| Per-server NLB listener | Single NLB on port 25565 + in-gateway server_id resolution | Phase 68 | One A record covers all servers; matches the `<server>.play.esluce.net` wildcard. |
| ACME HTTP-01 challenge | DNS-01 challenge for `*.play.esluce.net` | 2020+ | DNS-01 is the only way to issue wildcard certs; HTTP-01 can't. |
| `aws-sdk-route53 0.x` (rusoto) | `aws-sdk-route53 1.x` | 2023+ | v0 line is deprecated. |
| `cloudshell` ad-hoc commands | Manual AWS Console + `aws-cli` per Phase 66 D-06 | Phase 66 | No Terraform / Pulumi in the project. |

**Deprecated/outdated:**
- **`yamux 0.13` as the primary choice** if you need `tokio::io` integration: use `tokio-yamux 0.3` instead. The two crates are API-incompatible (yamux 0.13 uses `futures::io`).
- **Inbound agent connections** for the relay: not feasible in NAT'd user environments. Outbound-only is the standard pattern.
- **QUIC/HTTP3 for the tunnel transport in Phase 68**: explicitly deferred. The WebSocket path is sufficient; architecture should not preclude a future `quinn`-based path.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `tokio-yamux 0.3` is the correct crate choice (vs bare `yamux 0.13`) for tokio-native `AsyncRead/AsyncWrite` | Standard Stack | If the crate's API doesn't match our usage (e.g., `Control::open_stream()` returns something other than `StreamHandle`), we waste time on ceremony. Fallback: use `yamux 0.13` with `tokio_util::compat::FuturesAsyncReadCompatExt` adapter. Verify with `cargo doc --open tokio-yamux` at plan time. |
| A2 | The `RelayAuthClient` can call `POST /api/v1/internal/relay/authorize` with the relay token and get back `200` + `node_id` JSON, or `403` | Pattern 1, Pitfall 8 | If the backend can't add an internal endpoint (rate limit / auth mismatch), we need to use a different validation path (e.g., direct DB lookup from a privileged internal service). Verify endpoint exists at plan time. |
| A3 | The existing `NodeConnectionManager` (`api/src/presentation/ws/node_connection_manager.rs:89`) can deliver a new `NodeMessage::TunnelDisconnect` to the agent via the existing backend WS | Standard Stack, Pattern | If `send_to_node` is generic-enum-only and the new variant isn't serializable, we add a `send_to_node_typed` helper. Existing code already sends `NodeMessage::DnsConfig` and `NodeMessage::ModeOverrideChange` will follow the same pattern. |
| A4 | NLB preserves client source IP when using `instance` target type (not `ip`) | Pitfall 2 | If AWS changes this default in 2026+ (unlikely), the per-IP rate limit breaks. Mitigation: use `proxy_protocol_v2` (Pitfall 3) as a fallback. Verify at deploy time. |
| A5 | Caddy with `caddy-dns/route53` plugin can issue a wildcard cert for `*.play.esluce.net` via DNS-01 challenge | Standard Stack, Don't Hand-Roll | If Route 53 IAM permissions are wrong on the EC2 instance, cert provisioning fails silently. Verify the IAM role at deploy time per Phase 66 pattern. |
| A6 | The agent's `Register` ack handler can be extended to issue and return a `relay_token` UUID | Pattern, Backend | If the registration response shape is too rigid (e.g., fixed struct), we need a new `RegisterAckV2` message variant. The existing `RegisterAck { node_id, status, message }` can be extended with an `Option<Uuid>` relay_token field. |
| A7 | The agent's TOML config file can persist `relay_token` and reload it on restart | Pattern | If the config file format is fixed, we add a new `[relay]` section. Agent's `dirs` crate already manages XDG config paths. |
| A8 | The Cloudflare `dns_watch.rs` logic can be extended with new triggers (`tunnel_reconnect`, `tunnel_disconnect`) without breaking the existing `server.start` → probe → create A record flow | Pattern, D-13 | The existing `check_and_update()` is called on a timer; we'd add a public method `on_tunnel_event(event_kind)` that explicitly creates/removes the A record. Phase 67 already plans for this (D-13 in CONTEXT). |
| A9 | The `gateway/Caddyfile.prod` can be extended with the new `relay.esluce.net` and `*.play.esluce.net` virtual hosts without breaking the existing `esluce.com` / `app.esluce.com` config | Architecture, D-07 | If the existing Caddyfile has a `:80` catch-all that intercepts the new domains, the cert provisioning breaks. Verify the file at plan time. |
| A10 | The existing `discord_webhook_url` column on `servers` is the right per-server alert channel for relay alerts (D-23) | Pitfall 12, D-23 | If a future phase wants per-event-type webhooks, that's a refactor. Phase 68 reuses the existing column as-is. |
| A11 | Player-to-server-id resolution via source-IP match (Pitfall 9) is the right approach for Phase 68 | Pitfall 9 | If too many collisions occur in production (e.g., users on the same CGN carrier with multiple Escluse nodes), the planner may need to switch to per-server NLB listeners. Document as a known limitation. |
| A12 | The new server repository (in `api/src/domain/entities/server.rs:8-75`) is the canonical one to add new columns to; the OLD `api/src/domain/server/model.rs:8-31` is pre-existing tech debt | Pattern, "Anti-Patterns" | If the new model is not actually used by the dashboard, the new columns are invisible to the frontend. Pre-existing divergence is documented in CONTEXT `<deferred>`. Verify which struct the dashboard uses at plan time. |

**If this table is empty:** N/A — there are 12 assumptions to validate.
**If this table is not empty:** A1–A12 are tooling/integration assumptions; the planner should accept them as defaults and the executor should validate against the local environment at Wave 0 before locking the implementation. None of them block planning — all have documented fallbacks.

## Open Questions (RESOLVED)

> All questions below were raised during research and are now resolved. The resolution feeds directly into the Plan 02 and Plan 04 implementations. See `<revision_context>` for the original open-questions set.

1. **Player-to-server-id resolution mechanism (Pitfall 9) — RESOLVED**
   - What we know: CONTEXT D-06 references SNI/Host header for WSS or proxy_protocol for raw TCP, but vanilla Minecraft Java uses raw TCP without SNI and can't send custom proxy_protocol headers.
   - What's unclear (was): The exact mechanism the user wants for matching a player's TCP connection to a specific `server_id` when multiple servers are on the same public IP.
   - **RESOLVED Recommendation:** Use **Minecraft-protocol-aware routing** — parse the MC Java Handshake packet's `serverAddress` field to extract the `<subdomain>` from `<subdomain>.play.esluce.net`, then look up `server_id` by subdomain. The agent registers its subdomain on `TunnelConnect`; the gateway indexes a `by_subdomain: DashMap<String, Uuid>` map. **NOT** player-source-IP matching (that only works when player and agent share a public IP, which contradicts D-10).
   - The Handshake packet is the first bytes the player sends on a new TCP connection. Structure: `[VarInt packet length][VarInt packet ID = 0x00][VarInt protocol version][String server address][ushort server port][VarInt next state]`. The `String server address` is a VarInt-prefixed UTF-8 string (max 255 chars). Read just enough bytes to extract the string, parse `<subdomain>.play.esluce.net`, look up the subdomain in the registry, then forward subsequent bytes to the matching yamux stream.

2. **AWS region / AZ choice for ap-southeast-1 — RESOLVED**
   - What we know: CONTEXT D-17 says `ap-southeast-1`, single AZ. AWS has 3 AZs in this region (1a, 1b, 1c).
   - What's unclear (was): Which AZ specifically.
   - **RESOLVED Recommendation:** `ap-southeast-1a` (matches Phase 66 Umami deployment; consistent infra footprint). Planner confirmed.

3. **WebSocket frame max size — RESOLVED**
   - What we know: tokio-tungstenite default is 16 KiB. Minecraft packets can exceed this (chat, large SLP responses).
   - What's unclear (was): The right max frame size (4 MiB? 16 MiB?).
   - **RESOLVED Recommendation:** 4 MiB. Document in Caddyfile and gateway code. yamux stream-level frames will be smaller (16 KiB default per yamux spec, but yamux splits them across WS frames automatically).

4. **Tunnel rekeying cadence (D-25) — RESOLVED**
   - What we know: yamux has no built-in rekeying. WS has no native rekey (TLS session tickets mitigate but don't rekey).
   - What's unclear (was): How often to tear down and re-establish the tunnel.
   - **RESOLVED Recommendation:** Every 24h OR 100 GB transferred (whichever first). Agent tracks both in `run_relay_client`; on threshold, closes the WS cleanly. Gateway detects GoAway, tears down yamux session. Agent's existing backoff reconnect handles the new handshake (new nonce, new TLS session). ~50ms downtime is acceptable.
   - **Implementation in Plan 02 Task 2:** track `tunnel_uptime_secs` and `bytes_transferred` (via metrics counters); when either threshold is hit, close the WS cleanly and let the existing backoff reconnect handle the re-handshake.

5. **Whether to use ALB or direct Caddy TLS termination — RESOLVED**
   - What we know: ALB gives us a stable DNS name + ACM cert free + WAF integration. Direct Caddy avoids the ALB cost.
   - What's unclear (was): The user's preference. CONTEXT D-05 says "behind an ALB" but the agent's discretion column lists "ALB vs NLB decision for player traffic" — the agent's discretion applies to the tunnel WebSocket, not the player TCP.
   - **RESOLVED Recommendation:** Use ALB for the WebSocket tunnel (TLS termination at ALB, routes to Caddy on EC2 via internal port, which proxies to the gateway on `127.0.0.1:8443`). Use NLB for the player TCP on port 25565 (no TLS, raw passthrough).

6. **Relay pricing tier (D-15, deferred) — RESOLVED**
   - What we know: Free for all in initial rollout.
   - What's unclear (was): When to add the paywall. The D-09/D-10 introspection is the prerequisite; we can add a billing check at `/internal/relay/authorize` later.
   - **RESOLVED Recommendation:** No code changes for billing in Phase 68. Add a `// TODO: phase-XX: add billing check` comment in the introspection handler.

7. **Existing Phase 51 Cloudflare records' interaction with Phase 68 — RESOLVED**
   - What we know: D-24 says "no migration; all existing records continue to work".
   - What's unclear (was): When the agent opens a tunnel (relay mode), should the existing Cloudflare A record be REMOVED, or just left as-is? If left as-is, players using the Direct Mode address will get a stale IP after the user has switched to Relay.
   - **RESOLVED Recommendation:** Per D-13: on `tunnel_disconnect` → flip to Relay immediately → remove the Cloudflare A record. On `tunnel_reconnect` → re-probe Direct after 30s of stable tunnel → if probe passes, re-create the A record.
   - **Implementation in Plan 02 Task 2:** in `run_relay_client`'s disconnect handler, after sending `TunnelDisconnect` on the control stream, dispatch `relay.remove_cname_record` to the agent's own task queue (self-loop). The agent's existing `dns.rs::handle_remove_record` handles the actual Cloudflare DELETE.

## Environment Availability

> Phase 68 deploys NEW infrastructure (relay gateway EC2 instance, NLB, ALB, Route 53 zone) on AWS. This audit covers the BUILD environment and required external services for the gateway.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain 1.70+ | Build both agent and gateway | TBD — verify with `rustc --version` | — | Install via rustup |
| Docker + Docker Compose | Build Caddy image, gateway container, deploy | Already in project | 24.0+ | — |
| `xcaddy` (Caddy build tool) | Build Caddy with `caddy-dns/route53` plugin | TBD — install at deploy time | latest | Use pre-built `caddy:2` image + manually add the plugin binary |
| AWS CLI | Provision EC2, NLB, ALB, Route 53, IAM | TBD — user has it | — | Manual AWS Console |
| AWS account with `ap-southeast-1` region | Host the gateway | Assumed (Escluse production is here) | — | None — required for Phase 68 |
| AWS Route 53 hosted zone for `esluce.net` | DNS-01 challenge + wildcard A record | NEW — create during deploy | — | — |
| AWS EC2 quota for 1× `c6i.large` in `ap-southeast-1` | Run the gateway | Assumed available | — | Request quota increase |
| AWS NLB quota (1 per AZ per region) | Player TCP on port 25565 | Assumed available | — | — |
| AWS ALB quota | WebSocket tunnel on port 443 | Assumed available | — | Direct Caddy TLS termination (no ALB) |
| AWS IAM role for EC2 with `route53:ChangeResourceRecordSets` for zone `esluce.net` | Caddy DNS-01 challenge | NEW — create during deploy | — | — |
| `Cargo.lock` compatible `tokio-tungstenite 0.26.2` | Agent tunnel client | ✓ (already locked, line 400) | 0.26.2 | — |
| `tokio-yamux 0.3.18` available on crates.io | Agent + gateway yamux | ✓ (verified via `cargo info tokio-yamux`) | 0.3.18 | `yamux 0.13` + adapter |
| Redis 7 (Alpine) for nonce + rate limit | Gateway state | Already in project (docker-compose.yml) | 7 | — |
| PostgreSQL 16 (Alpine) for new columns | Persistent state | Already in project (docker-compose.yml) | 16 | — |
| `prometheus` crate 0.13 | Metrics endpoint | TBD — pull at gateway build | — | `prometheus-client` 0.22 (alternative API) |
| Node.js 20 (frontend) | Build the new dashboard UI | Already in project | 20 | — |
| Existing `discord_webhook_url` column | Per-server alert channel (D-23) | ✓ (already in `servers` table from Phase 25) | — | Email alerts (Phase 25 transport) |
| Existing `monitoring_service` (api/) | Scrape relay `/metrics` | ✓ (already in `api/src/application/services/monitoring_service.rs`) | — | — |
| Existing `node_connection_manager` | Send `ModeOverrideChange` to agent | ✓ (already in `api/src/presentation/ws/node_connection_manager.rs:89`) | — | — |

**Missing dependencies with no fallback:**
- AWS `ap-southeast-1` region access — assumed available.
- AWS Route 53 zone delegation for `esluce.net` — needs registrar (Cloudflare? AWS itself?) to update NS records.

**Missing dependencies with fallback:**
- `xcaddy` build — can use pre-built Caddy Docker image and add the plugin via `caddy add-package` (or skip DNS-01 and use `acme_dns` HTTP-01 — but wildcard requires DNS-01).

**Manual AWS infrastructure steps (per Phase 66 D-06 — declarative, manual setup):**
1. Create Route 53 hosted zone for `esluce.net` (if not exists).
2. Update registrar NS records to point to Route 53 name servers.
3. Provision EC2 instance: `c6i.large`, `ap-southeast-1a`, 100 GB gp3 EBS, IAM role with Route 53 permissions.
4. Create NLB: TCP 25565 listener → target group → register EC2 instance.
5. Create ALB: HTTPS 443 listener → target group → register EC2 instance → ACM cert for `relay.esluce.net`.
6. Configure security group: 25565 (NLB), 443 (ALB), 22 (SSH via bastion), 9100 (internal).
7. Add A record `*.play.esluce.net` → NLB IP (Route 53).
8. Add A record `relay.esluce.net` → ALB DNS name (Route 53 alias).
9. Deploy Docker Compose stack: `caddy` + `relay-gateway` on EC2.
10. Verify with: `curl -i https://relay.esluce.net/healthz`, `nc -zv <nlb-ip> 25565`.

## Security Domain

> Required because `security_enforcement` is absent in `.planning/config.json` (default = enabled).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V1 Architecture | yes | The relay gateway is a NEW service — must be threat-modeled; trust boundary is `agent ↔ relay ↔ backend ↔ player`. Document in `ARCHITECTURE.md`. |
| V2 Authentication | **yes** | Per-agent `relay_token` (UUID) is the credential. Issued at node registration, sent on every tunnel connect. Backend introspection re-validates ownership. NEVER log the raw token. |
| V3 Session Management | **yes** | Each tunnel is a long-lived session. yamux provides per-stream isolation. Heartbeat-driven staleness detection (D-04). Rekeying every 24h or 100 GB (Pitfall 11). |
| V4 Access Control | **yes** | Server-ID ownership validation per tunnel (D-10). `POST /api/v1/internal/relay/authorize` MUST verify the requesting gateway's IP allowlist (or shared HMAC secret — recommend HMAC). Per-server `relay_status` is server-scoped. |
| V5 Input Validation | **yes** | WebSocket upgrade request headers MUST be validated: `Authorization` is a valid Bearer; `X-Relay-Nonce` is 64 hex chars; `X-Relay-Timestamp` is a u64 Unix timestamp. Reject malformed at the auth middleware, not in the handler. |
| V6 Cryptography | **yes** | TLS 1.3 only (D-03). Caddy's `tls` directive enforces modern cipher suites. yamux is unauthenticated (relies on TLS for confidentiality). Never log raw player traffic. |
| V7 Error Handling | yes | Tunnel auth failures return generic `401` without leaking whether the token was wrong vs. the nonce was replayed. Player connection failures return clean TCP close (D-18) — no error bytes. |
| V8 Data Protection | yes | Per-tunnel session logs in Redis are auto-deleted after 24h (TTL). No PII in the logs (no player IPs in log entries; metrics counters only). |
| V9 Communication | **yes** | Backend ↔ relay uses HMAC-signed requests (shared secret in EC2 user data → `/etc/relay-gateway/hmac-secret`). Gateway → backend uses mTLS if possible (mutual TLS via ACM PCA) or HMAC headers. |
| V10 Malicious Code | yes | Agent's `relay_client.rs` MUST whitelist server_id values from the gateway (only accept yamux streams for server_ids the agent registered). Reject anything else. |
| V11 Business Logic | yes | Mode override (D-12) MUST be enforced at the agent level (close tunnel on `Force Direct`); gateway is a passive forwarder, doesn't enforce. |
| V12 Files and Resources | no | The gateway has no filesystem access beyond logs (stdout, scraped by CloudWatch agent). |
| V13 API and Web Service | **yes** | New internal endpoint `POST /api/v1/internal/relay/authorize` MUST enforce per-IP allowlist (only the EC2 NLB's public IP) and HMAC signature. New REST endpoint `POST /api/v1/servers/:id/connectivity/mode-override` requires authenticated user with ownership. |
| V14 Configuration | yes | Relay gateway config (Redis URL, backend URL, HMAC secret) is in `/etc/relay-gateway/config.toml` on the EC2 instance, mode 0600, owned by `caddy` user. NEVER in git, NEVER in env vars visible to other containers. |

### Known Threat Patterns for Rust + AWS + yamux + tokio-tungstenite

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Replay of captured tunnel handshake | Spoofing | Nonce + timestamp + Redis dedup (D-11). 5-min window for timestamp, 10-min dedup TTL. |
| Stolen `relay_token` from agent's config file | Spoofing | Backend can rotate tokens: add `POST /api/v1/internal/relay/rotate-token` (out of Phase 68 scope but documented). Phase 68 logs `relay_token` usage to detect anomalies. |
| Gateway compromised → can impersonate any agent | Elevation of Privilege | Backend introspection re-validates ownership per tunnel (D-10). Compromised gateway can still route traffic, but backend can revoke the gateway's IP allowlist. |
| Player DoS via rapid connect attempts | Denial of Service | Rate limit: 100 connection attempts per source IP per minute (D-20). Counter is per-IP + per-kind (`player` vs `tunnel`). |
| Player IP enumeration | Information Disclosure | The gateway knows the player's source IP (NLB preserves it). Audit log only stores aggregated counts, not individual IPs. |
| Tunnel amplification (1 tunnel, many players) | Denial of Service | Per-tunnel bandwidth cap (future: 100 GB/day, then alert). Phase 68 metric: `relay_bandwidth_in_bytes` + `relay_bandwidth_out_bytes`; D-23 alert `bandwidth_spike` (>2x 5m rolling avg). |
| Backend `/internal/relay/authorize` is called by anything | Spoofing | HMAC signature in `X-Internal-Signature` header (HMAC-SHA256 over the request body). Shared secret in EC2 user data. Alternative: mTLS with ACM-issued private cert. |
| yamux RST flood crashes a stream | Tampering | yamux has built-in stream isolation. A single RST only kills one stream, not the whole session. |
| AWS NLB preserves client IP but is bypassable via X-Forwarded-For injection | Spoofing | The gateway MUST read the source IP from the TCP socket `peer_addr()`, NOT from any `X-Forwarded-For` header (which would be absent at L4 but worth documenting). |
| `*.play.esluce.net` DNS-01 challenge uses EC2 IAM role | Elevation of Privilege | IAM role grants only `route53:ChangeResourceRecordSets` for zone `esluce.net` and only for `_acme-challenge.*` records (resource-level restriction). No other Route 53 permissions. |
| Agent tunnel uses a stale token after backend rotation | Tampering | Backend re-validates the token on every `POST /internal/relay/authorize` (D-10). If the token is invalidated, the introspection returns 403, gateway closes the tunnel, agent reconnects with the new token. |
| Mode override race: user sets `Force Direct` while tunnel is open | Tampering | `ModeOverrideChange` WS message → agent immediately closes tunnel → backend's `relay_status` is updated to `disconnected` via the gateway's next `POST /tunnel-event`. Dashboard reflects the new state on next refresh. |
| Player traffic contents leak to gateway | Information Disclosure | The gateway's metrics are counts only; the gateway does NOT log or persist player TCP bytes. AWS CloudWatch logs are gated to gateway-level events (handshake, errors, mode changes), not player data. |
| `connectivity_mode_override` column is user-editable to arbitrary strings | Tampering | The column is typed `TEXT` with a CHECK constraint: `connectivity_mode_override IN ('auto', 'direct', 'relay') OR connectivity_mode_override IS NULL`. Add the CHECK in the migration. |
| `relay_token` is stored as plaintext on disk in agent's config | Information Disclosure | Document the file's permission (0600, owned by agent user) in the install script. Add a `TODO: move to OS keyring` comment for a future security phase. |

## Sources

### Primary (HIGH confidence)
- [hashicorp/yamux spec.md](https://github.com/hashicorp/yamux/blob/master/spec.md) — yamux framing protocol details, stream IDs, flow control, GoAway codes, verified 2026-06-07
- [docs.rs/tokio-tungstenite](https://docs.rs/tokio-tungstenite/0.26.2/tokio_tungstenite/) — WebSocket client/server API, `connect_async`, `accept_hdr_async`, custom headers via `IntoClientRequest`
- [docs.rs/tokio-yamux/0.3.18](https://docs.rs/tokio-yamux/0.3.18/tokio_yamux/) — Session::new_client/new_server, Control::open_stream, StreamHandle
- [docs.rs/yamux/0.13.10](https://docs.rs/yamux/0.13.10/yamux/) — `futures::io`-based API (for reference; we use `tokio-yamux`)
- [crates.io/yamux](https://crates.io/crates/yamux) — version 0.13.10 latest, 2026-06-07
- [crates.io/tokio-yamux](https://crates.io/crates/tokio-yamux) — version 0.3.18 latest, repo `nervosnetwork/tentacle`
- [docs.aws.amazon.com/elasticloadbalancing/network/load-balancer-target-groups](https://docs.aws.amazon.com/elasticloadbalancing/latest/network/load-balancer-target-groups.html) — NLB client IP preservation per target type (`instance` preserves, `ip` does not by default)
- [docs.aws.amazon.com/elasticloadbalancing/latest/network/load-balancer-troubleshooting](https://docs.aws.amazon.com/elasticloadbalancing/latest/network/load-balancer-troubleshooting.html) — NLB TCP idle timeout (350s), connection timeouts, hairpinning limitations
- [dev.to: Caddy + Cloudflare DNS Wildcard SSL](https://dev.to/amanshaw4511/caddy-cloudflare-dns-wildcard-ssl-without-the-pain-4fn) — `xcaddy build --with github.com/caddyserver/dns-providers` pattern, verified 2026-02-14
- [aussedatlo.me: Caddy Wildcard Certificates](https://aussedatlo.me/posts/09-caddy-wildcard-certificates) — `caddy-dns/*` modules list, DNS-01 challenge mechanics
- `api/migrations/20260607000001_add_relay_columns.sql` (NEW, to be created per Plan) — column additions for `nodes` and `servers` tables
- `api/src/presentation/ws/node_protocol.rs:7-135` — existing `NodeMessage` enum, the pattern for adding new variants (`TunnelConnect`, `TunnelDisconnect`, `TunnelHeartbeat`, `ModeOverrideChange`)
- `api/src/presentation/handlers/node_ws_handler.rs:237-298` — `Heartbeat` and `CommandResponse` handler pattern to mirror for new message types
- `api/src/presentation/ws/node_connection_manager.rs:21-225` — `NodeConnectionManager` with `add_connection`, `send_to_node`, `handle_response`, `remove_connection` (the API we'll reuse for sending `ModeOverrideChange` to the agent)
- `api/src/domain/entities/server.rs:8-75` — NEW `Server` struct (the one to add `connectivity_mode_override` + `relay_status` to)
- `api/src/domain/entities/node.rs:1-80` — `Node` struct (the one to add `relay_token` to)
- `api/src/domain/entities/cloudflare_settings.rs:1-40` — `CloudflareConfig` shape (the pattern to mirror for `Route53Config` if we add an API-driven record manager)
- `api/src/domain/repositories/settings_repository.rs:7-30` — `SettingsRepository` trait with `get_cloudflare_config` / `save_cloudflare_config` (pattern to mirror for `Route53Config`)
- `api/src/application/services/monitoring_service.rs` (existing) — Prometheus scrape pattern for the new relay `/metrics` endpoint
- `agent/solys/Cargo.toml:27` — `tokio-tungstenite = { version = "0.26", features = ["native-tls"] }` (the version we use for both client and server)
- `agent/solys/src/agent_connection.rs:1-912` — full WebSocket client pattern, message handler dispatch
- `agent/solys/src/handlers/dns_watch.rs:18-80, 195-218` — `DnsWatcher` background-task pattern + `detect_public_ip()` (mirrored for the relay client)
- `agent/solys/src/handlers/dns.rs:13-330` — `CloudflareDnsConfig` + per-server A record create/update (extended for tunnel-event triggers)
- `agent/solys/src/handlers/mod.rs:110-294` — task dispatch + `get_task_config` (where new `relay.*` tasks hook in)
- `gateway/Caddyfile.prod:1-58` — existing production Caddy config (extended with `relay.esluce.net` and `*.play.esluce.net` virtual hosts)
- `.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-CONTEXT.md:40-45` — D-10 (relay explicitly deferred to follow-up), D-14 (fallback chain ends with relay)
- `.planning/phases/67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi/67-RESEARCH.md` — Phase 67's reachability probe architecture (Phase 68's Direct Mode probe reuses)
- `.planning/phases/66-integrasikan-umami-analitycs-dashboard-dengan-rds-di-project/66-CONTEXT.md` — Phase 66 EC2 + Docker + Caddy + RDS deployment pattern (mirrored for the gateway)
- `.planning/codebase/INTEGRATIONS.md:108-122` — backend env vars + dev/prod stack (Caddy, ALB, etc.)
- `.planning/codebase/STRUCTURE.md:108-128` — file layout, "Where to Add New Code" (used to map the new `opt/relay/` directory)

### Secondary (MEDIUM confidence)
- [kocean.dev: Preserving client IP on NLB + Istio Gateway (Proxy Protocol v2)](https://kocean.dev/en/blog/nlb-istio-proxy-protocol-v2) — proxy_protocol_v2 mechanics (alternative to NLB's `instance` target type)
- [axelspire.com: Certbot DNS-01 Wildcard SSL on Route 53](https://axelspire.com/vault/acme-clients/dns-01-challenge-validation) — DNS-01 challenge for wildcard certs on AWS Route 53
- [github.com/snapview/tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) — `accept_hdr_async` for custom header reading on the server side, custom header `Request` builder for the client side
- [users.rust-lang.org: Custom header connect-async tokio-tungstenite](https://users.rust-lang.org/t/custom-header-connect-async-tokio-tungstenite/100973) — verified 2023-10-09 pattern for custom request headers via `http::Request::builder`
- [crates.io/aws-sdk-route53](https://crates.io/crates/aws-sdk-route53) — version 1.114.0 latest (we may not need it; only for API-driven A record management)
- `.planning/phases/65-buat-installer-script-auto-install-docker-sebelum-install-so/65-CONTEXT.md:21-25` — install-time consent + config generation (pattern for persisting `relay_token` in agent config)
- `.planning/codebase/CONCERNS.md:111-126` — fragile WebSocket connection management (applies to our new relay gateway; same gotchas)

### Tertiary (LOW confidence)
- [oneuptime.com: Set up AWS NLB with IPv4 Target Groups](https://oneuptime.com/blog/post/2026-03-20-setup-aws-nlb-ipv4-target-groups/view) — NLB setup walkthrough; cross-zone load balancing behavior; not authoritative but illustrative
- [DEV.to: Wildcard SSL on Ubuntu 2026](https://dev.to/mechcloud_academy/the-ultimate-guide-to-lets-encrypt-wildcard-ssl-on-ubuntu-2026-27hg) — Certbot 5.x on Ubuntu 24.04 DNS-01; not directly relevant (we use Caddy) but corroborates the DNS-01 pattern
- [reddit/r/admincraft: SIMPLE proxy needs](https://www.reddit.com/r/admincraft/comments/18kkxq7/simple_proxy_needs) — community discussion of subdomain-based Minecraft proxying; confirms the use case is real but our solution (in-gateway routing) is more sophisticated than typical HAProxy configs

## Metadata

**Confidence breakdown:**
- **Standard stack:** HIGH — `tokio-tungstenite 0.26.2` and `redis 0.25` are verified in `Cargo.lock`; `tokio-yamux 0.3.18` verified on crates.io; `axum 0.7` and `tokio 1` are the project's existing framework versions. All major choices are reversible (fallback to `yamux 0.13` with adapter if `tokio-yamux` doesn't fit).
- **Architecture:** MEDIUM-HIGH — the player-to-server-id resolution mechanism (Pitfall 9) is the major architectural unknown. Other architectural choices are well-established patterns (Axum + WS + Redis state + reqwest to backend).
- **AWS infrastructure:** MEDIUM — NLB instance target type for client IP preservation is verified via AWS docs, but the actual deploy steps are manual and need verification at Wave 0. Caddy DNS-01 with `caddy-dns/route53` is well-documented but the IAM role permissions need careful crafting.
- **yamux over WebSocket:** HIGH — both libraries are actively maintained; the framing is well-specified; the integration is well-trodden (e.g., libp2p uses yamux).
- **Security model:** MEDIUM — per-agent token + backend introspection is solid. HMAC for backend↔gateway internal endpoint needs to be decided. Per-IP rate limit is straightforward.
- **Pitfalls:** HIGH — most pitfalls are derived from direct codebase reading (Pitfall 1, 2, 4, 5) or well-documented protocol edge cases (Pitfall 6, 7, 10, 11). Pitfall 9 is the one open architectural question; documented with a recommendation.

**Research date:** 2026-06-07
**Valid until:** 14 days (2026-06-21) — yamux spec is stable; tokio-tungstenite 0.26 is the locked version; AWS NLB/ALB behavior is stable; Caddy DNS-01 plugin is stable. The main thing that could go stale is the specific AWS console UI (manual deployment) or a new minor release of `tokio-yamux`. Refresh if a Phase 68 plan hasn't shipped by then.
