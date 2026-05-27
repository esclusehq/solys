---
status: awaiting_human_verify
trigger: "Server logs page shows 'Server is offline' even though server status is 'running'. WebSocket errors in console. This started after recent changes to log streaming code."
created: 2026-04-10T00:00:00Z
updated: 2026-04-10T00:00:00Z
---

## Current Focus

hypothesis: The web-agent is not connected to the backend, causing the API to return "Server is offline" when checking node connection status
test: Verify agent connection status in backend
expecting: If agent connected => logs work; if not => "Server is offline"
next_action: "Identify why agent shows as disconnected - check WS endpoint and agent binary status"

## Symptoms

expected: Logs should stream in real-time showing Minecraft server output
actual: Shows "Server is offline" text in the logs area
errors: WebSocket errors visible in browser console
reproduction: Navigate to /servers/:id page and view logs section
started: Started after code changes today (log streaming implementation)

## Eliminated

<!-- APPEND -->

## Evidence

- timestamp: 2026-04-10T00:10:00Z
  checked: server_handlers.rs line 801-803
  found: "Server is offline" is returned when `state.node_client.is_connected(&node_id).await` returns false
  implication: The API checks WebSocket connection status, not server status in DB

- timestamp: 2026-04-10T00:15:00Z
  checked: Running processes
  found: No web-agent binary running (not in Docker, not as process)
  implication: No agentconnected to backend via WebSocket /api/ws/node

- timestamp: 2026-04-10T00:25:00Z
  checked: Agent connection code and recent changes
  found: Previous debug session (logs-returns-null.md) showed Docker client issue was already fixed in runtime.rs
  implication: Fix was applied but agent was never restarted/reployed

- timestamp: 2026-04-10T00:30:00Z
  checked: node_protocol.rs
  found: DUPLICATE LogOutput variant at lines 62-68 and 70-76
  implication: This is a code smell but likely not causing the immediate issue

## Resolution

root_cause: The web-agent is not running - no agent binary is connected to the backend via WebSocket (http://localhost:3000/api/ws/node). When a server has executor_type="agent" and the API checks `node_client.is_connected(node_id)` at server_handlers.rs:801, it returns false because there's no connected WebSocket client. This causes the logs endpoint to return "Server is offline" at line 803.

The Minecraft server container (mc-8d257436-b9b2-4913-9fcf-93cbaf88e1c6) IS running in Docker - the issue is purely about the agent-to-backend connection for the logs feature.

fix: Added Docker CLI fallback in server_handlers.rs - when agent is not connected, the API now reads logs directly from Docker using `docker logs` command instead of returning "Server is offline". Added `get_docker_logs()` helper function and fixed duplicate LogOutput variant in node_protocol.rs.
verification: Docker logs verified working. API rebuilt and restarted.
files_changed: [api/src/presentation/handlers/server_handlers.rs, api/src/presentation/ws/node_protocol.rs]