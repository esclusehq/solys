---
phase: 77
slug: update-ui-https-app-esluce-com-templates
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-14
---

# Phase 77 — Validation Strategy

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
| 77-01-T1 | 77-01 | 0 | D-02 | T-77-05..T-77-08 | Read-only field addition | grep + cargo check | `grep -c "version: Option<String>" ... && grep -c "usage_count: i64" ...` | in PLAN.md | ⬜ pending |
| 77-01-T2 | 77-01 | 0 | D-02 | T-77-05..T-77-08 | DTO mapping | grep + cargo check | `grep -c "version: t.version" ... && grep -c "usage_count: 0" ...` | in PLAN.md | ⬜ pending |
| 77-01-T3 | 77-01 | 0 | D-02 | T-77-05..T-77-08 | SQL LEFT JOIN + COALESCE | grep + cargo check | `node -e "..." (8 checks) + cargo check` | in PLAN.md | ⬜ pending |
| 77-02-T1 | 77-02 | 1 | D-01, D-03, D-04 | T-77-09..T-77-12 | Client-side filtering only | grep + build | `node -e "..." (26 checks) + npm run build` | in PLAN.md | ⬜ pending |
| 77-02-T2 | 77-02 | 1 | D-02 | T-77-09..T-77-12 | Read-only data display | grep + build | `node -e "..." (8 checks) + npm run build` | in PLAN.md | ⬜ pending |
| 77-03-T1 | 77-03 | 2 | D-05, D-06, D-07 | T-77-13 | Pure styling — no new attack surface | grep + build | `node -e "..." (8 checks) + npm run build` | in PLAN.md | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Backend DTO fields: add `version`, `usage_count` to `TemplateResponse` in Rust backend (`api/src/` and `migration/src/`)
- [ ] Backend SQL: add `version` column to templates table (if not present), add usage_count query logic (COUNT of servers using template_id)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Filter tabs switch views | D-01 | No E2E test framework | Click Featured/Yours/All tabs → verify correct templates shown |
| Sort changes card order | D-03 | No E2E test framework | Select different sort → verify card order changes |
| Form styling appears correct | D-06 | Visual | Navigate to create/edit → verify cosmic theme consistency |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
