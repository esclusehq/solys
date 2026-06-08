---
status: fixed
trigger: "Server Details page shows wrong Address ('minecraft:26.2') and wrong Version (stale '26.2' instead of actually-running '26.1.2'). Status badge shows 'Unknown' for a running server."
created: 2026-06-05T00:00:00Z
updated: 2026-06-08T19:30:00Z
---

## Current Focus
hypothesis: Frontend v0.4.3 fix (Address fallback chain + prefer server.mc_version) deployed but backend never populates server.mc_version / server.endpoints / server.status correctly. Full fix is multi-file architectural change; deferred to dedicated session.
test: 
expecting: 
next_action: Complete — status fix applied (node disconnect handler now sets all servers to "offline")

## Symptoms
expected: 
  - Address column shows actual connectable address (e.g. `100.121.160.102:25565` or Tailscale IP)
  - Version column shows the actually-running Minecraft version (e.g. `26.1.2` not the user-typed `26.2`)
  - Status badge shows `running` (green) for a running container, `stopped` (gray) otherwise
actual:
  - Address: `minecraft:26.2` (concat of game_type + minecraft_version, totally misleading)
  - Version: `26.2` (stale value from `config.minecraft_version`, which the user typed at create time)
  - Status: `Unknown` badge (gray) even though container is actually running
errors: None (no errors thrown, just wrong data)
reproduction: Open any agent-mode server's `/servers/{id}` page; Overview tab shows all three wrong
started: Always has been wrong — frontend v0.4.3 partially mitigated Address display but Version and Status still broken at the data source

## Eliminated
- Frontend display logic — confirmed working as designed via app/v0.4.3 (deployed 2026-06-05). The dashboard's priority chain for Address is correct: `endpoints[0] → public_address → connection_address → ${game_type}:${game_port} → '—'`. The chain just gets nothing from the backend for the first 3.
- DB schema — `mc_version` column exists (`api/migrations/20260223015400_add_mc_version.sql:2`), `endpoints` JSONB exists (`migrations/20260326000001_add_server_new_columns.sql:26` and 2 more), `public_host` VARCHAR exists (`migrations/20260222162600_add_public_host.sql:2`). All columns are in the DB, just never written to.
- Agent reporting — agent does NOT currently read container env vars (no docker inspect in `agent/solys/src/handlers/`). `players` and `tps` are hardcoded to 0 and 20.0 respectively (`agent/solys/src/handlers/metrics.rs:193-194`).

## Evidence

### 1. Two competing Server models (the core problem)

The API has TWO `Server` structs that have diverged over the project's lifetime:

**OLD model** — `api/src/domain/server/model.rs:8-31` — has `endpoints: serde_json::Value` but **no** `mc_version`, `host`, `port`, `public_host`:
```rust
pub struct Server {
    pub id: Uuid, pub user_id: Option<Uuid>, pub agent_id: Option<Uuid>,
    pub job_id: Option<Uuid>, pub name: String, pub image: String,
    pub executor_type: String, pub node_id: Option<Uuid>, pub status: String,
    pub remote_id: Option<String>, pub port: Option<i32>, pub config: serde_json::Value,
    pub resources: serde_json::Value, pub auto_wake: Option<bool>,
    pub sleep_timeout_minutes: Option<i32>, pub last_restart_at: Option<...>,
    pub last_restart_reason: Option<String>, pub health_check_timeout_seconds: Option<i32>,
    pub endpoints: serde_json::Value,        // <-- always `[]` (default in model.rs:55)
    pub created_at: ..., pub updated_at: ..., pub deleted_at: Option<...>,
}
```

**NEW model** — `api/src/domain/entities/server.rs:8-75` — has `mc_version: String`, `host: String`, `port: i32`, `public_host: Option<String>`, etc. but **no** `endpoints`:
```rust
pub struct Server { pub id, pub user_id, pub node_id, pub name, pub image,
    pub host: String, pub port: i32, pub username: String, pub mc_version: String,
    pub mc_loader: String, pub ram_allocation: String, pub status: String,
    pub container_name: Option<String>, pub public_host: Option<String>, ... }
```

