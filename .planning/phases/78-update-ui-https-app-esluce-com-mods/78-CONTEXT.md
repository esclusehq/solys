# Phase 78: update UI https://app.esluce.com/mods - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Improve the Mod Browser page at `/mods` (ModBrowserPage) with enriched result cards, working Add-to-server flow, version detail modal, enhanced filters, and page-number pagination.

</domain>

<decisions>
## Implementation Decisions

### Result Cards & Details
- **D-01:** Add author, latest version number, and categories/tags to each card.
- **D-02:** Keep existing: icon, title, description, download count, Versions + Add buttons.

### Add Mod Behavior
- **D-03:** 'Add' opens a modal to select a target server, then installs the mod to that server's mods folder.
- **D-04:** User picks the mod version in the install modal (not auto-selected).
- **D-05:** Needs a new API endpoint or use existing server file management to copy mod files.

### View Versions
- **D-06:** 'Versions' opens a modal listing available versions for that mod.
- **D-07:** Each version in the modal has an 'Install' button that flows into the server-picker + install flow.

### Filters & Pagination
- **D-08:** Add category filter alongside existing version + loader filters.
- **D-09:** Fetch available game versions dynamically from Modrinth API instead of hardcoded list.
- **D-10:** Replace 'Load More' with page-number pagination.

### the agent's Discretion
- Modal design for server picker + version picker
- Category filter options and placement
- Page number component design
- Dynamic version fetching strategy (on mount or on filter interaction)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Mod Browser page
- `app/src/pages/templates/ModBrowserPage.jsx` — target page (search, filters, results, pagination)
- `app/src/components/ModSearchResult.jsx` — mod result card component
- `app/src/api/templatesApi.js` — modsApi (search, getVersions, etc.)

### Prior phases
- `.planning/phases/75-update-ui-https-app-esluce-com-servers/75-CONTEXT.md` — modal and filter patterns
- `.planning/phases/77-update-ui-https-app-esluce-com-templates/77-CONTEXT.md` — adjacent templates page patterns

### API
- Modrinth API docs for dynamic version listing and category fetching

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ModSearchResult.jsx` — existing card, will be enriched
- Server-picker modals exist in `ServerManagerPage.jsx` (CreateServerModal pattern)
- Debounced search pattern already in `ModBrowserPage.jsx`

### Established Patterns
- Glass-panel cards for search results
- Modal overlays for confirmations and details
- Filter bar pattern (search + dropdowns)

### Integration Points
- `/mods` route in App.jsx — ModBrowserPage
- Server mod installation likely uses existing file management or needs new API

</code_context>

<specifics>
## Specific Ideas

- Add modal: select server from a dropdown/list, pick mod version from a list, confirm to install
- Version modal: list each version with name, version number, release date, loaders, and Install button
- Category filter values from Modrinth (e.g., bukkit, mod, datapack, resourcepack, shader)
- Page numbers at bottom with prev/next buttons

</specifics>

<deferred>
## Deferred Ideas

- Mod collections/wishlist — future phase
- Mod detail detail page (separate route) — future phase

</deferred>

---

*Phase: 78-update-ui-https-app-esluce-com-mods*
*Context gathered: 2026-06-14*
