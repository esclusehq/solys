# Phase 86: Fix Logout Button on Landing Page — Research

**Researched:** 2026-06-16
**Domain:** Frontend Authentication (React + Supabase + Custom JWT)
**Confidence:** HIGH

## Summary

The logout button on the landing page (`App.tsx` → `Navbar` component) does not work because the `Navbar` defines its own inline `handleLogout` function that **completely bypasses the zustand store's `logout()` method**. The inline implementation uses `localStorage.clear()` + `window.location.href` (full page reload) instead of the store's targeted state reset. When the API call to the backend's `/api/v1/auth/logout` endpoint fails or the cookie-clearing doesn't propagate properly, the catch block silently redirects to `/` without clearing any auth state — and the subsequent `checkAuth()` call (triggered on mount) re-authenticates the user from the still-valid cookie.

**Primary recommendation:** Replace the `Navbar`'s inline `handleLogout` with a call to `useAuthStore.getState().logout()` followed by React Router navigation. Remove the separate inline implementation entirely.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| — | Fix logout button on landing page — users cannot log out | Root cause identified: Navbar bypasses zustand store logout; catch-block swallows failures without clearing state; full-page reload triggers `checkAuth()` which re-auths from intact cookie |
</phase_requirements>

<user_constraints>
## User Constraints (from CONTEXT.md)

No CONTEXT.md exists. This is the initial phase in the directory.

### Locked Decisions
None — no CONTEXT.md present.

### the agent's Discretion
Freedom to investigate and recommend the best fix approach.

### Deferred Ideas (OUT OF SCOPE)
None.
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Auth UI components (logout button) | Browser / Client | — | The button is rendered in the SPA; click handling happens in the browser |
| Auth state management | Browser / Client | — | Zustand store + localStorage persist; all auth state decisions are client-side |
| Auth API endpoints (logout, me) | API / Backend | — | Rust Axum server at port 3000, proxied via Caddy |
| Session token (JWT in HttpOnly cookie) | API / Backend | Browser | Server sets cookies via Set-Cookie; browser manages cookie lifecycle |
| Supabase session | API / Backend (Supabase) | — | OAuth identity via Supabase; sign-out is called client-side |
| Auth persistence | Browser / Client (localStorage) | — | Zustand persist middleware stores serialized auth data |

## Standard Stack

### Core

| Library/Component | Version | Purpose | Why Standard |
|------------------|---------|---------|--------------|
| React + Zustand | React 19 / Zustand 5 | Component rendering + state management | Existing project standard |
| @supabase/supabase-js | ^2.100.0 | OAuth identity provider | Existing project standard |
| custom Rust backend (Axum) | — | JWT-based auth with HttpOnly cookies | Existing project standard |

### Supporting

| Library/Component | Version | Purpose | When to Use |
|------------------|---------|---------|-------------|
| react-router-dom | ^7.14.0 | Client-side navigation | All route transitions |

## Architecture Patterns

### Auth Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Browser / Client                                                │
│                                                                 │
│  ┌──────────────┐   ┌────────────────┐   ┌──────────────────┐  │
│  │ Navbar        │   │ useAuthStore   │   │ useAuth hook     │  │
│  │ (logout btn)  │──▶│ (Zustand)      │   │ (useAuthStore    │  │
│  │               │   │                │   │  wrapper)        │  │
│  │ handleLogout: │   │ logout()       │   │                  │  │
│  │  BUG: inline  │   │  ✓ API call    │   │ handleLogout:    │  │
│  │  implement.   │   │  ✓ signOut()   │   │  → store.logout()│  │
│  │  bypasses     │   │  ✓ state reset │   │  → navigate('/') │  │
│  │  store        │   │  ✓ persist     │   │                  │  │
│  │               │   │    cleanup     │   │  (NOT USED by    │  │
│  │               │   │                │   │   Navbar)        │  │
│  └──────┬────────┘   └───────┬────────┘   └──────────────────┘  │
│         │                    │                                   │
│         │                    │ localStorage:                     │
│         │                    │  - "auth-storage" (zustand        │
│         │                    │    persist: user object)          │
│         │                    │  - "sb-...-auth-token" (Supabase) │
│         │                    │                                   │
│         │          ┌─────────▼──────────┐                        │
│         │          │ supabase.auth       │                        │
│         │          │ signOut()           │                        │
│         │          │ (global or local)   │                        │
│         │          └─────────┬──────────┘                        │
│         │                    │                                   │
│         ▼                    ▼                                   │
│  ┌──────────────────────────────────────────────────┐           │
│  │ fetch() / axios (withCredentials: true)           │           │
│  │ Sends HttpOnly cookie "access_token"             │           │
│  │ Receives Set-Cookie headers                      │           │
│  └──────────────────────┬───────────────────────────┘           │
└─────────────────────────┼───────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────────┐
│ Caddy reverse proxy (esluce.com → escluse_backend:3000)          │
│ Header modification: header_down Set-Cookie may interfere        │
└──────────────────────────────────┬───────────────────────────────┘
                                   │
                                   ▼
