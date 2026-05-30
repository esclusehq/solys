---
phase: 54-email-verification
plan: 01
subsystem: auth
tags: email-verification, verified-user, axum-extractor, rate-limiting, pending-email, resend

requires:
  - phase: 49-fix-login-functionality-in-landing-page
    provides: OAuth login infrastructure, User model with verification fields
  - phase: 53-user-profile-management
    provides: User model with profile fields, repository patterns

provides:
  - pending_email column on users table for email change flow
  - VerifiedUser Axum extractor for backend gating
  - POST /auth/resend-verification endpoint with in-memory rate limiting
  - POST /auth/change-email endpoint with pending-email pattern
  - verify_email handler extended with pending_email switch logic
  - OAuth auto-verification on registration (D-13)
  - ResendTracker rate limiter (60s cooldown, max 5 attempts)

affects:
  - 54-02 (frontend auth extensions consume these endpoints)
  - 54-03 (frontend components use VerifiedUser pattern for gating)
  - 54-04 (email change form in frontend SettingsPage)
  - 54-05 (backend gating of existing route handlers)

tech-stack:
  added: []
  patterns:
    - Custom Axum FromRequestParts extractor with DB query (VerifiedUser)
    - In-memory rate limiting with Arc<RwLock<HashMap>> (ResendTracker)
    - LazyLock static for shared application state in handlers
    - pending-email pattern for email changes (D-14)

key-files:
  created:
    - api/migrations/20260530000001_add_pending_email.sql
  modified:
    - api/src/domain/user/model.rs
    - api/src/domain/user/repository.rs
    - api/src/domain/user/sqlx_repository.rs
    - api/src/domain/auth/middleware.rs
    - api/src/presentation/handlers/auth_handlers.rs

key-decisions:
  - "VerifiedUser extractor queries DB for email_verified_at on every request (acceptable for alpha; can cache in JWT claims later)"
  - "ResendTracker uses in-memory HashMap (acceptable for alpha; upgrade to Redis-backed if multi-instance needed)"
  - "ResendTracker stored as std::sync::LazyLock static in handler module (simplest approach without modifying AppContainer)"
  - "OAuth auto-verify sets email_verified_at on new User before creation (requires mut User)"

requirements-completed: []

duration: 5 min
completed: 2026-05-30
---

# Phase 54: Email Verification — Plan 01 Summary

**Backend infrastructure for email verification: pending_email migration, VerifiedUser extractor, resend/change-email endpoints with rate limiting, and OAuth auto-verification**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-30T10:39:45Z
- **Completed:** 2026-05-30T10:44:52Z
- **Tasks:** 3
- **Files modified:** 6 (1 new, 5 modified)

## Accomplishments

- Created pending_email migration with ALTER TABLE and index
- Added `pending_email: Option<String>` to User struct and constructor default
- Added `find_by_pending_email()` to UserRepository trait and SqlxUserRepository impl
- Updated `update()` SET clause to include `pending_email = $15`
- Created `VerifiedUser` struct + `From<Claims>` + `FromRequestParts<S>` impl in middleware.rs
- VerifiedUser extractor validates JWT (Bearer header or cookie), queries DB for `email_verified_at`, returns `ApiError::forbidden("email_not_verified")` with resend metadata if unverified
- Modified OAuth handler to auto-verify new OAuth users by setting `email_verified_at = now()`
- Built `ResendTracker` with `check()` method implementing 60s cooldown and max 5 attempts via `Arc<RwLock<HashMap>>`
- Added `POST /auth/resend-verification` handler with rate limiting, token regeneration, and email sending
- Added `POST /auth/change-email` handler with validation, pending_email storage, verification token generation, and email sending to new address
- Modified `verify_email` handler to switch `user.email` to `pending_email` when set (D-14 pattern)
- Registered both new routes in `AuthHandlers::router()`
- `cargo check` passes with no new errors

## Task Commits

Each task was committed atomically (in the `api/` submodule):

1. **Task 1: pending_email migration, model field, repository methods** — `48458c4` (feat)
2. **Task 2: VerifiedUser extractor + OAuth auto-verification** — `1f483ae` (feat)
3. **Task 3: resend_verification handler, change_email handler, verify_email modification** — `d01d86b` (feat)

## Files Created/Modified

### New

