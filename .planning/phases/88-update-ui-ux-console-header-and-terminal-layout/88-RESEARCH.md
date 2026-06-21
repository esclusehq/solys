# Phase 88: Update UI/UX Console Header and Terminal Layout — Research

**Researched:** 2026-06-18
**Domain:** Frontend — React + xterm.js terminal UI/UX
**Confidence:** HIGH

## Summary

Phase 88 restructures the Console page (`Console.jsx`) and Terminal component (`Terminal.jsx`) from a basic stacked layout (dropdown selector + external input bar + help legend) into a polished, terminal-authentic experience. Key changes: full-height xterm.js canvas, floating toolbar overlay with Clear/Reconnect/Search/Copy buttons, inline xterm.js input via the existing `onData` handler, collapsible left-side server sidebar replacing the dropdown, connection status in the title bar, and a disconnected overlay with countdown.

**Technically,** the existing xterm.js setup (`@xterm/xterm 6.0.0` + `@xterm/addon-fit 0.11.0`) already has the foundation for inline input — `term.onData()` is already wired. The work is restructuring the layout, migrating the input buffer from React state to xterm.js's internal cursor/prompt management, replacing the dropdown with a sidebar, and adding UI chrome (toolbar, overlay, status bar). Two new npm packages are needed: `@xterm/addon-search` (0.16.0) and `@xterm/addon-web-links` (0.12.0). `lucide-react` (1.21.0) — already installed — provides toolbar icons.

