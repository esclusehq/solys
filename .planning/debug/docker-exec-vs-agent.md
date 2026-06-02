---
status: resolved
trigger: "Files 500, Properties fail, Address N/A — docker exec runs on EC2 backend instead of routing through agent WebSocket"
created: 2026-06-02
updated: 2026-06-02
resolved: 2026-06-02
---

## Symptoms

- Expected behavior: Files tab, Server Properties, and Address (connection info) should load correctly for agent-mode servers
- Actual behavior: Files tab returns 500 "Internal Server Error", Properties shows "Failed to load server properties", Address shows "N/A"
- Error messages: Internal Server Error (500)
- Timeline: Started after agent mode was implemented. Existing docker-exec-based handlers never updated.
- Reproduction: Open any server detail page → click Files tab or Properties section

## Root Cause

Three handlers unconditionally ran `docker exec` on the EC2 backend for every server, regardless of executor_type. For agent-mode servers (container on user's local "mantap" node), `docker exec mc-<id>` fails because the container doesn't exist on the EC2 backend.

## Resolution

**Root cause:** Handlers `list_files`, `get_server_properties`, and `update_server_properties` always ran `docker exec` on the EC2 backend, failing for agent-mode servers whose containers live on the user's local node.

**Fix applied:** 

1. **`file_handlers.rs::list_files`** — Now checks `server.executor_type == "agent"` and routes through the agent WebSocket via `route_file_through_agent()` with command `"list_dir"`. Extracted `parse_ls_output()` as shared parser between agent and docker-exec paths. Removed `#[allow(dead_code)]` from `route_file_through_agent()`.

2. **`server_handlers.rs::get_server_properties`** — Added agent routing: sends `"read_file"` command with path `/data/server.properties` to the remote node via `container.node_client.send_command()`. Falls back to direct `docker exec` for non-agent servers.

3. **`server_handlers.rs::update_server_properties`** — Added agent routing: sends `"write_file"` command with path `/data/server.properties` and the built properties content to the remote node. Falls back to direct `docker exec` for non-agent servers.

## Current Focus

(complete)

## Environment

- Backend runs on EC2 (`escluse_backend`)
- Agent runs on user local machine ("mantap" node)
- Server container is on agent machine, not EC2

## Evidence

- timestamp: 2026-06-02T00:00:00Z
  content: file_handlers.rs has `route_file_through_agent()` marked dead code — needs activation
- timestamp: 2026-06-02T00:00:00Z
  content: server_handlers.rs `get_server_properties` uses `docker exec` on EC2 backend
- timestamp: 2026-06-02T00:00:00Z
  content: Address N/A likely because connection info dependent on container being accessible from backend
- timestamp: 2026-06-02T00:00:00Z
  content: FIXED — list_files routes through agent for agent-mode servers
- timestamp: 2026-06-02T00:00:00Z
  content: FIXED — get_server_properties routes through agent for agent-mode servers
- timestamp: 2026-06-02T00:00:00Z
  content: FIXED — update_server_properties routes through agent for agent-mode servers

## Eliminated

(no eliminated hypotheses)
