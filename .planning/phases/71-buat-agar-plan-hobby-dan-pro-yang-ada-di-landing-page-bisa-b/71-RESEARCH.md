# Phase 71: Subscription Plans on Landing Page - Research

**Researched:** 2026-06-11
**Domain:** Frontend billing integration (React 19 / TypeScript), Lemon Squeezy checkout flow
**Confidence:** HIGH

## Summary

Phase 71 wires the landing page pricing section to the existing backend billing system so that Hobby and Pro plan buttons trigger real Lemon Squeezy checkout flows instead of just redirecting to `/signin`. The backend (Rust/Axum) already has all required endpoints — `GET /api/v1/billing/plans`, `POST /api/v1/billing/checkout`, `POST /api/v1/billing/portal`, `GET /api/v1/billing/subscription`. No backend changes are needed.

The work spans two separate frontend applications: the **landing page** (React 19 / TypeScript / Vite at `landing-page-escluse/`) and the **dashboard app** (React / JSX at `app/`). The landing page gets API-driven plan cards, billing toggle, auth-gated checkout flow, and current plan badge for logged-in users. The dashboard app gets a welcome modal for post-checkout success detection and a "Manage" button that opens the Lemon Squeezy Customer Portal.

**Key risk:** The sign-in flow in the landing page currently navigates to `/` on success with no URL param reading logic. The auth gate + auto-checkout flow requires modifying the sign-in page to read `?plan=hobby&plan_cycle=monthly` params after login and auto-trigger checkout. This touches existing auth and routing code.

**Primary recommendation:** Create a `billingApi` module in the landing page, refactor the `Pricing` component to be API-driven, add auth-redirect logic to the sign-in page, and wire the checkout flow using `window.location.href` to the Lemon Squeezy checkout URL returned by the backend.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Display plan cards from API data | Browser (landing page) | — | Pure UI rendering. Fetch on mount, render cards. |
| Monthly/yearly billing toggle | Browser (landing page) | — | Client-side toggle switches display price and checkout cycle param. |
| Auth gate (unauthenticated → signin) | Browser (landing page) | — | Check `isAuthenticated` from useAuthStore. Redirect to `/signin?plan=...`. |
| Create Lemon Squeezy checkout | API / Backend | Browser (landing page) | Backend creates checkout via Lemon Squeezy API. Frontend redirects to URL. |
| Welcome modal on dashboard | Browser (dashboard app) | — | Check `?checkout=success` URL param on dashboard mount, show modal. |
| Current plan badge on landing page | Browser (landing page) | API / Backend | Fetch `GET /api/v1/billing/subscription`, match plan name to card. |
| Subscription management | Lemon Squeezy (external) | API / Backend | Customer Portal hosted by Lemon Squeezy. Backend creates portal session URL. |
| Plan data source | API / Backend | Database | `GET /api/v1/billing/plans` reads from PostgreSQL `plans` table. |
| Post-checkout redirect | Browser (Lemon Squeezy → dashboard) | API / Backend | Backend sets `success_url = dashboard?checkout=success` in checkout creation. |

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01 (API source):** Fetch plans from `GET /api/v1/billing/plans` on Pricing component mount. All card content (display name, description, price, features, limits) comes from API response. Backend is source of truth.
- **D-02 (Loading state):** Block pricing section with centered spinner while plans API loads. No skeleton cards.
- **D-03 (Refresh frequency):** Fetch once per Pricing component mount. No stale-while-revalidate, no caching.
- **D-04 (Error fallback):** If API call fails, show error toast and render hardcoded defaults so the section never breaks.
- **D-05 (Unauthenticated flow):** Clicking a paid plan button while logged out redirects to `/signin` with `?plan=hobby&plan_cycle=monthly` params. After successful sign-in, automatically call checkout API and redirect to Lemon Squeezy.
- **D-06 (Authenticated flow):** Clicking a paid plan button while logged in calls `POST /api/v1/billing/checkout` with `plan_id` and `billing_cycle`. Backend returns Lemon Squeezy checkout URL. Frontend does `window.location.href = url`.
- **D-07 (Default billing cycle):** Monthly. Yearly is selected via the billing toggle.
- **D-08 (Checkout callback):** After Lemon Squeezy checkout completes, redirect to dashboard (`/` or app URL). Success/cancel status passed as URL params.
- **D-09 (Toggle visibility):** Show a "Monthly" / "Yearly" toggle above the plan cards.
- **D-10 (Price display):** Replace price text based on active toggle — show `/mo` when monthly, `/yr` when yearly, with yearly showing "Save ~X%" computed from API prices.
- **D-11 (Savings calculation):** Compute dynamically client-side: `(price_monthly * 12 - price_yearly) / (price_monthly * 12) * 100`.
- **D-12 (Welcome modal):** Show a welcome modal on dashboard after first successful subscription. Displays plan name, features unlocked, limits, and "Start creating servers" CTA.
- **D-13 (Current plan badge):** For logged-in users, fetch subscription status on landing page and show "Current plan" badge on the matching pricing card. "Manage" button opens Lemon Squeezy Customer Portal instead of checkout.
- **D-14 (Subscription management):** Use `POST /api/v1/billing/portal` to create Lemon Squeezy Customer Portal session for plan changes, cancellations, and billing history.
- **D-15 (Free plan behavior):** "Get Started Free" redirects to `/signin`. Sign-up creates a user account but does NOT create a subscription record. Free tier limits are enforced by the backend without a subscription entity. The Free plan card in the API is informational only.

