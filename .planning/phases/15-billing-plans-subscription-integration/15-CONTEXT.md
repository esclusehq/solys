# Phase 15: Billing Plans Subscription Integration - Context

**Gathered:** 2026-04-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable users to subscribe to Starter or Pro plans via Lemon Squeezy checkout, track their subscription status, and enforce plan limits when creating servers.
</domain>

<decisions>
## Implementation Decisions

### Subscription Limits Enforcement
- **D-01:** Enforce subscription limits at server creation - block creation if user exceeds plan limits (max_servers, max_ram_mb, max_cpu_cores, max_disk_gb, max_bandwidth_gb)

### Subscription Upgrade Processing
- **D-02:** Process subscription upgrades automatically via Lemon Squeezy webhook - when payment succeeds, update user's subscription in database based on webhook data
- **D-02a:** Create webhook handler for subscription_created, subscription_updated, subscription_cancelled events

### Plan Display & Upgrade Flow
- **D-03:** Show current subscription plan in billing page
- **D-04:** Display upgrade options (Starter $6.99/month, Pro $19.99/month) with Subscribe buttons
- **D-05:** Redirect to Lemon Squeezy checkout on Subscribe click
- **D-06:** Handle checkout success/cancel redirects to dashboard

### Subscription Tracking
- **D-07:** Track user's current plan in user or subscription table
- **D-08:** Display subscription status in dashboard (e.g., "Starter plan", "Pro plan")
- **D-09:** Store Lemon Squeezy customer_id and subscription_id for webhook matching

### Agent's Discretion
- Database schema details for subscription tracking (which table, which columns)
- Webhook signature verification implementation details
- Exact limit check logic (how to query current usage vs plan limits)

</decisions>

<canonical_refs>
## Canonical References

### Existing Code
- `api/src/infrastructure/billing/lemon_squeezy_service.rs` — Lemon Squeezy checkout API integration
- `api/src/presentation/handlers/billing_handlers.rs` — Billing API endpoints (create checkout, list plans)
- `app/src/pages/billing/BillingPage.jsx` — Frontend billing page with plan display
- `app/src/pages/dashboard/DashboardPage.jsx` — Dashboard with checkout success handling

### Database
- `api/migrations/20260324000004_create_plans_table.sql` — Plans table schema
- `api/migrations/20260324000005_create_subscriptions_table.sql` — Subscriptions table (if exists)

### External
- Lemon Squeezy API docs: https://docs.lemonsqueezy.com/api/checkouts/create-checkout

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- LemonSqueezyService already implements create_checkout_session
- BillingPage already displays plans and handles checkout redirect
- DashboardPage already handles checkout=success query parameter

### Established Patterns
- API uses ApiResponse wrapper for JSON responses
- Frontend uses Zustand for state management
- WebSocket for real-time updates

### Integration Points
- New: Subscription table needs user_id, plan_id, status, lemon_squeezy_subscription_id, customer_id
- New: Webhook endpoint at /api/v1/billing/webhook
- New: Limit check in server creation handler

</code_context>

<specifics>
## Specific Ideas

- Lemon Squeezy variant IDs: Starter=1490734, Pro=1517243
- Default APP_URL: http://127.0.0.1:5173
- Checkout redirects to /dashboard?checkout=success or /dashboard?checkout=cancelled
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope
</deferred>

---

*Phase: 15-billing-plans-subscription-integration*
*Context gathered: 2026-04-11*