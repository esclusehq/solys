---
phase: 72
slug: menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te
status: draft
shadcn_initialized: false
preset: none
created: 2026-06-12
---

# Phase 72 — UI Design Contract

> Visual and interaction contract for adding Minecraft Bedrock Edition as a first-class server type.
> Pre-populated from: 72-RESEARCH.md (full codebase analysis), existing CSS tokens, existing component inspection.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (Tailwind CSS v4 — `@import "tailwindcss"` with `@theme` block) |
| Preset | not applicable |
| Component library | none (all custom components, no shadcn/Radix/Base UI) |
| Icon library | Inline SVG paths + emoji (no dedicated icon library) |
| Font | `'Inter', sans-serif` (body), `'Fira Code', monospace` (code/terminal) |

**Source:** `app/src/index.css` `@theme` block.

---

## Spacing Scale

Declared values (must be multiples of 4):

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Icon gaps, inline padding |
| sm | 8px | Compact element spacing |
| md | 16px | Default element spacing (`p-4`, `gap-4`, `mb-4`) |
| lg | 24px | Section padding within cards |
| xl | 32px | Layout gaps between sections |
| 2xl | 48px | Major section breaks |
| 3xl | 64px | Page-level spacing |

**Exceptions:** none — follow existing CreateServerModal patterns (`p-6`, `space-y-4`, `mb-1`, `pt-4` for button row).

---

## Typography

| Role | Size | Weight | Line Height |
|------|------|--------|-------------|
| Body | 14px (`text-sm`) | 400 (normal) | 1.5 |
| Label | 12px | 400 (normal) | 1.5 |
| Heading | 20px (`text-xl`) | 700 (bold) | 1.2 |
| Display | 24px (`text-2xl`) | 700 (bold) | 1.2 |

**Context:** CreateServerModal uses `text-xl font-bold` for modal title, `text-sm` for inputs, `text-gray-400 text-xs` for hints. Follow same pattern for Bedrock additions.

---

## Color

| Role | Value | Usage |
|------|-------|-------|
| Dominant (60%) | `#080b15` (`--color-deep-space`) | Page background, modal overlay (`bg-black/50`) |
| Secondary (30%) | `#1f2937` (`bg-gray-800`) | Modal container, card surfaces |
| Accent (10%) | `#0ddff2` (`--color-cosmic-cyan`) | Button backgrounds, focus rings (`focus:ring-blue-500`), links |
| Destructive | `#ef4444` (`--color-cosmic-red`) | Delete/destroy actions only |

**Accent reserved for:** Primary CTA button (Create Server), focus rings, link hover states, selected active state.

**Form field colors (existing CreateServerModal pattern):**
- Input/select background: `bg-gray-700`
- Input text: `text-white`
- Placeholder: `text-gray-400`
- Label: `text-gray-400`
- Error/warning text: `text-yellow-400`
- Focus ring: `focus:ring-2 focus:ring-blue-500`
- Submit button: `bg-blue-600 text-white hover:bg-blue-700`
- Cancel button: `bg-gray-700 text-white hover:bg-gray-600`

---

## Copywriting Contract

| Element | Copy |
|---------|------|
| Primary CTA | "Create Server" (no change — remains same for all game types) |
| Game type option (fallback) | "Minecraft Bedrock" |
| Game type badge in server cards | "Bedrock" / "minecraft-bedrock" |
| Port label (Bedrock) | "Server Port (UDP)" or "Server Port — UDP (19132)" |
| UDP port hint | "Bedrock servers use UDP protocol instead of TCP" |
| Empty state heading | No change — uses existing "No servers found. Create your first server to get started." |
| Error state | No change — uses existing server creation error toast pattern |
| Destructive confirmation | No change — uses existing delete confirmation pattern |

**Bedrock-specific field labels:**
| Field | Label | Options |
|-------|-------|---------|
| Game Mode | Game Mode | Survival, Creative, Adventure |
| Allow Cheats | Allow Cheats | True, False |
| Level Name | Level Name | (text input, placeholder: "Bedrock Server") |
| Max Players | Max Players | 10, 20, 30, 50, 100 (same as Java) |

**Fields HIDDEN when Bedrock selected:**
- Minecraft Version (version dropdown) — Bedrock has no Java-version concept
- Server Type / Variant (Paper/Vanilla/Spigot/Forge/Fabric) — Bedrock uses `vanilla` only
- JVM Options — Bedrock is C++, not Java
- RAM Allocation / Max Memory — Apply at Docker level; hide from UI to avoid confusion
- Modpack selector — Bedrock doesn't use Java mods; behavior packs are different

---

## Component Inventory

| Component | File | Change Required |
|-----------|------|----------------|
| CreateServerModal.jsx | `app/src/features/server/CreateServerModal.jsx` | **MODIFY** — add bedrock game type, conditional field rendering |
| ServerManagerPage.jsx | `app/src/pages/servers/ServerManagerPage.jsx` | **MODIFY** — show game_type label for bedrock in server cards |
| Dashboard.jsx | `app/src/pages/Dashboard.jsx` | **VERIFY** — game_type display works generically |
| TemplateCreatePage.jsx | `app/src/pages/templates/TemplateCreatePage.jsx` | Already has `"bedrock"` option — no change needed |
| TemplateCard.jsx | `app/src/components/TemplateCard.jsx` | Already renders `game_type` generically — no change needed |

---

## Interaction Design: CreateServerModal — Bedrock Flow

### Game Type Selection

