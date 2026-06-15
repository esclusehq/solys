---
phase: 72
slug: menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-12
---

# Phase 72 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust backend) |
| **Config file** | api/Cargo.toml |
| **Quick run command** | `cargo test -p api` |
| **Full suite command** | `cargo test` + `npm --prefix app run build` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p api`
- **After every plan wave:** Run `cargo test` + `npm --prefix app run build`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 72-01-01 | 01 | 1 | REQ-05 | — | N/A — migration | migration | `cargo test` (sqlx migration check) | ❌ W0 | ⬜ pending |
| 72-01-02 | 01 | 1 | REQ-02 | — | Image selection follows mc_loader | integration | `cargo test --test create_server_bedrock` | ❌ W0 | ⬜ pending |
| 72-01-03 | 01 | 1 | REQ-04 | — | Correct port reported | code review | N/A — manual | ❌ Manual | ⬜ pending |
| 72-02-01 | 02 | 2 | REQ-03 | — | UDP binding for Bedrock | integration | `cargo test --test agent_udp_binding` | ❌ W0 | ⬜ pending |
| 72-03-01 | 03 | 2 | REQ-01 | — | Bedrock option in modal | manual | N/A — visual | ❌ Manual | ⬜ pending |
| 72-03-02 | 03 | 2 | REQ-07 | — | Bedrock-specific fields shown | manual | N/A — visual | ❌ Manual | ⬜ pending |
| 72-04-01 | 04 | 3 | REQ-06 | — | Connectivity probe works | existing | Already in connectivity_service test | ✅ | ⬜ pending |
| 72-04-02 | 04 | 3 | REQ-08 | — | Relay handling review | manual | Code review | ❌ Manual | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `api/tests/create_server_bedrock.rs` — integration test for bedrock server creation
- [ ] `api/tests/agent_udp_binding.rs` — integration test for UDP port binding
- [ ] Migration test sqlx check for bedrock game_types row

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Frontend shows Bedrock option | REQ-01 | Visual UI inspection | Open CreateServerModal, verify Bedrock appears in game type selector |
| Bedrock-specific form fields | REQ-07 | Visual UI inspection | Select Bedrock, verify only relevant fields shown (no RCON, no TYPE) |
| Agent reports correct ports | REQ-04 | Code review | Verify agent_connection.rs port map key uses game_port not "25565" |
| Relay handles Bedrock | REQ-08 | Code review | Verify yamux/WS relay limitation documented for UDP |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** {pending / approved YYYY-MM-DD}
