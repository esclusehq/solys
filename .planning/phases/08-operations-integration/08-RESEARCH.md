# Phase 8: Operations Integration — Research Findings

**Date:** 2026-06-04
**Status:** Complete

---

## 1. RCON Status

### 1.1 `RconServerExecutor` (`api/src/infrastructure/executors/rcon_server_executor.rs`)

**Fully implemented.** The executor can:
- `connect()` to RCON via the `rcon` crate (connects to `server.host:rcon_port`)
- `send_command()` — sends arbitrary commands via RCON protocol. Has a `say` → `tellraw` alias.
- `check_status()` — sends `list`, returns `"running"` or `"stopped"` based on TCP connect success.
- `collect_metrics()` — sends `list` (player count) and `tps` (Paper/Spigot TPS), returns `ServerMetrics`. CPU/memory/disk are set to 0 (not available via RCON).
- `stop_server()` — sends `stop` via RCON.
- `restart_server()` — calls `stop_server()` (relies on Docker auto-restart policy).
- `create_server()`, `delete_server()` — warns as RCON cannot manage containers.

**Note:** `rcon_port = server.port + 10`, `rcon_password = server.password_auth`.

### 1.2 Terminal Handlers (`api/src/presentation/handlers/terminal_handlers.rs`)

Three endpoints exist:

| Function | Type | Path | What it does |
|---|---|---|---|
| `ws_terminal` | WebSocket | `GET /ws/terminal/:id` | Real-time terminal via `docker exec` / `podman exec` into container `mc-{server_id}`. Uses `docker exec sh -c` (NOT rcon-cli). Has Redis-backed command history. |
| `exec_terminal` | REST POST | `POST /api/servers/:id/terminal/exec` | Same as WS but REST-based. Runs via container exec. |
| `exec_rcon` | REST POST | `POST /api/servers/:id/rcon` | Runs `docker exec <container> rcon-cli --password <pw> <command>`. Reads `rcon.port`, `rcon.password`, `enable-rcon` from `/data/server.properties` inside container. |

**Key distinction:** The WebSocket terminal (`ws_terminal`) uses `docker exec sh -c` (shell commands inside the container), NOT the RCON protocol. The `exec_rcon` handler is the one that actually uses `rcon-cli` inside the container.

### 1.3 Routes (`api/src/presentation/routes/server_routes.rs`)

The routes file has TWO sets of routes:
- **server_routes.rs routes** (uses `AppContainer`): `/api/servers/:id/terminal/exec`, `/ws/terminal/:id`
- **server_handlers.rs internal router** (uses `ApiState`): `/:id/command` (→ `terminal_handlers::exec_terminal`), `/:id/rcon` (→ `terminal_handlers::exec_rcon`)

The route `/:id/command` was **fixed in Plan 06** to point to `terminal_handlers::exec_terminal` (previously pointed to `server_handlers::send_command` which required `remote_id`).

### 1.4 Frontend Terminal Components

Three components exist:

| Component | Location | Description |
|---|---|---|
| `TerminalPanel.jsx` | `app/src/components/IDE/TerminalPanel.jsx` | WS-based terminal, connects to `/ws/terminal/:id`. Has history navigation, clear button, connected status indicator. Use in IDE panel context. |
| `Terminal.jsx` | `app/src/components/Terminal.jsx` | Full xterm.js terminal emulator. Connects to `/ws/terminal/:id`. Has full autocomplete, reconnect with exponential backoff, local history in localStorage. Used in ServerDetails page. |
| `Console.jsx` | `app/src/pages/Console.jsx` | Standalone console page. Uses REST `sendCommand` (→ `/:id/command`). Also streams docker container logs via `/ws/docker-logs`. |

### 1.5 Plan 06 Gap Fix Summary

Plan 06 fixed the **remote_id gap**: agent-based servers had no `remote_id`, so old `server_handlers::send_command` (which used `solys_client.send_command(&remote_id, ...)`) would fail with "Server has no remote_id". The fix:
- Route `/:id/command` now points to `terminal_handlers::exec_terminal` which runs `docker exec` (no remote_id required)
- File operations route through `file_handlers.rs` (direct filesystem or `docker exec ls -la`)

**This means the terminal is NOT using RCON for most operations—it's using docker exec.** RCON (`exec_rcon`) is a separate endpoint.

---

## 2. File Management Status

### 2.1 File Handlers (`api/src/presentation/handlers/file_handlers.rs`)

**Fully implemented.** All CRUD operations exist:

