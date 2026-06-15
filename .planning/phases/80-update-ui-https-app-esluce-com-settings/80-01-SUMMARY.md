# Phase 80 Plan 01: File Splitting

One-liner: Split the 1433-line SettingsPage.jsx monolith into a 63-line shell component and 4 child components (ProfileSettings, ApiKeySettings, WebhookSettings, RestartDefaultsSettings). No CSS changes.

## Files Created

| File | Description | Lines |
|------|-------------|-------|
| `app/src/pages/settings/ProfileSettings.jsx` | Profile tab + Security tab with all state/handlers/effects | 801 |
| `app/src/pages/settings/ApiKeySettings.jsx` | API Keys tab + Team tab with inline getRoleBadge | 349 |
| `app/src/pages/settings/WebhookSettings.jsx` | Webhooks tab with formatRelativeTime | 140 |
| `app/src/pages/settings/RestartDefaultsSettings.jsx` | Global Restart Defaults tab | 103 |

## Files Modified

| File | Description | Lines |
|------|-------------|-------|
| `app/src/pages/settings/SettingsPage.jsx` | Reduced from 1433 lines to 63-line shell | 63 |

## Tasks Completed

1. **Created ProfileSettings.jsx** — Extracted Profile tab (avatar upload, display name, profile info, password change, login history, delete account) and Security tab (2FA, active sessions, logout all) with all 31 state variables, 16 handlers, and 3 useEffect blocks. Props: `user`, `tab`.

2. **Created ApiKeySettings.jsx** — Extracted API Keys tab (generate/copy/revoke API keys) and Team tab (roles, invite members, member list, permissions table) with inline getRoleBadge. Props: `user`, `tab`.

3. **Created WebhookSettings.jsx** — Extracted Webhooks tab (list webhooks, test delivery, retry failed) with formatRelativeTime helper. No props.

4. **Created RestartDefaultsSettings.jsx** — Extracted Global Restart Defaults tab (max restart attempts, cooldown slider, save). No props.

5. **Reduced SettingsPage.jsx** — Shell now imports 6 child components, keeps only `activeTab` state, `getUserRole`/`isAdmin` helpers, tabs array with admin-gating, and conditional rendering. No behavior changes.

## Key Decisions

- ProfileSettings handles both 'profile' and 'security' tabs because they share state (2FA flows, session management) that was tightly coupled in the original monolith
- ApiKeySettings handles both 'api' and 'team' tabs (independent concerns but co-located in original)
- getRoleBadge is inlined in ApiKeySettings (not shared) since it's only used by Team tab rendering
- formatRelativeTime is inlined in WebhookSettings (only used there)
- All existing CSS classes preserved — no var(--color-cosmic-*) introduced, no CSS changes

## Deviations from Plan

None — plan executed exactly as written. All state, handlers, effects, and JSX were faithfully extracted from the original source lines without modification.