### the agent's Discretion
- Exact spinner/loading component styling for pricing section
- Welcome modal design (content, CTA buttons, dismiss behavior)
- Toast messages for checkout success/cancel
- Plan badge positioning and styling on pricing cards
- Whether the pricing section uses a `usePlans` hook or inline fetch
- API client module naming for billing calls in landing page

### Deferred Ideas (OUT OF SCOPE)
- **Enterprise plan pricing card** — Plan exists in DB ($99.99/mo) but has no pricing card on landing page. Needs content design (features, limits, target audience). Belongs in its own phase or quick task.
- **In-app plan management UI** — Currently delegated to Lemon Squeezy Customer Portal. A custom upgrade/downgrade/cancel UI inside the dashboard is a future enhancement.
- **Free plan subscription record** — Not creating a subscription entity for Free users. If plan-based feature gating needs to change, this can be revisited.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-01 | Pricing section fetches live plans from API | Backend `GET /api/v1/billing/plans` exists [VERIFIED: code]. Create `billingApi` module in landing page. |
| REQ-02 | Monthly/yearly billing toggle on pricing cards | Client-side toggle state. Compute savings from API prices via `(price_monthly*12 - price_yearly) / (price_monthly*12) * 100`. |
| REQ-03 | Auth gate + auto-checkout for unauthenticated users | Sign-in page currently navigates to `/` on success. Must add URL param reading (`?plan=`, `?plan_cycle=`) and auto-checkout after login. |
| REQ-04 | Authenticated checkout via backend API | Backend `POST /api/v1/billing/checkout` accepts `{plan_id, billing_cycle}`, returns `{checkout_url}`. Frontend does `window.location.href = checkout_url`. |
| REQ-05 | Post-checkout redirect to dashboard with status | Backend sets `success_url = "{app_url}/dashboard?checkout=success"` and `cancel_url`. Live in `billing_handlers.rs:126-128`. [VERIFIED: code] |
| REQ-06 | Welcome modal on dashboard after first subscription | Dashboard app (`app/`) must detect `?checkout=success` on mount. Modal shows plan name, features unlocked, limits, CTA. |
| REQ-07 | Current plan badge for logged-in users | `GET /api/v1/billing/subscription` returns `{status, plan: {name, display_name, limits}}`. Match `plan.name` to card. [VERIFIED: code] |
| REQ-08 | Subscription management via Lemon Squeezy Customer Portal | Backend `POST /api/v1/billing/portal` returns `{portal_url}`. Frontend redirects. [VERIFIED: code] |
| REQ-09 | Free plan stays informational, no subscription record | D-15 explicitly states Free plan does NOT create subscription entity. Free button stays as redirect to `/signin`. |
| REQ-10 | Error fallback for plans API | If `GET /billing/plans` fails, render hardcoded defaults (current data in App.tsx:536-690) with error toast. |

## Standard Stack

### Core (No new libraries needed)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axios | ^1.7.0 | API calls in landing page | Already in project. Used for `withCredentials` cookie-based auth. |
| react-router-dom | ^7.14.0 | Navigation & URL params | Already in project. `useNavigate()` + `useSearchParams()` for auth gate flow. |
| zustand | ^5.0.0 | Auth state | Already in project as `useAuthStore`. Provides `isAuthenticated`, `user`, `fetchUser()` |

### No Additional Dependencies
Phase 71 uses only existing stack:
- `motion` for scroll animations (already imported in Pricing component)
- `lucide-react` for icons (already imported)
- `axios` for API calls (already configured with 401 refresh interceptor)
- `react-router-dom` for routing and URL params

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| axios (existing) | fetch directly | Axios already configured with `withCredentials`, base URL, 401 interceptor. No reason to change. |
| react-router-dom (existing) | window.location | Use `useSearchParams()` for reading `?plan=` params. Cleaner than parsing `window.location.search`. |

