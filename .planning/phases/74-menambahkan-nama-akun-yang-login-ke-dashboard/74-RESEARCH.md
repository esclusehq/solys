# Phase 74: Menambahkan Nama Akun yang Login ke Dashboard — Research

**Researched:** 2026-06-14
**Domain:** Frontend UI (React + Tailwind v4 + Zustand)
**Confidence:** HIGH (all claims verified from source files)

## Summary

This phase adds a persistent top bar header showing the logged-in user's avatar and display name across all dashboard pages, plus a personalized welcome message on the server list page. The implementation is purely frontend — no new API endpoints, no database changes, and no new dependencies. The existing `authStore` already contains all required user fields (`user.display_name`, `user.avatar_url`, `user.email`), and `lucide-react` is already in `package.json`.

**Primary recommendation:** Create `app/src/components/TopBar.jsx` — a self-contained component managing its own dropdown open/close state via `useRef` + document click/Escape listeners (matching the existing `ContextMenu.jsx` pattern). Modify `Layout.jsx` to insert `<TopBar />` above a scrollable `<Outlet />` wrapper. Fix `DashboardPage.jsx` `getWelcomeMessage()` to use `user.display_name` instead of the incorrect `user.name`.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| _(none specified)_ | Create TopBar with avatar + display_name + dropdown (Profile/Settings/Logout) | authStore fields verified (display_name, avatar_url, email); lucide-react icons available; ContextMenu.jsx provides click-outside pattern |
| _(none specified)_ | Insert TopBar into Layout above Outlet | Layout.jsx structure confirmed — simple flex layout with Sidebar + main/Outlet |
| _(none specified)_ | Fix getWelcomeMessage() to use display_name | DashboardPage.jsx lines 48-56 confirmed using wrong field `user?.name`; correct field is `user.display_name` |
</phase_requirements>

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Placement**: Both — top bar header across all dashboard pages AND welcome message on main dashboard page
- **Top Bar Header**: Persistent across all pages inside Layout.jsx, above Outlet
- **Top Bar Content**: User avatar (circular) + display_name; click opens dropdown with Profile/Settings/Logout
- **Profile/Settings**: Both navigate to `/settings`
- **Welcome Message**: "Welcome, {display_name}" or "Welcome back, {display_name}" with fallback to email prefix
- **Display Fields**: `user.display_name`, `user.avatar_url` from authStore; fallback email prefix; initial-letter fallback when no avatar_url
- **Behavior**: Purely frontend — no new API endpoints, no new database fields

### the agent's Discretion
- _(none specified — all decisions locked)_

### Deferred Ideas (OUT OF SCOPE)
- Dropdown menu detailed design (exact items, logout confirmation) — to be decided during planning
- Click behavior for the avatar in sidebar (keep as-is or unify with top bar)
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| User avatar/name display | Browser / Client | — | Reads from authStore (Zustand persisted to localStorage); no server interaction |
| Dropdown menu state | Browser / Client | — | Local component state (useState + useRef); click-outside listener via document event |
| Navigation (Profile/Settings) | Browser / Client | — | Uses react-router-dom `navigate()` — client-side routing only |
| Logout | Browser / Client | API / Backend | Calls `useAuthStore.logout()` which calls `POST /auth/logout` API |
| Welcome message logic | Browser / Client | — | Pure string formatting from authStore user state |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | ^19.2.4 | Component framework | Already in project — TopBar is a functional component |
| Tailwind CSS | ^4.2.0 | Styling | Already configured via `@theme` in `index.css` |
| Zustand | ^5.0.12 | State management | `useAuthStore` provides logged-in user state |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|--------------|
| lucide-react | ^1.18.0 | Icons | For ChevronDown, User, Settings, LogOut icons in TopBar dropdown |
| react-router-dom | ^7.13.0 | Navigation | For `useNavigate()` in dropdown menu items and `Outlet` in Layout |

