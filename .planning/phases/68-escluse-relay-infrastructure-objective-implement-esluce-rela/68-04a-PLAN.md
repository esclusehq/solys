---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 04a
type: execute
wave: 3
depends_on:
  - 68-01
  - 68-02
  - 68-03
files_modified:
  - Cargo.toml
  - opt/relay/Cargo.toml
  - opt/relay/relay-gateway.toml
  - opt/relay/.env.example
  - opt/relay/src/main.rs
  - opt/relay/src/config.rs
  - opt/relay/src/state.rs
  - opt/relay/src/error.rs
  - opt/relay/src/auth.rs
  - opt/relay/src/registry.rs
  - opt/relay/src/tunnel.rs
  - opt/relay/src/player.rs
  - opt/relay/src/backend.rs
  - opt/relay/src/heartbeat.rs
  - opt/relay/src/metrics.rs
  - opt/relay/src/ratelimit.rs
  - opt/relay/src/session_log.rs
autonomous: true
requirements:
  - DEPLOY-01
  - DEPLOY-02
  - DEPLOY-03
  - DEPLOY-05
  - STATUS-01
  - STATUS-02

must_haves:
  truths:
    - "Gateway accepts outbound WSS from agents on wss://relay.esluce.net/relay/tunnel, authenticates via HMAC-signed POST to backend's /internal/relay/authorize, then opens yamux"
    - "Gateway accepts raw TCP on :25565 (NLB passthrough), parses the Minecraft Java Handshake packet to extract the subdomain from <subdomain>.play.esluce.net, looks up server_id by subdomain, and proxies to the matching yamux stream"
    - "Gateway enforces 1 active tunnel per server_id (D-08, D-21); a second TunnelConnect for the same server_id drops the older tunnel"
    - "Gateway enforces 100 req/min per source IP rate limit at the player TCP layer (D-20) via in-process token bucket (single-instance Phase 68 scope per D-05)"
    - "Gateway publishes /metrics on :9100 for Prometheus scraping (D-22): active_tunnels, total_connections, rejected_connections, auth_failures, rate_limited, bandwidth_in/out per subdomain — NO relay_mode_distribution (computed in backend from servers table per WARN 9)"
    - "Gateway runs as a Docker container on AWS EC2 c6i.large in ap-southeast-1a, fronted by Network Load Balancer for raw TCP and by Caddy for TLS 1.3 + WS upgrade"
  artifacts:
    - path: "Cargo.toml"
      provides: "Workspace root updated to include opt/relay as a member"
      contains: "members.*opt/relay"
    - path: "opt/relay/Cargo.toml"
      provides: "Workspace member for the gateway crate; declares all dependencies"
      contains: "name = \"relay-gateway\""
    - path: "opt/relay/relay-gateway.toml"
      provides: "Runtime config: tunnel_bind, player_bind, metrics_bind = 0.0.0.0:9100 (D-22, not 9090)"
      contains: "metrics_bind = \"0.0.0.0:9100\""
    - path: "opt/relay/src/main.rs"
      provides: "Service entrypoint; loads config, builds state, starts axum, starts metrics server, blocks on shutdown"
      contains: "fn main"
    - path: "opt/relay/src/auth.rs"
      provides: "HMAC-signed POST to backend /internal/relay/authorize with replay protection"
      contains: "pub async fn authorize"
    - path: "opt/relay/src/registry.rs"
      provides: "DashMap<subdomain, TunnelHandle> AND DashMap<server_id, TunnelHandle> for Handshake-parse routing (NO by_agent_ip map per BLOCKER 1)"
      contains: "by_subdomain: DashMap<String, Uuid>"
    - path: "opt/relay/src/player.rs"
      provides: "Minecraft-protocol-aware TCP forwarder: reads VarInt length + VarInt packet ID = 0x00 + VarInt protocol version + String server address, extracts subdomain, looks up server_id, opens yamux stream"
      contains: "parse_mc_handshake_subdomain"
    - path: "opt/relay/src/tunnel.rs"
      provides: "Per-agent yamux session: accept inbound WSS, open control stream, read TunnelConnect JSON (with subdomain), register in Registry by subdomain AND by server_id"
      contains: "pub async fn run_tunnel_session"
    - path: "opt/relay/src/backend.rs"
      provides: "HMAC-signed reqwest client for POST /internal/relay/tunnel-event"
      contains: "pub async fn report_tunnel_event"
    - path: "opt/relay/src/heartbeat.rs"
      provides: "30s ticker; marks tunnels stale after 3 missed heartbeats (D-04)"
      contains: "pub async fn run_heartbeat_watcher"
    - path: "opt/relay/src/metrics.rs"
      provides: "Prometheus metrics registry and HTTP /metrics exporter on :9100 (D-22) — NO relay_mode_distribution (WARN 9)"
      contains: "pub static ref METRICS"
    - path: "opt/relay/src/ratelimit.rs"
      provides: "Per-IP token bucket; 100 req/min, refilled at 100/60 per second (D-20, in-process; Redis-backed is a horizontal-scale follow-up)"
      contains: "pub fn check_rate_limit"
  key_links:
    - from: "opt/relay/src/tunnel.rs"
      to: "opt/relay/src/auth.rs"
      via: "Tunnel session calls auth::authorize before opening yamux"
      pattern: "auth::authorize"
    - from: "opt/relay/src/tunnel.rs"
      to: "opt/relay/src/registry.rs"
      via: "Tunnel session registers server_id and subdomain in Registry on TunnelConnect (BOTH maps)"
      pattern: "registry\\.register"
    - from: "opt/relay/src/player.rs"
      to: "opt/relay/src/registry.rs"
      via: "Player listener parses MC Handshake → extracts subdomain → looks up server_id in by_subdomain DashMap (NOT by source IP per BLOCKER 1)"
      pattern: "registry\\.lookup_by_subdomain"
    - from: "opt/relay/src/heartbeat.rs"
      to: "opt/relay/src/registry.rs"
      via: "Heartbeat watcher iterates Registry and marks stale tunnels"
      pattern: "registry\\.mark_stale"
    - from: "opt/relay/src/backend.rs"
      to: "opt/relay/src/registry.rs"
      via: "Tunnel events call backend::report_tunnel_event with server_id and event_type from Registry state"
      pattern: "backend::report_tunnel_event"