## Backend API Reference

### Billing Endpoints (Rust/Axum — all exist, no changes needed)

| Endpoint | Method | Auth | Request | Response |
|----------|--------|------|---------|----------|
| `/api/v1/billing/plans` | GET | None | — | `{success, data: [{id, name, display_name, description, price_monthly, price_yearly, limits, features}]}` |
| `/api/v1/billing/checkout` | POST | VerifiedUser | `{plan_id: string, billing_cycle?: "monthly"|"yearly"}` | `{success, data: {checkout_url: string}}` |
| `/api/v1/billing/portal` | POST | VerifiedUser | — | `{success, data: {portal_url: string}}` |
| `/api/v1/billing/subscription` | GET | VerifiedUser | — | `{success, data: {status: "active"|"none", plan: {id, name, display_name, limits}, current_period_end}}` |

**Source:** `billing_handlers.rs:31-45` [VERIFIED: code]

### Critical Backend Behavior

1. **Plan IDs accepted as UUID or name string** — `create_checkout` handler accepts both `plan_id = "hobby"` (name) or `plan_id = "c5ce2a82-..."` (UUID). [CITED: billing_handlers.rs:108-114]
2. **Default billing_cycle = "monthly"** — if `billing_cycle` is missing or not "yearly", defaults to monthly. [CITED: billing_handlers.rs:116-120]
3. **Checkout URLs** — Backend retrieves store ID dynamically via `GET /api/v1/stores` then creates checkout. The `user_id` is passed as `custom_data` in the checkout payload. [CITED: lemon_squeezy_service.rs:104-156]
4. **Redirect URLs (live in code)** — `success_url = "https://app.esluce.com/dashboard?checkout=success"`, `cancel_url = "https://app.esluce.com/dashboard?checkout=cancelled"`. [CITED: billing_handlers.rs:126-128]
5. **Variant → Plan mapping in webhooks** — Hardcoded mapping: variant 1490734 or 1741699 → Hobby (UUID c5ce2a82-...), variant 1517243 or 1741733 → Pro (UUID 7daa98c8-...). [CITED: billing_handlers.rs:275-288]

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Landing Page (React 19 / TypeScript / Vite)                           │
│                                                                         │
│  ┌──────────────┐   ┌──────────────────┐   ┌───────────────────────┐   │
│  │ Pricing      │   │ SignIn Page      │   │ Dashboard (separate   │   │
│  │ Component    │   │                  │   │ SPA at /dashboard)    │   │
│  │              │   │ Reads ?plan=     │   │                       │   │
│  │ Fetch plans ─┼──►│ ?plan_cycle=     │   │ ?checkout=success ────┼──►│
│  │ from API     │   │ params           │   │ → Welcome Modal       │   │
│  │              │   │                  │   │                       │   │
│  │ [CTA click] ─┼──►│ Login success ───┼──►│ [Manage] ─────────────┼──►│
│  │  ├─ auth?    │   │ auto-call        │   │ → Customer Portal     │   │
│  │  │  NO →     │   │ checkout API     │   │                       │   │
│  │  │  /signin  │   │ redirect LS      │   │                       │   │
│  │  │  YES →    │   └──────────────────┘   └───────────────────────┘   │
│  │  checkout    │                                                        │
│  │  API call    │                                                        │
│  └──────┬───────┘                                                        │
└─────────┼──────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  API / Backend (Rust / Axum)                                            │
│                                                                         │
│  GET /api/v1/billing/plans ─────────────────► PostgreSQL plans table    │
│  POST /api/v1/billing/checkout ─────────────► Lemon Squeezy Checkouts   │
│                                              API → returns checkout URL │
│  POST /api/v1/billing/subscription ─────────► PostgreSQL subscriptions  │
│  POST /api/v1/billing/portal ───────────────► Lemon Squeezy Customer    │
│                                                Portal API               │
│  POST /api/v1/billing/webhook ◄────────────── Lemon Squeezy Webhooks    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Checkout Flow State Machine

