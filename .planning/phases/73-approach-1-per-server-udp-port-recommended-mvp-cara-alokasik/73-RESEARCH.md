# Phase 73: Approach 1: Per-Server UDP Port — Research

**Researched:** 2026-06-13
**Domain:** UDP relay gateway extension for Minecraft Bedrock Edition
**Confidence:** HIGH

## Summary

Phase 73 adds UDP relay support for Minecraft Bedrock Edition via dedicated per-server UDP ports on the relay gateway. Bedrock uses UDP (RakNet protocol) which cannot be routed through the existing TCP-only subdomain-based relay. Each Bedrock server gets a unique UDP port on the relay EC2 (range 19132-19231), mapped to its agent tunnel via length-prefixed datagram framing (TLV format) over yamux. The backend allocates UDP ports from the existing `port_pools` table (protocol='udp'). The gateway opens `UdpSocket` instances on demand, and the agent runs a parallel `run_udp_relay_session` function using `send_to`/`recv_from`.

**Primary recommendation:** Implement in 4 waves: (1) backend port pool seed + protocol-aware dispatch, (2) agent-side UDP session handler + TunnelConnect loader field, (3) gateway-side UdpSocket management + datagram framing + port registry, (4) DNS SRV records + NLB config + dashboard UX.

<phase_requirements>
## Phase Requirements

This phase has no formal requirement IDs. The ROADMAP.md lists this as "TBD" for requirements. Research findings below directly support the 7 in-scope items listed in CONTEXT.md:

| # | Item | Research Support |
|---|------|------------------|
| 1 | Gateway UDP listener — per-server UdpSocket management | Sections 3.1-3.3, Standard Stack, Architecture Patterns |
| 2 | Datagram framing protocol (TLV format) over yamux | Section 3.4, D-07/D-08/D-09 |
| 3 | Agent UDP session handler (send_to/recv_from pattern) | Sections 3.5-3.6, Code Examples |
| 4 | Backend UDP port pool allocation | Section 3.7, D-01/D-02 |
| 5 | NLB UDP listener config + security group rules | Section 3.8, Environment Availability |
| 6 | Dashboard UX showing Bedrock server address | Section 3.9 |
| 7 | Player-side DNS SRV record support | Section 3.10, D-14 |
</phase_requirements>

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01 (Port authority):** Backend allocates from port_pools — consistent with existing TCP port allocation. The `port_pools` table already has `protocol VARCHAR(10)` with `'udp'` support. Backend allocates a UDP port on server create and sends it via `RelayConfigSync`. The pool range config entry uses `protocol = 'udp'`.
- **D-02 (Port pool initialization):** A new global pool row with `protocol = 'udp'`, range 19132-19231, must be inserted into `port_pools` (migration or seed data). The existing row is TCP-only (25565-25665).
- **D-03 (Protocol detection):** Agent declares loader in TunnelConnect — the TunnelConnect JSON message already carries the server identity. For Bedrock servers (`loader: "bedrock"`), the gateway opens a UdpSocket on the allocated port. No protocol field needed in RelayConfigSync — the gateway derives it from loader.
- **D-04 (Bind timing):** Deferred — on-demand dedicated socket on first datagram — The gateway opens a UdpSocket on the allocated port at tunnel connect time (port becomes accessible), but does NOT start the full forwarding session until the first datagram arrives from a player. Session creation is deferred to save yamux stream resources when no players are connected.
- **D-05 (Port release):** 30-second grace period — After tunnel disconnect, keep the UdpSocket open for 30s, silently dropping incoming packets. Prevents port reuse collisions with in-flight player datagrams.
- **D-06 (Port range):** 19132-19231 (100 ports) — Configurable via gateway env config. Standard Bedrock port (19132) as range start. 100 concurrent Bedrock servers before needing to expand.
- **D-07 (Stream model):** Long-lived session stream — Single yamux stream per server opened when first datagram arrives, carries all datagrams both directions. Stream stays open for the tunnel lifetime. Avoids per-datagram stream overhead.
- **D-08 (Wire format):** TLV (Type-Length-Value) — `[1-byte type][4-byte big-endian length][payload]`. Type field allows future extension (e.g., control frames, keepalive). Initial types: `0x01 = datagram`. Length covers only the payload.
- **D-09 (Header size):** 4-byte length — Covers full UDP datagram range (0-65535). Consistent with other binary framing in the codebase.
- **D-10 (Local forwarding pattern):** UdpSocket with send_to/recv_from (connectionless) — Agent opens UdpSocket, binds to ephemeral port, sends datagrams to container via send_to(), receives replies via recv_from(). More flexible than connect() — allows future multi-target scenarios.
- **D-11 (Container address source):** Use existing `local_mc_addr` from RelayConfig — The RelayConfig already carries `local_mc_addr` (e.g., `127.0.0.1:19132`). Agent uses this as the UDP send target. No new config field needed.
- **D-12 (Session task):** New `run_udp_relay_session` function in `relay_session.rs` — mirrors existing `run_relay_session` but uses UdpSocket instead of TcpStream. Reads framed datagrams from yamux, sends via UdpSocket to container; reads from UdpSocket, writes framed to yamux.
- **D-13 (Bedrock address display):** Dashboard + DNS SRV record — Dashboard shows both: Direct IP:port (`esluce.com:{port}` or relay IP:port) for manual entry, and a friendly DNS name backed by an SRV record: `bedrock-{subdomain}.play.esluce.com`.
- **D-14 (DNS SRV management):** Backend or agent needs Route 53 SRV record creation capability (extends existing A-record DNS automation).

