# TEMP_CHANGELOG — v0.4.3 + v0.4.4 + Unreleased (solys Phase 70)

## Unreleased — solys Phase 70 + CI fixes

### Added
- [solys] **Auto-fetch relay config via WebSocket (Phase 70)** — Backend sends single `RelayConfigSync` message after `RegisterAck` (replaces N-per-server `RelayConnect`), carrying `relay_token`, `gateway_url`, `region`, and all server relay info. Agent splits config into `GlobalRelayConfig` (env/TOML, immutable) and `RelaySessionState` (WS push, dynamic replace). Agent `apply_relay_config()` implements diff-based hot update: cancels tunnels for removed servers, starts new servers, restarts tunnels with changed config (subdomain/public_port/local_mc_addr). Agent WS message loop handles `RelayConfigSync` natively with backward compat for legacy env-var bootstrap.

### Fixed
- [ci] **Windows x86_64 cross-compile fails with "cannot find -lPacket"** — `pnet_sys` (transitive dep via `upnp-rs`) links `Packet.lib` from Npcap/WinPcap, unavailable in cross-compile toolchain. Added CI step: download Npcap SDK, create MingW-compatible `libPacket.a` via `dlltool`, set `RUSTFLAGS=-L /tmp/npcap-lib`. Applied to canary.yml, ci.yml, release.yml. Verified: all 3 platforms green.
- [solys] **DnsWatcher never syncs DNS when config arrives after first tick** — removed IP-change guard so DNS records sync every polling cycle (300s)
- [solys] **RelayClient default gateway URL uses unregistered domain `esluce.net`** — changed default to `wss://relay.esluce.com/relay/tunnel`

---

## v0.4.4 (2026-06-08) — Patch

### Fixed
- [api] Server status stuck at "running" when node WebSocket disconnects — added `find_by_node_id` + `update_status("offline")` loop in WebSocket disconnect handler (`node_ws_handler.rs`)
- [api] Server status still shows "running" on offline nodes after backend restart — added server cascade to "disconnected" in Node Offline Detection Service for already-offline nodes (`bootstrap/mod.rs`)
- [ci] `api/Cargo.toml` missing `[workspace]` table — added workspace marker + excluded from root workspace to allow standalone compilation

---

## v0.4.3 (2026-06-05) — Patch (Hotfix Deploy, app)

Hotfix deploy fokus ke display bugs di dashboard Server Details page + auto-connect dari Open Console link + restored esluce.com → landing page routing.

### Fixed
- [app] Server Details page "Address" column menampilkan `minecraft:26.2` (concat `game_type` + `minecraft_version`) — misleading. Priority chain: `endpoints[0]` → `public_address` → `connection_address` → `${game_type}:${game_port}` → `—`
- [app] Server Details page "Version" column prefers `server.mc_version` over stale `config.minecraft_version`
- [app] Server Details page: cyan "⌨ Open Console" button links to `/console?serverId=<id>`
- [app] Console auto-connects via `?serverId=` URL param + "← Back to Server" button
- [app] PluginManager: Load More + Empty State moved into Search tab scope (was rendering outside)
- [app] PluginManager reads `server.config.game_type` (nested) instead of flat `server.game` — Paper/Spigot/Fabric plugins now visible
- [landing] `escluse-landing:latest` rebuilt from source (was accidentally overwritten with dashboard bundle)
- [gateway] Caddyfile routing restored: `esluce.com` → `landing:80`, `app.esluce.com` → `frontend:80`

---

## v0.4.2 (2026-06-05) — Patch (Hotfix Deploy, solys)

Hotfix deploy untuk fix terminal RCON yang completely broken di v0.4.1.

### Fixed
- [solys] Agent `file.*` long-form commands rejected with "Unknown task type" — match table only knew short form; dashboard sends long form. Root cause of terminal "Disconnected - Reconnecting..." loop
- [solys] `CommandParams` missing `command`, `rcon_port`, `rcon_password` fields — serde silently dropped them
- [solys] RCON handler hardcoded `127.0.0.1` — resolve IP via `docker.inspect_container` instead
- [solys] Bump version to 0.4.2 (Cargo.toml) — was out of sync with git tag