**Current state:** Fallback options show only Minecraft, Palworld (disabled), Rust (disabled), Valheim (disabled). When templates are loaded, options are dynamically generated from template game_types.

**Target state:** When no templates are loaded (fallback mode), add "Minecraft Bedrock" as an active (non-disabled) option:

```
<option value="minecraft">Minecraft</option>
<option value="bedrock">Minecraft Bedrock</option>
<option value="palworld" disabled>Palworld (Coming Soon)</option>
<option value="rust" disabled>Rust (Coming Soon)</option>
<option value="valheim" disabled>Valheim (Coming Soon)</option>
```

### Conditional Field Rendering

When `gameType === 'bedrock'` (or `['bedrock', 'minecraft-bedrock'].includes(gameType)`):

**HIDE the following Minecraft-Java block (lines 475–633):**
```
{gameType === 'minecraft' && (...)}
```

**SHOW a new Bedrock-specific block with these fields:**

1. **Max Players** — `<select>` with PLAYER_OPTIONS (reuse existing constant)
2. **Online Mode** — `<select>` with True/False (reuse existing UI pattern)
3. **Game Mode** — `<select>`: Survival, Creative, Adventure
4. **Difficulty** — `<select>`: Peaceful, Easy, Normal, Hard (reuse existing UI pattern)
5. **Allow Cheats** — `<select>`: True, False
6. **Level Name** — `<input>` with placeholder "Bedrock Server"
7. **World Seed** — `<input>` with placeholder "Leave empty for random" (reuse existing UI pattern)
8. **Server Port (UDP)** — `<input type="number">` with default `19132`, hint text "Bedrock servers use UDP port 19132 by default"

### Port Validation

Existing port validation (`10000–30000`, uniqueness check) applies to Bedrock too. The default port for Bedrock should be `19132` (UDP default for Bedrock protocol) instead of `25565` (Java TCP default).

### Form Submission Payload

When `game_type === 'bedrock'`, the form payload should be:
```javascript
const serverData = {
  name: name.trim(),
  game_type: 'bedrock',        // matches selected game type
  minecraft_version: undefined, // no version for bedrock
  ram_mb: 2048,                // sensible default if hidden; or send null/undefined
  max_ram_mb: undefined,       // no JVM max memory concept
  max_players: parseInt(maxPlayers),
  port: parseInt(port),        // defaults to 19132
  online_mode: onlineMode === 'true',
  world_seed: worldSeed || undefined,
  difficulty,
  op: op || undefined,
  server_type: undefined,      // no server type variants
  jvm_opts: undefined,         // no JVM options
  node_id: nodeId || undefined,
  modpack_template_id: null,   // no modpacks for bedrock
}
```

### Reset Form Behavior

When game type switches between bedrock and other types, the `resetForm` / `handleGameTypeChange` handler should reset bedrock-specific fields too (gamemode, allow_cheats, level_name).

Default values on Bedrock selection:
- `port` → `'19132'`
- `difficulty` → `'normal'` (reuse existing)
- `maxPlayers` → `'20'` (reuse existing)

### Form Layout Pattern

Follow the exact same form field layout as existing fields:
```jsx
<div>
  <label className="block text-gray-400 mb-1">Field Name</label>
  <select
    value={value}
    onChange={handler}
    className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
  >
    <option value="x">Option X</option>
    ...
  </select>
</div>
```

---

## Template Routing: Bedrock Caveat

The CreateServerModal dynamically loads templates from `/api/v1/templates`. If a template with `game_type: "bedrock"` exists in the database, it will automatically appear in the game type dropdown when templates are loaded. This means the fallback addition is only needed for the case when the API returns no templates (network error, empty DB).

The variant/sub-category dropdown will also work automatically if bedrock templates have variants like `vanilla`, `pocketmine`, `nukkit`, `powernukkitx` (as defined in TemplateCreatePage.jsx line 140).

---

## Bedrock Badge/Display in Server Cards

**In `ServerManagerPage.jsx` line 163:**
```jsx
// Current:
<p className="text-gray-400 text-sm capitalize">{server.config?.game_type || ... || 'server'}</p>

// Render already works generically — "bedrock" will display as "Bedrock" via capitalize.
// No change needed for the display logic itself.
```

**In other pages** that render `game_type` (Dashboard, ServerDetailsPage, DashboardPage): already use generic capitalize or config-based rendering. Verify no hardcoded "minecraft" → "Minecraft" mapping exists that would miss "bedrock" → "Minecraft Bedrock".

If human-friendly labels are needed, use a simple mapping:
```javascript
const GAME_TYPE_LABELS = {
  minecraft: 'Minecraft',
  bedrock: 'Minecraft Bedrock',
  'minecraft-bedrock': 'Minecraft Bedrock',
  palworld: 'Palworld',
  valheim: 'Valheim',
  rust: 'Rust',
}
```

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not required (no shadcn in project) |
| third-party | none | not required |

---

## States & Edge Cases

| State | Behavior |
|-------|----------|
| Templates API fails | Fallback game types include "Minecraft Bedrock" as active option |
| Templates API succeeds, has bedrock | Bedrock appears in dynamic dropdown automatically — no fallback needed |
| User selects Bedrock then switches to Minecraft | Reset Bedrock-specific fields; restore Java fields |
| User selects Minecraft then switches to Bedrock | Reset Java-specific fields; show Bedrock fields; default port to 19132 |
| Port conflict (19132 taken) | Same validation error pattern: "Port already in use by server: {name}" |
| Form submission with bedrock | `server_type`, `jvm_opts`, `minecraft_version` set to `undefined`/omitted |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
