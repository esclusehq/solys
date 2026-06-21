# Quick Task 260617: Lock Hobby & Pro Subscribe buttons for non-owners

**Status:** complete
**Date:** 2026-06-17

## Task Results

### Task 1: App BillingPage.jsx
- **Modified:** `app/src/pages/billing/BillingPage.jsx`
- **Change:** Hobby now shows "Locked" badge + disabled subscribe button (same as Pro which shows "Coming Soon")
- **Build:** ✅ Passed

### Task 2: Landing page PlanCard.tsx
- **Modified:** `landing-page-escluse/src/components/pricing/PlanCard.tsx`
- **Change:** Hobby shows "Locked" badge (gray) + disabled button. Pro retains "Coming Soon" badge (orange) + disabled button.
- **Build:** ✅ Passed

## Summary
All non-free plan subscribe buttons are now locked. Hobby shows "Locked" (gray), Pro shows "Coming Soon" (orange). Both buttons are disabled with `disabled:opacity-40 disabled:cursor-not-allowed`. Users with active subscriptions still see "Current Plan" label on their active plan.
