# Phase 80: Update UI https://app.esluce.com/settings — Research

**Researched:** 2026-06-15
**Domain:** Frontend UI cosmic restyle + file splitting (1433-line monolith → shell + 4 extracted components)
**Confidence:** HIGH

## Summary

Phase 80 restyles the Settings page (8 tabs, 1433 lines) from flat `bg-gray-700`/`bg-gray-800` styling to the cosmic `glass-panel` theme, **AND** splits the monolithic file into a shell + 4 extracted component files. This is the most complex of the cosmic restyle phases because it combines CSS class replacement across ~40+ distinct HTML patterns with non-trivial file-splitting that must preserve all existing functionality.

**Key findings:**

1. **SettingsPage.jsx is a monolith with 8 tabs** rendered via 6 `render*Tab()` functions (Profile, Security, API Keys, Restart Defaults, Team, Webhooks) plus 2 imported components (CloudflareSettings, S3ProfileSettings). Tab routing uses `activeTab` state with conditional rendering.

2. **File splitting is well-scoped** — group into 4 new files following CONTEXT.md: ProfileSettings.jsx (profile + security tabs), ApiKeySettings.jsx (API keys + team tabs), WebhookSettings.jsx (webhooks tab), RestartDefaultsSettings.jsx (restart defaults tab). CloudflareSettings and S3ProfileSettings stay in `components/settings/`.

3. **CSS class replacement is mechanical** — ~15 distinct Tailwind patterns appear 3-40 times each across the file. Every `bg-gray-700`, `border-gray-700`, `focus:ring-blue-500`, etc. maps to a verified cosmic equivalent. `--color-bg-secondary` does NOT exist in index.css — use `--color-cosmic-card` for inputs.

4. **No new API endpoints, no functional changes** — all data comes from existing supabase/auth + webhooksApi calls. Zero test files exist (matching Phase 79 finding).

5. **3-plan execution recommended:** (1) File splitting + new component creation, (2) Cosmic restyle of new components + Profile tab sub-section grouping, (3) Cloudflare + S3 restyle. Plans 1 and 3 are parallel-safe.

**Primary recommendation:** Execute file splitting first (preserving old styling), then apply cosmic class replacements as a mechanical find-and-replace pass across the new files. This gives clear diffs at each stage and avoids the risk of restyling + reorganizing in one shot.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- All `bg-gray-700 rounded`, `bg-gray-800 rounded` → `glass-panel` with appropriate padding
- Tab bar: horizontal tabs remain but restyled with cosmic theme
  - Active tab: `text-[var(--color-cosmic-cyan)] border-b-2 border-[var(--color-cosmic-cyan)]`
  - Inactive tab: `text-gray-400 hover:text-gray-200`
  - Tab bar separator: `border-b border-[var(--color-cosmic-border)]`
- All inputs: `bg-gray-700` → `bg-[var(--color-cosmic-card)]` with `focus:ring-[var(--color-cosmic-cyan)]` instead of `focus:ring-blue-500`
- All buttons: keep existing color semantics (blue=action, red=danger, green=success) but update `rounded` → `rounded-xl`
- Tables: `border-gray-700` → `border-[var(--color-cosmic-border)]`
- Status badges: reuse `bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]` pattern
- Loading states (skeleton/spinner): keep as-is, already consistent
- Danger Zone: keep red border-left accent, use `border-red-500/60` for subtlety
- File splitting grouped as:
  - `SettingsPage.jsx` — shell, tabs, navigation, imports
  - `ProfileSettings.jsx` — Profile tab + Security tab (personal account settings)
  - `ApiKeySettings.jsx` — API Keys tab + Team tab (developer/team management)
  - `WebhookSettings.jsx` — Webhooks tab (heavier content)
  - `RestartDefaultsSettings.jsx` — Restart Defaults tab (admin, inline form)
- CloudflareSettings, S3ProfileSettings stay in `components/settings/` (already extracted)
- Each extracted component receives necessary state/props from parent — keep prop drilling minimal, no new context/store for now
- ProfileSettings sub-section grouping:
  1. Personal Information — avatar upload, display name, profile info
  2. Account Security — change password
  3. Login Activity — login history table
  4. Danger Zone — delete account + transfer ownership (red left border accent)
- Use `pt-6 border-t border-[var(--color-cosmic-border)]` dividers between groups
- No new API endpoints needed
- No functional changes to any tab's logic
- Team tab remains mock data
- Tab bar stays horizontal, no sidebar conversion
- Security tab (2FA, sessions) stays in its own tab — NOT merged into Profile
- No mobile-specific layout changes

### the agent's Discretion
- Exact props interface for each extracted component
- Whether `getRoleBadge` stays in parent or moves to shared utility
- Whether each child imports `addToast` from `useUIStore` directly (matching CloudflareSettings pattern) or receives it as a prop
- Exact plan boundary between file-splitting and restyling tasks
- Whether to move handlers inline into child components or keep passing them as props

