---
phase: 71-buat-agar-plan-hobby-dan-pro-yang-ada-di-landing-page-bisa-b
plan: 01
subsystem: billing, ui
tags: billing, pricing, checkout, lemon-squeezy, react, typescript

requires: []
provides:
  - API-driven pricing section with live plan data from backend
  - Auth-gated checkout flow with auto-checkout after sign-in
  - Monthly/yearly billing toggle with savings calculation
  - Current plan badge for logged-in users
  - Subscription management via Lemon Squeezy Customer Portal
affects: []

tech-stack:
  added: []
  patterns:
    - billingApi module following existing auth.ts API module pattern
    - API response unwrapping with response.data?.data ?? response.data fallback
    - Auth gate checkout flow with sessionStorage for OAuth resilience

key-files:
  created:
    - landing-page-escluse/src/lib/api/billing.ts — billingApi module with fetchPlans, createCheckout, getSubscription, createPortal
    - landing-page-escluse/src/components/pricing/BillingToggle.tsx — Monthly/yearly toggle with calculateSavings helper
    - landing-page-escluse/src/components/pricing/PlanCard.tsx — Reusable plan card component with features, limits, auth-aware buttons
    - landing-page-escluse/src/components/pricing/PricingSection.tsx — API-driven pricing section with skeleton loading, error fallback, subscription awareness
  modified:
    - landing-page-escluse/src/App.tsx — Import PricingSection, remove inline Pricing component
    - landing-page-escluse/src/pages/SignIn.tsx — Add plan context reading and auto-checkout after login
    - landing-page-escluse/src/pages/oauth/OAuthCallback.tsx — Add intended_plan sessionStorage handling for OAuth resilience

key-decisions:
  - "CurrentPlanBadge logic integrated into PricingSection (not a separate component)"
  - "Skeleton card placeholders used for loading state (UI-SPEC override of D-02)"
  - "Fallback hardcoded plans rendered on API failure with error text"
  - "SessionStorage used for plan context to survive OAuth redirect flow"

requirements-completed: [REQ-01, REQ-02, REQ-03, REQ-04, REQ-07, REQ-08, REQ-09, REQ-10]

duration: 7 min
completed: 2026-06-11
---

# Phase 71 Plan 01: Hobby/Pro Plan Checkout Integration — Summary

**API-driven landing page pricing section with auth-gated checkout flows, monthly/yearly billing toggle, and current plan badge for logged-in users**

## Performance

- **Duration:** 7 min
- **Started:** 2026-06-11T01:04:37Z
- **Completed:** 2026-06-11T01:11:58Z
- **Tasks:** 3
- **Files modified/created:** 7

## Accomplishments

- Created `billingApi` module (`fetchPlans`, `createCheckout`, `getSubscription`, `createPortal`) following existing `auth.ts` pattern with API response unwrapping
- Created `BillingToggle` component with Monthly/Yearly toggle and `calculateSavings` helper for yearly discount display
- Created `PlanCard` component with features list, limits display, plan-specific emoji, "Most Popular" badge, yearly savings badge, and auth-aware button logic (Current Plan + Manage buttons for subscribers)
- Created `PricingSection` component with fetch-on-mount API loading, skeleton card placeholders, error fallback with hardcoded plans, billing toggle, subscription-aware current plan badge, and auth-gated checkout handler
- Modified `App.tsx` to replace inline hardcoded `<Pricing />` with `<PricingSection />` import
- Modified `SignIn.tsx` to read plan context from `location.state`, URL params, and `sessionStorage`, with auto-checkout after successful login
- Modified `OAuthCallback.tsx` to detect `intended_plan` from `sessionStorage` and auto-trigger checkout after OAuth completion

## Task Commits

Each task was committed atomically to the landing-page-escluse sub-repo:

1. **Task 1: Create billing API module** — `aca2047` (feat)
2. **Task 2: Create pricing section components** — `32c584d` (feat)
3. **Task 3: Wire pricing section into App, SignIn, and OAuth** — `e06ee11` (feat)

