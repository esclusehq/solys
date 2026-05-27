# Phase 50: Automasi Build Binary untuk Agent/Solys — Pattern Map

**Mapped:** 2026-05-27
**Files analyzed:** 6 new files, 0 modified files
**Analogs found:** 3 with partial / 6 total

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `.github/workflows/release.yml` | workflow | event-driven | `worker/Dockerfile` (Rust build pattern) + RESEARCH.md Pattern 1 | no-repo-analog |
| `.github/workflows/canary.yml` | workflow | event-driven | `release.yml` (same structure, simplified upload) | no-repo-analog |
| `.github/workflows/ci.yml` | workflow | event-driven | `release.yml` (build matrix only, no package/sign/upload) | no-repo-analog |
| `install.sh` | utility | file-I/O | RESEARCH.md Pattern 4 (anchore/syft style install.sh) | no-repo-analog |
| `install.ps1` | utility | file-I/O | RESEARCH.md Pattern 4 (PowerShell variant for Windows) | no-repo-analog |
| `agent/solys/installer/escluse-agent.nsi` | config | transform | Already exists (needs updated version path for CI) | exact-match |

## Pattern Assignments

### `.github/workflows/release.yml` (workflow, event-driven)

**Analog:** No existing GitHub Actions in repo. Use RESEARCH.md Pattern 1 (lines 356–440) and Pattern 3 (lines 470–493) as primary source. Use `worker/Dockerfile` for project-specific Rust build conventions.

**Workflow structure (RESEARCH.md lines 214–233):**
```yaml
# Multi-job workflow:
# Job 1: validate — extract version from git tag
# Job 2: build (matrix) — build 3 targets in parallel
# Job 3: package — collect artifacts, generate checksums
# Job 4: sign — cosign sign-blob on SHA256SUMS
# Job 5: upload — aws s3 cp to R2 (version + latest paths)
# Job 6: manifest — update versions.json
```

**Build matrix pattern (RESEARCH.md lines 356–440):**
```yaml
name: Release
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always
  BINARY_NAME: escluse-agent
  PROJECT_PATH: agent/solys

jobs:
  validate:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
    steps:
      - uses: actions/checkout@v4
      - id: version
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"

  build:
    needs: [validate]
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            ext: ""
            artifact: solys-linux-x86_64.tar.gz
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-24.04-arm
            ext: ""
            artifact: solys-linux-aarch64.tar.gz
          - target: x86_64-pc-windows-gnu
            os: ubuntu-latest
            ext: ".exe"
            artifact: solys-windows-x86_64.zip

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross-compilation tools
        uses: taiki-e/setup-cross-toolchain-action@v1
        with:
          target: ${{ matrix.target }}
      - name: Build
        run: |
          cargo build --release --manifest-path ${{ env.PROJECT_PATH }}/Cargo.toml \
            --target ${{ matrix.target }}
      - name: Stage binary
        working-directory: ${{ env.PROJECT_PATH }}
        run: |
          mkdir -p staging
          SRC="target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}${{ matrix.ext }}"
          cp "$SRC" "staging/${{ env.BINARY_NAME }}${{ matrix.ext }}"
      - name: Upload build artifact
        uses: actions/upload-artifact@v4
        with:
          name: binary-${{ matrix.target }}
          path: ${{ env.PROJECT_PATH }}/staging/
```

**Rust toolchain pattern (from `worker/Dockerfile` lines 1–13):**
```dockerfile
FROM rust:1.88-alpine3.20 AS builder
RUN apk add --no-cache openssl-dev musl-dev pkgconfig
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include
RUN cargo build --release --bin worker
```
→ In CI, `dtolnay/rust-toolchain@stable` replaces the manual `rustup` + installs specific target triples.

**Cosign signing pattern (RESEARCH.md lines 442–468):**
```yaml
- name: Install cosign
  uses: sigstore/cosign-installer@v4.1.0

- name: Generate SHA256 checksums
  working-directory: release/
  run: sha256sum solys-* escluse-* > SHA256SUMS.txt

- name: Sign checksums with cosign (keyless, GitHub OIDC)
  env:
    COSIGN_EXPERIMENTAL: "1"
  working-directory: release/
  run: |
    cosign sign-blob SHA256SUMS.txt \
      --bundle SHA256SUMS.txt.bundle \
      --output-signature SHA256SUMS.txt.sig \
      --output-certificate SHA256SUMS.txt.pem
```

**User verification command (from RESEARCH.md lines 463–468):**
```bash
cosign verify-blob SHA256SUMS.txt \
  --bundle SHA256SUMS.txt.bundle \
  --certificate-identity-regexp "https://github.com/escluse/escluse/.github/workflows/release.yml@refs/tags/v" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com"
```

