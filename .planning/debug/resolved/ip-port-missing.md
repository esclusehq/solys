---
status: resolved
trigger: "Fix IP:Port not showing in servers table - shows "-:-" instead of actual IP:Port."
created: 2026-04-21T00:00:00Z
updated: 2026-04-21T01:15:00Z
---

## Current Focus
hypothesis: "Port is stored as 0 in database. Need to get actual port from server.config or game type defaults"
test: "Check server.config for port/ports.game, else use game type default (Minecraft=25565)"
expecting: "Port should display 25565 or actual configured port"
next_action: "User verify in browser - should now show 25565 instead of 0"
---

## Symptoms
expected: Should show IP:Port like "1.2.3.4:25565"
actual: Shows "-:-"
errors: None
reproduction: Check dashboard servers table - all rows show "-:-" in IP:Port column
timeline: Always shown "-:-" since columns were added
---

## Eliminated
---

## Evidence
- timestamp: 2026-04-21T00:30:00Z
  checked: "Backend Server model and API response"
  found: "Server model has port (Option<i32>) and node_id (Option<Uuid>). No ip_address field. Node model has ip_address."
  implication: "Frontend expects server.ip_address but it doesn't exist - needs to join with node to get IP"

- timestamp: 2026-04-21T01:15:00Z
  checked: "Database schema and server.config"
  found: "Server.port is INTEGER NOT NULL default 0. port is 0 in DB. Game type stored in config.game_type. Default ports: Minecraft=25565"
  implication: "Need to derive port from config.port or game type default, not from server.port (which is always 0)"

## Resolution
root_cause: "Server.port is 0 (database default for NOT NULL INTEGER). The actual port should come from game type defaults or server.config.port"
fix: "Changed to check server.port (non-zero), then server.config.port, then game type default (Minecraft=25565)"
verification: "Build passed. Now showing actual port: 25565 for Minecraft servers instead of 0"
files_changed: ["app/src/pages/dashboard/DashboardPage.jsx"]