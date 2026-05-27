---
phase: 41-packaging-core-release
plan: "02"
subsystem: release-package
tags: [release, packaging, installer, config]
dependency_graph:
  requires:
    - "41-01"
  provides:
    - "release/package/config.toml"
    - "release/package/install.sh"
    - "release/package/uninstall.sh"
  affects:
    - "web-agent"
key_files:
  created:
    - "release/package/config.toml"
    - "release/package/install.sh"
    - "release/package/uninstall.sh"
tech_stack:
  added:
    - "shell-scripts"
  patterns:
    - "systemd-service-integration"
    - "installation-rollback-on-error"
decisions:
  - "config.toml contains only required fields (backend_url, api_key) per D-05"
  - "install.sh performs full installation with error handling"
  - "uninstall.sh performs full uninstallation with cleanup"
metrics:
  duration: ~1 min
  completed: "2026-05-03"
---

# Phase 41 Plan 02: Release Package Summary

**One-liner:** Release package with install/uninstall scripts and minimal config template for easy agent deployment.

## Objective

Create the release package with installation scripts and config template to provide users with a complete, easy-to-use distribution.

## Tasks Executed

| Task | Name | Commit | Files Created |
|------|------|--------|---------------|
| 1 | Create config.toml minimal template | 758c18c | release/package/config.toml |
| 2 | Create install.sh script | 23f863d | release/package/install.sh |
| 3 | Create uninstall.sh script | c8dae42 | release/package/uninstall.sh |

## Details

### Task 1: Create config.toml minimal template
- Created `release/package/config.toml` with only required fields
- `backend_url` - user's API endpoint
- `api_key` - user's authentication key
- Added comments with optional fields user can uncomment (agent_name, heartbeat_interval_secs)

### Task 2: Create install.sh script
- Full installation script with 7 steps:
  1. Check running as root
  2. Create directories (/opt/devnode-agent, /etc/devnode-agent)
  3. Copy binary to /opt/devnode-agent/
  4. Copy config.toml to /etc/devnode-agent/
  5. Copy systemd service file to /etc/systemd/system/
  6. Reload systemd daemon
  7. Enable and start the service
- Includes error handling and color-coded output

### Task 3: Create uninstall.sh script
- Full uninstallation script with 7 steps:
  1. Check if service is running, stop if so
  2. Disable service
  3. Remove binary from /opt/devnode-agent/
  4. Remove config from /etc/devnode-agent/
  5. Remove systemd service file
  6. Reload systemd daemon
  7. Show success message
- Includes cleanup of empty directories

## Verification Results

- [x] config.toml contains only backend_url and api_key (required fields)
- [x] install.sh exists and is executable, contains all installation steps
- [x] uninstall.sh exists and is executable, contains all uninstallation steps
- [x] All tasks committed individually with proper commit messages

## Usage

**Installation:**
```bash
sudo ./install.sh
# Then edit /etc/devnode-agent/config.toml with your credentials
sudo systemctl restart devnode-agent
```

**Uninstallation:**
```bash
sudo ./uninstall.sh
```

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None - all release package files are complete and ready for use.

## Commits

- 758c18c: feat(41-02): create minimal config.toml template
- 23f863d: feat(41-02): create install.sh script
- c8dae42: feat(41-02): create uninstall.sh script

---

## Self-Check: PASSED

- [x] All files created in release/package/
- [x] config.toml has required fields only
- [x] install.sh is executable (755)
- [x] uninstall.sh is executable (755)
- [x] All commits verified in git log