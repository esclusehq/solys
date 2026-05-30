# Phase 54: Email Verification Flow — Research

**Researched:** 2026-05-30
**Domain:** User authentication, email verification, rate limiting, feature gating
**Confidence:** HIGH

## Summary

The backend already has the core verification infrastructure: `register` handler generates verification tokens and sends via Resend, `verify_email` handler confirms tokens, and DB columns (`email_verified_at`, `verification_token`, `verification_expires`) exist on the `users` table. The frontend `VerifyEmailPage.jsx` exists but has a URL bug (calls `/api/auth/verify-email` instead of the correct `/api/v1/auth/verify-email`).

The phase needs to implement: (1) a `VerifiedUser` custom Axum extractor for backend gating (analogous to `AuthUser`), (2) a `resend-verification` endpoint with rate limiting (60s cooldown, 5 max), (3) OAuth auto-verification, (4) `pending_email` migration for the email-change pattern, (5) frontend banner + blocking dialog + success page, and (6) gating on backend + frontend for 4 categories of sensitive features.

The existing `ApiError` response type already supports `details` and `action` fields — we can extend it to carry resend state for D-12 without a separate struct.

**Primary recommendation:** Implement in 5 waves — (1) backend infrastructure: `VerifiedUser` extractor, resend endpoint, rate limiting, OAuth auto-verify; (2) frontend store extensions + banner component; (3) verify success page + `returnTo` redirect; (4) feature-gating on backend routes with `VerifiedUser`; (5) frontend gating and blocking dialog.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Hybrid flow — signup auto-logs in the user (as now), then shows a persistent global banner prompting email verification immediately. No separate "Check your email" page.
- **D-02:** Global banner — small banner on all pages: `⚠ Verify your email — Check your inbox at user@example.com [Resend Email] [Change Email]`. The [Change Email] link navigates to profile settings email change form.
- **D-03:** Feature-blocking dialog — when an unverified user tries to access a gated feature, show a modal: "Email verification required — Verify your email before using this feature. [Resend Email]"
- **D-04:** Verification success — show brief success page → 2–3 second auto-redirect → if `returnTo` URL param exists, redirect there; otherwise redirect to dashboard. Manual button as fallback.
- **D-05:** Rate limiting — 60-second cooldown between resends.
- **D-06:** Hard cap — maximum 5 resend attempts total per registration. After cap, verification token expires and user sees "Contact support" message.
- **D-07:** Cooldown UX — button stays visible during cooldown. Clicking during cooldown shows tooltip with remaining time. Successful send shows toast notification.
- **D-08:** Gated feature categories (require `email_verified_at != null`):
  - **Identity & Access:** API Keys, Personal Access Tokens, OAuth Applications, Team Invites, Team Creation
  - **Resource Creation:** Project Creation, Server Creation, Server Deployment, Node Registration, Instance Creation, Environment Creation
  - **Integration Features:** Webhooks, External Integrations, SDK Credentials
  - **Financial Features:** Billing, Subscription Changes, Invoice Management, Payment Methods
- **D-09:** Ungated features (always accessible without verification): Login, Dashboard, Profile Settings, Security Settings, Login History, Documentation, Pricing, Community/Discord, Account Preferences
- **D-10:** Enforcement — defense in depth: backend + frontend.
- **D-11:** Backend mechanism — custom Axum extractor `VerifiedUser` (analogous to existing `AuthUser`) that checks `email_verified_at`. Used on gated route handlers.
- **D-12:** Error response format — enhanced JSON: `{ error: "email_not_verified", message: "Verify your email before accessing this resource." }` with resend data so frontend can inline the resend dialog without a separate API call.
- **D-13:** OAuth users (Google/GitHub/Discord) are auto-verified on registration/login — set `email_verified_at = now()` in the `oauth` handler since the provider already verified the email.
- **D-14:** Email change for any user (OAuth or email/password) follows the pending-email pattern:
  - Keep old email verified
  - Store `pending_email` on user record
  - Send verification to the new email
  - Keep all features working (old email remains verified)
  - Switch to new email only after verification succeeds
  - Then mark new email as verified