---

<objective>
Build the relay gateway: a new Rust + Axum service deployed on a single AWS EC2 c6i.large in ap-southeast-1a. The gateway accepts outbound WSS tunnels from agents, authenticates them via HMAC-signed callbacks to the backend, and forwards raw TCP player connections on :25565 to the right yamux stream by parsing the Minecraft Java Handshake packet's `serverAddress` field to extract the `<subdomain>.play.esluce.net` subdomain (NOT by source-IP match per BLOCKER 1). The gateway is the only Phase 68 component that owns player-facing network exposure.

This is sub-plan 04a of a 3-part split of the original Plan 04 (the other sub-plans are 04b: Docker/Caddy/compose; 04c: DEPLOY.md).

Output:
- `Cargo.toml` (workspace root) — add `opt/relay` to members
- `opt/relay/Cargo.toml` (NEW, workspace member)
- `opt/relay/relay-gateway.toml` (NEW, config file with `metrics_bind = "0.0.0.0:9100"` per D-22)
- `opt/relay/.env.example` (NEW)
- `opt/relay/src/{main,config,state,auth,registry,tunnel,player,backend,heartbeat,metrics,ratelimit,session_log,error}.rs` (13 NEW source files — the original 12 plus the new Handshake-parse logic in player.rs)
</objective>

<execution_context>
@/home/rhnbztnl/.config/opencode/get-shit-done/workflows/execute-plan.md
@/home/rhnbztnl/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/ROADMAP.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-CONTEXT.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-RESEARCH.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-PATTERNS.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-01-PLAN.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-02-PLAN.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-03-PLAN.md
</context>

