---
phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w
plan: 02
subsystem: database
tags: [sql, migrations, billing, whop, postgres, sqlx]
requires:
  - phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w
    provides: existing plans and subscriptions table schemas
provides:
  - billing_products table for provider-agnostic product/price ID mapping
  - currency column on plans table
  - processed_webhook_events dedup table with UNIQUE(provider, event_id)
  - billing_events audit trail table with JSONB raw_payload/headers
  - subscription lifecycle fields (scheduled_plan_change_at, scheduled_plan_id, manage_url)
  - seed data for Whop product IDs (hobby + pro, monthly + yearly)
affects:
  - api infrastructure billing code (will reference billing_products)
  - subscription model (will use lifecycle fields)
  - webhook handlers (will use processed_webhook_events and billing_events)
tech-stack:
  added: []
  patterns:
    - Idempotent migrations with IF NOT EXISTS / ADD COLUMN IF NOT EXISTS
    - Seed data with WHERE NOT EXISTS for re-runnability
    - Sequel-style YYYYMMDDHHMMSS migration naming
key-files:
  created:
    - api/migrations/20260621000001_create_billing_products.sql
    - api/migrations/20260621000002_add_currency_to_plans.sql
    - api/migrations/20260621000003_seed_billing_products_whop.sql
    - api/migrations/20260621000004_create_processed_webhook_events.sql
    - api/migrations/20260621000005_create_billing_events.sql
    - api/migrations/20260621000006_add_subscription_lifecycle_fields.sql
  modified: []
key-decisions:
  - "Used separate migration files for each schema change (not one big migration) for clarity and reviewability"
  - "Seed billing_products with placeholder Whop IDs marked for replacement before production deployment"
  - "scheduled_plan_id FK references plans(id) for referential integrity on plan changes"
patterns-established:
  - "Indexes created alongside tables in the same migration for complete schema definition"
  - "All CREATE TABLE and ALTER TABLE ADD COLUMN statements use IF NOT EXISTS for idempotent re-runs"
requirements-completed:
  - D-04
  - D-09
  - D-20
  - D-24
  - D-27
duration: 5 min
completed: 2026-06-21
---

# Phase 91 Plan 02: Database Migrations Summary

**6 migration files for the Whop billing integration: billing_products table, processed_webhook_events dedup, billing_events audit trail, currency column, and subscription lifecycle fields — all with idempotent patterns.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-21T17:13:00Z
- **Completed:** 2026-06-21T17:18:14Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Created `billing_products` table (D-04) mapping plans → provider product/price IDs with FK to plans
- Added `currency` column to `plans` (D-24) with default 'USD' for at-rest currency tracking
- Seeded `billing_products` with 4 Whop product entries (hobby monthly/yearly, pro monthly/yearly) via idempotent migration (D-27)
- Created `processed_webhook_events` table (D-20) with UNIQUE(provider, event_id) for exactly-once webhook processing
- Created `billing_events` table (D-09) with JSONB `raw_payload` and `raw_headers` for webhook audit trail
- Added subscription lifecycle fields (D-22): `scheduled_plan_change_at`, `scheduled_plan_id` (FK→plans), and `manage_url` (Whop customer portal URL)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create billing_products migration + currency column + seed migration** - `a4db194` (feat)
2. **Task 2: Create processed_webhook_events + billing_events + lifecycle migrations** - `9a2d2ac` (feat)

**Plan metadata:** (committed per-task in api sub-repo)

## Files Created

- `api/migrations/20260621000001_create_billing_products.sql` - `billing_products` table with `plan_id` FK, provider, interval, product/price IDs, amounts, and UNIQUE(plan_id, provider, interval)
- `api/migrations/20260621000002_add_currency_to_plans.sql` - Adds `currency VARCHAR(3) NOT NULL DEFAULT 'USD'` to plans table
- `api/migrations/20260621000003_seed_billing_products_whop.sql` - Seeds 4 Whop product rows for hobby/pro plans with `WHERE NOT EXISTS` idempotency
- `api/migrations/20260621000004_create_processed_webhook_events.sql` - Webhook dedup table with UNIQUE(provider, event_id) constraint
- `api/migrations/20260621000005_create_billing_events.sql` - Audit trail table with JSONB `raw_payload` and `raw_headers` columns
- `api/migrations/20260621000006_add_subscription_lifecycle_fields.sql` - Adds `scheduled_plan_change_at`, `scheduled_plan_id` (FK), and `manage_url` to subscriptions

## Decisions Made

- **Separate migration files per concern** — Each table/column change has its own migration file rather than one large file. Easier to review, roll back, and reason about.
- **Placeholder Whop IDs with clear NOTE** — `prod_hobby_monthly` / `plan_hobby_monthly` etc. are marked as placeholders to be replaced with real Whop IDs before production deployment. This satisfies D-27 without requiring live Whop credentials during migration creation.
- **Indexes alongside tables** — Each CREATE TABLE migration includes relevant indexes (billing_products by plan_id/provider, processed_webhook_events by provider+event_id, billing_events by processing_status) for performance.
- **FK on scheduled_plan_id** — The `scheduled_plan_id` column references `plans(id)` for referential integrity when scheduling plan changes.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all migrations created cleanly and verified against acceptance criteria.

## User Setup Required

None - migration files are created and ready for sqlx tooling. No external service configuration required.

## Threat Notes

- **T-91-03 (Information Disclosure):** `billing_events.raw_payload` stores JSONB which may contain PII. Access control should be applied at the application layer when reading from this table.
- **T-91-04 (Tampering):** Seed migration uses placeholder Whop product IDs (`prod_*`/`plan_*`). These must be replaced with real Whop IDs before running against production.

## Next Phase Readiness

- All database schema artifacts for Whop billing integration are in place
- Ready for Plan 03 (API code changes that reference these tables/columns)
- Plan 05 (cleanup) will handle `DROP COLUMN` for legacy lemon_squeezy_variant_id fields

---

*Phase: 91-mengganti-code-pembayaran-dari-lemon-squeezy-menjadi-untuk-w*
*Plan: 02*
*Completed: 2026-06-21*

## Self-Check: PASSED

- ✓ All 6 migration files exist in `api/migrations/`
- ✓ 2 atomic commits in api sub-repo: `a4db194`, `9a2d2ac`
- ✓ SUMMARY.md committed to main repo: `11128e7`
- ✓ No modifications to shared orchestrator artifacts (STATE.md, ROADMAP.md)
