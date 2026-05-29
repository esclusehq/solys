# Phase 53: User Profile Management - Context

**Gathered:** 2026-05-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Enhance the existing Profile tab in the Settings page to add avatar upload (Supabase Storage), display name (separate from login identifier), login history (Supabase events + custom enrichment), and delete account flow (soft delete with 14-day grace period). Builds on the existing settings page at `app/src/pages/settings/SettingsPage.jsx` which already has a Profile tab with name, email, and change password form.

</domain>

<decisions>
## Implementation Decisions

### Avatar Upload
- **D-01:** Store avatars in Supabase Storage — reuse existing `@supabase/supabase-js` integration, no extra infra
- **D-02:** Upload UX — click avatar area OR drag-and-drop onto it. Auto-upload on file selection/drop (no crop step)
- **D-03:** Constraints — 2MB max, formats JPG/PNG/WebP. Enforce client-side before upload

### Login History
- **D-04:** Data source — use Supabase auth events as base, enrich with custom fields (user-agent, device info) via backend middleware
- **D-05:** Display fields — full detail including session ID, timestamp, IP address, device/browser info, OAuth provider. Session ID enables remote session termination for future phases
- **D-06:** Retention — 90 days with scheduled cleanup (cron-based)

### Delete Account
- **D-07:** Flow — soft delete with 14-day grace period. Account marked as deleted, data retained; user can cancel within grace period. After 14 days, permanent deletion
- **D-08:** Resource handling — allow user to transfer ownership of servers to another account before deletion deadline
- **D-09:** Confirmation — re-authentication (re-enter password) + type "DELETE" text. Prevents accidental deletion

### Display Name
- **D-10:** Separate display name field independent from email/OAuth identifier. User can change freely
- **D-11:** Visibility — show in sidebar user info area only (not in server cards or headers)

### the agent's Discretion
- Specific API endpoint design for profile CRUD (PUT /api/profile, etc.)
- Login history table schema in Postgres (migration design)
- Avatar storage bucket naming and access policy in Supabase Storage
- UI layout of the delete account section within the profile tab
- Transfer ownership UX flow

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Profile Page
- `app/src/pages/settings/SettingsPage.jsx` — the settings page with existing Profile tab (name, email, change password). Lines 311-389 contain the current renderProfileTab implementation. This is where profile features will be added
- `app/src/store/authStore.js` — Zustand store for auth state (user, isAuthenticated). Defines how user data is loaded and persisted

### Auth Infrastructure
- `app/src/api/auth.js` — API client for auth operations (getMe, etc.)
- `app/src/lib/supabase.js` — Supabase client instance (used for Storage operations)

### Prior Phase Context
- `.planning/phases/49-fix-login-functionality-in-landing-page/49-CONTEXT.md` — OAuth login setup (Phase 53 depends on Phase 49)

### Codebase Maps
- `.planning/codebase/CONVENTIONS.md` — coding conventions (Tailwind CSS v4, Zustand, React patterns)
- `.planning/codebase/STRUCTURE.md` — directory layout (pages, store, api, hooks)
- `.planning/codebase/STACK.md` — tech stack (React 19, Supabase, Tailwind CSS v4, Zustand)

### Roadmap
- `.planning/ROADMAP.md` §Phase 53 — phase goal and dependencies

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app/src/pages/settings/SettingsPage.jsx` — existing settings page with tab-based layout (profile, security, API keys tabs). Profile tab already has name/email/change password form. Ready for extending with new sections
- `app/src/store/authStore.js` — auth state management with Zustand + persist middleware. User object available across app
- `app/src/api/auth.js` — auth API calls pattern
- `app/src/lib/supabase.js` — Supabase client used for auth, also usable for Storage (avatar upload)

### Established Patterns
- Tab-based settings page with `activeTab` state
- Form input styling via Tailwind CSS (gray-700 background, white text, blue-600 buttons)
- Zustand stores with persist middleware for auth state
- API client pattern in `app/src/api/` directory
- Disabled email field (cannot change email) with muted styling + note text

### Integration Points
- New profile features (avatar, display name, login history, delete account) go into the existing `renderProfileTab` function or as new sections within it
- Display name stored in existing or new user profile table — needs backend endpoint or Supabase user metadata update
- Login history needs new `login_history` table in Postgres + backend endpoints
- Delete account needs backend endpoint for soft delete + cron job for hard delete after grace period

</code_context>

<specifics>
## Specific Ideas

Profile tab already exists with name, email (disabled), and change password. Phase 53 extends this with avatar (click/drag to upload), display name (new dedicated field), login history (table of recent logins), and delete account (with re-auth + confirm text).

</specifics>

<deferred>
None — discussion stayed within phase scope

</deferred>

---

*Phase: 53-user-profile-management*
*Context gathered: 2026-05-29*