┌──────────────────────────────────────────────────────────────────┐
│ Backend (Rust Axum)                                              │
│ ┌───────────────┐  ┌──────────────┐  ┌───────────────────────┐  │
│ │ POST /auth/   │  │ GET /auth/me │  │ POST /auth/refresh   │  │
│ │ logout        │  │              │  │                       │  │
│ │               │  │ AuthUser     │  │ AuthUser              │  │
│ │ No auth reqd  │  │ extractor:   │  │ extractor:            │  │
│ │ Returns       │  │  1. Bearer   │  │  1. Bearer            │  │
│ │ Set-Cookie:   │  │     header   │  │     header            │  │
│ │ Max-Age=0     │  │  2. Cookie   │  │  2. Cookie            │  │
│ └───────┬───────┘  └──────┬───────┘  └───────────┬───────────┘  │
│         │                 │                      │              │
│         │          ┌──────▼──────┐               │              │
│         │          │ JwtService  │               │              │
│         │          │ validate()  │               │              │
│         │          └─────────────┘               │              │
│         │                                        │              │
│         └──────────────────┬─────────────────────┘              │
│                            │                                    │
│                     ┌──────▼──────┐                             │
│                     │ COOKIE_DOMAIN:                             │
│                     │ .esluce.com  │                             │
│                     └─────────────┘                             │
└──────────────────────────────────────────────────────────────────┘
```

### Auth State Flow (Correct)

```
LOGIN:
  SignInForm → authApi.login() [axios withCredentials] 
    → POST /api/v1/auth/login 
    → Backend sets Set-Cookie: access_token=<JWT> (HttpOnly, Secure, SameSite=Lax)
    → Response body: { success: true, data: { access_token, refresh_token, message } }
    → authStore sets isAuthenticated: true
    → authStore.fetchUser() → GET /api/v1/auth/me (cookie sent automatically)
    → Zustand persist saves { user } to localStorage "auth-storage"

LOGOUT (CORRECT path via useAuthStore.logout()):
  useAuthStore.logout():
    1. POST /api/v1/auth/logout (cookie sent)
    2. Backend responds Set-Cookie: access_token=; Max-Age=0 (deletes cookie)
    3. supabase.auth.signOut() (global scope — revokes session server-side)
    4. Object.keys(localStorage).forEach: remove keys with 'supabase' or 'auth'
    5. set({ user: null, isAuthenticated: false, isLoading: false, error: null })
    6. localStorage.removeItem('auth-storage')
    
  → Immediately: Components re-render with isAuthenticated: false
  → Navbar shows "Log in" button
  → (No page reload needed)

CHECK AUTH (on mount):
  Navbar.checkAuth():
    1. Sets isLoading: true, user: null, isAuthenticated: false (pessimistic)
    2. GET /api/v1/auth/me with credentials: 'include'
    3. If cookie valid → 200 → data.data exists → set isAuthenticated: true
    4. If cookie invalid → 401 → set isAuthenticated: false