```
User clicks "Start for Hobby" / "Upgrade to Pro"
  ↓
Is user authenticated? (useAuthStore.isAuthenticated)
  ├─ No  → navigate('/signin', { state: { plan: 'hobby', planCycle: 'monthly' } })
  │        // Or use URL params: /signin?plan=hobby&plan_cycle=monthly
  │        // After successful login:
  │        //   1. Read plan/plan_cycle from location.state or URL params
  │        //   2. Call POST /api/v1/billing/checkout { plan_id, billing_cycle }
  │        //   3. window.location.href = checkout_url
  │
  └─ Yes → Call POST /api/v1/billing/checkout { plan_id, billing_cycle }
           window.location.href = checkout_url
  ↓
Lemon Squeezy hosted checkout (user enters payment info)
  ↓
Success → Lemon Squeezy redirects to:
          https://app.esluce.com/dashboard?checkout=success
          → Dashboard detects param → shows Welcome Modal
Cancel  → Lemon Squeezy redirects to:
          https://app.esluce.com/dashboard?checkout=canceled
          → Dashboard shows toast: "Checkout canceled"
```

### Recommended Project Structure Changes

```
landing-page-escluse/src/
├── lib/
│   ├── api/
│   │   ├── client.ts          # unchanged — existing Axios instance
│   │   ├── auth.ts            # unchanged — existing auth functions
│   │   └── billing.ts         # NEW — fetchPlans(), createCheckout(), createPortal(), getSubscription()
│   └── stores/
│       └── authStore.ts       # unchanged — existing auth store
├── components/
│   └── pricing/
│       ├── PricingSection.tsx  # NEW — extracted from App.tsx, API-driven
│       ├── PlanCard.tsx        # NEW — individual plan card component
│       ├── BillingToggle.tsx   # NEW — monthly/yearly toggle
│       └── CurrentPlanBadge.tsx # NEW — "Current Plan" badge for logged-in users
├── pages/
│   ├── SignIn.tsx              # MODIFY — handle ?plan=&plan_cycle= params
│   └── SignUp.tsx              # unchanged
└── App.tsx                     # MODIFY — replace Pricing inline component with PricingSection import

app/src/
└── pages/
    └── dashboard/
        └── WelcomeModal.jsx    # NEW — post-checkout welcome modal
```

### Pattern 1: API-driven Plan Rendering
**What:** Fetch plans from backend on component mount, render plan cards dynamically.
**When to use:** Pricing section always shows data from API.
**Source:** CONTEXT.md D-01 + D-03

```typescript
// landing-page-escluse/src/lib/api/billing.ts
// Pattern: existing API module pattern (see auth.ts)
import apiClient, { ApiResponse } from './client';

export interface Plan {
  id: string;
  name: string;
  display_name: string;
  description: string | null;
  price_monthly: number;
  price_yearly: number | null;
  limits: Record<string, unknown>;
  features: string[];
}

export interface CheckoutResponse {
  checkout_url: string;
}

export interface PortalResponse {
  portal_url: string;
}

export interface SubscriptionResponse {
  status: string;
  plan: { id: string; name: string; display_name: string; limits: Record<string, unknown> } | null;
  current_period_end?: string;
}

export const billingApi = {
  fetchPlans: async (): Promise<Plan[]> => {
    const response = await apiClient.get<ApiResponse<Plan[]>>('/billing/plans');
    return response.data?.data ?? [];
  },
  createCheckout: async (planId: string, billingCycle: string = 'monthly'): Promise<CheckoutResponse> => {
    const response = await apiClient.post<ApiResponse<CheckoutResponse>>('/billing/checkout', {
      plan_id: planId,
      billing_cycle: billingCycle,
    });
    return response.data?.data ?? response.data;
  },
  createPortal: async (): Promise<PortalResponse> => {
    const response = await apiClient.post<ApiResponse<PortalResponse>>('/billing/portal');
    return response.data?.data ?? response.data;
  },
  getSubscription: async (): Promise<SubscriptionResponse> => {
    const response = await apiClient.get<ApiResponse<SubscriptionResponse>>('/billing/subscription');
    return response.data?.data ?? response.data;
  },
};
```

### Pattern 2: Auth Gate + Redirect Flow
**What:** Unauthenticated users are redirected to sign-in with plan context, then auto-checkout after login.
**When to use:** A paid plan button is clicked and the user is not authenticated.

```typescript
// In PricingSection component:
const handleSubscribe = async (plan: Plan, cycle: 'monthly' | 'yearly') => {
  const { isAuthenticated } = useAuthStore.getState();
  
  if (!isAuthenticated) {
    // Redirect to sign-in with plan context via location.state
    navigate('/signin', {
      state: { plan: plan.name, planCycle: cycle }
    });
    return;
  }
  
  // Authenticated — call checkout API directly
  try {
    const { checkout_url } = await billingApi.createCheckout(plan.name, cycle);
    window.location.href = checkout_url;
  } catch (err) {
    // Show error toast
  }
};

// In SignIn component (after successful login):
const { state } = useLocation();
const plan = state?.plan || params.get('plan');
const planCycle = state?.planCycle || params.get('plan_cycle');

if (plan && planCycle && isAuthenticated) {
  const { checkout_url } = await billingApi.createCheckout(plan, planCycle);
  window.location.href = checkout_url;
}
```

