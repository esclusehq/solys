# Phase 80 Plan 02: Cosmic Theme Restyle of Settings Pages

One-liner: Cosmic theme restyle across all 5 Settings page components — CSS variables replacing hardcoded gray/blue classes, glass-panel wrapping, 4 sub-section regrouping of ProfileSettings, and button/badge/table/heath-badge pattern alignment with the existing cosmic design.

## Files Modified

| File | Description | Lines |
|------|-------------|-------|
| `app/src/pages/settings/SettingsPage.jsx` | Tab bar and title cosmic restyle | 63 |
| `app/src/pages/settings/ProfileSettings.jsx` | Full restyle + 4 sub-section regroup + security tab | 819 |
| `app/src/pages/settings/ApiKeySettings.jsx` | Glass-panel wrap + cosmic API/team tabs | 353 |
| `app/src/pages/settings/WebhookSettings.jsx` | Glass-panel wrap + cosmic health badges + cards | 142 |
| `app/src/pages/settings/RestartDefaultsSettings.jsx` | Glass-panel wrap + cosmic inputs/buttons | 105 |

## Tasks Completed

1. **Cosmic restyle SettingsPage.jsx shell** (Task 1)
   - Page title: `font-bold` → `font-semibold`
   - Tab bar separator: `border-gray-700` → `border-[var(--color-cosmic-border)]`
   - Active tab: `text-blue-400 border-b-2 border-blue-400` → `text-[var(--color-cosmic-cyan)] border-b-2 border-[var(--color-cosmic-cyan)]`
   - Inactive tab: `text-gray-400 hover:text-white` → `text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]`
   - Added `transition-colors` to tab buttons

2. **Cosmic restyle ProfileSettings.jsx + sub-section grouping** (Task 2, most complex)
   - Profile tab wrapped in `glass-panel p-6 border border-[var(--color-cosmic-border)]`
   - Reorganized flat layout into 4 sub-sections with cosmic-themed dividers:
     - **Section 1 — Personal Information**: avatar, display name, profile info (name, email, change email)
     - **Divider**: `pt-6 border-t border-[var(--color-cosmic-border)] mt-6`
     - **Section 2 — Account Security**: new `<h3>Account Security</h3>` heading + change password form
     - **Section 3 — Login Activity**: login history with skeleton/empty/error/table states
     - **Section 4 — Danger Zone**: `border-l-4 border-red-500/60 pl-4` wrapper + delete + transfer
   - All inputs: `bg-gray-700 text-white rounded` → `bg-[var(--color-cosmic-card)] text-[var(--color-text-main)] rounded-xl border border-[var(--color-cosmic-border)] focus:ring-[var(--color-cosmic-cyan)]`
   - All buttons: added `rounded-xl transition-colors`
   - Disabled inputs: `bg-gray-600` → `bg-[var(--color-nebula)]`
   - Danger inputs: `focus:ring-red-500` → `focus:ring-[var(--color-cosmic-red)]`
   - All text labels/descriptions: `text-gray-400` → `text-[var(--color-text-muted)]`
   - All h3 headings: `text-white` → `text-[var(--color-text-main)]`
   - Table header: `text-gray-400 border-b border-gray-700` → `text-[var(--color-text-muted)] border-b border-[var(--color-cosmic-border)]`
   - Table body: `text-gray-300` → `text-[var(--color-text-main)]`
   - Green status: `text-green-400` → `text-[var(--color-cosmic-green)]`
   - Danger heading: `text-red-400` → `text-[var(--color-cosmic-red)]`
   - Cancel/secondary buttons: `bg-gray-600` → `bg-[var(--color-nebula)] border border-[var(--color-cosmic-border)]`
   - Security tab: all mappings applied (inputs, headings, labels, dividers, session items, QR panel)
   - QR panel: `bg-gray-700 rounded` → `bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl`

3. **Cosmic restyle ApiKeySettings.jsx** (Task 3)
   - Both API tab and Team tab wrapped in `glass-panel p-6 border border-[var(--color-cosmic-border)]`
   - API key reveal banner: `bg-green-900 border-green-700 rounded` → `bg-[var(--color-cosmic-green)]/10 border border-[var(--color-cosmic-green)]/30 rounded-xl`
   - API key code block: `bg-gray-800 text-green-400` → `bg-[var(--color-nebula)] text-[var(--color-cosmic-green)]`
   - Copy button: `bg-green-700` → `bg-[var(--color-cosmic-green)]/20 text-[var(--color-cosmic-green)]`
   - getRoleBadge: added `rounded-xl border border-[var(--color-cosmic-border)]`
   - Avatar initials: `w-8 h-8 bg-gray-600 rounded-full` → `w-8 h-8 bg-[var(--color-nebula)] border border-[var(--color-cosmic-border)] rounded-full`
   - Select dropdown: cosmic card styling
   - All standard CSS mappings applied (inputs, labels, table headers/body, dividers, buttons)

