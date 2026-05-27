---
status: root-cause-found
trigger: "CORS error on Google OAuth login from landing page (esluce.com → api.esluce.com)"
created: 2026-05-16
updated: 2026-05-16
---

## Symptoms
- Expected: Google OAuth login completes and user is authenticated
- Actual: CORS error blocks the POST to `https://api.esluce.com/api/v1/auth/oauth`
- Errors: `CORS Missing Allow Origin`, `Status code: 502`, `AxiosError: Network Error`
- Timeline: Occurs on every Google OAuth attempt from esluce.com landing page
- Reproduction: Click "Sign in with Google" on esluce.com, complete Google OAuth, get redirected back

## Current Focus
- hypothesis: Invalid CORS config in Rust backend (allow_origin(Any) + allow_credentials(true)) violates CORS spec, and 502 from Caddy suggests backend may not handle OPTIONS preflight
- test: Fix CORS config to use explicit origins instead of Any when credentials are included
- expecting: OAuth callback POST succeeds without CORS errors
- next_action: Apply fix to backend CORS config and optionally add CORS headers in Caddy

## Evidence
- timestamp: 2026-05-16 browser error shows `CORS header 'Access-Control-Allow-Origin' missing` with status 502
- timestamp: 2026-05-16 `api/src/bootstrap/mod.rs:103-108` uses `CorsLayer::new().allow_origin(Any).allow_credentials(true)` — violated CORS spec
- timestamp: 2026-05-16 `gateway/Caddyfile.prod:32-35` has NO CORS headers for api.esluce.com, only reverse_proxy
- timestamp: 2026-05-16 Landing page client at `landing-page-escluse/src/lib/api/client.ts:36` uses `baseURL: https://api.esluce.com/api/v1` (cross-origin from esluce.com)
- timestamp: 2026-05-16 502 status on OPTIONS preflight suggests Caddy got an error from backend or backend is down

## Eliminated
(none)

## Resolution
- root_cause: Two compounding issues: (1) Backend CORS config uses `allow_origin(Any)` with `allow_credentials(true)` — CORS spec forbids wildcard origin with credentials, browsers block this. (2) The 502 status on OPTIONS preflight indicates Caddy couldn't reach the backend or backend returned an error.
- fix: Change backend CORS to use explicit allowed origins (esluce.com, app.esluce.com) instead of Any. Optionally add CORS headers in Caddyfile.prod as a defense-in-depth measure.
- files_changed: api/src/bootstrap/mod.rs, gateway/Caddyfile.prod (optional)
