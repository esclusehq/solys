# Phase 54: Email Verification Flow - Pattern Map

**Mapped:** 2026-05-30
**Files analyzed:** 13 (6 backend + 7 frontend)
**Analogs found:** 13 / 13

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `api/src/domain/auth/middleware.rs` | middleware (extractor) | request-response | existing `AuthUser` impl (same file, lines 16-92) | exact |
| `api/src/presentation/handlers/auth_handlers.rs` | controller | request-response | existing `verify_email` (lines 621-650) + `forgot_password` (lines 537-582) | exact |
| `api/src/presentation/routes/api_routes.rs` | route/config | CRUD | existing route nesting pattern (lines 28, 34-36) | exact |
| `api/migrations/` | migration | file-I/O (DDL) | `20260405000001_add_password_reset_columns.sql` | exact |
| `api/src/domain/user/model.rs` | model | CRUD | existing `User` struct (lines 6-24) | exact |
| `api/src/domain/user/repository.rs` | model/interface | CRUD | existing `UserRepository` trait (lines 7-21) | exact |
| `api/src/domain/user/sqlx_repository.rs` | model/data-access | CRUD | existing `update` method (lines 105-137) | exact |
| `app/src/store/authStore.js` | store | CRUD | existing `verifyEmail` in `auth.js` + `forgotPassword` in authStore (lines 85-94) | exact |
| `app/src/api/auth.js` | api-client | request-response | existing `verifyEmail` function (lines 47-52) | exact |
| `app/src/pages/auth/VerifyEmailPage.jsx` | component/page | request-response | existing file itself (lines 1-79) | exact |
| `app/src/components/EmailVerificationBanner.jsx` | component | event-driven (UI) | `ToastContainer.jsx` (UI pattern) + `Onboarding.jsx` (global component pattern) | partial |
| `app/src/components/EmailVerificationDialog.jsx` | component/modal | event-driven (UI) | no existing modal/AuthUser gating component — use patterns from `uiStore.js` `openModal/closeModal` (lines 56-58) | no-exact |
| `app/src/app/App.jsx` | component/layout | request-response | existing `App.jsx` layout + `ProtectedRoute` pattern (lines 54-116) | exact |

## Pattern Assignments

### `api/src/domain/auth/middleware.rs` — ADD `VerifiedUser` extractor

**Analog:** Existing `AuthUser` extractor (same file, lines 16-92)

**Imports pattern** (lines 1-14):
```rust
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRef, FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::auth::service::{Claims, JwtService};
use crate::presentation::routes::api_routes::ApiState;
```

**AuthUser struct pattern** (lines 16-28):
```rust
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role: String,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == "admin" || self.role == "owner"
    }
}
```

**From<Claims> conversion pattern** (lines 30-39):
```rust
impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.user_id,
            tenant_id: claims.tenant_id,
            email: claims.sub.clone(),
            role: claims.role.clone(),
        }
    }
}
```

**FromRequestParts implementation pattern** (lines 41-93) — core extractor logic:
```rust
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    ApiState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = ApiState::from_ref(state);
        
        // First try Authorization header
        let auth_header = parts.headers.get("Authorization")
            .and_then(|v| v.to_str().ok());

        if let Some(token) = auth_header.and_then(|auth| auth.strip_prefix("Bearer ")) {
            let jwt = JwtService::new(state.jwt_secret.clone());
            if let Ok(claims) = jwt.validate_token(token) {
                return Ok(AuthUser::from(claims));
            }
        }
        
        // Then try cookies
        let cookies = parts.headers.get("Cookie")
            .and_then(|v| v.to_str().ok());
        
        if let Some(cookie_str) = cookies {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("access_token=") {
                    let token = cookie.strip_prefix("access_token=").unwrap_or("");
                    let jwt = JwtService::new(state.jwt_secret.clone());
                    if let Ok(claims) = jwt.validate_token(token) {
                        return Ok(AuthUser::from(claims));
                    }
                }
                // ... refresh_token check similarly ...
            }
        }

        Err(AuthError::MissingToken)
    }
}
```

