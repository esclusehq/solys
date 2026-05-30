# Phase 54: Email Verification Flow - Context

**Gathered:** 2026-05-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver email verification for users who sign up with email — send verification email on registration (backend already does this), provide resend option with rate limiting, show verification status in UI, gate sensitive actions behind verified email. OAuth users are auto-verified (provider already verified the email). Email changes use a pending-email pattern (switch only after new email is verified).

Backend infrastructure already exists: `verify_email` handler, Resend email sending via `RESEND_API_KEY`, DB columns (`email_verified_at`, `verification_token`, `verification_expires`). Frontend `VerifyEmailPage.jsx` exists but has a URL bug (calls `/api/auth/verify-email` instead of `/api/v1/auth/verify-email`).

</domain>

<decisions>
## Implementation Decisions

### Post-Registration UX
- **D-01:** Hybrid flow — signup auto-logs in the user (as now), then shows a persistent global banner prompting email verification immediately. No separate "Check your email" page.
- **D-02:** Global banner — small banner on all pages: `⚠ Verify your email — Check your inbox at user@example.com [Resend Email] [Change Email]`. The [Change Email] link navigates to profile settings email change form.
- **D-03:** Feature-blocking dialog — when an unverified user tries to access a gated feature, show a modal: "Email verification required — Verify your email before using this feature. [Resend Email]"
- **D-04:** Verification success — show brief success page → 2–3 second auto-redirect → if `returnTo` URL param exists, redirect there; otherwise redirect to dashboard. Manual button as fallback.

### Resend Verification
- **D-05:** Rate limiting — 60-second cooldown between resends.
- **D-06:** Hard cap — maximum 5 resend attempts total per registration. After cap, verification token expires and user sees "Contact support" message.
- **D-07:** Cooldown UX — button stays visible during cooldown. Clicking during cooldown shows tooltip with remaining time. Successful send shows toast notification.

### Sensitive Action Gating
- **D-08:** Gated feature categories (require `email_verified_at != null`):
  - **Identity & Access:** API Keys, Personal Access Tokens, OAuth Applications, Team Invites, Team Creation
  - **Resource Creation:** Project Creation, Server Creation, Server Deployment, Node Registration, Instance Creation, Environment Creation
  - **Integration Features:** Webhooks, External Integrations, SDK Credentials
  - **Financial Features:** Billing, Subscription Changes, Invoice Management, Payment Methods
- **D-09:** Ungated features (always accessible without verification): Login, Dashboard, Profile Settings, Security Settings, Login History, Documentation, Pricing, Community/Discord, Account Preferences
- **D-10:** Enforcement — defense in depth: backend + frontend.
- **D-11:** Backend mechanism — custom Axum extractor `VerifiedUser` (analogous to existing `AuthUser`) that checks `email_verified_at`. Used on gated route handlers.
- **D-12:** Error response format — enhanced JSON: `{ error: "email_not_verified", message: "Verify your email before accessing this resource." }` with resend data so frontend can inline the resend dialog without a separate API call.

### OAuth and Verification
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend Auth (source of truth for existing infrastructure)
- `api/src/presentation/handlers/auth_handlers.rs` — all auth handlers: register (w/ verification token), verify_email, oauth, login, etc. The `register` handler already generates tokens and sends via Resend
- `api/src/presentation/routes/api_routes.rs` — route definitions where auth routes are mounted at `/api/v1/auth`
- `api/src/domain/auth/middleware.rs` — `AuthUser` extractor (pattern for creating `VerifiedUser`)
- `api/src/presentation/middleware/auth.rs` — alternative auth middleware (unused in current routes)

### Database Schema
- `api/migrations/20260324000001_create_users_table.sql` — users table with `email_verified_at` column
- `api/migrations/20260405000001_add_password_reset_columns.sql` — adds `verification_token`, `verification_expires` columns

### Frontend Auth
- `app/src/store/authStore.js` — Zustand auth store (needs extension: verification state, resend action, gating)
- `app/src/api/auth.js` — auth API client (has unused `verifyEmail` function — currently calls correct `/api/v1/auth/verify-email` path)
- `app/src/lib/supabase.js` — Supabase client for OAuth
- `app/src/pages/auth/VerifyEmailPage.jsx` — existing verification page (has URL bug: calls `/api/auth/verify-email` without `v1`)

### Codebase Maps
- `.planning/codebase/STACK.md` — tech stack (Supabase, React 19, Zustand, Axum)
- `.planning/codebase/ARCHITECTURE.md` — architecture (Clean Architecture, extractor patterns)
- `.planning/codebase/CONVENTIONS.md` — coding conventions (Zustand stores, Tailwind CSS v4)
- `.planning/codebase/INTEGRATIONS.md` — external integrations (Supabase, Resend)

### Roadmap
- `.planning/ROADMAP.md` §Phase 54 — phase goal: "Verifikasi email untuk users yang signup dengan email - send verification email on registration, resend verification option, require verified email for sensitive actions"

### Prior Phase Context
- `.planning/phases/49-fix-login-functionality-in-landing-page/` — OAuth login setup (Phase 54 depends on Phase 49)
- `.planning/phases/53-user-profile-management/53-CONTEXT.md` — profile management patterns (settings page, avatar upload, authStore patterns)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AuthUser` extractor at `api/src/domain/auth/middleware.rs` — pattern to replicate for `VerifiedUser` custom extractor
- `authStore.js` with Zustand + persist middleware — ready to extend with `emailVerified`, `resendVerification()` methods
- `auth.js` API client — has `verifyEmail()` function already defined but unused; add `resendVerification()`
- `VerifyEmailPage.jsx` — existing page component (needs API path fix and enhanced redirect logic with `returnTo`)
- `Supabase` client at `app/src/lib/supabase.js` — fully configured with PKCE flow

### Established Patterns
- Custom Axum extractors for authentication (`AuthUser` via `FromRequestParts`)
- Zustand stores with persist middleware for auth state
- HttpOnly cookies for JWT access/refresh tokens
- Backend email sending via Resend API (`RESEND_API_KEY` env var)
- Feature-gated route nesting (admin routes pattern)

### Integration Points
- Backend: add `VerifiedUser` extractor, add `/auth/resend-verification` endpoint, add `pending_email` column to users table
- Frontend: extend `authStore` with verification state, create global `EmailVerificationBanner` component, add gating logic to route definitions or feature components
- Gated routes: verify which existing route handlers need the new `VerifiedUser` extractor (API Keys, Billing, Team, Webhooks, Server creation, Node registration)
- OAuth handler: set `email_verified_at = now()` in the existing `oauth` handler

</code_context>

<specifics>
## Specific Ideas

Flow diagram for post-registration:
```
Signup → Auto-login → Dashboard
  ↓
⚠ Verify your email — Check your inbox at user@example.com [Resend] [Change Email]
  ↓
User clicks verification link in email
  ↓
POST /api/v1/auth/verify-email
  ↓
Success page → 2-3s → redirect to dashboard (or returnTo)
```

Flow diagram for email change:
```
Request email change → Keep old email verified → Store pending_email
  → Send verification to new email → Keep all features working
  → User verifies new email → Switch emails → Mark new email verified
```

Gating philosophy: gate based on **potential for resource abuse and cost**, not on feature importance. Identity, Resource Creation, Integration, and Financial features are gated. Read-only, profile, and information features are not.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 54-email-verification*
*Context gathered: 2026-05-30*
