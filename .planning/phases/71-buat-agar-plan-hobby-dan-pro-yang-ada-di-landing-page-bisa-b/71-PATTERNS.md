# Phase 71: Subscription Plans on Landing Page - Pattern Map

**Mapped:** 2026-06-11
**Files analyzed:** 8 (5 new, 2 modified, 1 reference)
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `landing-page-escluse/src/lib/api/billing.ts` | api/client | CRUD (request-response) | `landing-page-escluse/src/lib/api/auth.ts` | exact |
| `landing-page-escluse/src/components/pricing/PricingSection.tsx` | component | request-response (fetch + render) | `landing-page-escluse/src/App.tsx:536-690` (inline Pricing) | exact |
| `landing-page-escluse/src/components/pricing/PlanCard.tsx` | component | request-response (presentation) | `landing-page-escluse/src/App.tsx:554-589` (Free card block) | exact |
| `landing-page-escluse/src/components/pricing/BillingToggle.tsx` | component | event-driven (toggle) | `landing-page-escluse/src/App.tsx:40-58` (ThemeToggle) | exact |
| `landing-page-escluse/src/components/pricing/CurrentPlanBadge.tsx` | component | request-response (display) | `app/src/pages/billing/BillingPage.jsx:301-304` (Current Plan badge) | role-match |
| `app/src/pages/dashboard/WelcomeModal.jsx` | component (modal) | event-driven (param detection) | `app/src/components/InviteFriendsModal.jsx` | exact |
| `landing-page-escluse/src/pages/SignIn.tsx` | page | request-response (+ redirect) | `landing-page-escluse/src/components/auth/SignInForm.tsx` | exact |
| `landing-page-escluse/src/App.tsx` | app root | composition | same file (inline Pricing at lines 536-690) | exact |

## Pattern Assignments

### `landing-page-escluse/src/lib/api/billing.ts` (api/client, CRUD)

**Analog:** `landing-page-escluse/src/lib/api/auth.ts` (exact role-match)

**Imports pattern** (auth.ts lines 1-1):
```typescript
import apiClient, { ApiResponse, AuthApiResponse, User } from './client';
```

Use the same `ApiResponse<T>` wrapper type with `apiClient` (existing Axios instance at client.ts line 5-12):
```typescript
export interface ApiResponse<T> {
  success?: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}
```

**Core API module pattern** (auth.ts lines 22-67) — export a `billingApi` object with named async functions:
```typescript
export const authApi = {
  login: async (email: string, password: string): Promise<ApiResponse<AuthApiResponse>> => {
    const response = await apiClient.post<ApiResponse<AuthApiResponse>>('/auth/login', {
      email,
      password,
    });
    return response.data;
  },

  me: async (): Promise<ApiResponse<User>> => {
    const response = await apiClient.get<ApiResponse<User>>('/auth/me');
    return response.data;
  },
};
```

**Error handling** — The `ApiResponse<T>` wrapper has an `error` field. Consumer checks `response.error` before using `response.data`. The `apiClient` also has a 401 interceptor (client.ts lines 50-68) that auto-refreshes tokens on 401.

**Pattern to follow for billing.ts** (mapped from RESEARCH.md § Standard Stack and auth.ts):
```typescript
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

export const billingApi = {
  fetchPlans: async (): Promise<Plan[]> => {
    const response = await apiClient.get<ApiResponse<Plan[]>>('/billing/plans');
    return response.data?.data ?? [];
  },
  createCheckout: async (planId: string, billingCycle: string = 'monthly') => {
    const response = await apiClient.post<ApiResponse<{checkout_url: string}>>('/billing/checkout', {
      plan_id: planId,
      billing_cycle: billingCycle,
    });
    return response.data?.data ?? response.data;
  },
  getSubscription: async () => {
    const response = await apiClient.get<ApiResponse<{status: string; plan: object}>>('/billing/subscription');
    return response.data?.data ?? response.data;
  },
  createPortal: async () => {
    const response = await apiClient.post<ApiResponse<{portal_url: string}>>('/billing/portal');
    return response.data?.data ?? response.data;
  },
};
```

---

### `landing-page-escluse/src/components/pricing/PricingSection.tsx` (component, request-response)

**Analog:** `landing-page-escluse/src/App.tsx:536-690` (inline Pricing component — exact match)

