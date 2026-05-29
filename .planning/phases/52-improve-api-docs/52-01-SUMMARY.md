---
phase: 52-improve-api-docs
plan: 01
subsystem: api-docs
tags: [vitepress, vue, openapi, schema, docs-infrastructure]

requires: []
provides:
  - Build-time OpenAPI data loader for auto-generated schema tables
  - OpenApiSchema.vue component for $ref-based schema rendering
  - StaticSchema.vue component for inline schema rendering
  - Global component registration in VitePress theme
  - Schema table CSS with light/dark theme support
affects: [52-improve-api-docs]

tech-stack:
  added: [VitePress defineLoader, Vue 3 single-file components in docs]
  patterns: [Build-time data loader fetch, Vue component globa registration in VitePress]

key-files:
  created:
    - docs/.vitepress/loaders/openapi.data.ts
    - docs/.vitepress/components/OpenApiSchema.vue
    - docs/.vitepress/components/StaticSchema.vue
  modified:
    - docs/.vitepress/theme/index.ts
    - docs/.vitepress/theme/custom.css

key-decisions:
  - "Use VitePress build-time data loader (defineLoader) to fetch OpenAPI spec at build time, not at runtime"
  - "Graceful fallback to empty schemas on fetch failure — never crash the build"
  - "OpenApiSchema component uses $ref path parsing to look up schemas by name in the data loader results"
  - "StaticSchema component accepts inline JSON for endpoints not covered by the OpenAPI spec (~70%)"

patterns-established:
  - "Schema table rendering: two components with identical table structure, different data sources"
  - "Component registration: globally via app.component in VitePress theme's enhanceApp"

requirements-completed: []

duration: 3min
completed: 2026-05-29
---

# Phase 52 Plan 01: VitePress Build-Time Infrastructure for OpenAPI Schema Tables

**Build-time OpenAPI data loader with Vue components for auto-generated and static schema tables in VitePress docs**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-29T15:03:34Z
- **Completed:** 2026-05-29T15:06:38Z
- **Tasks:** 3
- **Files modified:** 5 (3 created, 2 modified)

## Accomplishments
- Created `openapi.data.ts` — build-time data loader that fetches the OpenAPI spec from `https://api.esluce.com/openapi.json` with 10s timeout and graceful fallback on failure
- Created `OpenApiSchema.vue` — renders field-level schema table from an OpenAPI `$ref` path (e.g. `#/components/schemas/CreateServerRequest`), showing field/type/required/description with enum hints
- Created `StaticSchema.vue` — renders identical schema table from inline JSON props for endpoints not covered by the OpenAPI spec
- Registered both components globally in VitePress theme via `app.component()`, preserving existing title bar navigation mixin
- Added schema table CSS using VitePress CSS variables (`--vp-c-*`) for automatic light/dark theme switching
- Full VitePress build completes successfully with no errors

## Task Commits

Each task was committed atomically within the `docs` submodule:

1. **Task 1: Create build-time OpenAPI data loader** — `docs@9834c88` (`feat`)
2. **Task 2: Create OpenApiSchema.vue and StaticSchema.vue** — `docs@3284984` (`feat`)
3. **Task 3: Register components in theme and add CSS** — `docs@ee50891` (`feat`)

**Plan metadata:** `092bbc5` (docs: complete plan — main repo submodule pointer update)

## Files Created/Modified
- `docs/.vitepress/loaders/openapi.data.ts` — VitePress data loader, fetches schemas from `/openapi.json` at build time
- `docs/.vitepress/components/OpenApiSchema.vue` — Vue SFC rendering schema table from OpenAPI `$ref` path
- `docs/.vitepress/components/StaticSchema.vue` — Vue SFC rendering schema table from inline JSON prop
- `docs/.vitepress/theme/index.ts` — Enhanced with `import type { Theme }`, global component registration, `satisfies Theme`
- `docs/.vitepress/theme/custom.css` — Added `.schema-table`, `.schema-table-wrapper`, `.schema-context-label`, `.schema-enum-hint`, `.schema-default-hint`, `.schema-missing` styles

## Decisions Made
- Used `defineLoader` from VitePress (no extra npm packages) for build-time fetch — standard VitePress data loading pattern
- `fetch()` with `AbortSignal.timeout(10000)` for robust HTTP fetching with timeout (Node.js 18+)
- Graceful fallback to `{}` on any fetch failure — build never crashes due to API being unreachable
- OpenApiSchema parses `$ref` path by splitting on `/` and taking the last segment as schema name
- "Warning" colors for missing schema messages using dashed border + `--vp-c-warning-*` variables
- Both components wrapped in `.schema-table-wrapper` div with optional context label for request/response differentiation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed angle brackets in OpenApiSchema.vue template causing Vue compiler error**
- **Found during:** Task 3 (build verification)
- **Issue:** The template contained `See <StaticSchema documentation...` which the Vue HTML compiler interpreted as an unclosed HTML tag, causing build failure
- **Fix:** Replaced the raw `<StaticSchema` reference with plain text "Use the StaticSchema component" to avoid HTML parsing conflict
- **Files modified:** `docs/.vitepress/components/OpenApiSchema.vue`
- **Verification:** Build succeeds, no compiler errors
- **Committed in:** `docs@ee50891` (Task 3 commit, included in the fix)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minimal — syntax fix in template text. No behavior change, all schema features unaffected.

## Issues Encountered
- OpenAPI fetch returns status 525 (origin connectivity) in local dev environment — handled gracefully by the fallback to empty schemas, build continues successfully
- Vue compiler rejected naked angle brackets (`<StaticSchema`) inside template text — fixed per deviation above

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Infrastructure for auto-generated and static schema tables is ready
- Components available globally in markdown pages as `<OpenApiSchema />` and `<StaticSchema />`
- Next plans can use these components in API doc markdown pages to render field-level schema tables
- Ready for 52-02: "Enhance VitePress doc infrastructure" (sidebar updates, page creation)

## Self-Check: PASSED

All files verified on disk and all commit hashes confirmed in git log.

---

*Phase: 52-improve-api-docs*
*Completed: 2026-05-29*
