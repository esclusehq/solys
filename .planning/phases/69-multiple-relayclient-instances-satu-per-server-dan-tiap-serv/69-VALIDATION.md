---
phase: 69
slug: multiple-relayclient-instances-satu-per-server-dan-tiap-serv
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-09
---

# Phase 69 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[cfg(test)]` + `cargo test` |
| **Config file** | `Cargo.toml` (workspace root) |
| **Quick run command** | `cargo test -p escluse-relay-gateway --lib` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --workspace`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 69-01-01 | 01 | 1 | — | — | N/A (migration — data only) | manual | `psql -c "SELECT column_name FROM information_schema.columns WHERE table_name = 'servers' AND column_name = 'subdomain';"` | ❌ W0 | ⬜ pending |
| 69-01-02 | 01 | 1 | — | — | Subdomain generation on create | unit | `grep -n "subdomain" api/src/domain/entities/server.rs && grep -n "sha2\|hex::encode\|subdomain" api/src/application/services/server_service.rs` | ❌ W0 | ⬜ pending |
| 69-01-03 | 01 | 1 | — | — | relay.connect push on reconnect | unit | `grep -n "push_all_servers\|relay.connect" api/src/presentation/handlers/node_ws_handler.rs` | ❌ W0 | ⬜ pending |
| 69-02-01 | 02 | 2 | — | T-69-01 | PerServerRuntime struct with child_token | unit | `grep -n "PerServerRuntime\|child_token" src/handlers/relay_client.rs` | ❌ W0 | ⬜ pending |
| 69-02-02 | 02 | 2 | — | T-69-02 | HashMap add/remove per server | unit | `grep -n "RwLock.*HashMap\|HashMap.*ServerId" src/handlers/relay_client.rs` | ❌ W0 | ⬜ pending |
| 69-02-03 | 02 | 2 | — | T-69-03 | RelayConfig split + task payload handling | unit | `grep -n "PerServerRelayConfig\|shared_config\|task.payload" src/handlers/relay.rs src/handlers/relay_client.rs` | ❌ W0 | ⬜ pending |
| 69-03-01 | 03 | 3 | — | — | relay_session.rs doc update for per-server sessions | manual | `grep -n "per-server\|PerServerRuntime" src/handlers/relay_session.rs` | ❌ W0 | ⬜ pending |
| 69-03-02 | 03 | 3 | — | — | mod.rs dispatch arm for server_id extraction | unit | `grep -n "server_id\|payload" src/handlers/mod.rs` | ❌ W0 | ⬜ pending |
| 69-04-01 | 04 | 3 | — | — | Heartbeat staggering (0-10s jitter) | unit | `grep -n "gen_range\|jitter\|stagger" src/handlers/relay_client.rs` | ❌ W0 | ⬜ pending |
| 69-04-02 | 04 | 3 | — | T-69-04 | Per-server AtomicU64 bandwidth accounting | unit | `grep -n "AtomicU64\|bytes_transferred" src/handlers/relay_client.rs` | ❌ W0 | ⬜ pending |
| 69-05-01 | 05 | 3 | — | T-69-05 | Gateway auth 1:N relay_token→server_id | integration | `grep -n "authorize\|relay_token\|server_id" opt/relay/src/auth.rs` | ❌ W0 | ⬜ pending |
| 69-05-02 | 05 | 3 | — | T-69-06 | N concurrent WS from same agent IP | integration | `grep -n "concurrent\|same.*IP\|multi.*agent" opt/relay/src/tunnel.rs` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/handlers/relay_client.rs` — test stubs for PerServerRuntime, HashMap connect/disconnect
- [ ] `opt/relay/src/auth.rs` — test for 1:N token authorization
- [ ] `cargo test --workspace` — confirm existing tests pass before changes

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Subdomain migration rollback | D-10 | Destructive — affects production data | Verify `servers.subdomain` column via psql before/after |
| End-to-end tunnel with subdomain | D-12 | Requires live relay gateway and agent | Deploy Plan 69-05 first, then verify player can connect to `{hash}.play.esluce.net:25565` |
| Agent reconnect → backend pushes all servers | D-07 | Requires backend WS round-trip | Kill agent → restart → verify backend sends relay.connect for each server |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
