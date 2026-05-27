# Phase 15: Billing Plans Subscription Integration - Research

**Researched:** 2026-04-11
**Status:** Ready for planning

## Domain Understanding

### What This Phase Delivers
Enable users to subscribe to Starter or Pro plans via Lemon Squeezy checkout, track their subscription status, and enforce plan limits when creating servers.

### Key Technical Components

1. **Lemon Squeezy Webhook Security**
   - Uses HMAC-SHA256 signature verification via `X-Signature` header
   - Signing secret configured in Lemon Squeezy dashboard
   - Signature is hex-encoded, must use `timingSafeEqual` to prevent timing attacks
   - Raw body required for signature calculation (not parsed JSON)

2. **Subscription Limit Enforcement**
   - Check existing server usage against plan limits before creation
   - Plans table contains `limits` JSONB with: max_servers, max_ram_mb, max_cpu_cores, max_disk_gb, max_bandwidth_gb
   - Subscription table links users to plans
   - Current implementation in server_handlers.rs has quota check COMMENTED OUT

3. **Database Schema**
   - `plans` table: name, display_name, price_monthly, limits JSONB, features JSONB
   - `subscriptions` table: user_id (unique), plan_id, status, provider_subscription_id
   - Default plans: free (0), starter (5), pro (15), enterprise (50)

## Existing Code Analysis

### LemonSqueezyService (`api/src/infrastructure/billing/lemon_squeezy_service.rs`)
- ✅ create_checkout_session: Working with custom_data for user_id
- ⚠️ parse_webhook_event: Has TODO comment for signature verification
- Portal session creation exists

### BillingHandlers (`api/src/presentation/handlers/billing_handlers.rs`)
- ✅ /plans GET: Lists active plans
- ✅ /checkout POST: Creates checkout session
- ✅ /webhook POST: Receives webhooks (partial implementation)
- ⚠️ Webhook handler processes events but doesn't verify signature

### Server Handlers (`api/src/presentation/handlers/server_handlers.rs`)
- ⚠️ create_server has quota check COMMENTED OUT (lines 189-203)
- Uses SqlxServerRepository for persistence
- No integration with subscription limits

### Frontend (`app/src/pages/billing/BillingPage.jsx`)
- ✅ Displays plans (filtered for starter, pro)
- ✅ Handles checkout redirect
- ❌ Doesn't show current subscription status
- ❌ No display of user's current plan/limits

## Technical Patterns

### Webhook Signature Verification (Rust)
```rust
use std::io::Read;

fn verify_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    use std::io::Write;
    
    let mut mac = hmac_sha256::HMAC::new(secret.as_bytes());
    mac.write_all(payload).unwrap();
    let result = mac.finalize();
    
    let expected = hex::decode(signature).unwrap();
    result[..] == expected[..]
}
```
Note: Need `hmac` or `sha2` crate for Rust implementation.

### Plan Limit Enforcement Pattern
```rust
async fn check_quota(pool: &PgPool, user_id: Uuid, requested: &CreateServerRequest) -> Result<QuotaCheck> {
    // 1. Get user's subscription with plan
    // 2. Parse plan limits JSONB
    // 3. Count user's existing servers
    // 4. Sum used resources (RAM, CPU, disk)
    // 5. Compare against limits
}
```

## Implementation Considerations

### Security (Priority: High)
- Webhook signature verification is MANDATORY for production
- Use constant-time comparison to prevent timing attacks
- Store webhook secret in environment variable, not hardcoded
- Log verification failures but don't expose details

### Database Changes
- Existing schema supports the required fields
- May need migration to add lemon_squeezy_customer_id to subscriptions
- Plans table already seeded with limits JSONB

### Edge Cases
- User with no subscription (free tier)
- Subscription expired/cancelled mid-period
- Multiple checkouts (handle idempotency)
- Failed payment handling

## Validation Architecture

### Test Scenarios
1. Webhook with valid signature → 200 OK
2. Webhook with invalid signature → 400 + logged
3. Server creation with active subscription → allowed
4. Server creation exceeding limits → 403 with details
5. Server creation with no subscription → use free tier limits

### Integration Points
- server_handlers.rs → subscription service
- billing_handlers.rs → LemonSqueezyService
- BillingPage → /api/v1/billing/plans (existing)
- Need: BillingPage → get current subscription status

---

*Research complete - ready for planning*