# Phase 37: Terminal untuk Server Minecraft - Context

**Gathered:** 2026-05-04
**Status:** Ready for planning

<domain>
## Phase Boundary

In-browser terminal emulator untuk melihat live output dan mengirim commands ke game server. Mirip console RCON tapi dengan UI terminal yang lebih baik.

</domain>

<decisions>
## Implementation Decisions

### Terminal UI Component
- **D-01:** xterm.js
- Industry standard, full ANSI support, actively maintained
- Web-based terminal emulator yang berjalan di browser

### Connection Method
- **D-02:** WebSocket + RCON
- WebSocket untuk live output (stdout/stderr dari server)
- RCON untuk mengirim commands (stdin ke server)
- Real-time, low latency

### Command Features
- **D-03:** Advanced
- Command history (simpan dan recall)
- Command aliases
- History search
- Clear command

### Supported Games
- **D-04:** RCON-enabled games saja
- Games yang punya built-in RCON support
- Includes: Minecraft (Java), Palworld, Rust, Valheim, dll
- Tidak untuk games tanpa RCON

</decisions>

<canonical_refs>
## Canonical References

- `app/src/components/PluginManager.jsx` — Reference untuk component integration
- `api/src/presentation/handlers/rcon_handlers.rs` — RCON implementation
- RCON protocol documentation

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- RCON handlers sudah ada di backend
- PluginManager component pattern untuk integrating new features
- WebSocket infrastructure untuk real-time communication

### Integration Points
- Server detail page → Terminal tab
- WebSocket connection per server
- RCON command endpoint

</code_context>

<specifics>
## Specific Details

1. UI:
   - Tab di server detail page: "Terminal"
   - xterm.js terminal emulator di browser
   - Dark theme yang cocok dengan app design

2. Features:
   - Live console output (stdout/stderr)
   - Input field untukcommands
   - Command history (arrow up/down)
   - Autocomplete untuk common commands

3. Connection:
   - WebSocket stream dari agent
   - RCON untuk send command
   - Auto-reconnect jika koneksi terputus

</specifics>

<deferred>
## Deferred Ideas

- Terminal syntax highlighting (future)
- Multiple terminal tabs (future)
- Save/export terminal sessions (future)

</deferred>

---

## ▶ Next Up

**Phase 37: Terminal** — In-browser terminal emulator

`/clear` then:

`/gsd-plan-phase 37 v1.0`