# Phase 77: update UI https://app.esluce.com/templates - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Improve the Templates section UI at `/templates` and sub-pages: Template Library (list), Template Create/Edit (form). Focus on layout reorganization, enriched card info, improved filters, and form styling refinement. No new API endpoints.

</domain>

<decisions>
## Implementation Decisions

### Template Library Layout
- **D-01:** Replace stacked sections (Featured + Your Templates) with filter tabs: "Featured" / "Yours" / "All".
- **D-02:** Cards show additional info: version tag, last updated date, usage count (servers using this template).
- **D-03:** Add sorting (name, updated, popular) and category filter alongside existing search + game type filter.
- **D-04:** Filter/sort preferences not persisted (session-only).

### Create/Edit Form
- **D-05:** Keep existing 2-section layout (Basic Info + Configuration).
- **D-06:** Refine styling and form field presentation (cosmic theme consistency).
- **D-07:** No new form sections or fields.

### Mod Browser
- **D-08:** No changes to Mod Browser page. Scope limited to Library and Create/Edit form.

### the agent's Discretion
- Exact tab design for Featured/Yours/All (pill buttons or tab bar)
- Sort options and category filter UI placement
- Version/updated/count layout within cards
- Form styling details

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Template pages
- `app/src/pages/templates/TemplateLibraryPage.jsx` — target page (card grid, search/filter, sections)
- `app/src/pages/templates/TemplateCreatePage.jsx` — create/edit form (Basic Info + Configuration sections)
- `app/src/pages/templates/ModBrowserPage.jsx` — mod browser (unchanged)
- `app/src/components/TemplateCard.jsx` — template card component (card layout, fields, badges, actions)

### Prior phase patterns
- `.planning/phases/75-update-ui-https-app-esluce-com-servers/75-CONTEXT.md` — filter/sort patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TemplateCard.jsx` — existing card component, will need enrichment for version/updated/count
- Search + game type filter pattern already exists in `TemplateLibraryPage.jsx`
- Phase 75's filter/sort patterns

### Established Patterns
- Cosmic theme styling throughout
- Glass-panel cards for template display

### Integration Points
- `/templates` route in App.jsx — TemplateLibraryPage
- `/templates/create` — TemplateCreatePage
- `/templates/:id/edit` — TemplateCreatePage (edit mode)

</code_context>

<specifics>
## Specific Ideas

- Filter tabs: "Featured" shows built-in templates, "Yours" shows user-created, "All" shows both merged
- Sort options: Name (A-Z), Last Updated (newest), Most Used
- Category filter derived from available categories in the template data
- Version tag could be from template.properties.version or template.created_at

</specifics>

<deferred>
## Deferred Ideas

- Quick-install from template library (one-click deploy) — future phase
- Mod Browser UI improvements — future phase
- Template icon/screenshot upload — future phase

</deferred>

---

*Phase: 77-update-ui-https-app-esluce-com-templates*
*Context gathered: 2026-06-14*
