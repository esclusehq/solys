# Phase 53: User Profile Management - Research

**Researched:** 2026-05-30
**Domain:** User profile management, Supabase Storage (avatars), PostgreSQL schema, API endpoints
**Confidence:** HIGH

## Summary

This phase extends the existing Profile tab in the Settings page with avatar upload (Supabase Storage), display name (separate from login identifier), login history (stored in a new Postgres table, tracked via backend middleware), and account deletion (soft delete with 14-day grace period). It also adds a user info area in the sidebar showing avatar, display name, and email.

The project has a well-established Rust/Axum backend with existing auth and user endpoints (`/api/v1/auth/*`, `/api/v1/users/*`). The `users` table already has a `deleted_at` column (soft delete support). The frontend has an existing SettingsPage with a tab-based layout and the current Profile tab shows name, email (disabled), and change password form.

Key new infrastructure: (1) a Postgres migration adding `display_name` and `avatar_url` columns to `users`, (2) a new `login_history` table, (3) a Supabase Storage bucket for avatars, (4) new backend handlers for self-deletion, transfer ownership, and login history retrieval, and (5) a user info component in the Sidebar.

**Primary recommendation:** Build both backend (Rust/Axum) and frontend (React) changes in parallel waves. Wave 1: Database migration + Supabase Storage bucket setup. Wave 2: Backend handler changes (new columns, new routes). Wave 3: Frontend profile tab extensions. Wave 4: Sidebar user info + login history tracking middleware.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Store avatars in Supabase Storage — reuse existing `@supabase/supabase-js` integration, no extra infra
- **D-02:** Upload UX — click avatar area OR drag-and-drop onto it. Auto-upload on file selection/drop (no crop step)
- **D-03:** Constraints — 2MB max, formats JPG/PNG/WebP. Enforce client-side before upload
- **D-04:** Data source — use Supabase auth events as base, enrich with custom fields (user-agent, device info) via backend middleware
- **D-05:** Display fields — full detail including session ID, timestamp, IP address, device/browser info, OAuth provider. Session ID enables remote session termination for future phases
- **D-06:** Retention — 90 days with scheduled cleanup (cron-based)
- **D-07:** Flow — soft delete with 14-day grace period. Account marked as deleted, data retained; user can cancel within grace period. After 14 days, permanent deletion
- **D-08:** Resource handling — allow user to transfer ownership of servers to another account before deletion deadline
- **D-09:** Confirmation — re-authentication (re-enter password) + type "DELETE" text. Prevents accidental deletion
- **D-10:** Separate display name field independent from email/OAuth identifier. User can change freely
- **D-11:** Visibility — show in sidebar user info area only (not in server cards or headers)

