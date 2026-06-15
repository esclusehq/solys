<domain>
Show the logged-in user's account identity (display name + avatar) prominently on the Dashboard — both as a persistent top bar header across all dashboard pages and as a welcome greeting on the main server list page. Replace the current sidebar-only user info with a more visible and interactive presence.
</domain>

<canonical_refs>
- ROADMAP.md — Phase 74
- app/src/components/Sidebar.jsx — existing sidebar user display (baseline)
- app/src/store/authStore.js — user state source of truth
</canonical_refs>

<code_context>
- Sidebar at `app/src/components/Sidebar.jsx:100-121` already renders user avatar + display_name + email
- `authStore.user` contains `id`, `email`, `display_name`, `avatar_url`, `role`, `user_metadata`
- No top bar/header component exists — Layout.jsx only wraps Sidebar + Outlet
- ServerManagerPage has no personalized greeting
</code_context>

<decisions>

### Placement
- **Both**: Add a top bar header across all dashboard pages AND a welcome message on the main dashboard (server list) page.

### Top Bar Header
- Persistent across all pages (inside Layout.jsx, above the Outlet or as a fixed header)
- Shows: user avatar (circular) + display_name
- Click opens a small dropdown menu with options:
  - "Profile" → navigates to /settings
  - "Settings" → navigates to /settings
  - "Logout" → calls logout action
- Styling: matches existing cosmic theme, compact height, no extra chrome

### Welcome Message
- Displayed on ServerManagerPage (main dashboard): "Welcome, {display_name}" or "Welcome back, {display_name}"
- Positioned at the top of the page content, above the server cards/search bar
- Uses display_name from authStore; fallback to email prefix if display_name is empty

### Display Fields
- Display name + avatar (same fields as sidebar: `user.display_name`, `user.avatar_url`)
- Fallback: email prefix if display_name is not set
- Avatar shows initial-letter fallback when no avatar_url exists

### Behavior
- Purely frontend — no new API endpoints needed
- User state already available via authStore
- No new database fields required

</decisions>

<deferred>
- Dropdown menu detailed design (exact items, logout confirmation) — to be decided during planning
- Click behavior for the avatar in sidebar (keep as-is or unify with top bar)
</deferred>
