#!/usr/bin/env bash
set -euo pipefail
BINARY_PATH="$1"
BINARY_NAME="$2"
ARCH="$3"
VERSION="$4"
if [ -z "$VERSION" ]; then
  echo "Error: VERSION is empty" >&2
  exit 1
fi
OUTPUT_DIR="$5"

PKG_NAME="escluse-agent"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Create package directory structure
PKGDIR="${TMPDIR}/${PKG_NAME}_${VERSION}_${ARCH}"
mkdir -p "${PKGDIR}/DEBIAN"
mkdir -p "${PKGDIR}/usr/local/bin"

# Copy binary
cp "${BINARY_PATH}/${BINARY_NAME}" "${PKGDIR}/usr/local/bin/"

# Generate control file from template
cat > "${PKGDIR}/DEBIAN/control" <<- CTRL
Package: ${PKG_NAME}
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: Escluse Team <team@escluse.com>
Description: Escluse Agent - Background service for managing game servers
 Escluse Agent connects your server to the Escluse platform, enabling
 game server management through a web control panel.
CTRL

# Build .deb
dpkg-deb --build "${PKGDIR}" "${OUTPUT_DIR}/${PKG_NAME}_${ARCH}.deb"
echo "Created ${OUTPUT_DIR}/${PKG_NAME}_${ARCH}.deb"
