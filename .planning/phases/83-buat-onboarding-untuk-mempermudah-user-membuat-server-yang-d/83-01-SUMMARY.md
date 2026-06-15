---
phase: 83-buat-onboarding-untuk-mempermudah-user-membuat-server-yang-d
plan: 01
subsystem: ui
tags: onboarding, wizard, react, server-creation, 4-step, modal

# Dependency graph
requires:
  - phase: 83-buat-onboarding-untuk-mempermudah-user-membuat-server-yang-d
    provides: UI spec (UI-SPEC.md), pattern map (PATTERNS.md), existing CreateServerModal constants
provides:
  - Shared constants file (MINECRAFT_VERSIONS, RAM_OPTIONS, etc.) extracted from CreateServerModal
  - 4-step ServerOnboardingWizard component for first-time server creation
  - DashboardPage "Create your first server" button opens wizard instead of navigating to /servers
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - useRef + useState for multi-step wizard data persistence (no Zustand for wizard state)
    - renderStepper helper to eliminate repetition in resource stepper controls
    - Constants extraction pattern for sharing data between CreateServerModal and ServerOnboardingWizard

key-files:
  created:
    - app/src/features/server/constants.js — Shared MINECRAFT_VERSIONS, RAM_OPTIONS, MAX_RAM_OPTIONS, PLAYER_OPTIONS, GAME_TYPE_LABELS
    - app/src/features/server/ServerOnboardingWizard.jsx — 4-step onboarding wizard (496 lines)
  modified:
    - app/src/features/server/CreateServerModal.jsx — Imports from constants.js instead of inline definitions (695 lines, -120 lines)
    - app/src/pages/dashboard/DashboardPage.jsx — Wired ServerOnboardingWizard, replaced navigate('/servers') with showWizard state

key-decisions:
  - "Constants extracted to shared file to avoid duplication between wizard and legacy CreateServerModal"
  - "useRef for wizard data persistence (prevents data loss on step navigation), not Zustand"
  - "renderStepper helper reduces 3 repetitive resource stepper blocks to 3 function calls"
  - "Single-file component (~496 lines) keeps all 4 steps together rather than separate sub-components"
  - "serversApi.create() reused (not duplicated) — both components call the same shared API function"
  - "Version dropdown hidden for Bedrock/PocketMine; port 10000-30000 validation with random suggestion"

patterns-established:
  - "Step wizard uses useRef + tick state for persistence without Zustand dependency"
  - "Constants pattern: exported shared arrays from constants.js, imported by multiple components"
  - "Resource steppers use a shared renderStepper function (field, label, limits, step) for DRY controls"

requirements-completed:
  - ONB-01
  - ONB-02
  - ONB-03
  - ONB-04
  - ONB-05
  - ONB-06

# Metrics
duration: 6 min
completed: 2026-06-15
---

# Phase 83: Buat Onboarding untuk Mempermudah User Membuat Server — Plan 1 Summary

**4-step server creation wizard (ServerOnboardingWizard) replacing navigate('/servers') redirect, with shared constants extracted for DRY code**

## Performance

- **Duration:** 6 min
- **Started:** 2026-06-15T21:48:55Z
- **Completed:** 2026-06-15T21:54:44Z
- **Tasks:** 3 of 3
- **Files modified:** 4

## Accomplishments

- Extracted 6 shared constant arrays (MINECRAFT_VERSIONS, groupedVersions, RAM_OPTIONS, MAX_RAM_OPTIONS, PLAYER_OPTIONS, GAME_TYPE_LABELS) into `app/src/features/server/constants.js`, removing 120 lines of inline definitions from CreateServerModal
- Created ServerOnboardingWizard (496 lines) with 4-step modal: Game Type → Plan & Resources → Config → Review & Deploy
- Step 0: 3 game type cards (Java/Bedrock/PocketMine) with icons and descriptions, Java pre-selected
- Step 1: Plan cards (Free/Hobby/Pro) with feature lists, resource steppers (RAM/CPU/Disk) constrained to plan limits
- Step 2: Server name (max 64 chars), version dropdown (Java only), port (10000-30000 with random suggestion), template (optional)
- Step 3: Read-only review summary with "Deploy Server" button — calls serversApi.create(), on success redirects to /servers/{id}
- Wired ServerOnboardingWizard into DashboardPage: "Create your first server" button opens wizard as modal instead of navigating to /servers
- All 3 commits verified: constants extracted, wizard created, dashboard wired