**Version verification:** [VERIFIED: npm registry / app/package.json] — All libraries confirmed in `app/package.json`.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom click-outside hook | shadcn/ui or Radix UI DropdownMenu | Would add build-time dependency; existing ContextMenu.jsx pattern proves custom approach works fine |

**Installation:**
```bash
# No new packages needed — all dependencies already installed
npm install  # already satisfied
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│  Browser                                                             │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │  Layout.jsx                                                    │  │
│  │  ┌──────────┐  ┌───────────────────────────────────────────┐   │  │
│  │  │          │  │  TopBar (fixed 56px)                      │   │  │
│  │  │ Sidebar  │  │  ┌──────────────────────────┐             │   │  │
│  │  │ (60 w)   │  │  │  [spacer] [avatar] [name] [▼]         │   │  │
│  │  │          │  │  └──────────┬───────────────┘             │   │  │
│  │  │  ┌──────┐│  │             │ click open/close            │   │  │
│  │  │  │user  ││  │  ┌──────────▼───────────────┐             │   │  │
│  │  │  │info  ││  │  │  Dropdown Menu:           │            │   │  │
│  │  │  └──────┘│  │  │  Profile    → /settings   │            │   │  │
│  │  │          │  │  │  Settings   → /settings   │            │   │  │
│  │  │          │  │  │  ─────────────────────    │            │   │  │
│  │  │          │  │  │  Logout  → authStore.logout()         │   │  │
│  │  │          │  │  └───────────────────────────┘             │   │  │
│  │  │          │  ├───────────────────────────────────────────┤   │  │
│  │  │          │  │  <div class="flex-1 overflow-y-auto">      │   │  │
│  │  │          │  │  ┌─────────────────────────────────────┐  │   │  │
│  │  │          │  │  │  <Outlet /> (page content)           │  │   │  │
│  │  │          │  │  │  e.g., DashboardPage:                │  │   │  │
│  │  │          │  │  │  "Welcome back, {display_name}!"     │  │   │  │
│  │  │          │  │  │  [Servers] [Billing] [Agents] cards  │  │   │  │
│  │  │          │  │  └─────────────────────────────────────┘  │   │  │
│  │  └──────────┘  └───────────────────────────────────────────┘   │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  authStore ──┐                                                        │
│  ┌───────────┤                                                        │
│  │ user: {   │                                                        │
│  │   id,     │                                                        │
│  │   email,  │── TopBar reads: display_name, avatar_url               │
│  │   display_name,── WelcomeMessage reads: display_name, email        │
│  │   avatar_url,── Fallback logic: display_name || email_prefix       │
│  │   ...      │                                                        │
│  │ }          │                                                        │
│  │ logout() ──── Logout item calls this                               │
│  └───────────┘                                                        │
└─────────────────────────────────────────────────────────────────────┘
```

**Data flow for primary use case (TopBar):**
1. App loads → `ProtectedRoute` checks auth → `authStore.checkAuth()` populates `user`
2. `Layout.jsx` renders → `TopBar` reads `authStore.user`
3. `TopBar` displays avatar + display_name (with fallback logic)
4. User clicks avatar area → dropdown state toggles open
5. User clicks "Logout" → `authStore.logout()` → `POST /auth/logout` → state cleared → redirect to login

### Recommended Project Structure
```
app/src/
├── components/
│   ├── TopBar.jsx        # NEW — 56px fixed header with avatar + name + dropdown
│   ├── Layout.jsx        # MODIFY — Add <TopBar /> above <Outlet />
│   ├── IDE/ContextMenu.jsx  # REFERENCE — click-outside pattern for dropdown
│   └── Sidebar.jsx       # NO CHANGE — keep existing user area as-is
├── pages/dashboard/
│   └── DashboardPage.jsx  # MODIFY — Fix getWelcomeMessage() field name
├── store/
│   └── authStore.js       # NO CHANGE — user object already has all fields
└── index.css              # NO CHANGE — all CSS variables already defined
```

