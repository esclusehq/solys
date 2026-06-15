# Phase 68: Escluse Relay Infrastructure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-07
**Phase:** 68-escluse-relay-infrastructure-objective-implement-esluce-rela
**Mode:** yolo-autonomous (single-pass, no interactive Q&A; rationale below)
**Areas discussed:** 9 (tunnel protocol, gateway architecture, server_id auth, mode policy, DNS strategy, security hardening, monitoring, scope confirmation, backward compatibility)

---

## Mode rationale

The user invoked `/gsd-discuss-phase 68` with no flags and `mode: yolo` in `.planning/config.json`. The discuss-phase workflow is interactive by design (4-question turns per area per `modes/default.md`); in yolo mode the user has explicitly stepped back from interactive Q&A. Given the rich ROADMAP specification (9 numbered requirements) and the strong Phase 67 carryforward (D-10 explicitly defers Relay to this phase), single-pass autonomous decisions were made with clear `the agent's Discretion` boundaries. User can edit CONTEXT.md before `/gsd-plan-phase 68` if any decision is wrong.

---

## Tunnel Protocol & Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| WebSocket over TLS 1.3 | Reuses `tokio-tungstenite 0.26` (already in stack); custom framing for multiplexed streams | ✓ |
| Raw TCP + custom handshake | More efficient but adds cert management complexity and breaks WebSocket symmetry with existing agent↔backend | |
| QUIC (quinn) | Best mobile performance but no Rust client in stack; new dep | |
| SSH tunneling | Standard tooling but wrong tool for this; hides ownership/auth model | |

**Decision:** D-01 = WebSocket over TLS 1.3
**Rationale:** Lowest friction — reuses existing infra, follows established WS pattern, no new dependencies beyond yamux. Custom framing layer is small.

---

## Multiplexing Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| yamux | Single WebSocket per agent, multiplexed streams; per-server stream isolation | ✓ |
| One TCP connection per server | Simpler code, more connection overhead, harder to atomically replace on reconnect | |
| HTTP/2 streams | Possible but adds HTTP semantics overhead for what is essentially raw TCP forwarding | |

**Decision:** D-02 = yamux
**Rationale:** Industry standard (used by tonic/gRPC). Single connection per agent simplifies reconnection logic. Stream isolation gives per-server error handling.

---

## Server ID Authentication at Relay

| Option | Description | Selected |
|--------|-------------|----------|
| Per-agent token + backend HTTP introspection | Agent sends token; relay validates against backend on each registration; cached for tunnel lifetime | ✓ |
| Signed JWT (self-contained) | Agent gets JWT at registration; relay verifies signature; no backend round-trip | |
| Mutual TLS (mTLS) | Cert-based; very secure but adds cert lifecycle complexity for agents | |

**Decision:** D-09/D-10 = per-agent token + backend introspection
**Rationale:** Lowest operational complexity, easiest to revoke (just delete the token). Introspection latency budget <50ms p99 is achievable. The HTTP round-trip is one-time per tunnel registration, not per request.

---

## Mode Selection Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Fully automatic (no user override) | Agent picks mode; user has no control | |
| Automatic with per-server user override | Agent picks by default; user can pin "Force Direct" or "Force Relay" per server | ✓ |
| User picks mode at server creation (one-way) | Simpler but loses automatic optimization | |

**Decision:** D-12 = automatic with per-server override
**Rationale:** Best of both — automatic works for 95% of users, override exists for power users (e.g., paid tier users who want guaranteed Direct). Stored in `servers.connectivity_mode_override` (nullable text).

---

## Player Address UX

| Option | Description | Selected |
|--------|-------------|----------|
| Show relay address only | Simplest, but Direct Mode users lose the fast-path benefit | |
| Show both addresses when applicable | User can choose; shareable invite UI with QR codes | ✓ |
| Single address that auto-resolves | Like Cloudflare's load balancer; harder to debug; hides mode from user | |