### Deferred Ideas (OUT OF SCOPE)
- Real API integration for Team tab (still mock data)
- Sidebar navigation for tabs
- API Keys migration from supabase RPC to app's API client
- Usage statistics / monthly breakdown in settings
- Notification preferences section
- Theme/customization settings

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SPLIT-01 | Extract inline render functions to child components | Full inventory of render functions, state variables, and handlers below |
| SPLIT-02 | SettingsPage shell retains tab routing + shared state | Tab bar pattern, `activeTab` state, conditional rendering pattern documented |
| SPLIT-03 | Props design for each child | Props interface design in Standard Stack section |
| RESTYLE-01 | Replace all bg-gray-700/bg-gray-800 with cosmic equivalents | Complete CSS class mapping table (40+ patterns) provided |
| RESTYLE-02 | Profile tab sub-section grouping with dividers | 6→4 sub-section mapping with divider CSS provided |
| RESTYLE-03 | CloudflareSettings + S3ProfileSettings cosmic restyle | Class inventory for both files provided |
| NO-CHANGE-01 | Zero functional changes to handlers or API calls | Verified all handlers use existing stores/APIs only |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tab routing (activeTab state) | Browser | — | Pure frontend state, no server interaction |
| Tab bar rendering | Browser | — | JSX conditional rendering based on activeTab |
| Profile tab content | Browser | Backend (data) | Uses supabase.auth + authStore for all data |
| Security tab content | Browser | Supabase MFA | 2FA uses supabase.auth.mfa.* directly |
| API Key management | Browser | Supabase RPC | Uses supabase.rpc() for list/create/revoke |
| Team tab (mock) | Browser | — | Pure client-side mock data, no API needed |
| Webhook management | Browser | API server | Uses webhooksApi (REST calls to backend) |
| Restart defaults | Browser | API server | Uses fetchApi to GET/PUT /api/v1/settings/restart-defaults |
| Cloudflare DNS | Browser | API server | Uses cloudflareApi (REST calls to backend) |
| S3 Storage | Browser | API server | Uses s3ProfilesApi (REST calls to backend) |
| CSS theming (cosmic) | Browser | — | Pure CSS variables + Tailwind classes, no server |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 18.x | UI framework | Existing project dependency |
| Tailwind CSS | v4 | Utility-first CSS | Existing project dependency with `@theme` cosmic CSS variables |
| Zustand | 4.x | State management | `useAuthStore`, `useUIStore` already used project-wide |
| Supabase JS | 2.x | Auth + RPC + MFA | Used for password change, 2FA, API keys |
| Lucide React | — | Icons | Already imported in CloudflareSettings.jsx |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `fetchApi` | — | App's custom API client | For REST endpoints (webhooks, restart defaults) |
| `webhooksApi` | — | Webhook API wrapper | In WebhookSettings.jsx |
| `cloudflareApi` | — | Cloudflare API wrapper | In CloudflareSettings.jsx (unchanged) |
| `s3ProfilesApi` | — | S3 profile API wrapper | In S3ProfileSettings.jsx (unchanged) |
| `uploadAvatar` | — | Avatar upload helper | In ProfileSettings.jsx |

**Verified versions:** React 18 confirmed in `package.json` (existing project), Tailwind v4 confirmed via `@import "tailwindcss"` in `index.css`, all API wrappers confirmed in `app/src/lib/api.js`.

## Architecture Patterns

### System Architecture Diagram

```
SettingsPage.jsx (Shell)
│
├── activeTab state
├── tabs array definition  
├── tab bar rendering
├── isAdmin derived (getUserRole)
├── conditional tab content:
│
├── activeTab === 'profile'  ──→  ProfileSettings.jsx (user, tab="profile")
│   ├── Personal Information (avatar, display name, email)
│   ├── Account Security (change password)
│   ├── Login Activity (login history table)
│   └── Danger Zone (delete + transfer)
│
├── activeTab === 'security'  ──→  ProfileSettings.jsx (user, tab="security")
│   ├── Two-Factor Authentication (MFA enroll/verify/disable)
│   ├── Active Sessions (current session)
│   └── Logout All Devices
│
├── activeTab === 'api'  ──→  ApiKeySettings.jsx (user, tab="api")
│   ├── Generate API Key
│   └── API Keys List
│
├── activeTab === 'team'  ──→  ApiKeySettings.jsx (user, tab="team")
│   ├── Your Role (badge)
│   ├── Invite Team Members
│   ├── Team Members List
│   └── Role Permissions Table
│
├── activeTab === 'webhooks'  ──→  WebhookSettings.jsx
│   ├── Webhook Cards
│   └── Test / Retry buttons
│
├── activeTab === 'restart-defaults'  ──→  RestartDefaultsSettings.jsx
│   ├── Max Restart Attempts
│   └── Restart Cooldown
│
├── activeTab === 'cloudflare'  ──→  CloudflareSettings.jsx (existing, cosmic restyle)
└── activeTab === 'storage'  ──→  S3ProfileSettings.jsx (existing, cosmic restyle)
```

### Recommended Project Structure
```
app/src/pages/settings/
├── SettingsPage.jsx              # Shell — imports children, tab routing, tab bar
├── ProfileSettings.jsx          # Profile + Security tab content
├── ApiKeySettings.jsx           # API Keys + Team tab content
├── WebhookSettings.jsx          # Webhooks tab content
└── RestartDefaultsSettings.jsx  # Restart Defaults tab content

app/src/components/settings/
├── CloudflareSettings.jsx       # Existing — cosmic restyle in-place
└── S3ProfileSettings.jsx        # Existing — cosmic restyle in-place
```

### Pattern 1: Tab Routing via Conditional Rendering
**What:** SettingsPage.jsx renders child components based on `activeTab` state. Components receive a `tab` prop to distinguish between the two tabs they own.

**Source:** [VERIFIED: SettingsPage.jsx lines 1060-1431]

```jsx
// SettingsPage.jsx — tab routing (after file split)
const isAdmin = ['owner', 'admin'].includes(getUserRole())

const tabs = [
  { id: 'profile', label: 'Profile' },
  { id: 'team', label: 'Team' },
  { id: 'security', label: 'Security' },
  { id: 'api', label: 'API Keys' },
  { id: 'webhooks', label: 'Webhooks' },
  ...(isAdmin ? [{ id: 'cloudflare', label: 'Cloudflare DNS' }] : []),
  ...(isAdmin ? [{ id: 'storage', label: 'Storage' }] : []),
  ...(isAdmin ? [{ id: 'restart-defaults', label: 'Restart Defaults' }] : []),
]

return (
  <div className="p-6">
    <h1 className="text-2xl font-semibold text-white mb-6">Settings</h1>
    
    {/* Tab bar */}
    <div className="flex gap-4 border-b border-[var(--color-cosmic-border)] mb-6">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => setActiveTab(tab.id)}
          className={`pb-2 px-1 text-sm font-medium ${
            activeTab === tab.id
              ? 'text-[var(--color-cosmic-cyan)] border-b-2 border-[var(--color-cosmic-cyan)]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'
          }`}
        >
          {tab.label}
        </button>
      ))}
    </div>

    {/* Tab content — child components */}
    {(activeTab === 'profile' || activeTab === 'security') && (
      <ProfileSettings user={user} tab={activeTab} />
    )}
    {(activeTab === 'api' || activeTab === 'team') && (
      <ApiKeySettings user={user} tab={activeTab} />
    )}
    {activeTab === 'webhooks' && <WebhookSettings />}
    {activeTab === 'restart-defaults' && <RestartDefaultsSettings />}
    {activeTab === 'cloudflare' && <CloudflareSettings />}
    {activeTab === 'storage' && <S3ProfileSettings />}
  </div>
)
```

### Pattern 2: Component Props Interface
**What:** Each child component receives minimal props and accesses stores directly for data.

**Source:** [VERIFIED: CloudflareSettings.jsx lines 1-6] — existing pattern for component accessing `useUIStore` directly