**Error type + IntoResponse pattern** (lines 112-130):
```rust
#[derive(Debug, Clone, Serialize)]
pub enum AuthError {
    InvalidToken,
    MissingToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (code, message) = match self {
            AuthError::InvalidToken => ("INVALID_TOKEN", "Invalid or expired token"),
            AuthError::MissingToken => ("MISSING_TOKEN", "Authorization header missing"),
        };
        let body = serde_json::json!({
            "success": false,
            "error": { "code": code, "message": message }
        });
        (StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
    }
}
```

**Key difference for `VerifiedUser`:** Must add a DB query for `email_verified_at` after JWT extraction, returning `ApiError::forbidden("email_not_verified")` with resend details if unverified. Use `api/src/presentation/responses/api_response.rs` `ApiError::forbidden()` (line 87-95) + `.with_details()` (line 67-70) pattern.

---

### `api/src/presentation/handlers/auth_handlers.rs` — ADD `resend_verification` handler + OAuth auto-verify

**Analog 1:** `verify_email` handler (lines 621-650) — for token verification pattern
**Analog 2:** `forgot_password` handler (lines 537-582) — for rate-limiting + email-sending pattern
**Analog 3:** `register` handler (lines 75-146) — for token generation + email-sending pattern

**Handler imports pattern** (lines 1-17):
```rust
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};

use crate::domain::auth::middleware::AuthUser;
use crate::domain::auth::service::{JwtService, PasswordService, LoginRequest, RegisterRequest, RefreshTokenRequest, OAuthRequest};
use crate::domain::user::model::User;
use crate::domain::user::sqlx_repository::SqlxUserRepository;
use crate::domain::user::repository::UserRepository;
use crate::presentation::responses::api_response::{ApiResponse, ApiError};
use crate::presentation::responses::cookie::{CookieBuilder, delete_auth_cookies, AuthResponse};
use crate::presentation::routes::api_routes::ApiState;
```

**Request struct pattern** (lines 31-35):
```rust
#[derive(serde::Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}
```

**Existing verify_email handler pattern** (lines 621-650) — exact pattern for token validation:
```rust
pub async fn verify_email(
    State(state): State<ApiState>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<axum::response::Response, ApiError> {
    if req.token.is_empty() {
        return Err(ApiError::new("VALIDATION_ERROR", "Token is required"));
    }

    let repo = SqlxUserRepository::new(state.pool.clone());
    
    let mut user = repo.find_by_verification_token(&req.token)
        .await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", &e.to_string()))?
        .ok_or_else(|| ApiError::new("INVALID_TOKEN", "Invalid verification token"))?;

    if !user.is_verification_valid() {
        return Err(ApiError::new("EXPIRED_TOKEN", "Verification token has expired"));
    }

    user.email_verified_at = Some(chrono::Utc::now());
    user.verification_token = None;
    user.verification_expires = None;

    repo.update(&user).await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", &e.to_string()))?;

    Ok(ApiResponse::success(serde_json::json!({
        "message": "Email verified successfully"
    })).into_response())
}
```

**Email sending pattern (for resend)** — from `register` handler (lines 107-119) and `forgot_password` (lines 564-577):
```rust
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
```

**Verification token generation** — from `register` handler (lines 100-101):
```rust
let verification_token = uuid::Uuid::new_v4().to_string();
user.set_verification_token(verification_token.clone());
```

**ApiError response with details** — from `api_response.rs` (lines 67-70, 87-95):
```rust
// For email_not_verified error with resend data:
Err(ApiError::forbidden("Verify your email before accessing this resource.")
    .with_details(serde_json::json!({
        "error": "email_not_verified",
        "can_resend": true,
        "cooldown_remaining": 0,
        "attempts_remaining": 5,
    })))
```

**OAuth user creation pattern** (lines 481-491) — where to add `email_verified_at = now()`:
```rust
None => {
    let password_hash = PasswordService::hash(
        &format!("oauth_{}", uuid::Uuid::new_v4()), 12
    ).map_err(|e| ApiError::new("INTERNAL_ERROR", &e.to_string()))?;
    
    let new_user = User::new(email.clone(), password_hash);
    // D-13: ADD: new_user.email_verified_at = Some(chrono::Utc::now());
    repo.create(&new_user)
        .await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", &e.to_string()))?
}
```