**Decision:** D-14 = show both when applicable, primary = relay
**Rationale:** Transparency about mode is important for support ("I'm using relay because..."). Friends can use either address. Default "Copy join address" copies the relay one (always works).

---

## Pricing & Access Tier

| Option | Description | Selected |
|--------|-------------|----------|
| Free for all users in initial rollout | Absorb relay cost as platform cost; build user base | ✓ |
| Paid per server from day 1 | Recovers cost but limits adoption | |
| Free with usage caps (e.g., 100 GB/month per server) | Middle ground but adds metering complexity | |

**Decision:** D-15 = free for all
**Rationale:** Matches the "no port forwarding" Tier 1 differentiator from STRATEGI.md. Per-server relay cost is small (~few GB/month per active server, ~$0.01/month bandwidth at AWS prices). Future pricing is a follow-up.

---

## Minecraft Edition Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Java TCP only | Matches ROADMAP requirement 9 | ✓ |
| Java + Bedrock UDP | Architecturally different (UDP tunnel); needs `quinn` or similar | |

**Decision:** D-16 = Java TCP only
**Rationale:** Matches ROADMAP. Bedrock UDP is a follow-up phase; yamux is TCP-only, would need a separate QUIC-based path.

---

## AWS Region & HA

| Option | Description | Selected |
|--------|-------------|----------|
| Single region (ap-southeast-1) + single AZ | Matches existing infra; lowest complexity | ✓ |
| Multi-region with geo-DNS | Global latency and HA but ~3× the AWS cost and ops complexity | |

**Decision:** D-17 = single region, single AZ
**Rationale:** Matches ROADMAP. Multi-region is a follow-up; the architecture is region-agnostic and can scale horizontally.

---

## Backward Compatibility for Existing `esluce.com` Records

| Option | Description | Selected |
|--------|-------------|----------|
| No migration — preserve existing Phase 51 records | All existing `<server>.play.esluce.com` A records continue to work; agent updates only when Direct Mode is active | ✓ |
| Migrate `esluce.com` to Route 53 | Unified DNS but loses Cloudflare DDoS protection + CDN | |
| Force re-registration of all records | Cleanest state but breaks any server that hasn't been touched by Phase 68 agent yet | |

**Decision:** D-24 = no migration
**Rationale:** Lowest risk; existing users see no behavior change; Phase 68's relay path is purely additive.

---

## Security Hardening

| Option | Description | Selected |
|--------|-------------|----------|
| Token + introspection + nonce-based replay protection | Defense in depth; matches ROADMAP requirements | ✓ |
| Token only | Simpler but allows replay attacks if handshake is captured | |
| mTLS only | Most secure but operational overhead | |

**Decision:** D-09, D-10, D-11 = token + introspection + nonce
**Rationale:** ROADMAP requires "replay protection" (req 7) and "TLS 1.3+" (req 7). Nonce-based replay protection is the standard pattern.

---

## the agent's Discretion

Items deferred to the agent (full list in CONTEXT.md `the agent's Discretion`):
- Exact Caddy config for player TLS termination
- Specific AWS instance type (recommend c6i.large)
- WebSocket frame max size, yamux window size
- Heartbeat payload contents
- Exact reconnect jitter formula
- Prometheus metric label values
- ALB vs NLB decision for player traffic (recommend NLB for raw TCP)
- Tunnel session rekeying cadence

---

## Deferred Ideas

(All preserved in CONTEXT.md `deferred` section. Summary:)
- Bedrock Edition UDP support
- Multi-region relay failover
- IPv6 dual-stack relay
- HTTP/3 (QUIC) for tunnel transport
- Per-server relay pricing tier
- Custom Esluce Relay agent binary
- IPv6-only nodes
- Migrating Phase 51's `esluce.com` zone to Route 53
- Re-architecting the two competing `Server` models (pre-existing tech debt)