### Pattern 3: Post-Checkout Welcome Modal (Dashboard App)
**What:** After Lemon Squeezy checkout redirects to `/dashboard?checkout=success`, show a welcome modal.
**When to use:** On dashboard mount when `?checkout=success` is detected.

```jsx
// app/src/pages/dashboard/WelcomeModal.jsx
// Pattern: existing modal patterns in app
import { useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';

export default function WelcomeModal() {
  const [searchParams] = useSearchParams();
  const [isOpen, setIsOpen] = useState(false);
  const checkoutStatus = searchParams.get('checkout');

  useEffect(() => {
    if (checkoutStatus === 'success') {
      setIsOpen(true);
      // Clean the URL without page reload
      window.history.replaceState({}, '', '/dashboard');
    }
  }, [checkoutStatus]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      {/* Modal content: plan name, features, limits, "Start creating servers" CTA */}
    </div>
  );
}
```

### Pattern 4: Savings Calculation
**What:** Compute yearly savings dynamically from API prices.
**Source:** CONTEXT.md D-11

```typescript
const calculateSavings = (monthly: number, yearly: number | null): number | null => {
  if (!yearly || monthly <= 0) return null;
  const yearlyIfMonthly = monthly * 12;
  const savings = ((yearlyIfMonthly - yearly) / yearlyIfMonthly) * 100;
  return Math.round(savings);
};

// Usage: {typeof savings === 'number' && `Save ${savings}%`}
```

### Anti-Patterns to Avoid
- **Hardcoding plan data after API migration:** The whole point is API-driven. Don't keep hardcoded fallback for "simplicity" — only use fallback on API failure.
- **Inline checkout URL opening in new tab:** D-06 specifies `window.location.href = url`, not `window.open(url)`.
- **Creating Free plan subscription record:** D-15 explicitly says Free plan should NOT create a subscription entity. Don't accidentally wire it.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Subscription management UI | Custom upgrade/downgrade/cancel page | Lemon Squeezy Customer Portal (via `POST /api/v1/billing/portal`) | Modal dialog for auth-free auto-login. Complex to build custom UI with proration, payment method changes, cancellation flows. |
| Lemon Squeezy checkout iframe | In-page checkout overlay | Hosted checkout URL redirect | Browser compatibility, security (CSP issues), UX complexity. Redirect is simpler and more reliable. |
| Caching/stale-while-revalidate for plan data | Custom SWR/caching layer | Fetch-on-mount (no cache) | D-03 specifies fetch-on-mount. Plan data changes infrequently. No cache complexity needed. |
| Custom sign-in auth state sync | Manual param passing | `location.state` or URL search params | React Router's `location.state` is purpose-built for passing data between routes. Use URL params for shareable/bookmarkable state. |

**Key insight:** This phase is a **frontend wiring exercise** — the backend has all the infrastructure. The danger is over-engineering the auth gate flow or introducing new state management patterns. Stick to existing patterns (axios, zustand, react-router).

## Common Pitfalls

### Pitfall 1: Auth Gate Timing — Logging in and auto-checkout race condition
**What goes wrong:** After successful login, the `isAuthenticated` state may not yet be set when the automatic checkout call fires, causing a 401 error.
**Why it happens:** Zustand state updates are async React renders. The login API succeeds, but `isAuthenticated` is still `false` if checked synchronously after `login()`.
**How to avoid:** After `login()` resolves, call `useAuthStore.getState().fetchUser()` and verify `isAuthenticated` is `true` before calling the checkout API. Or use the `login()` function's own `onSuccess` callback pattern.
**Warning signs:** 401 errors after login during the auto-checkout flow.

### Pitfall 2: URL Param Confusion — Using `location.state` vs URL search params
**What goes wrong:** The plan context (`?plan=hobby&plan_cycle=monthly`) gets sent via URL params to `/signin`, but after OAuth flow (Supabase redirects to `/oauth/callback` first, then to `/signin` or `/`), the params are lost.
**Why it happens:** OAuth redirect flow doesn't preserve URL params. The callback page uses `intendedDestination` from `sessionStorage`.
**How to avoid:** Save the plan context to `sessionStorage` before navigating to `/signin`, then read it in the callback flow or after sign-in. For regular email/password login, `location.state` with `useNavigate` is reliable.
**Warning signs:** Users subscribe for Hobby but get redirected without plan context.