### Pattern 1: Dropdown with Click-Outside (from ContextMenu.jsx)
**What:** A dropdown menu that opens/closes on trigger click and closes on click-outside or Escape.
**When to use:** For the TopBar user menu dropdown with Profile/Settings/Logout items.
**Example:**
```jsx
// Pattern from app/src/components/IDE/ContextMenu.jsx lines 84-98
const [isOpen, setIsOpen] = useState(false);
const menuRef = useRef(null);

useEffect(() => {
    const handleClick = (e) => {
        if (menuRef.current && !menuRef.current.contains(e.target)) {
            setIsOpen(false);
        }
    };
    const handleKeyDown = (e) => {
        if (e.key === 'Escape') setIsOpen(false);
    };
    
    if (isOpen) {
        document.addEventListener('click', handleClick);
        document.addEventListener('keydown', handleKeyDown);
    }
    
    return () => {
        document.removeEventListener('click', handleClick);
        document.removeEventListener('keydown', handleKeyDown);
    };
}, [isOpen]);
```
**Source:** [VERIFIED: app/src/components/IDE/ContextMenu.jsx] — Existing pattern with document event listeners.

### Pattern 2: lucide-react Named Imports
**What:** Import only needed icons from lucide-react.
**When to use:** For icons in TopBar dropdown (ChevronDown, User, Settings, LogOut).
**Example:**
```jsx
// Pattern from app/src/pages/dashboard/WelcomeModal.jsx line 5
import { CheckCircle2, Sparkles, X } from 'lucide-react';
```
**Source:** [VERIFIED: app/src/pages/dashboard/WelcomeModal.jsx] — Project convention.

### Pattern 3: user Display Logic (from Sidebar.jsx)
**What:** Display avatar with fallback, then display_name with email fallback.
**When to use:** For TopBar user area rendering.
**Example:**
```jsx
// Pattern from app/src/components/Sidebar.jsx lines 101-121
{user && (
    <div className="flex items-center gap-3 px-3 py-2 rounded-lg bg-[rgba(255,255,255,0.03)]">
        <div className="w-8 h-8 rounded-full overflow-hidden flex-shrink-0 bg-gray-700">
            {user.avatar_url ? (
                <img src={user.avatar_url} alt="" className="w-full h-full object-cover" />
            ) : (
                <div className="w-full h-full flex items-center justify-center text-gray-400 text-xs font-medium">
                    {(user.display_name || user.email || '?')[0].toUpperCase()}
                </div>
            )}
        </div>
        <div className="min-w-0 flex-1">
            <p className="text-sm text-white truncate">
                {user.display_name || user.email?.split('@')[0] || 'User'}
            </p>
            <p className="text-xs text-gray-500 truncate">{user.email}</p>
        </div>
    </div>
)}
```
**Source:** [VERIFIED: app/src/components/Sidebar.jsx lines 100-121]

### Anti-Patterns to Avoid
- **Hardcoded hex colors:** Use CSS custom properties (`var(--color-cosmic-cyan)`) instead — the app has light/dark theme switching via `[data-theme="light"]` [VERIFIED: app/src/index.css lines 288-302]
- **Inline SVG icons when lucide-react available:** The app already has `lucide-react ^1.18.0` — use `<ChevronDown size={16} />`, `<User size={16} />`, etc. instead of inline SVGs
- **Reading from wrong authStore field:** `user.name` does NOT exist in authStore — the correct field is `user.display_name` [VERIFIED: authStore.js + Sidebar.jsx lines 109/115]
- **Using `user?.name` without fallback:** The `getWelcomeMessage()` in DashboardPage.jsx uses `user?.name` which is undefined — must be `user?.display_name || user?.email?.split('@')[0] || 'User'`

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| User state management | Custom auth context | `useAuthStore` (Zustand) | Already provides `user`, `logout()`, `isAuthenticated` — no new store needed |
| Dropdown menu | Full custom dropdown from scratch | Simple `useState` toggle + click-outside via `useRef` + document events | Existing `ContextMenu.jsx` proves this pattern works at 150+ lines; TopBar dropdown is simpler (4 items, no keyboard navigation) |
| Icons | Inline SVG paths | `lucide-react` named imports | Already in package.json; used in WelcomeModal.jsx |

