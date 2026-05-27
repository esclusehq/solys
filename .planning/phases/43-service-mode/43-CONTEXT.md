# Phase 43: SERVICE MODE (WAJIB) - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Systemd service for auto-start and background running — ensure agent starts automatically on boot and runs continuously.
</domain>

<decisions>
## Implementation Decisions

### Service Configuration
- **D-01:** Use systemd (Linux standard)
- **D-02:** Service file: `/etc/systemd/system/solys.service`
- **D-03:** Auto-start: `systemctl enable solys`
- **D-04:** Start: `systemctl start solys`
- **D-05:** Restart on failure: `Restart=always` with 10s delay

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `release/package/solys.service` — existing systemd service file

</canonical_refs>

## Existing Code Insights

### Reusable Assets
- `release/package/solys.service` — systemd service file with security hardening
- Install script already handles service installation (Phase 42)
- Uninstall script handles service removal

### Service Features
- Type: simple
- User: root
- WorkingDir: /opt/solys
- Restart: always (10s delay)
- Security: NoNewPrivileges, PrivateTmp, ProtectSystem

</code_context>

<specifics>
## Specific Ideas

No additional specifics needed — service file is ready.

</specifics>

<deferred>
## Deferred Ideas

None

</deferred>

---

*Phase: 43-service-mode*
*Context gathered: 2026-05-03*