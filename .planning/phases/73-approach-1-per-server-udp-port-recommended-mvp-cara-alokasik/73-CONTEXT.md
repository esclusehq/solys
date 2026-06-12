# Phase 73: Approach 1: Per-Server UDP Port (⭐ Recommended MVP) - Context

**Gathered:** 2026-06-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Add UDP relay support for Minecraft Bedrock Edition servers via dedicated per-server UDP ports on the relay gateway. Bedrock uses UDP (RakNet protocol) which cannot be routed through the existing TCP-only subdomain-based relay architecture. Instead, each Bedrock server gets a unique UDP port on the relay EC2, mapped to its agent tunnel via a length-prefixed datagram framing over yamux.

**In scope:**
1. Gateway UDP listener — per-server UdpSocket management in opt/relay
2. Datagram framing protocol (TLV format) over yamux streams
3. Agent UDP session handler (send_to/recv_from pattern)
4. Backend UDP port pool allocation (reuses existing port_pools table)
5. NLB UDP listener config + security group rules
6. Dashboard UX showing Bedrock server address (DNS SRV record + IP:port)
7. Player-side DNS SRV record support for cleaner Bedrock addresses

**Out of scope:**
- Multi-region relay failover (deferred from Phase 68)
- UDP relay on the existing *.play.esluce.net subdomain system (port-based, not subdomain-based)
- RakNet protocol parsing at the gateway (blind forwarding)
- Java Edition UDP support (not applicable)
</domain>

<decisions>
## Implementation Decisions

### Port Allocation Authority
- **D-01 (Port authority):** **Backend allocates from port_pools** — consistent with existing TCP port allocation. The `port_pools` table already has `protocol VARCHAR(10)` with `'udp'` support. Backend allocates a UDP port on server create and sends it via `RelayConfigSync`. The pool range config entry uses `protocol = 'udp'`.
- **D-02 (Port pool initialization):** A new global pool row with `protocol = 'udp'`, range 19132-19231, must be inserted into `port_pools` (migration or seed data). The existing row is TCP-only (25565-25665).

### Gateway UDP Port Lifecycle
- **D-03 (Protocol detection):** **Agent declares loader in TunnelConnect** — the TunnelConnect JSON message already carries the server identity. For Bedrock servers (`loader: "bedrock"`), the gateway opens a UdpSocket on the allocated port. No protocol field needed in RelayConfigSync — the gateway derives it from loader.
- **D-04 (Bind timing):** **Deferred — on-demand dedicated socket on first datagram** — The gateway opens a UdpSocket on the allocated port at tunnel connect time (port becomes accessible), but does NOT start the full forwarding session until the first datagram arrives from a player. Session creation is deferred to save yamux stream resources when no players are connected.
- **D-05 (Port release):** **30-second grace period** — After tunnel disconnect, keep the UdpSocket open for 30s, silently dropping incoming packets. Prevents port reuse collisions with in-flight player datagrams.
- **D-06 (Port range):** **19132-19231 (100 ports)** — Configurable via gateway env config. Standard Bedrock port (19132) as range start. 100 concurrent Bedrock servers before needing to expand.

### Datagram Framing Protocol
- **D-07 (Stream model):** **Long-lived session stream** — Single yamux stream per server opened when first datagram arrives, carries all datagrams both directions. Stream stays open for the tunnel lifetime. Avoids per-datagram stream overhead.
- **D-08 (Wire format):** **TLV (Type-Length-Value)** — `[1-byte type][4-byte big-endian length][payload]`. Type field allows future extension (e.g., control frames, keepalive). Initial types: `0x01 = datagram`. Length covers only the payload.
- **D-09 (Header size):** **4-byte length** — Covers full UDP datagram range (0-65535). Consistent with other binary framing in the codebase.

### Agent UDP Session Handling
- **D-10 (Local forwarding pattern):** **UdpSocket with send_to/recv_from (connectionless)** — Agent opens UdpSocket, binds to ephemeral port, sends datagrams to container via send_to(), receives replies via recv_from(). More flexible than connect() — allows future multi-target scenarios.
- **D-11 (Container address source):** **Use existing `local_mc_addr` from RelayConfig** — The RelayConfig already carries `local_mc_addr` (e.g., `127.0.0.1:19132`). Agent uses this as the UDP send target. No new config field needed.
- **D-12 (Session task):** New `run_udp_relay_session` function in `relay_session.rs` — mirrors existing `run_relay_session` but uses UdpSocket instead of TcpStream. Reads framed datagrams from yamux, sends via UdpSocket to container; reads from UdpSocket, writes framed to yamux.

