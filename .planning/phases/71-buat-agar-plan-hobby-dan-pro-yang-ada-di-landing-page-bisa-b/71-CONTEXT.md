# Phase 71: buat agar plan hobby dan pro yang ada di landing page, bisa benar berfungsi untuk berlangganan - Context

**Gathered:** 2026-06-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the existing backend billing/subscription system (Plan entity, Subscription entity, `GET /api/v1/billing/plans`, `POST /api/v1/billing/checkout`, Lemon Squeezy webhook integration) into the landing page pricing UI so that Hobby and Pro plan buttons create real subscription checkout flows instead of just redirecting to `/signin`.

**Before Phase 71:** Landing page pricing section has hardcoded plan data (names, prices $0/$6.99/$24.99, features, limits). All plan buttons (Free, Hobby, Pro) redirect to `/signin`. No subscription checkout integration exists on the frontend.

**After Phase 71:**
- Pricing section fetches live plans from backend API
- "Start for Hobby" / "Upgrade to Pro" buttons create Lemon Squeezy checkout sessions for authenticated users
- Unauthenticated users are redirected to sign in first, then auto-checkout
- Monthly/yearly billing toggle on pricing cards
- Post-checkout redirect goes to dashboard with welcome modal
- Logged-in users see their current plan badge on pricing section
- Subscription management via Lemon Squeezy Customer Portal