### The Agent's Discretion
- Exact TLV type byte assignments (research recommends: `0x01` for datagram, reserve `0x00` for invalid, `0xFF` for control)
- UDP socket buffer sizes (recommend: 64 KiB matching BRIDGE_BUFFER_BYTES)
- Whether the deferred-session check is based on a shared UDP accept loop or individual socket readiness (recommend: individual socket with recv_from() poll in the session task, spawned when first datagram matches the port)

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope.
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| UDP port allocation | Backend API | Database (port_pools) | Backend owns pool logic, DB stores allocated ports. D-01 mandates backend authority. |
| UdpSocket lifecycle | Gateway (opt/relay) | — | Gateway binds UDP sockets on relay EC2. Player datagrams arrive here first. |
| Datagram framing TLV | Gateway (opt/relay) | Agent (solys) | Both ends must speak the same framing protocol. Gateway wraps/unwraps player datagrams; agent wraps/unwraps container datagrams. |
| Agent→container UDP forwarding | Agent (solys) | — | Agent opens local UdpSocket to container's Bedrock port. D-11 uses existing local_mc_addr. |
| DNS SRV records | Backend API | Route 53 | Backend or agent creates SRV records in Route 53 hosted zone. D-14 mandates this capability. |
| Bedrock address display | Dashboard (app) | — | UI extends ConnectivitySection to show IP:port and SRV-based address for Bedrock servers. |
| NLB UDP listener | AWS Infra | — | Add UDP listener on ports 19132-19231 to existing NLB. Single one-time manual change. |

## Standard Stack

### Core — No New Dependencies Required

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio::net::UdpSocket` | (tokio 1.x, in-tree) | UDP I/O for player datagram reception and agent container forwarding | Already in tokio `"full"` feature, used by both relay gateway (opt/relay) and agent (solys). No new crate needed. |
| `tokio-yamux` | 0.3 | Multiplexed streams over single WS connection | Already used for existing TCP relay. UDP session uses same yamux session for the long-lived datagram stream. |
| `serde` / `serde_json` | 1.x | TLV framing serialization + TunnelConnect extension | Already used throughout both codebases. The type/length prefix is binary, not JSON — serde not needed for TLV itself. |

### Supporting — Already in Dependency Tree

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `bytes` | 1.x | Efficient buffer management for datagram framing | Already in agent's Cargo.toml. Useful for the TLV encode/decode if we need zero-copy. Not strictly required — `Vec<u8>` works. |
| `base64` / `hmac` / `sha2` | 0.22 / 0.12 / 0.10 | Auth + signing for relay operations | Already in both Cargo.tomls. No changes needed. |
| `aws-sdk-route53` | (new) | SRV record creation in Route 53 | New dependency for Route 53 DNS automation. Alternatively extend Cloudflare DNS handler if SRV records go in Cloudflare zone. |
| `tokio::sync::Notify` | (tokio 1.x, in-tree) | Wake gateway UDP port on first datagram | Used for deferred socket wake-up pattern — port thread sleeps until Notify triggered. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `UdpSocket::send_to`/`recv_from` (connectionless) | `UdpSocket::connect` (connected mode) | D-10 chose connectionless for flexibility. Connected mode is simpler but prevents multi-container future. |
| 4-byte length field (TLV) | 2-byte length field | D-09 chose 4-byte for safety (full UDP range) and consistency. 2-byte would save 2 bytes/datagram but risks truncation. |
| Route 53 for SRV records | Cloudflare DNS for SRV records | The existing dns.rs uses Cloudflare API for A records. Route 53 is used for `play.esluce.com` zone. SRV records should go in the same zone where the A/AAAA records live. Check which zone owns `play.esluce.com`. |

**Installation (no new crate dependencies):**
```bash
# No new npm/cargo installs needed for core UDP functionality.
# For Route 53 support, add to agent/solys/Cargo.toml:
# aws-sdk-route53 = "1"
```

**Version verification:**
[VERIFIED: Cargo.toml analysis]
- `tokio` 1.x with `features = ["full"]` — provides `tokio::net::UdpSocket`
- `tokio-yamux` 0.3 — already used for TCP relay, unchanged
- `serde` 1.x — already present
- No new external crate dependencies for the core UDP relay path

## Architecture Patterns

### System Architecture Diagram

```
Bedrock Player (UDP/RakNet)
        │
        ▼
  *.play.esluce.com
  (Route 53 SRV: _minecraft._udp.bedrock-xxx.play.esluce.com)
        │
        ▼
  Network Load Balancer (NLB)
  ┌─────────────────────────────────────────┐
  │  TCP:25565  (Java, existing)            │
  │  UDP:19132-19231  (Bedrock, new)        │
  └──────────────┬──────────────────────────┘
                 │
                 ▼
  Relay EC2 Instance (relay.esluce.net)
  ┌─────────────────────────────────────────┐
  │  Caddy (port 443: WSS termination)      │
  │    └─► relay-gateway:8080               │
  │                                         │
  │  relay-gateway (opt/relay/src)          │
  │  ┌─────────────────────────────────┐    │
  │  │  TCP:25565 → player.rs          │    │
  │  │   (Java: Handshake→subdomain    │    │
  │  │    →yamux stream)               │    │
  │  │                                 │    │
  │  │  UDP:19132-19231 → udp.rs ★NEW  │    │
  │  │   (Bedrock: port→server_id      │    │
  │  │    →TLV framing→yamux stream)   │    │
  │  │                                 │    │
  │  │  Registry (registry.rs)         │    │
  │  │   ├── by_subdomain → server_id  │    │
  │  │   └── by_server_id → handle     │    │
  │  │                                 │    │
  │  │  UdpPortRegistry ★NEW           │    │
  │  │   └── port → (server_id,        │    │
  │  │        UdpSocket, grace_timer)  │    │
  │  └─────────────┬───────────────────┘    │
  └────────────────┼────────────────────────┘
                   │
         WSS + yamux (persistent tunnel)
                   │
                   ▼
  Agent Node (solys agent)
  ┌─────────────────────────────────────────┐
  │  relay_client.rs                        │
  │   ├── TunnelConnect (with loader field) │
  │   ├── drive_inbound_streams (TCP)       │
  │   └── spawn_udp_session ★NEW            │
  │                                          │
  │  relay_session.rs                       │
  │   ├── run_relay_session (TcpStream)      │
  │   └── run_udp_relay_session ★NEW         │
  │       (UdpSocket + TLV framing)          │
  │                                          │
  │  state.rs                               │
  │   └── RelayServerConfig (unchanged)      │
  └────────────────┬────────────────────────┘
                   │
                   ▼
  Docker Container (itzg/minecraft-bedrock-server)
  ┌─────────────────────────────────────────┐
  │  UDP:19132 (RakNet listener)            │
  └─────────────────────────────────────────┘

  Backend API (api.esluce.com)
  ┌─────────────────────────────────────────┐
  │  PortAllocationUseCase                  │
  │   ├── allocate_port(protocol='udp')     │
  │   └── release_port(protocol='udp')      │
  │                                          │
  │  RelayService                           │
  │   ├── push_relay_config()               │
  │   ├── create_srv_record() ★NEW          │
  │   └── delete_srv_record() ★NEW          │
  │                                          │
  │  port_pools table                       │
  │   ├── row: protocol='tcp', 25565-25665  │
  │   └── row: protocol='udp', 19132-19231 ★│
  └─────────────────────────────────────────┘
