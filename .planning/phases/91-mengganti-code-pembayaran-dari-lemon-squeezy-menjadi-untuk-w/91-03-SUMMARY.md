---
phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w
plan: 03
subsystem: payments
tags: whop, subscription, webhook, hmac, domain-events, billing
requires:
  - phase: 91-01
    provides: BillingProvider enum, BillingCycle enum, whop_service.rs skeleton
  - phase: 91-02
    provides: Database migrations for billing_products, billing_events, processed_webhook_events, subscription lifecycle fields
provides:
  - SubscriptionStatus enum with Display/FromStr/is_active
  - Subscription model with scheduled_plan_change_at, scheduled_plan_id, manage_url fields
  - SubscriptionDomainEvent enum with 5 variants + to_payload()
  - Whop webhook endpoint with Standard Webhooks HMAC-SHA256 verification
  - Dedup via processed_webhook_events table
  - Audit trail via billing_events table
  - Refactored handle_checkout_completed using BillingProvider enum
  - Refactored create_checkout using plan.id (Uuid) + BillingCycle enum
affects: frontend (BillingPage.jsx), future cleanup plans

tech-stack:
  added: hmac (reuse), base64 (reuse), sha2 (reuse), subtle (reuse)
  patterns: Standard Webhooks signature verification, domain event emission, BillingProvider enum usage

key-files:
  created:
    - api/src/domain/billing/events.rs — SubscriptionDomainEvent enum with to_payload()
  modified:
    - api/src/domain/subscription/model.rs — Added SubscriptionStatus enum + 3 new fields
    - api/src/domain/billing/mod.rs — Added events module + re-export
    - api/src/domain/billing/service.rs — Refactored to use BillingProvider enum
    - api/src/presentation/handlers/billing_handlers.rs — Added whop_webhook handler + refactored checkout
    - api/src/infrastructure/billing/lemon_squeezy_service.rs — Updated trait signature + Subscription fields
    - api/src/infrastructure/billing/whop_service.rs — Added missing Subscription fields
    - api/src/presentation/handlers/subscription_handlers.rs — Verified is_active() delegation

key-decisions:
  - SubscriptionStatus stored as string in DB to avoid migration; enum used only in Rust for type safety
  - Standard Webhooks verification implemented directly in handler (not delegated to parse_webhook_event)
  - manage_url stored on subscription record during membership.activated webhook processing
  - create_checkout now uses plan.id (Uuid) + BillingCycle enum instead of LS variant IDs
  - Dedup check returns 200 with duplicate:true instead of 409 to match webhook expectations

patterns-established:
  - Domain event pattern: SubscriptionDomainEvent enum with to_payload() mirroring BillingEvent pattern
  - Standard Webhooks: HMAC-SHA256 with base64-decoded secret, signed_content = "{id}.{timestamp}.{body}"
  - Provider enum usage: BillingProvider passed to domain service methods instead of hardcoded strings

requirements-completed: [D-06, D-08, D-18, D-21, D-22, D-23, D-25]
---
# Phase 91 Plan 03: Whop Webhook Endpoint, SubscriptionStatus Enum, Domain Events & Checkout Refactor

**Whop webhook endpoint with Standard Webhooks HMAC-SHA256 verification, SubscriptionStatus enum with Display/FromStr, domain events for billing lifecycle, and checkout refactored to use BillingProvider enum + BillingCycle**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-06-21T17:25:06Z
- **Completed:** 2026-06-21T18:00:00Z
- **Tasks:** 4
- **Files modified:** 8

## Accomplishments
- Added `SubscriptionStatus` enum (Trialing, Active, PastDue, Canceled, Expired) with Display, FromStr, and is_active() extending to Subscription model
- Added 3 new lifecycle fields to Subscription: `scheduled_plan_change_at`, `scheduled_plan_id`, `manage_url`
- Created `SubscriptionDomainEvent` enum with 5 variants (SubscriptionActivated, SubscriptionCanceled, PlanChanged, PaymentFailed, SubscriptionExpired) and to_payload()
- Added `whop_webhook` handler implementing full Standard Webhooks verification (HMAC-SHA256, timestamp validation, dedup, audit trail)
- Refactored `handle_checkout_completed` to accept `BillingProvider` enum instead of hardcoded "stripe" string
- Refactored `create_checkout` to pass `plan.id` (Uuid) + `BillingCycle` enum instead of LS variant IDs
- Registered `/api/billing/webhook/whop` route in billing router
- Updated error messages in `create_portal` and `cancel_subscription` from "Lemon Squeezy" to "Billing"

## Task Commits

Each task was committed atomically:

1. **Task 1: SubscriptionStatus enum + lifecycle fields** - `35fb150` (feat)
2. **Task 2: Domain events for subscription lifecycle** - `ec46f21` (feat)
3. **Task 3: Service refactor + Whop webhook handler** - `305dda5` (feat)
4. **Task 4: Verify subscription_handlers delegation** - `bfcc6bd` (docs)
5. **Compilation fixes** - `c62a930` (fix)

## Files Created/Modified
- `api/src/domain/subscription/model.rs` - **Modified**: Added SubscriptionStatus enum, 3 new fields (scheduled_plan_change_at, scheduled_plan_id, manage_url), updated is_active()
- `api/src/domain/billing/events.rs` - **Created**: SubscriptionDomainEvent enum with 5 variants + to_payload()
- `api/src/domain/billing/mod.rs` - **Modified**: Added events module + SubscriptionDomainEvent re-export
- `api/src/domain/billing/service.rs` - **Modified**: Refactored handle_checkout_completed to use BillingProvider enum
- `api/src/presentation/handlers/billing_handlers.rs` - **Modified**: Added whop_webhook handler (168 lines), refactored create_checkout, updated error messages
- `api/src/infrastructure/billing/lemon_squeezy_service.rs` - **Modified**: Updated create_checkout_session to match refactored trait, added Subscription fields
- `api/src/infrastructure/billing/whop_service.rs` - **Modified**: Added missing Subscription fields
- `api/src/presentation/handlers/subscription_handlers.rs` - **Verified**: is_active() delegation confirmed

