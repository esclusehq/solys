# Phase 42: AUTO INSTALLER (PENTING) - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

One-command install via curl: `curl -sSL https://get.esluce.com/agent | bash` — auto-detect OS, install dependencies, setup binary, config, and service.

</domain>

<decisions>
## Implementation Decisions

### Download Source
- **D-01:** Custom server (get.esluce.com) — host binary there for download

### OS Detection
- **D-02:** Support major distros: Ubuntu, Debian, CentOS, Fedora, RHEL, AlmaLinux

### Dependencies
- **D-03:** Auto-install podman/docker if not found

### Install Steps
- **D-04:** Full auto install: download binary → copy to /usr/local/bin → create config → enable service
- **D-05:** Interactive: ask for backend_url and api_key during install

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `release/package/install.sh` — existing install script (Phase 41)
- `release/package/config.toml` — config template
- `release/package/solys.service` — systemd service

</canonical_refs>

## Existing Code Insights

### Reusable Assets
- Phase 41 install.sh has installation steps to reference
- config.toml template already exists
- systemd service file exists

### Integration Points
- New installer script should supersede/extend existing install.sh
- Should work with existing config and service files

</code_context>

<specifics>
## Specific Ideas

No specific references — standard installer patterns.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 42-auto-installer*
*Context gathered: 2026-05-03*