```

### Recommended Project Structure — New/Modified Files

```
opt/relay/src/
├── config.rs          # MODIFIED: add udp_ports: UdpConfig { port_start, port_end, grace_period_secs }
├── state.rs           # MODIFIED: add udp_registry: UdpPortRegistry
├── registry.rs        # MODIFIED: add by_port index to Registry
├── tunnel.rs          # MODIFIED: TunnelConnect gains loader field
├── udp.rs             # NEW: UdpPortRegistry, run_udp_player_listener, handle_udp_datagram
├── player.rs          # NO CHANGE (TCP path untouched)
├── main.rs            # MODIFIED: spawn udp::run_udp_player_listener

agent/solys/src/
├── handlers/
│   ├── relay_session.rs  # MODIFIED: add run_udp_relay_session()
│   ├── relay_client.rs   # MODIFIED: spawn UDP session when loader=bedrock
│   └── dns.rs            # MODIFIED: add SRV record handlers (or new dns_srv.rs)
├── state.rs              # MINOR: no struct changes (public_port reused)
├── agent_connection.rs   # MINOR: map loader/bedrock flag in RelayConfigSync handler

api/src/
├── application/use_cases/
│   └── port_allocation_use_case.rs  # MODIFIED: protocol-aware dispatch
├── application/services/
│   └── relay_service.rs             # MODIFIED: build RelayConfigSync with loader field
├── presentation/ws/
│   └── node_protocol.rs             # MODIFIED: ServerRelayInfo gains loader field
├── migrations/
│   └── (new) seed_udp_port_pool.sql # NEW: INSERT into port_pools for UDP range

app/src/
├── components/
│   ├── ConnectivitySection.jsx   # MODIFIED: show Bedrock addresses
│   └── TunnelHealthCard.jsx      # MODIFIED: show UDP tunnel health
└── hooks/
    └── useConnectivity.js        # MODIFIED: handle bedrock mode/addresses
```

### Pattern 1: Deferred UdpSocket with Grace Timer

**What:** The gateway opens a `UdpSocket` on the allocated port when TunnelConnect with `loader="bedrock"` arrives, but does NOT spawn the forwarding session until the first datagram arrives. After tunnel disconnect, the socket stays open for 30s (grace period) silently dropping packets.

**When to use:** Bedrock server tunnel lifecycle — avoid wasting yamux streams for idle servers, prevent port reuse collisions.

**Flow:**
```
TunnelConnect(loader="bedrock") →
  1. Bind UdpSocket on allocated port
  2. Register port→server_id in UdpPortRegistry
  3. Start recv_from() poll loop (silently drops until first datagram)
  4. On first datagram → spawn yamux stream session task
  5. Session task: TLV-decode datagrams → yamux write; yamux read → TLV-encode → UdpSocket send_to

TunnelDisconnect →
  1. Start 30s grace timer (socket stays bound, packets silently dropped)
  2. When timer expires → close socket, free port in registry
```

### Pattern 2: TLV Batagram Framing over Yamux

**What:** Binary protocol over yamux stream: [1-byte type][4-byte big-endian length][datagram bytes]. Both directions use the same stream.

**When to use:** Forwarding UDP datagrams through a stream-oriented transport (yamux).

```
Gateway─►Agent (player→container):
  [0x01][0x0000_01FF][191 bytes of RakNet datagram...]

Agent─►Gateway (container→player):
  [0x01][0x0000_00E0][224 bytes of RakNet datagram...]