**Imports pattern** (App.tsx lines 6-8):
```typescript
import { motion } from 'motion/react';
import { useNavigate } from 'react-router-dom';
import { CheckCircle2 } from 'lucide-react';
import { useAuthStore } from '../../lib/stores/authStore';
import { billingApi } from '../../lib/api/billing';
```

**Animation pattern** — Each card uses `motion.div` with `initial`, `whileInView`, `viewport`, `transition`:
```typescript
<motion.div
  initial={{ opacity: 0, y: 20 }}
  whileInView={{ opacity: 1, y: 0 }}
  viewport={{ once: true }}
  transition={{ duration: 0.5, delay: 0.1 }}
>
```

**Section wrapper pattern** (App.tsx lines 539-540):
```typescript
<section id="pricing" className="py-24 bg-surface">
  <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
```

**Grid layout pattern** (App.tsx line 552):
```typescript
<div className="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto items-center mb-12">
```

**Auth gate + checkout flow** — From RESEARCH.md § Pattern 2 (lines 279-297):
```typescript
// In PricingSection component:
const handleSubscribe = async (plan, cycle) => {
  const { isAuthenticated } = useAuthStore.getState();

  if (!isAuthenticated) {
    navigate('/signin', {
      state: { plan: plan.name, planCycle: cycle }
    });
    return;
  }

  try {
    const { checkout_url } = await billingApi.createCheckout(plan.name, cycle);
    window.location.href = checkout_url;
  } catch (err) {
    // Show error toast via existing pattern
  }
};
```

**Free plan button** (stays as redirect to signin, App.tsx lines 583-588):
```typescript
<button
  onClick={() => navigate('/signin')}
  className="w-full py-3 px-4 rounded-xl font-semibold text-on-surface border border-surface-container hover:bg-surface-container-low transition-colors"
>
  Get Started Free
</button>
```

**API fetch pattern** (useEffect on mount):
```typescript
const [plans, setPlans] = useState<Plan[]>([]);
const [isLoading, setIsLoading] = useState(true);
const [error, setError] = useState<string | null>(null);

useEffect(() => {
  const loadPlans = async () => {
    try {
      const data = await billingApi.fetchPlans();
      setPlans(data);
    } catch (err) {
      setError('Failed to load plans');
      // Render hardcoded defaults as fallback per D-04
    } finally {
      setIsLoading(false);
    }
  };
  loadPlans();
}, []);
```

**Loading state pattern** — Centered spinner (D-02):
```typescript
if (isLoading) {
  return (
    <div className="flex items-center justify-center py-24">
      <div className="w-12 h-12 border-4 border-primary/30 border-t-primary rounded-full animate-spin" />
    </div>
  );
}
```

---

### `landing-page-escluse/src/components/pricing/PlanCard.tsx` (component, presentation)

**Analog:** Each individual card block from App.tsx (e.g., Hobby card lines 592-631)

**Card structure pattern:**
```typescript
<motion.div
  initial={{ opacity: 0, y: 20 }}
  whileInView={{ opacity: 1, y: 0 }}
  viewport={{ once: true }}
  transition={{ duration: 0.5, delay: 0.2 }}
  className={`bg-surface-container-lowest rounded-3xl p-8 border ${
    isHighlighted
      ? 'border-2 border-primary shadow-[0_20px_40px_-10px_rgba(70,72,212,0.1)] relative transform lg:-translate-y-4'
      : 'border-surface-container shadow-sm'
  }`}
>
```

**"Most Popular" badge pattern** (App.tsx lines 599-601):
```typescript
{isHighlighted && (
  <div className="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-primary text-white px-4 py-1 rounded-full text-xs font-bold uppercase tracking-wider">
    Most Popular
  </div>
)}
```

**Feature list pattern** (App.tsx lines 617-622):
```typescript
<ul className="space-y-3 mb-6">
  {features.map((feature, i) => (
    <li key={i} className="flex items-start gap-3 text-sm text-on-surface font-medium">
      <CheckCircle2 className="w-4 h-4 text-primary shrink-0 mt-0.5" />
      <span>{feature}</span>
    </li>
  ))}
</ul>
```

