#!/usr/bin/env bash
set -euo pipefail
VERSION="$1"
RELEASE_DIR="${2:-.}"

# Read SHA256 checksums from SHA256SUMS.txt
declare -A checksums
while IFS=' ' read -r hash filename; do
  checksums["$filename"]="$hash"
done < "${RELEASE_DIR}/SHA256SUMS.txt"

# Build platforms entry
PLATFORMS_JSON="{}"
for f in solys-linux-x86_64.tar.gz; do
  if [ -n "${checksums[$f]:-}" ]; then
    PLATFORMS_JSON=$(echo "$PLATFORMS_JSON" | jq --arg url "https://get.esluce.com/v${VERSION}/${f}" --arg sha "${checksums[$f]}" '. + {"linux-x86_64": {"url": $url, "sha256": $sha}}')
  fi
done
for f in solys-linux-aarch64.tar.gz; do
  if [ -n "${checksums[$f]:-}" ]; then
    PLATFORMS_JSON=$(echo "$PLATFORMS_JSON" | jq --arg url "https://get.esluce.com/v${VERSION}/${f}" --arg sha "${checksums[$f]}" '. + {"linux-aarch64": {"url": $url, "sha256": $sha}}')
  fi
done
for f in solys-windows-x86_64.zip; do
  if [ -n "${checksums[$f]:-}" ]; then
    PLATFORMS_JSON=$(echo "$PLATFORMS_JSON" | jq --arg url "https://get.esluce.com/v${VERSION}/${f}" --arg sha "${checksums[$f]}" '. + {"windows-x86_64": {"url": $url, "sha256": $sha}}')
  fi
done

# Create new version entry
NEW_ENTRY=$(echo "{}" | jq \
  --arg ver "$VERSION" \
  --arg date "$(date -u +%Y-%m-%d)" \
  --argjson platforms "$PLATFORMS_JSON" \
  --arg checksums_url "https://get.esluce.com/v${VERSION}/SHA256SUMS.txt" \
  --arg sig_url "https://get.esluce.com/v${VERSION}/SHA256SUMS.txt.bundle" \
  '. + {version: $ver, date: $date, platforms: $platforms, checksums_url: $checksums_url, signature_url: $sig_url}'
)

# Version entry for versioned path
echo "$NEW_ENTRY" > "version-entry-${VERSION}.json"

# Build or update versions.json
if [ -f versions.json ]; then
  # Read existing, append, update latest
  jq --argjson new "$NEW_ENTRY" --arg ver "$VERSION" '
    .latest = $ver
    | .versions = ([$new] + .versions)
    ' versions.json > versions.tmp && mv versions.tmp versions.json
else
  echo '{"latest":"'"${VERSION}"'","versions":[]}' | jq --argjson new "$NEW_ENTRY" '.versions = [$new]' > versions.json
fi

echo "Updated versions.json: latest=$VERSION"
