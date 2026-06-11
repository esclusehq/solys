#!/usr/bin/env bash
# Escluse Agent Installer
# Usage: curl -fsSL https://get.esluce.com/latest/install.sh | bash
#        curl -fsSL https://get.esluce.com/latest/install.sh | bash -s v1.2.3
set -euo pipefail

REPO="esclusehq/solys"
INSTALL_DIR="${ESCLUSE_BIN_DIR:-/usr/local/bin}"
VERSION="${1:-latest}"

# --- Color helpers ---
info()  { printf "\033[1;34m%s\033[0m\n" "$*"; }
ok()    { printf "\033[1;32m%s\033[0m\n" "$*"; }
warn()  { printf "\033[1;33m%s\033[0m\n" "$*"; }
fail()  { printf "\033[1;31m%s\033[0m\n" "$*"; exit 1; }

# --- Cleanup ---
CLEANUP_DIRS=()
_cleanup() {
    local _d
    for _d in "${CLEANUP_DIRS[@]}"; do rm -rf "$_d" 2>/dev/null || true; done
}
trap _cleanup EXIT

_mktemp() {
    mktemp -d
}

# --- Container runtime globals ---
RUNTIME_TYPE=""
RUNTIME_SOCKET_PATH=""
RUNTIME_VERSION=""

# --- Container runtime detection ---
check_docker() {
    if ! command -v docker &>/dev/null; then
        return 1
    fi
    RUNTIME_VERSION=$(docker --version 2>/dev/null | head -1) || true
    if ! docker info &>/dev/null; then
        warn "Docker binary found but daemon not running. Attempting to start..."
        systemctl start docker 2>/dev/null || return 1
        sleep 2
        docker info &>/dev/null || return 1
    fi
    RUNTIME_TYPE="docker"
    RUNTIME_SOCKET_PATH="/var/run/docker.sock"
    return 0
}

check_podman() {
    if ! command -v podman &>/dev/null; then
        return 1
    fi
    RUNTIME_VERSION=$(podman --version 2>/dev/null | head -1) || true
    if [ "$(id -u)" -eq 0 ]; then
        PODMAN_SOCKET="/run/podman/podman.sock"
    else
        PODMAN_SOCKET="/run/user/$(id -u)/podman/podman.sock"
    fi
    if [ -S "$PODMAN_SOCKET" ]; then
        RUNTIME_TYPE="podman"
        RUNTIME_SOCKET_PATH="$PODMAN_SOCKET"
        return 0
    fi
    return 1
}

# --- Container runtime installation ---
install_docker() {
    local DOCKER_TMP
    DOCKER_TMP=$(_mktemp) && CLEANUP_DIRS+=("$DOCKER_TMP")

    info "Downloading Docker convenience script..."
    if curl -fsSL https://get.docker.com -o "${DOCKER_TMP}/get-docker.sh"; then
        sh "${DOCKER_TMP}/get-docker.sh" 2>&1 || true
    else
        warn "Failed to download Docker convenience script. Using package manager..."
    fi

    if command -v docker &>/dev/null; then
        systemctl enable --now docker 2>/dev/null || true
        ok "Docker installed via convenience script"
        return 0
    fi

    info "Attempting Docker installation via package manager..."
    if command -v apt-get &>/dev/null; then
        apt-get update -qq && apt-get install -y -qq docker.io
    elif command -v dnf &>/dev/null; then
        dnf install -y -q docker-ce
    elif command -v yum &>/dev/null; then
        yum install -y -q docker-ce
    else
        fail "Unsupported package manager. Install Docker manually: https://docs.docker.com/engine/install/"
    fi

    systemctl enable --now docker 2>/dev/null || true
    if command -v docker &>/dev/null; then
        ok "Docker installed via package manager"
        return 0
    fi
    return 1
}

