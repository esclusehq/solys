---
phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w
plan: 01
subsystem: billing
tags: [rust, whop, billing, payment, refactoring]

# Dependency graph
requires: []
provides:
  - Refactored BillingService trait with BillingCycle enum (plan_id + billing_cycle instead of variant_id)
  - WhopBillingService implementing BillingService trait
  - BillingProvider enum (Whop, LemonSqueezy, Stripe) with Display/FromStr
  - MockBillingService with canned responses for CI/testing
  - Updated module exports in infrastructure/billing/mod.rs and domain/billing/mod.rs
affects: [billing handlers, container wiring, AppConfig, subscription model]

# Tech tracking
tech-stack:
  added: []
  patterns: [BillingService trait pattern, BillingProvider enum with Display/FromStr]

key-files:
  created:
    - api/src/infrastructure/billing/whop_service.rs
    - api/src/infrastructure/billing/mock_billing_service.rs
    - api/src/domain/billing/provider.rs
  modified:
    - api/src/infrastructure/billing/billing_trait.rs
    - api/src/infrastructure/billing/mod.rs
    - api/src/domain/billing/mod.rs

key-decisions:
  - "BillingCycle enum (Monthly/Yearly) added above BillingService trait with as_str()/from_str() methods"
  - "create_checkout_session now accepts plan_id: Uuid + billing_cycle: BillingCycle instead of variant_id: String"
  - "WhopBillingService uses reqwest with Bearer token auth, API base https://api.whop.com/api/v1"
  - "WhopBillingService.create_portal_session returns Error — Whop provides manage_url natively"
  - "MockBillingService gated behind #[cfg(test)] for CI-safe unit testing"
  - "BillingProvider enum stored as Display string in DB, reconstructed via FromStr (D-17)"
  - "Existing LemonSqueezyService kept during transition — removed in Plan 05 cleanup"

patterns-established:
  - "BillingProvider enum: multi-variant provider selection with Display/FromStr round-trip"
  - "BillingService impl: provider-agnostic trait pattern with internal plan_id + billing_cycle"

requirements-completed:
  - D-01
  - D-02
  - D-03
  - D-07
  - D-16
  - D-17
  - D-26

# Metrics
duration: 12min
completed: 2026-06-22
---

# Phase 91: Mengganti Code Pembayaran dari Lemon Squeezy Menjadi untuk Whop — Plan 01 Summary

**Foundational Whop billing infrastructure: refactored BillingService trait with BillingCycle enum, WhopBillingService implementation, BillingProvider enum, and MockBillingService for CI**

## Performance

- **Duration:** 12 min
- **Started:** 2026-06-22T17:30:00Z
- **Completed:** 2026-06-22T17:42:00Z
- **Tasks:** 4
- **Files modified:** 6 (3 created, 3 modified)

## Accomplishments

- Refactored BillingService trait: `create_checkout_session` now accepts `plan_id: Uuid` + `billing_cycle: BillingCycle` instead of `variant_id: String` (D-02, D-03)
- Added `BillingCycle` enum with `Monthly`/`Yearly` variants and `as_str()`/`from_str()` helper methods
- Created `BillingProvider` enum (Whop, LemonSqueezy, Stripe) with `Display`/`FromStr` round-trip (D-07, D-17)
- Created `WhopBillingService` implementing all 6 `BillingService` trait methods with Whop API semantics (D-01)
- Created `MockBillingService` with canned responses for all trait methods, gated behind `#[cfg(test)]` (D-26)
- Updated `infrastructure/billing/mod.rs` to export `BillingCycle`, `WhopBillingService`, and `MockBillingService` (D-16)
- Updated `domain/billing/mod.rs` to export `BillingProvider`

## Task Commits

Each task was committed atomically in the `api/` sub-repo:

1. **Task 1: Refactor BillingService trait + create BillingCycle enum** - `e342c76` (feat)
2. **Task 2: Create BillingProvider enum + update domain billing mod.rs** - `c8a1b21` (feat)
3. **Task 3: Create WhopBillingService implementation** - `72c5a0f` (feat)
4. **Task 4: Create MockBillingService + update billing module exports** - `f7eb5fe` (feat)

