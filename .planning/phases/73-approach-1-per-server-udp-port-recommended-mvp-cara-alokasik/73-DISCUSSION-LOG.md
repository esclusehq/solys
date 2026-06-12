# Phase 73: Approach 1: Per-Server UDP Port - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-12
**Phase:** 73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik
**Areas discussed:** Port allocation authority, Datagram framing, Port range & lifecycle, Player connection UX, Agent UDP session

---

## Port Allocation Authority

| Option | Description | Selected |
|--------|-------------|----------|
| Backend (port_pools) | Consistent with existing TCP mechanism — port_pools table already has protocol='udp' support | ✓ |
| Gateway auto-allocates | Simpler gateway logic, but less backend visibility and separate port config | |
| Backend allocates, gateway binds | Backend owns the pool, gateway just binds the pre-allocated port | |

**User's choice:** Backend (port_pools)
**Notes:** Port_pools table already has protocol='udp' support — no migration needed, just a seed row

### Protocol Detection (sub-question)

| Option | Description | Selected |
|--------|-------------|----------|
| Agent declares loader in TunnelConnect | TunnelConnect JSON already carries loader/game_type — "bedrock" → gateway opens UdpSocket | ✓ |
| Backend includes protocol in RelayConfigSync | Extend ServerRelayInfo to include protocol field | |

**User's choice:** Agent declares loader in TunnelConnect

### Bind Timing (sub-question)

| Option | Description | Selected |
|--------|-------------|----------|
| On TunnelConnect immediately | Open port right when tunnel is established | |
| Deferred until first player datagram | Open on-demand when first UDP packet hits a range port | ✓ |

**User's choice:** Deferred until first player datagram

### Deferred Mechanism (sub-question)

| Option | Description | Selected |
|--------|-------------|----------|
| Single socket + SO_ORIGINAL_DST | One shared UdpSocket, iptables REDIRECT to recover original dest port | |
| On-demand dedicated socket | Open UdpSocket on first datagram arrival on that port | ✓ |
| Simplify — bind on TunnelConnect | Open dedicated UdpSocket when tunnel connects | |

**User's choice:** On-demand dedicated socket

---

## Datagram Framing

### Yamux Stream Model

| Option | Description | Selected |
|--------|-------------|----------|
| Long-lived session stream | Single yamux stream per server, open for tunnel lifetime, carries all datagrams | ✓ |
| On-demand streams per datagram burst | Open yamux stream when datagrams flow, close on idle | |

**User's choice:** Long-lived session stream

### Header Size

| Option | Description | Selected |
|--------|-------------|----------|
| 2-byte length (65535 max) | Covers full UDP datagram range | |
| 4-byte length | Safer, consistent with other binary protocols | ✓ |

**User's choice:** 4-byte length

### Wire Format

| Option | Description | Selected |
|--------|-------------|----------|
| Length-prefixed only | [4-byte length][datagram bytes]. Simple. | |
| Type-length-value (TLV) | [packet_type][4-byte length][payload]. Extensible. | ✓ |

**User's choice:** Type-length-value (TLV)

---

## Port Range & Lifecycle

### Port Range

| Option | Description | Selected |
|--------|-------------|----------|
| 19132-19231 (100 ports) | Standard Bedrock port + 99 slots | ✓ |
| 19132-19631 (500 ports) | More headroom | |
| 19132-20131 (1000 ports) | Maximum safe range | |

**User's choice:** 19132-19231 (100 ports)

### Port Release

| Option | Description | Selected |
|--------|-------------|----------|
| Immediate release | Close socket and free port as soon as tunnel disconnects | |
| Grace period (30s) | Keep socket open for 30s, drop packets silently | ✓ |

**User's choice:** Grace period (30s)

---

## Player Connection UX

### Bedrock Address

| Option | Description | Selected |
|--------|-------------|----------|
| Dashboard shows IP:port directly | Player copies IP:port and enters manually | |
| Dashboard + DNS SRV record | Friendly DNS name backed by SRV record | ✓ |
| Both | Both methods available | |

**User's choice:** Dashboard + DNS SRV record

---

## Agent UDP Session

### Agent UDP Pattern

| Option | Description | Selected |
|--------|-------------|----------|
| UdpSocket::connect to container target | Simple, one connected socket | |
| UdpSocket with send_to/recv_from | Connectionless mode, more flexible | ✓ |

**User's choice:** UdpSocket with send_to/recv_from

### Container Address

| Option | Description | Selected |
|--------|-------------|----------|
| Use existing local_mc_addr from RelayConfig | Already carries the address, no new config | ✓ |
| Resolve via Docker inspect | Same pattern as Java TCP | |

**User's choice:** Use existing local_mc_addr from RelayConfig

---

## the agent's Discretion

- Exact TLV type byte assignments (recommend: 0x01 for datagram, 0xFF for control)
- UDP socket buffer sizes (recommend: 64 KiB matching BRIDGE_BUFFER_BYTES)
- Deferred session check mechanism (recommend: individual socket with recv_from() poll, spawned on first datagram match)

## Deferred Ideas

None — discussion stayed within phase scope.