**Route registration pattern** (lines 56-73) — new route for resend:
```rust
pub fn router(state: ApiState) -> Router<ApiState> {
    Router::new()
        .route("/register", post(Self::register))
        .route("/login", post(Self::login))
        .route("/verify-email", post(Self::verify_email))
        .route("/resend-verification", post(Self::resend_verification))  // NEW
        .route("/forgot-password", post(Self::forgot_password))
        // ...
        .with_state(state)
}
```

---

### `api/src/presentation/routes/api_routes.rs` — Mount new route

**Analog:** Existing route definitions (lines 21-107)

**Route nesting pattern** (lines 26-28):
```rust
let api_router = Router::new()
    .nest("/api/v1/auth", AuthHandlers::router(state.clone()))
    .nest("/api/v1/billing", BillingHandlers::router(state.clone()));
```

No changes needed to `api_routes.rs` if the new route is added inside `AuthHandlers::router()` — that router is already nested at `/api/v1/auth`.

---

### `api/migrations/` — NEW: Add `pending_email` column

**Analog:** `20260405000001_add_password_reset_columns.sql` (10 lines)

**Migration pattern:**
```sql
-- Migration: Add pending email column for email change flow
-- Description: D-14: pending-email pattern for email changes

ALTER TABLE users ADD COLUMN IF NOT EXISTS pending_email VARCHAR(255);

CREATE INDEX IF NOT EXISTS idx_users_pending_email ON users(pending_email);
```

---

### `api/src/domain/user/model.rs` — ADD `pending_email` field

**Analog:** Existing User struct (lines 5-24)

**Field addition pattern** (add `pending_email` alongside existing Optional fields):
```rust
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    // ... existing fields ...
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub pending_email: Option<String>,        // NEW
    pub scheduled_deletion_at: Option<DateTime<Utc>>,
}
```

**Constructor default pattern** (add into `User::new()`, line 47):
```rust
pending_email: None,
```

---

### `api/src/domain/user/repository.rs` — ADD `find_by_pending_email`

**Analog:** Existing trait methods (lines 7-21)

**New method pattern:**
```rust
async fn find_by_pending_email(&self, email: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
```

---

### `api/src/domain/user/sqlx_repository.rs` — ADD `pending_email` support

**Analog:** Existing `find_by_verification_token` method (lines 84-93) + `update` method (lines 105-137)

**New find_by_pending_email implementation:**
```rust
async fn find_by_pending_email(&self, email: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE pending_email = $1 AND deleted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(&self.pool)
    .await?;
    Ok(result)
}
```

**Update query changes** (line 108-117) — add `pending_email = $15` to SET clause:
```sql
UPDATE users 
SET email = $2, password_hash = $3, role = $4, is_active = $5, 
    email_verified_at = $6, last_login_at = $7, 
    password_reset_token = $8, password_reset_expires = $9,
    verification_token = $10, verification_expires = $11,
    display_name = $12, avatar_url = $13, scheduled_deletion_at = $14,
    pending_email = $15,                             -- NEW
    updated_at = NOW()
WHERE id = $1 AND deleted_at IS NULL
RETURNING *
```

---

### `app/src/store/authStore.js` — ADD verification state + resend action

**Analog:** Existing `forgotPassword` action (lines 85-94) and `register` action (lines 52-67)

**Store creation + persist pattern** (lines 1-196):
```javascript
import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { fetchApi } from '../api/client'
import { useUIStore } from './uiStore'
import * as authApi from '../api/auth'

export const useAuthStore = create(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
```

**Existing action pattern** — `forgotPassword` (lines 85-94):
```javascript
forgotPassword: async (email) => {
  set({ isLoading: true, error: null })
  try {
    await authApi.forgotPassword(email)
    set({ isLoading: false })
  } catch (err) {
    set({ error: err.message, isLoading: false })
    throw err
  }
},
```

**New action pattern to add — `resendVerification`:**
```javascript
resendVerification: async () => {
  set({ isLoading: true, error: null })
  try {
    const result = await authApi.resendVerification()
    set({ isLoading: false })
    // D-07: show toast on success
    useUIStore.getState().addToast({ 
      type: 'success', 
      message: 'Verification email sent!' 
    })
    return result
  } catch (err) {
    set({ error: err.message, isLoading: false })
    throw err
  }
},
```