```

### Recommended Project Structure

No structural changes needed — files already exist at the right paths.

### Pattern: Navbar Should Call Store Logout

**What:** The Navbar component should call the zustand store's `logout()` method when the user clicks Logout, not define its own inline logout.

**Why:** The store's `logout()` handles all cleanup (API call, Supabase signout, state reset, persistence cleanup) and ensures immediate UI reactivity through React state updates. Any component that needs logout should call the same source of truth.

### Anti-Patterns to Avoid

- **Inline logout implementations:** Every logout path should go through `useAuthStore.logout()`. Duplicate implementations create bugs.
- **`localStorage.clear()` for logout:** This nukes all localStorage data, not just auth. Use targeted removals instead.
- **`window.location.href = '/'` for redirect after logout:** Use React Router's `navigate()` to avoid full page reload, which triggers unnecessary `checkAuth()` calls and creates a race condition where a still-valid cookie re-authenticates the user.
- **Silent catch blocks:** The catch block should at minimum clear local auth state if API calls fail. The current catch block just logs and redirects without cleanup.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Auth state management | Custom state container | Zustand (already in use) | Already integrated; just call the existing `logout()` |
| HTTP requests with auth | Raw fetch with manual header/cookie handling | Axios with interceptors (already in use for `apiClient`) | The store's `logout()` already uses the right approach |

## Common Pitfalls

### Pitfall 1: Multiple Auth State Sources
**What goes wrong:** The Navbar component manages its own logout flow independently from the zustand store, creating two divergent sources of truth for auth state.

**Why it happens:** The `Navbar` component was written with an inline `handleLogout` that duplicates (and conflicts with) the store's `logout()`. The inline version was probably created because the developer wanted a "reset everything" approach after logout.

**How to avoid:** All logout paths must go through `useAuthStore.logout()`. Define the logout handler once in the store; components just call it.

**Warning signs:** Components destructuring `logout` from `useAuthStore` but never calling it (line 82 of App.tsx).

### Pitfall 2: Full Page Reload After Auth Change
**What goes wrong:** `window.location.href = '/'` after logout triggers a full page reload, which re-mounts the React app, re-hydrates persisted state from localStorage, and re-fires `checkAuth()`. If the cookie wasn't properly cleared (due to API failure, proxy interference, etc.), the user gets silently re-authenticated.

**Why it happens:** Full page reload is a sledgehammer approach that destroys the in-memory Zustand state and relies on the server cookie being gone — but doesn't verify this precondition.

**How to avoid:** Use React Router's `navigate('/signin')` after successful state reset. The zustand state update (`set({ user: null, isAuthenticated: false })`) propagates reactively before any navigation occurs.

### Pitfall 3: Cookie Deletion Through Reverse Proxy
**What goes wrong:** The backend sends `Set-Cookie: access_token=; Max-Age=0` to delete cookies, but the Caddy reverse proxy may modify or corrupt the `Set-Cookie` header before it reaches the browser.

**Why it happens:** The `Caddyfile.prod` has `header_down Set-Cookie` directives that could interfere with cookie deletion.

**How to avoid:** The fix should not rely solely on the API call deleting the cookie. The zustand store's `logout()` is properly defensive — it ignores API errors and always resets local state, handles persistence cleanup, and calls `supabase.auth.signOut()` as a secondary cleanup.

## Code Examples

### Current Buggy Code (Navbar in App.tsx, lines 103-124)
```typescript
const handleLogout = async () => {
    try {
        const API_URL = import.meta.env.VITE_API_URL || 'https://esluce.com';
        await fetch(`${API_URL}/api/v1/auth/logout`, {
            method: 'POST',
            credentials: 'include',
        });
        await supabase.auth.signOut({ scope: 'local' });
        localStorage.clear();                           // NUKES all localStorage
        sessionStorage.clear();
        await new Promise((resolve) => setTimeout(resolve, 800));  // Useless delay
        window.location.href = '/';                     // Full reload
    } catch (error) {
        console.error('Logout failed:', error);
        window.location.href = '/';                     // Redirect without cleanup!
    }
};
```

### Correct Approach (Use Existing Store Method)
```typescript
// In Navbar, replace the inline handleLogout with:
const handleLogout = async () => {
    await useAuthStore.getState().logout();
    navigate('/signin');
};