4. **Cosmic restyle WebhookSettings.jsx** (Task 3)
   - All content wrapped in `glass-panel p-6 border border-[var(--color-cosmic-border)]`
   - Webhook cards: `p-4 bg-gray-700 rounded-lg` → `p-4 bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl`
   - Health badges: failure `bg-red-600 text-white` → `bg-[var(--color-cosmic-red)]/10 text-[var(--color-cosmic-red)]`
   - Health badges: healthy `bg-green-600 text-white` → `bg-[var(--color-cosmic-green)]/10 text-[var(--color-cosmic-green)]`
   - All badges: `rounded` → `rounded-xl text-xs font-normal`
   - All standard CSS mappings applied

5. **Cosmic restyle RestartDefaultsSettings.jsx** (Task 3)
   - All content wrapped in `glass-panel p-6 border border-[var(--color-cosmic-border)]`
   - Inputs, labels, headings, helper text, and save button all mapped to cosmic variables
   - Loading state text color updated

## CSS Mapping Applied Consistently

| Pattern | Cosmic Replacement |
|---------|-------------------|
| `bg-gray-700 text-white rounded focus:ring-blue-500` (inputs) | `bg-[var(--color-cosmic-card)] text-[var(--color-text-main)] rounded-xl border border-[var(--color-cosmic-border)] focus:ring-[var(--color-cosmic-cyan)] focus:border-[var(--color-cosmic-cyan)] transition-colors` |
| `bg-gray-600 text-gray-400 rounded cursor-not-allowed` (disabled) | `bg-[var(--color-nebula)] text-[var(--color-text-muted)] rounded-xl border border-[var(--color-cosmic-border)] cursor-not-allowed` |
| `text-gray-400` (labels/descriptions/empty) | `text-[var(--color-text-muted)]` |
| `text-white` (h3 headings) | `text-[var(--color-text-main)]` |
| `text-lg font-medium text-white mb-4` (section) | `text-lg font-semibold text-[var(--color-text-main)] mb-4` |
| `text-gray-300` (table body) | `text-[var(--color-text-main)]` |
| `border-gray-700` (borders/dividers) | `border-[var(--color-cosmic-border)]` |
| `bg-gray-700 rounded` (cards/skeleton) | `bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl` |
| `bg-gray-800 rounded` (danger panel) | `bg-[var(--color-nebula)] border border-red-500/60 rounded-xl` |
| `bg-gray-600` buttons | `bg-[var(--color-nebula)] border border-[var(--color-cosmic-border)]` |
| `text-blue-400` (active/links) | `text-[var(--color-cosmic-cyan)]` |
| `text-green-400` (success) | `text-[var(--color-cosmic-green)]` |
| `text-red-400` (danger) | `text-[var(--color-cosmic-red)]` |
| `text-yellow-400` (pending) | `text-[var(--color-cosmic-orange)]` |
| `border-red-900` | `border-red-500/60` |
| `border-l-4 border-red-500 pl-4` | `border-l-4 border-red-500/60 pl-4` |
| `focus:ring-red-500` (danger input) | `focus:ring-[var(--color-cosmic-red)] focus:border-[var(--color-cosmic-red)]` |

## Key Decisions

- Profile tab gets glass-panel; Security tab does not (it renders below the tab bar as a separate view in the same container as other tabs that also lack glass-panel wrapping per existing pattern)
- Only the 5 settings page files were modified; CloudflareSettings and S3ProfileSettings were already cosmic-themed in a prior phase
- All CSS variable names follow the established pattern from the cosmic theme: `--color-cosmic-cyan`, `--color-cosmic-border`, `--color-cosmic-card`, `--color-cosmic-green`, `--color-cosmic-red`, `--color-cosmic-orange`, `--color-nebula`, `--color-text-main`, `--color-text-muted`

## Deviations from Plan

- **Intentional cleanup (Rule 3):** In ProfileSettings's API keys section, the API key list card items had `flex items-center justify-between p-3 bg-gray-700 rounded` as the card pattern, matching the plan's specified `p-3 bg-gray-700 rounded` → `p-3 bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)] rounded-xl` mapping — applied consistently.

- **Sub-section heading:** The plan specified adding `<h3>Account Security</h3>` for Section 2. The existing "Change Password" `<h3>` was kept below it to maintain clear hierarchy — the "Account Security" heading serves as the section title, and "Change Password" remains as the form's heading within that section. This matches the existing pattern where Section 1 has no extra wrapper heading but Section 2 benefits from a clear label.

- **Divider elements:** Used `<div className="pt-6 border-t border-[var(--color-cosmic-border)] mt-6" />` (self-closing div) as visual dividers between sections, matching the plan's specified pattern.