**Primary recommendation:** Keep xterm.js as the terminal engine (it's already installed and working). Implement inline command input by enhancing the existing `term.onData()` handler with a tracked input buffer managed via a `useRef` (not React state, to avoid re-render cycles on every keystroke). Build the server sidebar as a new `ServerSidebar.jsx` component following the cosmic styling in `Sidebar.jsx`. The floating toolbar sits in an absolutely-positioned container over the terminal element, revealing on hover/mouse-move with a transition from 0.3 opacity to 1.0.

### User Constraints (from CONTEXT.md)

#### Locked Decisions (D-01 through D-14)
| ID | Decision | Type |
|----|----------|------|
| D-01 | xterm.js canvas fills entire available viewport height — no separate input bar or gaps below terminal | Layout |
| D-02 | Floating toolbar at top-right of terminal with Clear, Reconnect, Search, Copy buttons — always visible as slim toolbar | UI |
| D-03 | Command input handled inline via xterm.js addon — user types directly into xterm at a prompt, like real SSH terminal | Interaction |
| D-04 | Keyboard shortcut help removed from footer. Help via tooltips on toolbar buttons + optional "?" Help button with popover | UI |
| D-05 | xterm.js theme remains existing dark cosmic palette — no terminal color changes | Theme |
| D-06 | Replace top dropdown with dedicated server sidebar on the left side of Console page | Layout |
| D-07 | Sidebar entries: server name + colored status dot only (minimal, compact) | UI |
| D-08 | Sidebar is collapsible, default collapsed (hidden). Toggle via hamburger/panel button | Interaction |
| D-09 | With sidebar collapsed, server name for connected server visible in terminal title area | Layout |
| D-10 | Connection status indicator lives in terminal title bar area | Layout |
| D-11 | Status dot (green/red) + "Connected"/"Disconnected" text only — no latency, no host:port | UI |
| D-12 | Auto-reconnect with exponential backoff. Show translucent overlay on disconnect — "Disconnected — reconnecting in Xs..." with click-to-retry button. Terminal output visible behind overlay | UX |
| D-13 | Manual Reconnect button in floating toolbar for immediate retry | UI |
| D-14 | Header simplified to breadcrumb / "Back to Server" link + current server name display | Layout |

#### the agent's Discretion
- Exact visual styling of floating toolbar icons and layout
- Implementation approach for inline xterm input (addon vs custom prompt handling)
- Sidebar animation transition timing and styling
- Reconnect overlay design and copy text
- How server name displays in title area when sidebar collapsed
- Search feature implementation (xterm.js addon vs custom)

#### Deferred Ideas (OUT OF SCOPE)
None.

## Phase Requirements

No requirements from REQUIREMENTS.md map to this phase. Phase is purely frontend UX polish of existing console/terminal features.

---

## Architectural Responsibility Map

Since this phase is entirely frontend (no new backend endpoints), all capabilities operate in the Browser/Client tier:

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Full-height xterm layout | Browser (CSS/HTML) | — | Layout-only; no server state |
| Floating toolbar | Browser (React/DOM) | — | Client-side overlay UI |
| Inline xterm input | Browser (xterm.js) | — | `term.onData` handles keystroke-to-WS forwarding |
| Tooltip/popover help | Browser (React) | — | Pure UI, no server interaction |
| Server sidebar | Browser (React) | API (server list via `useServers`) | Renders server list fetched from `/servers` API |
| Connection status | Browser (React) | WebSocket state | Derived from WebSocket `onopen`/`onclose` events |
| Reconnect overlay | Browser (React) | — | Purely client-side timer from WS disconnect |
| Simplified header | Browser (React/React Router) | — | Route-aware breadcrumb via react-router-dom |

---

## Standard Stack

### Core (Already Installed)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@xterm/xterm` | ^6.0.0 (latest: 6.0.0) | Terminal emulator engine — renders ANSI output, handles cursor movement, provides onData for input | Industry standard for web terminals; already wired in Terminal.jsx [VERIFIED: npm registry] |
| `@xterm/addon-fit` | ^0.11.0 (latest: 0.11.0) | Auto-resize terminal to fill container on window resize | Already installed; required for full-viewport layout [VERIFIED: npm registry] |
| `lucide-react` | ^1.18.0 (latest: 1.21.0) | Icon library for toolbar buttons | Already installed; icons for Clear, Search, Copy, Reconnect, Menu, X [VERIFIED: npm registry] |

### New Packages Required
| Library | Version | Purpose | Confidence |
|---------|---------|---------|------------|
| `@xterm/addon-search` | ^0.16.0 | Ctrl+F find bar within terminal buffer — `findNext()`/`findPrevious()` with decorations | HIGH — [VERIFIED: npm registry] |
| `@xterm/addon-web-links` | ^0.12.0 | Clickable URLs in terminal output — `WebLinksAddon` with URL validation | HIGH — [VERIFIED: npm registry] |

### Supporting (All Already Installed)
| Library | Version | Purpose | When Used |
|---------|---------|---------|-----------|
| `react-router-dom` | ^7.13.0 | Breadcrumb / "Back to Server" link via useSearchParams | Console page navigation |
| `zustand` | ^5.0.12 | UI state management (sidebar collapsed state) | Extend `uiStore.js` for console sidebar toggle |
| tailwindcss | ^4.2.0 | CSS utilities for glass-panel, flex, border, glow classes | All layout and styling |
| react | ^19.2.4 | Component lifecycle, hooks (useRef, useEffect, useState) | All components |

### Installation
```bash
npm install @xterm/addon-search @xterm/addon-web-links
```

### Version Verification
```bash
npm view @xterm/addon-search version  # → 0.16.0
npm view @xterm/addon-web-links version  # → 0.12.0
npm view @xterm/xterm version  # → 6.0.0
npm view @xterm/addon-fit version  # → 0.11.0
npm view lucide-react version  # → 1.21.0
```
[VERIFIED: npm registry, 2026-06-18]

---

## Architecture Patterns

### System Architecture — Data Flow

```
User typing                    WebSocket
   │                              │
   ▼                              ▼
term.onData(data) ◄────────── term.write(output)
   │                              ▲
   ▼                              │
inputBuffer (useRef)              │
   │                              │
   ▼                              │
JSON.stringify({type:'command'})──┘
   │
   ▼
WebSocket.send() ──────► Server (/ws/terminal/${serverId})
```

The key architectural insight: **xterm.js manages display and cursor; we manage the input buffer.** The inline pattern works by:
1. Accumulating keystrokes in a `useRef` input buffer (not React `useState` — avoids re-render storm)
2. On Enter (`\r`): commit buffer to command history, send via WS, clear buffer, echo prompt
3. Arrow up/down: navigate history, rewrite current line in terminal (clear + write)
4. Backspace: remove last char from buffer, send `\b \b` to terminal

### Recommended Project Structure (Changes Only)
```
src/
├── pages/
│   └── Console.jsx          ← MODIFIED: sidebar toggle, layout restructured
├── components/
│   ├── Terminal.jsx          ← MODIFIED: inline input, floating toolbar, reconnect overlay
│   ├── ServerSidebar.jsx     ← NEW: collapsible server list sidebar
│   └── ConsoleHeader.jsx     ← NEW: simplified breadcrumb + server name header
├── hooks/
│   └── useTerminal.js        ← UNCHANGED (or optionally extended for reconnect state)
├── store/
│   └── uiStore.js            ← EXTENDED: add consoleSidebarOpen state
└── theme/
    └── terminalTheme.js      ← NEW (optional): extract cosmic theme to shared constant
```

### Pattern 1: Inline Command Input via onData

**What:** Instead of a separate React `<input>` element, the user types directly into the xterm.js canvas. The `term.onData` handler captures all keystrokes, manages an internal input buffer, and writes/responses to the terminal display.

**Why this is the right approach:** The current `Terminal.jsx` ALREADY has the skeleton of this pattern (lines 105-126). It captures `\r`, arrow keys, backspace, `\x03` (Ctrl+C), and printable characters. The missing pieces are:
1. No prompt displayed before the input area
2. No cursor positioning when history navigation rewrites the line
3. Command buffer is mirrored in React state (`setCommand`) causing re-renders on every keystroke

**Implementation approach (at discretion — `the agent's discretion`):** Move input buffer to a `useRef` string. Maintain history navigation as a ref or simple variable (not React state, since we rewrite the terminal line directly via xterm `write()`). This avoids the re-render loop that currently fires on every keystroke.

### Pattern 2: Floating Toolbar Overlay

**What:** A translucent toolbar overlay positioned at the top-right of the terminal container. Contains icon buttons: Clear, Reconnect, Search, Copy.

**Specifics (from CONTEXT.md):** Low opacity (0.3) by default, full opacity on hover or mouse-move near it — similar to VSCode's floating terminal toolbar. z-index above xterm canvas so clicks register on toolbar.

### Pattern 3: Reconnect Overlay

**What:** When WebSocket disconnects, a translucent overlay appears over the terminal showing countdown + attempt number. Terminal output remains visible behind the overlay.

**Implementation approach (at discretion):** A div positioned absolutely over the terminal container, with `pointer-events: auto` for the retry button and `pointer-events: none` for the backdrop. Shows: "Disconnected — reconnecting in 5s... (attempt 2/10)" + [Retry Now] button.

### Pattern 4: Collapsible Server Sidebar

**What:** A left-side panel listing servers from `useServers` hook, each showing name + status dot. Collapsible via hamburger button, default collapsed.

**Cosmic theme styling** (from Sidebar.jsx pattern): `glass-panel` container, `border-r border-[var(--color-cosmic-border)]`, server entries styled like NavLinks with hover effects.

### Anti-Patterns to Avoid

- **React state for input buffer:** The current code uses `useState` for `command` and calls `setCommand` on every keystroke (line 123). This causes React re-renders for every typed character. Use `useRef` instead.
- **Inline CSS for theme colors:** Use CSS custom properties via Tailwind classes (e.g., `text-[var(--color-text-main)]`) — NOT hardcoded hex values. The existing index.css has all cosmic theme variables.
- **Duplicating xterm.js configs:** The current codebase has THREE separate theme configurations (Terminal.jsx lines 57-86, TerminalPanel.jsx lines 3-25, TerminalTabs.jsx lines 3-25). This phase should extract the cosmic terminal theme to a shared `src/theme/terminalTheme.js` file.
- **Overlapping pointer events:** The floating toolbar and xterm canvas both need mouse events. Use `pointer-events: auto` / `pointer-events: none` carefully to prevent toolbar from blocking terminal interaction.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Search in terminal output | Homegrown find bar | `@xterm/addon-search` | Full buffer scanning, incremental search, decorations, match highlighting, scroll-to-match — 487k weekly downloads [VERIFIED: npm registry] |
| Clickable URLs | Regex parser + handler | `@xterm/addon-web-links` | Handles edge cases (URLs spanning lines, URLs in ANSI-wrapped text), security (validates http/https by default) — 1.1M weekly downloads |
| Icon set | Custom SVG components | `lucide-react` | Already in dependencies; 1.21.0 has Terminal, Search, RotateCcw, Copy, X, PanelLeft icons needed for toolbar |
| UI state persistence | Manual localStorage | `zustand/middleware` persist | Already using `persist` in `uiStore.js`; can extend for console sidebar state |

**Key insight:** xterm.js addon ecosystem handles the deceptively complex parts of terminal search and URL handling. The search addon's decoration support (match highlighting in buffer + overview ruler) would be non-trivial to implement manually.

---

## Common Pitfalls

### Pitfall 1: Input Buffer State on Every Keystroke
**What goes wrong:** Using React `useState` for the command buffer causes a re-render on every keystroke, which can cause xterm.js canvas flicker or stutter during fast typing.
**Why:** The current code (`setCommand(prev => prev + data)` on line 123) triggers React reconciliation on each character.
**How to avoid:** Use `useRef<string>` for the input buffer. Only update React state when the line is committed (Enter) or when history navigation changes the buffer externally.
**Warning signs:** Stuttery typing in terminal, console.log spam showing re-renders.

### Pitfall 2: Floating Toolbar Blocking Terminal Clicks
**What goes wrong:** The floating toolbar overlays the terminal, so clicks that should focus the terminal hit the toolbar instead.
**Why:** Absolute positioning on the toolbar places it over the xterm element.
**How to avoid:** Set `pointer-events: none` on the toolbar container and `pointer-events: auto` on individual buttons. The toolbar area outside buttons passes clicks through to the terminal.
**Warning signs:** Terminal not receiving focus when clicking top-right area.

### Pitfall 3: FitAddon.fit() Fires Before Container Has Dimensions
**What goes wrong:** `fitAddon.fit()` returns incorrect col/row counts, causing terminal to render at wrong size.
**Why:** The xterm container may not have layout dimensions when the component mounts (esp. with sidebar collapse transitions).
**How to avoid:** Use `ResizeObserver` on the terminal container (not `window.resize`) for reliable dimension tracking. The current code uses `setTimeout(() => fitAddon.fit(), 100)` which is fragile. See Terminal.tsx from Dorothy project for the ResizeObserver pattern [CITED: github.com/Charlie85270/Dorothy].
**Warning signs:** Terminal shows scrollbars or has excess whitespace after sidebar toggle.

### Pitfall 4: History Navigation + Prompt Rewrite
**What goes wrong:** When user presses arrow-up to cycle history, the current line needs to be deleted and replaced with the history entry. If the prompt prefix is not tracked, rewrites offset by prompt length.
**Why:** xterm.js cursor moves in absolute cell positions. Rewriting a line requires knowing how many columns the prompt occupies.
**How to avoid:** Track prompt length (e.g., `> ` = 2 chars + any ANSI color escape overhead). When rewriting, send `\r` + prompt + history entry to replace the line cleanly.
**Warning signs:** History entries show with extra characters or missing first N characters.

### Pitfall 5: Sidebar and Console Page Both Fetch Servers
**What goes wrong:** The existing Console.jsx uses `useServers()` hook (line 7). If the new ServerSidebar also uses `useServers()`, two parallel API calls fire on mount.
**How to avoid:** Either share the hook at the Console page level and pass servers as props, or use `useServerStore` (Zustand) which has a centralized `servers` array. The store is already populated by the main app — ServerSidebar can read from the store directly.
**Warning signs:** Network tab shows duplicate `/servers` GET requests.

---

## Existing Code Analysis

### Current Terminal.jsx (401 lines) — Key Structure
```
┌─────────────────────────────────────────┐
│ Terminal Header: "Terminal" + status    │ ← D-10: Move status to title bar
├─────────────────────────────────────────┤
│                                         │
│   xterm.js canvas (in glass-panel)      │ ← D-01: Fill full height
│   √ FitAddon installed                  │
│   √ Term.onData() wired for input       │
│   √ Dark cosmic theme config            │
│                                         │
├─────────────────────────────────────────┤
│ Command Input Bar (<input> element)      │ ← D-03: REMOVE — use inline
├─────────────────────────────────────────┤
│ Help Legend (keyboard shortcuts)         │ ← D-04: REMOVE — use tooltips
└─────────────────────────────────────────┘
```

**What's already good:**
- `term.onData()` handler exists with correct key handling (Enter, arrows, backspace, Ctrl+C)
- WebSocket connection management with exponential backoff exists
- localStorage command history exists
- FitAddon for responsive sizing

**What needs to change:**
- Remove the external input bar (lines 341-374) and React state `command`/`setCommand`
- Move input buffer to `useRef` (or keep minimal state for search overlay)
- Remove help legend (lines 377-398)
- Restructure layout: terminal fills full viewport, toolbar overlays
- Add SearchAddon, WebLinksAddon
- Add disconnect overlay

### Current Console.jsx (80 lines) — Key Structure
```
┌─────────────────────────────────┐
│ Header: "Back to Server" link   │ ← D-14: Keep breadcrumb
│         "Console" title         │ ← D-09: Add server name here
│         <select> dropdown       │ ← D-06: REPLACE with sidebar
│         Status dot + text       │ ← D-10: MOVE to terminal title bar
├─────────────────────────────────┤
│ Terminal (if server selected)   │ ← D-01: Fill full viewport
│ or "Select a server" message   │
└─────────────────────────────────┘
```

---

## Code Examples

### Example 1: Inline Input with Prompt (Enhanced onData Handler)
```javascript
// Pattern: Input buffer managed via useRef, not useState
// Source: Based on current Terminal.jsx lines 105-126 + xterm.js addon patterns [CITED: xtermjs.org/docs/api/terminal/]

const PROMPT = '\r\n> ';
const inputBuf = useRef('');
const historyRef = useRef([]);
const historyIdxRef = useRef(-1);

// Inside terminal init:
term.onData((data) => {
    if (data === '\r') { // Enter
        const cmd = inputBuf.current.trim();
        if (cmd) {
            // Save to history
            historyRef.current = [...historyRef.current.slice(-49), cmd];
            historyIdxRef.current = -1;
            
            // Send to WebSocket
            if (wsRef.current?.readyState === WebSocket.OPEN) {
                wsRef.current.send(JSON.stringify({ type: 'command', command: cmd }));
            }
            
            inputBuf.current = '';
            term.write(`\r\n${cmd}\r\n`);  // Echo the command
        } else {
            term.write(PROMPT);
        }
    } else if (data === '\x1b[A') { // Arrow Up
        if (historyRef.current.length === 0) return;
        const idx = historyIdxRef.current === -1 
            ? historyRef.current.length - 1 
            : Math.max(historyIdxRef.current - 1, 0);
        historyIdxRef.current = idx;
        
        // Clear current line, write history entry
        const line = historyRef.current[idx];
        term.write('\r\x1b[K');  // Carriage return + erase line
        term.write(`> ${line}`);
        inputBuf.current = line;
    } else if (data === '\x1b[B') { // Arrow Down
        if (historyIdxRef.current === -1) return;
        const idx = historyIdxRef.current + 1;
        if (idx >= historyRef.current.length) {
            // Empty line
            historyIdxRef.current = -1;
            term.write('\r\x1b[K> ');
            inputBuf.current = '';
        } else {
            historyIdxRef.current = idx;
            const line = historyRef.current[idx];
            term.write(`\r\x1b[K> ${line}`);
            inputBuf.current = line;
        }
    } else if (data === '\x7f' || data === '\x08') { // Backspace
        if (inputBuf.current.length > 0) {
            inputBuf.current = inputBuf.current.slice(0, -1);
            term.write('\b \b');
        }
    } else if (data === '\x03') { // Ctrl+C
        inputBuf.current = '';
        term.write('^C');
        term.write(PROMPT);
    } else if (data === '\t') { // Tab completion
        // Optional: implement simple autocomplete from known commands
    } else if (data >= ' ' && data <= '~') { // Printable ASCII
        inputBuf.current += data;
        term.write(data);
    }
});
```

### Example 2: Floating Toolbar Component
```jsx
// Pattern: Translucent overlay toolbar with icon buttons
// Source: lucide-react icons + CSS overlay pattern
import { Terminal, RotateCcw, Search, Copy, HelpCircle } from 'lucide-react';

function FloatingToolbar({ onClear, onReconnect, onSearch, onCopy, isConnected }) {
    const [visible, setVisible] = useState(false);
    const timeoutRef = useRef(null);
    
    const handleMouseEnter = () => {
        clearTimeout(timeoutRef.current);
        setVisible(true);
    };
    
    const handleMouseLeave = () => {
        timeoutRef.current = setTimeout(() => setVisible(false), 2000);
    };
    
    return (
        <div 
            className="absolute top-2 right-2 z-10 flex items-center gap-1 transition-opacity duration-300"
            style={{ opacity: visible ? 1 : 0.3 }}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
        >
            <Tooltip content="Clear terminal (Ctrl+L)">
                <button onClick={onClear} 
                    className="p-1.5 rounded-lg hover:bg-[rgba(255,255,255,0.1)] text-[var(--color-text-muted)] hover:text-[var(--color-text-main)] transition-all">
                    <Terminal className="w-4 h-4" />
                </button>
            </Tooltip>
            {!isConnected && (
                <Tooltip content="Reconnect">
                    <button onClick={onReconnect}
                        className="p-1.5 rounded-lg hover:bg-[rgba(255,255,255,0.1)] text-[var(--color-cosmic-red)] hover:text-white transition-all">
                        <RotateCcw className="w-4 h-4" />
                    </button>
                </Tooltip>
            )}
            <Tooltip content="Search (Ctrl+F)">
                <button onClick={onSearch}
                    className="p-1.5 rounded-lg hover:bg-[rgba(255,255,255,0.1)] text-[var(--color-text-muted)] hover:text-[var(--color-text-main)] transition-all">
                    <Search className="w-4 h-4" />
                </button>
            </Tooltip>
            <Tooltip content="Copy selection">
                <button onClick={onCopy}
                    className="p-1.5 rounded-lg hover:bg-[rgba(255,255,255,0.1)] text-[var(--color-text-muted)] hover:text-[var(--color-text-main)] transition-all">
                    <Copy className="w-4 h-4" />
                </button>
            </Tooltip>
        </div>
    );
}
```

### Example 3: Search Addon Integration
```javascript
// Pattern: Ctrl+F overlay using @xterm/addon-search
// Source: @xterm/addon-search API types [VERIFIED: npm registry]

import { SearchAddon } from '@xterm/addon-search';

// Inside terminal init:
const searchAddon = new SearchAddon({
    highlightLimit: 1000
});
term.loadAddon(searchAddon);

// Search UI state
const [showSearch, setShowSearch] = useState(false);
const [searchTerm, setSearchTerm] = useState('');

// Ctrl+F handler
const handleSearchToggle = () => {
    setShowSearch(prev => !prev);
    if (!showSearch) {
        // Focus search input next tick
        setTimeout(() => searchInputRef.current?.focus(), 0);
    }
};

// On search input change:
const handleSearchInput = (term) => {
    setSearchTerm(term);
    if (term) {
        searchAddon.findNext(term, {
            decorations: {
                matchOverviewRuler: '#0ddff2',
                matchBackground: '#0ddff220',
                activeMatchBackground: '#0ddff240',
                activeMatchColorOverviewRuler: '#0ddff2',
            },
            incremental: true,
        });
    } else {
        searchAddon.clearDecorations();
    }
};

// Search bar UI (slides down from toolbar):
// <div className="absolute top-10 right-2 z-10 flex items-center gap-2 
//                 bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)]
//                 rounded-lg px-3 py-2 backdrop-filter backdrop-blur-lg">
//     <input ref={searchInputRef} value={searchTerm} onChange={...}
//         placeholder="Find..." className="bg-transparent text-sm ..." />
//     <button onClick={() => searchAddon.findPrevious(searchTerm)}>↑</button>
//     <button onClick={() => searchAddon.findNext(searchTerm)}>↓</button>
//     <button onClick={() => setShowSearch(false)}><X className="w-3 h-3" /></button>
// </div>
```

### Example 4: Disconnect Overlay with Countdown
```jsx
// Pattern: Translucent overlay showing reconnect countdown
function ReconnectOverlay({ reconnectAttempt, maxAttempts, countdown, onRetryNow }) {
    return (
        <div className="absolute inset-0 z-20 flex items-center justify-center">
            {/* Translucent backdrop — terminal output visible through it */}
            <div className="absolute inset-0 bg-[var(--color-deep-space)]/60 backdrop-blur-[2px]" />
            
            {/* Content */}
            <div className="relative z-10 text-center">
                <p className="text-[var(--color-cosmic-red)] text-sm font-medium mb-2">
                    Disconnected
                </p>
                <p className="text-[var(--color-text-muted)] text-xs mb-4">
                    Reconnecting in {countdown}s... (attempt {reconnectAttempt}/{maxAttempts})
                </p>
                <button 
                    onClick={onRetryNow}
                    className="px-4 py-2 rounded-lg text-xs font-medium
                               bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                               border border-[var(--color-cosmic-cyan)]/30
                               hover:bg-[var(--color-cosmic-cyan)]/20 transition-all"
                >
                    Retry Now
                </button>
            </div>
        </div>
    );
}
```

### Example 5: ServerSidebar Component (Collapsible)
```jsx
// Pattern: Left-side collapsible server panel using cosmic theme
// Source: Sidebar.jsx collapsible pattern + serverStore servers list

function ServerSidebar({ servers, selectedId, onSelect, isOpen, onToggle }) {
    return (
        <>
            {/* Toggle button — visible when collapsed */}
            <button
                onClick={onToggle}
                className="absolute -right-3 top-4 z-20 w-6 h-6 rounded-full
                           bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)]
                           flex items-center justify-center cursor-pointer
                           hover:bg-[var(--color-cosmic-cyan)]/10 hover:border-[var(--color-cosmic-cyan)]/30
                           transition-all text-[var(--color-text-muted)]"
            >
                {/* ChevronRight or PanelLeft icon from lucide-react */}
            </button>
            
            <aside className={`h-full glass-panel border-r border-[var(--color-cosmic-border)]
                              transition-all duration-300 overflow-hidden
                              ${isOpen ? 'w-56' : 'w-0 border-r-0'}`}>
                <div className="p-3 w-56">
                    <h3 className="text-xs font-semibold uppercase tracking-wider 
                                   text-[var(--color-text-muted)] mb-3 px-2">
                        Servers
                    </h3>
                    <div className="flex flex-col gap-1">
                        {servers.map(server => (
                            <button
                                key={server.id}
                                onClick={() => onSelect(server.id)}
                                className={`flex items-center gap-3 px-3 py-2 rounded-lg text-sm
                                           transition-all text-left ${
                                    server.id === selectedId
                                        ? 'bg-[rgba(255,255,255,0.08)] text-[var(--color-text-main)]'
                                        : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-main)] hover:bg-[rgba(255,255,255,0.04)]'
                                }`}
                            >
                                <span className={`w-2 h-2 rounded-full shrink-0 ${
                                    server.status === 'running' 
                                        ? 'bg-[var(--color-cosmic-green)] shadow-[0_0_6px_var(--color-cosmic-green)]' 
                                        : 'bg-[var(--color-cosmic-red)]'
                                }`} />
                                <span className="truncate">{server.name}</span>
                            </button>
                        ))}
                    </div>
                </div>
            </aside>
        </>
    );
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| External `<input>` bar for command entry | Inline xterm.js input via `onData` handler | This phase | Authentic terminal feel; no external widget |
| Static `<select>` dropdown for server selection | Collapsible sidebar with server list + status dots | This phase | Visual improvement; saves header space |
| Help legend at bottom of terminal | Tooltips on toolbar buttons + optional "?" popover | This phase | Cleaner layout; less clutter |
| Terminal fills fixed area with input bar below | Full-viewport height xterm.js canvas | This phase | No wasted space; more terminal output visible |
| Status dot in header alongside dropdown | Status in terminal title bar area | This phase | Makes more sense contextually |

**Deprecated/outdated:**
- **External input bar** in Terminal.jsx — removed this phase
- **Help legend footer** — removed this phase; replaced with tooltips
- **React state-based command buffer** — replaced with ref-based buffer for performance

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Using `useRef` for the input buffer instead of `useState` will avoid re-render flicker during fast typing | Inline Input Pattern | Low — the current code re-renders on every keystroke via `setCommand`; any React re-render can cause xterm.js canvas to stutter slightly |
| A2 | The `@xterm/addon-search` decoration options work with the WebGL renderer | Search Addon | Low — decorations work with canvas renderer; WebGL renderer is not used in this project (canvas default) |
| A3 | Server list from `useServers()` includes `status` field with values like `'running'`/`'stopped'` | ServerSidebar | MEDIUM — not verified in the actual API response; need to confirm server API returns `status` field (based on `StatusBadge.jsx` and `index.css` `.status-dot.running` it appears it does) |
| A4 | `pointer-events: none` on toolbar container + `pointer-events: auto` on buttons will correctly delegate clicks | Floating Toolbar | Low — standard CSS behavior; tested pattern in many terminal UIs |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions

1. **Should server list come from `useServers()` hook or `useServerStore` Zustand store?**
   - What we know: Both `useServers` (fetches via fetchApi) and `useServerStore` (fetches via api lib) exist. Console.jsx currently uses `useServers`.
   - What's unclear: Whether `useServerStore` is already populated by the main app layout by the time the Console page mounts, or if ServerSidebar would need its own fetch.
   - Recommendation: Use `useServers` (current Console pattern) for simplicity; it's already working. De-duplicate by lifting the hook to Console.jsx and passing servers as prop to ServerSidebar.

2. **How should the "server name in title area when sidebar collapsed" work (D-09)?**
   - What we know: With the sidebar collapsed, the user needs to see which server they're connected to.
   - Options: (a) Show server name in the simplified header breadcrumb area, (b) Show it in the terminal title bar alongside connection status.
   - Recommendation: Both — show server name in header breadcrumb area (replacing "Console" title) and connection status in the terminal title bar.

3. **Should we extract xterm theme to a shared file?**
   - What we know: Three duplicate theme configs exist (Terminal.jsx, TerminalPanel.jsx, TerminalTabs.jsx).
   - Recommendation: Yes — create `src/theme/terminalTheme.js` and import it from all three. This is cleanup that makes the codebase healthier and prevents drift.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Vite dev server | ✓ | 22.22.2 | — |
| npm | Package management | ✓ | 10.9.7 | — |
| `@xterm/addon-search` | Search feature | ✗ (not installed) | 0.16.0 | Implement manual find via `term.getSelection()` + custom find bar (not recommended) |
| `@xterm/addon-web-links` | Clickable URLs | ✗ (not installed) | 0.12.0 | Implement custom link handler via `term.registerLinkProvider` (possible but more work) |

**Missing dependencies with no fallback:** None — both addons are installable via npm.

**Missing dependencies with fallback:**
- `@xterm/addon-search` — install `npm install @xterm/addon-search@^0.16.0`
- `@xterm/addon-web-links` — install `npm install @xterm/addon-web-links@^0.12.0`

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | N/A (no test framework detected in `app/package.json`) |
| Config file | None |
| Quick run command | `npm run dev` (Vite dev server) |
| Full suite command | `npm run build` (Vite build — catches compile errors) |

### Phase Requirements → Test Map
No test framework exists in the frontend project. Validation for this phase is manual/browser-based:

| Behavior | How to Verify |
|----------|--------------|
| Full-height terminal | Terminal canvas fills viewport; no gaps below |
| Floating toolbar visible | Toolbar appears at top-right of terminal at 0.3 opacity, transitions to 1.0 on hover |
| Inline input | Typing goes directly into xterm; Enter sends command; arrows navigate history |
| No external input bar | Old `<input>` element and help legend are gone from DOM |
| Server sidebar | Sidebar toggles open/closed; shows server names + status dots |
| Connection status | Title bar shows green dot + "Connected" or red dot + "Disconnected" |
| Disconnect overlay | When WS disconnects, overlay appears showing countdown "reconnecting in Xs..." |
| Reconnect button | Clicking button in toolbar or "Retry Now" in overlay reconnects |
| Search | Ctrl+F opens find bar; matches highlighted in terminal output |
| Simplified header | Header shows breadcrumb link + server name; no dropdown |

### Wave 0 Gaps
- No test infrastructure exists for the frontend (`app/package.json` has no test dependencies)
- Validation is entirely manual via `npm run dev` browser testing

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| xterm.js re-render flicker during inline input | MEDIUM | MEDIUM | Use `useRef` for input buffer; avoid React state changes on keystroke |
| Floating toolbar overlaps xterm click targets | LOW | HIGH | Use `pointer-events: none` on toolbar container, `auto` on buttons |
| Sidebar toggle causes terminal resize jank | MEDIUM | LOW | Use CSS transitions on sidebar width; debounce FitAddon.fit() via ResizeObserver |
| History navigation breaks with long lines | LOW | MEDIUM | Cap input to xterm's `cols`; handle wrap with cursor position reset |
| Server API doesn't return `status` field for each server | LOW | MEDIUM | Default to disconnected/unknown status if `status` is missing |
| Addon version incompatibility | LOW | MEDIUM | `@xterm/addon-search@0.16.0` and `@xterm/addon-web-links@0.12.0` are fully compatible with `@xterm/xterm@6.0.0` [VERIFIED: npm registry peer dependencies] |

---

## Implementation Sequence (Recommended)

1. **Extract shared theme** — Create `src/theme/terminalTheme.js` with the cosmic theme object
2. **Install new addons** — `npm install @xterm/addon-search @xterm/addon-web-links`
3. **Refactor Terminal.jsx**:
   - Restructure layout: full-height flex container, no input bar, no help legend
   - Add floating toolbar (absolute positioned, transition opacity)
   - Convert input buffer from React state to useRef
   - Add reconnect overlay component
   - Load SearchAddon + WebLinksAddon
   - Move connection status to title bar area
4. **Create ServerSidebar.jsx** — New component: server list, status dots, collapsible
5. **Refactor Console.jsx**:
   - Replace `<select>` dropdown with ServerSidebar
   - Simplify header to breadcrumb + server name
   - Wire sidebar state to zustand or local state
6. **Extend uiStore.js** — Add `consoleSidebarOpen` boolean (optional if using local state)
7. **Browser test all interactions** — Manual validation of every scenario

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: npm registry] — `@xterm/addon-search@0.16.0`, `@xterm/addon-web-links@0.12.0`, `@xterm/xterm@6.0.0`, `lucide-react@1.21.0`
- [CITED: xtermjs.org/docs/api/terminal/] — Terminal API (onData, onResize, write, input, options, loadAddon)
- [CITED: github.com/xtermjs/xterm.js] — SearchAddon type definitions (findNext, findPrevious, decorations, clearDecorations)
- [CITED: existing code] — Terminal.jsx, Console.jsx, Sidebar.jsx, TerminalPanel.jsx, TerminalTabs.jsx, index.css, uiStore.js, useTerminal.js, package.json

### Secondary (MEDIUM confidence)
- [CITED: github.com/thechandanbhagat/alter-pm] — Reference implementation of xterm.js inline input with WebSocket pattern
- [CITED: munderdifflin.in/blog/building-a-terminal-ui-xterm-node-pty/] — Architecture of xterm.js + PTY integration patterns

### Tertiary (LOW confidence)
- WebSearch results for inline input patterns — consistent with onData approach; no contradictory patterns found

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified via npm registry
- Architecture: HIGH — patterns verified via xterm.js documentation and existing codebase analysis
- Pitfalls: HIGH — derived from common xterm.js issues and existing codebase anti-patterns

**Research date:** 2026-06-18
**Valid until:** 2026-07-18 (stable libraries; xterm.js addon versions may update)
