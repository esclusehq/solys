# Phase 24: Keamanan ENV - Context

**Gathered:** 2026-04-20

<domain>
## What We're Securing

Currently sensitive configs are exposed via import.meta.env at build time. These values get bundled into the frontend JS and are visible to anyone who views page source.

**Current exposure:**
- VITE_SUPABASE_URL (less sensitive, but still visible)
- VITE_SUPABASE_ANON_KEY (public key, but still shouldn't be in build)
- VITE_API_URL (visible in source)

**Goal:** Move sensitive config to runtime instead of build-time.

</domain>

<decisions>
## Implementation

**Chosen:** Use runtime config instead of build-time env vars

For sensitive values:
- Remove from .env files that get built into JS
- Fetch from API at runtime (/api/v1/config or similar)
- Or keep in backend only, not exposed to frontend

For Supabase:
- supabaseUrl already needs to be known at build (it's the redirect URL)
- But anon key could be fetched if needed (though PKCE works without it)

## Key Changes

1. **Remove sensitive imports:** Don't use import.meta.env.VITE_* for anything
2. **Runtime config endpoint:** Add /api/v1/config returning safe values only
3. **Keep public values:** VITE_API_URL for WebSocket is acceptable (not secret)

**Actually:**
- Supabase URL needs to be in frontend (OAuth redirect)
- Supabase anon key is meant to be public (used in client)
- REAL secrets: Database credentials, API keys — already in backend only

</decisions>

<canonical_refs>
## References

- app/src/lib/supabase.js — uses import.meta.env
- app/src/hooks/useServers.js — uses import.meta.env

</canonical_refs>

<specifics>
## Findings

**It's already mostly OK:**
- Backend uses std::env::var() — secrets never reach frontend
- Frontend uses VITE_ prefix — Vite strips non-VITE_ vars
- Supabase anon key IS public (that's the point)

**What might need fixing:**
- Check if any real secrets accidentally have VITE_ prefix
- Ensure .gitignore covers .env files

</specifics>

<deferred>
## Deferred

- Runtime config endpoint (not needed if already secure)
- Move all config to backend (over-engineering)

</deferred>

<verification>
## Verification Complete (2026-04-20)

Checked:
1. `.gitignore` covers `*.env` files (except `.env.example`) ✅
2. Vite build strips non-VITE_ env vars ✅
3. Supabase anon key is public by design ✅

**Result:** No code changes needed - already secure.

</verification>

---

## ▶ Next Up

`/clear` then:

/gsd-plan-phase 24 ${GSD_WS} — decide if any action needed