- `api/migrations/20260530000001_add_pending_email.sql` — ALTER TABLE users ADD COLUMN pending_email + index

### Modified

- `api/src/domain/user/model.rs` — Added `pending_email: Option<String>` field + constructor default
- `api/src/domain/user/repository.rs` — Added `find_by_pending_email(&self, email) -> Result<Option<User>>` trait method
- `api/src/domain/user/sqlx_repository.rs` — Implemented `find_by_pending_email`, updated `update()` SET clause with `pending_email = $15`
- `api/src/domain/auth/middleware.rs` — Added `VerifiedUser` struct, `From<Claims>` impl, `FromRequestParts<S>` impl with DB query for `email_verified_at`
- `api/src/presentation/handlers/auth_handlers.rs` — Added imports, `ResendTracker`/`ResendState` structs, `resend_verification` handler, `change_email` handler, `ChangeEmailRequest` struct, pending_email switch in `verify_email`, OAuth auto-verify, route registration

## Decisions Made

- **ResendTracker as LazyLock static:** Used `std::sync::LazyLock<ResendTracker>` in the handler module rather than modifying `AppContainer` or `ApiState`. Simplest approach that avoids structural changes — the in-memory HashMap is acceptable for alpha and resets on server restart (documented limitation).
- **VerifiedUser rejection type = ApiError:** Reuses the existing `ApiError` type (with `.forbidden()` and `.with_details()`) instead of a custom rejection enum. Simplifies error handling and produces consistent error responses with resend metadata.
- **OAuth auto-verify requires mut User:** The plan showed immutable `let new_user = ...`, but setting `email_verified_at` requires `let mut new_user`. Changed to mutable binding (Rule 1 auto-fix).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] OAuth new_user needs to be mutable**
- **Found during:** Task 2 (OAuth auto-verification)
- **Issue:** Plan showed `let new_user = User::new(...)` but `new_user.email_verified_at = Some(...)` requires mutable binding
- **Fix:** Changed `let` to `let mut`
- **Files modified:** `api/src/presentation/handlers/auth_handlers.rs`
- **Verification:** `cargo check` passes
- **Committed in:** `1f483ae` (Task 2 commit)

**2. [Rule 1 - Bug] ChangeEmailRequest cannot be nested inside impl block**
- **Found during:** Task 3 implementation
- **Issue:** Plan placed `pub struct ChangeEmailRequest` inside `impl AuthHandlers { ... }` block, which is not valid Rust — struct definitions must be at module level
- **Fix:** Moved `ChangeEmailRequest` to module level alongside other request structs
- **Files modified:** `api/src/presentation/handlers/auth_handlers.rs`
- **Verification:** `cargo check` passes
- **Committed in:** `d01d86b` (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes necessary for compilation. No scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backend infrastructure for email verification complete
- Ready for Plan 54-02 (Frontend Auth Infrastructure: authStore + API client extensions)
- Ready for Plan 54-03 (Frontend Verification Components: VerifyEmailPage fix, Banner, Dialog)

## Self-Check: PASSED

- ✅ Migration file exists at `api/migrations/20260530000001_add_pending_email.sql`
- ✅ `User` struct has `pending_email: Option<String>` field (2 instances confirmed)
- ✅ `UserRepository` trait has `find_by_pending_email()` (1 instance confirmed)
- ✅ `SqlxUserRepository` implements `find_by_pending_email()` (1 instance) and `update()` includes `pending_email = $15` (1 instance confirmed)
- ✅ `VerifiedUser` struct exists in `middleware.rs` (1 instance confirmed)
- ✅ `FromRequestParts for VerifiedUser` exists in `middleware.rs` (1 instance confirmed)
- ✅ OAuth handler sets `email_verified_at` via `Some(chrono::Utc::now())` (2 instances: verify_email + OAuth)
- ✅ `ResendTracker` struct exists (1 instance confirmed)
- ✅ `resend_verification` handler in auth_handlers.rs (2 references confirmed — impl + route)
- ✅ `change_email` handler in auth_handlers.rs (2 references confirmed — impl + route)
- ✅ `pending_email.take()` in verify_email handler (1 instance confirmed)
- ✅ All 3 commits found in api submodule git log
- ✅ `cargo check` passes with no new errors

---
*Phase: 54-email-verification*
*Completed: 2026-05-30*
