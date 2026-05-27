# Phase 41: PACKAGING (CORE RELEASE) - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Create a distributable release package for the agent so users can run it without coding — single static binary, config template, install/uninstall scripts.

</domain>

<decisions>
## Implementation Decisions

### Binary Format
- **D-01:** Static binary (no dynamic linking) — works on any Linux without dependencies

### Release Structure
- **D-02:** Contents: devnode-agent binary + config.toml + install.sh + uninstall.sh + README.md + systemd service file

### Installation Scripts
- **D-03:** install.sh: Full install — copy binary, create config, setup systemd, enable service
- **D-04:** uninstall.sh: Full uninstall — stop service, remove binary and config

### Default Config
- **D-05:** config.toml: Minimal template with only required fields (backend_url, api_key) — user fills in values

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `web-agent/Cargo.toml` — existing release profile configuration
- `agent-core/crates/agent-config/src/schema.rs` — config schema reference

</canonical_refs>

## Existing Code Insights

### Reusable Assets
- Cargo release profile already configured in web-agent/Cargo.toml
- agent-config handles TOML loading from Phase 39
- Existing systemd service patterns in similar projects

### Integration Points
- Binary needs to be built from web-agent package
- Config format follows agent-config schema

</code_context>

<specifics>
## Specific Ideas

No specific references — standard release packaging for Rust applications.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 41-packaging-core-release*
*Context gathered: 2026-05-03*