### Player Connection UX
- **D-13 (Bedrock address display):** **Dashboard + DNS SRV record** — Dashboard shows both:
  - Direct IP:port (`esluce.com:{port}` or relay IP:port) for manual entry
  - A friendly DNS name backed by an SRV record: `bedrock-{subdomain}.play.esluce.net`
  - Bedrock client uses the SRV record (`_minecraft._udp.bedrock-xxx.play.esluce.net → port`) for cleaner UX
- **D-14 (DNS SRV management):** Backend or agent needs Route 53 SRV record creation capability (extends existing A-record DNS automation).

### the agent's Discretion
- Exact TLV type byte assignments (recommend: `0x01` for datagram, reserve `0x00` for invalid, `0xFF` for control)
- UDP socket buffer sizes (recommend: 64 KiB matching BRIDGE_BUFFER_BYTES)
- Whether the deferred-session check is based on a shared UDP accept loop or individual socket readiness (recommend: individual socket with recv_from() poll in the session task, spawned when first datagram matches the port)

### Folded Todos
None — no todos matched Phase 73.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and prior roadmap context
- `.planning/ROADMAP.md` § Phase 73 — Goal: "Approach 1: Per-Server UDP Port (⭐ Recommended MVP)" — layer-by-layer change description
- `.planning/ROADMAP.md` § Phase 72 — Prior phase: Bedrock game type added (direct mode only)
- `.planning/ROADMAP.md` § Phase 68 — Full relay infrastructure spec: tunnel protocol, gateway architecture, player routing
- `.planning/ROADMAP.md` § Phase 69 — Per-server tunnel refactor (one WS per server)

### Phase 68 carryforward (relay fundamentals + Bedrock deferral)
- `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-CONTEXT.md` — D-16 (Bedrock UDP deferred), D-05 (Gateway architecture: Rust + Axum on EC2), D-06 (player routing via subdomain → not used for Bedrock)
- `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-CONTEXT.md` § `<code_context>` — Gateway tunnel/player code patterns
- `opt/relay/src/tunnel.rs` — Tunnel connect protocol (TunnelConnect, TunnelHeartbeat). Phase 73 extends to carry loader field.
- `opt/relay/src/player.rs` — TCP player routing. Phase 73 adds parallel UDP path.
- `opt/relay/src/config.rs` — Gateway config struct. Phase 73 adds UDP port range config.
- `opt/relay/src/state.rs` — AppState. Phase 73 may add a UDP port registry.

### Phase 69-70 carryforward (per-server tunnels + config delivery)
- `.planning/phases/69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv/69-CONTEXT.md` — D-01 (HashMap storage), D-11 (wildcard DNS, not used for Bedrock), D-13/D-14 (config in task payload)
- `.planning/phases/70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe/70-CONTEXT.md` — D-07 (RelayConfigSync shape with public_port — Phase 73 reuses same field for UDP port), D-01/D-02 (config storage split)
- `agent/solys/src/state.rs:137-162` — RelayServerConfig + RelayClientHandle structs. Phase 73 may need protocol discriminator.
- `agent/solys/src/handlers/relay_client.rs` — Tunnel client lifecycle. Phase 73 adds Bedrock-aware connect logic.
- `agent/solys/src/handlers/relay_session.rs` — Session handler. Phase 73 adds `run_udp_relay_session`.

### Phase 72 (Bedrock game type — direct mode only, now extended)
- `.planning/phases/72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te/72-RESEARCH.md` — Bedrock game type architecture
- `.planning/phases/72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te/72-PATTERNS.md` — Pattern map for Bedrock support
- `api/src/domain/server/entities/game_type.rs` — Game type enum with "bedrock" variant
- `api/src/presentation/handlers/server_handlers.rs` — Handler with dynamic loader dispatch

### Backend database schema
- `api/migrations/20260409000002_create_port_pools_table.sql` — Port pool table already has `protocol` column with `CHECK (protocol IN ('tcp', 'udp', 'both'))`. Phase 73 adds a new pool row for UDP.
- `api/migrations/20260612000001_add_bedrock_game_type.sql` — Bedrock game type row in game_types table

### Existing DNS automation
- `agent/solys/src/handlers/dns.rs` — Existing Cloudflare DNS client. Phase 73 may extend for Route 53 SRV records.
- `.planning/phases/51-automasi-dns-cloudflare/51-CONTEXT.md` — DNS automation patterns. Phase 73 adds SRV record support.

