---
phase: 74
slug: menambahkan-nama-akun-yang-login-ke-dashboard
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-14
---

# Phase 74 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None (frontend-only, no test suite detected) |
| **Config file** | N/A |
| **Quick run command** | `npm run dev` (manual browser testing) |
| **Full suite command** | `npm run build` (compile check) |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm run build` to verify no compilation errors
- **After every plan wave:** Full manual walkthrough of all 3 deliverables
- **Before `/gsd-verify-work`:** Full manual walkthrough
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 74-01-01 | 01 | 1 | D-01 | — | N/A | compile | `npm run build` | ❌ | ⬜ pending |
| 74-01-02 | 01 | 1 | D-02 | — | N/A | compile | `npm run build` | ❌ | ⬜ pending |
| 74-01-03 | 01 | 1 | D-03 | — | N/A | compile | `npm run build` | ❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements (no test framework needed).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TopBar renders with avatar + display_name | D-01 | Visual — no screenshot testing | Open app, verify avatar + name appear in top bar |
| Dropdown opens on click | D-01 | Interactive | Click avatar — Profile/Settings/Logout appear |
| Dropdown closes on click-outside | D-01 | Interactive | Open dropdown, click outside — dropdown closes |
| Dropdown navigation (Profile) | D-01 | Interactive | Click "Profile" — navigates to /settings |
| Dropdown navigation (Settings) | D-01 | Interactive | Click "Settings" — navigates to /settings |
| Logout calls authStore.logout() | D-01 | Side effect | Click "Logout" — user is logged out |
| TopBar not visible when not logged in | D-01 | Auth flow | Verify TopBar only shows for authenticated users |
| Welcome message shows display_name | D-02 | Visual | Dashboard page shows "Welcome, {display_name}" |
| Welcome fallback to email prefix | D-02 | Edge case | User with no display_name shows email prefix fallback |
| Welcome fallback to "User" | D-02 | Edge case | Null display_name and null email shows "User" |

---

## Validation Sign-Off

- [ ] All tasks have compile-check verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