### Pitfall 3: Welcome Modal Fires on Every Dashboard Visit
**What goes wrong:** The welcome modal appears every time the user visits `/dashboard` because `?checkout=success` persists in browser history.
**Why it happens:** Lemon Squeezy redirect adds the param to the URL. If the user bookmarks or revisits the dashboard URL (even without the param saved in a bookmark), the browser may still have it in history state.
**How to avoid:** Use `window.history.replaceState()` to clean the URL params immediately after detecting them. Or check that the current URL actually has the param (not just browser state).
**Warning signs:** Users see "Welcome to Hobby!" modal on every login.

### Pitfall 4: Plans API response format mismatch
**What goes wrong:** The landing page expects `ApiResponse<T>` wrapper (`{success, data: [...]}`) but the actual API returns data in a different shape.
**Why it happens:** The backend uses `ApiResponse::new(plans_response)` which wraps in `{success: true, data: [...]}`. But the existing dashboard's `BillingPage.jsx` handles both wrapped and unwrapped formats (`plansData?.data || plansData`). The landing page must handle the same.
**How to avoid:** Test the API response shape. The landing page `apiClient.get<ApiResponse<Plan[]>>` should unwrap `response.data.data` or `response.data` depending on the actual response structure.
**Warning signs:** Pricing section renders empty or shows "undefined" values.

## Code Examples

### Existing Patterns to Follow

#### API client creation (existing pattern in `auth.ts`):
```typescript
// Source: landing-page-escluse/src/lib/api/auth.ts [VERIFIED: code]
import apiClient, { ApiResponse } from './client';

export const billingApi = {
  fetchPlans: async () => {
    const response = await apiClient.get<ApiResponse<Plan[]>>('/billing/plans');
    return response.data?.data ?? [];
  },
};
```

#### Auth check (existing pattern in `useAuth.ts`):
```typescript
// Source: landing-page-escluse/src/lib/hooks/useAuth.ts [VERIFIED: code]
const { isAuthenticated } = useAuthStore.getState(); // Sync check outside React
const isAuth = useAuthStore((state) => state.isAuthenticated); // React reactive
```

#### Navigation with state (existing pattern in `useAuth.ts:56`):
```typescript
navigate('/signin', { state: { from: location } });
// Consumer reads: const { state } = useLocation(); state?.from
```

#### Checkout creation (existing pattern in dashboard `BillingPage.jsx:62-73`):
```javascript
const handleUpgrade = async (planId, billingCycle = 'monthly') => {
  const response = await billingApi.createCheckout(planId, billingCycle);
  const checkoutUrl = response?.data?.checkout_url || response?.checkout_url;
  if (checkoutUrl) {
    window.location.href = checkoutUrl;
  }
};
```

#### API response unwrapping (existing pattern in dashboard `BillingPage.jsx:27`):
```javascript
setPlans(plansData?.data || plansData || []);
```

## State of the Art

| Old Approach (Before Phase 71) | Current Approach (After Phase 71) | When Changed | Impact |
|--------------------------------|----------------------------------|--------------|--------|
| Hardcoded plan data in App.tsx:536-690 | API-driven data from `GET /api/v1/billing/plans` | Phase 71 | Prices/features update without deploy. Adds API dependency to landing page. |
| All plan buttons redirect to `/signin` | Auth-gated checkout: paid plans → checkout, Free → signin | Phase 71 | Landing page now handles real subscriptions. |
| No billing toggle on landing page | Monthly/yearly toggle with dynamic savings calculation | Phase 71 | Users see yearly discount and can choose billing cycle. |
| No subscription awareness on landing page | Current plan badge + Manage button for logged-in users | Phase 71 | Landing page becomes personalized for authenticated users. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The Lemon Squeezy redirect URLs point to `https://app.esluce.com/dashboard` (dashboard is on a different subdomain/origin from landing page) [VERIFIED: billing_handlers.rs:126-128] | Architecture | Correct — confirmed in code. No risk. |
| A2 | The `apiClient` in `client.ts` sends cookies for auth on `/billing/*` endpoints correctly (uses `withCredentials: true`) | Standard Stack | Correct — confirmed in code at line 40. No risk. |
| A3 | Zustand's `useAuthStore` has `isAuthenticated` as a reactive property accessible both inside and outside React components | Pattern 2 | Correct — confirmed in code. `useAuthStore.getState()` for sync reads works. |
| A4 | `POST /api/v1/billing/checkout` accepts `plan_id` as a string (UUID or name) | Backend API | Correct — confirmed in billing_handlers.rs:108-114. Uses `Uuid::parse_str` fallback to `find_by_name`. |
| A5 | The Lemon Squeezy checkout passes `user_id` in `custom_data` which the webhook uses to link subscriptions | Backend API | Correct — confirmed in lemon_squeezy_service.rs:114-119. |
| A6 | Post-checkout welcome modal belongs in the `app/` SPA (dashboard), not the landing page | Architecture | **MEDIUM** — confirmed via `success_url = "{dashboard}"`. The welcome modal should detect `?checkout=success` in the dashboard app. The planner must verify the dashboard SPA has route access to `useSearchParams` or can parse `window.location.search`. |

