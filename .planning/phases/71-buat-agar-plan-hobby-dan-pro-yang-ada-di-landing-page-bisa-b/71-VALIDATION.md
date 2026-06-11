---
phase: 71
slug: buat-agar-plan-hobby-dan-pro-yang-ada-di-landing-page-bisa-b
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-11
---

# Phase 71 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None detected |
| **Config file** | Not found |
| **Quick run command** | `npx tsc --noEmit` (landing page type-check) |
| **Full suite command** | `npm run build` (dashboard app build) |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx tsc --noEmit`
- **After every plan wave:** Run `npm run build` (dashboard app)
- **Before `/gsd-verify-work`:** Type-check must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | 01 | 1 | N/A | T-71-01 | Lemon Squeezy API key not exposed to frontend | manual | `npx tsc --noEmit` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Landing page type-check passes: `npx tsc --noEmit` should return 0 errors
- [ ] Dashboard app build passes: `npm run build`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Plans fetch from API on mount | N/A | No test framework | Load landing page, check network tab for `GET /api/v1/billing/plans` |
| Monthly/yearly toggle updates display | N/A | No test framework | Click toggle and verify price shows `/mo` vs `/yr` with savings |
| Auth gate redirect to sign-in | N/A | No test framework | Click "Start for Hobby" while logged out, verify redirect to `/signin?plan=hobby&plan_cycle=monthly` |
| Auto-checkout after login | N/A | No test framework | Log in with plan params, confirm redirect to Lemon Squeezy |
| Welcome modal on dashboard | N/A | No test framework | Navigate to `/dashboard?checkout=success` and verify modal appears |
| Current plan badge | N/A | No test framework | Log in with subscription, check landing page for "Current Plan" badge |
| Error fallback for plans API | N/A | No test framework | Block `/api/v1/billing/plans`, verify hardcoded defaults render with error toast |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
