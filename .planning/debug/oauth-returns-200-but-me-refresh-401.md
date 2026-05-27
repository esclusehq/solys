---
status: resolved
trigger: OAuth login - user clicks "Sign in with Google", Supabase returns 200 but subsequent /auth/me and /auth/refresh return 401
created: 2026-05-16
updated: 2026-05-16
---

## Symptoms

| Endpoint | Expected | Actual |
|----------|----------|--------|
| POST /api/v1/auth/oauth | 200, user authenticated | ✅ 200 |
| GET /api/v1/auth/me | 200, user data | ❌ 401 |
| POST /api/v1/auth/refresh | 200, new token | ❌ 401 |

## Root Cause

**Primary:** Frontend axios client missing `withCredentials: true` - cookies not sent with requests

**Secondary:** Backend cookies missing Domain attribute for cross-subdomain sharing

## Resolution

**Fix Applied:**

1. **Frontend (client.ts):**
   - Added `withCredentials: true` to axios config (line 40)
   - Fixed 401 interceptor to use `apiClient` instead of direct axios (line 57)

2. **Backend (cookie.rs):**
   - Added `COOKIE_DOMAIN` environment variable support
   - Cookies now include Domain attribute when `COOKIE_DOMAIN` is set

**Configuration Required:**

For production, set environment variable:
- `COOKIE_DOMAIN=.esluce.com` (or your domain)

This allows cookies to be shared across subdomains (e.g., api.esluce.com ↔ esluce.com)