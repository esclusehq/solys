# Phase 87: Selesaikan fitur 'Create server from template' secara menyeluruh - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-17
**Phase:** 87-selesaikan-fitur-create-server-from-template-secara-menyeluruh
**Areas discussed:** Entry Point, Config Overrides, Resource Enforcement, Plugin/Modpack Auto-Install

---

## Entry Point — how to start creating from template

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated template detail page | Navigate to /templates/:id — full template info + Create button | ✓ |
| Inline modal from card | Click → modal opens with template pre-selected | |
| Route into CreateServerModal | Navigate to /servers?create with template pre-filled | |
| Onboarding Wizard route | Route into ServerOnboardingWizard with template base config | |

**User's choice:** Dedicated template detail page
**Notes:** Show config preview + dependency list on the detail page before the Create button

---

## Config Overrides — what users can change

| Option | Description | Selected |
|--------|-------------|----------|
| Full control | Name, RAM, version, port, max players pre-filled from template | |
| Minimal overrides | Name and RAM only. Version/port locked | |
| Mid-level | Name, RAM, version. Port locked. Player limit auto-calculated | |
| Custom (freeform) | User typed specific fields | ✓ |

**User's choice:** Name, DISK, choose agent/node, Online/Offline mode, World seed, player limit (freeform response)
**Notes:** Template defaults pre-fill all fields. User can override what they want.

---

## Resource Enforcement — Plan/Node Compatibility

| Option | Description | Selected |
|--------|-------------|----------|
| Block with explanation | Show message + link to billing to upgrade | |
| Allow with warning | Show warning, let them create with reduced resources | ✓ |
| Auto-suggest upgrade | One-click upgrade then proceed | |

**User's choice:** Allow with warning (freeform response)
**Notes:** Warning should say "This template requires X GB RAM. Your computer/VPS/homelab (letak agent nya) can't handle it." References agent node capacity, not billing plan.

---

## Plugin/Modpack Auto-Install

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-install during creation | Plugins install as part of pipeline. No extra step. | |
| Show preview, then install | Before creating, show list of what will be installed. User confirms. | ✓ |
| Manual post-creation install | Create server first, install mods later from Mod Browser | |

**User's choice:** Show preview, then install
**Notes:** Dependencies shown with versions and game version compatibility. User confirms before deployment proceeds. Installation happens during creation pipeline.

---

## the agent's Discretion

- Exact visual layout of the template detail page (config preview, dependency list presentation)
- UI design of the creation modal (step form vs single scrollable form)
- Progress indicators during deployment with dependency installation
- Warning message wording and visual treatment for resource mismatches
- How template version and usage_count are displayed on detail page

## Deferred Ideas

None
