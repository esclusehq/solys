---
status: resolved
trigger: "Rename agent from devnode to solys"
created: 2026-05-03T00:00:00Z
updated: 2026-05-03T03:30:00Z
---

## Current Focus
All references renamed from "solys-agent" to "solys"

## Symptoms
expected: Agent binary and references named "solys"
actual: Agent binary and references currently named "solys-agent"
errors: []
reproduction: Check release/package/ directory
started: Discovered after Phase 41 packaging

## Evidence

- timestamp: 2026-05-03
  checked: First rename phase (devnode -> solys-agent)
  found: All files renamed except binary/service files still had "-agent" suffix

- timestamp: 2026-05-03
  checked: Final rename phase per user checkpoint
  found: |
    - Binary: solys-agent → solys
    - Service: solys-agent.service → solys.service
    - All paths: /opt/solys-agent → /opt/solys, /etc/solys-agent → /etc/solys
    - All references in scripts updated
    - README.md updated with new names
    - config.toml header updated
    - .gitignore updated (web-agent/target/, solys/target/)
  implication: All references now use "solys" without "-agent" suffix

## Resolution
root_cause: Binary and service files were named "solys-agent" instead of "solys"
fix: |
  - Renamed release/package/solys-agent → solys
  - Renamed release/package/solys-agent.service → solys.service
  - Updated all references in install.sh (binary name, paths)
  - Updated all references in uninstall.sh (binary name, paths)
  - Updated solys.service (WorkingDirectory, ExecStart, ReadWritePaths)
  - Updated README.md (all references)
  - Updated config.toml header
  - Updated .gitignore (web-agent, solys)

  NOTE: Did NOT rename internal naming:
  - devnode-minecraft (network name - product branding)
  - devnode-podman-* (container names)
  - devnode-* (screen sessions)
  - devnode_workspace_mode (UI settings)
  - devnode-cosmic (Monaco editor theme)
  - devnode_bookmarks (IDE bookmarks)
  - devnode-s3 (S3 bucket)
verification: |
  - All "solys-agent" references eliminated from release/package/
  - Grep shows no matches for "solys-agent" in release/package/
files_changed:
  - release/package/solys (binary - renamed)
  - release/package/solys.service (service - renamed)
  - release/package/install.sh
  - release/package/uninstall.sh
  - release/package/README.md
  - release/package/config.toml
  - .gitignore