```jsx
// ProfileSettings.jsx
import { useState, useEffect, useCallback } from 'react'
import { useAuthStore } from '../../store/authStore'
import { useUIStore } from '../../store/uiStore'
import { supabase } from '../../lib/supabase'
import { uploadAvatar } from '../../hooks/useProfile'

export default function ProfileSettings({ user, tab }) {
  const { addToast } = useUIStore()
  // tab === 'profile' → render profile content
  // tab === 'security' → render security tab content
  // All state and handlers live inside this component
}
```

```jsx
// ApiKeySettings.jsx
import { useState, useEffect } from 'react'
import { useAuthStore } from '../../store/authStore'
import { useUIStore } from '../../store/uiStore'
import { supabase } from '../../lib/supabase'

export default function ApiKeySettings({ user, tab }) {
  const { addToast } = useUIStore()
  // tab === 'api' → render API key content
  // tab === 'team' → render team tab content
}
```

```jsx
// WebhookSettings.jsx
import { useState, useEffect } from 'react'
import { useUIStore } from '../../store/uiStore'
import { webhooksApi } from '../../lib/api'

export default function WebhookSettings() {
  const { addToast } = useUIStore()
  // Self-contained: all state, handlers, and rendering
}
```

```jsx
// RestartDefaultsSettings.jsx
import { useState, useEffect } from 'react'
import { useUIStore } from '../../store/uiStore'
import { fetchApi } from '../../api/client'

export default function RestartDefaultsSettings() {
  const { addToast } = useUIStore()
  // Self-contained: all state, handlers, and rendering
}
```

### Pattern 3: Profile Tab Sub-Section Grouping
**What:** The currently flat 6-section Profile tab is reorganized into 4 grouped sections with visual dividers.

**Source:** [CITED: UI-SPEC.md lines 161-177]

```
┌─ glass-panel ──────────────────────────────────┐
│                                                  │
│  1. Personal Information                         │
│     ├── Avatar Upload (drag/drop/click)          │
│     ├── Display Name (input + Save Changes)      │
│     └── Profile Info (name, current email,       │
│         change email form)                       │
│                                                  │
│  ── pt-6 border-t border-[var(--color-cosmic-border)] mt-6 ──
│                                                  │
│  2. Account Security                             │
│     └── Change Password (current + new + confirm)│
│                                                  │
│  ── pt-6 border-t border-[var(--color-cosmic-border)] mt-6 ──
│                                                  │
│  3. Login Activity                               │
│     └── Login History Table (or empty/error state)│
│                                                  │
│  ── pt-6 border-t border-[var(--color-cosmic-border)] mt-6 ──
│                                                  │
│  4. Danger Zone                                  │
│     └── border-l-4 border-red-500/60 pl-4        │
│         ├── Delete Account (confirmation flow)   │
│         └── Transfer Ownership                   │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Divider CSS between groups:**
```jsx
<div className="pt-6 border-t border-[var(--color-cosmic-border)] mt-6">
```

### Anti-Patterns to Avoid
- **Splitting into one file per tab:** CONTEXT.md explicitly prohibits this (too much churn). Group Profile+Security into one file, API Keys+Team into another.
- **Keeping all state in parent:** Moving state into child components reduces prop-drilling and follows the existing CloudflareSettings.jsx pattern.
- **Using `--color-bg-secondary`:** This CSS variable does NOT exist in `index.css`. Use `--color-cosmic-card` for input backgrounds instead.
- **Changing behavior during restyle:** No functional changes. No handler logic changes. Pure CSS class swaps + code reorganization.
- **Adding new imports or dependencies:** All dependencies already exist in the project.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| State management | Context for passing addToast | `useUIStore.getState().addToast` | Existing pattern in CloudflareSettings, zero boilerplate |
| Tab content switching | Custom tab component library | Simple `{activeTab === 'x' && <Component />}` | Already works, 8 tabs, no animation needed |
| CSS framework | Custom CSS for glass panels | `glass-panel` utility class | Already defined in index.css with backdrop-filter + border-radius |
| Icon library | SVG icons | lucide-react | Already imported in project (CloudflareSettings uses it) |
| Form validation | Custom validation library | Inline checks + supabase errors | Current pattern works, no complex validation needed |

**Key insight:** Every dependency needed for this phase already exists in the codebase. The entire phase is mechanical: move code between files, swap CSS class strings, and add div wrappers. No new libraries, no new APIs, no new patterns.

## Runtime State Inventory

> Not a rename/refactor/migration phase — greenfield file creation + CSS restyle. No runtime state inventory needed.

## Common Pitfalls

### Pitfall 1: Breakage from Moving Handlers to Child Components
**What goes wrong:** Handlers reference state variables that are moved to child components, but the parent still references them.

**Why it happens:** The file split moves some state to child components but leaves handler references in the parent. For example, `handlePasswordChange` references `currentPassword`, `newPassword`, `confirmPassword` — all must move together.

**How to avoid:** Move ALL related state + ALL related handlers together as a unit. Follow the dependency map below — each handler's state dependencies are documented.

**Warning signs:** React errors about undefined state setters, or handlers that can't access their state.

### Pitfall 2: Import Path Drift
**What goes wrong:** After file splitting, imported functions like `fetchApi`, `supabase`, `uploadAvatar` are imported in the parent but needed in children.

**How to avoid:** Each child component adds its own imports. This is safe because all dependencies already exist. The child should NOT rely on the parent to pass `supabase` or `fetchApi` as props.

### Pitfall 3: Accidentally Merging Security Tab Into Profile
**What goes wrong:** The CONTEXT.md says "Security tab stays in its own tab" but ProfileSettings.jsx groups both profile + security content. Implementor might merge security content INTO the profile tab's sub-sections.

**How to avoid:** ProfileSettings.jsx renders DIFFERENT content based on `tab` prop. When `tab === 'profile'`, render the 4 sub-sections. When `tab === 'security'`, render the MFA/sessions/logout content. The tab bar still shows them as separate tabs.

### Pitfall 4: Using `--color-bg-secondary` for Input Backgrounds
**What goes wrong:** CONTEXT.md mentions `bg-[var(--color-bg-secondary)]` but this CSS variable is NOT defined in `index.css`.

**How to avoid:** Use `bg-[var(--color-cosmic-card)]` for input backgrounds. UI-SPEC.md correctly uses `--color-cosmic-card`.

### Pitfall 5: Changing `rounded` → `rounded-xl` on Inline Elements  
**What goes wrong:** `rounded-xl` applies 12px border-radius. On small elements like badges or status dots, 12px border-radius might look wrong.

**How to avoid:** For small/badge elements (like `w-2 h-2` status dots), keep their existing border-radius. Only change `rounded` → `rounded-xl` on: input fields, buttons, panel containers, card items, and form elements. Small elements, table cells, and inline decorations keep their existing `rounded` or `rounded-full`.

## Code Examples

### Pattern: Input Field Cosmic Replacement
```jsx
// BEFORE:
className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"

