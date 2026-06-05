# TEMP_CHANGELOG — v0.4.0 + v0.4.2

Changelog sementara untuk release **v0.4.0** (Minggu depan) + hotfix **v0.4.2** (deploy Jumat).

> **Gunakan file ini sebagai referensi saat update `Changelog.tsx` di landing page pada hari Minggu.**
> Update format sesuai SEMVER.md dengan component tags.
>
> **v0.4.2 adalah hotfix** yang di-deploy Jumat 5 Juni 2026 — tidak menunggu rilis mingguan. Tetap catat di sini agar user tahu apa yang berubah.

---

## v0.4.2 (2026-06-05) — Patch (Hotfix Deploy)

Hotfix deploy di tengah minggu (Jumat) untuk fix terminal RCON yang completely broken di v0.4.1.

### Fixed
- [solys] Agent `file.*` long-form commands (`file.read_file`, `file.list_dir`, dll) ditolak dengan "Unknown task type: unknown" — match table di `src/agent_connection.rs` cuma kenal short form (`read_file`); dashboard kirim long form. Ini juga yang bikin terminal "Disconnected - Reconnecting..." loop
- [solys] `CommandParams` struct tidak deklarasikan field `command`, `rcon_port`, `rcon_password` — serde silently dropped them, payload builder tidak copy ke `Task` payload. RCON command return `missing field 'command'`
- [solys] RCON handler hardcode `127.0.0.1` untuk TCP connect — Minecraft server ada di dalam Docker container, port RCON tidak dipublish ke host. Result: `Failed to connect to RCON server: Connection refused`. Fix: resolve IP via `docker.inspect_container(...).NetworkSettings.Networks`, fallback ke explicit `host` di payload, atau 127.0.0.1 dengan warning
- [solys] Bump version ke 0.4.2 (Cargo.toml) — sebelumnya tag v0.4.2 di luar sync dengan `--version` output

### Changed
- [solys] CI workflow di-harden: `ci.yml` tambah `lint` (fmt+clippy -D warnings), `test`, `security audit` (rustsec/audit-check) jobs; `Swatinem/rust-cache` di semua build job; `timeout-minutes` per job; `concurrency` group; canary artifact upload pakai `compression-level: 0` (sebelumnya upload OOM di canary pipeline)
- [solys] Rcon handler dispatcher signature diubah: `rcon::handle_command(task, runtime)` — perlu `runtime` untuk call bollard inspect

---

## v0.4.3 (2026-06-05) — Patch (Hotfix Deploy, app)

Hotfix deploy di hari yang sama dengan v0.4.2 (Jumat) — fokus ke display bugs di dashboard Server Details page yang baru ketahuan setelah RCON fix di v0.4.2, plus auto-connect dari Open Console link dan restored esluce.com → landing page routing.

### Fixed
- [app] Server Details page "Address" column menampilkan `minecraft:26.2` (concat `game_type` + `minecraft_version`) untuk server yang running tapi belum punya `public_address` — misleading, looks like a version label. Ganti ke priority chain: `endpoints[0]` → `public_address` → `connection_address` → `${game_type}:${game_port}` → `—` (honest "no address" instead of bogus version string)
- [app] Server Details page "Version" column menampilkan `config.minecraft_version` (user-supplied at create time, bisa stale / default ke LATEST). Sekarang prefer `server.mc_version` (reported by agent from running container) — yang akan accurate 26.1.2 bukan yang user ketik 26.2
- [app] Server Details page tidak ada link "Open Console" — operator harus guess URL `/console?serverId=<id>`. Tambah cyan button "⌨ Open Console" di header next to "Scheduled Tasks" button
- [app] "Open Console" link dari Server Details page sekarang auto-connect ke RCON terminal — link pass `?serverId=<id>` ke `/console`, dan `Console.jsx` baca URL param via `useSearchParams` + pre-select server di dropdown + switch status indicator ke "Connected" (green pulse) + render `<Terminal serverId={id}/>` yang auto-connect ke `/ws/terminal/<id>`. Sebelumnya user harus manual pilih server dari dropdown setelah navigate — defeating the purpose of one-click shortcut