## Files Created/Modified

- `api/src/infrastructure/billing/billing_trait.rs` - Refactored: added BillingCycle enum, changed `create_checkout_session` signature to `plan_id: Uuid` + `billing_cycle: BillingCycle`
- `api/src/infrastructure/billing/whop_service.rs` - New: WhopBillingService with full trait implementation (reqwest HTTP client, Whop API semantics)
- `api/src/infrastructure/billing/mock_billing_service.rs` - New: MockBillingService with canned responses for testing
- `api/src/infrastructure/billing/mod.rs` - Updated: added exports for BillingCycle, WhopBillingService, MockBillingService
- `api/src/domain/billing/provider.rs` - New: BillingProvider enum (Whop, LemonSqueezy, Stripe) with Display/FromStr
- `api/src/domain/billing/mod.rs` - Updated: added `pub mod provider;` and `pub use provider::BillingProvider;`

## Decisions Made

- Followed plan exactly as specified — no deviations from planned implementation
- BillingCycle defined in billing_trait.rs above the trait since both are tightly coupled
- English doc comments for consistency with existing codebase
- MockBillingService gated behind `#[cfg(test)]` so it never ships to production
- Kept `lemon_squeezy_service` and `stripe_service` modules and exports for transition period

## Must-Haves Verification

| Artifact | Min Lines | Actual | Status |
|----------|-----------|--------|--------|
| billing_trait.rs | 45 | 66 | ✅ |
| whop_service.rs | 200 | 212 | ✅ |
| mock_billing_service.rs | 80 | 80 | ✅ |
| provider.rs | 30 | 32 | ✅ |

| Key Link | Pattern | Status |
|----------|---------|--------|
| whop_service.rs → billing_trait.rs via impl | `impl BillingService for WhopBillingService` | ✅ |
| provider.rs → domain/billing/mod.rs via pub use | `pub use.*BillingProvider` | ✅ |
| whop_service.rs → billing_products via plan_id | `plan_id.*Uuid` | ✅ |

### Truths
| Truth | Status |
|-------|--------|
| WhopBillingService compiles as implementation of BillingService trait | ✅ |
| BillingService trait uses BillingCycle + plan_id instead of variant_id | ✅ |
| BillingProvider enum round-trips via Display/FromStr | ✅ |
| MockBillingService provides canned responses for all methods | ✅ |
| mod.rs exports WhopBillingService and MockBillingService | ✅ |
| domain/billing/mod.rs exports BillingProvider | ✅ |

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- `cargo check --package backend` produces compilation errors in pre-existing files (`lemon_squeezy_service.rs` and `billing_handlers.rs`) that use the old `create_checkout_session` signature. This is **expected** per the plan — these files will be updated in subsequent plans (Plan 03 handles billing_handlers, Plan 05 cleans up LS).

## Next Phase Readiness

- All foundational types (BillingCycle, BillingProvider, WhopBillingService, MockBillingService) are in place
- Ready for Plan 02: Database migrations (billing_products, processed_webhook_events, billing_events tables)
- Downstream plans can import BillingService with new signature, BillingProvider, BillingCycle
- Existing code using old variant_id signature will need updating in subsequent plans

## Self-Check: PASSED

- [x] billing_trait.rs - refactored with BillingCycle enum + plan_id/billing_cycle signature (66 lines ≥ 45)
- [x] whop_service.rs - WhopBillingService with all 6 trait methods (212 lines ≥ 200)
- [x] mock_billing_service.rs - MockBillingService with canned responses (80 lines ≥ 80)
- [x] provider.rs - BillingProvider enum with Display/FromStr (32 lines ≥ 30)
- [x] billing/mod.rs - exports BillingCycle, WhopBillingService, MockBillingService
- [x] domain/billing/mod.rs - exports BillingProvider
- [x] Task commits present: e342c76, c8a1b21, 72c5a0f, f7eb5fe

---
*Phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w*
*Completed: 2026-06-22*
