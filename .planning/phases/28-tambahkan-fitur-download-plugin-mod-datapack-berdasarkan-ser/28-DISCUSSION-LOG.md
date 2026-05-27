# Phase 28: Download Plugin/Mod/Datapack - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 28-download-plugins
**Areas discussed:** Plugin Tab Integration, Server Type Detection, Game Server Support

---

## Plugin Tab Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Add to ServerDetailsPage | Add Plugins tab alongside Files and Logs - reuse existing PluginManager component | ✓ |
| Use old ServerDetails.jsx | Use old ServerDetails.jsx instead which already has plugin handling | |

**User's choice:** Add to ServerDetailsPage
**Notes:** Reuse existing PluginManager.jsx component, integrate with current page

---

## Server Type Detection

| Option | Description | Selected |
|--------|-------------|----------|
| mc_loader field | Use mc_loader field - PAPER, SPIGOT, BUKKIT, PURPUR, FORGE, FABRIC, NEOFORGE show Plugins; VANILLA shows Datapacks | ✓ |
| game_type field | Use game_type field (minecraft, palworld) | |

**User's choice:** mc_loader field
**Notes:** Based on server loader type - modded loaders get plugins, vanilla gets datapacks

---

## Game Server Support

| Option | Description | Selected |
|--------|-------------|----------|
| Minecraft only | Minecraft plugins + datapacks, no other games yet | ✓ |
| Minecraft + Palworld | Minecraft + other game servers like Palworld mods | |

**User's choice:** Minecraft only
**Notes:** Focus on Minecraft plugins/datapacks for this phase

---

## Agent's Discretion

Plugin tab integration approach is straightforward - reuse existing component.

## Deferred Ideas

- Palworld mod support — mentioned as future phase