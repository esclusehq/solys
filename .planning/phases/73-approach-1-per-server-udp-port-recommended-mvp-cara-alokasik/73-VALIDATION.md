---
phase: 73
slug: approach-1-per-server-udp-port-recommended-mvp-cara-alokasik
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-13
---

# Phase 73 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (opt/relay, agent/solys, api) |
| **Config file** | opt/relay/Cargo.toml, agent/solys/Cargo.toml, api/Cargo.toml |
| **Quick run command** | `cargo test -p relay-gateway --test udp_bind; cargo test -p solys -- udp_relay_session; cargo test -p api` |
| **Full suite command** | `cargo test --workspace 2>&1 | tail -30` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check` for modified crate
- **After every plan wave:** Run `cargo test` for affected workspace members
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 73-01-01 | 01 | 1 | D-01/D-02/D-06 | — | UDP port pool seed row | migration | `cargo test` (sqlx migration check) | ❌ W0 | ⬜ pending |
| 73-01-02 | 01 | 1 | D-01 | — | Protocol-aware allocate_port | unit | `cargo test -p api -- port_allocation_protocol` | ❌ W0 | ⬜ pending |
| 73-01-03 | 01 | 1 | D-03/D-05 | — | ServerRelayInfo.loader field set for Bedrock | code review | grep -c loader relay_service.rs | ✅ grep | ⬜ pending |
| 73-02-01 | 02 | 2 | D-08/D-09 | T-73-02-01 | TLV encode/decode roundtrip | unit | `cargo test -p relay-gateway -- tlv_roundtrip` | ❌ W0 | ⬜ pending |
| 73-02-02 | 02 | 2 | D-03/D-10 | — | TunnelConnect.loader sent for Bedrock | code review | grep -c loader relay_client.rs | ✅ grep | ⬜ pending |
| 73-02-03 | 02 | 2 | D-07/D-11/D-12 | T-73-02-03 | run_udp_relay_session forwards datagrams | integration | `cargo test -p solys -- udp_relay_session` | ❌ W0 | ⬜ pending |
| 73-03-01 | 03 | 3 | D-04/D-06 | — | UdpSocket bind on TunnelConnect(bedrock) | integration | `cargo test -p relay-gateway --test udp_bind` | ❌ W0 | ⬜ pending |
| 73-03-02 | 03 | 3 | D-04/D-07 | — | Per-port recv_from loop + yamux stream on first datagram | integration | `cargo test -p relay-gateway -- udp_session_spawn` | ❌ W0 | ⬜ pending |
| 73-03-03 | 03 | 3 | D-05 | — | Grace period prevents immediate port reuse | integration | `cargo test -p relay-gateway -- grace_period` | ❌ W0 | ⬜ pending |
| 73-04-01 | 04 | 4 | D-13/D-14 | T-73-04-02 | create_srv_record + delete_srv_record | grep | grep -c create_srv_record relay_service.rs | ✅ grep | ⬜ pending |
| 73-04-02 | 04 | 4 | D-13 | — | Dashboard Bedrock address display | manual | N/A — visual | ❌ Manual | ⬜ pending |
| 73-04-03 | 04 | 4 | D-13 | — | TunnelHealthCard UDP badge | manual | N/A — visual | ❌ Manual | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `opt/relay/tests/udp_bind.rs` — integration test for UdpSocket bind/unbind/grace lifecycle
- [ ] `opt/relay/tests/tlv_framing.rs` — unit test for TLV encode/decode roundtrip
- [ ] `opt/relay/tests/udp_session.rs` — integration test for per-port recv_from session spawning
- [ ] `agent/solys/tests/udp_session.rs` — integration test for run_udp_relay_session lifecycle
- [ ] `api/tests/port_allocation_protocol.rs` — unit test for protocol-aware dispatch
- [ ] Migration test sqlx check for protocol='udp' seed row

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dashboard shows Bedrock address | D-13 | Visual UI inspection | Open server detail page for Bedrock server, verify ConnectivitySection shows relay.esluce.net:{port} + bedrock-{subdomain}.play.esluce.com |
| TunnelHealthCard shows UDP badge | D-13 | Visual UI inspection | Verify TunnelHealthCard shows "UDP" badge for Bedrock servers, no badge for Java servers |
| NLB UDP listener + security group | D-06 | Manual AWS console | Verify NLB has UDP listener on 19132-19231, security group allows UDP inbound on that range |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** {pending / approved YYYY-MM-DD}
