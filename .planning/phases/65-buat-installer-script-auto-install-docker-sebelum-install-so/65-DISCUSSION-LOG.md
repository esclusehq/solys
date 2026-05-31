# Phase 65: Buat installer script auto-install Docker sebelum install Solys agent - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-31
**Phase:** 65-buat-installer-script-auto-install-docker-sebelum-install-so
**Areas discussed:** Container Runtime Choice, Distro Support, Post-install Actions, Existing Docker Check, Non-interactive Behavior

---

## Container Runtime Choice

| Option | Description | Selected |
|--------|-------------|----------|
| Docker only | Install Docker CE. Most common, best community support. | |
| Podman only | Daemonless, rootless by default, native on RHEL/Fedora. | |
| Detect and prefer | Auto-detect which runtime is installed. If none, install Docker. | ✓ |
| Docker preferred | If both installed, use Docker first. | |
| Podman preferred | If both installed, use Podman first. | ✓ |
| Let user config decide | Both detected, config determines socket. | |
| Yes, auto-configure | Enable Podman Docker socket via systemd/system service. | ✓ |
| Document only | Just detect Podman binary, document socket setup separately. | |
| Official Docker convenience script | curl -fsSL get.docker.com | sh | |
| Package manager only | apt/yum/dnf install | |
| Official script + fallback | Try official script, fall back to package manager. | ✓ |

**User's choice:** Detect and prefer. Podman preferred over Docker. Auto-configure Podman socket. Docker via official script + package manager fallback.
**Notes:** Podman is preferred for its daemonless model on small VPS. Docker convenience script handles most distros; package manager fallback ensures reliability.

---

## Distro Support

| Option | Description | Selected |
|--------|-------------|----------|
| Major 3: Debian + RHEL + Fedora | apt, yum, dnf. Covers ~90% of VPS users. | ✓ |
| Major 5: + Arch + openSUSE | Broader coverage, more code. | |
| Docker script only | Rely entirely on get.docker.com. | |
| All detected Podman installations | Enable socket universally. | ✓ |
| Major 3 only | Only auto-configure on primary distros. | |

**User's choice:** Major 3 distro families for package fallback. Podman socket auto-config on ALL detected installations.
**Notes:** Docker convenience script covers additional distros; only the package manager fallback is scoped to Major 3.

---

## Post-install Actions

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, start + enable | systemctl enable --now docker/podman socket. | ✓ |
| Start only, document enable | Start now but don't enable on boot. | |
| Fail and instruct | If not root, fail and tell user to use sudo. | ✓ |

**User's choice:** Start + enable service. Auto-generate config.toml with socket path. Fail if not root.
**Notes:** Config generation with correct socket path ensures agent works immediately after install without manual configuration.

---

## Existing Docker Check

| Option | Description | Selected |
|--------|-------------|----------|
| Binary + daemon running | docker --version AND docker info succeeds. | ✓ |
| Binary only | docker --version exists. | |
| Socket accessible | /var/run/docker.sock exists. | |
| Binary + Docker socket | podman --version AND socket active. | ✓ |
| Binary only (Podman) | podman --version exists. | |
| Auto-start daemon | Try systemctl start, then re-verify. | ✓ |
| Warn and continue | Print warning, proceed with agent install. | |

**User's choice:** Docker: binary + daemon running. Podman: binary + Docker socket active. Auto-start daemon if binary exists but not running.
**Notes:** Daemon responsiveness check prevents installing the agent only to have it fail connecting to a non-running runtime.

---

## Non-interactive Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Informative (current style) | Colored progress messages matching install.sh. | ✓ |
| Silent | No output unless something fails. | |
| Yes, SKIP_DOCKER env var | Flag to skip Docker check entirely. | |
| No, always check | Check is lightweight, always run. | ✓ |
| Abort agent install | If Docker install fails, exit with error. | ✓ |
| Warn and continue | Install agent even if Docker fails. | |
| Non-interactive Docker, interactive config | Docker auto, prompt for backend_url/api_key. | ✓ |
| Prompt for Docker confirmation | Ask user before installing Docker. | |

**User's choice:** Informative output matching existing style. No skip flag (always check). Abort on Docker install failure. Non-interactive Docker install with interactive config prompts.
**Notes:** The curl | bash pattern requires careful stdin handling for interactive prompts — the pipe consumes stdin, so prompts need to read from /dev/tty.

---

## The Agent's Discretion

- Exact order of detection checks (Docker first or Podman first)
- How to detect Podman's Docker socket path per distro
- Which systemd unit names to use (docker.service, podman.socket, etc.)
- Exact messages/warnings displayed to user
- How to write config.toml (template details)
- Whether to check `docker ps` as part of daemon responsiveness test
- Specific package names per distro for fallback installation

## Deferred Ideas

None — discussion stayed within phase scope.
