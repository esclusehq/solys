# TEMP_CHANGELOG — v0.4.0

Changelog sementara untuk release **v0.4.0** (Minggu depan).

> **Gunakan file ini sebagai referensi saat update `Changelog.tsx` di landing page pada hari Minggu.**
> Update format sesuai SEMVER.md dengan component tags.

---

## v0.4.0 (2026-06-08) — Minor

### Added
- [api] Founder role with admin-level permissions (bypass RBAC)
- [api] Built-in template edit by admin/owner
- [api] `is_active` toggle for built-in templates (Coming Soon)

### Improved
- [solys] Agent logs to stdout by default (interactive mode); `--quiet` flag for headless/daemon
- [solys] Agent detects public IP on registration instead of hardcoded `127.0.0.1`
- [api] Default heartbeat interval increased from 10s to 30s (matches agent interval)
- [api] Degraded threshold raised from 50% to 90% of interval (reduces false degraded)
- [api] Node IP now updates on re-registration (was stuck at `0.0.0.0`)
- [app] Template `is_active` toggle now works — `update_template` includes `is_active` in SQL

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
- [api] Server deploy_config used hardcoded `version: "LATEST"` instead of reading `config.minecraft_version`
- [api] Server deploy_config used hardcoded `ram_mb: 2048` instead of reading `config.ram_mb`
- [api] Agent command timeout 30s → 120s (Docker image pulls exceeded old limit)
- [api] Server status stayed "pending" after container started (WS handler required `MC_READY` in output, which agent never sends)
- [solys] Agent DeployConfig field `env` didn't match backend's `env_vars` (env vars silently dropped)
- [api] Missing `MEMORY` env var in deploy_config (itzg image defaulted to 1G regardless of ram_mb setting)

---

## Referensi Format

```typescript
{
  version: '0.4.0',
  date: '2026-06-08',
  type: 'minor',
  changes: {
    added: ['[api] Founder role with admin-level permissions', '[api] Built-in template edit by admin/owner', '[api] is_active toggle for built-in templates'],
    improved: ['[solys] Agent logs to stdout by default (interactive mode)', '[solys] Agent detects public IP on registration', '[api] Default heartbeat interval 10s→30s', '[api] Degraded threshold 50%→90% of interval', '[api] Node IP updates on re-registration'],
    fixed: ['[solys] read -p hidden by 2>/dev/null', '[solys] TOML missing [server] section', '[solys] XDG config path', '[solys] gzip CRC no abort', '[solys] Agent panic on log dir not writable', '[solys] IP 127.0.0.1 on registration', '[api] Node always degraded (heartbeat mismatch)', '[api] Template is_active never persisted', '[api] get_template_by_id filtered by is_active', '[api] list_templates_by_user excluded inactive', '[api] Node ip_address not updated on re-register'],
    removed: [],
    security: []
  }
}
```