// AFTER:
className="w-full px-4 py-2 bg-[var(--color-cosmic-card)] text-[var(--color-text-main)] rounded-xl border border-[var(--color-cosmic-border)] focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)] focus:border-[var(--color-cosmic-cyan)] transition-colors"
```

### Pattern: Secondary Button Replacement
```jsx
// BEFORE:
className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500"

// AFTER:
className="px-4 py-2 bg-[var(--color-nebula)] text-[var(--color-text-main)] rounded-xl border border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.06)] transition-colors"
```

### Pattern: Status Badge (Webhook Health)
```jsx
// BEFORE:
className={`px-2 py-1 rounded text-xs ${
  webhook.failure_count > 0
    ? 'bg-red-600 text-white'
    : 'bg-green-600 text-white'
}`}

// AFTER:
className={`px-2 py-1 rounded-xl text-xs font-normal ${
  webhook.failure_count > 0
    ? 'bg-[var(--color-cosmic-red)]/10 text-[var(--color-cosmic-red)]'
    : 'bg-[var(--color-cosmic-green)]/10 text-[var(--color-cosmic-green)]'
}`}
```

### Pattern: Glass Panel Section Wrapping
```jsx
// Each tab's content gets wrapped in glass-panel:
<div className="glass-panel p-6">
  {/* tab content */}
  {/* exception: Danger Zone confirmation gets bg-[var(--color-nebula)] instead */}
</div>

// Before tab content:
<div className="space-y-6">
  {/* ... */}
</div>

// After:
<div className="glass-panel p-6">
  <div className="space-y-6">
    {/* ... */}
  </div>
</div>
```

### Pattern: Role Badge with Cosmic Border
```jsx
// BEFORE:
<span className={`px-2 py-1 ${r.bg} text-white text-xs rounded`}>

// AFTER:
<span className={`px-2 py-1 ${r.bg} text-white text-xs rounded-xl border border-[var(--color-cosmic-border)]`}>
```

### Pattern: API Key Reveal Banner
```jsx
// BEFORE:
<div className="p-4 bg-green-900 border border-green-700 rounded mb-4">

// AFTER:
<div className="p-4 bg-[var(--color-cosmic-green)]/10 border border-[var(--color-cosmic-green)]/30 rounded-xl mb-4">
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single 1433-line SettingsPage.jsx | Shell + 4 extracted component files | This phase | Easier maintenance, clearer ownership |
| Flat `bg-gray-700`/`bg-gray-800` styling | Cosmic `glass-panel` theme with CSS variables | This phase | Visual consistency with all other pages |
| Flat Profile tab (6 sections) | Grouped Profile tab (4 sub-sections with dividers) | This phase | Better information hierarchy |
| `focus:ring-blue-500` on all inputs | `focus:ring-[var(--color-cosmic-cyan)]` | This phase | Matches cosmic accent color system |
| `rounded` on all elements | `rounded-xl` on containers/inputs, keep `rounded`/`rounded-full` on small elements | This phase | Softer, more modern appearance |

**Deprecated/outdated:**
- `bg-gray-700` for card backgrounds → replaced by `bg-[var(--color-cosmic-card)]` + `glass-panel`
- `bg-gray-800` for darker panels → replaced by `bg-[var(--color-nebula)]`
- `border-gray-700` for dividers → replaced by `border-[var(--color-cosmic-border)]`
- `rounded` (4px) on containers → replaced by `rounded-xl` (12px) via `glass-panel`

## Assumptions Log

No claims in this research are `[ASSUMED]`. All findings are verified against source code (SettingsPage.jsx, CloudflareSettings.jsx, S3ProfileSettings.jsx, index.css, authStore.js) or documented in CONTEXT.md / UI-SPEC.md.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | None — all claims verified | — | — |

## Open Questions (RESOLVED)

1. **Should ProfileSettings.jsx handle tab switching internally or receive `tab` prop?**
   - What we know: Component needs different rendering for 'profile' vs 'security' activeTab values
   - What's unclear: Whether to use `tab` prop (parent passes `activeTab`) or wrap in an internal switch
   - Recommendation: Use `tab` prop from parent — simple, explicit, easy to test
   - **RESOLVED:** Plan 80-01 implements `tab` prop pattern. ProfileSettings receives `{ user, tab }` and conditionally renders profile vs security content based on the prop value.

2. **Should `getRoleBadge` be duplicated in both ApiKeySettings.jsx and SettingsPage.jsx shell, or extracted to a shared utility?**
   - What we know: Used by Team tab (moves to ApiKeySettings.jsx) and potentially by Profile tab
   - What's unclear: Whether ProfileSettings.jsx needs role badges
   - Recommendation: Define `getRoleBadge` inline in ApiKeySettings.jsx since it's only 14 lines of JSX + color mapping. If ProfileSettings also needs it, extract to a small helper.
   - **RESOLVED:** Plan 80-01 defines `getRoleBadge` inline in ApiKeySettings.jsx (the Team tab consumer). ProfileSettings.jsx does not need it.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | React app dev server | ✓ | — | — |
| npm | Package management | ✓ | — | — |
| Tailwind CLI | CSS processing | ✓ | v4 | Build step in existing scripts |
| Supabase project | Auth, RPC, MFA | ✓ (existing) | — | — |

**Missing dependencies with no fallback:** None — all dependencies are already set up and working in the project.

## Validation Architecture

> `workflow.nyquist_validation` is not explicitly set in config.json (absent). Treat as disabled by convention — no test infrastructure needed for this pure CSS restyle + file extraction phase.