| Function | Route | What it does |
|---|---|---|
| `list_files` | `GET /:id/files` | Lists directory. For agent servers: routes through agent WS. For others: `docker exec ls -la` inside container. |
| `read_file` | `GET /:id/files/read` | Reads text file (max 5MB). Agent routing supported. |
| `write_file` | `PUT /:id/files/write` | Writes text content. Agent routing supported. |
| `download_file` | `GET /:id/files/download` | Streams binary file download. Direct filesystem only (no agent routing). |
| `delete_path` | `DELETE /:id/files` | Deletes file or directory. Agent routing supported. |
| `mkdir` | `POST /:id/files/mkdir` | Creates directory. Agent routing supported. |
| `rename_path` | `POST /:id/files/rename` | Renames/moves. Agent routing supported. |
| `copy_path` | `POST /:id/files/copy` | Copies file/dir. Agent routing supported. |
| `compress_path` | `POST /:id/files/compress` | Creates ZIP archive. No agent routing. |
| `extract_path` | `POST /:id/files/extract` | Extracts ZIP/tar.gz. No agent routing. |
| `upload_file` | `POST /:id/files/upload` | Multipart file upload. No agent routing. |
| `upload_chunk` | `POST /:id/files/upload/chunked` | Chunked upload with resume. Base64 encoded chunks. |
| `get_upload_status` | `GET /:id/files/upload/status/:filename` | Returns received chunks for resume. |

**Security:** `get_secure_path()` blocks `..` traversal, verifies ownership via `get_server_use_case.execute()`, and canonicalizes paths.

### 2.2 Agent File Operations

For agent-mode servers, file operations route through `route_file_through_agent()` which sends the command via `NodeClient` to the remote agent. The agent's `files.rs` handler processes `file.list_dir`, `file.read_file`, etc.

For non-agent servers (legacy EC2 backend), operations use `docker exec` inside the container or direct filesystem access.

### 2.3 SFTP (`agent/solys/src/handlers/sftp.rs`)

SFTP exists **in the agent** (`agent/solys/src/handlers/sftp.rs`), NOT in `web-agent`. It provides:
- `sftp.upload` — Uploads a file via SSH/SFTP using an SSH connection stored in cache
- `sftp.download` — Downloads a file via SSH/SFTP

**Key: SFTP requires an SSH connection established via `ssh.connect` first** (SSH credentials cached by connection key). The SFTP handlers are NOT wired through the API file handlers. There is NO frontend SFTP upload/download path. The API file handlers use direct filesystem, docker exec, or the agent WebSocket task protocol—NOT SFTP.

### 2.4 Frontend File Browser

**`FileManager.jsx`** (`app/src/components/FileManager.jsx`, 1006 lines) is a full-featured file browser:
- List/tree view toggle (plan 08-02)
- Lazy-loaded tree view with expand/collapse
- File upload (regular + chunked with progress bar)
- File download
- Inline editing for text files
- Delete, rename, copy, move, compress, extract
- Drag-and-drop upload
- Search
- Context menu (right-click)
- Used in `ServerDetails.jsx`, `ServerDetailsPage.jsx`, and `WebIDE.jsx`

---

## 3. Existing Plans (08-01 through 08-06)

### Plan 08-01: RCON Command Execution
- **Goal:** Verify RCON executor is wired to use cases, ensure command endpoint exists, add Redis command history
- **Files modified:** `terminal_handlers.rs`, `send_command_use_case.rs`, `rcon_server_executor.rs`
- **Dependencies:** none
- **Status:** COMPLETE per summaries. However `send_command_use_case.rs` does NOT exist in the codebase. The route `/:id/command` now points directly to `terminal_handlers::exec_terminal`.

### Plan 08-02: File Browser with Tree View
- **Goal:** Verify file handlers, add tree view UI
- **Files modified:** `file_handlers.rs`, `FileManager.jsx`
- **Dependencies:** none
- **Status:** COMPLETE. Tree view toggle, lazy loading implemented in `FileManager.jsx`.

### Plan 08-03: Chunked Upload with Resume
- **Goal:** Add chunked upload endpoint with resume, update UI
- **Files modified:** `file_handlers.rs`, `FileManager.jsx`
- **Dependencies:** none
- **Status:** COMPLETE. `upload_chunk`, `get_upload_status`, and base64 decode added.

### Plan 08-04: Path Security Verification
- **Goal:** Verify `get_secure_path` implementation
- **Files modified:** `file_handlers.rs`
- **Dependencies:** none
- **Status:** COMPLETE. Security model verified.

### Plan 08-05: WebSocket Terminal Integration
- **Goal:** Verify WS terminal, add history navigation
- **Files modified:** `terminal_handlers.rs`, `TerminalPanel.jsx`
- **Dependencies:** none
- **Status:** COMPLETE. Command history from Redis, history navigation arrows.

### Plan 08-06: Gap Closure (Agent-based servers)
- **Goal:** Fix operations for agent-based servers (no remote_id)
- **Files modified:** `file_handlers.rs` (added `list_dir_via_docker`), route config
- **Dependencies:** none
- **Status:** COMPLETE. Routes fixed to point to `terminal_handlers::exec_terminal` and `file_handlers`.

---

## 4. Gaps: What's NOT Implemented

### 4.1 RCON-Specific Gaps

| Gap | Details |
|---|---|
| `send_command_use_case.rs` | Referenced in plan but does NOT exist in codebase. Command execution goes directly from route to handler. |
| RCON-via-agent | `AgentServerExecutor::send_command()` sends `command` as raw task type to agent—the agent handles it. But the agent's `rcon.rs` handler expects an RCON connection, while the standard terminal goes through `docker exec`. This path may conflict. |
| No RCON WebSocket | The WS terminal (`ws_terminal`) uses `docker exec sh -c`, NOT RCON protocol. There's no RCON-specific WS for real-time MC console output. |
| No RCON connection pooling | Each command opens a new TCP connection. |
| No auth to RCON | The RCON password is stored in `server.password_auth` and `server.properties`. No mechanism to update RCON password when server settings change. |

