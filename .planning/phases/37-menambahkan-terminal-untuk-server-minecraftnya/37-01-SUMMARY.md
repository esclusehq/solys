---
phase: 37
plan: 01
subsystem: frontend
tags: [terminal, xterm, rcon, websocket]
dependency_graph:
  requires: []
  provides: [terminal-ui]
  affects: [server-details]
tech_stack:
  added: [@xterm/xterm, @xterm/addon-fit]
  patterns: [xterm-js-terminal, websocket-streaming]
key_files:
  created:
    - path: app/src/components/Terminal.jsx
      size: "400 lines"
      provides: "xterm.js terminal emulator component"
    - path: app/src/hooks/useTerminal.js
      size: "211 lines"
      provides: "WebSocket and RCON connection hook"
  modified:
    - path: app/src/pages/ServerDetails.jsx
      changes: "Added Terminal tab"
    - path: app/package.json
      changes: "Added @xterm/xterm, @xterm/addon-fit"
decisions:
  - Used modern @xterm packages (deprecated xterm package in favor of @xterm/xterm)
  - Implemented auto-reconnect with exponential backoff (1s base, 30s max)
  - Command history stored in localStorage with 50 command limit
metrics:
  duration: ~3 min
  completed_date: "2026-05-03"
---

# Phase 37 Plan 01: Terminal untuk Server Minecraft Summary

One-liner: In-browser terminal emulator using xterm.js with WebSocket live output and RCON command input for game servers

## Implementation

### Created Files

1. **Terminal.jsx** (400 lines)
   - xterm.js terminal emulator component with dark theme
   - WebSocket connection management
   - Command input with history navigation (arrow keys)
   - Autocomplete for common Minecraft commands
   - Clear terminal command support

2. **useTerminal.js** (211 lines)
   - WebSocket hook for terminal connection
   - Auto-reconnect with exponential backoff
   - Command history in localStorage
   - Output buffering (1000 lines)

### Modified Files

1. **ServerDetails.jsx**
   - Added Terminal import
   - Added "Terminal" tab button
   - Added Terminal tab content area

2. **package.json**
   - Added @xterm/xterm
   - Added @xterm/addon-fit

## Features

- Live console output from game server via WebSocket
- Send commands via RCON protocol
- Command history with arrow key navigation (up/down)
- Command autocomplete (Tab key)
- Clear terminal (clear/cls command)
- Auto-reconnect on disconnect
- Dark theme matching app design

## Verification

- [x] Terminal.jsx exists and imports xterm correctly
- [x] useTerminal.js exports connect/disconnect/sendCommand functions
- [x] ServerDetails.jsx has Terminal tab in tab navigation
- [x] Build passes without import errors

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None - all functionality implemented.

## Threat Flags

None - no new security surface introduced.