**R2 upload pattern (RESEARCH.md lines 470–493):**
```yaml
- name: Upload to R2
  env:
    AWS_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY_ID }}
    AWS_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_ACCESS_KEY }}
    AWS_DEFAULT_REGION: auto
    R2_ENDPOINT: https://${{ secrets.CLOUDFLARE_ACCOUNT_ID }}.r2.cloudflarestorage.com
    R2_BUCKET: ${{ secrets.R2_BUCKET }}
    VERSION: ${{ needs.validate.outputs.version }}
  run: |
    # Upload to versioned path
    aws s3 cp release/ "s3://${R2_BUCKET}/v${VERSION}/" \
      --endpoint-url "${R2_ENDPOINT}" \
      --recursive
    # Copy to latest/
    aws s3 cp "s3://${R2_BUCKET}/v${VERSION}/" "s3://${R2_BUCKET}/latest/" \
      --endpoint-url "${R2_ENDPOINT}" \
      --recursive \
      --copy-props none
```

**CRITICAL: Manifest update pattern (RESEARCH.md lines 274–301):**
```json
{
  "latest": "1.2.3",
  "versions": [
    {
      "version": "1.2.3",
      "date": "2026-05-27",
      "platforms": {
        "linux-x86_64": {
          "url": "https://get.esluce.com/v1.2.3/solys-linux-x86_64.tar.gz",
          "sha256": "abc123..."
        },
        "linux-aarch64": {
          "url": "https://get.esluce.com/v1.2.3/solys-linux-aarch64.tar.gz",
          "sha256": "def456..."
        },
        "windows-x86_64": {
          "url": "https://get.esluce.com/v1.2.3/solys-windows-x86_64.zip",
          "sha256": "ghi789..."
        }
      },
      "checksums_url": "https://get.esluce.com/v1.2.3/SHA256SUMS.txt",
      "signature_url": "https://get.esluce.com/v1.2.3/SHA256SUMS.txt.bundle"
    }
  ]
}
```

**IMPORTANT: Project-specific Cargo.toml conventions (lines 74–103):**
- Binary names: `escluse-agent`, `escluse-service` (both built from `agent/solys`)
- Profile release: `lto = "fat"`, `codegen-units = 1`, `strip = true`, `opt-level = 3`
- Build path: `--manifest-path agent/solys/Cargo.toml` or set `working-directory: agent/solys`
- Release binary output at: `agent/solys/target/${{ matrix.target }}/release/escluse-agent${{ matrix.ext }}`

---

### `.github/workflows/canary.yml` (workflow, event-driven)

**Analog:** Same as `release.yml` structure. Differs in:
- Trigger: `push: branches: [main]` (not tags)
- Upload path: `canary/` instead of `v{VERSION}/` + `latest/`
- No `validate` job needed (no version extraction from tag — use commit SHA or timestamp)
- Version.json uses `"canary"` as identifier with build timestamp

**Canary trigger pattern:**
```yaml
on:
  push:
    branches:
      - main
  workflow_dispatch:
```

**Canary upload pattern (overwrite not version):**
```yaml
- name: Upload to R2 (canary)
  env:
    AWS_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY_ID }}
    AWS_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_ACCESS_KEY }}
    AWS_DEFAULT_REGION: auto
    R2_ENDPOINT: https://${{ secrets.CLOUDFLARE_ACCOUNT_ID }}.r2.cloudflarestorage.com
    R2_BUCKET: ${{ secrets.R2_BUCKET }}
  run: |
    # Upload to canary path (overwrites previous canary)
    aws s3 cp release/ "s3://${R2_BUCKET}/canary/" \
      --endpoint-url "${R2_ENDPOINT}" \
      --recursive
```

---

### `.github/workflows/ci.yml` (workflow, event-driven)

**Analog:** Same build matrix as `release.yml`. Differs in:
- Trigger: `pull_request: branches: [main]`
- Only build step (no packaging, signing, upload)
- `fail-fast: true` is acceptable (CI should fail fast)

**CI-only trigger pattern:**
```yaml
on:
  pull_request:
    branches:
      - main
```

**CI-only build (copy from RESEARCH.md lines 355–439, but only the build step, no post-build jobs):**
```yaml
- name: Build
  run: |
    cargo build --release --manifest-path ${{ env.PROJECT_PATH }}/Cargo.toml \
      --target ${{ matrix.target }}
```

---

### `install.sh` (utility, file-I/O)

