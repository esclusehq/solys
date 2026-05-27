# Phase 46: MULTI-PLATFORM - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Add Windows support - binary, service, installer, config paths.
</domain>

<decisions>
## Implementation Decisions

### Current State
- ✅ Installer supports Linux (Ubuntu, Debian, CentOS, Fedora, RHEL, AlmaLinux, Rocky)
- ✅ Architectures: x86_64, amd64, aarch64, arm64
- ✅ systemd service (Linux)

### Windows Support (D-01 to D-04)

- **D-01:** Add Windows build target: `x86_64-pc-windows-msvc`
- **D-02:** Windows service using NSSM (similar to systemd)
- **D-03:** Config paths use APPDATA (not XDG)
- **D-04:** Windows installer (PowerShell-based)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `release/package/install.sh` - existing Linux installer (reference)
- `release/package/solys.service` - systemd service (reference)
- `agent-core/crates/agent-config/src/loader.rs` - config path logic

</canonical_refs>

<specifics>
## Specific Ideas

Full Windows support:
1. Binary for Windows (x86_64-pc-windows-msvc)
2. Windows service via NSSM
3. Config in APPDATA
4. PowerShell installer

</specifics>

<deferred>
## Deferred Ideas

- macOS support (later)
- ARM32 Windows (later)

</deferred>

---

*Phase: 46-multi-platform*
*Context gathered: 2026-05-03*