**Plan button pattern** — Passing `plan` and `cycle` up to parent via `onSubscribe`:
```typescript
interface PlanCardProps {
  plan: Plan;
  billingCycle: 'monthly' | 'yearly';
  isHighlighted: boolean;
  isCurrentPlan: boolean;
  onSubscribe: (plan: Plan, cycle: 'monthly' | 'yearly') => void;
  onManage: () => void;
}

// Inside render:
{isCurrentPlan ? (
  <div className="space-y-2">
    <div className="w-full py-3 px-4 rounded-xl font-semibold text-center bg-primary/10 text-primary border border-primary/30">
      Current Plan
    </div>
    <button onClick={onManage} className="w-full py-2 px-4 rounded-xl font-semibold text-sm text-on-surface-variant border border-surface-container hover:bg-surface-container-low transition-colors">
      Manage
    </button>
  </div>
) : (
  <button
    onClick={() => onSubscribe(plan, billingCycle)}
    className="w-full py-3 px-4 rounded-xl font-semibold ..."
  >
    {plan.name === 'Free' ? 'Get Started Free' : `Start for ${plan.display_name}`}
  </button>
)}
```

---

### `landing-page-escluse/src/components/pricing/BillingToggle.tsx` (component, event-driven)

**Analog:** `ThemeToggle` component at App.tsx lines 40-58 (same toggle pattern)

**Core pattern** (App.tsx lines 42-54):
```typescript
const [isMonthly, setIsMonthly] = useState(true); // default monthly per D-07

// Alternative: controlled component
interface BillingToggleProps {
  isMonthly: boolean;
  onChange: (isMonthly: boolean) => void;
}
```

**Savings calculation** (from RESEARCH.md § Pattern 4, lines 348-355):
```typescript
const calculateSavings = (monthly: number, yearly: number | null): number | null => {
  if (!yearly || monthly <= 0) return null;
  const yearlyIfMonthly = monthly * 12;
  const savings = ((yearlyIfMonthly - yearly) / yearlyIfMonthly) * 100;
  return Math.round(savings);
};
```

**Toggle UI pattern:**
```typescript
<div className="flex items-center justify-center gap-4 mb-8">
  <span className={`text-sm font-semibold ${isMonthly ? 'text-on-surface' : 'text-on-surface-variant'}`}>
    Monthly
  </span>
  <button
    onClick={() => setIsMonthly(!isMonthly)}
    className={`w-14 h-7 rounded-full p-1 transition-colors ${
      isMonthly ? 'bg-primary' : 'bg-surface-container-high'
    }`}
  >
    <div className={`w-5 h-5 rounded-full bg-white shadow-sm transition-transform ${
      isMonthly ? 'translate-x-0' : 'translate-x-7'
    }`} />
  </button>
  <span className={`text-sm font-semibold ${!isMonthly ? 'text-on-surface' : 'text-on-surface-variant'}`}>
    Yearly
  </span>
</div>
```

---

### `landing-page-escluse/src/components/pricing/CurrentPlanBadge.tsx` (component, display)

**Analog:** `app/src/pages/billing/BillingPage.jsx:301-304` (Current Plan badge/detection)

**Pattern from BillingPage.jsx lines 256-312:**
```javascript
// Check if current plan matches card
{currentSubscription?.plan?.name === p.name ? (
  <div className="w-full py-2.5 bg-green-600 text-white rounded text-center font-medium">
    Current Plan
  </div>
) : (
  <button onClick={() => handleUpgrade(p.id)}>Subscribe</button>
)}
```

**Manage button pattern** — Opens Lemon Squeezy Customer Portal (D-13, D-14):
```typescript
const handleManage = async () => {
  try {
    const { portal_url } = await billingApi.createPortal();
    window.location.href = portal_url;
  } catch (err) {
    console.error('Failed to open portal:', err);
  }
};
```

**Extracted as reusable component:**
```typescript
interface CurrentPlanBadgeProps {
  planName: string;       // plan.name from API
  currentPlanName: string | null;  // subscription.plan.name from getSubscription()
  onManage: () => void;
}

// Returns null if no match, returns badge + Manage button if matches
```

---

### `app/src/pages/dashboard/WelcomeModal.jsx` (component/modal, event-driven)

**Analog:** `app/src/components/InviteFriendsModal.jsx` (exact same role — modal in dashboard app)