**New derived state / after `checkAuth()` refresh user state** (lines 15-33):
```javascript
// checkAuth already returns email_verified from the `/me` endpoint
// The `user` object already has `email_verified` boolean
// No new state needed — use `user?.email_verified` directly
```

**partialize pattern for persistence** (lines 191-194):
```javascript
partialize: (state) => ({
  user: state.user,
  isAuthenticated: state.isAuthenticated,
}),
```

---

### `app/src/api/auth.js` — ADD `resendVerification`

**Analog:** Existing `verifyEmail` function (lines 47-52)

**Function pattern:**
```javascript
import { fetchApi } from './client';

export async function verifyEmail(token) {
    return fetchApi('/auth/verify-email', {
        method: 'POST',
        body: JSON.stringify({ token })
    });
}

export async function resendVerification() {
    return fetchApi('/auth/resend-verification', {
        method: 'POST',
    });
}
```

**fetchApi client pattern** (`client.js` lines 1-37):
```javascript
const API_BASE = '/api/v1';

export async function fetchApi(endpoint, options = {}) {
    const headers = { ...options.headers, };
    if (!(options.body instanceof FormData) && !headers['Content-Type']) {
        headers['Content-Type'] = 'application/json';
    }
    const url = `${API_BASE}${endpoint}`;
    const res = await fetch(url, {
        ...options, headers, credentials: 'include',
    });
    // ... error handling + response parsing ...
    if (!json.success && json.success !== undefined) {
        throw new Error(json?.error?.message || json?.message || 'API Error');
    }
    return json.data ?? json;
}
```

---

### `app/src/pages/auth/VerifyEmailPage.jsx` — FIX URL bug + add `returnTo` redirect

**Analog:** Existing file itself (lines 1-79)

**Current bug pattern** (lines 24-29) — **MUST FIX**:
```javascript
// BUG: raw fetch without /api/v1 prefix
const response = await fetch('/api/auth/verify-email', {
```

**Fix to use `authApi.verifyEmail`** (replaces lines 22-44):
```javascript
import { useSearchParams, useNavigate, Link } from 'react-router-dom'
import { useAuthStore } from '../../store/authStore'
import { useUIStore } from '../../store/uiStore'
import * as authApi from '../../api/auth'     // ADD this import

// ...inside component:
const returnTo = searchParams.get('returnTo') || '/dashboard'  // D-04

useEffect(() => {
  if (!token) {
    setStatus('error')
    addToast({ type: 'error', message: 'Invalid verification link' })
    return
  }

  const verifyEmail = async () => {
    try {
      const result = await authApi.verifyEmail(token)   // FIXED
      setStatus('success')
      addToast({ type: 'success', message: 'Email verified successfully!' })
      await checkAuth()   // Refresh user state (D-04 pitfall fix)
      setTimeout(() => navigate(returnTo), 2500)  // D-04: auto-redirect
    } catch (err) {
      setStatus('error')
      addToast({ type: 'error', message: err.message || 'Verification failed' })
    }
  }

  verifyEmail()
}, [token, navigate, addToast, checkAuth, returnTo])
```

**Success page UI** (D-04, lines 61-66) — update to show auto-redirect + manual fallback button:
```jsx
{status === 'success' ? (
  <>
    <div className="text-green-400 text-5xl mb-4">✓</div>
    <h1 className="text-2xl font-bold text-white mb-2">Email Verified!</h1>
    <p className="text-gray-400 mb-6">
      Redirecting to {returnTo === '/dashboard' ? 'dashboard' : 'previous page'}...
    </p>
    <Link to={returnTo} className="text-blue-400 hover:underline">
      Click here if not redirected
    </Link>
  </>
) : (
  // ...error state unchanged...
)}
```

---

### `app/src/components/EmailVerificationBanner.jsx` — NEW component

**Analog:** `ToastContainer.jsx` (lines 1-30) for UI pattern + store integration

