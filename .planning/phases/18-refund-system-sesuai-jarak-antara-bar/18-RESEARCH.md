# Phase 18: Refund System - Research

**Researched:** 2026-04-18
**Status:** Ready for planning

## Domain Understanding

### What This Phase Delivers
A refund system that calculates refund eligibility based on the time elapsed between when a user subscribed and when they request a refund. This is a common SaaS pattern - typically full refund within 7 days, prorated refund within 30 days.

### Key Technical Components

1. **Refund Policy Calculation**
   - Time-based eligibility: compare `current_period_start` vs refund request timestamp
   - Full refund window: typically 7 days from subscription start
   - Prorated refund window: typically 7-30 days
   - No refund after 30 days

2. **Database Schema Requirements**
   - Existing `subscriptions` table has `current_period_start` - key field for calculation
   - May need `refunds` table to track refund requests
   - Need to track: refund_amount, refund_reason, processed_at, status

3. **Lemon Squeezy Integration**
   - LS supports refunds via API: https://docs.lemonsqueezy.com/api/refunds
   - Need to check if refund is supported on the subscription or order
   - May need to cancel subscription after refund

## Existing Code Analysis

### Subscription Service (`api/src/infrastructure/`)
- `current_period_start` stored in subscriptions table - can calculate days since subscription
- Subscription status handling exists (active, cancelled, etc.)
- Need to add refund request handling

### Billing Handlers (`api/src/presentation/handlers/billing_handlers.rs`)
- Existing webhook handling for subscription events
- Could extend to handle refund events from Lemon Squeezy
- No current refund endpoints

### Lemon Squeezy Service (`api/src/infrastructure/billing/`)
- Already handles checkout and webhooks
- Would need to add refund API call capability
- Check LS API for: /v1/refunds endpoint

## Technical Patterns

### Refund Eligibility Calculation (Rust)
```rust
fn calculate_refund_eligibility(current_period_start: DateTime<Utc>, request_time: DateTime<Utc>) -> RefundEligibility {
    let days_elapsed = (request_time - current_period_start).num_days();
    
    match days_elapsed {
        0..=7 => RefundEligibility::Full,
        8..=30 => RefundEligibility::Prorated,
        _ => RefundEligibility::None,
    }
}
```

### Prorated Refund Calculation
```rust
fn calculate_prorated_amount(plan_price: f64, days_elapsed: i64, total_days: i64) -> f64 {
    let remaining_days = total_days - days_elapsed;
    let daily_rate = plan_price / total_days as f64;
    daily_rate * remaining_days as f64
}
```

### Database Schema for Refunds
```sql
CREATE TABLE IF NOT EXISTS refunds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id),
    amount_cents INTEGER NOT NULL,
    refund_type VARCHAR(20) NOT NULL, -- 'full', 'prorated', 'none'
    status VARCHAR(20) NOT NULL, -- 'pending', 'processed', 'rejected'
    reason TEXT,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Implementation Considerations

### API Endpoints Needed
- `POST /api/v1/billing/refund` - Request refund
- `GET /api/v1/billing/refunds` - List user's refunds
- `GET /api/v1/billing/refund/eligibility` - Check eligibility without requesting

### Frontend Integration
- Add refund button to billing page
- Show eligibility status before requesting
- Display refund history

### Edge Cases
- Multiple refunds - only one refund per subscription
- Partial month - calculate based on days
- Already cancelled subscription - check if within refund window
- Trial period - different refund rules

---

*Research complete - ready for planning*