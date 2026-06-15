---
phase: 78-update-ui-https-app-esluce-com-mods
plan: 03
type: execute
subsystem: mods
tags: [mods, install, modal, ui, server-picker]
dependency:
  requires: [78-02]
  provides: [78-04]
  affects: [ModBrowserPage, templatesApi]
tech-stack:
  added: []
  patterns: [useCallback, useUIStore, useServerStore, inline-modal, api-post]
key-files:
  created: []
  modified:
    - app/src/api/templatesApi.js
    - app/src/pages/templates/ModBrowserPage.jsx
decisions:
  - "closeInstallModal defined before executeInstall to avoid temporal dead zone with const useCallback"
  - "Install modal inlined in ModBrowserPage per plan directive — no new component file"
metrics:
  duration: "~8 min"
  completed: "2026-06-15"
  tasks: 2
  files-changed: 2
---

# Phase 78 Plan 03: Add-to-Server Install Modal — Summary

Add modsApi.install method and wire a full Add-to-Server modal in ModBrowserPage with version picker, server picker, success/error toasts, and Escape/backdrop dismiss.

## One-liner

Add-to-Server install modal in ModBrowserPage — version picker (dynamic dropdown or pre-selected read-only), server picker from useServerStore, install via modsApi.install, toast feedback, Escape/backdrop dismiss.

## Tasks

### Task 1: Add modsApi.install method to templatesApi.js

**Status:** Done

Added `install: (serverId, payload) => api.post(\`/servers/${serverId}/plugins/install\`, payload)` to the `modsApi` export object. The method POSTs the `{ project_id, version_id }` payload to the backend endpoint.

### Task 2: Add install modal state, handlers, and JSX to ModBrowserPage.jsx

**Status:** Done

Applied 10 surgical edits:

1. **Imports:** Added `useCallback` to React import, added `useUIStore` and `useServerStore` imports
2. **Install modal state:** `showInstallModal`, `installMod`, `installVersion`, `installVersionOptions`, `installVersionLoading`, `selectedVersionId`, `selectedServerId`, `installing`
3. **Store hooks:** `addToast` from `useUIStore`, `servers` + `fetchServers` from `useServerStore`
4. **handleInstallFromVersion replacement:** Opens install modal with version pre-selected (read-only display with font-mono version_number)
5. **handleAddFromCard:** Opens modal, fetches versions via `modsApi.getVersions`, populates dropdown
6. **onAdd wiring:** `onAdd={() => {}}` replaced with `onAdd={handleAddFromCard}`
7. **executeInstall:** Calls `modsApi.install`, shows success/error toasts
8. **closeInstallModal:** Resets all install state
9. **Escape key useEffect:** `keydown` listener for Escape key dismiss
10. **Install modal JSX:** Version picker (dropdown/read-only/loading/empty), server picker (dropdown/empty), Cancel/Install buttons, spinner during installation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Code ordering] Reordered closeInstallModal before executeInstall**

- **Found during:** Task 2
- **Issue:** The plan specified adding `executeInstall` before `closeInstallModal`, but `executeInstall` calls `closeInstallModal()` — JavaScript `const` temporal dead zone would cause a ReferenceError at runtime
- **Fix:** Defined `closeInstallModal` before `executeInstall` in the component
- **Files modified:** `app/src/pages/templates/ModBrowserPage.jsx`
- **Commit:** N/A (no git available)

## Verification Results

All 10 verification checks pass:

| # | Check | Result |
|---|-------|--------|
| 1 | `npm run build` from `app/` | ✅ exit 0 (14.53s) |
| 2 | `modsApi.install` method exists | ✅ 1 match with `/servers/` |
| 3 | Install modal state (6+ vars) | ✅ 8 state vars present |
| 4 | Handlers (handleAddFromCard, executeInstall, closeInstallModal) | ✅ 3+ references |
| 5 | Store imports (useUIStore, useServerStore) | ✅ 2 imports + 2 hook calls |
| 6 | Toast messages ("installed to", "Failed to install") | ✅ 2 matches |
| 7 | Empty states (servers, versions) | ✅ 2 matches |
| 8 | useCallback usage | ✅ 5 references |
| 9 | No `alert()` calls | ✅ 0 matches |
| 10 | Escape key handler | ✅ 1 match |

## Threat Surface Scan

No new threat flags introduced. The install modal sends only user-selected `version_id` and `server_id` from controlled dropdowns, matching the plan's threat model (T-78-07 mitigated by backend ownership validation, T-78-08 mitigated by `installing` state preventing double-submit).

## Self-Check: PASSED

- `app/src/api/templatesApi.js` exists (17 lines) — contains `install:` with `api.post` and `/servers/`
- `app/src/pages/templates/ModBrowserPage.jsx` exists (521 lines, exceeds 450-line minimum) — contains all required state vars, handlers, store hooks, toast messages, empty states, and Escape key handler