### Fixed (lintas-komponen)
- [app] PluginManager component Load More button + Empty State render di luar Search tab — closing tags salah tempat di end-of-file (setelah Templates dan Installed tabs). Fix: move closing JSX + Load More + Empty State ke dalam Search tab scope. Tiap tab sekarang self-contained
- [app] `PluginManager` mode-detection di Server Details page membaca `server?.game || server?.executor_type` (flat fields yang tidak exist) — default ke `'datapack'` mode untuk semua server, sembunyikan Paper/Spigot/Fabric plugins. Fix: read `server?.config?.game_type` (nested, yang actual di-store dari create-server form), fallback ke `server?.game`
- [landing] `escluse-landing:latest` ECR image (yang di-serve oleh `escluse_landing` container) accidentally ke-overwrite dengan dashboard bundle waktu deploy v0.4.3 — bikin `esluce.com` serve dashboard instead of landing page. Rebuild dari `landing-page-escluse/dist` via `Dockerfile.landing`, push ke ECR (manifest digest `76cc938804ad1fa7bee33f8487298538d4d04b57b0dbdaba271a93d158ea05fd`), redeploy container
- [gateway] `gateway/Caddyfile.prod` routing restored ke original intent: `esluce.com` → `landing:80` (real landing page), `app.esluce.com` → `frontend:80` (dashboard). Dulu `esluce.com` route pernah di-tweak ke `frontend:80` waktu investigasi Caddy mislabeling, tapi karena image content udah benar sekarang, original routing udah cukup

---

## v0.3.1 (2026-06-01) — Patch (Hotfix Deploy)

Hotfix deploy di tengah minggu. Semua fix akan merge ke v0.4.0 (Minggu).

### Fixed
- [api] Server deploy_config used hardcoded `version: "LATEST"` instead of reading `config.minecraft_version`
- [api] Server deploy_config used hardcoded `ram_mb: 2048` instead of reading `config.ram_mb`
- [api] Agent command timeout 30s → 120s (Docker image pulls exceeded old limit)
- [api] Server status stayed "pending" after container started (WS handler required `MC_READY` in output, which agent never sends)
- [solys] Agent DeployConfig field `env` didn't match backend's `env_vars` (env vars silently dropped)
- [api] Missing `MEMORY` env var in deploy_config (itzg image defaulted to 1G regardless of ram_mb setting)
- [api] Files tab, Server Properties, Address N/A now work for agent-mode servers (docker exec routed through agent WebSocket instead of EC2)
- [gateway] WebSocket at `wss://app.esluce.com/ws*` now routes to backend (was falling through to frontend SPA → HTTP 200)
- [app] Settings > Connection Address shows same fallback as Overview (`game_type:version`) instead of bare `N/A` when `endpoints` array is empty
- [solys] Agent had no file operation handlers (`list_dir`, `read_file`, `write_file` all mapped to `"unknown"` task type and returned error)
- [solys] Added `file.list_dir`, `file.read_file`, `file.write_file`, `file.delete`, `file.mkdir`, `file.rename`, `file.copy` handlers to agent
- [api] Agent command response `success` was hardcoded to `true` in `agent_client.rs` — errors silently swallowed; now propagates actual success from agent

---

## v0.4.0 (2026-06-08) — Minor

### Added
- [api] Founder role with admin-level permissions (bypass RBAC)
- [api] Built-in template edit by admin/owner
- [api] `is_active` toggle for built-in templates (Coming Soon)
- [api] SFTP upload/download API handlers (`/api/servers/:id/sftp/upload`, `/api/servers/:id/sftp/download`)
- [api] `send_rcon_command` WebSocket handler for terminal console
- [app] `/console` route with Terminal.jsx (xterm.js) — interactive RCON terminal with command history, autocomplete, reconnect
- [app] FileManager.jsx — SFTP file browser with upload/download, pagination
- [app] `useServers.js` — RCON API client (`sendRconCommand`)
- [app] Server details page with operation tabs (Console, Files, Overview)

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

---

## Referensi Format

