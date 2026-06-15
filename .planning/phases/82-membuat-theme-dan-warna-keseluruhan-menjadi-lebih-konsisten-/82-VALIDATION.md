---
phase: 82
slug: membuat-theme-dan-warna-keseluruhan-menjadi-lebih-konsisten-
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-15
---

# Phase 82 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None — CSS/theme phase, no testable logic |
| **Config file** | N/A |
| **Quick run command** | `npm run build` (verify no build errors after CSS changes) |
| **Full suite command** | `npm run build` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every plan:** `npm run build` (verify CSS variables and class names compile)
- **After every plan:** manual visual check in browser (switch light/dark, verify no broken styles)

---

## Per-Task Verification Map

| Task ID | Plan | Verification Method | Status |
|---------|------|---------------------|--------|
| 82-01-xx | 01 | grep for new CSS variables in index.css | ⬜ pending |
| 82-02-xx | 02 | grep — no remaining hardcoded structural colors | ⬜ pending |
| 82-03-xx | 03 | grep — no remaining hardcoded structural colors | ⬜ pending |

*Status: ⬜ pending · ✅ verified*

---

## Manual-Only Verifications

| Behavior | Why Manual | Test Instructions |
|----------|------------|-------------------|
| Light theme looks correct | Visual/CSS — no automated visual regression | Toggle to light mode, verify: bg colors, text contrast, glass panels, no stars/glows |
| Dark theme unchanged | Visual/CSS | Verify existing dark theme still works correctly after CSS changes |
| Toggle transition smooth | Subjective/visual | Toggle between themes, verify smooth 400ms transition |
| System preference detected | Needs browser API | Clear localStorage, set OS to light mode, verify page loads in light theme |
| No flash on page load | Visual | Hard refresh page, verify no theme flash before React mounts |

---

## Validation Sign-Off

- [ ] All CSS variables from UI-SPEC exist in index.css
- [ ] Build passes without errors
- [ ] Light theme renders correctly (manual check)
- [ ] No flash on page load (manual check)
- [ ] Toggle works correctly (manual check)

**Approval:** pending