### 4.2 SFTP-Specific Gaps

| Gap | Details |
|---|---|
| SFTP NOT wired through API | SFTP handlers exist in the agent (`agent/solys/src/handlers/sftp.rs`) but there is NO API endpoint, NO route, and NO frontend path that uses SFTP for file operations. |
| SFTP requires SSH pre-connection | SFTP flow requires: `ssh.connect` → establish SSH session → `sftp.upload/download`. The API file handlers use direct filesystem or docker exec instead. |
| No `web-agent/src/handlers/sftp.rs` | The path referenced in plans does not exist. SFTP lives in `agent/solys/`. |
| No frontend SFTP component | FileManager uses REST API endpoints, not SFTP. |

### 4.3 Architectural Gaps

| Gap | Details |
|---|---|
| Dual route systems | Two separate route systems exist: `server_routes.rs` (under `AppContainer`) and `server_handlers.rs` internal router (under `ApiState`). These have different route paths (`/api/servers/:id/` vs `/:id/`). |
| No executor integration for file ops | File handlers bypass the executor pattern entirely—they operate directly on the filesystem or via docker exec. Only the agent-mode path goes through `NodeClient`. |
| Console.jsx uses REST, not WS | The Console page uses `sendCommand()` REST endpoint. TerminalPanel.jsx and Terminal.jsx use WebSocket. This means Console.jsx doesn't get real-time streaming output. |

### 4.4 What IS Implemented vs Planned

| Planned Feature | Status | Reality |
|---|---|---|
| RCON commands via REST | ✅ | `POST /:id/rcon` → `exec_rcon` → `docker exec rcon-cli` |
| RCON commands via WS | ⚠️ Partial | WS terminal exists but uses `docker exec sh -c`, not RCON protocol |
| Command history in Redis | ✅ | Implemented in `terminal_handlers.rs` |
| File browser tree view | ✅ | Implemented in `FileManager.jsx` |
| Chunked upload with resume | ✅ | Implemented in `file_handlers.rs` |
| Path security | ✅ | `get_secure_path` blocks traversal, checks ownership |
| Agent-based server support | ✅ | Files + terminal fixed in Plan 06 |
| SFTP file management | ❌ | SFTP exists in agent but NOT wired through API/frontend. Context.md mentions `web-agent/src/handlers/sftp.rs` which doesn't exist. |

---

## 5. Key Architectural Patterns

### Executor Pattern

```
ServerExecutor trait (domain)
  ├── RconServerExecutor — RCON protocol (minecraft/rcon executor_type)
  ├── AgentServerExecutor — routed through NodeClient WebSocket to remote agent
  ├── PodmanServerExecutor — local podman commands
  ├── SshServerExecutor — SSH into remote host
  └── MockServerExecutor — test mock
```

Dispatch via `SimpleExecutorFactory::get_executor()`:
- `"minecraft" | "rcon"` → `RconServerExecutor`
- `"agent"` → `AgentServerExecutor` (requires `NodeClient`)
- `"podman"` → `PodmanServerExecutor`
- other → `SshServerExecutor`

### WebSocket Handler Pattern

Two WS architectures:
1. **Event bus WS** (`ws_handler.rs`): Long-lived WS at `/ws`. Auth via JWT cookie. Streams `ServerEvent`s (StatusChanged, MetricsUpdated, AlertTriggered, etc.). Used for real-time dashboard updates.
2. **Terminal WS** (`terminal_handlers.rs`): Short-lived WS at `/ws/terminal/:id`. Direct command/response pattern. No event bus subscription.

### File Operation Flow

```
Frontend ←REST→ file_handlers.rs
  ├── agent executor_type → route_file_through_agent() → NodeClient → Agent WS → agent handlers (files.rs)
  └── other → docker exec ls -la OR direct filesystem access
```

### Monitoring Service RCON Usage

`monitoring_service.rs` uses `ExecutorFactory` → `RconServerExecutor` (or configured executor) to:
- `check_status()` — every 30s loop
- `collect_metrics()` — for sleep detection and RCON health check (Phase 57)
- `stop_server()` — for sleep/inactivity timeout

---

## 6. Recommended Planning Focus

Based on these findings, the remaining work for Phase 8 should prioritize:

1. **SFTP wiring** — Connect the existing SFTP handlers in the agent to the API. Add API endpoints and frontend path. The handlers exist but are completely disconnected.

2. **RCON WebSocket** — Consider adding a true RCON protocol WebSocket for real-time Minecraft console output (vs current docker exec approach).

3. **Unify route systems** — The dual `ApiState`/`AppContainer` route setup is confusing and may cause duplication.

4. **Executor integration for files** — File operations could theoretically use the executor pattern (e.g., `SshServerExecutor` could handle file ops via SFTP), but currently they're independent.
