---
phase: 71-buat-agar-plan-hobby-dan-pro-yang-ada-di-landing-page-bisa-b
reviewed: 2026-06-11T08:20:00Z
depth: deep
files_reviewed: 10
files_reviewed_list:
  - landing-page-escluse/src/lib/api/billing.ts
  - landing-page-escluse/src/components/pricing/BillingToggle.tsx
  - landing-page-escluse/src/components/pricing/PlanCard.tsx
  - landing-page-escluse/src/components/pricing/PricingSection.tsx
  - landing-page-escluse/src/App.tsx
  - landing-page-escluse/src/pages/SignIn.tsx
  - landing-page-escluse/src/pages/oauth/OAuthCallback.tsx
  - app/src/pages/dashboard/WelcomeModal.jsx
  - app/src/lib/api.js
  - app/src/pages/dashboard/DashboardPage.jsx
findings:
  critical: 0
  warning: 9
  info: 6
  total: 15
status: issues_found
---

# Phase 71: Code Review Report — Hobby/Pro Plan Checkout & Welcome Modal

**Reviewed:** 2026-06-11T08:20:00Z
**Depth:** deep (cross-file call-chain tracing)
**Files Reviewed:** 10
**Status:** issues_found

## Summary

10 files across two sub-repos (landing-page-escluse + app) were reviewed at deep depth, tracing API call chains, auth flows, React hook dependencies, and URL param handling across module boundaries.

**Key concerns:** Multiple instances of `JSON.parse` on `sessionStorage` without try/catch that throw unhandled promise rejections due to fire-and-forget `onSuccess()` calling pattern; attacker-controllable URL search params passed directly to the billing checkout API without validation; `createCheckout` parameter naming contradicts actual caller values; several silent failure paths where checkout/portal errors disappear without user feedback; and violation of React hooks exhaustive-deps rules.

No blocker/critical issues found — all warnings are addressable without architectural changes.

---

## Warnings

### WR-01: Unvalidated `JSON.parse` on sessionStorage in SignIn.tsx — throws unhandled promise rejection

**File:** `landing-page-escluse/src/pages/SignIn.tsx:30-31`

**Issue:** `JSON.parse(planFromStorage)` can throw a runtime error if the `sessionStorage` `intended_plan` value is missing, corrupted, or contains invalid JSON. This error is **not** inside a try/catch block. Because `SignInForm.tsx` (line 34) calls `onSuccess?.()` fire-and-forget (without `await`), the resulting unhandled promise rejection:

1. Silently cancels auto-checkout — user never gets redirected to Lemon Squeezy
2. Prevents the fallback `navigate('/')` at line 48 from executing (thrown earlier in the function)
3. Leaves the user staring at the sign-in page with no error indication

This is a real risk: while `sessionStorage` is set by the app's own code, any runtime issue (storage quota exceeded, corrupted data from interrupted write, or cross-tab race) will trigger this crash path.

**Fix:** Wrap `JSON.parse` in try/catch or use a safe parsing helper:

```typescript
function safeParseJSON<T>(raw: string | null): T | null {
  if (!raw) return null;
  try { return JSON.parse(raw) as T; } catch { return null; }
}

// Then:
const planData = safeParseJSON<{plan: string; planCycle: string}>(planFromStorage);
const plan = location.state?.plan || searchParams.get('plan') || planData?.plan || null;
const planCycle = location.state?.planCycle || searchParams.get('plan_cycle') || planData?.planCycle || null;
```

---

### WR-02: Unvalidated `JSON.parse` on sessionStorage in OAuthCallback.tsx — same crash pattern

**File:** `landing-page-escluse/src/pages/oauth/OAuthCallback.tsx:53-55`

**Issue:** Identical pattern to WR-01. `JSON.parse(intendedPlanRaw)` at line 53 can throw if `sessionStorage` data is corrupted. If thrown, the catch block at line 62 catches nothing, and the user is redirected to `/` (line 71-73 timeout still fires) but the checkout is silently lost. No user-facing error.

**Fix:** Use the same `safeParseJSON` helper pattern as WR-01:

```typescript
const intendedPlanRaw = sessionStorage.getItem('intended_plan');
sessionStorage.removeItem('intended_plan');
if (intendedPlanRaw) {
  const planData = safeParseJSON<{plan: string; planCycle: string}>(intendedPlanRaw);
  if (planData) {
    try {
      const response = await billingApi.createCheckout(planData.plan, planData.planCycle);
      // ...
    } catch (err) {
      console.error('OAuth auto-checkout failed:', err);
      // Consider showing user feedback here
    }
  }
}
```

---

### WR-03: Attacker-controllable URL search params passed directly to billing checkout API

**File:** `landing-page-escluse/src/pages/SignIn.tsx:30-37`

**Issue:** The `plan` and `planCycle` values from `searchParams.get('plan')` and `searchParams.get('plan_cycle')` are passed directly to `billingApi.createCheckout(plan, planCycle)` with zero validation. A crafted URL like `/signin?plan=../../../evil&plan_cycle=malicious` sends arbitrary strings as `plan_id` in the checkout API request body. While the backend should validate, this is defense-in-depth violation — the frontend should at minimum sanity-check that `plan` matches an expected set of values (e.g., known plan names/slugs) before sending to the API.

**Fix:** Validate against known plan names before calling the API:

```typescript
const VALID_PLANS = ['free', 'hobby', 'pro'] as const;
const VALID_CYCLES = ['monthly', 'yearly'] as const;

const rawPlan = location.state?.plan || searchParams.get('plan') || planData?.plan;
const rawCycle = location.state?.planCycle || searchParams.get('plan_cycle') || planData?.planCycle;
const plan = VALID_PLANS.includes(rawPlan) ? rawPlan : null;
const planCycle = VALID_CYCLES.includes(rawCycle) ? rawCycle : null;
```

---

### WR-04: Silent auto-checkout failure in OAuthCallback.tsx — user redirected to `/` without feedback

**File:** `landing-page-escluse/src/pages/oauth/OAuthCallback.tsx:56-64`

**Issue:** When the auto-checkout call to `billingApi.createCheckout()` fails (catch block at line 62-64), the error is only logged via `console.error`. No toast, no error message to the user. The function falls through to line 67 (`const redirectTo = intendedDestination || '/'`) and the user is redirected to the landing page with no indication that their intended plan checkout failed.

**Fix:** Add user-visible feedback and a retry mechanism:

```typescript
} catch (err) {
  console.error('OAuth auto-checkout failed:', err);
  // Set error state for UI display
  setError('Auto-checkout failed. You can try again from the pricing page.');
  // Still navigate home but with context
}
```

---

### WR-05: Silent auto-checkout failure in SignIn.tsx — user redirected to `/` without feedback

**File:** `landing-page-escluse/src/pages/SignIn.tsx:42-46`

**Issue:** Same as WR-04. The catch block logs the error and navigates to `/`. The user experiences a redirect to the landing page with no explanation that their plan checkout could not be initiated.

**Fix:** Add a toast or error state, or redirect to pricing page with a parameter:

```typescript
} catch (err) {
  console.error('Auto-checkout failed:', err);
  navigate('/?checkout_error=true');
  return;
}
```

---

### WR-06: Missing useEffect dependency array entries in DashboardPage.jsx

**File:** `app/src/pages/dashboard/DashboardPage.jsx:18-21`

**Issue:** `loadServers` and `loadSubscription` are defined outside the `useEffect` callback but used inside it without being listed in the dependency array. React's `exhaustive-deps` lint rule would flag this. While neither function captures changing state (they reference no hooks), this pattern suppresses the lint rule and can mask real dependency bugs if the functions are later modified to reference external state.

```jsx
useEffect(() => {
  loadServers()
  loadSubscription()
}, [])  // loadServers and loadSubscription are used but not in deps
```

**Fix:** Either define the functions inside the effect, or include them in the dependency array:

```jsx
useEffect(() => {
  const loadServers = async () => { ... };
  const loadSubscription = async () => { ... };
  loadServers();
  loadSubscription();
}, []);
```

Or if they must be defined outside, use `useCallback`:

```jsx
const loadServers = useCallback(async () => { ... }, []);
const loadSubscription = useCallback(async () => { ... }, []);
// Then include in deps
useEffect(() => { loadServers(); loadSubscription(); }, [loadServers, loadSubscription]);
```

---

### WR-07: `createCheckout` parameter named `planId` but callers pass plan name/slug, not UUID