```typescript
{
  version: '0.3.1',
  date: '2026-06-01',
  type: 'patch',
  changes: {
    added: [],
    improved: [],
    fixed: ['[api] Server deploy_config uses config.minecraft_version (not hardcoded LATEST)', '[api] Server deploy_config uses config.ram_mb (not hardcoded 2048)', '[api] Agent command timeout 30s→120s (Docker pull timeouts)', '[api] Server status updates on any start/restart (not just MC_READY)', '[solys] DeployConfig field env → env_vars (env vars no longer dropped)', '[api] MEMORY env var in deploy_config (itzg no longer defaults to 1G)', '[api] Files/Properties/Address route through agent WS for agent-mode servers', '[gateway] Caddy routes /ws* to backend (fixes WebSocket 101 upgrade)', '[app] Settings > Connection Address shows fallback (not bare N/A)', '[solys] Added file handlers (list_dir, read_file, write_file, delete, mkdir, rename, copy)', '[api] Agent success not hardcoded to true (propagates actual response)'],
    removed: [],
    security: []
  }
}

```typescript
{
  version: '0.4.0',
  date: '2026-06-08',
  type: 'minor',
  changes: {
    added: ['[api] Founder role with admin-level permissions', '[api] Built-in template edit by admin/owner', '[api] is_active toggle for built-in templates', '[api] SFTP upload/download API handlers', '[api] send_rcon_command WebSocket handler', '[app] /console route with xterm.js terminal', '[app] FileManager.jsx SFTP browser', '[app] useServers.js RCON client', '[app] Server details operation tabs'],
    improved: ['[solys] Agent logs to stdout by default (interactive mode)', '[solys] Agent detects public IP on registration', '[api] Default heartbeat interval 10s→30s', '[api] Degraded threshold 50%→90% of interval', '[api] Node IP updates on re-registration', '[app] Server details page tabs'],
    fixed: ['[solys] read -p hidden by 2>/dev/null', '[solys] TOML missing [server] section', '[solys] XDG config path', '[solys] gzip CRC no abort', '[solys] Agent panic on log dir not writable', '[solys] IP 127.0.0.1 on registration', '[solys] Container DNS set to 8.8.8.8, 1.1.1.1', '[api] Node always degraded (heartbeat mismatch)', '[api] Template is_active never persisted', '[api] get_template_by_id filtered by is_active', '[api] list_templates_by_user excluded inactive', '[api] Node ip_address not updated on re-register', '[api] SFTP download borrow error in file_handlers.rs', '[api] plugin_templates table missing (500 INTERNAL_ERROR); migration + repository fallback', '[app] Terminal component name collision with @xterm/xterm', '[app] Node Created dialog sudo install command', '[app] Templates tab locked behind Hobby+ plan', '[app] plugin-templates API called with loader_type instead of game', '[app] isHobbyPlus only checked user.plan (never set); added user.role check', '[app] Templates sub-tab rendered empty — block was nested inside Marketplace block; moved to top-level sibling', '[api] Terminal RCON hardcoded to docker exec on EC2 — agent-mode servers failed; refactored to dispatcher routing to file.read_file + server.command via agent', '[api] CommandParams extended with rcon_port, rcon_password, command for server.command payload', '[solys] CommandResponse.output dropped error message (None serialized to "null"); now uses "<code>: <message>" from result.error on failure'],
    removed: [],
    security: []
  }
}
```

```typescript
{
  version: '0.4.3',
  date: '2026-06-05',
  type: 'patch',
  hotfix: true,
  changes: {
    added: [],
    improved: [],
    fixed: [
      '[app] Server Details Address column: priority chain endpoints[0] → public_address → connection_address → game_type:game_port → — (no longer shows misleading minecraft:<version> concat)',
      '[app] Server Details Version column: prefer server.mc_version (from running container) over stale config.minecraft_version',
      '[app] Server Details page: cyan "⌨ Open Console" button links to /console?serverId=<id>',
      '[app] Console page reads ?serverId= from URL and auto-connects to that server\'s RCON terminal — no manual select needed',
      '[app] PluginManager: Load More button + Empty State were rendering outside Search tab due to misplaced closing tags; moved into Search tab scope',
      '[app] PluginManager mode-detection (plugin vs datapack) was reading server.game / server.executor_type (flat fields that don\'t exist) and falling back to "datapack" for every server; now reads server.config.game_type (nested, actually populated by create-server form) — Paper/Spigot/Fabric plugins now show up in the Search tab',
      '[landing] escluse-landing:latest ECR image had been overwritten with the dashboard bundle (accidental); rebuilt from landing-page-escluse/dist via Dockerfile.landing and pushed',
      '[gateway] Caddyfile routing restored: esluce.com → landing:80 (landing page), app.esluce.com → frontend:80 (dashboard)',
    ],
    removed: [],
    security: []
  }
}
```