<interfaces>
From Plan 02 (the agent's WSS request shape):
```http
GET /relay/tunnel HTTP/1.1
Host: relay.esluce.net
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: <base64>
Sec-WebSocket-Version: 13
Authorization: Bearer {relay_token}
```

From Plan 02 (the first yamux stream message — note the `subdomain` field, which is what the gateway uses for routing):
```json
{"type":"tunnel_connect","server_id":"...","subdomain":"abc12345","public_port":25565,"agent_public_ip":"1.2.3.4","region":"ap-southeast-1"}
```

From Plan 03 (the internal HMAC endpoints):
- `POST /internal/relay/authorize` body: `{"relay_token":"...","server_id":"..."}`; HMAC headers `X-Esluce-Signature`, `X-Esluce-Timestamp`, `X-Esluce-Nonce`
- `POST /internal/relay/tunnel-event` body: `{"server_id":"...","event_type":"connected|disconnected|stale","reason":"..."}`

**Minecraft Java Handshake packet structure (BLOCKER 1 fix — the new routing mechanism):**
```
[VarInt packet length][VarInt packet ID = 0x00][VarInt protocol version][String server address][ushort server port][VarInt next state]
```
- The `String server address` is a VarInt-prefixed UTF-8 string (max 255 chars). Format: `<subdomain>.play.esluce.net`.
- The player sends this as the first bytes on a new TCP connection.
- Read just enough bytes to extract the VarInt length + the bytes of the string, parse the string, extract `<subdomain>` by splitting on `.play.esluce.net`, look up the subdomain in the registry, then forward subsequent bytes to the matching yamux stream.

VarInt encoding (Minecraft uses standard VarInt — 7 bits per byte, MSB = continuation):
```
fn read_varint(buf: &[u8]) -> Option<(u32, usize)> {
    let mut value: u32 = 0;
    let mut shift = 0;
    for (i, &b) in buf.iter().enumerate() {
        value |= ((b & 0x7F) as u32) << shift;
        if b & 0x80 == 0 { return Some((value, i + 1)); }
        shift += 7;
        if shift >= 35 { return None; }  // overflow
    }
    None
}
```

From `Cargo.toml` (workspace root) — need to add `opt/relay` to the workspace members list:
```toml
[workspace]
members = [".", "api", "opt/relay"]  # add opt/relay
```

From RESEARCH.md (the standard library choices):
- axum 0.7 (WS + HTTP server)
- tokio-yamux 0.2 (multiplexing)
- dashmap 6 (concurrent maps)
- hmac 0.12, sha2 0.10, hex 0.4 (HMAC)
- reqwest 0.12 (HTTP client to backend)
- prometheus 0.13 (metrics)
- tracing 0.1, tracing-subscriber 0.3
- serde, serde_json, uuid, chrono, anyhow, thiserror
- config 0.14 (config file)
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Create gateway crate with workspace membership, config, and .env.example</name>
  <files>Cargo.toml, opt/relay/Cargo.toml, opt/relay/relay-gateway.toml, opt/relay/.env.example</files>
  <read_first>
    - cat Cargo.toml (locate the [workspace] section to add opt/relay as a member)
    - ls opt/ (confirm the opt/ directory exists and contains any pre-existing crates for the pattern)
  </read_first>
  <action>
    1. Add `opt/relay` to the workspace members in the root `Cargo.toml`:
       ```toml
       [workspace]
       members = [".", "api", "opt/relay"]
       ```
       (Replace the existing `members` list with this one; preserve any other members.)

    2. Create `opt/relay/Cargo.toml` with this exact content:
       ```toml
       [package]
       name = "relay-gateway"
       version = "0.1.0"
       edition = "2021"

       [[bin]]
       name = "relay-gateway"
       path = "src/main.rs"

       [dependencies]
       axum = { version = "0.7", features = ["ws", "macros"] }
       tokio = { version = "1", features = ["full"] }
       tokio-tungstenite = "0.26"
       tokio-yamux = "0.2"
       futures = "0.3"
       dashmap = "6"
       hmac = "0.12"
       sha2 = "0.10"
       hex = "0.4"
       reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
       prometheus = "0.13"
       serde = { version = "1", features = ["derive"] }
       serde_json = "1"
       uuid = { version = "1", features = ["v4", "serde"] }
       chrono = { version = "0.4", features = ["serde"] }
       anyhow = "1"
       thiserror = "1"
       tracing = "0.1"
       tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
       config = { version = "0.14", default-features = false, features = ["toml"] }
       redis = { version = "0.25", features = ["aio", "tokio-comp", "connection-manager"] }
       base64 = "0.22"
       ```

    3. Create `opt/relay/relay-gateway.toml` (the runtime config) — note `metrics_bind = "0.0.0.0:9100"` per D-22 (NOT 9090, per WARN 5):
       ```toml
       [server]
       tunnel_bind = "0.0.0.0:8080"      # axum serves WSS upgrade + /healthz
       player_bind = "0.0.0.0:25565"     # raw TCP for MC Java players
       metrics_bind = "0.0.0.0:9100"     # Prometheus /metrics (D-22)

       [backend]
       base_url = "https://api.esluce.net"
       hmac_secret_env = "GATEWAY_HMAC_SECRET"  # read from env at startup
       request_timeout_secs = 5

       [redis]
       url = "redis://localhost:6379"     # for nonce dedup

       [tunnel]
       heartbeat_interval_secs = 10
       heartbeat_missed_threshold = 3     # mark stale after 3 missed
       max_tunnels_per_server = 1         # D-21

       [ratelimit]
       requests_per_minute = 100          # D-20 (in-process token bucket; Redis-backed is a horizontal-scale follow-up per CONTEXT D-20 RESOLVED)

       [logging]
       level = "info"
       ```

    4. Create `opt/relay/.env.example` documenting the env vars:
       ```
       GATEWAY_HMAC_SECRET=<set via AWS Secrets Manager>
       RUST_LOG=info
       RUST_BACKTRACE=1
       ```
  </action>
  <verify>
    <automated>cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && grep -n "members.*opt/relay" Cargo.toml | head -2 && echo "---" && ls opt/relay/Cargo.toml opt/relay/relay-gateway.toml opt/relay/.env.example && echo "---" && grep "metrics_bind" opt/relay/relay-gateway.toml && echo "---" && grep -c "9100" opt/relay/relay-gateway.toml</automated>
  </verify>
  <acceptance_criteria>
    - Root `Cargo.toml` `[workspace] members` includes `"opt/relay"`
    - `opt/relay/Cargo.toml` exists with all required deps
    - `opt/relay/relay-gateway.toml` exists with `metrics_bind = "0.0.0.0:9100"` (NOT 9090)
    - `opt/relay/.env.example` exists
  </acceptance_criteria>
  <done>Workspace membership, gateway Cargo.toml, runtime config (with 9100 metrics port), and .env.example in place</done>
</task>

<task type="auto">
  <name>Task 2: Create registry.rs (by_subdomain routing) and player.rs (Minecraft Handshake-packet parser)</name>
  <files>opt/relay/src/registry.rs, opt/relay/src/player.rs
  <read_first>
    - cat opt/relay/src/registry.rs (none — this is a NEW file)
    - Re-read the interfaces block in this plan (VarInt decoding snippet)
  </read_first>
  <action>
    1. Create `opt/relay/src/registry.rs` (≈120 lines). Define the routing maps:
       ```rust
       use dashmap::DashMap;
       use std::net::IpAddr;
       use std::sync::atomic::{AtomicU64, Ordering};
       use std::time::{Instant, SystemTime};
       use uuid::Uuid;
       use tokio_yamux::Control;

       pub struct TunnelHandle {
           pub server_id: Uuid,
           pub subdomain: String,                // e.g. "abc12345"
           pub agent_public_ip: String,          // for audit; not used for routing
           pub last_heartbeat: AtomicU64,        // unix seconds
           pub yamux_control: tokio::sync::Mutex<Option<Control>>,
           pub started_at: Instant,
           pub bytes_in: AtomicU64,
           pub bytes_out: AtomicU64,
       }

       #[derive(Clone, Default)]
       pub struct Registry {
           /// Primary routing index — Handshake-packet subdomain → server_id (BLOCKER 1 fix)
           pub by_subdomain: std::sync::Arc<DashMap<String, Uuid>>,
           /// Secondary index for fast lookup from the heartbeat watcher
           pub by_server_id: std::sync::Arc<DashMap<Uuid, std::sync::Arc<TunnelHandle>>>,
           /// NOTE: There is intentionally NO `by_agent_ip` map. Vanilla Minecraft Java
           /// clients do not send SNI, and the player source IP is unrelated to the
           /// server's agent public IP (the whole point of the relay is to bridge
           /// CGNAT agents to remote players). Routing is by subdomain only.
       }

       impl Registry {
           pub fn new() -> Self {
               Self {
                   by_subdomain: std::sync::Arc::new(DashMap::new()),
                   by_server_id: std::sync::Arc::new(DashMap::new()),
               }
           }

           /// Register a new tunnel. If a tunnel for the same server_id already exists,
           /// drops the older one (D-21). The agent's `subdomain` is what the gateway
           /// uses for player routing, so the same subdomain can only map to one
           /// server at a time.
           pub fn register(&self, handle: std::sync::Arc<TunnelHandle>) -> Result<(), RegistryError> {
               // Check existing tunnel for this server_id (D-21)
               if let Some(old) = self.by_server_id.insert(handle.server_id, handle.clone()) {
                   tracing::info!("[REGISTRY] Replacing existing tunnel for server_id={}", handle.server_id);
                   // Free the old subdomain (in case it differs)
                   self.by_subdomain.remove(&old.subdomain);
               }
               // Enforce 1:1 subdomain → server_id mapping
               if let Some(existing_server_id) = self.by_subdomain.insert(handle.subdomain.clone(), handle.server_id) {
                   if existing_server_id != handle.server_id {
                       // Subdomain collision: another server is using this subdomain
                       self.by_subdomain.insert(handle.subdomain.clone(), handle.server_id);
                       return Err(RegistryError::SubdomainInUse);
                   }
               }
               Ok(())
           }

           /// Look up server_id by subdomain (the Handshake-routing path used by player.rs)
           pub fn lookup_by_subdomain(&self, subdomain: &str) -> Option<Uuid> {
               self.by_subdomain.get(subdomain).map(|e| *e.value())
           }

           pub fn get(&self, server_id: &Uuid) -> Option<std::sync::Arc<TunnelHandle>> {
               self.by_server_id.get(server_id).map(|e| e.value().clone())
           }

           pub fn unregister(&self, server_id: &Uuid) {
               if let Some((_, handle)) = self.by_server_id.remove(server_id) {
                   self.by_subdomain.remove(&handle.subdomain);
               }
           }

           pub fn mark_stale(&self, server_id: &Uuid) {
               if let Some(handle) = self.by_server_id.get(server_id) {
                   tracing::warn!("[REGISTRY] Marking tunnel stale: server_id={}", server_id);
               }
               self.unregister(server_id);
           }

           pub fn iter(&self) -> impl Iterator<Item = std::sync::Arc<TunnelHandle>> + '_ {
               self.by_server_id.iter().map(|e| e.value().clone())
           }
       }

       #[derive(Debug, thiserror::Error)]
       pub enum RegistryError {
           #[error("Subdomain already in use by another server")]
           SubdomainInUse,
       }
       ```

    2. Create `opt/relay/src/player.rs` (≈140 lines). This is the BLOCKER 1 fix — the Minecraft-protocol-aware routing forwarder:
       ```rust
       use std::net::SocketAddr;
       use std::sync::Arc;
       use std::time::Duration;
       use tokio::io::{copy_bidirectional, AsyncReadExt};
       use tokio::net::TcpStream;
       use tracing::{debug, error, info, warn};
       use crate::state::AppState;

       /// The suffix every relay subdomain has on `*.play.esluce.net`.
       const RELAY_SUFFIX: &str = ".play.esluce.net";
       /// Max length of a Minecraft Java String per protocol spec (VarInt-prefixed, 32767 chars max, but DNS labels cap at 253).
       const MAX_MC_STRING_BYTES: usize = 255;
       /// How long to wait for the Handshake packet to arrive before giving up.
       const HANDSHAKE_READ_TIMEOUT: Duration = Duration::from_secs(5);

       pub async fn run_player_listener(state: Arc<AppState>) -> anyhow::Result<()> {
           let listener = tokio::net::TcpListener::bind(state.config.server.player_bind).await?;
           info!("[PLAYER] Listening on {}", state.config.server.player_bind);
           loop {
               let (stream, peer) = listener.accept().await?;
               let state = state.clone();
               tokio::spawn(async move {
                   if let Err(e) = handle_player_connection(state, stream, peer).await {
                       debug!("[PLAYER] Connection ended: {}", e);
                   }
               });
           }
       }

       async fn handle_player_connection(
           state: Arc<AppState>,
           mut tcp: TcpStream,
           peer: SocketAddr,
       ) -> anyhow::Result<()> {
           // Rate-limit per source IP (D-20)
           if !state.rate_limiter.check(peer.ip()) {
               warn!("[PLAYER] Rate-limited: peer={}", peer);
               // Clean close (D-18)
               drop(tcp);
               return Ok(());
           }

           // Read the MC Java Handshake packet (BLOCKER 1 fix).
           // The Handshake is the FIRST packet the client sends on a new TCP connection.
           // Format: [VarInt packet length][VarInt packet ID = 0x00][VarInt protocol version][String server address][ushort server port][VarInt next state]
           //
           // We only need the String server address (the rest is forwarded to the agent verbatim).
           let (subdomain, prefix_bytes) = match read_mc_handshake_subdomain(&mut tcp, peer).await {
               Ok(s) => s,
               Err(e) => {
                   warn!("[PLAYER] Failed to parse Handshake from {}: {}; closing", peer, e);
                   drop(tcp);  // D-18: clean close
                   return Ok(());
               }
           };
           debug!("[PLAYER] Handshake from {}: subdomain={}", peer, subdomain);

           // Look up the server by subdomain (NOT by source IP — see BLOCKER 1).
           let server_id = match state.registry.lookup_by_subdomain(&subdomain) {
               Some(id) => id,
               None => {
                   warn!("[PLAYER] No active tunnel for subdomain={}; closing", subdomain);
                   drop(tcp);  // D-18: clean close
                   return Ok(());
               }
           };

           let handle = match state.registry.get(&server_id) {
               Some(h) => h,
               None => {
                   warn!("[PLAYER] Registry race: subdomain={} → server_id={} but no handle; closing", subdomain, server_id);
                   drop(tcp);
                   return Ok(());
               }
           };

           // Open a new yamux stream on the agent's tunnel
           let control_lock = handle.yamux_control.lock().await;
           let control = match control_lock.as_ref() {
               Some(c) => c,
               None => {
                   warn!("[PLAYER] Tunnel handle has no yamux control (stale); closing");
                   drop(tcp);
                   return Ok(());
               }
           };
           let mut yamux_stream = match control.open_stream().await {
               Ok(s) => s,
               Err(e) => {
                   error!("[PLAYER] Failed to open yamux stream for server={}: {}", server_id, e);
                   drop(tcp);
                   return Ok(());
               }
           };

           // Forward the Handshake prefix bytes we already buffered, then bidirectionally copy.
           tokio::io::AsyncWriteExt::write_all(&mut yamux_stream, &prefix_bytes).await?;

           // Bidi copy with 5-min idle timeout (D-19)
           let copy_result = tokio::time::timeout(
               Duration::from_secs(300),
               copy_bidirectional(&mut tcp, &mut yamux_stream),
           ).await;

           match copy_result {
               Ok(Ok((in_bytes, out_bytes))) => {
                   handle.bytes_in.fetch_add(in_bytes as u64, std::sync::atomic::Ordering::Relaxed);
                   handle.bytes_out.fetch_add(out_bytes as u64, std::sync::atomic::Ordering::Relaxed);
                   crate::metrics::PLAYER_BYTES_IN.inc_by(in_bytes as f64);
                   crate::metrics::PLAYER_BYTES_OUT.inc_by(out_bytes as f64);
                   crate::session_log::log_session_end(server_id, in_bytes as u64, out_bytes as u64);
               }
               Ok(Err(e)) => {
                   debug!("[PLAYER] Bidi copy error: {}", e);
                   crate::session_log::log_session_error(server_id, &e.to_string());
               }
               Err(_) => {
                   debug!("[PLAYER] Idle timeout (5m); closing");
               }
           }
           Ok(())
       }

       /// Read the Minecraft Java Handshake packet and extract the subdomain from
       /// the `server address` field. Returns (subdomain, buffered_bytes_to_forward).
       ///
       /// The packet structure is:
       ///   [VarInt packet length][VarInt packet ID = 0x00][VarInt protocol version]
       ///   [String server address (VarInt length + UTF-8 bytes)][ushort server port]
       ///   [VarInt next state]
       ///
       /// We read at most `HANDSHAKE_READ_TIMEOUT` worth of bytes; if the Handshake
       /// doesn't arrive, we close (D-18). We buffer the bytes we read (the full
       /// Handshake packet) and return them in `prefix_bytes` so the caller can
       /// forward them to the agent (the agent expects to receive the entire
       /// Handshake packet on a fresh yamux stream, not just the server address).
       pub async fn read_mc_handshake_subdomain(
           tcp: &mut TcpStream,
           peer: SocketAddr,
       ) -> anyhow::Result<(String, Vec<u8>)> {
           // Read up to 1 KiB — the Handshake packet for typical MC clients is
           // ~30-100 bytes (server address is short: "abc12345.play.esluce.net").
           let mut buf = vec![0u8; 1024];
           let mut total_read = 0usize;
           let read_fut = async {
               loop {
                   let n = tcp.read(&mut buf[total_read..]).await?;
                   if n == 0 { break; }  // EOF
                   total_read += n;
                   // If we've read at least the packet length + packet ID + the start
                   // of the server address's VarInt length, try to parse.
                   if total_read >= 2 && try_parse_handshake(&buf[..total_read]).is_some() {
                       break;
                   }
                   if total_read >= buf.len() {
                       anyhow::bail!("Handshake packet > 1 KiB (unusual)");
                   }
               }
               Ok::<(), anyhow::Error>(())
           };
           tokio::time::timeout(HANDSHAKE_READ_TIMEOUT, read_fut).await??;

           let (subdomain, consumed) = try_parse_handshake(&buf[..total_read])
               .ok_or_else(|| anyhow::anyhow!("Incomplete or invalid Handshake"))?;

           // Sanity check: the address must end in `.play.esluce.net`
           if !subdomain.ends_with(RELAY_SUFFIX) {
               anyhow::bail!("Server address does not end in {}: {}", RELAY_SUFFIX, subdomain);
           }
           // Extract the subdomain (everything before `.play.esluce.net`)
           let subdomain = subdomain[..subdomain.len() - RELAY_SUFFIX.len()].to_string();
           if subdomain.is_empty() || subdomain.len() > 63 {
               anyhow::bail!("Subdomain length out of range: '{}'", subdomain);
           }
           // Validate charset (lowercase alphanumeric + hyphens)
           if !subdomain.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
               anyhow::bail!("Subdomain has invalid characters: '{}'", subdomain);
           }
           // Return the FULL buffered Handshake bytes so the caller can forward them
           let prefix_bytes = buf[..consumed].to_vec();
           Ok((subdomain, prefix_bytes))
       }

       /// Try to parse a Minecraft Handshake packet from `buf`. Returns
       /// `Some((server_address, bytes_consumed))` on success, `None` if the buffer
       /// is incomplete (caller should read more). See VarInt encoding in the plan's
       /// `<interfaces>` block.
       fn try_parse_handshake(buf: &[u8]) -> Option<(String, usize)> {
           let mut p = 0usize;
           // [VarInt packet length]
           let (packet_length, n) = read_varint(buf, p)?;
           p += n;
           let packet_end = p.checked_add(packet_length as usize)?;
           if buf.len() < packet_end { return None; }  // need more bytes
           // [VarInt packet ID] — must be 0x00 for Handshake
           let (packet_id, n) = read_varint(buf, p)?;
           p += n;
           if packet_id != 0x00 { return None; }  // not a Handshake
           // [VarInt protocol version] — read but don't validate value
           let (_proto, n) = read_varint(buf, p)?;
           p += n;
           // [String server address]
           let (server_address, n) = read_mc_string(buf, p)?;
           p += n;
           // [ushort server port] and [VarInt next state] follow; we don't need them
           Some((server_address, packet_end))
       }

       fn read_varint(buf: &[u8], start: usize) -> Option<(u32, usize)> {
           let mut value: u32 = 0;
           let mut shift = 0u32;
           let mut i = start;
           loop {
               if i >= buf.len() { return None; }
               let b = buf[i];
               value |= ((b & 0x7F) as u32) << shift;
               i += 1;
               if b & 0x80 == 0 { return Some((value, i - start)); }
               shift += 7;
               if shift >= 35 { return None; }  // VarInt overflow
           }
       }

       fn read_mc_string(buf: &[u8], start: usize) -> Option<(String, usize)> {
           let (byte_len, n) = read_varint(buf, start)?;
           let byte_len = byte_len as usize;
           if byte_len > MAX_MC_STRING_BYTES { return None; }
           let str_start = start + n;
           let str_end = str_start.checked_add(byte_len)?;
           if buf.len() < str_end { return None; }
           let s = std::str::from_utf8(&buf[str_start..str_end]).ok()?;
           Some((s.to_string(), n + byte_len))
       }
       ```
  </action>
  <verify>
    <automated>cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && ls opt/relay/src/registry.rs opt/relay/src/player.rs && echo "---" && grep -n "by_subdomain\|by_server_id\|by_agent_ip" opt/relay/src/registry.rs | head -10 && echo "---" && grep -n "parse_mc_handshake_subdomain\|read_mc_handshake_subdomain\|read_varint\|read_mc_string\|RELAY_SUFFIX" opt/relay/src/player.rs | head -10</automated>
  </verify>
  <acceptance_criteria>
    - `opt/relay/src/registry.rs` exists with `by_subdomain: DashMap<String, Uuid>` and `by_server_id: DashMap<Uuid, Arc<TunnelHandle>>`
    - There is NO `by_agent_ip` map in the registry (BLOCKER 1 fix)
    - `Registry::register` enforces 1 tunnel per server_id (D-21) and 1 server per subdomain
    - `opt/relay/src/player.rs` has `read_mc_handshake_subdomain` that parses the MC Java Handshake VarInt-prefixed string and extracts the subdomain
    - `Registry::lookup_by_subdomain` is the routing primitive used by `player::handle_player_connection`
    - `try_parse_handshake` correctly handles partial buffers (returns None until the full Handshake has arrived)
  </acceptance_criteria>
  <done>Registry uses by_subdomain (not by_agent_ip); player.rs parses MC Handshake to extract subdomain per BLOCKER 1</done>
</task>

<task type="auto">
  <name>Task 3: Create the remaining 11 gateway source files (main, config, state, error, auth, tunnel, backend, heartbeat, metrics, ratelimit, session_log)</name>
  <files>opt/relay/src/main.rs, opt/relay/src/config.rs, opt/relay/src/state.rs, opt/relay/src/error.rs, opt/relay/src/auth.rs, opt/relay/src/tunnel.rs, opt/relay/src/backend.rs, opt/relay/src/heartbeat.rs, opt/relay/src/metrics.rs, opt/relay/src/ratelimit.rs, opt/relay/src/session_log.rs</files>
  <read_first>
    - cat opt/relay/src/registry.rs (read Task 2's output; the new files will import from it)
    - Re-read the interfaces block in this plan for the WSS request shape and HMAC endpoints
  </read_first>
  <action>
    1. Create `opt/relay/src/main.rs` (≈80 lines): `fn main() -> anyhow::Result<()>` reads `relay-gateway.toml` + env, builds `AppState`, spawns 4 tokio tasks (tunnel listener on :8080, player listener on :25565, metrics on :9100, heartbeat watcher), waits on `tokio::signal::ctrl_c()`, and shuts down gracefully. The Caddy reverse proxy fronts :8080 (TLS-terminated WSS path); the NLB fronts :25565 (raw TCP for player traffic).

    2. Create `opt/relay/src/config.rs` (≈60 lines): `pub struct Config` with nested `server`, `backend`, `redis`, `tunnel`, `ratelimit`, `logging` substructs matching relay-gateway.toml. `pub fn load() -> Result<Config, anyhow::Error>` uses the `config` crate to read the TOML file. **Note:** `server.metrics_bind` is `0.0.0.0:9100` (NOT 9090) per D-22.

    3. Create `opt/relay/src/state.rs` (≈40 lines): `pub struct AppState { pub config: Arc<Config>, pub registry: Arc<Registry>, pub backend: Arc<BackendClient>, pub redis: redis::aio::ConnectionManager, pub rate_limiter: Arc<RateLimiter>, pub start_time: Instant }` with `pub fn new(config: Config) -> anyhow::Result<Self>`.

    4. Create `opt/relay/src/error.rs` (≈50 lines): `pub enum GatewayError { Auth, RateLimited, BackendUnreachable, TunnelLimit, Internal(anyhow::Error) }` with `thiserror` derives + `impl IntoResponse for GatewayError` that maps to HTTP 401/429/502/429/500.

    5. Create `opt/relay/src/auth.rs` (≈120 lines): `pub async fn authorize(state: &AppState, token: &Uuid, server_id: &Uuid) -> Result<Authorization, GatewayError>`. Builds the HMAC-signed POST to backend's `/internal/relay/authorize`, sends via reqwest, parses the JSON response. The HMAC signing helper is `fn sign_request(method: &str, path: &str, body: &str, timestamp: i64, nonce: &str, secret: &str) -> String` using `Hmac<Sha256>`.

    6. Create `opt/relay/src/tunnel.rs` (≈180 lines): `pub async fn run_tunnel_session(ws: WebSocketStream, state: Arc<AppState>)`. Upgrades the WS, parses the first message as `TunnelConnect { server_id, subdomain, public_port, agent_public_ip, region }`, calls `auth::authorize` with the bearer token, opens yamux over the WS, sends TunnelConnect JSON on the control stream, registers in `Registry` (which indexes by BOTH subdomain and server_id), then loops: `client.next_stream().await` → for each inbound yamux stream, spawns `player::forward_to_tcp(stream, server_id, state.clone())`. On any error or disconnect, calls `registry.unregister(server_id)` and reports `tunnel-event: disconnected` to the backend.

    7. Create `opt/relay/src/backend.rs` (≈100 lines): `pub struct BackendClient { base_url: String, hmac_secret: String, http: reqwest::Client }`. Methods: `pub async fn report_tunnel_event(&self, server_id: Uuid, event_type: &str, reason: &str) -> Result<(), GatewayError>`. Internally signs the request with `auth::sign_request` and sends POST to `/internal/relay/tunnel-event`.

    8. Create `opt/relay/src/heartbeat.rs` (≈60 lines): `pub async fn run_heartbeat_watcher(state: Arc<AppState>)`. 30s `tokio::time::interval` ticker. Each tick: iterate `state.registry.iter()`. For each handle, if `now - last_heartbeat > 30s * heartbeat_missed_threshold` (3 misses = 90s), call `registry.mark_stale(server_id)` and `backend.report_tunnel_event(server_id, "stale", "missed_heartbeats")`.

    9. Create `opt/relay/src/metrics.rs` (≈80 lines): `pub static ref METRICS: Registry = Registry::new();` plus lazy_static counters: `ACTIVE_TUNNELS`, `TOTAL_CONNECTIONS`, `REJECTED_CONNECTIONS`, `AUTH_FAILURES`, `RATE_LIMITED`, `TUNNEL_EVENTS_TOTAL` (with `event_type` label), `PLAYER_BYTES_IN`, `PLAYER_BYTES_OUT`, `BANDWIDTH_IN_PER_SUBDOMAIN` (labeled by subdomain). **NOTE (WARN 9): NO `relay_mode_distribution` counter — that is computed by the backend's `monitoring_service` (Plan 03 Task 3) by querying the `servers` table, since the gateway has no visibility into user-pinned mode overrides.** `pub async fn run_metrics_server(state: Arc<AppState>)` runs an axum router on `:9100/metrics` (NOT :9090) that returns the encoded Prometheus exposition format.

    10. Create `opt/relay/src/ratelimit.rs` (≈80 lines): `pub struct RateLimiter { buckets: DashMap<IpAddr, Mutex<TokenBucket>>, requests_per_minute: u32 }`. `TokenBucket { tokens: f64, last_refill: Instant }`. `pub fn check(&self, ip: IpAddr) -> bool`: refills based on time since last_refill, decrements, returns true if tokens > 0. **NOTE (CONTEXT D-20 RESOLVED):** This is an **in-process** token bucket. Phase 68 is single-instance (D-05), so atomicity within the process is sufficient. A future horizontal-scale phase MUST migrate this to a Redis-backed Lua counter (the `redis` crate is already a dep for the nonce dedup).

    11. Create `opt/relay/src/session_log.rs` (≈40 lines): `pub fn log_session_start(server_id: Uuid, agent_ip: IpAddr)`, `pub fn log_session_end(server_id: Uuid, bytes_tx: u64, bytes_rx: u64)`, `pub fn log_session_error(server_id: Uuid, error: &str)`. All use `tracing::info!` / `tracing::error!` with structured fields.

    12. Run `cargo check -p relay-gateway` to verify everything compiles. Resolve any errors.
  </action>
  <verify>
    <automated>cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && cargo check -p relay-gateway 2>&1 | tail -15 && echo "---" && ls opt/relay/src/ | wc -l && echo "expected 13" && echo "---" && grep -c "9100" opt/relay/src/main.rs opt/relay/src/config.rs opt/relay/src/metrics.rs | head -5</automated>
  </verify>
  <acceptance_criteria>
    - All 13 source files exist in `opt/relay/src/`
    - `cargo check -p relay-gateway` exits 0
    - `metrics.rs` references `:9100` (NOT `:9090`)
    - `metrics.rs` does NOT define a `relay_mode_distribution` counter (WARN 9)
    - `ratelimit.rs` is in-process (no Redis call inside `check()`); Redis usage stays in `auth.rs` for nonce dedup
    - `tunnel.rs` calls `registry.register(handle)` where `handle.subdomain` is the agent's claimed subdomain (used as the routing key in `registry.by_subdomain`)
  </acceptance_criteria>
  <done>Gateway crate compiles; 13 source files + config + .env.example in place; by_subdomain routing + Handshake parser live; metrics on :9100; no mode_distribution; ready for Docker (04b) and AWS deploy (04c)</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Agent → Gateway WSS | Agent opens outbound WSS to `relay.esluce.net:443` (Caddy TLS 1.3 terminated, then proxied to gateway :8080). Gateway validates Bearer token by HMAC-signed POST to backend. |
| Player → Gateway TCP | Player opens raw TCP to `<subdomain>.play.esluce.net:25565` (NLB passthrough to gateway). Gateway parses the Minecraft Handshake to extract the subdomain and looks up `server_id` in the `by_subdomain` map. **The player's source IP is NOT used for routing (BLOCKER 1).** |
| Gateway → Backend | Internal HMAC-signed POSTs to `/internal/relay/authorize` and `/internal/relay/tunnel-event`. |
| Gateway → Redis | Nonce dedup (NOT rate-limit coordination; rate-limit is in-process per D-20 RESOLVED). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-68-17 | Spoofing | Bearer token on WSS | mitigate | Token is a 122-bit UUIDv4 verified by HMAC-signed backend callback. Token never sent in cleartext (TLS 1.3). |
| T-68-18 | Spoofing | Player → server routing (BLOCKER 1) | mitigate | Routing is by Handshake-parsed subdomain (NOT source IP). The agent registered its subdomain on `TunnelConnect`; the gateway indexes `by_subdomain: DashMap<String, Uuid>`. A malicious player cannot claim a different subdomain to hijack another server's traffic. |
| T-68-19 | Tampering | Player → yamux stream | mitigate | All traffic is inside TLS 1.3 (WSS) and yamux (control + data). |
| T-68-20 | Repudiation | Tunnel lifecycle events | mitigate | All `tunnel-event` reports are HMAC-signed and logged on both sides. Backend is the source of truth for `relay_status`. |
| T-68-21 | Denial of Service | Player TCP flooding | mitigate | Per-IP rate limit (100 req/min) via in-process token bucket (D-20 RESOLVED). 1 active tunnel per server_id (D-21). |
| T-68-22 | Denial of Service | Reconnect storm | mitigate | Agent side has exponential backoff (Plan 02). Gateway side rejects duplicate `TunnelConnect` for the same `server_id` by dropping the older (D-21). |
| T-68-23 | Information Disclosure | TunnelConnect.agent_public_ip | mitigate | Only used in the audit log; never used for routing. |
| T-68-24 | Elevation of Privilege | Tunnel limit bypass | mitigate | `Registry::register` drops the older tunnel if 1 tunnel already exists for the same `server_id` (D-21). |
| T-68-25 | Tampering | HMAC secret in env | mitigate | Secret loaded from AWS Secrets Manager, not committed to git. `.env.example` documents the env var name only, no value. |
| T-68-26 | Spoofing | Handshake-parser string injection | mitigate | `read_mc_handshake_subdomain` validates subdomain charset (`[a-z0-9-]+`), length (1-63), and suffix (`.play.esluce.net`) before using it as a DashMap key. Malformed subdomains are rejected with a clean socket close. |
| T-68-27 | Tampering | Malformed VarInt | mitigate | `read_varint` enforces shift < 35 to prevent overflow attacks; `try_parse_handshake` returns None on overflow; `read_mc_handshake_subdomain` rejects the connection on parse failure. |

## ASVS L1 Mappings (Phase 68 gateway tier only — full coverage lives in plans 02, 03, 05)

- **V1.4 Access Control:** Bearer token required; HMAC-signed backend authorization on every connect. Subdomain-based routing is verified against the agent's registered subdomain (the `by_subdomain` map is only populated via authenticated `TunnelConnect`).
- **V2.1 Authentication:** HMAC-SHA256 with 32-byte secret from AWS Secrets Manager.
- **V3.7 Session Management:** yamux sessions are per-connection; tunnel is single-use per `server_id`. Subdomain is unique per server.
- **V4.1 Input Validation:** Handshake-parsed subdomain is validated against `[a-z0-9-]{1,63}` charset + required suffix.
- **V6.2 Cryptographic Practices:** TLS 1.3 enforced at Caddy level (`tls { protocols tls1.3 }`); yamux over WSS.
- **V6.4 Secret Management:** `GATEWAY_HMAC_SECRET` injected from Secrets Manager, never logged.
- **V9.1 Rate Limiting:** 100 req/min per source IP at the player TCP layer (in-process; Redis-backed is a future horizontal-scale follow-up).
- **V11.1 Data Classification:** No PII processed; only UUIDs, subdomains, and tunnel lifecycle events.
</threat_model>

<verification>
After all 3 tasks complete:

```bash
# 1. Crate compiles
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && cargo check -p relay-gateway 2>&1 | tail -5
# Expected: exit 0

# 2. All 13 source files exist
ls opt/relay/src/ | wc -l
# Expected: 13

# 3. No by_agent_ip map in registry (BLOCKER 1 fix verified)
grep -c "by_agent_ip" opt/relay/src/registry.rs
# Expected: 0 (the map is gone; the original plan had it)

# 4. Handshake parser exists
grep -c "read_mc_handshake_subdomain\|try_parse_handshake\|read_varint" opt/relay/src/player.rs
# Expected: >= 3

# 5. Metrics port is 9100 (D-22, NOT 9090)
grep -E "9100" opt/relay/relay-gateway.toml opt/relay/src/main.rs opt/relay/src/metrics.rs | wc -l
# Expected: >= 3

# 6. No mode_distribution metric (WARN 9)
grep -c "relay_mode_distribution\|mode_distribution" opt/relay/src/metrics.rs
# Expected: 0
```

End-to-end behavior is NOT verifiable in this plan — it requires Plans 02 (agent) and 03 (backend) running, plus a real MC Java client to send a Handshake. This plan verifies the artifacts compile and are correctly structured.
</verification>

<success_criteria>
- [ ] `Cargo.toml` (workspace root) has `members = [".", "api", "opt/relay"]`
- [ ] `opt/relay/Cargo.toml` exists and lists all required deps
- [ ] `opt/relay/relay-gateway.toml` exists with `metrics_bind = "0.0.0.0:9100"`
- [ ] `opt/relay/.env.example` exists
- [ ] All 13 source files exist in `opt/relay/src/`
- [ ] `registry.rs` uses `by_subdomain: DashMap<String, Uuid>` (NO `by_agent_ip`)
- [ ] `player.rs` parses the MC Java Handshake to extract the subdomain
- [ ] `metrics.rs` exposes counters on `:9100` and does NOT include `relay_mode_distribution`
- [ ] `ratelimit.rs` is in-process (D-20 RESOLVED)
- [ ] `cargo check -p relay-gateway` exits 0
</success_criteria>

<output>
After completion, create `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04a-SUMMARY.md`
</output>