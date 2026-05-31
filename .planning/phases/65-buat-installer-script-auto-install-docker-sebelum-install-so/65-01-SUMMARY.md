---
phase: 65-buat-installer-script-auto-install-docker-sebelum-install-so
plan: 1
subsystem: infra
tags: script, docker, podman, agent, installer, bash

# Dependency graph
requires:
  - phase: 50-automasi-binary-build-solys
    provides: Installer scripts (install.sh, install.ps1)
  - phase: 42-auto-installer
    provides: D-03 decision to auto-install container runtime
provides:
  - Modified Solys agent install script with container runtime detection and auto-install
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Shared cleanup via CLEANUP_DIRS array for multiple temp dirs in piped scripts
    - /dev/tty redirection for interactive prompts in curl|bash pipe pattern
    - Priority-based container runtime detection: Podman (with socket) → Docker → auto-install

key-files:
  created: []
  modified:
    - agent/solys/install.sh

key-decisions:
  - "Shared CLEANUP_DIRS array replaces per-function traps (fixes trap overwrite bug)"
  - "_prompt() falls back to /dev/tty for curl | bash pipe pattern compatibility"

patterns-established:
  - "Container runtime detection uses global RUNTIME_TYPE/RUNTIME_SOCKET_PATH/RUNTIME_VERSION"
  - "Podman preferred over Docker when both are available and socket is active"
  - "generate_config() writes /etc/escluse/config.toml with socket path after agent binary install"

requirements-completed: []

# Metrics
duration: 20min
completed: 2026-05-31
---

# Phase 65: Add container runtime auto-install to Solys installer — Summary

**Docker/Podman detection, auto-install, and config.toml generation built into agent/solys/install.sh**

## Performance

- **Duration:** 20 min
- **Started:** 2026-05-31T16:40:00Z
- **Completed:** 2026-05-31T17:00:00Z
- **Tasks:** 7
- **Files modified:** 1

## Accomplishments

- `check_docker()` detects Docker with binary + daemon check, auto-starts stopped daemon (D-10, D-12)
- `check_podman()` detects Podman + Docker-compatible socket availability (rootful/rootless, D-11)
- `install_docker()` runs get.docker.com convenience script with apt/yum/dnf fallback (D-04, D-05, D-07)
- `configure_podman_socket()` enables podman.socket via systemd for rootful/rootless (D-03)
- `ensure_container_runtime()` orchestrates Podman→Docker→install priority flow in main() (D-02, D-14)
- `root_check()` fails early with clear message if not running as root (D-08)
- `generate_config()` writes /etc/escluse/config.toml with runtime socket path and prompts for backend_url/api_key (D-09)
- `_prompt()` handles interactive input via /dev/tty for curl|bash compatibility (D-16)
- `print_success()` enhanced with runtime type and socket path info
- Shared CLEANUP_DIRS array replaces conflicting individual trap handlers

## Task Commits

All tasks committed atomically in a single commit in the nested agent/solys repo:
1. **Tasks 1-7: Add all container runtime features** — `0a3f0fd` (feat)

## Files Created/Modified

- `agent/solys/install.sh` — Modified: 156 → 340 lines, added container runtime detection, installation, socket config, config generation, and interactive prompts

## Decisions Made

- Shared `CLEANUP_DIRS` array approach avoids trap overwrite conflicts from multiple temp dirs
- `_prompt()` reads from `/dev/tty` when stdin is not a terminal, enabling interactive config in curl|bash pipe mode
- Podman checked before Docker per D-02 preference; Docker auto-installed as fallback

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The `agent/solys/` directory is a nested git repo (not a submodule) and gitignored by the parent repo. Changes committed in the nested repo directly.
- Initial `_mktemp` cleanup design had a subshell scoping bug (function ran in `$()` losing the array assignment) — fixed with explicit `CLEANUP_DIRS+=()` call in the caller's scope.

## User Setup Required

None - the script handles everything autonomously. User prompted for `backend_url` and `api_key` during first run.

## Next Phase Readiness

- Solys installer now handles container runtime detection and installation automatically
- Ready for `/gsd-execute-phase 66` or `/gsd-verify-work`

---

*Phase: 65-buat-installer-script-auto-install-docker-sebelum-install-so*
*Completed: 2026-05-31*
