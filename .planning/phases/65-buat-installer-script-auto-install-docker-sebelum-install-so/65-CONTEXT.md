# Phase 65: Buat installer script auto-install Docker sebelum install Solys agent - Context

**Gathered:** 2026-05-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Modify the existing `agent/solys/install.sh` to auto-detect and optionally install a container runtime (Docker or Podman) before installing the Solys agent binary. The script handles detection of existing runtimes, installation of missing runtimes, service configuration, and agent config generation.

The result is a `curl -fsSL https://get.esluce.com/latest/install.sh | bash` experience that ensures the target system has a working container runtime before the agent binary is installed and configured. Phase 42 (Auto Installer) decided D-03: "Auto-install podman/docker if not found" — this phase implements that decision for the Solys agent installer.

</domain>

<decisions>
## Implementation Decisions

### Container Runtime — Detection and Preference
- **D-01:** **Detect and prefer** — Check both Docker and Podman. If neither is found, install Docker as the default fallback.
- **D-02:** **Podman preferred** — If both Docker AND Podman are installed, the installer configures Podman first. Fall back to Docker if Podman's Docker-compatible socket is unavailable.
- **D-03:** **Auto-configure Podman socket** — When Podman is detected or installed, automatically enable the Docker-compatible API socket via `podman system service` or systemd socket activation.
- **D-04:** **Docker install method** — Use the official Docker convenience script (`get.docker.com`) first. Fall back to package manager if the script fails.

### Distro Support
- **D-05:** **Major 3 distro families** — Debian/Ubuntu (apt), CentOS/RHEL/AlmaLinux (yum), Fedora (dnf). The Docker convenience script covers additional distros; the package-manager fallback is scoped to these three.
- **D-06:** **Podman socket all distros** — If Podman is detected on any distro, auto-enable its Docker-compatible socket universally (not limited to Major 3).

### Post-Install Actions
- **D-07:** **Start + enable service** — After installing Docker/Podman, run `systemctl enable --now` so the service starts immediately and on boot.
- **D-08:** **Root check** — If script is not run as root/sudo, fail immediately and instruct the user to re-run with appropriate privileges.
- **D-09:** **Auto-generate agent config** — Pre-generate `config.toml` with the correct container runtime socket path (e.g., `/run/podman/podman.sock` or `/var/run/docker.sock`). User still fills in `backend_url` and `api_key`.

### Container Runtime Detection
- **D-10:** **Docker detection** — Check `docker --version` succeeds AND `docker info` returns successfully (daemon responsive).
- **D-11:** **Podman detection** — Check `podman --version` exists AND the Docker-compatible socket is active/listening.
- **D-12:** **Daemon not running** — If the binary exists but daemon isn't active, auto-start it via `systemctl start`, then re-verify.

### Non-Interactive Behavior
- **D-13:** **Informative output** — Use colored progress messages (info/ok/warn) consistent with the existing install.sh style. No silent mode.
- **D-14:** **No skip flag** — The detection check is lightweight (negligible overhead). Always run it.
- **D-15:** **Fail on Docker install error** — If Docker/Podman installation fails, print error with troubleshooting info and abort. Do not install the agent binary.
- **D-16:** **Non-interactive Docker, interactive config** — Docker install runs fully automatically without prompts. Only after Docker is confirmed working does the script prompt for `backend_url` and `api_key` (following Phase 42 D-05 pattern).

### The Agent's Discretion
- Exact order of detection checks (Docker first or Podman first)
- How to detect Podman's Docker socket path per distro
- Which systemd unit names to use (docker.service, podman.socket, etc.)
- Exact messages/warnings displayed to user
- How to write config.toml (template details, etc.)
- Whether to check `docker ps` as part of daemon responsiveness test
- Specific package names per distro for fallback installation
- Cosign/signature verification order relative to Docker check

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary Source Material (Existing Install Script)
- `agent/solys/install.sh` — Current 156-line install script to modify. Handles platform detection, binary download, SHA256 verification, cosign verification, binary installation.
- `agent/solys/install.ps1` — Windows counterpart (not in scope for this phase).

### Prior Phase Decisions
- `.planning/phases/42-auto-installer/42-CONTEXT.md` — D-03: Auto-install podman/docker if not found (Phase 65 implements this decision for Solys agent)
- `.planning/phases/41-packaging-core-release/41-CONTEXT.md` — Release package structure, static binary, install/uninstall scripts pattern

### Codebase Maps (Tech Context)
- `.planning/codebase/STACK.md` — Docker/Podman versions, Bollard v0.18, platform requirements
- `.planning/codebase/INTEGRATIONS.md` — Container runtime integrations (Bollard, Docker API)

### Phase Goal
- `.planning/ROADMAP.md` § Phase 65 — "Buat installer script auto-install Docker sebelum install Solys agent"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `agent/solys/install.sh` — 156-line install script with: platform detection (OS/ARCH normalization), download from `get.esluce.com`, SHA256 checksum verification, cosign signature verification, binary installation to `/usr/local/bin`. The Docker detection/install logic will be added as new functions called before the existing binary install flow.
- Script uses `set -euo pipefail` — fail-fast on any error. The Docker install function needs to handle this carefully (Docker script may have different error behavior).

### Established Patterns
- Platform normalization: `detect_platform()` normalizes OS/arch — consistent approach for Docker platform detection
- Color helpers: `info()`, `ok()`, `warn()`, `fail()` — consistent output style
- `curl -fsSL` for downloads — used for both binary and SHA256SUMS
- `install -m 755` for binary placement with sudo fallback
- `mktemp -d` with `trap 'rm -rf ...' EXIT` for temp file cleanup

### Integration Points
- Docker detection check will go **before** `download_artifacts()` — no point downloading agent binary if Docker isn't available
- Podman socket auto-configuration must happen **before** config.toml generation (socket path needed in config)
- Config generation should be added as a new step after `install_binary()` (or replace `print_success()` with a more comprehensive post-install flow)
- The curl|bash pipe pattern must be preserved — stdin will be consumed by the pipe, so interactive prompts (backend_url, api_key) need special handling (e.g., read from tty)

### Creative Options
- The Docker install could be a separate function `install_docker()` called inline in `main()`, keeping the existing flow intact
- The official Docker convenience script (`get.docker.com`) is itself a curl | sh script — running it inside the install.sh is straightforward
- Podman socket path detection: `/run/podman/podman.sock` (root) or `/run/user/$UID/podman/podman.sock` (rootless)

</code_context>

<specifics>
## Specific Ideas

Build Docker detection and installation into the existing install.sh as additional functions, called before binary download. The existing platform detection feeds into Docker platform detection. Output style matches existing install.sh conventions.

Flow: detect_platform → detect_container_runtime (Docker/Podman check) → install_container_runtime if needed → verify_runtime → download_artifacts → ... → generate_config → print_success

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 65-buat-installer-script-auto-install-docker-sebelum-install-so*
*Context gathered: 2026-05-31*