## Open Questions

1. **Does the sign-in page currently support `?redirect=` or custom URL params?**
   - What we know: The `SignInForm` uses `onSuccess: () => navigate('/')` (hardcoded). The `useAuth` hook's `handleLogin` uses `redirectTo` which reads from `location.state?.from`.
   - What's unclear: Whether there's existing param-parsing logic. Need to add it.
   - Recommendation: Add `useSearchParams()` to `SignIn.tsx` to read plan context. Pass via `location.state` from the pricing button click (more reliable than URL params for email/password login). Use `sessionStorage` for OAuth flow as fallback.

2. **Does the dashboard app (`app/`) have `useSearchParams` from react-router?**
   - What we know: The dashboard is a separate React SPA.
   - What's unclear: Whether it uses react-router and has access to search params. There may not be a router in the dashboard — it might be simple component-based routing.
   - Recommendation: Check `app/src/app/App.jsx` routes. If it uses `react-router-dom`, use `useSearchParams`. Otherwise, read from `URLSearchParams(window.location.search)`.

3. **How does `VITE_API_URL` map to the backend in production?**
   - What we know: `client.ts` uses `VITE_API_URL || 'http://localhost:8080'`. The `apiClient` has `baseURL: `${API_URL}/api/v1``.
   - What's unclear: In production, does the backend serve on the same origin as the landing page (so `withCredentials` cookies work), or is there a separate API domain?
   - Recommendation: Verify CORS + cookie config for production. If API is on a different origin, cookies won't be sent (same-origin policy). The existing config suggests same-origin or already-configured proxy.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Run landing page + dashboard dev servers | ✓ | (check runtime) | — |
| npm | Install packages | ✓ | — | — |
| Backend server | API calls to `/api/v1/billing/*` | ✓ | Rust/Axum | Hardcoded fallback for plans (D-04) |
| Lemon Squeezy API | Checkout + portal creation | ✓ | (backend config) | `BILLING_NOT_CONFIGURED` error from backend |
| Vite | Landing page dev/build | ✓ | ^6.2.0 | — |

**Missing dependencies with no fallback:**
- None — all required dependencies exist in the project.

**Missing dependencies with fallback:**
- Backend offline → plan data falls back to hardcoded defaults per D-04.

## Validation Architecture

### Test Framework
Current project does not appear to have a frontend test framework configured for the landing page (`landing-page-escluse/package.json` has `tsc --noEmit` lint but no test scripts). The dashboard app (`app/package.json`) also has no test scripts.

| Property | Value |
|----------|-------|
| Framework | None detected |
| Config file | Not found |
| Quick run command | `npx tsc --noEmit` (type-check only) |
| Full suite command | Not available |

### Phase Requirements → Test Map
Testing for Phase 71 is manual-verification-only due to no existing test infrastructure:

| Behavior | Test Type | Justification |
|----------|-----------|---------------|
| Plans fetch from API on mount | Manual | No test framework. Verify by loading landing page and checking network tab. |
| Monthly/yearly toggle updates display | Manual | No test framework. Verify by clicking toggle and seeing price change. |
| Auth gate redirect to sign-in | Manual | No test framework. Verify by clicking "Start for Hobby" while logged out. |
| Auto-checkout after login | Manual | No test framework. Verify by logging in with plan params and confirming redirect to Lemon Squeezy. |
| Welcome modal on dashboard | Manual | No test framework. Verify by navigating to `/dashboard?checkout=success`. |
| Current plan badge | Manual | No test framework. Verify by logging in with subscription and checking landing page. |
| Error fallback for plans API | Manual | No test framework. Verify by blocking `/api/v1/billing/plans` and checking fallback rendering. |

### Wave 0 Gaps
- [ ] Landing page type-check passes: `npx tsc --noEmit` should return 0 errors
- [ ] Dashboard app build passes: verify build with `npm run build`

