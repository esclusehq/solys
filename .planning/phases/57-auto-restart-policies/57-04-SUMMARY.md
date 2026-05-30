---
phase: 57-auto-restart-policies
plan: 04
subsystem: frontend
tags: server-details, settings, restart-policy, global-defaults
requires:
  - 57-02 (Backend API — DTOs, handlers, global defaults endpoints)
provides: "Restart Policy UI in Server Settings tab + global restart defaults admin tab in Settings page"
affects: []
tech-stack:
  added: []
  patterns:
    - "Restart Policy section follows Sleep & Wake section pattern exactly"
    - "State management, load, save handler, toast pattern identical to sleep config"
    - "Global defaults tab follows admin-only settings patern (Cloudflare DNS / Storage)"
    - "Number inputs clamped with Math.min/Math.max for input validation"
key-files:
  created: []
  modified:
    - app/src/pages/ServerDetails.jsx
    - app/src/pages/settings/SettingsPage.jsx
key-decisions:
  - "Restart Policy section placed after Sleep & Wake in Settings tab"
  - "Only health_check_timeout_seconds exposed in UI (last_restart_at/reason are monitoring-service-only)"
  - "Restart History always visible (even when auto-restart is off)"
  - "Global defaults use existing Settings page (not a separate page)"
  - "Number inputs clamped to valid ranges (attempts 1-20, cooldown 30-3600, timeout 1-60)"
  - "Restart reason displayed in orange for visual distinction"
duration: 8 min
completed: 2026-05-30
---

# Phase 57: Auto Restart Policies — Plan 04 Summary

**Restart Policy configuration UI in Server Details Settings tab + global restart defaults admin tab in Settings page**

Added a Restart Policy section to the Server Settings tab (after Sleep & Wake) with auto-restart toggle, max attempts (1-20), restart cooldown (30-3600s), health check timeout (1-60s) inputs, restart history display (count, last time, reason), and save button. Added a Global Restart Defaults tab in the Settings page for admin users with max_attempts and cooldown defaults fetch/save.

## Commits

**Tasks 1+2: Restart Policy UI + global defaults tab** — `7c45708`
- 6 restart policy state variables, load from server data, save handler
- Restart Policy section JSX (toggle, inputs, history display, save button)
- Restart defaults tab in SettingsPage with fetchApi GET/PUT
- Number input validation with Math.min/Math.max clamping
- fetchApi import + admin-only tab rendering

## Files Modified

- `app/src/pages/ServerDetails.jsx` — 360 insertions, 8 deletions
  - State variables: autoRestart, maxRestartAttempts, restartCooldown, healthCheckTimeout, restartSaving, restartToast
  - Load: setAutoRestart, setMaxRestartAttempts, setRestartCooldown, setHealthCheckTimeout
  - Save handler: handleSaveRestartConfig
  - JSX: Restart Policy section with toggle, 3 conditional inputs, restart history display, save button

- `app/src/pages/settings/SettingsPage.jsx` — Added import, state, fetch/load, render function, route
  - fetchApi import
  - 3 global restart state variables
  - loadRestartDefaults + activeTab-triggered useEffect
  - renderRestartDefaultsTab with max_attempts and cooldown inputs
  - Tab registration and content rendering

## Verification

- ✅ Vite build passes (dist output generated)
- ✅ ServerDetails.jsx has Restart Policy section heading
- ✅ All 6 state variables defined
- ✅ handleSaveRestartConfig calls updateServer with all 4 fields
- ✅ Restart History displays server.restart_count, server.last_restart_at, server.last_restart_reason
- ✅ SettingsPage.jsx has restart-defaults tab in admin-only section
- ✅ loadRestartDefaults fetches from /api/v1/settings/restart-defaults
- ✅ Save button PUTs to /api/v1/settings/restart-defaults

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- ✅ SUMMARY.md exists at expected path
- ✅ Both files modified correctly
- ✅ Vite build passes
- ✅ All required patterns verified by grep
