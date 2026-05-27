# Phase 33: Plugins Templates for Hobby and Pro plans - Context

**Gathered:** 2026-05-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Plugin templates - pre-configured plugin lists per game type+variant combination that auto-install when creating a server. Similar to Phase 32 server templates, but for plugins instead of server configuration.

</domain>

<decisions>
## Implementation Decisions

### UI Location
- **D-01:** Server detail page - Plugins tab
- Reuses existing `PluginManager.jsx` component
- User goes to server detail → sees "Plugins" tab with template options

### Installation Flow
- **D-02:** Fully automatic
- When server is created from a template, all configured plugins auto-install silently
- No user confirmation needed - this is the efficiency benefit of templates

### Plugin Template Configuration
- **D-03:** By game type + variant
- Each game type+variant combination has its pre-configured plugin list
- Example: Minecraft Paper could have EssentialsX, Vault, LuckPerms pre-configured
- Follows same pattern as Phase 32 server templates

### Availability by Plan
- **D-04:** All plugins available to all plans (no restrictions on what plugins users can install manually)
- **BUT** plugin templates (auto-install feature) only available for Hobby and Pro plans
- Starter plan gets manual plugin selection only
- This gives value differentiation without restricting plugin availability

</decisions>

<canonical_refs>
## Canonical References

- Phase 32 CONTEXT.md — Server templates pattern
- Phase 15 CONTEXT.md — Plan limits (Starter, Hobby, Pro, Enterprise)
- `app/src/components/PluginManager.jsx` — Existing plugin manager component
- `app/src/hooks/usePlugins.js` — Plugin hooks
- `api/src/application/dto/plugin_dtos.rs` — Plugin DTOs

</canonical_refs>

 {#code_context}
## Existing Code Insights

### Reusable Assets
- `PluginManager.jsx` - Existing component with marketplace + installed tabs
- `usePlugins.js` - Hooks for search, install, uninstall
- Server templates from Phase 32 - pattern to follow

### Integration Points
- Server detail page → Plugins tab
- Server creation flow → Apply plugin template
- Plan tier check for template access

</code_context>

<specifics>
## Specific Details

1. Workflow:
   - User creates server (selects game type + variant)
   - If Hobby/Pro plan: User can also select a "plugin template" (e.g., "Survival Server Pack", "Creative Server Pack")
   - Server created with both server config + plugins auto-installed
   - User can still add/remove plugins manually after creation

2. Example plugin templates:
   - "Survival Pack" (EssentialsX, Vault, LuckPerms, CoreProtect)
   - "Creative Pack" (WorldEdit, VoxelSniper, FastAsyncWorldEdit)
   - "Economy Pack" (Vault, ShopGUIPlus, EconomyShopGUI)
   - Per game type: Minecraft Paper, Fabric, Forge, etc.

</specifics>

<deferred>
## Deferred Ideas

- Custom user-defined plugin templates (future phase)
- Community-shared plugin templates (future phase)
- Per-plan plugin limits (not this phase - all plugins available to all)

</deferred>

---

## ▶ Next Up

**Phase 33: Plugin Templates** — Pre-configured plugin templates for Hobby+ plans

`/clear` then:

`/gsd-plan-phase 33 v1.0`