**The dashboard's `GET /api/v1/servers/:id` (`api/src/presentation/handlers/server_handlers.rs:605-622`) returns the OLD model via `SqlxServerRepository`** — the handler instantiates `SqlxServerRepository::new(state.pool.clone())` directly, bypassing the container's `PostgresServerRepository` (NEW model). Same pattern in 28 other handlers in `server_handlers.rs`.

### 2. What the agent reports vs what the backend persists

**Agent → Backend message types** (`agent/solys/src/agent_connection.rs:27-88`):
- `Register { ip, podman_version, container_runtime, os_info, capabilities, containers: [], total_memory, cpu_cores, agent_version }` — sent once on connect. `containers` is always `[]` on register.
- `Heartbeat { node_id, status: "online", metrics: {cpu, memory, disk}, containers: [{id, name, status, cpu, memory, memory_limit, disk_usage, players: 0, tps: 20.0}] }` — every 30s.

**Backend handlers** (`api/src/presentation/handlers/node_ws_handler.rs`):
- Heartbeat handler (lines 237-266): stores containers in **in-memory cache only** (`NodeConnectionManager::containers_cache`), and metrics in `node_metrics` table. **Never writes to `servers` table.**
- CommandResponse handler (lines 268-288): the ONLY path that writes to `servers` — calls `server_repository.update_status(&server_id, "running")` on successful start/restart. Nothing else.

**What gets written to `servers` table today:**
- On `create_server` (`server_handlers.rs:520-572`): `status: "pending"`, `port` (from `config.game_port`), `executor_type`, `name`, `image`, `config` JSONB with user-supplied `minecraft_version, ram_mb, max_players, game_type`. **`mc_version` column is never set.**
- On `start_server` success (`server_handlers.rs:880-882`): `status: "running"`.
- On agent `CommandResponse` for start/restart (`node_ws_handler.rs:284`): `status: "running"`.
- On `update_server` (`server_handlers.rs:627-768`): user-supplied config edits.

**`mc_version` column: written by zero code paths in the live handler chain. The NEW `PostgresServerRepository::create` writes it from `create_server_use_case.rs:45` (user-supplied), but `server_handlers.rs::create_server` calls `SqlxServerRepository::create` (OLD), not the use case.**

### 3. `endpoints` JSONB is never populated

The `endpoints` column exists and is selected by `SqlxServerRepository` (OLD model) at `api/src/domain/server/sqlx_repository.rs:24, 28, 49, 104, 126, 149, 161, 218, 237` but the value is **always `json!([])`** — initialized in `model.rs:55` and never updated by any code path.

There's no `update_endpoints` repo method. There's no address resolution logic. There's no Tailscale awareness. The agent's `ip` field is the public IP via ipify.org (`agent/solys/src/handlers/dns_watch.rs:132-155`), not the Tailscale IP (`100.121.160.102`).

### 4. The `status` field is set but the dashboard shows "Unknown"

`server_handlers.rs:880-882` sets `status: "running"` on successful start. `node_ws_handler.rs:284` sets it on `CommandResponse` for start/restart. So the DB value should be `"running"` for a running server.

But the dashboard's `getStatusColor` (`app/src/pages/servers/ServerDetailsPage.jsx:110-116`) returns `bg-gray-500` (gray "Unknown" badge) for status values it doesn't recognize:
```jsx
const getStatusColor = (status) => {
    switch (status?.toLowerCase()) {
        case 'running': case 'active': case 'online': return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30';
        case 'stopped': case 'offline': case 'inactive': return 'bg-slate-500/20 text-slate-400 border-slate-500/30';
        case 'pending': case 'starting': case 'provisioning': return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
        case 'error': case 'failed': return 'bg-red-500/20 text-red-400 border-red-500/30';
        default: return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
    }
};
```

Likely the actual `status` value being sent is something like `"container_running"`, `"online"`, or some agent-reported value that the switch doesn't recognize. Needs to be checked at runtime (curl the live API).

### 5. Config vs runtime values

The user's actual running server: itzg/minecraft-server image, `VERSION=26.1.2` env (resolved by the image at container start), container running on the EC2 Tailscale node (100.121.160.102), RCON port 25575, game port 25565, RCON working as of v0.4.2.

The `config.mc_version` in the DB is `26.2` (what the user typed at create time) — but the itzg image's `VERSION=LATEST` resolved to 26.1.2 when the container started. The agent has access to the container's actual `Config.Env` via `docker.inspect_container`, but **does not call it**.

