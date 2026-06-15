---
phase: 76
slug: update-ui-https-app-esluce-com-nodes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 76 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None — UI-only phase |
| **Config file** | Not found |
| **Quick run command** | `npm run build` |
| **Full suite command** | `npm run build` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm run build`
- **After every plan wave:** Run `npm run build`
- **Before `/gsd-verify-work`:** Build must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 76-01-01 | 01 | 1 | D-02, D-03 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 76-01-02 | 01 | 1 | D-04, D-05 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 76-01-03 | 01 | 1 | D-01 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 76-02-01 | 02 | 2 | D-09 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 76-02-02 | 02 | 2 | D-07, D-08 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- No test infrastructure exists in the project
- All phase requirements are manual-only verification (visual inspection + browser testing)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| View toggle switches between card and table | D-02 | Visual UI behavior | Navigate to /nodes, click LayoutGrid/List buttons, observe card/table switch |
| View preference persisted | D-03 | localStorage | Switch to table view, refresh page, verify table view persists |
| Node cards show uptime + last seen | D-04 | Visual UI behavior | Observe node cards show uptime and last seen fields correctly |
| Split-panel layout renders correctly | D-01 | Visual UI behavior | Verify left panel (list) and right panel (detail) display correctly |
| Health metrics visual refresh | D-09 | Visual UI behavior | Click a node, observe Overview tab health metrics with progress bars/color coding |
| Detail panel tabs preserved | D-07, D-08 | Visual UI behavior | Verify Overview, API Keys, Tokens tabs exist and function |

---

## Validation Sign-Off

- [ ] All tasks have build verification or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without build verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