### Test Infrastructure
**No existing tests detected.** SettingsPage has zero test files. Prior phases (Phase 79 research confirmed "Zero test files for billing"). This is a pure visual restyle — manual verification via browser is the validation method.

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command |
|--------|----------|-----------|-------------------|
| SPLIT-01 | All 8 tabs render after file split | Manual | `npm run dev` + click each tab |
| RESTYLE-01 | No remaining `bg-gray-700` etc. in new files | Manual | `grep -r "bg-gray-7\|border-gray-7\|focus:ring-blue" app/src/pages/settings/` |
| RESTYLE-02 | Cloudflare/S3 have no gray classes | Manual | `grep -r "bg-gray-7\|border-gray-7\|focus:ring-blue" app/src/components/settings/` |
| NO-CHANGE-01 | All handlers work (password, MFA, API keys) | Manual | Functional smoke test in browser |

### Wave 0 Gaps
- No test framework setup exists for UI component testing
- All validation is manual/browser-based
- Phase recommendation: Post-restyle grep for remaining old classes as automated check

## Security Domain

> `security_enforcement` is not set in config.json. Omit Security Domain section — this phase has no security-relevant changes (pure CSS restyle + code reorganization, zero functional changes to auth/crypto/input handling).

## Source Code Analysis

### SettingsPage.jsx — Complete Structure Map

**Imports (lines 1-9):**
```jsx
import { useState, useEffect, useCallback } from 'react'
import { useAuthStore } from '../../store/authStore'
import { useUIStore } from '../../store/uiStore'
import { supabase, signOut } from '../../lib/supabase'
import { fetchApi } from '../../api/client'
import { webhooksApi, cloudflareApi } from '../../lib/api'
import { useProfile, uploadAvatar } from '../../hooks/useProfile'
import CloudflareSettings from '../../components/settings/CloudflareSettings'
import S3ProfileSettings from '../../components/settings/S3ProfileSettings'
```

**State variables (lines 15-71) — ~30 variables:**

| Variable | Type | Owner | Used By |
|----------|------|-------|---------|
| `name` | string | ProfileSettings | Profile tab — name field |
| `email` | string | ProfileSettings | Profile tab — current email display |
| `activeTab` | string | **SettingsPage shell** | Tab routing (STAYS) |
| `isLoading` | boolean | ProfileSettings | Logout all loading (ProfileSettings + handleLogoutAll) |
| `currentPassword` | string | ProfileSettings | Password change form |
| `newPassword` | string | ProfileSettings | Password change form |
| `confirmPassword` | string | ProfileSettings | Password change form |
| `passwordLoading` | boolean | ProfileSettings | Password change form |
| `mfaEnabled` | boolean | ProfileSettings | Security tab — 2FA status |
| `mfaLoading` | boolean | ProfileSettings | Security tab — 2FA operations |
| `showMfaSetup` | boolean | ProfileSettings | Security tab — QR flow |
| `mfaQrCode` | string | ProfileSettings | Security tab — QR image URL |
| `mfaSecret` | string | ProfileSettings | Security tab — QR secret |
| `mfaFactorId` | string | ProfileSettings | Security tab — MFA factor |
| `mfaChallengeId` | string | ProfileSettings | Security tab — MFA challenge |
| `mfaCode` | string | ProfileSettings | Security tab — verification code |
| `displayName` | string | ProfileSettings | Profile tab — display name field |
| `avatarUrl` | string | ProfileSettings | Profile tab — avatar display |
| `savingProfile` | boolean | ProfileSettings | Profile tab — save button state |
| `uploadingAvatar` | boolean | ProfileSettings | Profile tab — upload state |
| `dragOver` | boolean | ProfileSettings | Profile tab — drag state |
| `newEmail` | string | ProfileSettings | Profile tab — email change |
| `changingEmail` | boolean | ProfileSettings | Profile tab — email change loading |
| `loginHistory` | array | ProfileSettings | Profile tab — login history data |
| `loginHistoryLoading` | boolean | ProfileSettings | Profile tab — loading state |
| `loginHistoryError` | string/null | ProfileSettings | Profile tab — error state |
| `deletePassword` | string | ProfileSettings | Danger Zone — password input |
| `deleteConfirmText` | string | ProfileSettings | Danger Zone — confirm text |
| `showDeleteConfirm` | boolean | ProfileSettings | Danger Zone — toggle |
| `deletingAccount` | boolean | ProfileSettings | Danger Zone — loading |
| `transferEmail` | string | ProfileSettings | Danger Zone — transfer email |
| `showTransfer` | boolean | ProfileSettings | Danger Zone — toggle |
| `apiKeys` | array | ApiKeySettings | API Keys tab |
| `showNewApiKey` | string | ApiKeySettings | API Keys tab — reveal banner |
| `apiLoading` | boolean | ApiKeySettings | API Keys tab — generation |
| `globalRestart` | object | RestartDefaultsSettings | Restart Defaults tab |
| `globalRestartLoading` | boolean | RestartDefaultsSettings | Restart Defaults tab |
| `globalRestartSaving` | boolean | RestartDefaultsSettings | Restart Defaults tab |
| `teamMembers` | array | ApiKeySettings | Team tab — mock data |
| `inviteEmail` | string | ApiKeySettings | Team tab — invite form |
| `inviteRole` | string | ApiKeySettings | Team tab — invite form |
| `webhooks` | array | WebhookSettings | Webhooks tab |
| `webhooksLoading` | boolean | WebhookSettings | Webhooks tab |
| `testingWebhook` | string/null | WebhookSettings | Webhooks tab |
| `retryingWebhook` | string/null | WebhookSettings | Webhooks tab |

**Utility functions (lines 72-473):**
- `getUserRole()` → inline in each component that needs it
- `getRoleBadge(role)` → define in ApiKeySettings.jsx (used by Team tab)
- `checkMfaStatus()` → move to ProfileSettings.jsx (Security tab)
- `handlePasswordChange()` → move to ProfileSettings.jsx
- `handleEmailChange()` → move to ProfileSettings.jsx
- `handleAvatarUpload/handleAvatarClick/handleAvatarDrop/handleSaveDisplayName` → move to ProfileSettings.jsx
- `fetchLoginHistory()` → move to ProfileSettings.jsx
- `handleDeleteAccount()` → move to ProfileSettings.jsx
- `handleTransfer()` → move to ProfileSettings.jsx
- `handleEnable2FA/handleVerify2FA/handleDisable2FA` → move to ProfileSettings.jsx
- `loadApiKeys/generateApiKey/copyApiKey/deleteApiKey` → move to ApiKeySettings.jsx
- `loadRestartDefaults()` → move to RestartDefaultsSettings.jsx
- `handleLogoutAll/handleLogoutCurrent` → move to ProfileSettings.jsx (Security tab)
- `handleInvite/handleRemoveMember` → move to ApiKeySettings.jsx (Team tab)
- `loadWebhooks/handleTestWebhook/handleRetryWebhook/formatRelativeTime` → move to WebhookSettings.jsx