### the agent's Discretion
- Specific UI design of the banner (colors, position, dismissibility behavior)
- Specific UI design of the feature-blocking dialog
- Backend endpoint design for resend + implementation details
- Migration design for `pending_email` column
- Where to add the gating middleware/extractor in the route tree
- Frontend component structure (separate Banner component vs inline)
- How to handle the edge case where a user has no email (OAuth-only provider) trying to change email

### Deferred Ideas (OUT OF SCOPE)
None.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Email sending (Resend) | API / Backend | — | Backend has `EmailService`, env vars, sends via Resend API |
| Token generation & validation | API / Backend | — | Pure backend operation: UUID token + DB columns |
| Rate limiting (resend cooldown) | API / Backend | — | Must be server-enforced to prevent abuse |
| Feature gating enforcement | API / Backend | Frontend (UX) | D-10 mandates defense in depth: backend blocks API access, frontend shows dialog |
| `VerifiedUser` extractor | API / Backend | — | Custom Axum `FromRequestParts` implementation, analogous to `AuthUser` |
| Verification status display | Browser / Client | — | Banner + dialog are purely frontend UX |
| Verification success page | Browser / Client | — | Frontend page with auto-redirect |
| OAuth auto-verification | API / Backend | — | Set `email_verified_at` in oauth handler at login/registration |
| `pending_email` migration | API / Backend | — | New DB column + migration |
| Cooldown UX (tooltip + timer) | Browser / Client | — | Client-side countdown display with server-enforced cooldown fallback |

## Standard Stack

No new libraries needed. All work uses existing stack:

### Core Backend (Rust)
| Code | Location | Purpose |
|------|----------|---------|
| `AuthUser` extractor | `api/src/domain/auth/middleware.rs` | Pattern template for `VerifiedUser` |
| `ApiError` + `ApiResponse` | `api/src/presentation/responses/api_response.rs` | Extended for email_not_verified errors with resend data |
| `EmailService` | `api/src/infrastructure/email/mod.rs` | Already has `send_verification()` method |
| `User` model | `api/src/domain/user/model.rs` | Has `email_verified_at`, `verification_token`, `verification_expires`, `is_verification_valid()` |
| `UserRepository` | `api/src/domain/user/repository.rs` | Has `find_by_verification_token()` + `update()` |
| `SqlxUserRepository` | `api/src/domain/user/sqlx_repository.rs` | Complete CRUD with all columns |
| `JwtService` / `Claims` | `api/src/domain/auth/service.rs` | JWT validation for auth extractors |

### Core Frontend (JavaScript/React)
| Code | Location | Purpose |
|------|----------|---------|
| `authStore.js` | `app/src/store/authStore.js` | Extend with verification state + resend action |
| `auth.js` (API client) | `app/src/api/auth.js` | Already has `verifyEmail()`, add `resendVerification()` |
| `fetchApi()` client | `app/src/api/client.js` | Base URL `/api/v1`, all auth calls go through it |
| `useStore()` (uiStore) | `app/src/store/uiStore.js` | Has `addToast()`, `openModal()`, `closeModal()` |
| `VerifyEmailPage.jsx` | `app/src/pages/auth/VerifyEmailPage.jsx` | Fix URL bug + add `returnTo` support |
| `App.jsx` | `app/src/app/App.jsx` | Route definitions, banner placement |
| `ProtectedRoute.jsx` | `app/src/components/ProtectedRoute.jsx` | Auth gating pattern (may extend for email verification) |

