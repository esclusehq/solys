---
phase: 78
slug: update-ui-https-app-esluce-com-mods
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 78 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None detected (no Jest/Vitest config in `app/`, no test dep in `app/package.json`) |
| **Config file** | None |
| **Quick run command** | `npm run build` from `app/` (frontend); `cargo build` from `api/` (backend) |
| **Full suite command** | `npm run build` from `app/` (frontend) |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm run build` from `app/`
- **After every plan wave:** Run `npm run build` from `app/` + `cargo build` from `api/`
- **Before `/gsd-verify-work`:** Both builds must pass
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 78-01-T1 | 01 | 1 | D-01, D-02 | T-78-01 / — | N/A — read-only data display | grep-check | `node -e "..."` (grep-based) | in PLAN.md | ⬜ pending |
| 78-01-T2 | 01 | 1 | D-08, D-09, D-10, D-06, D-07 | T-78-02, T-78-03 | N/A — read-only proxy | grep-check | `node -e "..."` (grep-based) | in PLAN.md | ⬜ pending |
| 78-02-T1 | 02 | 2 | D-05 | T-78-04 | Server ownership validated by backend | grep-check | `grep -n "install:" app/src/api/templatesApi.js` | in PLAN.md | ⬜ pending |
| 78-02-T2 | 02 | 2 | D-03, D-04, D-07 | T-78-05, T-78-06 | Double-submit prevented (installing state) | grep-check | `node -e "..."` (grep-based) | in PLAN.md | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Backend DTO fields: add `author`, `latest_version` to `ModrinthProject` / `PluginSearchResult`; add `date_published` to `PluginVersionDto`
- [ ] Backend endpoint: add `GET /api/v1/plugins/game-versions` handler + route
- [ ] Fix category param name: `params.category` → `params.project_type` in `executeSearch`

*If backend work is treated as Plan 03 instead of Wave 0: "Backend prereqs handled by Plan 03."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Category filter filters results | D-08 | No E2E test framework | Open browser → type query → select category → verify results change |
| Version modal opens + displays | D-06 | No E2E test framework | Open browser → click Versions on card → verify modal shows version rows |
| Add-to-Server install flow completes | D-03, D-05 | Requires running server | Open browser → click Add → select version → select server → click Install → verify toast |

*All phase behaviors require manual verification until an E2E framework is added.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
