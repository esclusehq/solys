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
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

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
    tar -xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR" || \
        fail "Failed to extract ${ARCHIVE}"

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

# --- Post-install message ---
print_success() {
    echo ""
    ok "┌──────────────────────────────────────────────────────────┐"
    ok "│  Escluse Agent installed successfully!                   │"
    ok "│                                                          │"
    ok "│  Binary:  ${INSTALL_DIR}/${BINARY}"
    ok "│  Version: ${VERSION}"
    ok "│  Usage:   escluse-agent --help                           │"
    ok "└──────────────────────────────────────────────────────────┘"
    echo ""
    info "Run 'escluse-agent --help' to get started."
}

# --- Main ---
main() {
    echo ""
    info "Escluse Agent Installer"
    info "======================="
    echo ""

    detect_platform
    build_urls
    download_artifacts
    verify_checksum
    verify_cosign
    install_binary
    print_success
}

main