**Modal overlay pattern** (InviteFriendsModal.jsx lines 60-69):
```jsx
<div
  className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4"
  onClick={onClose}
  role="dialog"
  aria-modal="true"
>
  <div
    className="bg-gray-800 border border-gray-700 rounded-lg p-6 max-w-md w-full space-y-4"
    onClick={(e) => e.stopPropagation()}
  >
    {/* Modal content */}
  </div>
</div>
```

**Close button pattern** (InviteFriendsModal.jsx lines 72-79):
```jsx
<div className="flex items-center justify-between">
  <h2 className="text-lg font-semibold text-white">{title}</h2>
  <button onClick={onClose} className="text-gray-400 hover:text-white text-xl leading-none" aria-label="Close">
    ×
  </button>
</div>
```

**URL param detection pattern** (researched, confirmed app uses react-router-dom in App.jsx):
```jsx
import { useSearchParams } from 'react-router-dom';
import { useState, useEffect } from 'react';

export default function WelcomeModal() {
  const [searchParams] = useSearchParams();
  const [isOpen, setIsOpen] = useState(false);
  const checkoutStatus = searchParams.get('checkout');

  useEffect(() => {
    if (checkoutStatus === 'success') {
      setIsOpen(true);
      window.history.replaceState({}, '', '/dashboard');
    }
  }, [checkoutStatus]);

  if (!isOpen) return null;
  // render modal
}
```

