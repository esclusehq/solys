---
status: resolved
trigger: "wss://app.esluce.com/ws returns HTTP 200 instead of 101 (WebSocket upgrade) — Caddy reverse proxy not configured for WebSocket"
created: 2026-06-02
updated: 2026-06-02
resolved: 2026-06-02
---

## Symptoms

- Expected behavior: WebSocket connection to wss://app.esluce.com/ws should return HTTP 101 Switching Protocols
- Actual behavior: Returns HTTP 200 (normal HTTP response)
- Error messages: WebSocket connection fails in browser (HTTP 200, not 101)
- Timeline: Unclear — possibly never worked on app.esluce.com subdomain
- Reproduction: Open browser DevTools or use wscat: `wscat -c wss://app.esluce.com/ws`

## Current Focus

- Next action: Verify fix deployed

## Environment

- Domain: app.esluce.com
- Backend runs on EC2 (escluse_backend container)
- Reverse proxy: Caddy on EC2 instance
- Backend serves WebSocket at: `/ws`, `/ws/docker-logs`, `/ws/terminal/:server_id`
- Agent connects at `/api/ws/node` (already works via existing `/api/*` route)

## Evidence

- timestamp: 2026-06-02T00:00:00Z
  content: Browser WebSocket connection returns HTTP 200 instead of 101
- timestamp: 2026-06-02T00:00:00Z
  content: App.esluce.com Caddy block only routes /api/* to backend; /ws* falls through to frontend SPA container
- timestamp: 2026-06-02T00:00:00Z
  content: Agent WebSocket at /api/ws/node is covered by existing /api/* route — already works

## Eliminated

(no eliminated hypotheses yet)

## Resolution

- root_cause: Caddyfile.prod `app.esluce.com` block only matched `/api/*` to backend; `/ws*` (frontend WebSocket, terminal, docker-logs) unmatched → fell through to `frontend:80` → HTTP 200
- fix: Added `@ws path /ws*` route before the catch-all `reverse_proxy frontend:80` in `app.esluce.com` block
- files_changed: ["gateway/Caddyfile.prod"]
- verification: After deploy, `wscat -c wss://app.esluce.com/ws` should return 101 Switching Protocols

## Root Cause

**Caddyfile.prod at `gateway/Caddyfile.prod` does not route `/ws/*` paths to the backend for `app.esluce.com`.**

The production Caddy configuration for `app.esluce.com` (lines 38-43) only matches `/api/*` to send to the backend:

```caddy
app.esluce.com {
    @api path /api/*
    reverse_proxy @api escluse_backend:3000
    reverse_proxy frontend:80
    import security_headers
}
```

The backend (Axum in Rust) exposes these WebSocket endpoints:
| Route | Backend handler | Used by |
|---|---|---|
| `/ws` | `ws_handler.rs` — server event streaming | Frontend SPA (`useWebSocket` hook, `/ws`) |
| `/ws/terminal/:server_id` | `terminal_ws_handler.rs` — terminal access | Frontend SPA (`/ws/terminal/...`) |
| `/ws/docker-logs` | `docker_log_handler.rs` — container logs | Frontend SPA (Console page) |
| `/ws/build/:server_id` | `build_handler.rs` — build logs | Frontend IDE (`/ws/build/...`) |
| `/api/ws/node` | `node_ws_handler.rs` — agent registration | Agent (Solys) |

**Frontend WebSocket paths** (`/ws`, `/ws/terminal/*`, `/ws/build/*`) do NOT match the `/api/*` route matcher in Caddy. These requests fall through to `reverse_proxy frontend:80`, which serves the SPA static HTML — returning HTTP 200 instead of the expected 101 Switching Protocols upgrade.

**Agent WebSocket path** (`/api/ws/node`) DOES match `/api/*` and should route to the backend correctly through Caddy's transparent WebSocket proxy support.

**Caddy v2** handles WebSocket upgrades transparently in `reverse_proxy` — no special `websocket` directive is needed. The sole issue is the missing route match for `/ws` and `/ws/*` paths.

**Fix needed:** Add a route matcher for `/ws*` paths to proxy them to the backend, e.g.:

```caddy
@ws path /ws*
reverse_proxy @ws escluse_backend:3000
```

## Resolution

- **root_cause:** Caddyfile.prod for `app.esluce.com` only routes `/api/*` to the backend; WebSocket paths `/ws`, `/ws/*` are not matched and get routed to the frontend SPA, which returns HTTP 200 instead of the WebSocket upgrade (101 Switching Protocols).
- **fix:** Add a `@ws path /ws*` route matcher in `gateway/Caddyfile.prod` under `app.esluce.com` to proxy WebSocket traffic to the backend.
- **status:** pending (diagnose-only session)