## Task Commits

Each task was committed atomically in the `app/` sub-repo:

1. **Task 1: Extract shared constants** — `3e1df58` (feat)
2. **Task 2: Create ServerOnboardingWizard** — `c8234e6` (feat)
3. **Task 3: Wire into DashboardPage** — `6859e45` (feat)

## Files Created/Modified

- `app/src/features/server/constants.js` (NEW) — Shared MINECRAFT_VERSIONS (76 entries), groupedVersions, RAM_OPTIONS, MAX_RAM_OPTIONS, PLAYER_OPTIONS, GAME_TYPE_LABELS (118 lines)
- `app/src/features/server/ServerOnboardingWizard.jsx` (NEW) — 4-step wizard component with modal overlay, progress bar, step navigation, port validation, server creation via serversApi.create() (496 lines)
- `app/src/features/server/CreateServerModal.jsx` (MODIFIED) — Imports from constants.js, all inline constant definitions removed (695 lines, -120 lines)
- `app/src/pages/dashboard/DashboardPage.jsx` (MODIFIED) — Added import, showWizard state, wizard render; replaced navigate('/servers') with setShowWizard(true) (488 lines)

## Decisions Made

- **Constants extracted to shared file:** MINECRAFT_VERSIONS and related constants moved to constants.js so both CreateServerModal and ServerOnboardingWizard import from the same source — prevents future drift
- **useRef for wizard state:** wizardData persisted in useRef (not Zustand) per RESEARCH.md recommendation — prevents data loss on step re-renders
- **renderStepper helper:** 3 resource controls (RAM/CPU/Disk) refactored to a shared renderStepper function, ~150 lines saved vs naive repetition
- **Single-file component:** All 4 steps in one file (~496 lines) rather than separate sub-components — keeps the codebase lean for a simple wizard
- **serversApi.create() reused:** Both components call the same shared serversApi.create() function — no API logic duplication
- **Version dropdown conditional:** Hidden for Bedrock/PocketMine per existing pattern (CreateServerModal behavior)
- **Port validation:** 10000-30000 range with randomized port suggestion on mount, matching existing CreateServerModal validation

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Threat Flags

None — all security surface is within the plan's threat model (T-83-01 port validation, T-83-02 name maxLength). No new endpoints, auth paths, or schema changes introduced.

## Known Stubs

No stubs found — all functionality is fully wired. The ServerOnboardingWizard is production-ready with real API calls.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- ServerOnboardingWizard fully built and wired into DashboardPage
- First-time users clicking "Create your first server" see the 4-step wizard instead of being redirected to /servers
- Existing CreateServerModal unchanged and available on /servers page for experienced users
- All verification checks pass: build succeeds, all acceptance criteria met
- Phase 83 is complete (1 of 1 plans executed)

## Self-Check: PASSED

- ✅ All 5 files exist at expected paths (constants.js, ServerOnboardingWizard.jsx, CreateServerModal.jsx, DashboardPage.jsx, SUMMARY.md)
- ✅ 3 app commits found (3e1df58, c8234e6, 6859e45) with `feat(83-01):` prefix
- ✅ 1 parent repo commit (722163d) for SUMMARY.md with `docs(83-01):` prefix
- ✅ `npm run build` succeeds (2529 modules transformed, built in 9.54s)
- ✅ No inline `const MINECRAFT_VERSIONS` remains in CreateServerModal (grep returns 0)
- ✅ `create your first server` button now calls `setShowWizard(true)` instead of `navigate('/servers')`
- ✅ WelcomeModal import + render intact (2 occurrences)
- ✅ `serversApi.create()` used correctly in both components (shared API, not duplicated logic)

---

*Phase: 83-buat-onboarding-untuk-mempermudah-user-membuat-server-yang-d*
*Completed: 2026-06-15*