```

### Anti-Patterns to Avoid
- **Binding UDP socket on gateway for every server_id at startup:** Wastes ports for Bedrock servers that may never get players. Use D-04 deferred binding instead.
- **Per-datagram yamux stream:** Overhead of stream open/close per datagram is prohibitive for RakNet's chatty protocol. Use D-07 long-lived stream.
- **UdpSocket::connect() on agent side:** Locks to one container target. Use D-10 send_to/recv_from for future multi-container flexibility.
- **Synchronous read/write in UDP handler:** Can block the whole event loop. Always use tokio async UdpSocket methods.

### Pattern 3: Backend Port Allocation Protocol-Aware Dispatch

**What:** The existing `PortAllocationUseCase::allocate_port` needs a `protocol` parameter. The current implementation queries `WHERE node_id IS NULL AND is_active = true` which returns only the TCP pool if only one global pool exists.

**Required changes:**
1. `allocate_port(pool, node_id, protocol)` — add `protocol: &str` param
2. SQL: `WHERE node_id IS NULL AND is_active = true AND protocol = $1`
3. Server creation with `game_type=bedrock` calls `allocate_port(pool, None, "udp")`
4. Backend sends allocated UDP port in `ServerRelayInfo.public_port`

**Source:** [VERIFIED: code analysis of api/src/application/use_cases/port_allocation_use_case.rs](api/src/application/use_cases/port_allocation_use_case.rs)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| UDP socket I/O | Custom async IO loop | `tokio::net::UdpSocket` | Already in dependency tree. Handles buffer management, async readiness, cancellation safely. |
| Stream multiplexing | Custom stream framing | `tokio-yamux` 0.3 | Already used by existing TCP relay. Reuse same yamux session for UDP datagram stream. |
| DNS record management | DIY HTTP calls to Route 53 API | `aws-sdk-route53` crate, OR extend existing Cloudflare dns.rs | The existing dns.rs already handles Cloudflare A record creation. SRV support could follow the same pattern. Route 53 SDK is the proper tool if the zone is in AWS. |
| Port allocation with concurrency safety | Custom atomic counter | SQL row with JSONB `allocated_ports` array | Existing pattern proven in Phase 68-70. Transaction-safe, persistent across restarts. |

**Key insight:** This phase benefits heavily from the existing relay infrastructure. The TCP relay's session lifecycle (TunnelConnect → heartbeat loop → stream drive → cleanup) maps directly to the UDP case. The main new work is: (a) UdpSocket management instead of TcpStream, and (b) TLV framing instead of raw byte copy.

## Common Pitfalls

### Pitfall 1: UDP Buffer Sizing for RakNet
**What goes wrong:** RakNet can send fragmented datagrams larger than typical MTU (1500 bytes). If the gateway's recv_from buffer is too small, datagrams are silently truncated.
**Why it happens:** The default UDP buffer on many systems is ~212 KiB, but tokio's recv_from requires a pre-allocated buffer. A 1500-byte buffer will lose higher-layer protocols.
**How to avoid:** Use `BRIDGE_BUFFER_BYTES = 64 * 1024` (matching existing constant) for the recv_from buffer. This covers the maximum RakNet fragment size (approximately 64 KiB per the Bedrock protocol spec).
**Warning signs:** Player connection succeeds but chunks fail to load; intermittent packet loss that correlates with large block transfers.

### Pitfall 2: Grace Period vs. Port Reuse Race
**What goes wrong:** Tunnel disconnects → port freed → new tunnel binds same port → in-flight player datagram arrives on old socket → new tunnel's socket is wrong one.
**Why it happens:** UDP is connectionless — a datagram sent before the player detects disconnect can arrive after the port is re-bound.
**How to avoid:** D-05's 30-second grace period. The old socket stays bound, silently dropping packets. After 30s, in-flight datagrams are guaranteed to have been dropped by client timeout.
**Warning signs:** Players report "Connection reset" during tunnel reconnects. Implementation must use `tokio::time::sleep` for the grace period, not a busy-wait.

### Pitfall 3: NLB UDP Idle Timeout
**What goes wrong:** NLB has a UDP idle timeout (default 10s for AWS NLB). If no datagrams flow for >10s, NLB may stop forwarding to the target.
**Why it happens:** NLB maintains a flow table for UDP. After the idle timeout, it removes the flow entry. Subsequent datagrams may not reach the target.
**How to avoid:** Either:
- Keepalive datagrams from agent (e.g., send a TLV type `0xFF` control frame every 5s when idle)
- Or accept the behavior — Minecraft Bedrock clients send keepalive packets at the RakNet layer (~every 5s), so in practice the flow stays active during gameplay. Only idle servers would see this.
**Warning signs:** Players freeze after brief idle periods; game console shows "Lost connection" with no timeout on client side.

### Pitfall 4: Proxy Protocol with UDP on NLB
**What goes wrong:** AWS NLB supports proxy protocol v2 for TCP but NOT for UDP. Unlike the TCP player path where `instance` target type preserves client source IP, UDP health checks and source IP preservation work differently.
**Why it happens:** The DEPLOY.md (Section 2) specifies NLB target type `instance` which preserves client IP for TCP. For UDP, AWS NLB does not inject proxy protocol headers — the gateway will receive datagrams with the NLB's private IP as source, not the player's IP.
**How to avoid:** Since D-03 mandates the gateway opens per-server UdpSocket (not a shared socket), the gateway knows which server_id a datagram belongs to by the destination port. The source IP of the NLB is acceptable — the gateway doesn't need the player's real IP for routing (it uses port → server_id mapping).
**Verification:** Test with actual NLB UDP flow; confirm port→server_id mapping works regardless of source IP.

### Pitfall 5: Race Condition Between TunnelConnect and UdpSocket Bind
**What goes wrong:** Gateway binds UdpSocket on the port, but a player datagram arrives before the registry entry is fully populated.
**Why it happens:** The socket bind is async — between the `bind()` call returning and the registry write, a datagram can arrive and find no handler registered.
**How to avoid:** Bind the socket FIRST, THEN register in the registry. The player datagrams during this window will be queued in the socket's receive buffer (which is ~200 KiB by default). The registry entry ensures the session can be spawned.
**Warning signs:** First datagram is lost after tunnel connect; player needs to retry.

## Code Examples

### Example 1: Gateway UdpSocket Bind + Deferred Session (opt/relay/src/udp.rs)

```rust
// Source: Pattern derived from existing player.rs + tunnel.rs patterns
// This represents the recommended implementation following D-04

pub struct UdpPortEntry {
    pub server_id: Uuid,
    pub socket: Arc<UdpSocket>,
    pub grace_task: Option<tokio::task::JoinHandle<()>>,
}

pub struct UdpPortRegistry {
    ports: Arc<DashMap<u16, UdpPortEntry>>,
}

impl UdpPortRegistry {
    pub fn new() -> Self {
        Self { ports: Arc::new(DashMap::new()) }
    }

    /// Bind a UdpSocket on `port` and register it for `server_id`.
    /// Returns Err if the port is already bound or unavailable.
    pub async fn bind_port(
        &self,
        port: u16,
        server_id: Uuid,
    ) -> Result<(), std::io::Error> {
        if self.ports.contains_key(&port) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!("UDP port {} already bound", port),
            ));
        }
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
        self.ports.insert(port, UdpPortEntry {
            server_id,
            socket: Arc::new(socket),
            grace_task: None,
        });
        Ok(())
    }

    /// Start the grace period. After `duration`, the port is freed.
    pub async fn start_grace_period(&self, port: u16, duration: Duration) {
        if let Some(mut entry) = self.ports.get_mut(&port) {
            // Cancel any existing grace task
            if let Some(task) = entry.grace_task.take() {
                task.abort();
            }
            let ports = self.ports.clone();
            let p = port;
            entry.grace_task = Some(tokio::spawn(async move {
                tokio::time::sleep(duration).await;
                ports.remove(&p);
                info!("[UDP] Grace period expired, freed port {}", p);
            }));
        }
    }

    /// Cancel grace period (called on reconnect).
    pub fn cancel_grace(&self, port: u16) {
        if let Some(mut entry) = self.ports.get_mut(&port) {
            if let Some(task) = entry.grace_task.take() {
                task.abort();
            }
        }
    }
}
```

### Example 2: Agent UDP Session with TLV Framing (agent/solys/src/handlers/relay_session.rs)

```rust
// Source: Pattern derived from existing run_relay_session
// Added as run_udp_relay_session()