**UI pattern from ToastContainer** (lines 6-28):
```jsx
import { useAuthStore } from '../store/authStore'
import { useUIStore } from '../store/uiStore'

export default function EmailVerificationBanner() {
  const { user, resendVerification } = useAuthStore()
  const { addToast } = useUIStore()
  
  // D-02: Only show if user is authenticated AND email not verified
  if (!user || user.email_verified) return null
  
  // ...cooldown state + countdown timer per D-07...
  // ...[Resend Email] button + [Change Email] link per D-02...
  // ...max attempts check per D-06...
  
  return (
    <div className="bg-yellow-500/10 border-b border-yellow-500/20 px-4 py-2">
      <div className="flex items-center justify-between max-w-7xl mx-auto">
        <p className="text-yellow-400 text-sm">
          ⚠ Verify your email — Check your inbox at <strong>{user.email}</strong>
        </p>
        <div className="flex items-center gap-3">
          <button onClick={handleResend} disabled={cooldown > 0 || maxedOut}
            className="text-yellow-400 hover:text-yellow-300 text-sm underline disabled:opacity-50">
            {maxedOut ? 'Contact support' : cooldown > 0 ? `Resend in ${cooldown}s` : 'Resend Email'}
          </button>
          <Link to="/settings" className="text-gray-400 hover:text-white text-sm">
            Change Email
          </Link>
        </div>
      </div>
    </div>
  )
}
```

---

### `app/src/components/EmailVerificationDialog.jsx` — NEW modal component

**Analog:** No existing modal component. Pattern from `uiStore.js` `openModal/closeModal` (lines 56-58) + dialog overlay from tailwind patterns.

**uiStore modal pattern** (lines 56-58):
```javascript
openModal: (modal) => set({ modal }),
closeModal: () => set({ modal: null }),
```

**Dialog pattern:**
```jsx
import { useUIStore } from '../store/uiStore'
import { useAuthStore } from '../store/authStore'

export default function EmailVerificationDialog({ onClose }) {
  const { resendVerification } = useAuthStore()
  const { addToast } = useUIStore()
  const [cooldown, setCooldown] = useState(0)
  const [maxedOut, setMaxedOut] = useState(false)

  return (
    <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
      <div className="bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
        <h2 className="text-lg font-semibold text-white mb-2">
          Email Verification Required
        </h2>
        <p className="text-gray-400 mb-6">
          Verify your email before using this feature.
        </p>
        {/* Resend button + cooldown per D-07 */}
        <div className="flex justify-end gap-3">
          <button onClick={onClose}
            className="px-4 py-2 text-gray-400 hover:text-white">
            Cancel
          </button>
          {/* Resend button */}
        </div>
      </div>
    </div>
  )
}
```

---

### `app/src/app/App.jsx` — ADD banner + feature gating

**Analog:** Existing App.jsx layout (lines 54-116) + ProtectedRoute pattern

**Banner placement pattern** (around line 103, above the header area):
```jsx
<ProtectedRoute>
  {!isOnboarded && <Onboarding />}
  <EmailVerificationBanner />           {/* NEW: global banner D-02 */}
  <div className="flex min-h-screen bg-gray-900">
```

**Feature gating pattern** — extend `ProtectedRoute` or add wrapper for gated pages:
```jsx
// In the inner Routes, for gated routes like /billing:
<Route path="/billing" element={
  <RequireVerifiedEmail>
    <BillingPage />
  </RequireVerifiedEmail>
} />
```

Or reuse/extend the `ProtectedRoute` pattern from `app/src/components/ProtectedRoute.jsx` (lines 1-33):
```jsx
import { useEffect, useState } from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '../store/authStore'

export default function ProtectedRoute({ children }) {
  const { isAuthenticated, checkAuth } = useAuthStore()
  const location = useLocation()
  const [checking, setChecking] = useState(true)

  useEffect(() => {
    const initAuth = async () => {
      if (!isAuthenticated) {
        await checkAuth()
      }
      setChecking(false)
    }
    initAuth()
  }, [checkAuth, isAuthenticated])

  if (checking) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-900">
        <div className="text-gray-400">Checking authentication...</div>
      </div>
    )
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  return children
}
```

---

## Shared Patterns