### Alternatives Considered — None (all work uses existing stack, no new packages needed)

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                      FRONTEND (Browser / Client)                    │
│                                                                     │
│  ┌──────────┐   ┌──────────────┐   ┌──────────────────────┐       │
│  │authStore  │   │VerifyEmail   │   │EmailVerification     │       │
│  │(Zustand)  │◄──│Page          │   │Banner / GatingDialog │       │
│  │+persist   │   │/verify-email │   │(new components)      │       │
│  └─────┬─────┘   └──────┬───────┘   └──────────┬───────────┘       │
│        │                │                      │                   │
│        │       ┌────────▼────────┐              │                   │
│        └───────►  auth.js API    ◄──────────────┘                   │
│                │  client         │                                  │
│                └────────┬────────┘                                  │
│                         │ fetchApi('/api/v1/auth/...')              │
└─────────────────────────┼───────────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────────┐
│                      BACKEND (Rust / Axum)                          │
│                                                                     │
│  ┌─────────────────────┐                                            │
│  │     Axum Router     │                                            │
│  │  /api/v1/auth/...   │                                            │
│  └──┬──────┬──────┬────┘                                            │
│     │      │      │                                                 │
│  ┌──▼──┐┌──▼────┐┌─▼──────────┐                                    │
│  │verify││resend ││oauth       │                                    │
│  │email ││verifi-││handler     │◄── set email_verified_at = now()   │
│  │handler││cation ││(D-13)     │                                    │
│  └──┬───┘│handler│└────────────┘                                    │
│     │    └──┬────┘                                                  │
│     │       │  rate_limit(60s) + max_attempts(5)                    │
│     │       │                                                       │
│  ┌──▼───────▼──────────────────────────────────────┐                │
│  │         UserRepository + SqlxUserRepository      │                │
│  │  find_by_verification_token / update / find_by_id│               │
│  └──────────────┬────────────────────────────────────┘               │
│                 │                                                    │
│  ┌──────────────▼────────────────────────────────────┐                │
│  │              PostgreSQL (users table)              │                │
│  │  email_verified_at / verification_token /          │                │
│  │  verification_expires / pending_email (D-14)      │               │
│  └────────────────────────────────────────────────────┘               │
│                                                                     │
│  ┌─────────────────────────────────────────────────────┐             │
│  │         VerifiedUser Extractor (D-11)                │             │
│  │  ┌─────────────┐  ┌──────────────────────────┐      │             │
│  │  │Extract Auth  │  │Lookup user in DB,        │      │             │
│  │  │User from JWT │──►check email_verified_at   │      │             │
│  │  └─────────────┘  │return Ok or EmailNotVerified│     │             │
│  │                   └──────────────────────────┘      │             │
│  └─────────────────────────────────────────────────────┘             │
│                                                                     │
│  ┌──────────────────────────────────────────┐                       │
│  │   EmailService (Resend API)              │                       │
│  │  send_verification(to, token, app_url)   │                       │
│  └──────────────────────────────────────────┘                       │
└─────────────────────────────────────────────────────────────────────┘

User Flow:
  Signup ──► /register ──► Auto-login ──► Dashboard with Banner
                                                │
                  User clicks link in email     │
                        │                      │
                  /verify-email?token=X  ──────┤
                        │                      │
                  POST /api/v1/auth/verify-email│
                        │                      │
                  Success page (2-3s) ─────────► Redirect to Dashboard
                        │                      │   (token cleared)
                  Redirect to returnTo if set   │ Banner disappears
                                                │
  Gated feature access (unverified user):
    Click "Create Server" ──► POST /api/v1/servers
                                  │
                          VerifiedUser extractor ──► email_not_verified error
                                  │
                          Frontend shows blocking dialog
                                  │
                          [Resend Email] button on dialog
```

### Recommended Project Structure

```
api/src/
├── domain/
│   ├── auth/
│   │   ├── middleware.rs        # Add VerifiedUser extractor alongside AuthUser
│   │   └── service.rs           # No changes needed (JWT already has claims)
│   └── user/
│       ├── model.rs             # Add pending_email field + methods
│       ├── repository.rs        # Add find_by_pending_email, update_pending_email
│       └── sqlx_repository.rs   # Implement pending_email methods
├── presentation/
│   ├── handlers/
│   │   └── auth_handlers.rs     # Add resend_verification handler, fix verify_email
│   └── responses/
│       └── api_response.rs      # No changes needed (ApiError already supports details/action)