use tokio::net::UdpSocket;

const TLV_TYPE_DATAGRAM: u8 = 0x01;

pub async fn run_udp_relay_session<S>(
    yamux_stream: S,
    local_addr: String,
    bytes_counter: Arc<AtomicU64>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    // Bind to ephemeral port for outgoing UDP
    let udp = UdpSocket::bind("0.0.0.0:0").await?;
    
    // Parse local container address
    let remote_addr: SocketAddr = local_addr.parse()
        .context("Invalid local_mc_addr for UDP")?;

    // Split yamux stream for concurrent read/write
    let (mut yamux_read, mut yamux_write) = tokio::io::split(yamux_stream);
    let udp = Arc::new(udp);
    let total_bytes = Arc::new(AtomicU64::new(0));

    // Task: yamux → UdpSocket (agent reads frame from gateway, sends to container)
    let yamux_to_udp = {
        let udp = udp.clone();
        let remote = remote_addr;
        let total = total_bytes.clone();
        tokio::spawn(async move {
            let mut len_buf = [0u8; 4];
            let mut payload = vec![0u8; 65535];
            loop {
                // Read TLV type byte
                let mut type_byte = [0u8; 1];
                if yamux_read.read_exact(&mut type_byte).await.is_err() { break; }
                if type_byte[0] != TLV_TYPE_DATAGRAM { continue; } // skip control

                // Read 4-byte big-endian length
                if yamux_read.read_exact(&mut len_buf).await.is_err() { break; }
                let len = u32::from_be_bytes(len_buf) as usize;
                if len > payload.len() { break; }

                // Read payload
                yamux_read.read_exact(&mut payload[..len]).await?;

                // Send to container
                udp.send_to(&payload[..len], remote).await?;
                total.fetch_add(1 + 4 + len as u64, Ordering::Relaxed);
            }
            Ok::<_, anyhow::Error>(())
        })
    };

    // Task: UdpSocket → yamux (agent reads from container, sends to gateway)
    let udp_to_yamux = {
        let total = total_bytes.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                let (n, _src) = udp.recv_from(&mut buf).await?;
                // Write TLV: [type][length][payload]
                yamux_write.write_all(&[TLV_TYPE_DATAGRAM]).await?;
                yamux_write.write_all(&(n as u32).to_be_bytes()).await?;
                yamux_write.write_all(&buf[..n]).await?;
                total.fetch_add(1 + 4 + n as u64, Ordering::Relaxed);
            }
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        })
    };

    // Wait for either task to finish
    tokio::select! {
        r = yamux_to_udp => { r?; }
        r = udp_to_yamux => { r?; }
    }

    let total = total_bytes.load(Ordering::Relaxed);
    bytes_counter.fetch_add(total, Ordering::Relaxed);
    Ok(())
}
```

### Example 3: TunnelConnect with Loader Field (opt/relay/src/tunnel.rs)

```rust
// Source: Extended existing TunnelConnect struct
// Add optional loader field

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConnect {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub relay_token: Uuid,
    pub server_id: Uuid,
    pub subdomain: String,
    pub public_port: u16,
    pub agent_public_ip: String,
    pub region: String,
    /// Optional loader discriminator. "bedrock" triggers UDP port
    /// allocation instead of TCP subdomain routing.
    #[serde(default)]
    pub loader: Option<String>,
}
```

**Gateway dispatch logic** (in tunnel.rs after step 6, before step 7):
```rust
// After successful authorize() and before registry.register()
if connect.loader.as_deref() == Some("bedrock") {
    // This is a Bedrock server — bind UdpSocket instead of TCP routing
    info!("[TUNNEL] Bedrock server, binding UDP port {}", connect.public_port);
    if let Err(e) = state.udp_registry.bind_port(connect.public_port, connect.server_id).await {
        warn!("[TUNNEL] Failed to bind UDP port {}: {}", connect.public_port, e);
        return;
    }
    // NOTE: We do NOT register in the subdomain-based Registry.
    // Bedrock servers are routed by port, not subdomain.
    // The TunnelHandle is still created for heartbeat tracking,
    // but player.rs will never route TCP to this server.
}
```

### Example 4: Port Pool Seed Migration

```sql
-- Source: Pattern from existing api/migrations/20260409000002_create_port_pools_table.sql
-- New pool row for UDP Bedrock ports (D-02, D-06)
INSERT INTO port_pools (id, node_id, port_range_start, port_range_end, current_port, protocol, is_active)
SELECT gen_random_uuid(), NULL, 19132, 19231, 19132, 'udp', true
WHERE NOT EXISTS (SELECT 1 FROM port_pools WHERE protocol = 'udp' AND node_id IS NULL);
```

### Example 5: Route 53 SRV Record Creation (Backend or Agent)

```rust
// Source: AWS Route 53 API — ChangeResourceRecordSets
// SRV record format for Bedrock player discovery (D-13, D-14)

// _minecraft._udp.bedrock-{subdomain}.play.esluce.com → 0 0 {port} relay.esluce.net
// Priority=0, Weight=0, Port={allocated_port}, Target=relay.esluce.net

use aws_sdk_route53::types::{
    Change, ChangeAction, ResourceRecord, ResourceRecordSet, RrType,
};