### Authentication / VerifiedUser Extractor
**Source:** `api/src/domain/auth/middleware.rs` — AuthUser `FromRequestParts` impl (lines 41-93)
**Apply to:** `VerifiedUser` extractor (new code in same file)

The pattern is:
1. Extract `ApiState` from `FromRef<S>`
2. Check `Authorization: Bearer <token>` header, validate JWT
3. Fall back to `Cookie: access_token=...` extraction
4. For `VerifiedUser`: after extracting `AuthUser`, query DB for `email_verified_at`
5. Return `ApiError` rejection if not verified (with resend details via `.with_details()`)

**Rejection type:** `ApiError` from `api/src/presentation/responses/api_response.rs` — supports `.forbidden()`, `.with_details()`, `.with_action()`.

### Error Response Format (D-12)
**Source:** `api/src/presentation/responses/api_response.rs` lines 67-70, 87-95
**Apply to:** `VerifiedUser` extractor + `resend_verification` handler

```rust
// "email_not_verified" error with resend metadata:
Err(ApiError::forbidden("Verify your email before accessing this resource.")
    .with_details(serde_json::json!({
        "error": "email_not_verified",
        "can_resend": true,
        "cooldown_remaining": 0,
        "attempts_remaining": 5,
    })))
```

### Rate Limiting Pattern (D-05, D-06)
**Source:** Custom implementation for resend endpoint — use `Arc<RwLock<HashMap<Uuid, ResendState>>>` in Handler
**Apply to:** `resend_verification` handler

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ResendTracker {
    pub inner: Arc<RwLock<HashMap<uuid::Uuid, ResendState>>>,
}

#[derive(Clone)]
pub struct ResendState {
    pub last_sent: std::time::Instant,
    pub count: u32,
}

impl ResendTracker {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn check(&self, user_id: uuid::Uuid) -> Result<(), (u32, u32)> {
        let mut map = self.inner.write().await;
        let state = map.entry(user_id).or_insert(ResendState {
            last_sent: std::time::Instant::now(),
            count: 0,
        });
        
        if state.count >= 5 {
            return Err((60, 0)); // maxed out
        }
        
        let elapsed = state.last_sent.elapsed().as_secs();
        if elapsed < 60 {
            return Err((60 - elapsed as u32, 5 - state.count));
        }
        
        state.last_sent = std::time::Instant::now();
        state.count += 1;
        Ok(())
    }
}
```

### Frontend Verification State Pattern
**Source:** `app/src/store/authStore.js` — existing state derivation pattern
**Apply to:** authStore extension

The `/me` endpoint (auth_handlers.rs line 277) already returns `email_verified: user.email_verified_at.is_some()`.
The `user` object in the store already contains this data after `checkAuth()`.
No new state field needed — use `user?.email_verified` to determine if banner shows.

### Countdown/Timer Pattern (D-07)
**Source:** Standard React `setInterval` pattern
**Apply to:** `EmailVerificationBanner.jsx` and `EmailVerificationDialog.jsx`

```javascript
const [cooldown, setCooldown] = useState(0)

useEffect(() => {
  if (cooldown <= 0) return
  const timer = setInterval(() => {
    setCooldown(prev => prev - 1)
  }, 1000)
  return () => clearInterval(timer)
}, [cooldown])
```

### Toast Notification Pattern
**Source:** `app/src/store/uiStore.js` lines 21-27 + `app/src/components/ToastContainer.jsx`
**Apply to:** All frontend components showing feedback

```javascript
import { useUIStore } from '../store/uiStore'

const { addToast } = useUIStore()
addToast({ type: 'success', message: 'Verification email sent!' })
addToast({ type: 'error', message: 'Something went wrong' })
```

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns + code excerpts above):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `app/src/components/EmailVerificationBanner.jsx` | component | event-driven (UI) | No existing global banner component — closest is `ToastContainer.jsx` (totally different UX, but same store import pattern) |
| `app/src/components/EmailVerificationDialog.jsx` | component/modal | event-driven (UI) | No existing modal/dialog component in the codebase — build from scratch using `uiStore.js` modal state |

## Metadata

**Analog search scope:** All files under `api/src/`, `app/src/`, `api/migrations/`
**Files scanned:** ~15 key analog files read
**Pattern extraction date:** 2026-05-30