### Changed
- [solys] CI workflow hardened: lint (fmt+clippy -D warnings), test, security audit, rust-cache, timeout-minutes, concurrency

---

## v0.4.0 (2026-06-08) — Minor

### Added
- [api] Founder role with admin-level permissions (bypass RBAC)
- [api] Built-in template edit by admin/owner
- [api] `is_active` toggle for built-in templates (Coming Soon)
- [api] SFTP upload/download API handlers (`/api/servers/:id/sftp/upload`, `/api/servers/:id/sftp/download`)
- [api] `send_rcon_command` WebSocket handler for terminal console
- [api] Relay infrastructure: `relay_token`, `relay_url`, `server_id` columns on nodes; `NodeMessage` Relay variants (Connect/Disconnect/Heartbeat); `RelayService` (token issuance, mode override, tunnel event handler); relay REST handlers + internal HMAC handlers; relay-metrics scraper + D-23 alert evaluation
- [api] Connectivity: `connectivity_state` column + `ConnectivityAuditLog` table; `ConnectivityService` (probe + classify + auto-fix + 5-min re-probe); REST handlers at `/api/v1/servers/:server_id/connectivity*`
- [solys] Relay client: yamux session, WS dispatch, `relay_client.rs` (connect/disconnect/heartbeat); `relay_session.rs` session management; `relay.rs` task entrypoint
- [solys] Connectivity diagnostics: `connectivity/mod.rs` with firewall UPnP actions, diagnostics collector; `ConnectivityMonitor` background task
- [gateway] Relay gateway crate (`opt/relay/`) with yamux tunnel, player Handshake parser, by_subdomain routing, HMAC auth, Prometheus metrics, rate limiter, heartbeat, session log
- [gateway] Caddy reverse proxy with automatic Let's Encrypt wildcard cert via DNS-01 (Route 53); Docker Compose stack (gateway + Caddy)
- [gateway] Relay gateway deploy runbook: EC2 (c6i.large, AL2023), NLB (TCP:25565), IAM (scoped Route53), Route 53 wildcard `*.play.esluce.net`, `GATEWAY_HMAC_SECRET` via Secrets Manager
- [app] Connectivity tab on ServerDetailsPage with ConnectivitySection component
- [app] `relayApi` + `useConnectivity` hooks + 3 dashboard components

### Improved
- [solys] Agent logs to stdout by default (interactive mode); `--quiet` flag for headless/daemon
- [solys] Agent detects public IP on registration instead of hardcoded `127.0.0.1`
- [api] Default heartbeat interval increased from 10s to 30s (matches agent interval)
- [api] Degraded threshold raised from 50% to 90% of interval (reduces false degraded)
- [api] Node IP now updates on re-registration (was stuck at `0.0.0.0`)
- [app] Template `is_active` toggle now works — `update_template` includes `is_active` in SQL
- [app] Server details page restructured with tabs for better navigation