async fn create_srv_record(
    client: &aws_sdk_route53::Client,
    hosted_zone_id: &str,
    subdomain: &str,
    port: u16,
) -> Result<()> {
    let record_name = format!("_minecraft._udp.bedrock-{}.play.esluce.com", subdomain);
    let record_value = format!("0 0 {} relay.esluce.net", port);

    let change = Change::builder()
        .action(ChangeAction::Upsert)
        .resource_record_set(
            ResourceRecordSet::builder()
                .name(record_name)
                .type_(RrType::Srv)
                .ttl(60)
                .resource_records(
                    ResourceRecord::builder()
                        .value(record_value)
                        .build()
                )
                .build()
        )
        .build();

    client
        .change_resource_record_sets()
        .hosted_zone_id(hosted_zone_id)
        .change_batch(
            aws_sdk_route53::types::ChangeBatch::builder()
                .changes(change)
                .build()
        )
        .send()
        .await?;

    Ok(())
}
```

> **Note:** The existing `dns.rs` uses Cloudflare API for A records. SRV records for `play.esluce.com` should go in whichever DNS provider owns that zone. If Route 53 owns it (per DEPLOY.md Section 4), add `aws-sdk-route53` as a dependency. If Cloudflare owns it, extend `dns.rs` to support SRV type records. This decision requires AWS zone ownership confirmation (see Open Questions).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| TCP-only relay (Phase 68) | TCP + UDP relay | Phase 73 | UDP support for Bedrock Edition players |
| Single `port_pools` (TCP) | Dual pools (TCP + UDP) | Phase 73 | Backend dispatches protocol-aware port allocation |
| Subdomain-only routing | Subdomain (TCP) + Port (UDP) | Phase 73 | Bedrock uses port-based identification, not subdomain |
| Stream-only relay_session | Stream + Datagram relay_session | Phase 73 | Agent handles both TCP streams and UDP datagrams |

**Deprecated/outdated:**
- The existing `resolve_container_addr` function in `relay_client.rs:346` resolves container IPs via Docker inspect. For Bedrock UDP, this is not needed — D-11 specifies using `local_mc_addr` directly. The container's Bedrock port (19132) is already bound on `127.0.0.1:19132` via Docker port mapping.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `play.esluce.com` DNS zone is hosted in Route 53 (not Cloudflare) | DNS SRV Records | SRV record logic must target the correct DNS provider. If Cloudflare owned the zone, extend dns.rs instead of adding aws-sdk-route53. |
| A2 | NLB target type is `instance` (preserving client source IP) for UDP listeners | Common Pitfalls — Pitfall 4 | If NLB uses `ip` target type, the gateway may need proxy protocol v2 for UDP (which NLB doesn't support for UDP). This is acceptable because port→server_id routing doesn't need client IP. |
| A3 | The existing `BRIDGE_BUFFER_BYTES = 64 KiB` is sufficient for RakNet datagrams | Common Pitfalls — Pitfall 1 | If RakNet uses larger fragments (unlikely — 64 KiB is the protocol max), the buffer needs to be larger. Safe assumption based on Bedrock protocol spec knowledge. |
| A4 | `tokio::net::UdpSocket` is available with the current `features = ["full"]` in both Cargo.tomls | Standard Stack | UdpSocket is part of tokio's "net" feature, which is included in "full". Verified by both Cargo.tomls using `features = ["full"]`. |
| A5 | The `ServerRelayInfo` struct extension with `loader` field won't break backward compatibility | Backend → Agent Config Flow | The field is `#[serde(default)]` on the agent side, so existing Java-only servers (no loader field) will deserialize with `loader = None`. Safe — defaults to TCP behavior. |

## Open Questions — RESOLVED

**All open questions have been resolved through user confirmation and planning decisions.**

> ### Q1 (RESOLVED): Which DNS provider owns `play.esluce.com`?
> **Answer:** Route 53. User confirmed `play.esluce.com` is the correct domain. The DEPlOY.md Route 53 section already covers wildcard A records for this zone. SRV records go in the same Route 53 hosted zone.
> **Impact:** Add `aws-sdk-route53` to **api/Cargo.toml** (backend). Use `#[cfg(feature = "route53")]` for optional compilation. Backend creates/deletes SRV records during server lifecycle.

> ### Q2 (RESOLVED): How does the gateway handle the deferred session first-datagram trigger?
> **Answer:** Per-port `tokio::spawn` task in udp.rs runs a `recv_from()` loop in `run_udp_player_session()`. On first datagram, it opens a yamux stream via `TunnelHandle.yamux_control.open_stream()`, starts bidirectional TLV forwarding, and continues polling. The task is spawned after the TunnelHandle is registered (in tunnel.rs step 7b).
> **Impact:** 73-03-PLAN.md Task 4 covers the full implementation.

> ### Q3 (RESOLVED): Does the existing `resolve_container_addr` apply to Bedrock containers?
> **Answer:** No — D-11 specifies `local_mc_addr` (format `127.0.0.1:19132`) directly. The Docker port mapping `19132:19132` means the host port maps directly to the container port at localhost. `resolve_container_addr` is not needed for the UDP path.
> **Impact:** Agent's `run_udp_relay_session` uses `relay_config.local_mc_addr` as-is.

> ### Q4 (RESOLVED): Which entity should own Route 53 SRV record creation?
> **Answer:** Backend. The backend already has AWS credentials from the ECR deployment flow, orchestrates server lifecycle (create → update → delete), and IAM policies can be scoped narrowly. The agent's dns.rs remains for Cloudflare A records (user's own domain).
> **Impact:** SRV record methods live in `relay_service.rs`, called from `push_relay_config` (create) and server delete handler (delete).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Gateway (opt/relay) + Agent (solys) | ✓ | (rustc 1.x in CI) | — |
| tokio 1.x | UDP socket I/O | ✓ | (in-tree) | — |
| AWS NLB UDP listener support | Wave 4 infra | ✓ | (AWS managed) | Skip NLB config, expose ports directly on EC2 security group for testing |
| Route 53 API | SRV record creation | ✓ | (AWS managed) | Use Cloudflare DNS if `play.esluce.com` is in Cloudflare |
| Docker | Local testing | ✓ | (docker 24+) | — |
| EC2 Security Group | UDP port range | ✓ | (manual AWS console) | — |