**Render functions (lines 475-1399):**

| Function | Lines | Move To | Notes |
|----------|-------|---------|-------|
| `renderProfileTab()` | 475-790 | ProfileSettings.jsx | ~315 lines, 6 current sections → 4 grouped |
| `renderSecurityTab()` | 792-916 | ProfileSettings.jsx | ~125 lines, 2FA + sessions |
| `renderApiTab()` | 918-983 | ApiKeySettings.jsx | ~65 lines, API key management |
| `renderRestartDefaultsTab()` | 985-1056 | RestartDefaultsSettings.jsx | ~72 lines, inline form |
| `renderTeamTab()` | 1108-1263 | ApiKeySettings.jsx | ~155 lines, team mock |
| `renderWebhooksTab()` | 1330-1399 | WebhookSettings.jsx | ~70 lines, webhook cards |

**Tab bar + routing (lines 1058-1432):**
- `isAdmin` derived from `getUserRole()` → stays in SettingsPage shell
- `tabs` array → stays in SettingsPage shell
- Team mock data initialization (lines 1072-1080) → moves to ApiKeySettings.jsx
- Main return block (lines 1401-1431) → stays in SettingsPage shell, conditionally renders children

### CSS Class Inventory — Complete Before/After Mapping

| # | Old Pattern | New Pattern | Occurrences | Notes |
|---|------------|-------------|-------------|-------|
| 1 | `bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500` (input) | `bg-[var(--color-cosmic-card)] text-[var(--color-text-main)] rounded-xl border border-[var(--color-cosmic-border)] focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)] focus:border-[var(--color-cosmic-cyan)] transition-colors` | ~25 | Most common pattern; every `<input>`, `<select>`, `<textarea>` |
| 2 | `bg-gray-700 text-white rounded focus:outline-none` (select) | `bg-[var(--color-cosmic-card)] text-[var(--color-text-main)] rounded-xl border border-[var(--color-cosmic-border)] focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)]` | 1 | Select dropdown (team role picker) |
| 3 | `bg-gray-600 text-gray-400 rounded cursor-not-allowed` (disabled input) | `bg-[var(--color-nebula)] text-[var(--color-text-muted)] rounded-xl border border-[var(--color-cosmic-border)] cursor-not-allowed` | 1 | Disabled email input |
| 4 | `focus:ring-2 focus:ring-red-500` (danger input) | `focus:ring-2 focus:ring-[var(--color-cosmic-red)] focus:border-[var(--color-cosmic-red)]` | 2 | Delete confirmation inputs |
| 5 | `text-gray-400` (label) | `text-[var(--color-text-muted)]` | ~15 | All `<label>` elements |
| 6 | `text-gray-400 text-sm` (description) | `text-[var(--color-text-muted)] text-sm` | ~8 | Paragraph descriptions |
| 7 | `text-gray-500 text-xs` (hint text) | `text-[var(--color-text-muted)] text-xs` | ~5 | Helper text below inputs |
| 8 | `text-white` (heading) | `text-[var(--color-text-main)]` | ~10 | Section headings (`<h3>`) |
| 9 | `text-lg font-medium text-white mb-4` (heading) | `text-lg font-semibold text-[var(--color-text-main)] mb-4` | ~8 | Section headings, consolidate `font-medium` → `font-semibold` |
| 10 | `text-lg font-semibold text-white mb-4` (heading) | `text-lg font-semibold text-[var(--color-text-main)] mb-4` | ~3 | Already semibold, just change color |
| 11 | `text-blue-400` (tab active / link) | `text-[var(--color-cosmic-cyan)]` | 3 | Tab active, transfer link |
| 12 | `border-b-2 border-blue-400` (tab active) | `border-b-2 border-[var(--color-cosmic-cyan)]` | 1 | Tab bar active indicator |
| 13 | `text-gray-400 hover:text-white` (tab inactive) | `text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]` | 1 | Tab bar inactive tabs |
| 14 | `border-b border-gray-700` (tab bar, tables, dividers) | `border-b border-[var(--color-cosmic-border)]` | ~10 | Tab bar separator, table headers, table rows |
| 15 | `pt-6 border-t border-gray-700` (divider) | `pt-6 border-t border-[var(--color-cosmic-border)]` | ~8 | Dividers between sub-sections |
| 16 | `border border-gray-700` (table rows) | `border-b border-[var(--color-cosmic-border)]` | ~5 | Table row bottom borders |
| 17 | `bg-gray-700 rounded-lg` (card item) | `bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl` | ~8 | Webhook cards, S3 profile cards, empty state box |
| 18 | `p-3 bg-gray-700 rounded` (list item) | `p-3 bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl` | ~4 | API key items, team member items, session items |
| 19 | `bg-gray-700 rounded` (QR panel) | `bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl` | 1 | 2FA QR code panel |
| 20 | `bg-gray-800 rounded` (danger panel) | `bg-[var(--color-nebula)] border border-red-500/60 rounded-xl` | 1 | Delete confirmation panel |
| 21 | `border-red-900` (danger panel border) | `border-red-500/60` | 1 | Delete confirmation panel |
| 22 | `border-l-4 border-red-500 pl-4` (danger accent) | `border-l-4 border-red-500/60 pl-4` | 1 | Danger Zone left accent |
| 23 | `p-4 bg-green-900 border border-green-700 rounded` (API reveal) | `p-4 bg-[var(--color-cosmic-green)]/10 border border-[var(--color-cosmic-green)]/30 rounded-xl` | 1 | New API key banner |
| 24 | `px-2 py-1 rounded text-xs bg-red-600 text-white` (badge) | `px-2 py-1 rounded-xl text-xs font-normal bg-[var(--color-cosmic-red)]/10 text-[var(--color-cosmic-red)]` | variable | Webhook failure badge |
| 25 | `px-2 py-1 rounded text-xs bg-green-600 text-white` (badge) | `px-2 py-1 rounded-xl text-xs font-normal bg-[var(--color-cosmic-green)]/10 text-[var(--color-cosmic-green)]` | variable | Webhook healthy badge |
| 26 | `px-2 py-1 \${r.bg} text-white text-xs rounded` (role badge) | `px-2 py-1 \${r.bg} text-white text-xs rounded-xl border border-[var(--color-cosmic-border)]` | 1 | `getRoleBadge` function |
| 27 | `bg-gray-600 text-white rounded hover:bg-gray-500` (gray button) | `bg-[var(--color-nebula)] text-[var(--color-text-main)] rounded-xl border border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.06)] transition-colors` | ~5 | Cancel/Test secondary buttons |
| 28 | `bg-blue-600 ... rounded` (blue button) | Same but `rounded-xl` | ~12 | Primary action buttons |
| 29 | `bg-red-600 ... rounded` (red button) | Same but `rounded-xl` | ~5 | Destructive buttons |
| 30 | `bg-green-600 ... rounded` (green button) | Same but `rounded-xl` | ~2 | Enable 2FA, Verify & Enable |
| 31 | `bg-orange-600 ... rounded` (orange button) | Same but `rounded-xl` | 1 | Retry webhook |
| 32 | `text-gray-400 border-b border-gray-700` (table header) | `text-[var(--color-text-muted)] border-b border-[var(--color-cosmic-border)]` | 2 | `<th>` row in login history + role permissions |
| 33 | `border-b border-gray-700 hover:bg-[rgba(255,255,255,0.03)]` (table row) | `border-b border-[var(--color-cosmic-border)] hover:bg-[rgba(255,255,255,0.03)] transition-colors` | ~2 | `<tr>` in tables |
| 34 | `h-10 bg-gray-700 rounded animate-pulse` (skeleton) | `h-10 bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl animate-pulse` | 1 | Loading skeleton |
| 35 | `text-center py-8 text-gray-400` (empty state) | `text-center py-8 text-[var(--color-text-muted)]` | 2 | Empty login history, empty webhooks |
| 36 | `text-gray-400` (loading text) | `text-[var(--color-text-muted)]` | ~3 | Loading indicators |
| 37 | `rounded` (on buttons) | `rounded-xl` | ~25 | All `<button>` elements |
| 38 | `text-2xl font-bold text-white mb-6` (page title) | `text-2xl font-semibold text-white mb-6` | 1 | `font-bold` → `font-semibold` per UI-SPEC |
| 39 | `text-blue-400 hover:text-blue-300` (transfer link) | `text-[var(--color-cosmic-cyan)] hover:text-[var(--color-cosmic-cyan)]/80` | 1 | Transfer ownership link |
| 40 | `ml-3 text-sm text-blue-400 hover:text-blue-300` | `ml-3 text-sm text-[var(--color-cosmic-cyan)] hover:text-[var(--color-cosmic-cyan)]/80` | 1 | Transfer ownership link |