### Fixed
- [solys] Interactive prompt hidden by `2>/dev/null` on `read -p` in install.sh
- [solys] TOML config missing `[server]` section — `backend_url` and `api_key` not parsed
- [solys] Config path `/etc/escluse/` → `~/.config/escluse-agent/` (correct XDG path)
- [solys] Install no longer prompts for backend URL (hardcoded to `wss://app.esluce.com/api/ws/node`)
- [solys] gzip CRC errors from GitHub Actions no longer abort install
- [solys] Agent panic when `/var/log/escluse-agent/` exists but not writable by non-root user
- [solys] Agent registered with IP `127.0.0.1` instead of actual public IP
- [solys] Container DNS set to `8.8.8.8`, `1.1.1.1` on create/start (was using host DNS, causing resolution failures)
- [api] Node always marked 'degraded' due to heartbeat interval mismatch (10s default vs 30s agent)
- [api] Template `update_template` ignored `is_active` field (Coming Soon toggle never persisted)
- [api] `get_template_by_id` filtered by `is_active = true` (couldn't fetch inactive template to re-enable)
- [api] `list_templates_by_user` excluded inactive templates (Coming Soon cards invisible to admin)
- [api] Node `ip_address` not updated on re-registration (stuck at original value)
- [api] SFTP download borrow error in `file_handlers.rs` (Rust E0505 — `payload.remote_path` moved while borrowed)
- [app] Terminal.jsx component name `Terminal` collided with `@xterm/xterm` import (esbuild refused to build)
- [app] Node Created dialog showed `bash <(curl ...)` which fails without root; updated to `sudo bash -c "$(curl ...)"`
- [api] `plugin_templates` table was missing from DB — repository returned 500 INTERNAL_ERROR; added migration `20260604000001_create_plugin_templates.sql` (table + 9 seed rows) and made repository fall back to hardcoded templates when table is missing
- [app] Templates tab was locked behind Hobby+ plan check in `PluginManager.jsx`; removed lock icon and always show the tab (content/upgrade notice still gated by `isHobbyPlus`)
- [app] PluginManager called `plugin-templates` API with `server.loader_type` (e.g. `paper`) instead of `server.game` (e.g. `minecraft`); added `serverGameType` prop and filter by variant on frontend
- [app] `isHobbyPlus` check in `usePluginTemplates.js` / `useModpackTemplates.js` only checked `user.plan` (never set by `/auth/me`); expanded to also include `user.role` of `owner`/`founder`/`admin` so admins can see templates
- [app] Templates sub-tab rendered empty even when 6 templates loaded — `<Templates>` and `<UpgradeNotice>` blocks in `PluginManager.jsx` were accidentally nested *inside* the `<Marketplace>` block JSX, so when `activeSubTab === 'templates'` the marketplace condition returned null and hid the nested templates too; moved both to top-level siblings of marketplace/installed blocks
- [api] Terminal RCON handler hardcoded to `docker exec` on EC2 — for agent-executor servers the container lives on the remote node, so `get_rcon_info` always failed with "RCON not configured"; refactored into dispatcher (`get_rcon_info` / `send_rcon_command`) that routes to `file.read_file` + `server.command` via the agent WebSocket when `executor_type == "agent"`
- [api] `CommandParams` extended with `rcon_port` / `rcon_password` / `command` fields so backend can forward RCON parameters to the agent's `server.command` handler
- [solys] Agent `CommandResponse.output` was serializing `result.output = None` to literal string `"null"` when a task failed — error code and message were silently dropped, making agent failures undebuggable from the backend (e.g. terminal RCON showed "Failed to read server.properties: null" with no real cause); now falls back to `"<code>: <message>"` from `result.error` so backend can surface the actual failure to the user
- [gateway] Yamux server session using `Session::new_server` + `ws_bridge` (was not properly handling server-side yamux)
- [gateway] WS messages changed to Binary only (Text deleted) for tunnel control plane
- [gateway] `auth::authorize` now called with `relay_token` + `server_id` (was not being called)
- [gateway] NDJSON framing on all control stream writes (connect, heartbeat, on-demand) with `b'\n'` delimiter
- [gateway] `TunnelHeartbeat.server_id` marked `#[serde(default)]` to send `0` instead of nil UUID
- [api] Connectivity routes mounted at `/api/v1/servers/:server_id/connectivity*` with proper Path<Uuid> extraction (was at wrong path)

---

## Referensi Format

```typescript
{
  version: '0.4.4',
  date: '2026-06-08',
  type: 'patch',
  changes: {
    added: [],
    improved: [],
    fixed: [
      '[api] Server status transitions to "offline" on node WebSocket disconnect (node_ws_handler.rs:550-563)',
      '[api] Server status transitions to "disconnected" for already-offline nodes on bootstrap tick (bootstrap/mod.rs:111-126)',
      '[ci] api/Cargo.toml workspace isolation for standalone compilation',
    ],
    removed: [],
    security: []
  }
}
```
