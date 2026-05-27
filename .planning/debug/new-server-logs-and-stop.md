---
status: awaiting_human_verify
trigger: "Logs don't appear after starting a new Minecraft server, and the Stop button doesn't work."
created: 2026-04-10T00:00:00Z
updated: 2026-04-10T00:00:00Z
---

## Current Focus

hypothesis: Server has no node_id assigned, so there's no agent to handle logs requests or stop commands
test: Check how servers get node_id assigned during creation, verify API handlers check for node_id before sending commands
expecting: If node_id is missing, that's why logs/stop don't work - need to fix server creation or API handlers
next_action: Apply fix to get_logs and stop_server handlers to auto-find connected node when node_id is missing

## Symptoms

expected: Logs should appear in real-time, Stop button should stop the server
actual: Logs page shows "Waiting for logs...", Stop button does nothing (no action)
errors: No visible errors in UI
reproduction: Created new Minecraft server, started it, went to server details page, logs show nothing, clicked Stop button
started: Issue started after creating a new server (server name: "test2", id: b9f39b89-a311-4c14-b013-82a7088071f1)

## Eliminated

-

## Evidence

- timestamp: 2026-04-10T00:00:00Z
  checked: Database server table
  found: Server "test2" (id: b9f39b89-a311-4c14-b013-82a7088071f1) is in "running" status, node_id is NULL/empty
  implication: No agent on any node is handling this server

- timestamp: 2026-04-10T00:00:00Z
  checked: Database server table (comparison server)
  found: Server "test" (id: 8d257436-b9b2-4913-9fcf-93cbaf88e1c6) has node_id: 65b3928f-fb19-4215-ba88-afba10a21dd6 and logs/stop work
  implication: Working servers have node_id assigned

- timestamp: 2026-04-10T00:00:00Z
  checked: Web-agent connection
  found: Web-agent is connected to API (WebSocket active)
  implication: Agent infrastructure works, but server not assigned to any node

- timestamp: 2026-04-10T00:00:00Z
  checked: Docker container for test2
  found: Container mc-b9f39b89-a311-4c14-b013-82a7088071f1 exists with status "Created" (not running properly)
  implication: Container created but not properly started on an agent

## Resolution

root_cause: |
  When servers are created without specifying a node_id, the server creation process does not auto-assign a node.
  The start_server handler correctly auto-assigns a connected node and saves the node_id to the database.
  However, get_logs and stop_server handlers don't have this auto-assignment logic - they immediately fail with "Server has no node_id" error.
  
  The test2 server was started successfully and shows "running" status, BUT the start_server code updated the server.status without updating node_id in certain code paths (or there was a race condition).
  Either way, now get_logs/stop see node_id as NULL and fail silently.
  
  More likely: The start handler DID auto-assign a node, but there's a timing issue or the update didn't persist properly, leaving node_id as NULL.

fix: |
  1. In get_logs handler: Add logic to find a connected node if node_id is missing (similar to start_server logic) - DONE
  2. In stop_server handler: Add logic to find a connected node if node_id is missing - DONE
  3. In restart_server handler: Add logic to find a connected node if node_id is missing - DONE
  4. In stream_logs handler: Add logic to find a connected node if node_id is missing - DONE
  5. In delete_server handler: Add logic to find a connected node if node_id is missing - DONE
  
  All handlers now find an online node and persist the node_id to the database when the server doesn't have one assigned.

verification: |
  After fix: Create a new server without node_id, start it, then test logs and stop buttons. Should work.

files_changed: 
- api/src/presentation/handlers/server_handlers.rs