# Phase 31: Settings - server properties yang bisa di edit seperti form - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Add server properties editing form in the Settings tab - allow users to view and edit server.properties values through a form UI.

</domain>

<decisions>
## Implementation Decisions

### Editable Properties
- **D-01:** Allow editing core server.properties fields:
  - MOTD, maxplayers, gamemode, difficulty, level-seed, default-book, etc.

### UI/Form Layout
- **D-02:** Use form-based layout - each property is a form field
- Standard React form UI with labels and input fields

### Save Mechanism
- **D-03:** Replace entire file - validate all fields, then write complete server.properties
- Simpler and safer approach

### Field Validation
- **D-04:** Real-time validation
  - gamemode: survival/creative/adventure/spectator
  - difficulty: peaceful/easy/normal/hard
  - maxplayers: 1-1000
  - Real-time feedback as user types

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend
- `app/src/pages/servers/ServerDetailsPage.jsx` — Server detail page with Settings tab
- Lines ~31-37: tabs array includes Settings tab

### Backend
- `api/src/presentation/handlers/terminal_handlers.rs` — Read properties via exec cat
- Terminal handlers for file write operations

</canonical_refs>

\code_context

### Reusable Assets
- Settings tab already exists in ServerDetailsPage
- Terminal handlers for container exec commands

### Established Patterns
- Form state + validation from other features

### Integration Points
- Settings tab: Add property form component
- Backend: Add API endpoint to write server.properties

</code_context>

<specifics>
## Specific Ideas

Server properties form should:
- Display current server.properties values
- Provide form inputs for each editable property
- Real-time validation (gamemode options, difficulty, etc.)
- Save button to write changes to container
- Show success/error feedback

</specifics>

<deferred>
## Deferred Ideas

None

---

*Phase: 31-settings-server-properties-yang-bisa-di-edit-seperti-form*
*Context gathered: 2026-04-23*