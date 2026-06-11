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

HOST_ARCH=$(uname -m)
case "$HOST_ARCH" in
  x86_64|amd64) HOST_ARCH="x86_64" ;;
  aarch64|arm64) HOST_ARCH="aarch64" ;;
esac
if [ "$HOST_ARCH" != "$ARCH" ]; then
  echo "Warning: skipping RPM build for $ARCH on $HOST_ARCH (host mismatch)"
  exit 0
fi

PKG_NAME="escluse-agent"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Create RPM build tree
RPMDIR="${TMPDIR}/rpmbuild"
mkdir -p "${RPMDIR}/BUILD" "${RPMDIR}/RPMS" "${RPMDIR}/SOURCES" "${RPMDIR}/SPECS" "${RPMDIR}/SRPMS"
# Copy binary to SOURCES (spec %install handles BUILDROOT)
cp "${BINARY_PATH}/${BINARY_NAME}" "${RPMDIR}/SOURCES/"

# Create spec file
cat > "${RPMDIR}/SPECS/${PKG_NAME}.spec" <<- SPEC
Name: ${PKG_NAME}
Version: ${VERSION}
Release: 1%{?dist}
Summary: Escluse Agent - Game server management agent
License: MIT
URL: https://escluse.com
BuildArch: ${ARCH}

%description
Escluse Agent connects your server to the Escluse platform, enabling
game server management through a web control panel.

%install
mkdir -p %{buildroot}/usr/local/bin
cp %{_sourcedir}/${BINARY_NAME} %{buildroot}/usr/local/bin/

%files
/usr/local/bin/${BINARY_NAME}

%post
# Post-install: ensure executable
chmod 755 /usr/local/bin/${BINARY_NAME}

%changelog
* $(date '+%a %b %d %Y') Escluse Team <team@escluse.com> - ${VERSION}-1
- Automated build for version ${VERSION}
SPEC

# Build RPM
rpmbuild --define "_topdir ${RPMDIR}" -bb "${RPMDIR}/SPECS/${PKG_NAME}.spec"
find "${RPMDIR}/RPMS" -name "*.rpm" -exec cp {} "${OUTPUT_DIR}/" \;
echo "Created RPM for ${ARCH} in ${OUTPUT_DIR}"
