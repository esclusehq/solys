#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"
ARTIFACTS_DIR="$2"

if [ -z "$VERSION" ]; then
  echo "Error: VERSION is empty" >&2
  exit 1
fi

BASE_URL="https://get.esluce.com/v${VERSION}"

cd "$ARTIFACTS_DIR"

# Parse SHA256SUMS.txt - handles both binary (*prefix) and text (space) modes
declare -A SHA256_MAP
while IFS=' ' read -r hash path; do
  [ -z "$hash" ] && continue
  # path may have leading '*' in binary mode
  path="${path#\*}"
  SHA256_MAP["$path"]="$hash"
done < SHA256SUMS.txt

# Build versions.json
jq -n \
  --arg version "$VERSION" \
  --arg url_prefix "$BASE_URL" \
  --arg tar_x64 "${SHA256_MAP[solys-linux-x86_64.tar.gz]:-}" \
  --arg tar_arm64 "${SHA256_MAP[solys-linux-aarch64.tar.gz]:-}" \
  --arg zip_x64 "${SHA256_MAP[solys-windows-x86_64.zip]:-}" \
  --arg deb_amd64 "${SHA256_MAP[escluse-agent_amd64.deb]:-}" \
  --arg deb_arm64 "${SHA256_MAP[escluse-agent_arm64.deb]:-}" \
  --arg rpm_x64 "${SHA256_MAP[escluse-agent-${VERSION}-1.x86_64.rpm]:-}" \
  '{
    latest: $version,
    versions: {
      ($version): {
        "linux-x86_64":   { url: "\($url_prefix)/solys-linux-x86_64.tar.gz",     sha256: $tar_x64 },
        "linux-aarch64":  { url: "\($url_prefix)/solys-linux-aarch64.tar.gz",    sha256: $tar_arm64 },
        "windows-x86_64": { url: "\($url_prefix)/solys-windows-x86_64.zip",      sha256: $zip_x64 },
        "deb-amd64":      { url: "\($url_prefix)/escluse-agent_amd64.deb",       sha256: $deb_amd64 },
        "deb-arm64":      { url: "\($url_prefix)/escluse-agent_arm64.deb",       sha256: $deb_arm64 },
        "rpm-x86_64":     { url: "\($url_prefix)/escluse-agent-\($version)-1.x86_64.rpm", sha256: $rpm_x64 }
      }
    }
  }' > versions.json

echo "Generated versions.json for v${VERSION}"
