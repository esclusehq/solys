---
phase: 79
slug: update-ui-https-app-esluce-com-billing
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-15
---

# Phase 79 — Validation Strategy

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
- **After every plan wave:** Run `cargo build` from `api/`
- **Before `/gsd-verify-work`:** Both builds must pass
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 79-01-T1 | 79-01 | 1 | D-03 | T-79-01..T-79-03 | AuthUser middleware guards route | cargo build | `cd api && cargo build 2>&1 \| tail -5` | in PLAN.md | ⬜ pending |
| 79-02-T1 | 79-02 | 1 | D-10 | T-79-F-01..T-79-F-03 | API client uses Bearer token | grep + build | `grep -c "usageApi" app/src/lib/api.js && npm run build --prefix app 2>&1 \| tail -5` | in PLAN.md | ⬜ pending |
| 79-02-T2 | 79-02 | 1 | D-01..D-10 | T-79-F-01..T-79-F-03 | Read-only data display, React escaping | grep + build | `grep -c "glass-panel" app/src/pages/billing/BillingPage.jsx && grep -c "getBarColor" app/src/pages/billing/BillingPage.jsx && npm run build --prefix app 2>&1 \| tail -5` | in PLAN.md | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Backend route: `.nest("/api/v1/usage", UsageHandlers::router(state.clone()))` added to `api_routes.rs`
- [x] Frontend API: `usageApi` export with `getQuotas()` method in `api.js`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Usage bars show correct colors | D-03 | No E2E test framework | Navigate to /billing → verify green/yellow/red bars match usage thresholds |
| Plan cards have hover glow | D-04 | Visual | Hover over plan cards → verify cyan border glow appears |
| Section order matches spec | D-01 | Layout | Scroll through page → verify order: Subscription → Plan Cards → Payment History → Refund |
| Cancel subscription dialog | D-05 | Requires window.confirm | Click "Berhenti Berlangganan" → verify confirm dialog appears |
| Refund eligibility emoji | D-08 | Visual | Verify 🟢🟡🔴 emoji indicators render correctly |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