### Codebase maps (tech context)
- `.planning/ROADMAP.md` — Full roadmap with all phase dependencies
- `.planning/STATE.md` — Project state and progress

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`port_pools` table** (api/migrations/20260409000002_create_port_pools_table.sql) — Already has `protocol` column with `'udp'` and `'both'` support. No schema migration needed — just need a new pool row insert for UDP range 19132-19231.
- **`UdpSocket` in tokio** — Already available in the Rust standard library. `tokio::net::UdpSocket` is used by many projects. No new external dependency needed.
- **`relay_session.rs`** (agent) — Existing `run_relay_session` with bidirectional copy pattern can be adapted for UDP: replace TcpStream with UdpSocket, add datagram framing.
- **`player.rs`** (opt/relay) — Existing TCP player listener pattern. Phase 73 adds parallel UDP player listener.
- **`resolve_container_addr`** (relay_client.rs) — Docker container IP resolution for TCP. For UDP, the existing `local_mc_addr` from config is sufficient (D-11).
- **`Registry`** (opt/relay/registry.rs) — Server registry HashMap. Phase 73 extends to map port → server_id for UDP.

### Established Patterns
- **Bidirectional copy with byte counting** — `relay_session.rs` copy_bidirectional_with_count. Datagram-based copy follows same pattern but framed.
- **Per-server tunnel with CancellationToken** — Phase 69-70's per-server struct pattern. Phase 73's UDP session reuses the same lifecycle.
- **Config delivery via RelayConfigSync** — Phase 70's push-based config. Bedrock servers just carry a UDP port in `public_port` field.
- **NDJSON control stream** — Existing control stream pattern for heartbeat/connect messages. Phase 73 adds decoder/encoder for framed UDP data.
- **Length-prefix framing** — `read_json_message` in tunnel.rs already uses length-prefix pattern (NDJSON). Phase 73 adapts for binary datagrams.

### Integration Points
- **TunnelConnect message** (opt/relay/tunnel.rs) — Add `loader` or `game_type` field so gateway knows to open UdpSocket vs. expect TCP routing.
- **Gateway config** (opt/relay/config.rs) — Add `udp_port_range_start`, `udp_port_range_end` config fields.
- **Gateway AppState** (opt/relay/state.rs) — May need a `UdpPortRegistry` alongside existing `Registry`.
- **relay_session.rs** (agent) — New `run_udp_relay_session()` function alongside existing `run_relay_session()`.
- **relay_client.rs** (agent) — `drive_inbound_streams` currently spawns `run_relay_session` for each incoming yamux stream. For Bedrock, the single long-lived UDP stream is spawned at tunnel connect.
- **Backend server creation** — When creating a server with game_type=bedrock, backend allocates from UDP port pool.
- **Dashboard** (app/) — Server details page shows Bedrock address (IP:port + SRV name). Extends ConnectivitySection to show bedrock-style addresses.
- **DNS module** (agent/handlers/dns.rs or new module) — SRV record creation for `_minecraft._udp.bedrock-{subdomain}.play.esluce.net`.
- **NLB + Security Group** — AWS config: add UDP listener on port range 19132-19231 to existing NLB, add UDP rules to security group.
</code_context>

<specifics>
## Specific Ideas

### TunnelConnect extension for Bedrock
The existing TunnelConnect message in `opt/relay/tunnel.rs` should gain an optional loader field:

```json
{
    "type": "tunnel_connect",
    "relay_token": "...",
    "server_id": "...",
    "subdomain": "...",
    "loader": "bedrock",         // new field
    "public_port": 19133,         // UDP port allocated by backend
    "agent_public_ip": "...",
    "region": "..."
}
```

When `loader` is `"bedrock"`, the gateway skips the subdomain-based TCP routing and instead sets up a UdpSocket on `public_port`.

### Datagram TLV wire format
```
Gateway → Agent (player→container):
  [0x01] [4-byte big-endian length] [datagram payload from player]

Agent → Gateway (container→player):
  [0x01] [4-byte big-endian length] [datagram payload from container]
```

Both directions use the same long-lived yamux stream. Type byte `0x01` = datagram payload. Reserve `0xFF` for future control messages.

### DNS SRV record format
For Bedrock player discovery:
```
Service: _minecraft
Protocol: _udp
Name: bedrock-{subdomain}.play.esluce.net
TTL: 60
Target: relay.esluce.net
Port: {allocated-udp-port}
Priority: 0
Weight: 0
```

Minecraft Bedrock client resolves `bedrock-xxx.play.esluce.net` → queries SRV → connects to `relay.esluce.net:{port}` via UDP.

### Port allocation pool initialization
```sql
-- New pool row for UDP Bedrock ports
INSERT INTO port_pools (node_id, port_range_start, port_range_end, current_port, protocol, is_active)
VALUES (NULL, 19132, 19231, 19132, 'udp', true);
```
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. Phase 73 focuses on per-server UDP port relay for Bedrock.

### Reviewed Todos (not folded)
None — no todos matched Phase 73 during cross-reference scan.
</deferred>

---

*Phase: 73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik*
*Context gathered: 2026-06-12*
