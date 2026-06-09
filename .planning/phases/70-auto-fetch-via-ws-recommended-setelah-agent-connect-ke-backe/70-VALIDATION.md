---
phase: 70
slug: auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-09
---

# Phase 70 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | none — Rust unit tests inline |
| **Quick run command** | `cargo check` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 70-01-01 | 01 | 1 | — | T-70-01 | ws handler validates node_id before push | compile | `cargo check` | ❌ W0 | ⬜ pending |
| 70-01-02 | 01 | 1 | — | — | N/A | compile | `cargo check` | ❌ W0 | ⬜ pending |
| 70-02-01 | 02 | 2 | — | T-70-02 | RwLock guard not held across async connect | compile | `cargo check` | ❌ W0 | ⬜ pending |
| 70-02-02 | 02 | 2 | — | — | N/A | compile | `cargo check` | ❌ W0 | ⬜ pending |
| 70-03-01 | 03 | 2 | — | — | N/A | compile | `cargo check` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing test infrastructure covers all phase requirements. The Rust compiler provides type-level verification for serde serialization mismatches between backend and agent. No new test framework or stubs needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| RelayConfigSync push at register | D-05/D-06/D-07 | Requires running backend + agent WS handshake | Start backend, connect agent with AGENT_API_KEY, observe WS trace for `relay_config_sync` message |
| Hot-update on new server create | D-08 | Requires live server lifecycle | Create server via API while agent connected, verify new tunnel starts |
| Bootstrap skip when no AGENT_RELAY_TOKEN | D-03 | Requires agent startup with env var absent | Start agent with only AGENT_API_KEY, verify relay_client does not error at bootstrap |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