**Analog:** No existing shell scripts in repo. Use RESEARCH.md Pattern 4 (lines 495–561) — derived from anchore/syft install.sh and basecamp/hey-cli install.sh.

**Core installer pattern (RESEARCH.md lines 497–561):**
```bash
#!/usr/bin/env bash
set -euo pipefail

REPO="escluse/escluse"
INSTALL_DIR="${ESCLUSE_BIN_DIR:-/usr/local/bin}"
VERSION="${1:-latest}"

# Determine platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
esac

# Build download URL
if [ "$VERSION" = "latest" ]; then
  BASE_URL="https://get.esluce.com/latest"
else
  BASE_URL="https://get.esluce.com/v${VERSION}"
fi

case "$OS" in
  linux)
    ARCHIVE="solys-linux-${ARCH}.tar.gz"
    BINARY="escluse-agent"
    ;;
  mingw*|cygwin*)
    ARCHIVE="solys-windows-${ARCH}.zip"
    BINARY="escluse-agent.exe"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

# Download
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading $BINARY..."
curl -fsSL "${BASE_URL}/${ARCHIVE}" -o "${TMPDIR}/${ARCHIVE}"
curl -fsSL "${BASE_URL}/SHA256SUMS.txt" -o "${TMPDIR}/SHA256SUMS.txt"

# Verify checksum
echo "Verifying checksum..."
(cd "$TMPDIR" && sha256sum --ignore-missing -c SHA256SUMS.txt)

# Verify cosign signature (if cosign installed)
if command -v cosign &>/dev/null; then
  curl -fsSL "${BASE_URL}/SHA256SUMS.txt.bundle" -o "${TMPDIR}/SHA256SUMS.txt.bundle"
  cosign verify-blob "${TMPDIR}/SHA256SUMS.txt" \
    --bundle "${TMPDIR}/SHA256SUMS.txt.bundle" \
    --certificate-identity-regexp "https://github.com/${REPO}/.github/workflows/release.yml@refs/tags/v" \
    --certificate-oidc-issuer "https://token.actions.githubusercontent.com"
fi

# Extract and install
tar -xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"
sudo install -m 755 "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
```

---

### `install.ps1` (utility, file-I/O)

**Analog:** No existing PowerShell scripts in repo. Windows PowerShell variant of `install.sh`. Must handle:
- URL determination (latest vs version-pinned)
- Platform detection using `$env:PROCESSOR_ARCHITECTURE`
- Download via `Invoke-WebRequest`
- SHA256 verification via `Get-FileHash`
- Cosign verification (if cosign.exe in PATH)
- Binary extraction (zip from `solys-windows-x86_64.zip`)
- Install to `$env:ProgramFiles\Escluse` or `$env:LOCALAPPDATA\Escluse`

**PowerShell installer pattern (inferred from install.sh logic, adapted for Windows):**
```powershell
#!/usr/bin/env pwsh
param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"
$Repo = "escluse/escluse"
$InstallDir = Join-Path $env:ProgramFiles "Escluse"

# Determine platform
$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64"  { "x86_64" }
    "ARM64"  { "aarch64" }
    default  { throw "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}

# Build download URL
if ($Version -eq "latest") {
    $baseUrl = "https://get.esluce.com/latest"
} else {
    $baseUrl = "https://get.esluce.com/v$Version"
}

$archive = "solys-windows-${arch}.zip"
$binary = "escluse-agent.exe"

$tmpDir = Join-Path $env:TEMP "escluse-install"
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

Write-Host "Downloading $binary..."
Invoke-WebRequest -Uri "$baseUrl/$archive" -OutFile (Join-Path $tmpDir $archive)
Invoke-WebRequest -Uri "$baseUrl/SHA256SUMS.txt" -OutFile (Join-Path $tmpDir "SHA256SUMS.txt")

# Verify checksum
Write-Host "Verifying checksum..."
$expectedHash = (Get-Content (Join-Path $tmpDir "SHA256SUMS.txt") | Select-String $binary) -split '\s+' | Select-Object -First 1
$actualHash = (Get-FileHash (Join-Path $tmpDir $archive) -Algorithm SHA256).Hash.ToLower()
if ($actualHash -ne $expectedHash) {
    throw "Checksum mismatch for $archive"
}

# Extract
Expand-Archive -Path (Join-Path $tmpDir $archive) -DestinationPath $tmpDir -Force

# Install
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item (Join-Path $tmpDir $binary) (Join-Path $InstallDir $binary) -Force

# Add to PATH if not already
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
}

Write-Host "Installed $binary to $InstallDir"
```

---