## Common Pitfalls

### Pitfall 1: Click-Outside Listener Memory Leak
**What goes wrong:** Adding a document click listener without cleanup causes stale closures and multiple listeners when component re-renders.
**Why it happens:** `useEffect` without a cleanup function on the event listener.
**How to avoid:** Always return a cleanup function that removes both `click` and `keydown` listeners. Use the pattern from `ContextMenu.jsx` lines 90-98.
**Warning signs:** Dropdown stops closing on outside click after re-renders, or multiple dropdown instances.

### Pitfall 2: Forgetting `e.stopPropagation()` on Trigger Click
**What goes wrong:** Clicking the avatar area to open the dropdown immediately triggers the document click listener, which closes it.
**Why it happens:** The click event on the trigger bubbles up to the document, where the outside-click handler fires.
**How to avoid:** Use `e.stopPropagation()` on the trigger button's onClick handler, OR use a ref-based containment check (check if `menuRef.current.contains(e.target)`) which is more robust.
**Warning signs:** Dropdown opens and immediately closes on trigger click.

### Pitfall 3: Wrong User Field Name
**What goes wrong:** Using `user.name` instead of `user.display_name` causes blank welcome messages or "Welcome back, User!" always.
**Why it happens:** The authStore stores `display_name` (not `name`), as set by the backend's `/auth/me` response.
**How to avoid:** Always reference `user.display_name` — verified in Sidebar.jsx lines 109 and 115. `user.name` is set only temporarily in `authStore.register()` line 91 as `user: { email, name }`, but this is overwritten by `getMe()` which returns `display_name`.
**Warning signs:** Welcome message shows "User" instead of actual name.

### Pitfall 4: Sidebar User Area z-index Conflict
**What goes wrong:** The TopBar dropdown might appear behind the Sidebar or other elements.
**Why it happens:** The sidebar has `z-10` (line 36 of Sidebar.jsx).
**How to avoid:** Give TopBar dropdown `z-50` (matching ContextMenu.jsx line 170 which uses `z-50`).
**Warning signs:** Dropdown clipped by sidebar or page content.

## Code Examples

