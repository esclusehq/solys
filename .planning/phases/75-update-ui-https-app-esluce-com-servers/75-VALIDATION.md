---
phase: 75
slug: update-ui-https-app-esluce-com-servers
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 75 — Validation Strategy

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
| 75-01-01 | 01 | 1 | D-02, D-03 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 75-01-02 | 01 | 1 | D-04, D-05, D-06 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 75-01-03 | 01 | 1 | D-01 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 75-02-01 | 02 | 2 | D-07, D-08 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |
| 75-02-02 | 02 | 2 | D-09, D-10 | — | N/A | manual-only | — | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- No test infrastructure exists in the project
- All phase requirements are manual-only verification (visual inspection + browser testing)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| View toggle switches between card and table | D-02 | Visual UI behavior — no frontend test framework | Navigate to /servers, click LayoutGrid/List buttons, observe card/table switch |
| View preference persisted | D-03 | localStorage — no test framework | Switch to table view, refresh page, verify table view persists |
| Sort by name/status/activity | D-04 | Visual UI behavior | Click sort dropdown, select each option, verify server order changes |
| Game type filter | D-05 | Visual UI behavior | Click game type dropdown, select each game, verify filtered results |
| Filter/sort persistence | D-06 | localStorage | Set filter+sort, refresh page, verify preferences restored |
| Restart button with confirmation | D-07, D-08 | Visual UI + API call | Click restart on a server, verify modal appears, confirm, verify toast |
| 30s polling | D-09 | Time-dependent behavior | Open network tab, wait 30s, verify fetchServers() call appears |
| Status change toast | D-10 | Event-driven visual | Stop a running server manually, wait for next poll, verify toast appears |

---

## Validation Sign-Off

- [ ] All tasks have build verification or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without build verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
