---
phase: 86
plan: 01
name: Fix landing page logout button
completed: 2026-06-16
files_modified:
  - landing-page-escluse/src/App.tsx
verification: PASSED
build: PASSED
---

# Phase 86: Fix Logout Button on Landing Page — Plan 01 Summary

**Fix the Navbar's logout button by replacing the buggy inline `handleLogout` with a call to the existing zustand store's `logout()` method.**

## Root Cause

The `Navbar` component in `App.tsx` defined its own inline `handleLogout` that bypassed the zustand store's `logout()` method. It used:
- Raw `fetch` to POST `/api/v1/auth/logout` (store handles this)
- `supabase.auth.signOut({ scope: 'local' })` (local scope — should be global)
- `localStorage.clear()` (nuclear — destroys all localStorage data)
- 800ms delay (useless)
- `window.location.href = '/'` (full reload — re-triggers `checkAuth()`)

When the API call failed, the catch block redirected without clearing any auth state, and the subsequent `checkAuth()` re-authenticated the user from the still-valid cookie.

## Fix

Replaced the 22-line inline `handleLogout` (lines 103-124) with:

```typescript
const handleLogout = async () => {
    await logout();
    navigate('/signin');
};
```

Where `logout` is already destructured from `useAuthStore()` (line 82) and `navigate` is from `useNavigate()`.

## Verification

- ✅ `npm run build` exits 0 (build successful)
- ✅ No `localStorage.clear()` in App.tsx
- ✅ No `window.location.href` in App.tsx
- ✅ `handleLogout` now calls `logout()` + `navigate('/signin')`
- ✅ `supabase` import preserved (still used for avatar fetch)

## Files Modified

- `landing-page-escluse/src/App.tsx` — Replaced inline `handleLogout` with store-based implementation
