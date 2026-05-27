# Plan 49-04: Redirect to landing page after login - Summary

## Completed

Updated all login/signup redirect destinations to point to the landing page (`/`) instead of `/onboarding`.

### Changes Made

**`src/lib/hooks/useAuth.ts`:**
- Line 38: Changed `redirectTo('/onboarding')` → `redirectTo('/')` after successful login
- Line 45: Changed `redirectTo('/onboarding')` → `redirectTo('/')` after successful registration

**`src/pages/SignIn.tsx`:**
- Line 25: Changed `onSuccess={() => navigate('/onboarding')}` → `onSuccess={() => navigate('/')}`

**`src/pages/SignUp.tsx`:**
- Line 25: Changed `onSuccess={() => navigate('/onboarding')}` → `onSuccess={() => navigate('/')}`

**`src/pages/oauth/OAuthCallback.tsx`:**
- Already redirects to `/` — no changes needed

### Verification

- Build passes successfully
- All login flows now redirect to the landing page (esluce.com)

### Notes

- The Supabase anon key in `.env` has an unusual format (`sb_publishable_...` instead of starting with `eyJ...`). May need to verify with Supabase dashboard.
- Plan 49-01 (Supabase credentials) and 49-02 (OAuth providers) still need to be completed.