**Files:**
- `landing-page-escluse/src/lib/api/billing.ts:34` (parameter definition)
- `landing-page-escluse/src/components/pricing/PricingSection.tsx:126` (passes `plan.name`)
- `landing-page-escluse/src/pages/SignIn.tsx:36` (passes plan name from URL/storage)
- `landing-page-escluse/src/pages/oauth/OAuthCallback.tsx:56` (passes plan name from storage)

**Issue:** The function signature `createCheckout(planId: string, ...)` sends `{ plan_id: planId }` to the API. All three callers pass `plan.name` (the slug like "hobby" or "pro") or plan name from URL params — never `plan.id` (the UUID from the API response). The `Plan` interface clearly separates `id` (UUID) from `name` (slug). If the backend expects a UUID, every checkout will fail. If the backend accepts slugs, the parameter is misleadingly named and creates a fragile API contract that silently breaks if the backend changes to require IDs.

This is especially dangerous because `SignIn.tsx` and `OAuthCallback.tsx` pass plan names from URL params and `sessionStorage` — unvalidated strings that may not even be valid plan slugs.

**Fix:** Rename the parameter to `planSlug` or `planName` to accurately reflect what's being sent, and validate against known slugs before calling the API:

```typescript
// in billing.ts
createCheckout: async (planName: string, billingCycle: string = 'monthly'): Promise<CheckoutResponse> => {
  // ...
}
```

And validate in callers as described in WR-03.

---

### WR-08: Silent failure when `createPortal()` returns no URL in handleManage

**File:** `landing-page-escluse/src/components/pricing/PricingSection.tsx:136-146`

**Issue:** If `createPortal()` succeeds but returns a response without a `portal_url` (e.g., user has no active subscription), the function exits silently. The user clicked "Manage" expecting to be redirected to the Lemon Squeezy customer portal, but nothing happens and no error is shown.

```typescript
const handleManage = async () => {
  try {
    const response = await billingApi.createPortal();
    const portalUrl = (response as any)?.portal_url || response?.portal_url;
    if (portalUrl) {
      window.location.href = portalUrl;
    }
    // No else branch — silent failure
  } catch (err) {
    console.error('Portal error:', err);
  }
};
```

**Fix:** Add feedback when portal URL is missing:

```typescript
if (portalUrl) {
  window.location.href = portalUrl;
} else {
  console.error('No portal URL returned');
  // Consider showing error toast or redirecting to billing page
}
```

---

### WR-09: Silent failure when checkout URL is missing in handleSubscribe

**File:** `landing-page-escluse/src/components/pricing/PricingSection.tsx:128-130`

**Issue:** Same pattern as WR-08. If `billingApi.createCheckout()` returns a response without a `checkout_url`, the function exits silently. The user clicked "Subscribe" but nothing happens, no error feedback.

**Fix:** Log the full response and show user feedback when URL is missing:

```typescript
const checkoutUrl = (response as any)?.checkout_url || response?.checkout_url;
if (checkoutUrl) {
  window.location.href = checkoutUrl;
} else {
  console.error('No checkout URL in response:', response);
  // Surface error to user
}
```

---

## Info

### IN-01: Redundant `(response as any)?.checkout_url || response?.checkout_url` pattern across three files

**Files:**
- `landing-page-escluse/src/components/pricing/PricingSection.tsx:127,139`
- `landing-page-escluse/src/pages/SignIn.tsx:37`
- `landing-page-escluse/src/pages/oauth/OAuthCallback.tsx:57`

**Issue:** The `(response as any)?.checkout_url || response?.checkout_url` pattern appears identically in three files. The `as any` branch defeats TypeScript's type checking entirely, and the `response?.checkout_url` fallback is only reachable if the type assertion to `CheckoutResponse` was correct — making the `as any` branch dead code when types match, or a silent error sink when they don't. This pattern was acknowledged in the Summary as a workaround for the union-type unwrapping issue but propagated without consolidation.

**Suggestion:** Create a shared `unwrapCheckoutResponse` helper function, or fix the type system in `billing.ts` so callers receive properly typed responses:

```typescript
// In billing.ts
export async function createCheckout(...): Promise<CheckoutResponse> {
  const response = await apiClient.post<ApiResponse<CheckoutResponse>>('/billing/checkout', ...);
  const data = response.data?.data ?? response.data;
  if (!data || typeof data !== 'object' || !('checkout_url' in data)) {
    throw new Error('Invalid checkout response');
  }
  return data as CheckoutResponse;
}
```

