#!/usr/bin/env bash
set -euo pipefail
BINARY_PATH="$1"
BINARY_NAME="$2"
ARCH="$3"
VERSION="$4"
OUTPUT_DIR="$5"

PKG_NAME="escluse-agent"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Create RPM build tree
RPMDIR="${TMPDIR}/rpmbuild"
mkdir -p "${RPMDIR}/{BUILD,RPMS,SOURCES,SPECS,SRPMS}"
mkdir -p "${RPMDIR}/BUILDROOT/${PKG_NAME}-${VERSION}-1.${ARCH}/usr/local/bin"

# Copy binary
cp "${BINARY_PATH}/${BINARY_NAME}" "${RPMDIR}/BUILDROOT/${PKG_NAME}-${VERSION}-1.${ARCH}/usr/local/bin/"

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