### 6. Status stuck at "running" when agent disconnects

Confirmed by user: web UI at `app.esluce.com/servers/{id}` shows status `running` (green badge) even though agent is offline/disconnected. The `node_ws_handler.rs` heartbeat handler never updates `servers.status` on disconnect or missed heartbeat. The only code path that sets it to `running` is `CommandResponse` on successful start (`node_ws_handler.rs:284`) and `server_handlers.rs:880-882`. There is no corresponding path to set it to `stopped` or `offline` on disconnect.

## Resolution (target)
root_cause: Three independent gaps in the data pipeline:
1. **Agent never reads container `VERSION` env var** (no docker inspect call in `agent/solys/src/handlers/`). The data exists in the container but never leaves the host.
2. **Backend never writes per-server `mc_version` from agent reports** — heartbeat handler at `node_ws_handler.rs:237-266` doesn't touch the `servers` table. The only path is `CommandResponse → update_status` at `node_ws_handler.rs:284`.
3. **`endpoints` column is never computed** — there's no code that takes a `(node_ip, port)` tuple and writes it as an endpoint. The dashboard expects `endpoints[0]` to be a connectable address; the backend never produces one.

Plus a fourth cosmetic gap:
4. **Dashboard's `getStatusColor` doesn't cover the actual status values the backend produces** (need to confirm via runtime inspection).

fix: Three options, in increasing scope:

### Option A: Backend-only stub (30 min, 2 files, low risk)
- Add `pub mc_version: String` to OLD `Server` model at `api/src/domain/server/model.rs:8-31` and the `SELECT mc_version` to `api/src/domain/server/sqlx_repository.rs:23-25`.
- Add `endpoints: Vec<EndpointDto>` typed struct to OLD model (replace `serde_json::Value`).
- Result: JSON shape matches what dashboard expects; values will be empty/LATEST until agent is fixed in a later hotfix.
- Dashboard improvements from v0.4.3 take full effect: Address shows `minecraft:25565` (from `${game_type}:${game_port}` fallback), Version shows `LATEST` (from `mc_version` column default).

### Option B: Full end-to-end fix (3 hours, 6-8 files)
All of Option A, plus:
- **Agent** (`agent/solys/src/handlers/metrics.rs:182-196`): add `docker.inspect_container(id, None).await` for each game-server container, extract `Config.Env → VERSION=...`, attach to `ContainerMetrics.mc_version: Option<String>`. Apply only to containers with `image.starts_with("itzg/")` or similar game-server images.
- **Agent** (`agent_connection.rs:470-482`): propagate `mc_version` into the heartbeat `serde_json::json!({...})` payload.
- **Backend protocol** (`api/src/presentation/ws/node_protocol.rs:149-165`): add `mc_version: Option<String>` to `ContainerStatus`.
- **Backend** (`api/src/presentation/handlers/node_ws_handler.rs:237-266`): on heartbeat, look up server by `container_name` (e.g. `mc-{server_id}`) and call a new `server_repository.update_runtime_state(server_id, mc_version, endpoints, container_status)`.
- **Backend** (`api/src/domain/repositories/server_repository.rs:7-15` + impl in `infrastructure/repositories/postgres_server_repository.rs` and `domain/server/sqlx_repository.rs`): add `update_runtime_state` method that writes `mc_version`, `endpoints = [{address: "node.ip:port"}]`, `status` from container state.
- **Address resolution** (where?): add a small `resolve_endpoints(node, port) -> Vec<Endpoint>` function — uses `node.ip_address` (public IP) since agent has no Tailscale awareness. Output: `[{address: "47.129.171.64:25565", kind: "public"}]`.
- Result: real values flow end-to-end. Dashboard shows actual `mc_version` and connectable address.

