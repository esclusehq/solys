# Phase 88: Update UI/UX console header and terminal layout - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-18
**Phase:** 88-update-ui-ux-console-header-and-terminal-layout
**Areas discussed:** Terminal xterm layout, Server selector UX, Connection status & reconnection UX

---

## Terminal xterm layout

| Option | Description | Selected |
|--------|-------------|----------|
| Keep current layout, polish styling | Header + xterm canvas + command input + help legend — refined styling | |
| **Full-height terminal with floating controls** | **xterm fills all available height. Controls as overlay on hover** | **✓** |
| Integrated footer input | Embed input into xterm instance via addon | |

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: Clear + Reconnect | Small floating button group, visible on hover | |
| **Full toolbar: Clear, Reconnect, Search, Copy** | **More actions. Always visible slim toolbar** | **✓** |
| Context menu only | Right-click context menu, no floating buttons | |

| Option | Description | Selected |
|--------|-------------|----------|
| Floating input bar at bottom | Overlay-style input bar below terminal | |
| **Inline xterm input (authentic terminal feel)** | **User types directly into xterm instance at a prompt** | **✓** |
| Persistent bottom bar (refined) | Keep input bar as distinct element, visually tighter | |

| Option | Description | Selected |
|--------|-------------|----------|
| Collapsible help bar | Visible by default, collapsible via toggle | |
| **Hidden by default — tooltip help** | **No visible legend, help via tooltips/popover** | **✓** |
| Removed entirely | No help at all | |

**User's choice:** Full-height + floating toolbar + inline xterm input + tooltip help
**Notes:** Terminal should feel like a real SSH terminal. No separate input widget.

---

## Server selector UX

| Option | Description | Selected |
|--------|-------------|----------|
| Dropdown selector (refined) | Keep dropdown at top, cosmetic refinements | |
| **Dedicated server sidebar** | **Collapsible left sidebar with server list** | **✓** |
| Tab bar for recent/pinned | Tabs above terminal for quick switching | |

| Option | Description | Selected |
|--------|-------------|----------|
| **Name + status dot only** | **Compact entries — server name + green/red dot** | **✓** |
| Name + status + game type + IP | Richer entries with badges | |
| Name + status + quick metrics | Inline resource usage bars | |

| Option | Description | Selected |
|--------|-------------|----------|
| Collapsible (default expanded) | Visible by default, collapse to icon strip | |
| **Collapsible (default collapsed)** | **Hidden by default, show via toggle/hover** | **✓** |
| Always visible | Fixed sidebar, no collapse | |

**User's choice:** Dedicated server sidebar, collapsed by default, name+status only

---

## Connection status & reconnection UX

| Option | Description | Selected |
|--------|-------------|----------|
| Floating toolbar (top-right) | Part of toolbar, near other buttons | |
| Sidebar footer | Bottom of sidebar, hidden when collapsed | |
| **Terminal title bar area** | **Compact indicator in terminal's top area** | **✓** |

| Option | Description | Selected |
|--------|-------------|----------|
| **Dot + "Connected"/"Disconnected" only** | **Simple colored dot + label. Clean and minimal** | **✓** |
| Dot + label + latency/ping | Add ping display | |
| Dot + label + server info (host:port) | Show connected server details | |

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-reconnect (current) + manual button | Keep auto-reconnect, add manual Reconnect button | |
| Manual reconnect only | No auto-reconnect | |
| **Auto-reconnect with status overlay** | **Overlay on terminal: "Disconnected — reconnecting in Xs..." + click to retry** | **✓** |

**User's choice:** Title bar area, dot+text only, auto-reconnect with overlay + manual button

---

## the agent's Discretion

- Floating toolbar icon styling and layout
- Inline xterm input implementation approach
- Sidebar animation and transition effects
- Reconnect overlay design and copy
- Server name display when sidebar collapsed

## Deferred Ideas

None.