### TopBar Component Structure (Verified Patterns)
```jsx
// TopBar.jsx — based on Sidebar.jsx user pattern + ContextMenu.jsx click-outside pattern
import { useState, useRef, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { ChevronDown, User, Settings, LogOut } from 'lucide-react';

export default function TopBar() {
    const { user, logout } = useAuthStore();
    const navigate = useNavigate();
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef(null);

    // Click-outside + Escape: ContextMenu.jsx pattern lines 84-98
    useEffect(() => {
        const handleClickOutside = (e) => {
            if (dropdownRef.current && !dropdownRef.current.contains(e.target)) {
                setIsOpen(false);
            }
        };
        const handleEscape = (e) => {
            if (e.key === 'Escape') setIsOpen(false);
        };

        if (isOpen) {
            document.addEventListener('click', handleClickOutside);
            document.addEventListener('keydown', handleEscape);
        }

        return () => {
            document.removeEventListener('click', handleClickOutside);
            document.removeEventListener('keydown', handleEscape);
        };
    }, [isOpen]);

    if (!user) return null;

    const displayName = user.display_name || user.email?.split('@')[0] || 'User';
    const avatarLetter = (user.display_name || user.email || '?')[0].toUpperCase();

    const handleLogout = async () => {
        setIsOpen(false);
        await logout();
    };

    const handleNavigate = (path) => {
        setIsOpen(false);
        navigate(path);
    };

    return (
        <header className="h-14 flex-shrink-0 border-b border-[var(--color-cosmic-border)] bg-[var(--color-deep-space)] flex items-center justify-end px-4 gap-3">
            {/* User trigger area */}
            <div className="relative" ref={dropdownRef}>
                <button
                    onClick={() => setIsOpen(!isOpen)}
                    className="flex items-center gap-2 px-2 py-1 rounded-lg hover:bg-[rgba(255,255,255,0.05)] transition-colors min-h-[44px]"
                >
                    {/* Avatar: Sidebar.jsx pattern lines 104-111 */}
                    <div className="w-8 h-8 rounded-full overflow-hidden flex-shrink-0 bg-gray-700">
                        {user.avatar_url ? (
                            <img src={user.avatar_url} alt="" className="w-full h-full object-cover" />
                        ) : (
                            <div className="w-full h-full flex items-center justify-center text-xs font-medium"
                                 style={{ background: 'var(--color-cosmic-purple)', color: '#fff' }}>
                                {avatarLetter}
                            </div>
                        )}
                    </div>
                    <span className="text-sm font-medium text-[var(--color-text-main)] truncate max-w-[120px]">
                        {displayName}
                    </span>
                    <ChevronDown size={16} className="text-[var(--color-text-muted)]" />
                </button>

                {/* Dropdown: ContextMenu.jsx pattern lines 168-199 */}
                {isOpen && (
                    <div className="absolute right-0 top-full mt-1 w-48 py-1 bg-[var(--color-nebula)] 
                                    border border-[var(--color-cosmic-border)] rounded-lg shadow-xl z-50">
                        <button
                            onClick={() => handleNavigate('/settings')}
                            className="w-full flex items-center gap-3 px-3 py-2 text-sm text-[var(--color-text-main)] 
                                     hover:bg-[rgba(13,223,242,0.15)] transition-colors"
                        >
                            <User size={16} className="text-[var(--color-text-muted)]" />
                            Profile
                        </button>
                        <button
                            onClick={() => handleNavigate('/settings')}
                            className="w-full flex items-center gap-3 px-3 py-2 text-sm text-[var(--color-text-main)] 
                                     hover:bg-[rgba(13,223,242,0.15)] transition-colors"
                        >
                            <Settings size={16} className="text-[var(--color-text-muted)]" />
                            Settings
                        </button>
                        <div className="my-1 border-t border-[var(--color-cosmic-border)]" />
                        <button
                            onClick={handleLogout}
                            className="w-full flex items-center gap-3 px-3 py-2 text-sm 
                                     text-[var(--color-cosmic-red)]
                                     hover:bg-[rgba(239,68,68,0.1)] transition-colors"
                        >
                            <LogOut size={16} />
                            Logout
                        </button>
                    </div>
                )}
            </div>
        </header>
    );
}
```

### Layout.jsx Modification
```jsx
import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import TopBar from './TopBar';          // NEW import

export default function Layout() {
    return (
        <>
            <div className="stars-bg" />
            <div className="flex h-screen relative z-1 w-full overflow-hidden">
                <Sidebar />
                <main className="flex-1 flex flex-col">      {/* removed overflow-y-auto */}
                    <TopBar />                                 {/* NEW — fixed height 56px */}
                    <div className="flex-1 overflow-y-auto">   {/* moved overflow-y here */}
                        <Outlet />
                    </div>
                </main>
            </div>
        </>
    );
}
```