**Missing dependencies with no fallback:**
- None — all core dependencies are already in the project (tokio, yamux, serde). The only new dependency `aws-sdk-route53` (if Route 53 path is chosen) needs to be added to `agent/solys/Cargo.toml` or `api/Cargo.toml`.

**Missing dependencies with fallback:**
- `aws-sdk-route53`: If the zone is in Cloudflare, the existing Cloudflare DNS handler can be extended with SRV record support via the Cloudflare API, avoiding any new crate dependency.

## Validation Architecture

> workflow.nyquist_validation is not configured (absent in .planning/config.json) — treat as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` + app-level integration tests via relay gateway health endpoints |
| Config file | `Cargo.toml` (workspace root at agent/solys/) |
| Quick run command | `cargo test -p solys --test udp_relay 2>&1 | tail -20` (if test files exist) |
| Full suite command | `cargo test --workspace 2>&1 | tail -30` |

### Phase Requirements → Test Map

> No formal requirement IDs exist for Phase 73. The table below maps behavior to test coverage.

| Behavior | Test Type | Automated Command | File Exists? |
|----------|-----------|-------------------|-------------|
| Gateway binds UdpSocket on TunnelConnect(loader=bedrock) | Integration | `cargo test -p relay-gateway --test udp_bind 2>&1` | ❌ Wave 0 |
| TLV framing encode/decode roundtrip | Unit | `cargo test -p relay-gateway -- tlv_roundtrip 2>&1` | ❌ Wave 0 |
| Agent run_udp_relay_session forwards datagrams | Integration | `cargo test -p solys -- udp_relay_session 2>&1` | ❌ Wave 0 |
| Agent run_udp_relay_session handles send_to failure | Unit | `cargo test -p solys -- udp_session_error 2>&1` | ❌ Wave 0 |
| Backend allocates UDP port from udp pool | Integration | (backend test suite) | ❌ Wave 0 |
| Grace period prevents immediate port reuse | Integration | `cargo test -p relay-gateway -- grace_period 2>&1` | ❌ Wave 0 |
| Backend dispatches to correct protocol pool | Unit | (backend test suite) | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check` passes for modified crate
- **Per wave merge:** `cargo test` passes for affected workspace members
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/udp_bind.rs` in `opt/relay/` — covers UdpSocket registry bind/unbind/grace
- [ ] `tests/tlv_framing.rs` in `opt/relay/` — covers TLV encode/decode roundtrip
- [ ] `tests/udp_session.rs` in `agent/solys/tests/` — covers run_udp_relay_session lifecycle
- [ ] No test infrastructure upgrades needed — existing `cargo test` patterns suffice

## Security Domain

> `security_enforcement` key absent from `.planning/config.json` — treat as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Tunnel authentication uses existing relay_token/HMAC (unchanged) |
| V3 Session Management | No | UDP sessions are stateless per-datagram (no session tokens) |
| V4 Access Control | Yes | Only authorized agent can bind a UDP port — enforced by existing TunnelConnect auth |
| V5 Input Validation | Yes | TLV parser validates type byte and length field; reject malformed frames |
| V6 Cryptography | No | No new crypto — datagrams are forwarded blindly (RakNet encryption is end-to-end) |
| V13 API & Web | Partially | Dashboard shows Bedrock address — no new API for address display (uses existing endpoints) |

### Known Threat Patterns for Rust/UDP Relay

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| UDP port exhaustion (attacker opens many tunnels to exhaust 100-port range) | Denial of Service | Rate limiter already exists (`rate_limiter.rs` based on `requests_per_minute`). The port range is finite per D-06 (100 ports). Backend controls allocation rate. |
| Malformed TLV frame causes OOM | Tampering | Length field capped at 65535 (max UDP datagram). Buffer pre-allocated at 65536 bytes. Reject type bytes other than `0x01` and `0xFF`. |
| Port scanning — attacker sends datagrams to non-allocated ports | Information Disclosure | Gateway ignores datagrams on unbound ports (firewalled by NLB or OS). No response sent — silent drop. |
| Unauthorized port binding — agent claims loader="bedrock" but is not authorized | Spoofing | Existing `auth::authorize` validates relay_token maps to server_id. The backend's `game_type` determines whether UDP is appropriate — future enhancement could add backend-side validation. |
| Grace period starvation — attacker opens/closes tunnels rapidly to prevent port reuse | Denial of Service | Limit concurrent grace-period sockets. After N grace-period entries, close immediately (no grace). N is configurable (recommend: 10). |

## Planning Guidance

### Recommended Plan Breakdown (4 Waves)

**Wave 1: Backend — UDP Port Pool + Protocol-Aware Dispatch**
*Files: api/*, agent/solys/src/state.rs*
- Add UDP seed row migration to `port_pools` (19132-19231)
- Modify `PortAllocationUseCase` to accept `protocol: &str` parameter
- Update server creation handler to allocate UDP port when `game_type=bedrock`
- Add optional `loader` field to `ServerRelayInfo` in `node_protocol.rs`
- Update `RelayService::push_relay_config` to include `loader` field for Bedrock servers
- Update agent's `agent_connection.rs` to parse `loader` from `RelayConfigSync` and pass to `RelayServerConfig`
- *(No structural changes to RelayServerConfig — `public_port` is reused for UDP)*

**Wave 2: Agent — UDP Session Handler in relay_session.rs**
*Files: agent/solys/src/handlers/relay_session.rs, relay_client.rs*
- Add `run_udp_relay_session()` to `relay_session.rs` with TLV framing and `UdpSocket::send_to`/`recv_from`
- Modify `drive_inbound_streams` in `relay_client.rs` to detect Bedrock config and spawn UDP session instead of TCP relay session
- Update `TunnelConnect` JSON to include `loader: "bedrock"` field for Bedrock servers (add to `connect_msg` serde json in relay_client.rs)
- Add TLV framing helpers (encode/decode) as shared functions
- Write unit tests for TLV framing roundtrip

**Wave 3: Gateway — UdpSocket Management + Port Registry**
*Files: opt/relay/src/udp.rs (new), tunnel.rs, state.rs, config.rs, registry.rs, main.rs*
- Create `opt/relay/src/udp.rs` with:
  - `UdpPortRegistry` struct (DashMap-based, port→server_id mapping)
  - `UdpPortEntry` with `socket: Arc<UdpSocket>`, `grace_task: Option<JoinHandle>`
  - `bind_port()`, `unbind_port()`, `start_grace_period()` methods
  - `run_udp_player_listener()` task that polls the recv_from loop
- Modify `tunnel.rs`: add `loader` field to `TunnelConnect`; add dispatch logic after authorize step for Bedrock servers
- Modify `state.rs`: add `udp_registry: UdpPortRegistry` to `AppState`
- Modify `config.rs`: add `UdpConfig { port_start, port_end, grace_period_secs }` to `Config`
- Modify `main.rs`: spawn `udp::run_udp_player_listener()` background task
- Modify `registry.rs`: no structural change needed — Bedrock servers don't use subdomain registry
- Write integration tests for UdpSocket bind/unbind lifecycle

**Wave 4: DNS SRV Records + NLB Config + Dashboard UX**
*Files: app/src/components/ConnectivitySection.jsx, TunnelHealthCard.jsx; agent/solys/src/handlers/dns.rs (or new); infra*
- **If Route 53:** Add `aws-sdk-route53` to backend or agent Cargo.toml, create SRV record on server create, delete on server delete
- **If Cloudflare:** Extend `dns.rs` with SRV record type support via Cloudflare API
- **Infra:** Add UDP listener for ports 19132-19231 to NLB, add UDP rules to EC2 security group
- **Dashboard:** Extend `ConnectivitySection.jsx` to display Bedrock address (IP:port + SRV-based friendly name)
- **Dashboard:** Update `TunnelHealthCard.jsx` to show "UDP" mode label for Bedrock servers

### Ordered Dependency Graph
```
Wave 1 (Backend) ──► Wave 2 (Agent) ──► Wave 3 (Gateway) ──► Wave 4 (DNS + UI + Infra)
       │                    │                    │
       │                    │                    └── Depends on Wave 2's TLV framing
       │                    │                         and TunnelConnect(loader) working
       │                    └── Depends on Wave 1's port pool
       │                         + ServerRelayInfo.loader field
       └── Independent of other waves
