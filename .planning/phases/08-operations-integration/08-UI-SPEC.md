---
phase: 08
slug: operations-integration
status: draft
shadcn_initialized: false
preset: none
created: 2026-06-03
---

# Phase 8 — UI Design Contract

> Visual and interaction contract for phase 8: RCON console and SFTP file browser.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | Tailwind v4 + custom CSS variables |
| Preset | not applicable |
| Component library | custom (glass-panel, status-dot, modal, toast patterns) |
| Icon library | SVG inline (sidebar uses custom path-based icons) |
| Font | Inter (sans), Fira Code (mono), JetBrains Mono (terminal) |

---

## Spacing Scale

Declared values (Tailwind defaults + custom cosmic theme):

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Icon gaps, inline padding in toolbar |
| sm | 8px | Dot indicators, badge padding |
| md | 16px | Element spacing, input padding |
| lg | 24px | Section padding in glass-panels |
| xl | 32px | Layout gaps between sections |
| 2xl | 48px | Major section breaks |
| 3xl | 64px | Page-level spacing |

Exceptions: Terminal xterm.js uses its own internal spacing; FileManager tree nodes use 24px indent per level.

---

## Typography

| Role | Size | Weight | Line Height |
|------|------|--------|-------------|
| Body | 14px | 400 | 1.5 |
| Label | 12px | 500 | 1.4 |
| Heading (h3) | 18px | 700 | 1.3 |
| Page Title | 24px | 700 | 1.2 |
| Terminal | 13px | 400 | 1.3 |
| Monospace code | 13px | 400 | 1.5 |

---

## Color

| Role | Value | Usage |
|------|-------|-------|
| Dominant (60%) | `#080b15` (deep-space) | Page background |
| Secondary (30%) | `rgba(255,255,255,0.03)` (cosmic-card) | Glass panels, cards |
| Accent (10%) | `#0ddff2` (cosmic-cyan) | Active tab underline, primary button, focus ring, connection indicator |
| Destructive | `#ef4444` (cosmic-red) | Delete, stop, error states |
| Success | `#10b981` (cosmic-green) | Running status, connected indicator |
| Warning | `#f59e0b` (cosmic-orange) | Sleep status, warning states |

Accent reserved for: active tab indicators, primary action buttons, focus rings, connection status dots, command prompt prefixes ($), interactive hover states.

---

## Page Structure

### RCON Console Page (`/console`)

```
┌─────────────────────────────────────────────┐
│ Header: "Console" + server selector dropdown│
│         + connection status (pulsing dot)    │
├─────────────────────────────────────────────┤
│                                             │
│  ┌───────────────────────────────────────┐  │
│  │ xterm.js terminal (min 400px)         │  │
│  │ Cosmic dark theme                     │  │
│  │ JetBrains Mono 13px                   │  │
│  │ 10,000-line scrollback                │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  ┌───────────────────────────────────────┐  │
│  │ Command input bar with $ prompt       │  │
│  │ Tab autocomplete for Minecraft cmds   │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  Keyboard shortcuts: ↑↓ history, Tab       │
│  autocomplete, Ctrl+C cancel               │
└─────────────────────────────────────────────┘
```

### File Browser Tab (in ServerDetails)

```
┌─────────────────────────────────────────────┐
│ Toolbar: View toggle (list/tree)            │
│         + New folder + Upload + Refresh     │
├─────────────────────────────────────────────┤
│ Breadcrumb: root > world > plugins          │
├─────────────────────────────────────────────┤
│ ┌───┬────────────┬────────┬────────┬──────┐ │
│ │ ☐ │ Name       │ Size   │ Modified│Actions│
│ ├───┼────────────┼────────┼────────┼──────┤ │
│ │   │ 📁 plugins │ —      │ 2h ago  │ [...] │
│ │   │ 📄 config  │ 4.2KB  │ 1h ago  │ [...] │
│ └───┴────────────┴────────┴────────┴──────┘ │
│ Drag-and-drop upload overlay indicator       │
│ Upload progress bar (inline per file)        │
└─────────────────────────────────────────────┘
```

---

## Component Inventory

### RCON Console (NEW route + refine existing)

| Component | Source | Status |
|-----------|--------|--------|
| `Terminal.jsx` | Existing — xterm.js, WS at /ws/terminal/:id | REFINE: reuse in Console page |
| `Console.jsx` | Existing — lighter console page with log lines | FIX: add route to App.jsx |
| `useTerminal` hook | Existing — WS connection, history, reconnect | REUSE |
| Route `/console` | Sidebar links to it but no App.jsx route | ADD |

### File Browser (existing, SFTP backend wiring)

| Component | Source | Status |
|-----------|--------|--------|
| `FileManager.jsx` | Existing — full CRUD, tree/list view, chunked upload, clipboard | REUSE |
| Route in ServerDetails | Existing — `activeTab === 'files'` renders FileManager | REUSE |

---

## Copywriting Contract

| Element | Copy |
|---------|------|
| Console page title | "Console" |
| Terminal empty state | "Select a server to open its console" |
| Terminal disconnected | "Disconnected. Reconnecting..." |
| File browser empty directory | "This folder is empty" |
| File upload success | "Uploaded {filename}" |
| File upload error | "Failed to upload {filename}: {reason}" |
| Delete confirmation | "Are you sure you want to delete {filename}? This cannot be undone." |
| Command input prompt | `$` |

---

## Interaction Patterns

### Terminal
- **Connect:** WebSocket auto-connects on server select, auto-reconnect with backoff (max 10 attempts, 30s cap)
- **Send:** Enter key sends command via WS, response streams into xterm.js output
- **History:** ArrowUp/Down cycles localStorage history (max 50 commands)
- **Resize:** xterm fit-addon reflows on container resize
- **Visibility:** Reconnect on browser tab visibility change

### File Browser
- **Navigation:** Click breadcrumb path or tree node to navigate
- **Upload:** Drag-and-drop files onto the file list area with overlay indicator; regular upload (<5MB) or chunked (>5MB) with progress bar
- **Edit:** Double-click editable file extension → inline modal textarea → Save
- **Context actions:** Rename, Delete, Copy, Cut, Paste via action column dropdown or keyboard shortcuts
- **Selection:** Checkbox per row + select-all header checkbox; bulk delete/compress

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| npm packages | `@xterm/xterm`, `@xterm/addon-fit` | already in use |
| custom components | FileManager, Terminal, Console, StatusBadge | all existing |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