**Out of scope:**
- Enterprise plan full integration (plan exists in DB but no pricing card or special checkout — defer to phase 72 or later)
- In-app plan upgrade/downgrade/cancel UI (use Lemon Squeezy Customer Portal)
- Custom billing provider beyond Lemon Squeezy
- Free plan subscription record (sign-up only — Free users don't get a subscription entity)
</domain>

<decisions>
## Implementation Decisions

### Pricing Data Source
- **D-01 (API source):** Fetch plans from `GET /api/v1/billing/plans` on Pricing component mount. All card content (display name, description, price, features, limits) comes from API response. Backend is source of truth.
- **D-02 (Loading state):** Block pricing section with centered spinner while plans API loads. No skeleton cards.
- **D-03 (Refresh frequency):** Fetch once per Pricing component mount. No stale-while-revalidate, no caching.
- **D-04 (Error fallback):** If API call fails, show error toast and render hardcoded defaults so the section never breaks.

### Checkout Trigger Flow
- **D-05 (Unauthenticated flow):** Clicking a paid plan button while logged out redirects to `/signin` with `?plan=hobby&plan_cycle=monthly` params. After successful sign-in, automatically call checkout API and redirect to Lemon Squeezy.
- **D-06 (Authenticated flow):** Clicking a paid plan button while logged in calls `POST /api/v1/billing/checkout` with `plan_id` and `billing_cycle`. Backend returns Lemon Squeezy checkout URL. Frontend does `window.location.href = url`.
- **D-07 (Default billing cycle):** Monthly. Yearly is selected via the billing toggle.
- **D-08 (Checkout callback):** After Lemon Squeezy checkout completes, redirect to dashboard (`/` or app URL). Success/cancel status passed as URL params.

### Monthly/Yearly Toggle
- **D-09 (Toggle visibility):** Show a "Monthly" / "Yearly" toggle above the plan cards.
- **D-10 (Price display):** Replace price text based on active toggle — show `/mo` when monthly, `/yr` when yearly, with yearly showing "Save ~X%" computed from API prices.
- **D-11 (Savings calculation):** Compute dynamically client-side: `(price_monthly * 12 - price_yearly) / (price_monthly * 12) * 100`.

### Post-Checkout UX
- **D-12 (Welcome modal):** Show a welcome modal on dashboard after first successful subscription. Displays plan name, features unlocked, limits, and "Start creating servers" CTA.
- **D-13 (Current plan badge):** For logged-in users, fetch subscription status on landing page and show "Current plan" badge on the matching pricing card. "Manage" button opens Lemon Squeezy Customer Portal instead of checkout.
- **D-14 (Subscription management):** Use `POST /api/v1/billing/portal` to create Lemon Squeezy Customer Portal session for plan changes, cancellations, and billing history.

### Free Plan
- **D-15 (Free plan behavior):** "Get Started Free" redirects to `/signin`. Sign-up creates a user account but does NOT create a subscription record. Free tier limits are enforced by the backend without a subscription entity. The Free plan card in the API is informational only.

### the agent's Discretion
- Exact spinner/loading component styling for pricing section
- Welcome modal design (content, CTA buttons, dismiss behavior)
- Toast messages for checkout success/cancel
- Plan badge positioning and styling on pricing cards
- Whether the pricing section uses a `usePlans` hook or inline fetch
- API client module naming for billing calls in landing page
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and roadmap context
- `.planning/ROADMAP.md` § Phase 71 — Goal: "buat agar plan hobby dan pro yang ada di landing page, bisa benar berfungsi untuk berlangganan"

### Landing page pricing section (to be modified)
- `landing-page-escluse/src/App.tsx:536-690` — Current Pricing component. Hardcoded Free/Hobby/Pro cards with buttons pointing to `/signin`.
- `landing-page-escluse/src/lib/api/client.ts` — Existing Axios client. Phase 71 adds billing API calls here.
- `landing-page-escluse/src/lib/api/auth.ts` — Existing auth API functions. Phase 71 may need to extend for checkout redirect flow.

### Backend billing/subscription infrastructure (to be consumed)
- `migration/src/presentation/handlers/billing_handlers.rs:31-45` — Billing routes: GET /plans, POST /checkout, POST /portal, POST /webhook
- `migration/src/presentation/handlers/billing_handlers.rs:47-70` — `list_plans` handler — returns active plans with name, display_name, description, price_monthly, price_yearly, limits, features
- `migration/src/presentation/handlers/billing_handlers.rs:98-130` — `create_checkout` handler — requires VerifiedUser, returns Lemon Squeezy checkout URL
- `migration/src/presentation/handlers/billing_handlers.rs:275-288` — Lemon Squeezy variant ID mapping (Hobby/Pro monthly/yearly → variant IDs → plan UUIDs)
- `migration/src/domain/plan/model.rs` — Plan struct with all fields + PlanLimits for JSONB deserialization
- `migration/src/domain/subscription/model.rs` — Subscription struct
- `migration/src/domain/billing/service.rs` — BillingService: create_checkout, create_portal, handle webhooks
- `migration/src/domain/billing/billing_trait.rs` — Abstract BillingProvider trait

### Database schema
- `migration/migrations/20260324000004_create_plans_table.sql` — Plans table schema + seed data (free/hobby/pro/enterprise)
- `migration/migrations/20260324000005_create_subscriptions_table.sql` — Subscriptions table schema
- `migration/migrations/20260324000010_create_billing_tables.sql` — billing_customers, payment_transactions, invoices tables
- `migration/migrations/20260501000002_add_missing_plan_columns.sql` — Lemon Squeezy variant IDs on plans table

### Quick task reference (prior landing page pricing work)
- `.planning/quick/260611-5m0-update-bagian-plans-di-landing-page-agar/260611-5m0-SUMMARY.md` — Previous work added scroll animations to pricing section. Buttons still point to /signin.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`apiClient` (landing-page-escluse/src/lib/api/client.ts)** — Existing Axios instance with `baseURL: /api/v1`, `withCredentials: true`, 401 refresh interceptor. Phase 71 adds billing plan fetch + checkout call through this.
- **`Pricing` component (App.tsx:536-690)** — Already renders 3 plan cards with motion animations, feature lists, CTA buttons. Phase 71 replaces hardcoded data with API-driven rendering and wires button onClick to checkout flow.
- **`BillingHandlers::list_plans` (billing_handlers.rs:47-70)** — Returns structured plan data. Direct consumer for pricing section.
- **`BillingHandlers::create_checkout` (billing_handlers.rs:98-130)** — Creates Lemon Squeezy checkout. Accepts `plan_id` + `billing_cycle`. Returns checkout URL.

### Established Patterns
- **API client pattern** — Existing `apiClient.get<ApiResponse<T>>('/resource')` pattern in all landing page API modules. Phase 71 follows same pattern for billing.
- **Auth redirect with return URL** — Sign-in page presumably supports `?redirect=` param (verify during planning). Phase 71 passes `?plan=hobby&plan_cycle=monthly` as redirect context.
- **Lemon Squeezy hosted checkout** — Backend already creates checkout sessions. Frontend does `window.location.href` to the URL. No iframe/complex integration needed.
- **Toast notifications for async feedback** — Existing toast/notification pattern in the landing page/app. Phase 71 uses for checkout success/cancel.

### Integration Points
- **New API module:** `landing-page-escluse/src/lib/api/billing.ts` — Add `fetchPlans()`, `createCheckout(planId, cycle)`, `createPortal()`, `getSubscription()` functions.
- **Modify Pricing component:** Replace hardcoded plan data with `fetchPlans()` call → render from API. Replace onClick handlers with auth gate + checkout flow.
- **Add billing toggle UI:** Above plan cards in Pricing component. Toggle state determines which price is shown and which cycle is passed to checkout.
- **Add welcome modal:** After dashboard redirect with `?checkout=success`, detect param and show modal.
- **Add current plan badge:** Optional API call to `GET /api/v1/billing/subscription` on landing page (only when user is authenticated) to show plan badge.
- **Lemon Squeezy redirect URLs:** Must configure Lemon Squeezy product redirect URLs to point to dashboard with success/cancel params.
</code_context>

<specifics>
## Specific Ideas

### Checkout flow state machine
```
User clicks "Start for Hobby"
  ↓
Is user authenticated?
├─ No  → Save intended plan + cycle to URL params
│        Redirect to /signin?plan=hobby&plan_cycle=monthly
│        After sign-in: read params, call checkout API
│        window.location.href = checkout_url
│
└─ Yes → Call POST /api/v1/billing/checkout { plan_id, billing_cycle }
         window.location.href = checkout_url
  ↓
Lemon Squeezy checkout
  ↓
Success → Redirect to dashboard?checkout=success
  → Show welcome modal (plan name, features, "Start creating servers")
Cancel  → Redirect to dashboard?checkout=canceled
  → Show toast: "Checkout canceled"
```

### Pricing toggle + data flow
```
[Monthly ●────○ Yearly]  ← toggle state drives displayed price + checkout cycle

Free:     $0/mo         (no toggle effect)
Hobby:    $6.99/mo   or $69.99/yr (save ~17%)
Pro:      $24.99/mo  or $249.99/yr (save ~17%)
```

### Current plan badge (logged-in users)
```
[On page load]
  if (authenticated):
    GET /api/v1/billing/subscription → { plan_name: "hobby" }
    Match plan name to API plans list
    Add "Current Plan" badge + "Manage" button on matching card
  else:
    Show all "Subscribe" buttons normally
```
</specifics>

<deferred>
## Deferred Ideas

- **Enterprise plan pricing card** — Plan exists in DB ($99.99/mo) but has no pricing card on landing page. Needs content design (features, limits, target audience). Belongs in its own phase or quick task.
- **In-app plan management UI** — Currently delegated to Lemon Squeezy Customer Portal. A custom upgrade/downgrade/cancel UI inside the dashboard is a future enhancement.
- **Free plan subscription record** — Not creating a subscription entity for Free users. If plan-based feature gating needs to change, this can be revisited.

None — discussion stayed within phase scope.
</deferred>

---

*Phase: 71-buat-agar-plan-hobby-dan-pro-yang-ada-di-landing-page-bisa-b*
*Context gathered: 2026-06-11*