### `agent/solys/installer/escluse-agent.nsi` (config, transform) — Already Exists

**Purpose:** Build Windows NSIS installer for the agent.
**In CI:** Run `makensis escluse-agent.nsi` from `agent/solys/installer/` directory.
**Key references:** This NSIS script expects binaries at `../../release/package/` relative path.
**CI adjustment needed:** The binary source path in CI will be different — CI stages binaries in `agent/solys/staging/`. The NSIS script or CI packaging step must account for this path difference.

**Existing NSIS binary source (lines 59–62):**
```nsis
File "..\..\release\package\escluse-agent.exe"
File "..\..\release\package\escluse-service.exe"
File "..\..\release\package\escluse-gui.exe"
```

**CI approach:** Copy staged binaries into `agent/solys/installer/` before running `makensis`, OR update the NSIS source path to match the CI staging directory.

---

## Shared Patterns

### Build Matrix Environment Variables
**Source:** RESEARCH.md lines 367–370
**Apply to:** `release.yml`, `canary.yml`, `ci.yml`
```yaml
env:
  CARGO_TERM_COLOR: always
  BINARY_NAME: escluse-agent
  PROJECT_PATH: agent/solys
```

### Cross-compilation Toolchain Setup
**Source:** RESEARCH.md lines 411–421
**Apply to:** All workflow files
```yaml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable
  with:
    targets: ${{ matrix.target }}

- name: Install cross-compilation tools
  uses: taiki-e/setup-cross-toolchain-action@v1
  with:
    target: ${{ matrix.target }}
```

### Binary Names and Output Paths
**Source:** `agent/solys/Cargo.toml` lines 74–80
**Apply to:** All workflow build steps
```toml
[[bin]]
name = "escluse-agent"
path = "src/main.rs"

[[bin]]
name = "escluse-service"
path = "src/service_main.rs"
```
Output: `agent/solys/target/{target}/release/escluse-agent{ext}`

### Release Profile Optimization
**Source:** `agent/solys/Cargo.toml` lines 93–99
**Apply to:** All release builds in CI
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
overflow-checks = false
```

### R2 Upload Authentication
**Source:** RESEARCH.md lines 329–331, 474–493
**Apply to:** `release.yml`, `canary.yml`
- D-10 requires OIDC but R2 doesn't natively support it
- Current recommendation: Use R2 API tokens (Access Key + Secret) stored as GitHub secrets
- Required secrets: `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `CLOUDFLARE_ACCOUNT_ID`, `R2_BUCKET`
```yaml
env:
  AWS_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY_ID }}
  AWS_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_ACCESS_KEY }}
  AWS_DEFAULT_REGION: auto
  R2_ENDPOINT: https://${{ secrets.CLOUDFLARE_ACCOUNT_ID }}.r2.cloudflarestorage.com
  R2_BUCKET: ${{ secrets.R2_BUCKET }}
```

### Package Structure Per Target
**Apply to:** Build matrix packaging step in all workflow files
| Target | Binary Ext | Archive Format | System Package |
|--------|-----------|----------------|----------------|
| `x86_64-unknown-linux-gnu` | (none) | `.tar.gz` | `.deb` (amd64), `.rpm` (x86_64) |
| `aarch64-unknown-linux-gnu` | (none) | `.tar.gz` | `.deb` (arm64), `.rpm` (aarch64) |
| `x86_64-pc-windows-gnu` | `.exe` | `.zip` | `.exe` (NSIS installer) |

### R2 Bucket Layout
**Source:** RESEARCH.md lines 243–272
**Apply to:** All upload steps
```
Bucket: solys-releases
/v{VERSION}/   → version-pinned release artifacts
/latest/       → copied from v{VERSION} on each release
/canary/       → overwritten on each push to main
versions.json  → root-level manifest of all versions
```

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `.github/workflows/release.yml` | workflow | event-driven | No existing GitHub Actions workflows in repo |
| `.github/workflows/canary.yml` | workflow | event-driven | No existing GitHub Actions workflows in repo |
| `.github/workflows/ci.yml` | workflow | event-driven | No existing GitHub Actions workflows in repo |
| `install.sh` | utility | file-I/O | No existing shell scripts in entire repo |
| `install.ps1` | utility | file-I/O | No existing PowerShell scripts in entire repo |

## Metadata

**Analog search scope:** Entire repo (`*.yml`, `*.yaml`, `*.sh`, `*.bash`, `*.ps1`, `*.psm1`, workflows, Dockerfiles)
**Files scanned:** 15+ (all directories checked for CI/CD, shell scripts, Dockerfiles)
**Pattern extraction date:** 2026-05-27
