---
phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w
plan: 04
subsystem: payments
tags: whop, billing, config, container, env-vars, plan-model

# Dependency graph
requires:
  - phase: 91-03
    provides: WhopBillingService implementation, SubscriptionStatus enum, domain events
provides:
  - AppConfig with billing_provider + Whop credential fields and provider-aware validation
  - Container match expression selecting WhopBillingService or LemonSqueezyService by BILLING_PROVIDER
  - billing_webhook_secret sourced from correct provider config field
  - BILLING_PROVIDER, WHOP_API_KEY, WHOP_WEBHOOK_SECRET, WHOP_COMPANY_ID env vars in .env / .env.example
  - Plan model currency field defaulting to "USD" in all 4 constructors
affects: [91-05, 91-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Provider-based config validation via match on billing_provider
    - Fallback panic on unknown provider for deterministic startup

key-files:
  created: []
  modified:
    - api/src/config/app_config.rs
    - api/src/bootstrap/container.rs
    - api/src/domain/plan/model.rs
    - api/.env
    - api/.env.example

key-decisions:
  - "BILLING_PROVIDER defaults to 'lemon_squeezy' for backward compatibility during migration"
  - "Unknown provider panics at startup rather than silently falling back (T-91-09)"
  - "billing_webhook_secret dynamically selected from correct provider config field"
  - "Currency defaults to USD across all plan tiers"

patterns-established: []

requirements-completed:
  - D-10
  - D-11
  - D-12
  - D-19

# Metrics
duration: 4 min
completed: 2026-06-21
---

# Phase 91 Plan 04: Whop Configuration & Container Wiring Summary

**BILLING_PROVIDER env var, Whop API credential fields, container match expression selecting billing service by provider, env var documentation, and Plan model currency field**

## Performance

- **Duration:** 4 min
- **Started:** 2026-06-21T17:55:00Z
- **Completed:** 2026-06-21T17:59:20Z
- **Tasks:** 4 (3 committable + 1 gitignored)
- **Files modified:** 3 tracked + 2 gitignored (.env, .env.example)

## Accomplishments

- Added `billing_provider`, `whop_api_key`, `whop_webhook_secret`, `whop_company_id` fields to AppConfig struct
- Updated `validate()` to match on `billing_provider` — provider-specific webhook secret requirement + API key warnings
- Container.rs now constructs `WhopBillingService` or `LemonSqueezyService` via `match config.billing_provider.as_str()`
- `billing_webhook_secret` dynamically reads from correct provider's config field
- Added `BILLING_PROVIDER`, `WHOP_API_KEY`, `WHOP_WEBHOOK_SECRET`, `WHOP_COMPANY_ID` to `.env` and `.env.example`
- Added `currency: String` to `Plan` struct with `"USD"` default in all 4 constructors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Whop config fields to AppConfig + update validate()** - `c90b59d` (feat)
2. **Task 2: Update container.rs to match on BILLING_PROVIDER** - `60bdfc7` (feat)
3. **Task 3: Add Whop env vars to .env and .env.example** (gitignored — not committed)
4. **Task 4: Add currency field to Plan model** - `9ff8c45` (feat)

## Files Created/Modified

- `api/src/config/app_config.rs` - Added billing_provider + whop credential fields, updated validate() with match
- `api/src/bootstrap/container.rs` - Added WhopBillingService import, match-based billing service construction, dynamic webhook_secret, fallback default fields
- `api/src/domain/plan/model.rs` - Added currency: String field to Plan struct + all 4 constructors
- `api/.env` - Added BILLING_PROVIDER, WHOP_API_KEY, WHOP_WEBHOOK_SECRET, WHOP_COMPANY_ID (gitignored)
- `api/.env.example` - Same vars with example values (gitignored)

## Decisions Made

- BILLING_PROVIDER defaults to `"lemon_squeezy"` for backward compatibility during migration
- Unknown provider triggers `panic!` at startup rather than silent fallback (per T-91-09 threat mitigation)
- `billing_webhook_secret` is dynamically selected from the correct provider's config field via match
- Plan currency defaults to `"USD"` across all four tiers

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `.env` and `.env.example` are gitignored by `api/.gitignore` pattern. Changes written to disk but not tracked in git. This is expected for local config files.

## Verification Results

- `cargo check --package backend` passes (0 errors, pre-existing warnings only)
- `app_config.rs`: billing_provider + all 4 whop fields present, validate() matches on billing_provider
- `container.rs`: WhopBillingService import + match-based construction, dynamic billing_webhook_secret
- `.env`: BILLING_PROVIDER=whop + WHOP_* vars present
- `plan/model.rs`: currency field in struct + all 4 constructors set "USD"

## User Setup Required

None - no external service configuration required. Env vars added to local .env files.

## Next Phase Readiness

- All wire-up for WhopBillingService selection is complete
- Config validation for both providers in place
- Plan model has multi-currency readiness
- Ready for Plan 05 (Whop webhook handler implementation) or Plan 06 (subscription lifecycle)

---
*Phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w*
*Completed: 2026-06-21*