# --- Podman socket configuration ---
configure_podman_socket() {
    if [ "$(id -u)" -eq 0 ]; then
        systemctl enable --now podman.socket 2>/dev/null || true
        RUNTIME_SOCKET_PATH="/run/podman/podman.sock"
    else
        systemctl --user enable --now podman.socket 2>/dev/null || true
        loginctl enable-linger 2>/dev/null || true
        RUNTIME_SOCKET_PATH="/run/user/$(id -u)/podman/podman.sock"
    fi
    sleep 1
    if [ -S "$RUNTIME_SOCKET_PATH" ]; then
        RUNTIME_TYPE="podman"
        RUNTIME_VERSION=$(podman --version 2>/dev/null | head -1) || true
        ok "Podman Docker-compatible socket is now active at ${RUNTIME_SOCKET_PATH}"
        return 0
    fi
    RUNTIME_SOCKET_PATH=""
    return 1
}

# --- Ensure container runtime ---
ensure_container_runtime() {
    if check_podman; then
        info "Podman detected — using as container runtime"
        return 0
    fi

    if command -v podman &>/dev/null; then
        warn "Podman found but Docker-compatible socket is inactive. Configuring..."
        if configure_podman_socket; then
            return 0
        fi
        warn "Podman socket configuration failed. Falling back to Docker..."
    fi

    if check_docker; then
        info "Docker detected — using as container runtime"
        return 0
    fi

    info "No container runtime found. Installing Docker..."
    install_docker

    if check_docker; then
        ok "Container runtime ready: Docker"
        return 0
    fi

    fail "Failed to install Docker. Please install Docker or Podman manually and re-run this script."
}

# --- Determine platform ---
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    # Normalise OS
    case "$OS" in
        linux)  OS="linux" ;;
        darwin) OS="linux" ;;   # macOS uses same linux binaries for now
        mingw*|cygwin*)
            fail "This script does not support Windows. Please use install.ps1 instead."
            ;;
        *)      fail "Unsupported operating system: $OS" ;;
    esac

    # Normalise architecture
    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64"  ;;
        aarch64|arm64)  ARCH="aarch64" ;;
        *)             fail "Unsupported architecture: $ARCH" ;;
    esac

    info "Detected platform: ${OS}-${ARCH}"
}

# --- Build download URL ---
build_urls() {
    if [ "$VERSION" = "latest" ]; then
        BASE_URL="https://get.esluce.com/latest"
        info "Using latest version"
    else
        # Strip leading 'v' if present
        V="${VERSION#v}"
        BASE_URL="https://get.esluce.com/v${V}"
        info "Using version: v${V}"
    fi

    ARCHIVE="solys-${OS}-${ARCH}.tar.gz"
    BINARY="escluse-agent"
    ARCHIVE_URL="${BASE_URL}/${ARCHIVE}"
    CHECKSUM_URL="${BASE_URL}/SHA256SUMS.txt"
    BUNDLE_URL="${BASE_URL}/SHA256SUMS.txt.bundle"
}

# --- Download artifacts ---
download_artifacts() {
    TMPDIR=$(_mktemp) && CLEANUP_DIRS+=("$TMPDIR")

    info "Downloading ${ARCHIVE}..."
    curl -fsSL "$ARCHIVE_URL" -o "${TMPDIR}/${ARCHIVE}" || \
        fail "Failed to download ${ARCHIVE_URL}"

    info "Downloading SHA256SUMS.txt..."
    curl -fsSL "$CHECKSUM_URL" -o "${TMPDIR}/SHA256SUMS.txt" || \
        fail "Failed to download ${CHECKSUM_URL}"
}

# --- Verify SHA256 checksum ---
verify_checksum() {
    info "Verifying SHA256 checksum..."
    (cd "$TMPDIR" && sha256sum --ignore-missing -c SHA256SUMS.txt) || \
        fail "Checksum verification failed. Downloaded file may be corrupted."
    ok "Checksum verified successfully."
}

