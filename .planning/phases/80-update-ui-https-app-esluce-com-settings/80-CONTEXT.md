# Phase 80: Update UI https://app.esluce.com/settings — CONTEXT

## Source Files
- **Frontend (main):** `app/src/pages/settings/SettingsPage.jsx` (~1433 lines, single file)
- **Existing sub-components:** `app/src/components/settings/CloudflareSettings.jsx`, `S3ProfileSettings.jsx`
- **API:** `app/src/lib/api.js` — `webhooksApi`, `cloudflareApi`
- **Auth store:** `app/src/store/authStore.js` — user profile, `updateProfile`, `changeEmail`, `fetchLoginHistory`, etc.
- **Hooks:** `app/src/hooks/useProfile.js` — `useProfile`, `uploadAvatar`

## Current State
- Single 1433-line component with 8 tabs: Profile, Team, Security, API Keys, Webhooks, Cloudflare DNS, Storage (S3), Restart Defaults
- All sections use flat `bg-gray-700`, `bg-gray-800` styling — no cosmic theme
- Horizontal tab bar at top
- Team tab uses hardcoded mock data (1 member: self)
- API Keys use `supabase.rpc()` directly (not the app's `api` client)
- Cloudflare and S3 Storage are already separate components
- Profile tab has 6 subsections: avatar, display name, profile info, password, login history, danger zone

## Decisions

### Cosmic Theme Restyle (Entire Page)
- All `bg-gray-700 rounded`, `bg-gray-800 rounded` → `glass-panel` with appropriate padding
- Tab bar: horizontal tabs remain but restyled with cosmic theme
  - Active tab: `text-[var(--color-cosmic-cyan)] border-b-2 border-[var(--color-cosmic-cyan)]`
  - Inactive tab: `text-gray-400 hover:text-gray-200`
  - Tab bar separator: `border-b border-[var(--color-cosmic-border)]`
- All inputs: `bg-gray-700` → `bg-[var(--color-bg-secondary)]` with `focus:ring-[var(--color-cosmic-cyan)]` instead of `focus:ring-blue-500`
- All buttons: keep existing color semantics (blue=action, red=danger, green=success) but update Tailwind classes to match cosmic palette where applicable
- Tables: `border-gray-700` → `border-[var(--color-cosmic-border)]`
- Status badges: reuse `bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]` pattern from Nodes.jsx
- Loading states (skeleton/spinner): keep as-is, already consistent
- Danger Zone: keep red border-left accent, use `border-red-500/60` for subtlety

### File Splitting (3-4 Files)
**DO NOT** split into one file per tab (too much churn) and DO NOT keep single file.
Group into logical chunks:

```
app/src/pages/settings/
├── SettingsPage.jsx          # Shell — tabs, navigation, imports
├── ProfileSettings.jsx       # Profile tab + Security tab (personal account settings)
├── ApiKeySettings.jsx        # API Keys tab + Team tab (developer/team management)
├── WebhookSettings.jsx       # Webhooks tab (already heavier content)
└── RestartDefaultsSettings.jsx  # Restart Defaults tab (admin, inline form)
```

- CloudflareSettings, S3ProfileSettings stay in `components/settings/` (already extracted)
- Each extracted component receives necessary state/props from parent — keep prop drilling minimal, no new context/store for now
- Extract helper functions (e.g., `getRoleBadge`, `formatRelativeTime`) into the relevant child or keep in parent if shared

### Profile Tab Sub-section Grouping
Within ProfileSettings, group the 6 current sections under sub-headings:

1. **Personal Information** — avatar upload, display name, profile info (name, email, change email)
2. **Account Security** — change password (move password form from Profile tab into this group; 2FA stays in Security tab)
3. **Login Activity** — login history table (keep expanded, no collapse)
4. **Danger Zone** — delete account + transfer ownership (keep red left border accent)

Use `pt-6 border-t border-[var(--color-cosmic-border)]` dividers between groups.

### What's NOT Changing
- No new API endpoints needed (all data comes from existing supabase/auth calls + webhooksApi)
- No functional changes to any tab's logic
- Team tab remains mock data (no real API integration in this phase)
- Tab bar stays horizontal, no sidebar conversion
- Security tab (2FA, sessions) stays in its own tab — NOT merged into Profile
- No mobile-specific layout changes
- No changes to supabase RPC calls for API Keys (functional, just restyle)
- Cloudflare and Storage tabs are already component files — restyle them with glass-panel in-place

## Reusable Patterns
- `glass-panel` class (defined in global CSS) for all section containers
- `border border-[var(--color-cosmic-border)]` for default panel borders
- `focus:ring-[var(--color-cosmic-cyan)]` replacing `focus:ring-blue-500` on inputs
- Status badge pattern: `className="px-2 py-1 rounded text-xs bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]"`
- Sub-section divider: `pt-6 border-t border-[var(--color-cosmic-border)]`
- Cosmic hover transitions: `transition-colors hover:bg-[rgba(255,255,255,0.03)]` for table rows
- Input field pattern: `px-4 py-2 bg-[var(--color-bg-secondary)] text-white rounded focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)]`

## API Reference
- `webhooksApi.list()` → `GET /api/v1/webhooks`
- `webhooksApi.test(id)` → `POST /api/v1/webhooks/{id}/test`
- `webhooksApi.retry(id)` → `POST /api/v1/webhooks/{id}/retry`
- `supabase.auth.updateUser({ password })` — password change
- `supabase.auth.mfa.enroll/challenge/verify` — 2FA flow
- `supabase.rpc('get_user_security')` — 2FA status check
- `supabase.rpc('list_api_keys')`, `create_api_key`, `revoke_api_key` — API key management
- `useAuthStore.getState().fetchLoginHistory()` — login history
- `useAuthStore.getState().updateProfile()` — display name update
- `useAuthStore.getState().changeEmail()` — email change
- `useAuthStore.getState().requestAccountDeletion()` — delete account
- `useAuthStore.getState().transferOwnership()` — transfer servers
- `fetchApi('/api/v1/settings/restart-defaults')` — restart defaults (GET/PUT)

## Out of Scope (Deferred)
- Real API integration for Team tab (still mock data)
- Sidebar navigation for tabs
- API Keys migration from supabase RPC to app's API client
- Usage statistics / monthly breakdown in settings
- Notification preferences section
- Theme/customization settings