### the agent's Discretion
- Specific API endpoint design for profile CRUD (PUT /api/profile, etc.)
- Login history table schema in Postgres (migration design)
- Avatar storage bucket naming and access policy in Supabase Storage
- UI layout of the delete account section within the profile tab
- Transfer ownership UX flow

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Avatar file storage | Supabase Storage (cloud) | Frontend (client) | Supabase Storage is the D-01 locked decision; frontend handles upload trigger and validation |
| Avatar file validation | Browser / Client | — | D-03: 2MB, JPG/PNG/WebP validated client-side before upload |
| Display name CRUD | API / Backend | Database | Display name stored in `users` table; API handles read/write with auth check |
| Login history capture | API / Backend (middleware) | Database | D-04: Backend middleware enriches auth events with IP/user-agent, stores in `login_history` table |
| Login history display | Browser / Client | — | Read-only table rendered in profile tab; fetched via API |
| Account deletion flow | API / Backend | Browser / Client | D-07: Soft delete + 14-day grace period; cancellation also backend-driven |
| Re-authentication check | API / Backend | Browser / Client | D-09: Backend verifies password before processing deletion |
| Account deletion cleanup | Background (cron) | — | D-06: Scheduled job to permanently delete expired soft-delete accounts after 14 days |
| Transfer ownership | API / Backend | Database | D-08: Backend handles server ownership transfer |
| Sidebar user info display | Browser / Client | — | D-11: React component reads from authStore, renders avatar + display name + email |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@supabase/supabase-js` | ^2.100.0 (2.106.2 latest) [VERIFIED: npm registry] | Supabase client (auth + storage) | Already in project; D-01 locked decision |
| Axum | 0.7 [CITED: api/Cargo.toml] | Rust web framework for API | Already in project for all backend handlers |
| SQLx | 0.7 [CITED: api/Cargo.toml] | PostgreSQL ORM with async | Already in project for all DB access |
| Zustand | ^5.0.12 (5.0.14 latest) [VERIFIED: npm registry] | State management | Already in project for authStore |
| React Router | ^7.13.0 (7.16.0 latest) [VERIFIED: npm registry] | Client-side routing | Already in project |
| Tailwind CSS | ^4.2.0 (4.3.0 latest) [VERIFIED: npm registry] | CSS framework | Already in project; UI spec uses pure Tailwind |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `chrono` | 0.4 [CITED: api/Cargo.toml] | Date/time in Rust | Login history timestamps, retention logic |
| `uuid` | 1 [CITED: api/Cargo.toml] | UUID generation | Session IDs in login history |
| `serde` / `serde_json` | 1 [CITED: api/Cargo.toml] | JSON serialization | API request/response DTOs |
| `reqwest` | 0.12 [CITED: api/Cargo.toml] | HTTP client | Backend enrichment middleware if calling external IP/geolocation service |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Supabase Storage (avatars) | Direct S3/R2 | D-01 locked decision; Supabase Storage already uses S3 backend, no extra infra |
| Backend middleware (login history) | Supabase auth hooks (webhook) | D-04 specifically calls for backend middleware for enrichment |
| Custom cron for hard delete | pg_cron extension | User's existing cron infrastructure likely simpler to extend |

**Installation:** No new npm packages needed. The existing `@supabase/supabase-js` already supports Storage. No new Rust crates needed beyond those in Cargo.toml.

**Version verification:** Confirmed `@supabase/supabase-js@2.106.2` (latest) available. The existing `^2.100.0` range resolves to latest compatible.

## Architecture Patterns

### System Architecture Diagram

```
[Browser / React App]
      │
      │  ┌──────────────────────────────────────────┐
      │  │ SettingsPage: renderProfileTab()         │
      │  │  ├─ AvatarUpload (file picker + DnD)     │
      │  │  ├─ DisplayNameField (input + save)      │
      │  │  ├─ (existing Name/Email section)        │
      │  │  ├─ (existing Change Password section)   │
      │  │  ├─ LoginHistoryTable (read-only table)  │
      │  │  └─ DeleteAccountSection (2-step flow)   │
      │  │                                          │
      │  │ Sidebar: UserInfoBar (avatar+name+email) │
      │  └──────────────────────────────────────────┘
      │
      ├─────────── Supabase Storage ───────── direct upload ──► avatar bucket
      │                 ▲ (client-side, no backend needed)
      │                 │
      │                 └── File validation (size/type) before upload
      │
      └─────────── Axum API (port 3000, /api/v1/*) ───────────►
                          │
                          ├── GET /api/v1/auth/me ──► User profile (add display_name, avatar_url)
                          ├── PUT /api/v1/auth/profile ──► Update display_name
                          ├── GET /api/v1/auth/login-history ──► Paginated login history
                          ├── POST /api/v1/auth/account/delete ──► Soft delete (re-auth + confirm)
                          ├── POST /api/v1/auth/account/cancel-delete ──► Cancel pending deletion
                          ├── POST /api/v1/auth/account/transfer ──► Transfer server ownership
                          └── middleware: auth event hook ──► INSERT INTO login_history
                                                                       │
                                                                       ▼
                                                              PostgreSQL database
                                                              ├── users (add: display_name, avatar_url, scheduled_deletion_at)
                                                              └── login_history (new table)
                                                                       │
                                                                       ▼
                                                              Cron job (daily): 
                                                              DELETE users WHERE scheduled_deletion_at < NOW() - 14 days
```

### Recommended Project Structure (changes only)

**Backend (Rust/Axum):**
```
api/src/
├── domain/
│   └── user/
│       ├── model.rs           # Add display_name, avatar_url, scheduled_deletion_at fields
│       ├── repository.rs      # Add find_by_deletion_scheduled(), cancel_deletion()
│       ├── sqlx_repository.rs # Implement new repo methods + migration-aware queries
│       └── service.rs         # New: UserProfileService (profile CRUD, deletion logic)
├── presentation/
│   ├── handlers/
│   │   ├── user_handlers.rs   # Extend: profile update (display name), self-delete, transfer, login history
│   │   └── auth_handlers.rs   # Extend: include display_name + avatar_url in /me response
│   └── routes/
│       └── api_routes.rs      # Add new profile/login-history/account routes
└── application/
    └── services/
        └── deletion_cleanup.rs # New: Background cron job for hard delete after 14 days
```

**Frontend (React):**
```
app/src/
├── pages/settings/
│   └── SettingsPage.jsx        # Add avatar, display name, login history, delete sections
├── components/
│   └── settings/
│       ├── AvatarUpload.jsx    # New: Upload component (click + DnD)
│       ├── DisplayNameField.jsx # New: Display name input + save
│       ├── LoginHistoryTable.jsx# New: Login history table
│       └── DeleteAccountSection.jsx # New: 2-step deletion flow
├── hooks/
│   └── useProfile.js           # New: Profile CRUD hook (avatar, display name)
├── store/
│   └── authStore.js            # Add displayName, avatarUrl to persisted user state
├── lib/
│   └── api.js                  # Add profile endpoints (usersApi extended)
└── components/
    └── Sidebar.jsx              # Add user info area at bottom
```

**Database (migration):**
```
migration/
├── 20260530000001_add_display_name_and_avatar.sql
└── 20260530000002_create_login_history_table.sql
```

### Pattern 1: Avatar Upload via Supabase Storage
**What:** Upload image file directly from browser to Supabase Storage using the existing supabase-js client. No backend proxy needed for the upload itself — the frontend uses the anon key with RLS policies to authenticate.

**When to use:** Every avatar upload operation (create, replace).

**Key steps:**
1. Client-side validation: check file type (JPG/PNG/WebP) and size (< 2MB)
2. Generate file path: `avatars/{user_id}/{uuid}.{ext}` to prevent collisions and enable RLS
3. Upload with `upsert: true` to replace existing avatar
4. Get public URL via `getPublicUrl()` after upload succeeds
5. Update user profile via `PUT /api/v1/auth/profile` with the new `avatar_url`

**Supabase Storage bucket RLS policy (recommended):**
```sql
-- Migration to create bucket and RLS policy
INSERT INTO storage.buckets (id, name, public) VALUES ('avatars', 'avatars', true);

-- Allow authenticated users to upload to their own folder
CREATE POLICY "Users can upload their own avatar"
ON storage.objects FOR INSERT TO authenticated
WITH CHECK (
  bucket_id = 'avatars' AND
  (storage.foldername(name))[1] = 'avatars' AND
  (storage.foldername(name))[2] = auth.uid()::text
);

-- Allow public read access to avatars (public bucket)
CREATE POLICY "Anyone can view avatars"
ON storage.objects FOR SELECT TO anon, authenticated
USING (bucket_id = 'avatars');
```

### Pattern 2: Login History Tracking
**What:** Every time a user authenticates (login, OAuth, token refresh), the backend middleware captures IP address, user-agent, device info, and OAuth provider, then inserts a row into `login_history`.

**When to use:** On user authentication events.

**Implementation approach:**
- Create a middleware layer in the auth handlers that fires after successful login/OAuth
- Extract IP from `X-Forwarded-For` header or socket address
- Extract user-agent from `User-Agent` header
- Parse user-agent for device/browser info (optional — store raw string initially)
- Store OAuth provider from the login request context
- Insert into `login_history` with session ID (generated UUID)

### Anti-Patterns to Avoid
- **Uploading avatar via backend proxy:** Adds unnecessary latency and bandwidth through the API server. D-01 specifies direct client-to-Supabase-Storage.
- **Storing full file in database:** Avatar files go to Supabase Storage, only URL stored in `users.avatar_url`.
- **Manual avatar deletion on update:** Use `upsert: true` with the same file path to replace, avoid orphaned files.
- **Inline login history tracking in frontend:** D-04 specifies backend middleware enrichment, not client-side logging.
- **Hard deletion of users table record immediately:** D-07 soft-delete with grace period.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Image file storage | Custom file server | Supabase Storage | D-01 locked; already in project, S3-backed, CDN-cached |
| Avatar upload endpoint | Backend proxy endpoint | Direct client→Supabase Storage | D-01; extra latency, bandwidth, and complexity |
| Password hashing | Custom implementation | `bcrypt` (already in Rust crate) | Already in `PasswordService` at `api/src/domain/auth/service.rs` |
| JWT token management | Custom session tokens | Existing `JwtService` | Already implemented in auth service |
| Image transformation | Custom image processing | Supabase Storage's built-in transforms | If needed later; included in Pro plan |
| Drag-and-drop file upload | Custom drag-drop handler | Native HTML5 drag-drop + `react-dropzone` (optional) | D-02 requires DnD; a small library like `react-dropzone` simplifies edge cases, but the project has no component libraries — pure Tailwind is the established pattern |

**Key insight:** This phase leans heavily on existing infrastructure — Supabase Storage (already integrated), the existing Rust Axum backend (already has user CRUD), and the existing users table (already has `deleted_at`). The main custom work is the new middleware for login history enrichment and the 2-step account deletion flow.

## Runtime State Inventory

> Omit entirely — this is a greenfield enhancement phase, not a rename/refactor/migration.

## Common Pitfalls

### Pitfall 1: Supabase Storage RLS preventing uploads
**What goes wrong:** Avatar upload silently fails or returns 403 because the anon key lacks permission to insert into the `storage.objects` table.
**Why it happens:** Supabase Storage requires explicit RLS policies on `storage.objects` — public buckets alone don't grant upload permissions.
**How to avoid:** Create INSERT policies on `storage.objects` for the `avatars` bucket scoped to `auth.uid() = owner`. Test with both anon key and authenticated user.
**Warning signs:** Upload returns 403/400 errors, console shows CORS or authorization errors.

### Pitfall 2: `deleted_at` overlapping with soft-delete logic
**What goes wrong:** The existing `delete()` method in `SqlxUserRepository` already sets `deleted_at = NOW()`, but the Phase 53 model needs a **scheduled** deletion (14-day grace period). These are two different concepts.
**Why it happens:** The existing `users` table has `deleted_at` used for immediate soft-delete by admins. Phase 53 needs a separate `scheduled_deletion_at` column for user-initiated deletion with a grace period.
**How to avoid:** Add a dedicated `scheduled_deletion_at TIMESTAMPTZ` column. The query `WHERE deleted_at IS NULL` will continue to work. The cron job checks `scheduled_deletion_at < NOW() - INTERVAL '14 days'` for hard deletion.
**Warning signs:** Users getting immediately hidden after requesting deletion instead of having a 14-day grace period.

### Pitfall 3: Session ID in login history lacking future-proofing
**What goes wrong:** Login history stores a session ID but there's no existing session management system to revoke sessions by ID.
**Why it happens:** D-05 requires session ID for future remote session termination, but remote session termination is not implemented yet.
**How to avoid:** Store the session ID as a UUID — it's a forward-looking field. Don't try to implement session revocation in this phase.
**Warning signs:** N/A — this is a design awareness issue, not a bug.

### Pitfall 4: Avatar URL stale after update
**What goes wrong:** After uploading a new avatar with `upsert: true`, the CDN still serves the old cached version.
**Why it happens:** Supabase Storage CDN has propagation delays when overwriting files at the same path.
**How to avoid:** Use unique file paths per upload (e.g., include a timestamp or UUID in the filename) and update the `avatar_url` in the database. This gives a new URL, bypassing CDN cache. Alternatively, accept the delay — the URL will update eventually.
**Warning signs:** Old avatar shows after upload confirmation.

## Code Examples

### Upload avatar to Supabase Storage (frontend)
```javascript
// Source: Adapted from Supabase Storage docs [CITED: supabase.com/docs/guides/storage/uploads.md]
import { supabase } from '../lib/supabase'

async function uploadAvatar(file, userId) {
  // Client-side validation (D-03)
  const allowedTypes = ['image/jpeg', 'image/png', 'image/webp']
  if (!allowedTypes.includes(file.type)) {
    throw new Error('Only JPG, PNG, and WebP files are allowed')
  }
  if (file.size > 2 * 1024 * 1024) {
    throw new Error('File size must be under 2MB')
  }

  // Generate unique path: avatars/{userId}/{uuid}.{ext}
  const ext = file.name.split('.').pop()
  const filePath = `avatars/${userId}/${crypto.randomUUID()}.${ext}`

  // Upload with upsert (though unique paths mean no collisions)
  const { error } = await supabase.storage
    .from('avatars')
    .upload(filePath, file, { upsert: false })

  if (error) throw error

  // Get public URL
  const { data } = supabase.storage
    .from('avatars')
    .getPublicUrl(filePath)

  return data.publicUrl
}
```

### Login history table migration
```sql
-- Migration: 20260530000002_create_login_history_table.sql
CREATE TABLE IF NOT EXISTS login_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID NOT NULL DEFAULT gen_random_uuid(),
    ip_address VARCHAR(45),
    user_agent TEXT,
    device_info VARCHAR(255),
    browser_info VARCHAR(255),
    oauth_provider VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_login_history_user_id ON login_history(user_id);
CREATE INDEX IF NOT EXISTS idx_login_history_created_at ON login_history(created_at);

-- 90-day retention cleanup (D-06): DELETE FROM login_history WHERE created_at < NOW() - INTERVAL '90 days'
```

### Soft-delete with grace period — suggested backfill endpoint
```rust
// Pseudocode for the account deletion handler
async fn request_account_deletion(
    State(state): State<ApiState>,
    auth_user: AuthUser,
    Json(req): Json<DeleteAccountRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Re-authenticate: verify password (D-09)
    let repo = SqlxUserRepository::new(state.pool.clone());
    let user = repo.find_by_id(auth_user.user_id)
        .await?
        .ok_or(ApiError::new("NOT_FOUND", "User not found"))?;
    
    if !PasswordService::verify(&req.password, &user.password_hash) {
        return Err(ApiError::new("UNAUTHORIZED", "Invalid password"));
    }
    
    // 2. Set scheduled_deletion_at = NOW() + 14 days (D-07)
    repo.schedule_deletion(auth_user.user_id, chrono::Utc::now() + chrono::Duration::days(14))
        .await?;
    
    // 3. Log audit event
    // Future: trigger transfer ownership checks (D-08)
    
    Ok(ApiResponse::success(serde_json::json!({
        "message": "Account deletion scheduled. You have 14 days to cancel.",
        "scheduled_deletion_at": ...
    })))
}
```

### Display name update in user handler
```rust
// Extend the existing PUT /api/v1/users/{id} handler or create PUT /api/v1/auth/profile
async fn update_profile(
    State(state): State<ApiState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = SqlxUserRepository::new(state.pool.clone());
    let mut user = repo.find_by_id(auth_user.user_id)
        .await?
        .ok_or(ApiError::new("NOT_FOUND", "User not found"))?;
    
    if let Some(display_name) = req.display_name {
        user.display_name = Some(display_name);
    }
    if let Some(avatar_url) = req.avatar_url {
        user.avatar_url = Some(avatar_url);
    }
    
    let updated = repo.update(&user).await?;
    
    Ok(ApiResponse::new(serde_json::json!({
        "id": updated.id,
        "email": updated.email,
        "display_name": updated.display_name,
        "avatar_url": updated.avatar_url,
    })))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Edit name via `user.name` field | Separate display name field | Phase 53 | D-10: Display name is independent of login identifier |
| No avatar support | Avatar via Supabase Storage | Phase 53 | D-01: New storage bucket + upload flow |
| No login history | Login history with 90-day retention | Phase 53 | D-04/05/06: New table + tracking middleware |
| Account hard-delete by admins only | User self-deletion with grace period | Phase 53 | D-07/08/09: Soft delete + transfer + confirmation |

**Deprecated/outdated:**
- The existing `delete()` endpoint in `UserHandlers` is for admin-use only. Phase 53 adds a separate self-deletion flow. Don't modify the existing `DELETE /api/v1/users/{id}` — it serves a different purpose.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The backend API server (`api/`) is running on port 3000 with the routes defined in `api/src/presentation/routes/api_routes.rs` | Architecture | If the API has been refactored, new routes need different mount points |
| A2 | The existing `lib/api.js` (class-based bearer token client) is the canonical API client for profile operations | Standard Stack | If the project is migrating away from this client, profile endpoints should use a different pattern |
| A3 | The cron job for 14-day hard cleanup runs as a new service in `application/services/` | Architecture Patterns | If the project uses a different scheduling mechanism (e.g., pg_cron), the implementation approach changes |
| A4 | The `avatar_url` should be stored in the `users` table rather than a separate profiles table | Standard Stack | If the user profile data warrants a normalized `profiles` table, the migration design changes |

## Open Questions (RESOLVED)

1. **Where are avatar files stored in Supabase Storage bucket path format?** → RESOLVED: `avatars/{user_id}/{uuid}.{ext}` — unique per upload, avoids CDN staleness. Implemented in Plan 53-01 (bucket creation + RLS) and Plan 53-04 (uploadAvatar function generates unique UUID paths).

2. **Does the existing `me` response need to include `display_name` and `avatar_url`?** → RESOLVED: Extend `/api/v1/auth/me` to include `display_name` and `avatar_url`. Create `PUT /api/v1/auth/profile` endpoint for updates. Implemented in Plan 53-03 (auth handlers extend me, create update_profile handler).

3. **How to handle cron-based permanent deletion?** → RESOLVED: Background task in `api/src/application/services/deletion_cleanup.rs` with 1-hour timer interval. Checks `scheduled_deletion_at < NOW()`, performs hard DELETE + session cleanup. Implemented in Plan 53-06.

## Environment Availability

> Skip this section — this phase has no external dependencies beyond what's already in the project (Node.js, Rust, PostgreSQL, Supabase). All new components use existing infrastructure.

## Validation Architecture

> Skipped — `workflow.nyquist_validation` is explicitly `false` in `.planning/config.json`.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Existing JwtService + PasswordService. Re-authentication for deletion (D-09) follows V2.3.1 |
| V3 Session Management | yes | Existing cookie-based auth with access/refresh tokens. Session ID stored in login history for future revocation (V3.2) |
| V4 Access Control | yes | Existing AuthUser middleware + RBAC. Self-profile access is owner-only by user_id match (V4.1.1) |
| V5 Input Validation | yes | All API inputs validated server-side. Avatar upload validated client-side AND via Supabase Storage MIME/size limits (V5.1) |
| V6 Cryptography | no | Passwords handled by existing bcrypt (PasswordService). No new crypto operations |
| V8 Data Protection | yes | Soft delete preserves data for 14 days (D-07). Login history purged after 90 days (D-06). User can cancel deletion (V8.3) |
| V9 Communication | no | All API communication over HTTPS/TLS via existing reverse proxy |
| V10 Malicious Code | yes | Avatar upload restricted to image types only (MIME validation + Supabase Storage `allowedMimeTypes`) |

### Known Threat Patterns for Rust/Axum + Supabase Storage

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious file upload (avatar) | Tampering | Client-side validation (type + size) + Supabase Storage bucket `allowedMimeTypes` config + RLS policy on INSERT |
| Unauthorized profile update | Spoofing | JWT authentication via AuthUser middleware; user can only update own profile |
| Account deletion abuse | Repudiation | Re-authentication (password verify) + "DELETE" text confirmation: both conditions must be met before processing |
| Data retention violation | Information Disclosure | Cron job automatically hard-deletes after 14-day grace period; login history auto-purged at 90 days |

### Supabase-Specific Security (from skill checklist)

- **Storage RLS:** While the `avatars` bucket is public (for CDN access to avatar images), the INSERT policy must be restricted to `authenticated` role with `auth.uid() = owner` check. [CITED: supabase.com/docs/guides/storage/security/access-control]
- **JWT user_metadata:** Do not use `raw_user_meta_data` for authorization decisions — it's user-editable. The `display_name` and `avatar_url` are profile fields, not authorization data, so this is not a concern. [CITED: supabase skill]
- **Storage upsert permissions:** If using `upsert: true`, the RLS policy must grant INSERT + SELECT + UPDATE on `storage.objects`, not just INSERT. The recommended approach (unique file paths per upload) avoids this complexity. [CITED: supabase skill]

## Sources

### Primary (HIGH confidence)
- [Verified: npm registry] `@supabase/supabase-js@2.106.2`, `react@19.2.6`, `zustand@5.0.14`, `react-router-dom@7.16.0`, `tailwindcss@4.3.0`
- [CITED: supabase.com/docs/guides/storage/uploads.md] Standard file upload patterns
- [CITED: supabase.com/docs/guides/storage/security/access-control] Storage RLS policies
- [CITED: supabase.com/docs/guides/storage/serving/downloads] Public URL generation
- [CITED: supabase.com/docs/guides/storage/buckets/creating-buckets] Bucket creation with allowedMimeTypes and fileSizeLimit
- [CITED: supabase.com/docs/guides/storage/serving/image-transformations] Image transformation support
- [Codebase: api/src/domain/user/model.rs] Users table current schema
- [Codebase: api/src/domain/user/sqlx_repository.rs] User repository patterns
- [Codebase: api/src/presentation/handlers/user_handlers.rs] Existing user CRUD handlers
- [Codebase: api/src/presentation/handlers/auth_handlers.rs] Auth handler patterns (me, login, etc.)
- [Codebase: api/src/presentation/routes/api_routes.rs] Route mounting conventions
- [Codebase: app/src/pages/settings/SettingsPage.jsx] Current Profile tab implementation
- [Codebase: app/src/components/Sidebar.jsx] Current sidebar (no user info area)
- [Codebase: app/src/lib/api.js] Existing usersApi pattern with getProfile/updateProfile
- [Codebase: app/src/store/authStore.js] Auth state management
- [Codebase: app/src/lib/supabase.js] Supabase client with onAuthStateChange

### Secondary (MEDIUM confidence)
- [Codebase: migration/20260324000001_create_users_table.sql] Users table has `deleted_at` already
- [Codebase: app/src/lib/supabase.js] auth.onAuthStateChange already exported

### Tertiary (LOW confidence)
- None — all claims verified against codebase or official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries confirmed in package.json/Cargo.toml and verified versions
- Architecture: HIGH — all architectural decisions (Supabase Storage, Axum routes, etc.) verified against codebase
- Pitfalls: HIGH — based on codebase analysis and documented Supabase Storage RLS behaviors

**Research date:** 2026-05-30
**Valid until:** 2026-06-30 (30 days — stable ecosystem, fast-moving only in Supabase minor versions)
