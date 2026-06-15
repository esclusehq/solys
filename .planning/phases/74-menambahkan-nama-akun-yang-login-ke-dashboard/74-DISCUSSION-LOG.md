# Phase 74 Discussion Log

## Areas Discussed

### 1. Placement — where to show the name
- **Options presented:** Top bar header, Welcome message on dashboard, Both, Make sidebar more prominent
- **User selected:** Both (top bar header across all pages + welcome message on dashboard)

### 2. Click action
- **Options presented:** Navigate to Profile Settings, Dropdown menu, Nothing (purely informational)
- **User selected:** Dropdown menu with Profile, Settings, Logout options

### 3. Display format
- **Options presented:** Display name + avatar, Email only, Full name + avatar
- **User selected:** Display name + avatar (same as sidebar)

## Decisions Summary

- Top bar header with avatar + display_name across all dashboard pages
- Welcome message on ServerManagerPage: "Welcome, {display_name}"
- Dropdown menu on top bar avatar click: Profile, Settings, Logout
- Display fields: display_name + avatar_url from authStore
- Purely frontend — no new API endpoints

## Deferred Items

- Dropdown menu detailed design and logout confirmation
- Sidebar user info unification with top bar