---

### IN-02: Free plan button hardcodes `'monthly'` cycle regardless of billing toggle

**File:** `landing-page-escluse/src/components/pricing/PlanCard.tsx:114`

**Issue:** The Free plan's "Get Started Free" button always passes `'monthly'` as the billing cycle:
```tsx
onClick={() => onSubscribe(plan, 'monthly')}
```
This is harmless (the `handleSubscribe` handler in PricingSection navigates to `/signin` without using the cycle for Free plans), but it's misleading. A reader might expect the cycle to match the current toggle state.

**Suggestion:** Either pass the actual cycle from props for consistency, or document the intentional bypass with a comment.

---

### IN-03: Both US and UK spellings of "canceled"/"cancelled" handled in WelcomeModal

**File:** `app/src/pages/dashboard/WelcomeModal.jsx:37`

**Issue:** The param check `checkoutStatus === 'cancelled' || checkoutStatus === 'canceled'` handles both spellings, which indicates uncertainty about which variant the backend sends. This should be normalized to one canonical form.

**Suggestion:** Check which spelling the backend actually uses and drop the non-matching branch, or normalize in the backend response.

---

### IN-04: `getWelcomeMessage` returns "Welcome back" for users without `created_at` field

**File:** `app/src/pages/dashboard/DashboardPage.jsx:49`

**Issue:** The function first checks `if (!user?.created_at) return 'Welcome back, ...'` — returning "Welcome back" for users who are brand-new (and thus may not have a `created_at` field populated yet). The "Welcome back" phrasing is appropriate for returning users; new users should see "Welcome."

**Suggestion:** Default to "Welcome" when `created_at` is unavailable:

```jsx
if (!user?.created_at) return `Welcome, ${user?.name || 'User'}!`;
```

---

### IN-05: Mixed plan data sources without normalization layer in SignIn flow

**File:** `landing-page-escluse/src/pages/SignIn.tsx:30-31`

**Issue:** The plan and planCycle values are read from three different sources in priority order — `location.state`, `searchParams`, and `sessionStorage` — each with different shapes. `location.state` contains `{ plan, planCycle }`, `searchParams` has `plan, plan_cycle` (snake_case), and `sessionStorage` has `{ plan, planCycle }` (JSON). This mixing of camelCase and snake_case without normalization increases cognitive load and is fragile.

**Suggestion:** Normalize to a single interface early, or document the expected shape per source.

---

### IN-06: `handleSubscribe` reads auth state via `getState()` instead of hook value

**File:** `landing-page-escluse/src/components/pricing/PricingSection.tsx:119`

**Issue:** The handler reads `useAuthStore.getState().isAuthenticated` directly from the store rather than using the destructured `isAuthenticated` from the hook (line 80). While this is intentional (event handlers need fresh state, not the closure value), it creates an inconsistency where the render decision (`isLoading` toggle visibility, etc.) uses one auth value and the handler uses another. If auth state flips between render and click, behavior is inconsistent.

**Suggestion:** Either document this choice explicitly with a comment, or use the hook value consistently and accept the micro-latency trade-off.

---

## Cross-File Analysis Notes

**Call chain: Subscribe → SignIn → Checkout**
```
PricingSection.handleSubscribe(plan.name) 
  → if !auth → sessionStorage.setItem('intended_plan', {plan: plan.name})
  → navigate('/signin', {state: {plan, planCycle}})

SignIn.onSuccess()
  → location.state.plan || searchParams.get('plan') || JSON.parse(storage).plan
  → billingApi.createCheckout(plan, planCycle)   // plan is plan.name (slug), not plan.id

OAuthCallback
  → sessionStorage.getItem('intended_plan')
  → JSON.parse → .plan → billingApi.createCheckout(plan, planCycle)
```

**Key finding:** The `plan` value that flows through this chain is `plan.name` (slug like "hobby"/"pro"), never `plan.id` (UUID). If the backend expects UUIDs, the entire checkout flow silently fails. The `createCheckout` API body sends `plan_id: <slug>`, and three independent code paths all pass slugs with zero validation that the slug corresponds to a real plan.

---

_Reviewed: 2026-06-11T08:20:00Z_
_Reviewer: gsd-code-reviewer (deep analysis)_
_Depth: deep_