*(Existing test infrastructure covers none of the phase requirements. Manual verification is the only available method.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | Yes | Existing cookie-based auth with `withCredentials`. 401 interceptor handles refresh. |
| V3 Session Management | Yes | Existing refresh token pattern in auth interceptor. |
| V4 Access Control | Yes | Backend `VerifiedUser` middleware on checkout/portal/subscription endpoints. |
| V5 Input Validation | Yes | Backend validates `plan_id` and `billing_cycle`. Frontend reads from API, not user input. |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| CSRF on checkout API | Tampering | Backend uses cookie-based auth with `withCredentials` — CSRF-protected if SameSite cookie is set. [ASSUMED: SameSite is standard for this stack] |
| Lemon Squeezy API key exposure | Information Disclosure | Backend handles all Lemon Squeezy API calls. Frontend never sees the API key. |
| Webhook forgery | Spoofing | Backend verifies `x-signature` header using HMAC-SHA256. [CITED: billing_handlers.rs:198-220] |
| Insecure direct object reference on subscription | Information Disclosure | Backend uses `auth_user.user_id` from `VerifiedUser` middleware, not from user-supplied params. |

### What Phase 71 Changes
- No new auth logic — reuses existing `useAuthStore` and backend `VerifiedUser` middleware.
- The pricing cards render from API (read-only, no user input).
- Checkout creation requires auth (backend enforces `VerifiedUser`).
- Plan context in URL params (`?plan=hobby`) is user-facing and could be tampered with, but the backend validates plan existence and the worst case is a 404 error.

## Sources

### Primary (HIGH confidence)
- **Backend billing handlers** — `migration/src/presentation/handlers/billing_handlers.rs` — All 4 billing endpoint implementations verified.
- **Lemon Squeezy service** — `migration/src/infrastructure/billing/lemon_squeezy_service.rs` — Checkout and portal creation logic.
- **Plan model** — `migration/src/domain/plan/model.rs` — Schema with all fields (price_monthly, price_yearly, limits, features, lemon_squeezy_variant_ids).
- **Subscription model** — `migration/src/domain/subscription/model.rs` — Schema with status, plan_id, provider_subscription_id.
- **Landing page Pricing component (hardcoded)** — `landing-page-escluse/src/App.tsx:536-690` — Current pricing section to be modified.
- **Landing page API client** — `landing-page-escluse/src/lib/api/client.ts` — Existing Axios instance with `withCredentials: true`.
- **Landing page auth API** — `landing-page-escluse/src/lib/api/auth.ts` — Existing auth endpoint functions.
- **Landing page auth store** — `landing-page-escluse/src/lib/stores/authStore.ts` — Zustand store with `isAuthenticated`, `user`, `login()`, `fetchUser()`.
- **Dashboard billing page (reference)** — `app/src/pages/billing/BillingPage.jsx` — Existing checkout flow pattern.
- **Dashboard API client (reference)** — `app/src/lib/api.js` — Existing `billingApi` with `createCheckout`, `getPlans`, `getCurrentSubscription`.
- **Database plan seeds** — `migration/migrations/20260324000004_create_plans_table.sql` — Free ($0), Hobby ($6.99/mo, $69.90/yr), Pro ($24.99/mo, $249.90/yr), Enterprise ($99.99/mo, $999.90/yr).
- **Context & Discussion Log** — `.planning/phases/71-buat-agar-plan-hobby-dan-pro-yang-ada-di-landing-page-bisa-b/71-CONTEXT.md` — All locked decisions documented.

### Secondary (MEDIUM confidence)
- **App routing** — `landing-page-escluse/src/App.tsx:852-871` — Routes structure. `/signin`, `/signup`, `/onboarding`, `/` all handled.
- **OAuth callback flow** — `landing-page-escluse/src/pages/oauth/OAuthCallback.tsx` — Uses `sessionStorage.getItem('intended_destination')` for post-OAuth redirect.
- **SignIn page** — `landing-page-escluse/src/pages/SignIn.tsx` — Currently hardcodes `onSuccess: () => navigate('/')`.
- **useAuth hook** — `landing-page-escluse/src/lib/hooks/useAuth.ts` — `redirectTo` reads `location.state?.from?.pathname`.

### Tertiary (LOW confidence)
- None — all critical claims verified against code or backed by CONTEXT.md decisions.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All existing project libraries verified in codebase.
- Architecture: HIGH — Two-app architecture confirmed. All backend endpoints verified in code.
- Pitfalls: HIGH — Auth gate timing, OAuth param loss, welcome modal duplication all identified from code analysis and documented patterns.

**Research date:** 2026-06-11
**Valid until:** 2026-07-11 (30 days — stack is stable)
