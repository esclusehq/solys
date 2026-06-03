# TEMP_CHANGELOG — v0.4.0

Changelog sementara untuk release **v0.4.0** (Minggu depan).

> **Gunakan file ini sebagai referensi saat update `Changelog.tsx` di landing page pada hari Minggu.**
> Update format sesuai SEMVER.md dengan component tags.

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
- [api] Node always marked 'degraded' due to heartbeat interval mismatch (10s default vs 30s agent)
- [api] Template `update_template` ignored `is_active` field (Coming Soon toggle never persisted)
- [api] `get_template_by_id` filtered by `is_active = true` (couldn't fetch inactive template to re-enable)
- [api] `list_templates_by_user` excluded inactive templates (Coming Soon cards invisible to admin)
- [api] Node `ip_address` not updated on re-registration (stuck at original value)

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
    fixed: ['[solys] read -p hidden by 2>/dev/null', '[solys] TOML missing [server] section', '[solys] XDG config path', '[solys] gzip CRC no abort', '[solys] Agent panic on log dir not writable', '[solys] IP 127.0.0.1 on registration', '[api] Node always degraded (heartbeat mismatch)', '[api] Template is_active never persisted', '[api] get_template_by_id filtered by is_active', '[api] list_templates_by_user excluded inactive', '[api] Node ip_address not updated on re-register', '[app] Terminal component name collision with @xterm/xterm', '[api] SFTP download borrow error in file_handlers.rs'],
    removed: [],
    security: []
  }
}
```
