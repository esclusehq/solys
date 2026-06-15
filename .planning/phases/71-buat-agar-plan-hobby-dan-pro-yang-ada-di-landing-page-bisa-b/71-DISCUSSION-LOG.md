# Phase 71: buat agar plan hobby dan pro yang ada di landing page, bisa benar berfungsi untuk berlangganan - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-11
**Phase:** 71-buat-agar-plan-hobby-dan-pro-yang-ada-di-landing-page-bisa-b
**Areas discussed:** Pricing data source, Checkout trigger flow, Monthly/yearly toggle, Post-checkout UX

---

## Pricing Data Source

| Option | Description | Selected |
|--------|-------------|----------|
| Fetch from API (Recommended) | Call GET /api/v1/billing/plans on page load. Keeps prices/features in sync with DB. Adds ~1 API call but enables dynamic updates without redeploy. | ✓ |
| Keep hardcoded | Simpler, zero API calls on landing page. But prices/features drift if DB changes. Requires Vite build + deploy to update. | |

**User's choice:** Fetch from API
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Show skeleton cards | Render placeholder cards with shimmer animation while loading. If API fails, show error toast + fallback to hardcoded defaults. | |
| Render immediately with fallback | Show hardcoded defaults immediately, then swap in API data when it arrives. No skeleton needed. | |
| Block page section until loaded | Show a centered spinner where pricing section lives. Only render cards after data arrives. | ✓ |

**User's choice:** Block page section until loaded
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Fetch once per page load (Recommended) | Fetch plans on Pricing component mount. Fresh enough for pricing data. Simple — no stale-while-revalidate complexity. | ✓ |
| Fetch on every visit (no cache) | Always hit the API even if user navigates away and back. Most accurate, slightly more API calls. | |
| SSG at build time | Fetch at Vite build time. Zero API calls at runtime, but requires rebuild to update. | |

**User's choice:** Fetch once per page load
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Render API data directly (Recommended) | Display name, description, price, features, limits all come from API. Landing page just iterates the plans array. Backend is source of truth. | ✓ |
| API for prices only, rest stays hardcoded | Only fetch price_monthly/yearly from API. Features list, descriptions, limits stay in App.tsx. | |

**User's choice:** Render API data directly
**Notes:** None

---

## Checkout Trigger Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Redirect to /signin, then auto-checkout after login (Recommended) | Navigate to /signin with ?plan=hobby&redirect=/billing/checkout param. After login, automatically create checkout and go to Lemon Squeezy. | ✓ |
| Show a sign-in modal inline | Show a small modal/dialog on the pricing section asking user to sign in or sign up. No page navigation. After auth, proceed to checkout. | |
| Redirect to /signin, show plan details after login | After login, land on a 'select your plan' page (not auto-checkout). Let user confirm before hitting Lemon Squeezy. | |

**User's choice:** Redirect to /signin, then auto-checkout after login
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Backend redirect (Recommended) | Call POST /api/v1/billing/checkout with plan_id + billing_cycle. Backend returns Lemon Squeezy checkout URL. Frontend does window.location.href = url. | ✓ |
| Open Lemon Squeezy in new tab | Same backend call, but open checkout URL in a new browser tab. | |
| In-page iframe | Render Lemon Squeezy checkout in an iframe or popup on the landing page. More complex, potentially blocked by browsers. | |

**User's choice:** Backend redirect
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Monthly (Recommended) | Default to monthly billing. Simpler initial UX. Yearly toggle (if we add it) would override to yearly. | ✓ |
| Yearly | Default to yearly. Shows 'save X%' messaging. More commitment, higher upfront cost. | |

**User's choice:** Monthly
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Redirect back to landing page with status (Recommended) | Lemon Squeezy redirects to landing page (?checkout=success|cancel). Show a toast/banner. | |
| Redirect to dashboard | After checkout, go to app dashboard. Better for post-purchase onboarding. | ✓ |
| Show confirmation page on landing page | Dedicated /pricing/thank-you page with plan details, next steps. | |

**User's choice:** Redirect to dashboard
**Notes:** None

---

## Monthly/Yearly Toggle

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, show a toggle (Recommended) | Add a toggle above plan cards: 'Monthly' / 'Yearly (save ~20%)'. Cards show the active cycle's price. | ✓ |
| No, monthly only | Simpler UI. Just show monthly prices. | |

**User's choice:** Yes, show a toggle
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Replace price text (Recommended) | Show '/mo' or '/yr' suffix on price based on toggle. Optionally add a 'Save ~20%' badge when yearly is selected. | ✓ |
| Show both prices always | Card shows '$6.99/mo or $69.99/yr'. No toggle — user sees both. | |
| Toggle with savings callout | Default to monthly with a 'Save 20% with yearly' badge. Toggle switches to yearly prices with '/yr' suffix. | |

**User's choice:** Replace price text
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Compute from API prices (Recommended) | Calculate savings dynamically from price_monthly vs price_yearly. Always accurate. | ✓ |
| Return savings from API | Add a savings_percent_yearly field to API response. | |
| Hardcode ~20% | Just show a static 'Save 20%' badge. Simplest, but drifts if pricing changes. | |

**User's choice:** Compute from API prices
**Notes:** None

---

## Post-Checkout UX

| Option | Description | Selected |
|--------|-------------|----------|
| Success toast + subscription section in settings (Recommended) | Show success toast on dashboard. Add Subscription section in settings. | |
| Welcome modal with plan details | Show modal on first login after subscribing. Plan name, features, limits, CTA. | ✓ |
| Banner on dashboard | Persistent banner: 'You're now on Hobby! 5 servers, 2 nodes.' Dismissable. | |

**User's choice:** Welcome modal with plan details
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Show current plan badge (Recommended) | For logged-in users with subscription, show 'Current plan' badge on matching card. 'Manage' opens portal. | ✓ |
| No change — landing page stays public | Landing page always shows subscribe buttons. Subscription management in dashboard only. | |

**User's choice:** Show current plan badge
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Lemon Squeezy Customer Portal (Recommended) | Backend POST /api/v1/billing/portal creates portal session. Users manage there. | ✓ |
| In-app subscription management | Build custom upgrade/downgrade/cancel UI in dashboard. | |

**User's choice:** Lemon Squeezy Customer Portal
**Notes:** None

---

## the agent's Discretion

- Spinner/loading component styling for pricing section
- Welcome modal design (content, CTA buttons, dismiss behavior)
- Toast messages for checkout success/cancel
- Plan badge positioning and styling on pricing cards
- usePlans hook vs inline fetch pattern
- API client module naming

## Deferred Ideas

- Enterprise plan pricing card — exists in DB but no landing page card. Needs features/limits content design.
- In-app plan management UI — delegated to Lemon Squeezy Customer Portal for now.
- Free plan subscription record — no subscription entity for free tier.