app/src/
├── store/
│   └── authStore.js             # Add: emailVerified, resendVerification(), checkVerificationStatus()
├── api/
│   └── auth.js                  # Add: resendVerification(), fix verifyEmail() path
├── pages/
│   └── auth/
│       └── VerifyEmailPage.jsx  # Fix URL bug, add returnTo, auto-redirect after success
├── components/
│   ├── EmailVerificationBanner.jsx  # NEW: Global banner (D-02)
│   └── EmailVerificationDialog.jsx  # NEW: Blocking dialog (D-03)
└── app/
    └── App.jsx                  # Add banner, add email verification check to ProtectedRoute
```

### Pattern 1: VerifiedUser Extractor (D-11)
**What:** Custom Axum `FromRequestParts` extractor that authenticates the user (via JWT) then checks `email_verified_at` in the database. Returns `EMAIL_NOT_VERIFIED` error if unverified.
**When to use:** On any route handler that requires email verification (gated features from D-08).
**Pattern source:** `api/src/domain/auth/middleware.rs` — the existing `AuthUser` extractor. The key difference: `AuthUser` only validates JWT, `VerifiedUser` additionally queries DB for `email_verified_at`.

```rust
// Pseudo-implementation (pattern follows AuthUser + ApiError):
// 1. Extract AuthUser from JWT (reuse existing logic)
// 2. Query DB: SELECT email_verified_at FROM users WHERE id = $1
// 3. If email_verified_at.is_none() → return ApiError::forbidden("email_not_verified")
//    with details containing resend info (can_resend, cooldown_remaining, attempts_remaining)
// 4. Otherwise return VerifiedUser { user_id, tenant_id, email, role, email_verified_at }
```

### Pattern 2: Rate-Limited Resend Endpoint (D-05, D-06)
**What:** In-memory rate limiting tracked per user_id. 60s cooldown, max 5 attempts total.
**When to use:** On the `POST /auth/resend-verification` handler.
**Implementation approach:** Use a `HashMap<Uuid, ResendState>` behind an `Arc<Mutex<>>` or `tokio::sync::RwLock` in `AppContainer`. ResendState tracks: `last_sent_at`, `attempt_count`, `maxed_out`.

```rust
// Rate limit state (injected via AppContainer or use local state in handler)
pub struct ResendTracker {
    attempts: HashMap<Uuid, ResendState>,
}