## Decisions Made
- **SubscriptionStatus stored as string in DB**: Using String on the Subscription model maintains DB compatibility. The enum is used only in Rust code for type-safe business logic.
- **Standard Webhooks verification in handler**: Instead of delegating to `parse_webhook_event()`, the full verification flow runs in the `whop_webhook` handler for clarity and direct error reporting.
- **manage_url storage**: The `manage_url` from Whop memberships is stored on the subscription record during `membership.activated` webhook processing, avoiding the need for a separate portal session API call.
- **Dedup returns 200**: Duplicate `webhook-id` values return `200 OK` with `duplicate: true` flag, matching webhook best practices (idempotent retries don't cause errors).
- **BillingProvider enum in domain service**: Passed as a 5th parameter to `handle_checkout_completed`, enabling type-safe provider identification without magic strings.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] LemonSqueezyService create_checkout_session trait mismatch**
- **Found during:** Task 3 (cargo check verification)
- **Issue:** The refactored `BillingService` trait now accepts `plan_id: Uuid, billing_cycle: BillingCycle` instead of `variant_id: String`. The existing LS service wasn't updated to match.
- **Fix:** Updated trait signature and body to accept new params with placeholders (LS is pending removal in a later plan)
- **Files modified:** api/src/infrastructure/billing/lemon_squeezy_service.rs
- **Verification:** cargo check passes
- **Committed in:** c62a930 (fix commit — combined with other fixes)

**2. [Rule 1 - Bug] Missing Subscription fields in service constructors**
- **Found during:** Task 3 (cargo check verification)
- **Issue:** LemonSqueezyService and WhopBillingService construct Subscription structs without the new `scheduled_plan_change_at`, `scheduled_plan_id`, `manage_url` fields
- **Fix:** Added all 3 fields as `None` / `manage_url` (from local variable) in both constructors
- **Files modified:** api/src/infrastructure/billing/lemon_squeezy_service.rs, api/src/infrastructure/billing/whop_service.rs
- **Verification:** cargo check passes
- **Committed in:** c62a930

**3. [Rule 1 - Bug] Missing BillingProvider argument in LS webhook handler**
- **Found during:** Task 3 (cargo check verification)
- **Issue:** The existing LS `billing_webhook` handler calls `handle_checkout_completed` without the new 5th `BillingProvider` argument
- **Fix:** Added `BillingProvider::LemonSqueezy` as the 5th argument
- **Files modified:** api/src/presentation/handlers/billing_handlers.rs
- **Verification:** cargo check passes
- **Committed in:** c62a930

**4. [Rule 1 - Bug] Missing base64::Engine and hmac::Mac imports in whop_webhook**
- **Found during:** Task 3 (cargo check verification)
- **Issue:** The whop_webhook handler uses `base64::engine::general_purpose::STANDARD.decode()` and `hmac::Hmac::new_from_slice()` without having `base64::Engine` and `hmac::Mac` traits in scope
- **Fix:** Added module-level imports for both traits
- **Files modified:** api/src/presentation/handlers/billing_handlers.rs
- **Verification:** cargo check passes
- **Committed in:** c62a930

**5. [Rule 1 - Bug] Sqlx queries with bare `?` in whop_webhook return Result<_, sqlx::Error>**
- **Found during:** Task 3 (cargo check verification)
- **Issue:** Two sqlx queries inside `payment.succeeded` and `membership.activated` arms used `?` directly, but `ApiError` doesn't implement `From<sqlx::Error>`
- **Fix:** Replaced bare `?` with `.map_err(|e| ApiError::new("DB_ERROR", &e.to_string()))?`
- **Files modified:** api/src/presentation/handlers/billing_handlers.rs
- **Verification:** cargo check passes
- **Committed in:** c62a930

---

**Total deviations:** 5 auto-fixed (5 Rule 1 - Bug)
**Impact on plan:** All deviations were necessary to fix compilation errors introduced by refactoring. No scope creep.

## Issues Encountered
- The main repo's `.gitignore` excludes the `api/` directory (separate git sub-repo at `api/.git`). All commits were made to the api sub-repo.
- Several pre-existing compilation issues surfaced because the `BillingService` trait had already been refactored in Plan 01 but downstream consumers (LemonSqueezyService, existing webhook handler) hadn't been updated. These were all fixed as part of the auto-fix pass.

## Next Phase Readiness
- Ready for Wave 3: WhopBillingService trait implementation and container wiring
- Subscription model now has all lifecycle fields needed for upgrade/downgrade scheduling
- Webhook handler ready for Whop sandbox testing once Whop service is fully wired

---

## Self-Check: PASSED

- [x] `api/src/domain/billing/events.rs` — FOUND
- [x] `api/src/domain/subscription/model.rs` — FOUND
- [x] `SUMMARY.md` — FOUND
- [x] Commit `35fb150` — FOUND in api subrepo
- [x] Commit `ec46f21` — FOUND in api subrepo
- [x] Commit `305dda5` — FOUND in api subrepo
- [x] Commit `bfcc6bd` — FOUND in api subrepo
- [x] Commit `c62a930` — FOUND in api subrepo
- [x] Commit `120d67a` — FOUND in main repo
- [x] `cargo check --package backend` — passes (no errors, 82 pre-existing warnings)

---

*Phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w*
*Completed: 2026-06-21*
