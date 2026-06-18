---
phase: 88-update-ui-ux-console-header-and-terminal-layout
plan: 01
subsystem: Terminal
tags: [xterm, terminal, floating-toolbar, search, reconnect, theme]
key-files:
  - app/src/theme/terminalTheme.js
  - app/src/components/FloatingToolbar.jsx
  - app/src/components/Terminal.jsx
  - app/package.json
metrics:
  files_created: 2
  files_modified: 2
  build: pass
---

## Commits

| Task | Description | Hash |
|------|-------------|------|
| 1 | Extract xterm cosmic theme to shared terminalTheme.js | TBD |
| 2 | Install @xterm/addon-search and @xterm/addon-web-links | TBD |
| 3 | Create FloatingToolbar component with opacity hover, search, clear, copy, reconnect | TBD |
| 4 | Refactor Terminal.jsx: inline input, full-height canvas, status bar, reconnect overlay, search, web links | TBD |

## Deviations

None — all tasks implemented as specified in the plan.

## Self-Check

**Result: PASSED**

- [x] Build passes (`npm run build` exit 0)
- [x] Terminal.jsx: xterm fills full viewport height (no input bar or legend below)
- [x] Terminal.jsx: inline input via onData with useRef buffer
- [x] Terminal.jsx: prompt `"> "` shown after each command
- [x] Floating toolbar visible at top-right: Clear, Search, Copy, Reconnect
- [x] Floating toolbar: 0.3 opacity default, 1.0 on hover, 2s fade-out
- [x] Terminal title bar: green dot + "Connected" / red dot + "Disconnected"
- [x] Reconnect overlay: appears on disconnect with countdown + "Retry Now"
- [x] Search: Ctrl+F opens search bar with up/down navigation
- [x] Web-links: clickable URLs in terminal output
- [x] Ctrl+L clears terminal
- [x] History: arrow up/down navigates via xterm rewrite
- [x] Theme extracted: `src/theme/terminalTheme.js` imported by Terminal.jsx
- [x] npm packages: `@xterm/addon-search`, `@xterm/addon-web-links` installed