### CloudflareSettings.jsx — Class Replacements (Existing Component)

| Old Pattern | New Pattern | Lines | Count |
|------------|-------------|-------|-------|
| `bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500` | Standard input pattern (#1) | 134, 149, 159, 171, 187, 228 | ~6 |
| `bg-gray-600 text-white rounded hover:bg-gray-500` | Secondary button pattern (#27) | 210, 247 | 2 |
| `border-gray-700` | `border-[var(--color-cosmic-border)]` | 200 | 1 |
| `text-gray-400` (label/description) | `text-[var(--color-text-muted)]` | 100, 105, 125, 137, 165, 173, 189, 204, 221, 230 | ~10 |
| `text-white text-sm font-medium` | `text-[var(--color-text-main)] text-sm font-medium` | 203 | 1 |
| `text-lg font-medium text-white mb-2` | `text-lg font-semibold text-[var(--color-text-main)] mb-2` | 99 | 1 |
| `text-blue-400` (code elements) | Keep as-is (accent color) | 101, 104, 193 | ~3 |
| `bg-green-900/30 border border-green-700 rounded` | `bg-[var(--color-cosmic-green)]/10 border border-[var(--color-cosmic-green)]/30 rounded-xl` | 109 | 1 |
| `bg-yellow-900/30 border border-yellow-700 rounded` | `bg-[var(--color-cosmic-orange)]/10 border border-[var(--color-cosmic-orange)]/30 rounded-xl` | 116 | 1 |
| `bg-blue-600` (toggle) | Keep (maps to `--color-cosmic-blue`) | 210 | 1 |
| `rounded` (general) | `rounded-xl` (on containers, inputs, buttons) | various | ~8 |

### S3ProfileSettings.jsx — Class Replacements (Existing Component)

| Old Pattern | New Pattern | Lines | Count |
|------------|-------------|-------|-------|
| `bg-gray-700 rounded-lg` (card/container) | `bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl` | 121, 129, 172 | ~3 |
| `bg-gray-800 ... focus:ring-blue-500` (input) | Standard input pattern (#1, but bg-gray-800 → bg-[var(--color-cosmic-card)]) | 181, 193, 204, 216, 228, 243 | ~6 |
| `bg-gray-600 text-white rounded hover:bg-gray-500` | Secondary button pattern (#27) | 269 | 1 |
| `bg-blue-600 ... rounded` | Same but `rounded-xl` | 135, 145, 165, 262 | ~4 |
| `bg-red-600 ... rounded` | Same but `rounded-xl` | 151 | 1 |
| `text-gray-400` (label/description) | `text-[var(--color-text-muted)]` | 114, 122, 138, 174, 186, 198, 209, 220, 232, 255 | ~10 |
| `text-lg font-medium text-white mb-2` | `text-lg font-semibold text-[var(--color-text-main)] mb-2` | 113 | 1 |
| `text-white font-medium` | `text-[var(--color-text-main)] font-medium` | 133 | 1 |
| `text-xs bg-blue-600 text-white px-2 py-0.5 rounded` (badge) | `text-xs bg-blue-600 text-white px-2 py-0.5 rounded-xl` | 135 | 1 |
| `rounded` (general) | `rounded-xl` (on containers, buttons) | various | ~8 |
| `rounded bg-gray-800 border-gray-600` (checkbox) | `rounded bg-[var(--color-cosmic-card)] border-[var(--color-cosmic-border)]` | 253 | 1 |

## Dependency Graph

### File Dependencies
```
SettingsPage.jsx (shell)
├── ProfileSettings.jsx     → imports: useState, useEffect, useCallback, useAuthStore, useUIStore, supabase, uploadAvatar
├── ApiKeySettings.jsx      → imports: useState, useEffect, useAuthStore, useUIStore, supabase
├── WebhookSettings.jsx     → imports: useState, useEffect, useUIStore, webhooksApi
├── RestartDefaultsSettings.jsx → imports: useState, useEffect, useUIStore, fetchApi
├── CloudflareSettings.jsx  → imports: useState, useEffect, useUIStore, cloudflareApi (EXISTING)
└── S3ProfileSettings.jsx   → imports: useState, useEffect, useUIStore, s3ProfilesApi (EXISTING)
```

### Shared Module Dependencies
```
useAuthStore ('../../store/authStore')
  Used by: SettingsPage.jsx, ProfileSettings.jsx, ApiKeySettings.jsx
  Provides: user, logout, refreshUser, changeEmail, updateProfile, fetchLoginHistory,
            requestAccountDeletion, transferOwnership

useUIStore ('../../store/uiStore')
  Used by: ALL components (shell + 4 new + 2 existing)
  Provides: addToast

supabase ('../../lib/supabase')
  Used by: ProfileSettings.jsx (auth.updateUser, auth.mfa.*, rpc), ApiKeySettings.jsx (rpc)

webhooksApi ('../../lib/api')
  Used by: WebhookSettings.jsx

cloudflareApi ('../../lib/api')
  Used by: CloudflareSettings.jsx (EXISTING)

s3ProfilesApi ('../../lib/api')
  Used by: S3ProfileSettings.jsx (EXISTING)

fetchApi ('../../api/client')
  Used by: RestartDefaultsSettings.jsx, SettingsPage.jsx (shell may still import if needed)
```

### Circular Dependency Risks
**None identified.** The dependency graph is a simple tree: SettingsPage shell → children → shared modules. No child imports another child. No child re-imports the shell.

## Plan Decomposition Recommendation

### Plan 1: File Splitting + New Component Creation
**Scope:** Create ProfileSettings.jsx, ApiKeySettings.jsx, WebhookSettings.jsx, RestartDefaultsSettings.jsx. Reduce SettingsPage.jsx to shell. Move all state, handlers, and render functions to appropriate children. All children use existing (non-cosmic) CSS classes.

**Tasks:**
1. Create `ProfileSettings.jsx` — move renderProfileTab + renderSecurityTab + all profile/security state/handlers
2. Create `ApiKeySettings.jsx` — move renderApiTab + renderTeamTab + all API key/team state/handlers + getRoleBadge()
3. Create `WebhookSettings.jsx` — move renderWebhooksTab + all webhook state/handlers + formatRelativeTime()
4. Create `RestartDefaultsSettings.jsx` — move renderRestartDefaultsTab + all restart defaults state/handlers
5. Reduce `SettingsPage.jsx` — keep only: imports of children, activeTab state, tabs array, tab bar, isAdmin, conditional rendering

**Verification:** All 8 tabs render correctly in browser. No functional regressions.

### Plan 2: Cosmic Theme Restyle (New Components) + Profile Tab Sub-Section Grouping
**Scope:** Apply all CSS class replacements to the 4 new component files and SettingsPage.jsx shell. Add sub-section grouping with dividers in ProfileSettings.jsx.

**Tasks:**
1. SettingsPage.jsx — Replace tab bar classes, page title class
2. ProfileSettings.jsx — Apply all CSS replacements + add sub-section dividers + wrap profile content in glass-panel + add "Account Security" heading
3. ApiKeySettings.jsx — Apply all CSS replacements
4. WebhookSettings.jsx — Apply all CSS replacements
5. RestartDefaultsSettings.jsx — Apply all CSS replacements

**Verification:** `grep -r "bg-gray-7\|border-gray-7\|focus:ring-blue" app/src/pages/settings/` returns empty. Visual check of all 8 tabs in browser. Profile tab shows 4 grouped sections.

### Plan 3: Cloudflare + S3 Storage Cosmic Restyle
**Scope:** Apply same CSS class replacements to the existing CloudflareSettings.jsx and S3ProfileSettings.jsx.

**Tasks:**
1. CloudflareSettings.jsx — Replace all input/button/container/badge classes
2. S3ProfileSettings.jsx — Replace all input/button/container/badge classes

**Verification:** `grep -r "bg-gray-7\|border-gray-7\|focus:ring-blue" app/src/components/settings/` returns empty. Both Cloudflare DNS and Storage tabs render correctly with cosmic theme.

### Execution Order

```
Wave 1 (parallel):
├── Plan 1: File splitting (no style changes)
└── Plan 3: Cloudflare/S3 restyle (touches different files)

Wave 2 (sequential, depends on Plan 1):
└── Plan 2: Cosmic restyle of new components + Profile tab grouping
```

**Rationale:** Plans 1 and 3 touch completely different files (Plan 1: `app/src/pages/settings/*`, Plan 3: `app/src/components/settings/*`) and can execute in parallel. Plan 2 depends on the files created in Plan 1. This gives clean, reviewable diffs at each stage.

## Sources

### Primary (HIGH confidence)
- [VERIFIED: SettingsPage.jsx] — Full structure analysis of all 1433 lines, state variables, handlers, render functions, tab routing
- [VERIFIED: CloudflareSettings.jsx] — 255 lines, existing cosmic pattern analysis, class inventory
- [VERIFIED: S3ProfileSettings.jsx] — 278 lines, class inventory
- [VERIFIED: index.css lines 4-19] — Cosmic CSS variables (`@theme` block), confirmed NO `--color-bg-secondary`
- [VERIFIED: index.css lines 75-80] — `glass-panel` class definition
- [VERIFIED: authStore.js] — All available store methods (updateProfile, fetchLoginHistory, changeEmail, etc.)
- [VERIFIED: useProfile.js] — uploadAvatar function
- [VERIFIED: CONTEXT.md] — Locked design decisions
- [VERIFIED: UI-SPEC.md] — Visual/interaction contract, exact CSS class mappings

### Secondary (MEDIUM confidence)
- [CITED: CloudflareSettings.jsx lines 1-6] — Pattern for using `useUIStore` directly in child components
- [CITED: Phase 79 RESEARCH.md] — Confirmed "Zero test files" pattern for UI pages
- [CITED: Phase 79 execution pattern] — 2-plan structure (backend + frontend) for restyle phases

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All dependencies verified against source code
- Architecture: HIGH — File structure, props design, and tab routing verified against SettingsPage.jsx
- CSS mapping: HIGH — Every pattern verified against actual source, replacements tested against index.css variables
- Plan decomposition: HIGH — Based on prior phase patterns and dependency analysis

**Research date:** 2026-06-15
**Valid until:** 2026-07-15 (30-day window for stable frontend patterns — CSS variables won't change)
