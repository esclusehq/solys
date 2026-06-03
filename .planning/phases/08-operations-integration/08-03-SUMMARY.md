---
phase: 08-operations-integration
plan: 03
subsystem: api, frontend, agent
tags: sftp, ssh, file-transfer, agent, multipart, websocket

# Dependency graph
requires:
  - phase: 08-operations-integration
    provides: Context (RESEARCH.md, CONTEXT.md, agent sftp.rs handlers, existing file handler routes)
provides:
  - Agent dispatches sftp.upload and sftp.download task types from execute_command messages
  - REST API endpoints at /:id/files/sftp-upload and /:id/files/sftp-download
  - Frontend SFTP upload/download buttons and modal in FileManager.jsx
  - SFTP operations route through agent WebSocket to agent/solys/src/handlers/sftp.rs handlers
affects:
  - Future plans that need file transfer via SSH/SFTP for agent-mode servers

# Tech tracking
tech-stack:
  added: []
  patterns:
    - SFTP operations gated on executorType === 'agent' in frontend
    - Temp file cleanup pattern (immediate for upload, delayed 60s for download)
    - Dynamic import for useServers functions in FileManager.jsx to avoid circular deps

key-files:
  created: []
  modified:
    - api/src/presentation/ws/node_protocol.rs — CommandParams with SFTP fields
    - agent/solys/src/agent_connection.rs — SFTP dispatch + payload construction
    - api/src/presentation/handlers/file_handlers.rs — sftp_upload_file, sftp_download_file handlers
    - api/src/presentation/handlers/server_handlers.rs — SFTP route registrations
    - app/src/hooks/useServers.js — sftpUploadFile, sftpDownloadFile API functions
    - app/src/components/FileManager.jsx — SFTP buttons and modal UI

key-decisions:
  - "SFTP buttons gated on executorType === 'agent' in FileManager (non-breaking prop)"
  - "Dynamic import() for useServers functions in FileManager to avoid circular dependencies"
  - "Upload file written to temp dir first, routed through agent, then cleaned up immediately"
  - "Download file cleaned up after 60s via fire-and-forget tokio::spawn"

requirements-completed:
  - FILE-01  # User can browse server files via SFTP (agent-routed list_files)
  - FILE-02  # User can upload files to server via SFTP agent task
  - FILE-03  # User can download files from server via SFTP agent task

# Metrics
duration: 3 min
completed: 2026-06-03
---

# Phase 08: Operations Integration — Plan 03 Summary

**SFTP wiring through agent: CommandParams extended, agent dispatch added, API endpoints registered, and frontend SFTP upload/download UI with executerType gating**

## Performance

- **Duration:** 3 min
- **Started:** 2026-06-03T19:01:50Z
- **Completed:** 2026-06-03T19:05:29Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Extended `CommandParams` on both API and agent sides with `connection_key`, `local_path`, `remote_path` fields (all `Option` — backwards compatible)
- Added `sftp_upload` → `sftp.upload` and `sftp_download` → `sftp.download` dispatch entries in `agent_connection.rs` task_type match
- Added SFTP payload construction (`connection_key`, `local_path`, `remote_path`) in `execute_command`
- Created `route_sftp_through_agent()` helper that routes SFTP operations through agent WebSocket for agent-mode servers
- Created `sftp_upload_file` handler (multipart form → temp file → agent → cleanup)
- Created `sftp_download_file` handler (JSON request → agent → temp file → stream response → delayed cleanup)
- Registered `POST /:id/files/sftp-upload` and `POST /:id/files/sftp-download` routes in server_handlers.rs router
- Added `sftpUploadFile` and `sftpDownloadFile` API client functions to `useServers.js`
- Added SFTP Upload/Download buttons to FileManager toolbar (gated on `executorType === 'agent'`)
- Added SFTP modal dialog in FileManager with connection key input, remote path input, and file selector

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend CommandParams with SFTP fields + add agent dispatch** — `api@c54d461`, `agent/solys@0c69b87`
2. **Task 2: Add SFTP upload/download API handlers + routes** — `api@918c495`
3. **Task 3: Add frontend SFTP functions and FileManager UI** — `app@0597c78`

## Files Created/Modified

- `api/src/presentation/ws/node_protocol.rs` — Added `connection_key`, `local_path`, `remote_path` to `CommandParams`
- `agent/solys/src/agent_connection.rs` — Added SFTP fields to agent `CommandParams`, dispatch entries, payload construction
- `api/src/presentation/handlers/file_handlers.rs` — Added `SftpDownloadRequest`, `route_sftp_through_agent()`, `sftp_upload_file()`, `sftp_download_file()`
- `api/src/presentation/handlers/server_handlers.rs` — Registered `/:id/files/sftp-upload` and `/:id/files/sftp-download` routes
- `app/src/hooks/useServers.js` — Added `sftpUploadFile()` and `sftpDownloadFile()` exports
- `app/src/components/FileManager.jsx` — Added SFTP state, buttons (gated), and modal dialog

## Decisions Made

- **SFTP buttons gated on executorType:** Added optional `executorType` prop to FileManager — SFTP buttons only render when `executorType === 'agent'`. Existing callers pass only `serverId` (backward compatible).
- **Dynamic import for SFTP functions:** Used `import('../hooks/useServers')` inside onClick handlers rather than static imports, avoiding circular dependency issues.
- **Temp file cleanup:** Upload temp files removed immediately after agent confirms transfer. Download temp files cleaned up after 60s via fire-and-forget `tokio::spawn`.
- **API-level security:** `route_sftp_through_agent` checks `server.executor_type != "agent"` and verifies server ownership via `get_server_use_case.execute()` before any SFTP operation.

## Deviations from Plan

None — plan executed exactly as written.

### Additional Implementation Details

- **SFTP button gating (checker warning):** The plan mentioned the note about gating on `server.executorType === 'agent'`. Implemented via optional `executorType` prop — non-breaking for existing callers.
- **SFTP upload/download API pattern:** Upload uses multipart form (file + text fields), download uses JSON request body with binary stream response — matching the existing file handler conventions.

## Threat Surface Scan

No new security surface beyond what's modeled in the plan's threat register. All SFTP endpoints verify server ownership (`get_server_use_case.execute()`) and executor type before routing to agent. The `connection_key` is system-passed (user-provided — matches ssh.connect pattern already in the codebase). Temp file paths are non-user-controlled.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- FILE-01 (file browsing via agent) — already works via existing `list_files` → agent `file.list_dir`
- FILE-02 (upload via SFTP) — fully wired: frontend → API → agent → sftp.rs handler
- FILE-03 (download via SFTP) — fully wired: frontend → API → agent → sftp.rs handler
- All three SFTP requirements for Phase 08 completed
- Ready for next phase (Phase 09 or further operations integration tasks)

## Self-Check: PASSED

- ✅ SUMMARY.md exists at `.planning/phases/08-operations-integration/08-03-SUMMARY.md`
- ✅ Task 1 commits found: `api@c54d461` (feat), `agent/solys@0c69b87` (feat)
- ✅ Task 2 commit found: `api@918c495` (feat)
- ✅ Task 3 commit found: `app@0597c78` (feat)
- ✅ All 6 modified files exist at expected paths
- ✅ All 17 acceptance criteria verified with grep checks
- ✅ SFTP buttons gated on `executorType === 'agent'` (prop-based)
- ✅ Temp file cleanup implemented (immediate upload, delayed 60s download)
- ✅ All threat model mitigations present (ownership check, executor type gate, temp path isolation)

---

*Phase: 08-operations-integration*
*Completed: 2026-06-03*
