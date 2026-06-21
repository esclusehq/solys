# Phase 88: Update UI/UX console header and terminal layout - Context

**Gathered:** 2026-06-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Restructure and polish the Console page header and Terminal component layout. The Console page (`Console.jsx`) and Terminal component (`Terminal.jsx`) exist but use a basic stacked layout with dropdown server selector, separated input bar, and static help legend. This phase redesigns these into a more polished, terminal-authentic experience with a full-height xterm layout, server sidebar, and improved connection UX.

No new backend endpoints — purely frontend changes to existing components. The terminal WebSocket and xterm.js integration remain unchanged at the connection level.
</domain>

<decisions>
## Implementation Decisions

### Terminal Layout — Full-Height with Floating Toolbar
- **D-01:** xterm.js canvas fills the entire available viewport height — no separate input bar or gaps below the terminal
- **D-02:** Floating toolbar at top-right of the terminal with: Clear, Reconnect, Search, Copy buttons. Always visible as a slim toolbar.
- **D-03:** Command input is handled inline via xterm.js addon — user types directly into the xterm instance at a prompt, like a real SSH terminal. No external input widget.
- **D-04:** Keyboard shortcut help is removed from the page footer. Help is accessible via tooltip on toolbar buttons and/or a "?" Help button in the toolbar that shows a popover.
- **D-05:** The xterm.js theme remains the existing dark cosmic palette — no changes to terminal colors.

### Server Selector — Collapsible Sidebar
- **D-06:** Replace the top dropdown with a dedicated server sidebar on the left side of the Console page.
- **D-07:** Sidebar shows each server entry as: server name + colored status dot only (minimal, compact).
- **D-08:** Sidebar is collapsible, default collapsed (hidden). Toggle via a hamburger/panel button.
- **D-09:** The "Back to Server" link / breadcrumb area should be rethought — with the sidebar collapsed by default, the server name for the currently connected server should be visible in the terminal title area.

### Connection Status — Minimal + Terminal Title Bar
- **D-10:** Connection status indicator lives in the terminal title bar area (not in floating toolbar, not in sidebar footer).
- **D-11:** Shows: status dot (green=connected, red=disconnected) + "Connected"/"Disconnected" text only. No latency/ping, no host:port info.
- **D-12:** Auto-reconnect with exponential backoff (keep existing logic). Show a translucent overlay on the terminal when disconnected: "Disconnected — reconnecting in Xs..." with a click-to-retry-now button. Terminal output remains visible behind the overlay.
- **D-13:** Manual Reconnect button in the floating toolbar for immediate retry.

### What Happens to the Console Header (Breadcrumbs)
- **D-14:** The header area that previously held the dropdown + status can be simplified to just a breadcrumb / "Back to Server" link + current server name display (since the server selector moved to the sidebar).

### the agent's Discretion
- Exact visual styling of the floating toolbar icons and layout
- Implementation approach for inline xterm input (addon vs custom prompt handling)
- Sidebar animation transition timing and styling
- Reconnect overlay design and copy text
- How the server name displays in the title area when sidebar is collapsed
- Search feature implementation (xterm.js addon vs custom)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Target Files (Console & Terminal)
- `app/src/pages/Console.jsx` — Console page with header, server dropdown, terminal area
- `app/src/components/Terminal.jsx` — xterm.js terminal component with WebSocket, input bar, help legend
- `app/src/hooks/useTerminal.js` — WebSocket connection management hook for terminal

### Existing UI Patterns (Cosmic Theme)
- `app/src/index.css` — CSS variables, glass-panel, stars-bg, cosmic theme tokens
- `app/src/store/uiStore.js` — Zustand theme state management
- `app/src/components/Sidebar.jsx` — Main app sidebar (reference for new console sidebar styling)

### Related Console/Terminal Files
- `app/src/components/IDE/TerminalPanel.jsx` — Alternative terminal panel in Web IDE (reference for inline input pattern)
- `app/src/components/IDE/TerminalTabs.jsx` — Multi-tab terminal in Web IDE
- `app/src/pages/servers/ServerDetailsPage.jsx` — Server detail page with "Open Console" link (line 212-217)
- `app/src/app/App.jsx` — Route definition for `/console`

### Established UI Patterns (from prior phases)
- Phases 75-82 cosmic theme patterns: glass-panel, cosmic borders, cyan accent, focus rings
- Phase 81 dashboard sidebar/panel pattern
- Phase 82 semantic CSS variables and light/dark theme system

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `@xterm/xterm` + `@xterm/addon-fit` — already installed and configured in Terminal.jsx
- `useTerminal.js` hook — WebSocket connection management with auto-reconnect, history, state. Can be reused or referenced for the new layout
- `Sidebar.jsx` — Existing app sidebar with cosmic styling, collapsible pattern (reference for new server sidebar styling)
- `TerminalPanel.jsx` (IDE) — Uses inline pseudo-terminal input pattern without external command bar. Good reference for authentic terminal feel.
- `uiStore.js` — Zustand store pattern for state management (could extend for console-specific state)
- `glass-panel`, cosmic borders, status badges — established CSS utility patterns

### Established Patterns
- Cosmic theme: glass-panel containers, `border-[var(--color-cosmic-border)]`, `focus:ring-[var(--color-cosmic-cyan)]`, status badge variants (green/red/orange), glow hover effects
- Zustand for UI state, react-router-dom for routing with query params
- xterm.js with FitAddon for responsive terminal sizing

### Integration Points
- Console page route at `/console` with `?serverId=` query param for pre-selecting a server
- Sidebar nav item "RCON Console" at `/console` (Sidebar.jsx line 9)
- ServerDetailsPage "Open Console" link navigates to `/console?serverId=${id}` (line 212-217)
- WebSocket connects to `/ws/terminal/${serverId}` — no changes to WS endpoint or protocol
- xterm.js theme is configured inline in Terminal.jsx — may need extraction if search/find addons are added

### Creative Options
- Could use xterm.js `addon-search` and `addon-web-links` for search and clickable links in terminal output
- Could extract xterm theme config to a shared constants file for consistency across Terminal.jsx and IDE TerminalPanel/TerminalTabs
- Sidebar could use existing Zustand stores (`serverStore`) for live server list

</code_context>

<specifics>
## Specific Ideas

- The floating toolbar should be subtle — low opacity by default, full opacity on hover or when the user moves mouse near it (similar to VSCode's floating terminal toolbar)
- The reconnect overlay should show both the countdown and the current retry attempt number (e.g., "Reconnecting... attempt 3/10")
- Inline xterm input means removing the current `handleKeyDown`, `handleSubmit`, `navigateHistory` logic from the React component and relying on xterm.js `onData` handler instead — command history can still be managed but needs to integrate with xterm's write/input cycle
- Search overlay (Ctrl+F style) should be a small find bar that slides down from the toolbar

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope
</deferred>

---

*Phase: 88-update-ui-ux-console-header-and-terminal-layout*
*Context gathered: 2026-06-18*