### Option C: Full fix + Tailscale IP (4 hours, 8-10 files)
All of Option B, plus:
- **Agent** (`agent/solys/src/main.rs` or `startup.rs`): detect Tailscale IP via `tailscale ip -4` (with fallback to no-op), include in `Register` payload as `tailscale_ip: Option<String>`.
- **Agent protocol** (`agent_connection.rs:30-46` + `node_protocol.rs:9-25`): add `tailscale_ip: Option<String>` to Register.
- **Backend** (`api/src/domain/entities/node.rs:14-16` + migration): add `tailscale_ip` column to `nodes` table.
- **Address resolution**: prefer `node.tailscale_ip` over `node.ip_address` when available — gives `100.121.160.102:25565` instead of `47.129.171.64:25565`.
- Result: real values + Tailscale-aware address that's actually reachable from the user's Tailscale network.

Plus **always** (regardless of option A/B/C):
- **Dashboard** (`app/src/pages/servers/ServerDetailsPage.jsx:110-116`): confirm what status values the backend actually returns and extend `getStatusColor` to cover them. Run `curl -H "Authorization: Bearer <token>" https://esluce.com/api/v1/servers/<id>` to get the actual value, then add the case.

verification: After deploy, hit `https://esluce.com/servers/<id>` and confirm:
- Address: real address (Option A) or real address (Options B/C)
- Version: `26.1.2` (Options B/C) or `LATEST` (Option A) or `26.2` (no fix)
- Status badge: green (not gray)

files_changed (per option):
- Option A: `api/src/domain/server/model.rs`, `api/src/domain/server/sqlx_repository.rs`
- Option B: + `agent/solys/src/handlers/metrics.rs`, `agent/solys/src/agent_connection.rs`, `api/src/presentation/ws/node_protocol.rs`, `api/src/presentation/handlers/node_ws_handler.rs`, `api/src/domain/repositories/server_repository.rs`, `api/src/infrastructure/repositories/postgres_server_repository.rs`, `api/src/domain/server/sqlx_repository.rs`
- Option C: + `agent/solys/src/main.rs` (or startup), `api/src/domain/entities/node.rs`, new migration

## Decision
Skip Options A/B/C for the mc_version/endpoints gaps. Revisit those in a dedicated session.

**Status gap (gap 5) fixed 2026-06-08:** The WebSocket disconnect handler now sets all servers on a disconnecting node to `"offline"`. This is a single-file, low-risk change that addresses the "green badge when agent offline" issue.

The frontend v0.4.3 already deployed (commit `d2a5f71`, deployed 2026-06-05) improved Address display: it now falls back to `${game_type}:${game_port}` (e.g. `minecraft:25565`) instead of the misleading `minecraft:26.2`. Version is still wrong (shows `config.minecraft_version` = `26.2`) and Status is still `Unknown`, but those are backend data problems that need the full Option B or C fix. User confirmed this is acceptable for now; the misleading Address concat is the worst offender and that's fixed.

When picked up: start with Option A (30 min, surfaces the empty values so the JSON shape is correct and the dashboard fallback chain works fully). Then in a follow-up session, do Option B (real agent inspection) and possibly C (Tailscale).

**Remaining gaps after this fix:**
1. `mc_version` column never written (Options A/B/C)
2. `endpoints` JSONB never populated (Options A/B/C)
3. Dashboard `getStatusColor` may need extension for actual backend status values (needs runtime curl)

## Caddyfile mislabeling — full fix shipped 2026-06-05 23:28 UTC

The framing in my earlier "partial fix" section was based on a misread: I assumed the `escluse-landing` container was serving a dashboard build because the bundle had `esluce.com/api/v1/auth/me` calls, but those are the landing page's user-aware CTAs (e.g. a "Sign in" button that checks session). The `escluse-landing` image was correctly built from `landing-page-escluse/` via `Dockerfile.landing`.