```

**Key design thread:** The `loader` field flows through:
1. Backend sets it in `ServerRelayInfo` (Wave 1)
2. Agent includes it in `TunnelConnect` JSON (Wave 2) 
3. Gateway reads it to decide UdpSocket vs. TCP routing (Wave 3)

All 3 waves must agree on the field name and serialization strategy. Using `#[serde(default)]` on both ends ensures backward compatibility with existing Java-only agents/gateways.

## Sources

### Primary (HIGH confidence)
- **[VERIFIED: Code analysis]**
  - `opt/relay/src/tunnel.rs` — TunnelConnect struct, WS bridge, yamux session management
  - `opt/relay/src/player.rs` — TCP player listener with MC Handshake parsing
  - `opt/relay/src/registry.rs` — Registry with by_subdomain/by_server_id DashMap
  - `opt/relay/src/state.rs` — AppState struct
  - `opt/relay/src/config.rs` — Config struct with TOML+env loading
  - `opt/relay/src/main.rs` — Application entrypoint and task spawning
  - `opt/relay/Cargo.toml` — tokio 1.x with "full" features (UdpSocket available)
  - `agent/solys/src/handlers/relay_session.rs` — Existing run_relay_session pattern
  - `agent/solys/src/handlers/relay_client.rs` — Tunnel lifecycle, heartbeat, stream drive
  - `agent/solys/src/state.rs` — RelayServerConfig, RelayClientHandle, RelayManager
  - `agent/solys/src/agent_connection.rs` — RelayConfigSync handler
  - `agent/solys/Cargo.toml` — tokio 1.x with "full" features
  - `api/src/application/use_cases/port_allocation_use_case.rs` — Port allocation logic
  - `api/src/domain/server/entities/port_pool.rs` — PortPool struct and next-available logic
  - `api/migrations/20260409000002_create_port_pools_table.sql` — Schema with protocol='udp' support
  - `api/src/presentation/ws/node_protocol.rs` — ServerRelayInfo, NodeMessage enum
  - `api/src/application/services/relay_service.rs` — push_relay_config construction
  - `api/src/presentation/handlers/server_handlers.rs` — Bedrock loader dispatch
  - `agent/solys/src/handlers/dns.rs` — Cloudflare DNS handler for existing A records
  - `opt/relay/DEPLOY.md` — NLB, security group, Route 53 wildcard config
  - `app/src/components/ConnectivitySection.jsx` — Existing connectivity UI
  - `app/src/components/TunnelHealthCard.jsx` — Existing tunnel health card
  - `app/src/hooks/useConnectivity.js` — Existing connectivity data hook
  - `73-CONTEXT.md` — All locked decisions (D-01 through D-14)

### Secondary (MEDIUM confidence)
- **AWS NLB documentation** — UDP listener behavior, idle timeout defaults, proxy protocol v2 limitations
- **RakNet protocol** — UDP buffer sizing guidance (max fragment ~64 KiB)

### Tertiary (LOW confidence)
- None — all core technical claims in this research are verified against the actual codebase and decision log.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Verified against both Cargo.toml files. tokio provides UdpSocket, yamux is already in use.
- Architecture: HIGH — All patterns derived from existing, working code (relay_session.rs, tunnel.rs, player.rs patterns). D-01 through D-14 comprehensively documented.
- Pitfalls: MEDIUM — UDP idle timeout and proxy protocol behavior are based on AWS NLB documentation (standard behavior); buffer sizing is based on protocol knowledge (LOW for exact Budrock RakNet spec, MEDIUM for general UDP best practice).

**Research date:** 2026-06-13
**Valid until:** 2026-07-13 (30 days — stack is stable Rust ecosystem)
