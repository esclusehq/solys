# Phase 27: File Manager Tab - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Add file manager tab to server detail page - transform current single-view page into tabbed interface with file management capability.

</domain>

<decisions>
## Implementation Decisions

### Layout
- **D-01:** Split view layout - tree navigator on left, file list/content on right
- FileManager.jsx component exists and may need update to support split view

### Features
- **D-02:** Full CRUD operations - browse, upload, download, edit, delete, rename

### Backend
- **D-03:** SFTP API - REST API endpoints for file operations (established in codebase)

### Navigation
- **D-04:** Tree view - collapsible folder tree on left for navigation

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend
- `app/src/pages/servers/ServerDetailsPage.jsx` — Current server detail without tabs
- `app/src/components/FileManager.jsx` — Existing file manager component
- `app/src/pages/settings/SettingsPage.jsx` — Tab pattern reference (lines 925-947)
- `app/src/pages/Nodes.jsx` — Tab pattern reference (lines 279-451)

### Backend
- `api/src/presentation/handlers/sftp_handlers.rs` — SFTP API endpoints
- `api/src/presentation/routes/api_routes.rs` — SFTP route registration

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- FileManager.jsx: Existing component - needs update for split view
- Nodes.jsx tabs: Working tab pattern to reference
- SettingsPage.jsx tabs: Another working tab pattern

### Established Patterns
- useState + conditional rendering for tabs
- State management with Zustand for file data

### Integration Points
- ServerDetailsPage.jsx: Add tab navigation
- API endpoints: /servers/:id/files/* already exist

</code_context>

<specifics>
## Specific Ideas

The file manager should:
- Display as a tab alongside logs in ServerDetailsPage
- Use split view with tree on left
- Support all CRUD operations via SFTP API
- Include upload/download capabilities

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 27-file-manager*
*Context gathered: 2026-04-20*