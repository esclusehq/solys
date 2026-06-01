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

### Fixed
- [solys] Interactive prompt hidden by `2>/dev/null` on `read -p` in install.sh
- [solys] TOML config missing `[server]` section — `backend_url` and `api_key` not parsed
- [solys] Config path `/etc/escluse/` → `~/.config/escluse-agent/` (correct XDG path)
- [solys] Install no longer prompts for backend URL (hardcoded to `wss://app.esluce.com/api/ws/node`)
- [solys] gzip CRC errors from GitHub Actions no longer abort install
- [solys] Agent panic when `/var/log/escluse-agent/` exists but not writable by non-root user
- [solys] Agent registered with IP `127.0.0.1` instead of actual public IP
- [api] Node always marked 'degraded' due to heartbeat interval mismatch (10s default vs 30s agent)

---

## Referensi Format

```typescript
{
  version: '0.4.0',
  date: '2026-06-08',
  type: 'minor',
  changes: {
    added: ['[api] Founder role with admin-level permissions'],
    improved: ['[solys] Agent logs to stdout by default (interactive mode)'],
    fixed: ['[solys] Interactive prompt hidden by 2>/dev/null'],
    removed: [],
    security: []
  }
}
```
