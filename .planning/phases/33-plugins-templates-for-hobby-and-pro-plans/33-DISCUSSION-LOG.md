# Phase 33: Plugins Templates for Hobby and Pro plans - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-04
**Phase:** 33-plugins-templates-for-hobby-and-pro-plans
**Areas discussed:** UI location for plugin templates, Plugin installation flow, Plugin template configuration, Plugin availability by plan

---

## UI Location for Plugin Templates

| Option | Description | Selected |
|--------|-------------|----------|
| Create server wizard | In create server flow, after selecting game type and variant | |
| Settings page | Separate UI page for managing plugin templates | |
| Server detail page | Inside server detail page, Plugins tab | ✓ |
| Hybrid (both) | Both create wizard and server detail - context-aware | |

**User's choice:** Server detail page

---

## Plugin Installation Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Automatic with confirmation | Pre-select common plugins per template, user confirms during server creation | |
| Manual (recommended) | Template defines recommended plugins, but user must manually install | |
| Fully automatic | Auto-install all template plugins silently at server creation | ✓ |
| Per-plugin prompts | Prompt for each plugin during installation | |

**User's choice:** Fully automatic

---

## Plugin Template Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Backend-based (Modrinth) | Backend plugin for plugins | |
| Simple plugin list | List of plugin names/versions | |
| By game type + variant | Define per game type + variant | ✓ |
| User custom templates | Allow custom plugin lists | |

**User's choice:** By game type + variant

---

## Plugin Availability by Plan

| Option | Description | Selected |
|--------|-------------|----------|
| No restrictions | All plugins available to all plans | ✓ (manual) |
| Pro+ only | Pro/Enterprise only | |
| Hobby = limited | Hobby limited to N plugins | |
| By category | By plugin category | |

**User's choice:** All plugins available to all plans BUT template auto-install only Hobby/Pro