### DashboardPage.jsx getWelcomeMessage() Fix
```jsx
// CURRENT (broken) — lines 48-56 of DashboardPage.jsx:
const getWelcomeMessage = () => {
    if (!user?.created_at) return `Welcome back, ${user?.name || 'User'}!`;  // BUG: user.name
    const createdDate = new Date(user.created_at)
    const daysSinceCreation = (Date.now() - createdDate.getTime()) / (1000 * 60 * 60 * 24)
    if (daysSinceCreation <= 2) {
        return `Welcome, ${user?.name || 'User'}!`;       // BUG: user.name
    }
    return `Welcome back, ${user?.name || 'User'}!`       // BUG: user.name
}

// FIXED — uses display_name with fallback chain (matching Sidebar.jsx pattern):
const getWelcomeMessage = () => {
    const displayName = user?.display_name || user?.email?.split('@')[0] || 'User';
    if (!user?.created_at) return `Welcome back, ${displayName}!`;
    const createdDate = new Date(user.created_at);
    const daysSinceCreation = (Date.now() - createdDate.getTime()) / (1000 * 60 * 60 * 24);
    if (daysSinceCreation <= 2) {
        return `Welcome, ${displayName}!`;
    }
    return `Welcome back, ${displayName}!`;
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| User info only in sidebar (Sidebar.jsx lines 100-121) | Persistent TopBar across all dashboard pages | Phase 74 | User identity always visible while browsing |
| `getWelcomeMessage()` uses `user?.name` (undefined field) | Uses `user?.display_name` with email fallback | Phase 74 | Welcome message now shows actual name |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The Settings page route is `/settings` | Code Examples, Locked Decisions | Both "Profile" and "Settings" menu items navigate to `/settings` per CONTEXT.md — if the route is different, navigation fails silently |
| A2 | No confirmation dialog needed for logout | UI-SPEC Interaction Contract | Matches existing pattern in sidebar/settings — but user might expect confirmation for destructive action |
| A3 | `overflow-y-auto` should move from `<main>` to a wrapper div | Architecture Patterns | Currently on `<main>` — moving it ensures TopBar stays fixed while content scrolls. If any page relies on `<main>` having overflow-y, scrolling behavior changes |

## Open Questions

1. **Is the confirmation dialog needed for logout?**
   - What we know: CONTEXT.md deferred this decision, UI-SPEC says "no confirmation"
   - What's unclear: Whether the user wants a confirmation step before logging out
   - Recommendation: Follow UI-SPEC (no confirmation), matching existing pattern

2. **Should Profile and Settings go to different routes?**
   - What we know: CONTEXT.md says both navigate to `/settings`
   - What's unclear: Whether a future phase might add a dedicated profile page
   - Recommendation: Keep both pointing to `/settings` as decided — simple change if a profile page is added later

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | React dev server | ✓ | — | — |
| npm/yarn | Package management | ✓ | — | — |
| React 19 | Component rendering | ✓ | 19.2.4 | — |
| lucide-react | Icons | ✓ | ^1.18.0 | — |
| Tailwind CSS v4 | Styling | ✓ | ^4.2.0 | — |
| Zustand | State management | ✓ | ^5.0.12 | — |
| react-router-dom | Navigation | ✓ | ^7.13.0 | — |

**Missing dependencies with no fallback:** None — all dependencies are already installed.

**Missing dependencies with fallback:** None.

## Sources

### Primary (HIGH confidence)
- `app/src/components/Layout.jsx` — Current layout structure verified
- `app/src/components/Sidebar.jsx` — User display pattern at lines 100-121 verified
- `app/src/store/authStore.js` — User state structure and logout() verified
- `app/src/pages/dashboard/DashboardPage.jsx` — getWelcomeMessage() bug confirmed
- `app/src/index.css` — CSS variables and theme tokens verified
- `app/package.json` — lucide-react ^1.18.0 confirmed in dependencies
- `app/src/components/IDE/ContextMenu.jsx` — Dropdown click-outside pattern verified
- `app/src/api/auth.js` — logout() API function verified
- `.planning/phases/74-menambahkan-nama-akun-yang-login-ke-dashboard/74-CONTEXT.md` — User decisions
- `.planning/phases/74-menambahkan-nama-akun-yang-login-ke-dashboard/74-UI-SPEC.md` — Design contract

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified from package.json and source files
- Architecture: HIGH — Layout structure, authStore, and component patterns confirmed by reading source
- Pitfalls: HIGH — verified from actual code behavior (user.name bug, z-index pattern)

**Research date:** 2026-06-14
**Valid until:** No expiration — this is a stable frontend implementation based on existing project patterns