**Alt approach** (if dashboard doesn't use react-router properly for search params):
```jsx
useEffect(() => {
  const params = new URLSearchParams(window.location.search);
  if (params.get('checkout') === 'success') {
    setIsOpen(true);
    window.history.replaceState({}, '', window.location.pathname);
  }
}, []);
```

**Caveat** — The dashboard App.jsx wraps DashboardPage inside nested `<Routes>`. The `useSearchParams` hook should work since it's inside a `<Route>` context (App.jsx line 114: `<Route path="/" element={<DashboardPage />} />`). Confirm during implementation.

---

### `landing-page-escluse/src/pages/SignIn.tsx` (page, request-response + redirect)

**Analog:** `landing-page-escluse/src/components/auth/SignInForm.tsx` (existing pattern + onSuccess callback)

**Current import and structure** (SignIn.tsx lines 1-8):
```typescript
import React from 'react';
import { SignInForm } from '../components/auth/SignInForm';
import { Footer } from '../components/auth/Footer';
import { TopLogo } from '../components/auth/TopLogo';
import { useNavigate, useSearchParams } from 'react-router-dom';

export const SignIn = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
```

**Plan context extraction** — Read from `location.state` (from Pricing button click) or URL params:
```typescript
const location = useLocation();
const planFromState = location.state?.plan;
const planCycleFromState = location.state?.planCycle;
const planFromParams = searchParams.get('plan');
const planCycleFromParams = searchParams.get('plan_cycle');
const plan = planFromState || planFromParams;
const planCycle = planCycleFromState || planCycleFromParams;
```

**onSuccess callback modification** — Pass auto-checkout logic:
```typescript
<SignInForm
  onNavigateToSignUp={() => navigate('/signup')}
  onSuccess={() => {
    const plan = location.state?.plan || searchParams.get('plan');
    const planCycle = location.state?.planCycle || searchParams.get('plan_cycle');
    if (plan && planCycle) {
      // Save to sessionStorage for OAuth fallback, then auto-checkout
      billingApi.createCheckout(plan, planCycle).then(({ checkout_url }) => {
        window.location.href = checkout_url;
      });
    } else {
      navigate('/');
    }
  }}
/>
```

**OAuth fallback** — Save to sessionStorage before redirecting (relevant for `OAuthCallback.tsx` lines 46-49):
```typescript
// In the PricingSection handleSubscribe, before navigating:
sessionStorage.setItem('intended_plan', JSON.stringify({ plan: plan.name, cycle }));

// In OAuthCallback.tsx, after successful auth:
const intendedPlan = sessionStorage.getItem('intended_plan');
if (intendedPlan) {
  sessionStorage.removeItem('intended_plan');
  const { plan, cycle } = JSON.parse(intendedPlan);
  const { checkout_url } = await billingApi.createCheckout(plan, cycle);
  window.location.href = checkout_url;
}
```

---

### `landing-page-escluse/src/App.tsx` (app root, composition)

**Modification scope:** Replace inline `Pricing` component (lines 536-690) with `PricingSection` import.

**Import pattern** (near top of App.tsx, after existing imports on lines 6-38):
```typescript
import { PricingSection } from './components/pricing/PricingSection';
```

**Replacement** — The `<Pricing />` component definition (lines 536-690) is removed and the `<PricingSection />` component is placed at the same location in the `LandingPage` layout.

**Existing section insertion point** — Find where `<Pricing />` is called in the `LandingPage` component rendering (likely passes it as a component call in the page flow). Replace with `<PricingSection />`.

---

## Shared Patterns

### Authentication Check
**Source:** `landing-page-escluse/src/lib/stores/authStore.ts`
**Apply to:** `PricingSection`, `PlanCard`, `SignIn.tsx`

```typescript
// Synchronous check (use outside React render):
const { isAuthenticated } = useAuthStore.getState();

// Reactive check (inside React render):
const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
```

### Checkout Redirect
**Source:** `app/src/pages/billing/BillingPage.jsx:62-74`
**Apply to:** `PricingSection`, `SignIn.tsx`

```javascript
const handleUpgrade = async (planId, billingCycle = 'monthly') => {
  const response = await billingApi.createCheckout(planId, billingCycle);
  const checkoutUrl = response?.data?.checkout_url || response?.checkout_url;
  if (checkoutUrl) {
    window.location.href = checkoutUrl;  // NOT window.open()
  }
};
```

**Critical:** Use `window.location.href = url` not `window.open(url)` per D-06 and RESEARCH.md § Anti-Patterns.

### Navigation with State
**Source:** `landing-page-escluse/src/lib/hooks/useAuth.ts:56`
**Apply to:** `PricingSection` → `SignIn.tsx`

```typescript
// Sender (PricingSection):
navigate('/signin', { state: { plan: plan.name, planCycle: cycle } });

// Receiver (SignIn.tsx):
const { state } = useLocation();
const plan = state?.plan;
```

### API Response Unwrapping
**Source:** `app/src/pages/billing/BillingPage.jsx:26-28`
**Apply to:** All billing API consumers in landing page

```javascript
setPlans(plansData?.data || plansData || []);
setCurrentSubscription(subscriptionData?.data || subscriptionData || null);
```

The `ApiResponse<T>` wrapper has `{success, data}` shape. Always unwrap with `.data?.data ?? .data` fallback.

### Toast Notifications
**Source (dashboard app):** `app/src/store/uiStore.js:21-27` + `app/src/components/ToastContainer.jsx`
**Apply to:** Welcome success/cancel toast in dashboard, error toast in landing page

```javascript
// Dashboard pattern:
const { addToast } = useUIStore();
addToast({ type: 'success', message: 'Checkout successful!' });
addToast({ type: 'error', message: 'Checkout canceled' });
```

The landing page may use a different toast mechanism. Check `landing-page-escluse` for existing toast component before implementing.

### Spinner/Loading
**Source (landing page):** `landing-page-escluse/src/components/SkeletonLoader.tsx`
**Source (dashboard app):** `app/src/components/SkeletonLoader.jsx:158-164`

```tsx
// Landing page pattern — centered spinner for pricing loading state (D-02):
<div className="flex items-center justify-center py-24">
  <div className="w-12 h-12 border-4 border-primary/30 border-t-primary rounded-full animate-spin" />
</div>

// Dashboard app pattern:
import { EscluseSpinner } from '../../components/SkeletonLoader';
<EscluseSpinner size={100} color="#06b6d4" />
```

### Modal Pattern (Dashboard App)
**Source:** `app/src/components/InviteFriendsModal.jsx`
**Apply to:** `WelcomeModal.jsx`

```jsx
// InviteFriendsModal.jsx lines 60-69 — fixed overlay + centered card pattern:
<div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4" onClick={onClose}>
  <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 max-w-md w-full space-y-4" onClick={(e) => e.stopPropagation()}>
    {/* header, content, close */}
  </div>
</div>
```

## No Analog Found

All files have a matching analog in the codebase. No uncovered patterns.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All files covered |

## Metadata

**Analog search scope:** `landing-page-escluse/src/`, `app/src/`
**Files scanned:** ~20 files across both frontend apps
**Pattern extraction date:** 2026-06-11
