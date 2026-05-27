# Phase 28: Download Plugin/Mod/Datapack - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Add plugin/mod/datapack download feature to server detail page - integrate PluginManager component with server-type-aware tab.

</domain>

<decisions>
## Implementation Decisions

### Plugin Tab Integration
- **D-01:** Add Plugins tab to ServerDetailsPage.jsx alongside Overview, Files, Logs
- Reuse existing PluginManager.jsx component
- Backend API already exists in plugin_handlers.rs

### Server Type Detection
- **D-02:** Use `mc_loader` field to detect server type
- If mc_loader in [PAPER, SPIGOT, BUKKIT, PURPUR, FORGE, FABRIC, NEOFORGE] → show "Plugins" tab
- If mc_loader is VANILLA or null → show "Datapacks" tab

### Game Server Support
- **D-03:** Minecraft only for this phase
- Plugins for modded loaders (Forge, Fabric, etc.)
- Datapacks for vanilla servers

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend
- `app/src/pages/servers/ServerDetailsPage.jsx` — Current server detail page with tabs
- `app/src/components/PluginManager.jsx` — Existing plugin manager component
- `app/src/pages/ServerDetails.jsx` — Old server page with plugin tab pattern (lines 236-241)

### Backend
- `api/src/presentation/handlers/plugin_handlers.rs` — Plugin API endpoints
- `api/src/presentation/routes/api_routes.rs` — Plugin route registration

</canonical_refs>

\code_context

### Reusable Assets
- PluginManager.jsx: Existing component - just needs integration
- ServerDetailsPage.jsx: Just updated with tabs in Phase 27

### Established Patterns
- Tab state + conditional rendering (from Phase 27)
- mc_loader-based conditional (from old ServerDetails.jsx)

### Integration Points
- ServerDetailsPage.jsx: Add Plugins tab to tabs array
- Conditional rendering based on server.mc_loader

</code_context>

<specifics>
## Specific Ideas

The plugins tab should:
- Show "Plugins" label for modded loaders (Paper, Spigot, Forge, Fabric, etc.)
- Show "Datapacks" label for vanilla servers
- Use existing PluginManager component that interfaces with /api/plugins/* endpoints

</specifics>

<deferred>
## Deferred Ideas

- Palworld mod support — future phase

---

*Phase: 28-download-plugins*
*Context gathered: 2026-04-20*