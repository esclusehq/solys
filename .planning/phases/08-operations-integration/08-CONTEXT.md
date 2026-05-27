# Phase 8: Operations Integration - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

RCON console and SFTP file management for server administration. This phase establishes the console interface, file browser, transfer behavior, and security model.

**Requirements:** RCON-01, RCON-02, FILE-01, FILE-02, FILE-03

**Success criteria:**
1. User can connect to server via RCON protocol
2. User can execute console commands via RCON
3. User can browse server files via SFTP
4. User can upload files to server
5. User can download files from server
</domain>

<decisions>
## Implementation Decisions

### RCON Console Interface (D-28)
- **D-28:** WebSocket terminal
- Real-time interaction via WebSocket
- Command history stored in Redis
- Reference: `api/src/presentation/handlers/terminal_handlers.rs`

### SFTP/File Browser Interface (D-29)
- **D-29:** Web-based file browser
- In-browser file browser with tree view
- Integrated in dashboard
- Reference: `api/src/presentation/handlers/file_handlers.rs`

### File Transfer Behavior (D-30)
- **D-30:** Chunked upload with resume
- Chunked upload with progress bar
- Support resume on failure
- Reference: `web-agent/src/handlers/sftp.rs`

### Security Considerations (D-31)
- **D-31:** Path validation
- User must own server (checked via auth middleware)
- Path traversal blocked (../ sequences rejected)
- Allowed directories only (server data directory)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### RCON
- `api/src/infrastructure/executors/rcon_server_executor.rs` — RCON implementation
- `api/src/presentation/handlers/terminal_handlers.rs` — Terminal WebSocket
- `api/src/presentation/routes/server_routes.rs` — /ws/terminal/:id

### File Management
- `api/src/presentation/handlers/file_handlers.rs` — File handlers
- `api/src/presentation/routes/server_routes.rs` — /:id/files/list, /:id/files/upload
- `web-agent/src/handlers/sftp.rs` — SFTP implementation

### Security
- `api/src/domain/rbac/middleware.rs` — Permission checks
- `api/src/domain/auth/middleware.rs` — Auth middleware

</canonical_refs>

<specifics>
## Specific Ideas

- RCON executor exists: RconServerExecutor
- Terminal WebSocket handler exists: ws_terminal
- File handlers exist: list_files, upload, download
- SFTP handlers in web-agent: sftp.upload, sftp.download

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 08-operations-integration*
*Context gathered: 2026-04-09*