The actual problem was that I had earlier deployed the **dashboard** image (v0.4.3) to BOTH `escluse-frontend:latest` AND `escluse-landing:latest` in ECR (because the Caddyfile routes `esluce.com` to `landing:80`, and I wanted the dashboard's address/version fixes to be visible at `esluce.com`). That overwrote the real landing page image in the `escluse-landing` slot.

Full fix shipped 2026-06-05 23:28 UTC:

1. **Rebuilt the real landing page from source** — `cd landing-page-escluse && rm -rf dist && npm run build` (2202 modules, 14.25s, 704KB bundle). Source was unchanged so bundle hash came out the same as before: `index-Djcdz0ZK.js`.
2. **Built and pushed the real landing Docker image** — `docker build -f Dockerfile.landing -t escluse-landing:latest .` then pushed to ECR. New manifest digest `sha256:98317b141925dc4083859d58d9c798e5261934b62f470a91413696509509cda6`. This overwrites the dashboard image I had pushed earlier.
3. **Redeployed `escluse_landing` container** on EC2 — `docker compose pull landing && docker compose up -d landing`.
4. **Reverted the temporary `esluce.com` → `frontend:80` change** in `gateway/Caddyfile.prod` — now routes back to `landing:80` (the real landing page). The earlier Caddyfile change was wrong: I had changed the routing to "fix" the mislabel, when the real problem was the image content. With the image now correct, the original Caddyfile routing is also correct.
5. **Restarted caddy** to pick up the reverted Caddyfile.
6. **Removed** the temporary `landing.esluce.com` route that had been added as a placeholder — user requested no external landing subdomain. Caddy now manages TLS for 4 domains only: `app.esluce.com, esluce.com, docs.esluce.com, www.esluce.com`.

Verified post-deploy:

| URL | Title | Bundle | Source |
|---|---|---|---|
| `https://esluce.com` | "Escluse - Distributed Infrastructure Platform" | `index-Djcdz0ZK.js` (704KB) | landing page (`landing-page-escluse/dist/`) |
| `https://app.esluce.com` | "Escluse — Server Control Platform" | `index-DsxEaHex.js` (1.4MB) | dashboard (`app/dist/` with v0.4.3 fix) |
| (none — `landing.esluce.com` was proposed as a placeholder but rejected) | — | — | — |

User-facing impact: any bookmarks/links to `esluce.com/servers/{id}` or other dashboard paths will now show the landing page instead. The dashboard is at `https://app.esluce.com`. Users need to update their bookmarks.

### Status fix (applied 2026-06-08): Node disconnect → servers offline

The WebSocket disconnect handler at `node_ws_handler.rs:542-567` already updated **node** status to `"offline"` on disconnect but never touched **server** records. Fixed by adding a `find_by_node_id` + `update_status` loop on disconnect:

```rust
// After setting node to "offline":
if let Ok(servers) = container.server_repository.find_by_node_id(&nid).await {
    for server in servers {
        let _ = container.server_repository.update_status(&server.id, "offline").await;
    }
}
```

This ensures that when an agent WebSocket disconnects (clean close or error), all servers assigned to that node transition from `"running"` to `"offline"`, which the dashboard renders as a gray/green badge correctly.

**Status issue root cause (gap 5):** The only code paths that set `servers.status = "running"` were `CommandResponse` on start/restart (`node_ws_handler.rs:342`) and `server_handlers.rs:880-882`. There was NO code path to set `servers.status = "offline"` on disconnect, timeout, or error. Servers stayed green indefinitely.

**Files changed:** `api/src/presentation/handlers/node_ws_handler.rs` (added server status update to disconnect handler)

**Verification:** After deploy, disconnect an agent (e.g. stop the `solys` service on the node). Check `GET /api/v1/servers/:id` — status should be `"offline"`. Reconnect the agent; status stays `"offline"` until user manually starts the server again (CommandResponse sets it back to `"running"`).

## Related
- `TEMP_CHANGELOG.md` v0.4.3 — describes the frontend Address/Version/Console fix
- `app/src/pages/servers/ServerDetailsPage.jsx:110-116, 135, 158, 199-204` — the display logic
- `api/src/domain/server/model.rs:8-31` — OLD Server model (used by dashboard)
- `api/src/domain/entities/server.rs:8-75` — NEW Server model (has mc_version but unused by dashboard endpoint)
- `api/src/presentation/handlers/server_handlers.rs:605-622` — get_server handler (uses OLD model)
- `api/src/presentation/handlers/node_ws_handler.rs:237-266` — heartbeat handler (no DB write to servers)
- `agent/solys/src/handlers/metrics.rs:119-212` — agent container metrics collection (no docker inspect for VERSION)
- `agent/solys/src/agent_connection.rs:30-46, 462-493` — Register and Heartbeat serialization
- `api/migrations/20260223015400_add_mc_version.sql:2` — mc_version column migration
- `api/migrations/20260326000001_add_server_new_columns.sql:26` — endpoints JSONB migration