# --- Optional cosign verification ---
verify_cosign() {
    if command -v cosign &>/dev/null; then
        info "Cosign detected. Verifying signature..."
        curl -fsSL "$BUNDLE_URL" -o "${TMPDIR}/SHA256SUMS.txt.bundle" || \
            warn "Failed to download signature bundle. Skipping cosign verification."

        if [ -f "${TMPDIR}/SHA256SUMS.txt.bundle" ]; then
            cosign verify-blob "${TMPDIR}/SHA256SUMS.txt" \
                --bundle "${TMPDIR}/SHA256SUMS.txt.bundle" \
                --certificate-identity-regexp "https://github.com/${REPO}/.github/workflows/release.yml@refs/tags/v" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" || \
                warn "Cosign verification warning: signature could not be verified (non-fatal)."
            ok "Cosign signature verified."
        fi
    else
        info "Cosign not found. Skipping signature verification."
        info "Install cosign from https://docs.sigstore.dev/system_config/installation/"
    fi
}

# --- Extract and install binary ---
install_binary() {
    info "Extracting archive..."
    tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

    if [ ! -f "${TMPDIR}/${BINARY}" ]; then
        fail "Binary ${BINARY} not found in extracted archive."
    fi

    info "Installing ${BINARY} to ${INSTALL_DIR}..."
    if [ "$(id -u)" -eq 0 ]; then
        install -m 755 "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}" || \
            fail "Failed to install binary to ${INSTALL_DIR}"
    else
        sudo install -m 755 "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}" || \
            fail "Failed to install binary to ${INSTALL_DIR} (try running with sudo)"
    fi

    ok "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
}

# --- Interactive config prompts ---
_prompt() {
    local label="$1" var_name="$2" default_val="${3:-}"
    if [ -t 0 ]; then
        read -r -p "$label" "$var_name" || true
    elif [ -c /dev/tty ]; then
        read -r -p "$label" "$var_name" </dev/tty || true
    fi
}

# --- Generate agent config ---
generate_config() {
    local ak_val="" config_dir user_home

    if [ -n "${SUDO_USER:-}" ]; then
        user_home=$(getent passwd "$SUDO_USER" | cut -d: -f6)
    else
        user_home="$HOME"
    fi

    config_dir="${user_home}/.config/escluse-agent"
    mkdir -p "$config_dir" 2>/dev/null || return

    info "Configuring Escluse agent..."
    if [ -n "${AGENT_API_KEY:-}" ]; then
        ak_val="$AGENT_API_KEY"
    else
        _prompt "API Key: " ak_val
    fi

    cat > "${config_dir}/config.toml" <<-CONFIGEOF
# Escluse Agent Configuration
# Generated by install.sh on $(date)

[server]
backend_url = "wss://app.esluce.com/api/ws/node"
api_key = "${ak_val}"

[runtime]
preference = "auto"

[logging]
level = "info"
CONFIGEOF
    ok "Configuration saved to ${config_dir}/config.toml"
}

# --- Post-install message ---
print_success() {
    echo ""
    ok "┌──────────────────────────────────────────────────────────┐"
    ok "│  Escluse Agent installed successfully!                   │"
    ok "│                                                          │"
    ok "│  Binary:  ${INSTALL_DIR}/${BINARY}"
    ok "│  Version: ${VERSION}"
    [ -n "$RUNTIME_TYPE" ] && ok "│  Runtime: ${RUNTIME_TYPE}                                          │"
    [ -n "$RUNTIME_SOCKET_PATH" ] && ok "│  Socket:  ${RUNTIME_SOCKET_PATH}"
    ok "│  Usage:   escluse-agent --help                           │"
    ok "└──────────────────────────────────────────────────────────┘"
    echo ""
    info "Run 'escluse-agent --help' to get started."
    echo ""
    info "Configuration saved to ~/.config/escluse-agent/config.toml"
}

# --- Main ---
root_check() {
    if [ "$(id -u)" -ne 0 ]; then
        fail "This installer must be run as root. Please re-run with: sudo bash -c \"\$(curl -fsSL https://get.esluce.com/latest/install.sh)\""
    fi
}

main() {
    echo ""
    info "Escluse Agent Installer"
    info "======================="
    echo ""

    root_check
    detect_platform
    build_urls
    ensure_container_runtime
    download_artifacts
    verify_checksum
    verify_cosign
    install_binary
    generate_config
    print_success
}

main
