---
status: verifying
trigger: "agent-not-appearing-in-dashboard - Agent running on Windows PC not appearing in Escluse web dashboard, but showing as 'online' in local GUI (escluse-gui)"
created: 2026-05-08T00:00:00Z
updated: 2026-05-08T00:00:00Z
---

## Current Focus

hypothesis: "Root cause identified and fix implemented"
test: "Verify agent now appears in dashboard after fix"
expecting: "Agent auto-registered via WebSocket will now be associated with authenticated user's tenant and appear in dashboard"
next_action: "Human verification - test by running agent and checking dashboard"

## Symptoms

expected: Agent harus muncul di dashboard web (app.esluce.com) agar bisa dikelola dari sana
actual: Agent hanya muncul "online" di escluse-gui (GUI lokal), tidak muncul di dashboard web
errors: Tidak ada error di terminal saat menjalankan agent
reproduction: Jalankan agent di Windows,cek dashboard tidak muncul node baru
started: Setelah install dan jalankan agent, tidak muncul di dashboard

## Eliminated

- hypothesis: "Agent not connecting to backend WebSocket"
  evidence: "GUI shows 'online' = connected: true, which requires agent_id from backend RegisterAck"
  timestamp: "2026-05-08T00:00:00Z"

- hypothesis: "Different port or URL configuration"
  evidence: "WebSocket URL is correctly constructed to backend_url/api/ws/node"
  timestamp: "2026-05-08T00:00:00Z"

## Evidence

- timestamp: "2026-05-08T00:00:00Z"
  checked: "escluse-gui/src/ipc.rs + escluse-agent/src/api/routes.rs"
  found: "GUI 'online' status checks agent's local API at http://127.0.0.1:8642 which returns connected: true only when agent has received node_id from backend (via WebSocket RegisterAck)"
  implication: "Agent IS successfully connecting to backend WebSocket and getting registered"

- timestamp: "2026-05-08T00:00:00Z"
  checked: "api/src/presentation/handlers/node_ws_handler.rs + node_handlers.rs"
  found: "When agent registers via WebSocket, it creates node with user_id: None (line 111-177). When listing nodes (line 111-128), query filters by user.tenant_id - nodes with user_id=None don't match any tenant"
  implication: "ROOT CAUSE: Auto-registered nodes lack tenant association and are filtered out"

## Resolution

root_cause: "When agents auto-register via WebSocket (node_ws_handler.rs), they create nodes in the database WITHOUT setting the user_id/tenant_id. When users view the dashboard, list_nodes queries find_by_user_id(tenant_id) which excludes nodes with user_id=null. Even when authenticated via API key, the tenant_id was not propagated to the node."

fix: "1. node_ws_handler.rs: Capture user_id from authenticated node at registration time and set it on new nodes. Also associate existing unassigned nodes with authenticated user's tenant when re-registering. 2. node_handlers.rs: Update list_nodes to include both user's nodes AND unassigned nodes (user_id=null), so auto-registered nodes are visible even if user already has other nodes."

files_changed:
  - "api/src/presentation/handlers/node_ws_handler.rs"
  - "api/src/presentation/handlers/node_handlers.rs"

verification: "Code compiles successfully. Need human verification: run agent and check dashboard shows the node."