// Or even simpler — use the already-imported `logout` from useAuthStore:
// The Navbar line 82 already does: const { ..., logout, ... } = useAuthStore();
// So just call: await logout(); navigate('/signin');
```

### useAuthStore.logout() — The Correct Implementation
```typescript
logout: async () => {
    set({ isLoading: true });
    try {
        const API_URL = import.meta.env.VITE_API_URL || 'https://esluce.com';
        await fetch(`${API_URL}/api/v1/auth/logout`, {
            method: 'POST',
            credentials: 'include',
        });
    } catch {}  // API failure is non-fatal — continue cleanup
    
    try {
        await supabase.auth.signOut();  // Global scope — revokes Supabase session
    } catch {}  // Supabase failure is non-fatal — continue cleanup
    
    // Selective cleanup (not nuclear)
    Object.keys(localStorage).forEach(key => {
        if (key.includes('supabase') || key.includes('auth')) {
            localStorage.removeItem(key);
        }
    });
    
    set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: null,
    });
    
    localStorage.removeItem('auth-storage');
},
```

## State of the Art

| Old Approach (Navbar inline) | Correct Approach (store.logout()) | Impact |
|-----------------------------|-----------------------------------|--------|
| `localStorage.clear()` | Selective key removal | Non-auth data preserved |
| `supabase.auth.signOut({ scope: 'local' })` | `supabase.auth.signOut()` (global) | Server-side Supabase session revoked |
| Skip store state reset | `set({ user: null, isAuthenticated: false })` | UI reacts immediately |
| `window.location.href = '/'` | `navigate('/signin')` | No page reload; no race condition |
| 800ms delay | No delay | Instant UX |
| Catch block: redirect without cleanup | Catch blocks: continue with cleanup | Defensive — errors don't prevent logout |

## Root Cause Analysis

### Primary Root Cause

The `Navbar` component in `App.tsx` (lines 103-124) defines an inline `handleLogout` function that **duplicates and overrides** the existing `useAuthStore.logout()` method. The Navbar imports `logout` from the zustand store (line 82) but never calls it.

When the user clicks Logout:

1. **`Navbar.handleLogout` fires a raw `fetch`** to `POST /api/v1/auth/logout` — this should clear the HttpOnly cookie via `Set-Cookie: Max-Age=0`.

2. If the fetch call **succeeds**, `localStorage.clear()` removes the persisted zustand state (`auth-storage`), then `window.location.href = '/'` triggers a full page reload.

3. On reload, `checkAuth()` fires (line 86-89). If the cookie was properly cleared, `checkAuth` returns `isAuthenticated: false`. But if the cookie survived (e.g., Caddy proxy interfered, or the Set-Cookie didn't propagate correctly), `checkAuth` re-authenticates the user.

4. If the fetch call **fails** (network error, CORS issue in dev, proxy timeout, etc.), the `catch` block runs: it logs `'Logout failed:'` and redirects to `/` **without calling `localStorage.clear()`** and without resetting any auth state. The cookie was never cleared so the API call failed. On reload, `checkAuth()` finds the still-valid cookie and immediately re-authenticates.

### Secondary Issues

- **Unused import**: `logout` is destructured from `useAuthStore()` on line 82 but never called.
- **Inconsistent Supabase signOut scope**: Navbar uses `{ scope: 'local' }` (only clears local state), while the store's `logout()` uses no scope (global — revokes session server-side).
- **Debounce without purpose**: 800ms delay before redirect serves no function.
- **Dead code in useAuth.ts**: `localStorage.getItem('access_token')` checks for a value that is never stored by any component.

## Files That Need Modification

| File | Change Required | Risk |
|------|----------------|------|
| `landing-page-escluse/src/App.tsx` (lines 103-124) | Replace inline `handleLogout` with call to `useAuthStore.getState().logout()` + `navigate('/signin')` | LOW — well-defined change, existing store method tested |
| `landing-page-escluse/src/App.tsx` (line 103-124) | Remove unused `logout` from destructuring if applicable | LOW — cleanup |
| `landing-page-escluse/src/App.tsx` | Remove unused imports if any (`supabase` import might still be needed for avatar fetch on line 94) | LOW |

## Files That Need Review (potential improvements)

| File | Issue | Priority |
|------|-------|----------|
| `landing-page-escluse/src/App.tsx` line 82 | `logout` destructured but unused — remove or use | MEDIUM |
| `landing-page-escluse/src/lib/hooks/useAuth.ts` line 22 | `access_token` localStorage check references a value never stored — dead code | LOW |
| `gateway/Caddyfile.prod` line 30-31 | `header_down Set-Cookie` directive may interfere with cookie operations | LOW (needs production testing) |
| `api/src/presentation/handlers/auth_handlers.rs` line 270-278 | `logout()` returns `Set-Cookie: Max-Age=0` but response body is not wrapped in `ApiResponse` — minor inconsistency | LOW |

## Integration Points

| System | Integration | Notes |
|--------|-------------|-------|
| Backend API `POST /api/v1/auth/logout` | Called by both Navbar inline and store.logout() | Endpoint does not require auth; just clears cookies |
| Backend API `GET /api/v1/auth/me` | Called by `checkAuth()` on mount | Uses cookie-based JWT auth via `AuthUser` extractor |
| Supabase `auth.signOut()` | Called during logout | Navbar uses `{ scope: 'local' }` (wrong); store uses global scope (correct) |
| Zustand persist (localStorage) | `auth-storage` key holds persisted user object | Need to clear on logout |
| Caddy reverse proxy | Proxies `/api/*` to backend; sets `header_down Set-Cookie` | Potential source of cookie deletion failures |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Frontend dev server | ✓ | — | — |
| npm | Package management | ✓ | — | — |
| Docker/Docker Compose | Full stack deployment | ✓ | — | — |
| Backend API | Auth endpoints | ✓ (via Caddy proxy) | — | — |

No missing dependencies — this is a frontend code fix.

## Validation Architecture

### Test Framework

No test infrastructure detected for the landing page (`landing-page-escluse`). The `package.json` has no test script. This is a gap.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LOGOUT-01 | Clicking logout calls store.logout() | unit | `npm test` (none) | ❌ |
| LOGOUT-02 | After logout, isAuthenticated is false | unit | `npm test` (none) | ❌ |
| LOGOUT-03 | After logout, user is redirected to /signin | e2e | — | ❌ |

### Sampling Rate
- **Per task commit:** N/A — no test infrastructure
- **Per wave merge:** N/A
- **Phase gate:** Manual verification only

### Wave 0 Gaps
- [ ] `landing-page-escluse/tests/` — no test directory exists
- [ ] No test framework configured in `package.json`
- [ ] Manual verification required: click logout, verify redirect to /signin, verify refresh shows login form

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V3 Session Management | yes | HttpOnly cookies; backend clears cookies on logout |
| V2 Authentication | no | Not changing login flow |

### Known Threat Patterns for React + JWT Auth

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Session fixation (logout failure) | Spoofing | Store-side state reset + cookie deletion; defensive programming (reset state even if API fails) |

The current fix approach (calling `useAuthStore.logout()`) already follows the principle of defense-in-depth — it resets local state regardless of API success.

## Assumptions Log

No `[ASSUMED]` claims were made in this research. All findings were verified against the actual codebase at known file paths and line numbers.

## Open Questions

1. **Does the Caddy `header_down Set-Cookie` directive interfere with cookie deletion?**
   - What we know: `Caddyfile.prod` line 30 has `header_down Set-Cookie "SameSite=Lax; Secure" "SameSite=Lax; Secure"` which may replace or corrupt the backend's Set-Cookie headers.
   - What's unclear: Whether this is an active issue in production. The Caddy syntax may be misconfigured.
   - Recommendation: The fix should not rely on cookie deletion being 100% reliable. The store's `logout()` already handles this by resetting local state unconditionally. Address the Caddyfile as a separate issue if logout still fails after the frontend fix.

2. **Does `access_token` in localStorage serve any purpose?**
   - What we know: `useAuth.ts` line 22 checks `localStorage.getItem('access_token')` to trigger `fetchUser()`, but no code ever sets this value.
   - What's unclear: Was this intended for a different auth flow? Could it be set by Supabase or some other library?
   - Recommendation: Investigate and remove dead code if confirmed unused.

## Sources

### Primary (HIGH confidence)
- [VERIFIED: Codebase] `landing-page-escluse/src/App.tsx` — Navbar's inline `handleLogout` (lines 103-124)
- [VERIFIED: Codebase] `landing-page-escluse/src/lib/stores/authStore.ts` — zustand store's `logout()` (lines 81-110)
- [VERIFIED: Codebase] `landing-page-escluse/src/lib/hooks/useAuth.ts` — useAuth hook's `handleLogout` (lines 49-52)
- [VERIFIED: Codebase] `api/src/presentation/handlers/auth_handlers.rs` — backend logout handler (lines 270-278)
- [VERIFIED: Codebase] `api/src/domain/auth/middleware.rs` — AuthUser extractor (cookie-based JWT auth)
- [VERIFIED: Codebase] `gateway/Caddyfile.prod` — Caddy reverse proxy config with header_down Set-Cookie
- [VERIFIED: Codebase] `api/src/presentation/responses/cookie.rs` — CookieBuilder and delete_auth_cookies

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against actual package.json and imports
- Architecture: HIGH — traced full auth flow through frontend, backend, and proxy
- Pitfalls: HIGH — verified each pitfall against actual code paths

**Research date:** 2026-06-16
**Valid until:** 2026-07-16 (stable — no fast-moving dependencies involved)