struct ResendState {
    last_sent: Instant,
    count: u32,
}
```

### Pattern 3: Global Banner with Cooldown UX (D-02, D-07)
**What:** After auth check, if user exists and `email_verified_at` is null, show a top-of-page banner. Banner shows user's email, [Resend Email] button, and [Change Email] link.
**When to use:** In `App.jsx` wrapper, always visible for unverified users.
**Cooldown UX:** Button stays visible but disabled during cooldown. Tooltip shows remaining seconds. On successful resend, toast notification. After hard cap (5), button disappears and shows "Contact support" message.

### Anti-Patterns to Avoid
- **DB query per request on gated routes:** The `VerifiedUser` extractor must query the DB. Cache `email_verified_at` in the JWT claims to avoid a DB hit on every gated request — but only after a JWT claim change. For initial implementation, a DB query per gated request is acceptable (low traffic).
- **Frontend-only gating:** D-10 mandates defense in depth. Never trust frontend gating alone.
- **Blocking the banner after verification:** Once verified, banner must disappear immediately on next authStore refresh. The `authStore` must have a reactive `emailVerified` flag derived from user data.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rate limiting | Custom sliding window algorithm | Simple `Arc<RwLock<HashMap>>` with timestamp + count | Low traffic phase; HashMap is sufficient. Upgrade to Redis-based rate limiting if needed later |
| Email sending | SMTP or custom email client | Existing `EmailService` via Resend API | Already integrated, used for both verification and password reset |
| Timer/countdown cooldown | Complex interval logic | `setInterval` with `Date.now()` comparison or `useCountdown` hook | Simple client-side timer; server enforces the actual cooldown |

**Key insight:** This phase is almost entirely leveraging existing infrastructure. The only "new" code is the `VerifiedUser` extractor (pattern from `AuthUser`), the resend rate limiter (simple HashMap state), and the frontend components.

## Common Pitfalls

### Pitfall 1: Race condition on verify_email
**What goes wrong:** User clicks verification link while also trying to access a gated feature — the token is consumed by verify_email but a stale cached state shows unverified.
**Why it happens:** The `me` endpoint response includes `email_verified: user.email_verified_at.is_some()`. The frontend checks this field. If the JWT still has old claims (pre-verification), the user appears unverified.
**How to avoid:** After `verify_email` succeeds, force a `checkAuth()` call to refresh user state. The backend `/me` endpoint will return the updated `email_verified` status.
**Warning signs:** Banner persists after successful verification until manual refresh.

### Pitfall 2: The VerifyEmailPage URL bug
**What goes wrong:** `VerifyEmailPage.jsx` calls `fetch('/api/auth/verify-email', ...)` which hits the wrong path. The correct path is `/api/v1/auth/verify-email`.
**Why it happens:** The page uses raw `fetch` instead of the `fetchApi` client which prepends `/api/v1`. See `app/src/pages/auth/VerifyEmailPage.jsx` line 24.
**How to avoid:** Replace raw `fetch` call with `authApi.verifyEmail(token)` from `app/src/api/auth.js` which uses `fetchApi('/auth/verify-email', ...)` → correctly resolves to `/api/v1/auth/verify-email`.
**Warning signs:** 404 error when clicking verification link from email.

### Pitfall 3: OAuth auto-verify not set on registration
**What goes wrong:** OAuth users (Google/GitHub/Discord) are not auto-verified when they create a new account via OAuth for the first time.
**Why it happens:** The `oauth` handler in `auth_handlers.rs` creates a new `User::new()` which sets `email_verified_at: None` by default. It never sets it to `now()`.
**How to avoid:** After creating the new user in the OAuth handler (line 486-489), set `new_user.email_verified_at = Some(chrono::Utc::now())` on the created user before returning, or update it after creation.
**Warning signs:** OAuth users show the verification banner after first login.

### Pitfall 4: Resend attempts exceeding hard cap
**What goes wrong:** User uses all 5 resend attempts. The verification token expires. User is stuck — cannot verify email, cannot access gated features.
**Why it happens:** D-06 specifies that after the cap, the verification token expires. The user sees "Contact support".
**How to avoid:** After hard cap, the resend endpoint returns a clear error. The frontend should show the "Contact support" message and disable the resend button. In the email change flow (D-14), this resets because a new token is generated for the new email.
**Warning signs:** "Contact support" message in both banner and blocking dialog.

### Pitfall 5: Redis state lost on restart (rate limiting)
**What goes wrong:** Server restarts reset the in-memory `ResendTracker`, effectively resetting all cooldowns and attempt counts.
**Why it happens:** The simple HashMap approach stores state in process memory. This is acceptable for the phase scope but should be noted as a limitation.
**How to avoid:** Accept for now (documented limitation). If needed, use Redis-backed rate limiting. For a game server hosting platform in alpha, in-memory is acceptable.
**Warning signs:** Rate limiting resets after server restart.

## Code Examples

### Resend Verification Endpoint (Backend)
```rust
// In auth_handlers.rs — new handler
pub async fn resend_verification(
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<axum::response::Response, ApiError> {
    let repo = SqlxUserRepository::new(state.pool.clone());

    let user = repo.find_by_id(auth_user.user_id)
        .await
        .map_err(|e| ApiError::internal_error(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    // Already verified?
    if user.email_verified_at.is_some() {
        return Err(ApiError::new("ALREADY_VERIFIED", "Email is already verified"));
    }

    // Check rate limit (in-memory tracker)
    // ... rate limit logic ...

    // Generate new token
    let new_token = uuid::Uuid::new_v4().to_string();
    let mut updated_user = user;
    updated_user.set_verification_token(new_token.clone());

    repo.update(&updated_user).await
        .map_err(|e| ApiError::internal_error(&e.to_string()))?;

    // Send email via existing EmailService
    if let (Ok(api_key), Ok(from_email)) = (
        std::env::var("RESEND_API_KEY"),
        std::env::var("EMAIL_FROM")
    ) {
        if !api_key.is_empty() && !from_email.is_empty() {
            let email_service = crate::infrastructure::email::EmailService::new(
                api_key, from_email,
            );
            let app_url = std::env::var("APP_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5173".to_string());
            let _ = email_service.send_verification(
                &updated_user.email, &new_token, &app_url
            ).await;
        }
    }

    Ok(ApiResponse::success(serde_json::json!({
        "message": "Verification email sent",
        "cooldown_seconds": 60,
    })).into_response())
}
```

### Fixed VerifyEmailPage (Frontend)
```typescript
// Key changes to VerifyEmailPage.jsx:
// 1. Use authApi.verifyEmail(token) instead of raw fetch
// 2. Add returnTo redirect support
// 3. Show success page with auto-redirect (D-04)
// Source: existing pattern in authStore.js + auth.js

import { useSearchParams, useNavigate } from 'react-router-dom';
import * as authApi from '../../api/auth';
import { useAuthStore } from '../../store/authStore';
import { useUIStore } from '../../store/uiStore';

// On mount:
//   1. Get token from URL search params
//   2. Call authApi.verifyEmail(token)
//   3. On success: setStatus('success'), call useAuthStore.getState().checkAuth()
//   4. On failure: setStatus('error')
//   5. On success page: setTimeout(() => navigate(returnTo || '/dashboard'), 2500)
```

### EmailVerificationBanner (Frontend)
```jsx
// NEW component — placed in App.jsx above the main content but below the header
// Shows when user.isAuthenticated && !user.email_verified

function EmailVerificationBanner() {
  const { user, resendVerification } = useAuthStore();
  const [cooldown, setCooldown] = useState(0);
  const [attempts, setAttempts] = useState(0);
  const [maxedOut, setMaxedOut] = useState(false);

  // D-07: Cooldown timer with visible countdown
  useEffect(() => {
    if (cooldown <= 0) return;
    const timer = setInterval(() => {
      setCooldown(prev => prev - 1);
    }, 1000);
    return () => clearInterval(timer);
  }, [cooldown]);

  // D-08: Not shown for OAuth users (auto-verified)
  // D-02: Shows user's email, [Resend Email] button, [Change Email] link
  // D-05/D-06: Rate limited — 60s cooldown, max 5 attempts
  // If maxedOut: show "Contact support" instead of resend button
}
```

### VerifiedUser Extractor Pattern (Backend)
```rust
// In api/src/domain/auth/middleware.rs
// Pattern follows AuthUser extractor but adds email_verified_at DB check

#[derive(Clone, Debug)]
pub struct VerifiedUser {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role: String,
    pub email_verified_at: Option<DateTime<Utc>>,
}

#[async_trait]
impl<S> FromRequestParts<S> for VerifiedUser
where
    S: Send + Sync,
    ApiState: FromRef<S>,
{
    type Rejection = ApiError; // Reuse existing ApiError

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract AuthUser from JWT (reuse logic from AuthUser impl)
        //    Returns AuthError::MissingToken if no valid JWT

        // 2. Query DB for the user's email_verified_at
        //    Return ApiError::internal_error on DB failure

        // 3. If email_verified_at is None:
        //    Return ApiError::forbidden("email_not_verified")
        //      .with_details(serde_json::json!({
        //        "can_resend": true/false,
        //        "cooldown_remaining": seconds,
        //        "attempts_remaining": count,
        //      }))

        // 4. Return VerifiedUser
    }
}
```

## State of the Art

This phase introduces no new technology. It leverages:
- Existing Resend email infrastructure (already used for password reset + registration)
- Existing JWT auth pattern (`AuthUser` → `VerifiedUser`)
- Existing Zustand store pattern (extend `authStore`)
- Existing `fetchApi` client (fix `VerifyEmailPage` to use it)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Registration sends verification email but no visible feedback | Persistent global banner + resend option | This phase | Users can act on verification status |
| OAuth users NOT auto-verified (same flow as email users) | OAuth users auto-verified (D-13) | This phase | OAuth users skip the banner entirely |
| Email cannot be changed (disabled field in settings) | Email change via pending-email pattern (D-14) | This phase | Users can change email addresses |
| No feature gating based on verification | 4 categories of gated features (D-08) | This phase | Reduces resource abuse risk |
| VerifyEmailPage calls wrong API path | Fixed to use `/api/v1/auth/verify-email` | This phase | Verification actually works |

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rate limiting state | Redis-based or external rate limiter | `Arc<RwLock<HashMap<Uuid, ResendState>>>` | Simple enough for alpha. 60s cooldown + 5 max attempts — HashMap is fine. Document as non-persistent. |
| Email sending | SMTP client or SendGrid | Existing `EmailService` (Resend) | Already wired, handles HTML templates, DKIM/SPF |
| JWT validation | Custom token parsing | `jsonwebtoken` crate | Already in use, validated pattern |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The `ResendState` HashMap is sufficient for rate limiting at current scale (alpha) | Code Examples | If user base grows, in-memory state across multiple API instances will break cooldown enforcement. Fix: migrate to Redis. |
| A2 | The `VerifiedUser` extractor's DB query per request is acceptable | Pattern 1 | If gated routes become hot paths, every request hits DB for `email_verified_at`. Fix: include verification status in JWT claims. |
| A3 | The `pending_email` column only needs a simple VARCHAR(255) column | D-14 | If multiple pending emails are needed (e.g., per-user queue), the simple column won't suffice. Current scope: single pending email. |
| A4 | The `Change Email` link in the banner can navigate to profile settings via React Router | D-02 | If the SettingsPage's email change form doesn't exist yet, the link will 404. Verifying: the email field in SettingsPage is currently disabled (line 510-514). The email change form needs implementation. |

## Open Questions (RESOLVED)

1. **(RESOLVED) Email change form implementation (D-14)**
   - What we know: SettingsPage has a disabled email field (line 506-513). D-14 specifies pending-email pattern. The [Change Email] link in the banner navigates here.
   - Resolution: The email change form IS built in this phase as part of Plan 54-04, Task 3. A "Change Email" section is added to SettingsPage with: current email display (disabled), new email input field, submit button with loading state, and helper text explaining the pending-email process. Depends on Plan 54-01 (backend change_email endpoint) and Plan 54-02 (authStore changeEmail action).

2. **(RESOLVED) VerifiedUser — fresh JWT or stale `me` response?**
   - What we know: The `me` endpoint returns `email_verified` boolean. The frontend checks this.
   - Resolution: No new endpoint needed. The existing `/me` response already returns `email_verified`. The authStore refreshes user state after verify_email via `checkAuth()` (implemented in Plan 54-03, Task 1). Verification success handler calls `checkAuth()` to pull updated `email_verified` status.

3. **(RESOLVED) Resend state — how does the frontend know the cooldown and attempt count?**
   - What we know: D-12 says the error response should include resend data. D-07 says the button shows tooltip with remaining time.
   - Resolution: Hybrid approach implemented per D-07:
     - Frontend tracks cooldown locally with `useState(0)` + `setInterval` countdown in both `EmailVerificationBanner` and `EmailVerificationDialog` (Plan 54-03, Tasks 2-3).
     - On successful resend, cooldown is set to 60 seconds client-side.
     - D-12 error responses from backend include `cooldown_remaining` and `attempts_remaining` in the `with_details()` payload.
     - Server-enforced rate limiting (Plan 54-01, Task 3 via `ResendTracker`) is the source of truth; client timer is UX-only convenience.
     - Per the D-07 cooldown click fix (Plan 54-03 revision), button stays enabled during cooldown and clicking shows toast with remaining time.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| PostgreSQL | Backend (UserRepository queries) | ✓ | 16 | — |
| Redis | Rate limiting (future: optional) | ✓ | 7 | In-memory HashMap for alpha |
| Resend API | Email sending (verification emails) | ✓ (via ENV) | — | — |
| Node.js | Frontend build/dev | ✓ | v20 | — |
| Rust/Cargo | Backend build | ✓ | — | — |

**Missing dependencies with no fallback:**
- None — all dependencies are existing project infrastructure

**Missing dependencies with fallback:**
- Redis for rate limiting → In-memory HashMap (acceptable for alpha)

## Validation Architecture

> Skipped — `workflow.nyquist_validation` is explicitly `false` in `.planning/config.json`.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Existing JWT auth + `AuthUser` extractor |
| V3 Session Management | yes | HttpOnly cookies for JWT access/refresh tokens |
| V4 Access Control | yes | `VerifiedUser` extractor for gated routes (D-10, D-11) |
| V5 Input Validation | yes | Token validation in `verify_email` and `resend_verification` handlers |
| V6 Cryptography | no | No new cryptographic operations |

### Known Threat Patterns for Phase 54

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Verification token brute-force | Tampering | UUID v4 tokens (unpredictable), `verification_expires` (24h TTL). Rate limiting on verify endpoint not strictly needed but nice-to-have. |
| Resend email abuse (SPAM) | Denial of Service | D-05 (60s cooldown) + D-06 (5 max attempts). Prevents automated abuse of the resend endpoint. |
| Stale verification token reuse | Tampering | Token cleared on successful verification. `is_verification_valid()` checks expiry. D-06 specifies token expires after max resend attempts. |
| Frontend bypass of gating | Elevation of Privilege | D-10 mandates defense in depth: `VerifiedUser` extractor on ALL backend routes for gated features. Frontend gating is UX-only. |
| Email change without verification | Tampering | D-14 requires new email to be verified before the switch. Old email stays verified and features remain accessible. |

## Sources

### Primary (HIGH confidence)
- [CODE] `api/src/domain/auth/middleware.rs` — `AuthUser` extractor pattern for `VerifiedUser`
- [CODE] `api/src/presentation/handlers/auth_handlers.rs` — Existing verify_email, register, oauth handlers
- [CODE] `api/src/domain/user/model.rs` — User model with verification fields
- [CODE] `api/src/domain/user/sqlx_repository.rs` — User CRUD with all columns
- [CODE] `api/src/presentation/responses/api_response.rs` — ApiError with details/action support
- [CODE] `api/src/infrastructure/email/mod.rs` — EmailService with send_verification method
- [CODE] `api/src/presentation/routes/api_routes.rs` — Route definitions at /api/v1/auth
- [CODE] `app/src/store/authStore.js` — Zustand auth store pattern
- [CODE] `app/src/api/auth.js` — Auth API client
- [CODE] `app/src/api/client.js` — fetchApi with /api/v1 base URL
- [CODE] `app/src/pages/auth/VerifyEmailPage.jsx` — Existing page with URL bug
- [CODE] `app/src/app/App.jsx` — Route definitions
- [CODE] `app/src/components/ProtectedRoute.jsx` — Auth gating component pattern
- [CODE] `.planning/config.json` — Config with nyquist_validation: false

### Secondary (MEDIUM confidence)
- [CODE] `api/migrations/20260324000001_create_users_table.sql` — Users table schema
- [CODE] `api/migrations/20260405000001_add_password_reset_columns.sql` — Verification columns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all verified from codebase inspection
- Architecture: HIGH — patterns established from existing code (AuthUser, Zustand stores, API client)
- Pitfalls: HIGH — URL bug confirmed in code, OAuth auto-verify confirmed missing, race condition identified

**Research date:** 2026-05-30
**Valid until:** 2026-06-30 (stable codebase, no fast-moving dependencies)
