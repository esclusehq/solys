---
phase: 66-integrasikan-umami-analitycs-dashboard-dengan-rds-di-project
plan: 02
subsystem: frontend
tags: umami, analytics, tracking, javascript
---

# Phase 66: Integrasikan Umami Analytics Dashboard — Plan 2 Summary

**Injected Umami tracking scripts into both frontend properties' index.html files**

## Performance

- **Duration:** ~1 min
- **Completed:** 2026-05-31
- **Tasks:** 2 (1 already done, 1 applied)

## Accomplishments

- Verified `landing-page-escluse/index.html` already had deferred Umami tracking script with `analytics.js`, `data-website-id="REPLACE_WITH_LANDING_PAGE_WEBSITE_ID"`, and `data-domains="esluce.com"`
- Added deferred Umami tracking script to `app/index.html` with `analytics.js`, `data-website-id="REPLACE_WITH_APP_WEBSITE_ID"`, and `data-domains="app.esluce.com"`
- Both scripts use `defer` attribute (non-blocking page rendering)
- Both use renamed `analytics.js` path for ad blocker evasion
- Both have distinct placeholder website IDs for deployer to replace after Umami dashboard setup

## Acceptance Criteria

1. ✅ landing-page-escluse/index.html — deferred Umami script with analytics.js, data-domains="esluce.com" (pre-existing)
2. ✅ app/index.html — deferred Umami script with analytics.js, data-domains="app.esluce.com"
3. ✅ Both use `defer` for non-blocking load
4. ✅ Both use `analytics.js` path (ad blocker evasion)
5. ✅ D-05 (track all subdomains) — implemented

## Decisions Made

- Landing page (`esluce.com`) and app dashboard (`app.esluce.com`) get separate data-website-IDs for independent tracking
- Both scripts loaded deferred in `<head>` to avoid render blocking
- Placeholder IDs used — deployer replaces after Umami first login

## Next Steps

Phase 66 is complete. All Umami infrastructure (Plan 1) and frontend tracking (Plan 2) are done. Remaining steps are manual deployment per DEPLOYMENT.md.