## Files Created/Modified

- `src/lib/api/billing.ts` — New billing API module (51 lines)
- `src/components/pricing/BillingToggle.tsx` — Monthly/yearly toggle (39 lines)
- `src/components/pricing/PlanCard.tsx` — Reusable plan card (137 lines)
- `src/components/pricing/PricingSection.tsx` — API-driven pricing section (233 lines)
- `src/App.tsx` — Modified (+1 import, -154 lines inline Pricing, replacement)
- `src/pages/SignIn.tsx` — Modified (+28 lines, plan context + auto-checkout)
- `src/pages/oauth/OAuthCallback.tsx` — Modified (+18 lines, intended_plan handling)

## Decisions Made

- **CurrentPlanBadge integrated into PricingSection:** The plan's own instruction (line 305) revised the approach to not create a separate CurrentPlanBadge.tsx file. Current plan detection and display logic lives in PricingSection.
- **Skeleton card loading state:** Per UI-SPEC override of CONTEXT.md D-02, 3 skeleton card placeholders are shown during API loading instead of a centered spinner.
- **Fallback plans on API error:** Hardcoded plan data (same as the previous inline Pricing component) is rendered when the plans API fails, with a red error text above the cards.
- **SessionStorage for OAuth resilience:** Plan context is saved to `sessionStorage` before navigating to `/signin`, so OAuth callback can read it after the OAuth redirect (which would lose URL params).
- **Free plan stays informational:** No subscription record created for Free plan. "Get Started Free" button redirects to `/signin` without any plan context.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- **TypeScript strict type check with API unwrapping:** The `response.data?.data ?? response.data` pattern creates a union type incompatible with the strict return type. Fixed by adding `as CheckoutResponse` type assertion, consistent with how consumers (PricingSection, SignIn) already access the response via `(response as any)?.checkout_url`.
- **React 19 `key` prop handling:** React 19 no longer strips `key` from JSX props automatically. PlanCard JSX with `key={plan.id}` caused a TypeScript error. Fixed by wrapping PlanCard in a `<div key={plan.id}>` container.

## Verification Results

### Type Check
```
npx tsc --noEmit --project landing-page-escluse/tsconfig.json
```
✅ Passes — zero errors (pre-existing `oauth.ts: skip_http_redirect` type issue excluded per scope boundary)

### Acceptance Criteria Verification

| Criterion | Status | Method |
|-----------|--------|--------|
| billing.ts exists ≥ 50 lines | ✅ Pass | 51 lines, all 4 exports verified |
| BillingToggle.tsx exports toggle + calculateSavings | ✅ Pass | grep confirmed |
| PlanCard.tsx exports PlanCardProps + uses CheckCircle2 | ✅ Pass | 2 CheckCircle2 occurrences |
| PricingSection.tsx uses fetchPlans, handleSubscribe, createPortal | ✅ Pass | All 3 confirmed via grep |
| PricingSection saves intended_plan to sessionStorage | ✅ Pass | sessionStorage.setItem confirmed |
| App.tsx imports PricingSection, no old Pricing component | ✅ Pass | Import found, old component count = 0 |
| SignIn.tsx uses billingApi.createCheckout | ✅ Pass | grep confirmed |
| SignIn.tsx uses useSearchParams/useLocation | ✅ Pass | Both imported |
| OAuthCallback.tsx reads intended_plan + calls createCheckout | ✅ Pass | Both confirmed via grep |

## Known Stubs

No stubs found — all plan cards are fully wired with live API data. The fallback plans on API error are intentional per D-04.

## Next Phase Readiness

- Plan 1 complete — pricing section is fully API-driven with checkout flows
- Ready for Plan 71-02 (if any remaining tasks) or the next phase

---

*Phase: 71-buat-agar-plan-hobby-dan-pro-yang-ada-di-landing-page-bisa-b*
*Completed: 2026-06-11*
