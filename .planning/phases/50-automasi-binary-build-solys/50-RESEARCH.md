# Phase 50: Automasi Build Binary untuk Agent/Solys — Research

**Researched:** 2026-05-27
**Domain:** CI/CD Pipeline — Rust cross-compilation, binary signing, R2 CDN distribution
**Confidence:** HIGH

## Summary

This phase builds a fully automated CI/CD pipeline: push to GitHub → GitHub Actions builds binaries for 3 targets → packages them → signs with Sigstore/cosign → uploads to Cloudflare R2 → serves via Cloudflare CDN at `get.esluce.com`. Two channels exist: **canary** (every push to `main`) and **stable** (tagged semver releases).

Three key technical decisions drive the architecture: (1) use native `ubuntu-24.04-arm` runner for aarch64 instead of cross-compiling from x86 (ARM64 runners are now standard GitHub-hosted at $0.005/min for 2-core); (2) use `x86_64-pc-windows-gnu` target with mingw-w64 cross-compiler on Ubuntu for Windows builds (NOT `x86_64-pc-windows-msvc` which requires a Windows runner); (3) use the `aws` CLI with S3-compatible API to upload to R2 (no third-party action needed), with GitHub OIDC for authentication to avoid storing long-lived R2 tokens.

For signing, `sigstore/cosign-installer@v4.1.0` installs cosign v3.x, and `cosign sign-blob` signs the `SHA256SUMS.txt` file using GitHub OIDC (keyless). The certificate identity includes the workflow ref, enabling users to verify `cosign verify-blob --certificate-identity-regexp`.

**Primary recommendation:** Build a single `release.yml` workflow with matrix strategy for 3 targets, a `canary.yml` workflow for main branch pushes, and implement the CDN upload via raw `aws s3 cp` commands (no action dependency) authenticated via OIDC.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Platforms — Linux x86_64, Linux aarch64 (ARM64), Windows x86_64
- **D-02:** Windows build — cross-compile from Linux runner using mingw-w64 (x86_64-pc-windows-gnu target)
- **D-03:** ARM build — use native GitHub ARM64 runner for Linux aarch64
- **D-04:** Packaging — raw binary tarballs + system packages (.deb, .rpm for Linux; .msi for Windows)
- **D-05:** Version scheme — semver via git tags (push v1.2.3 → triggers release build)
- **D-06:** CDN layout — Per-version directories + latest redirect + versions.json manifest
- **D-07:** Upload schedule — Push to main → canary path; Tagged release → latest + version-pinned
- **D-08:** Triggers — push to main, PR to main (CI only), semver tags, manual workflow dispatch
- **D-09:** Binary integrity — SHA256 checksums + signing with Sigstore/cosign (keyless via GitHub OIDC)
- **D-10:** Credentials — GitHub OIDC for authenticating to Cloudflare R2

### the agent's Discretion
- Installer update mechanism (install.sh / PowerShell script) — standard approach: fetch latest binary + verify checksum + cosign signature

### Deferred Ideas (OUT OF SCOPE)
- None
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Build Rust binaries | CI/CD (GitHub Actions) | — | Cross-compilation requires CI runner matrix; not a runtime concern |
| Cross-compile for targets | CI/CD (GitHub Actions) | — | Uses `dtolnay/rust-toolchain` + target triples; no runtime involvement |
| Sign binaries with cosign | CI/CD (GitHub Actions) | — | Leverages GitHub OIDC for keyless signing; occurs at build time |
| Upload artifacts to R2 | CI/CD (GitHub Actions) | Infrastructure (R2 bucket) | CI authenticates via OIDC and pushes to R2; bucket config is infra |
| Serve binaries via CDN | Infrastructure (Cloudflare) | — | R2 bucket configured with custom domain; CDN caching handled by Cloudflare |
| User install/update | Client (user machine) | CDN (download) | Install script runs on user machine, fetches from CDN |

---

## Standard Stack

### Core

| Library/Tool | Version | Purpose | Why Standard |
|-------------|---------|---------|--------------|
| `dtolnay/rust-toolchain` | `@stable` | Install Rust + targets in CI | Industry standard for Rust CI; used by dtolnay, serde, tokio projects |
| `sigstore/cosign-installer` | `@v4.1.0` | Install cosign for binary signing | Official Sigstore action; includes integrity verification of cosign itself |
| Cloudflare R2 (S3 API) | — | Binary artifact storage | Zero egress fees; S3-compatible; Cloudflare-managed |
| `aws` CLI | v2 | Upload to R2 via S3 API | Pre-installed on ubuntu-latest; `aws s3 cp` works with R2 endpoint |

### Supporting

| Library/Tool | Version | Purpose | When to Use |
|-------------|---------|---------|-------------|
| `taiki-e/setup-cross-toolchain-action` | `@v1` | Install cross-compilation toolchains (mingw, aarch64 GCC) | When cross-compiling from x86 Linux |
| `cargo-zigbuild` | `0.22.3` | Cross-compile Rust with zig as linker | Alternative to system toolchains; simplifies aarch64 cross-compile |
| `softprops/action-gh-release` | `@v2` | Upload to GitHub Releases | If also publishing to GitHub Releases (not R2-only) |
| `dpkg-deb` | system | Build .deb packages | Pre-installed on Ubuntu; minimal wrapper needed |
| `rpmbuild` | system | Build .rpm packages | Requires `rpmdevtools` package |
| `makensis` / NSIS | system | Build Windows .exe installer | For the existing `escluse-agent.nsi` script |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Native ARM64 runner | Cross-compile aarch64 from x86 | Native is simpler, same performance, avoids `cross`/`zigbuild` complexity |
| `aws s3 cp` (raw CLI) | `ryand56/r2-upload-action` | Raw CLI has no third-party dependency; action is slightly simpler for directory uploads |
| mingw-w64 + `x86_64-pc-windows-gnu` | Windows runner + MSVC | Cross-compile is faster (no Windows VM boot); but GNU target may have subtle runtime differences from MSVC |

### ⚠️ Decision D-02 Technical Note

The CONTEXT.md states `x86_64-pc-windows-msvc` target with mingw-w64 cross-compiler, but **mingw-w64 is for the GNU target, not MSVC**. Cross-compiling `x86_64-pc-windows-msvc` from Linux is not possible (requires MSVC toolchain, Windows-only). Two valid approaches:

| Approach | Target | Runner | Cross-compile? | Notes |
|----------|--------|--------|----------------|-------|
| **Recommended** | `x86_64-pc-windows-gnu` | `ubuntu-latest` | ✅ Yes (mingw-w64) | Simplest; matches decision intent |
| Native Windows | `x86_64-pc-windows-msvc` | `windows-latest` | ❌ No (native) | Slower (Windows VM boot); true MSVC runtime |

**Decision needed:** Change target to `x86_64-pc-windows-gnu` (via mingw-w64 on Ubuntu) OR use `windows-latest` runner for native MSVC build.

The existing `Cargo.toml` Windows dependencies (`winapi`, `windows-service`, `tray-item`, etc.) use `cfg(windows)` which works with both `*-windows-gnu` and `*-windows-msvc` targets.

**Installation:**
```bash
# Cross-compilation dependencies (installed in CI via setup-cross-toolchain-action)
# For Windows GNU cross-compile: mingw-w64
# For aarch64 Linux: aarch64-linux-gnu-gcc
```

**Version verification:**
```bash
npm view @sigstore/cosign-installer  # Not an npm package — refer to GitHub releases
# sigstore/cosign-installer latest: v4.1.1 (March 2026)
# cosign latest: v3.0.6 (April 2026)
# cargo-zigbuild latest: 0.22.3 (April 2026)
```

---

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                      CI/CD Pipeline Flow                            │
└─────────────────────────────────────────────────────────────────────┘

  [ git push (main / v* tag) ]
            │
            ▼
  ┌──────────────────────────────────────────────────┐
  │           GitHub Actions Workflow                 │
  │                                                   │
  │  ┌─────────────────────────────────────────────┐  │
  │  │  1. Validate: check version, Cargo.toml     │  │
  │  └─────────────────────────────────────────────┘  │
  │            │                                      │
  │            ▼                                      │
  │  ┌─────────────────────────────────────────────┐  │
  │  │  2. Build Matrix                            │  │
  │  │                                             │  │
  │  │  ┌─────────────────────────────────────┐    │  │
  │  │  │ linux-x86_64  │ ubuntu-latest       │    │  │
  │  │  │ target: x86_64-unknown-linux-gnu    │    │  │
  │  │  └─────────────────────────────────────┘    │  │
  │  │  ┌─────────────────────────────────────┐    │  │
  │  │  │ linux-aarch64 │ ubuntu-24.04-arm    │    │  │
  │  │  │ target: aarch64-unknown-linux-gnu   │    │  │
  │  │  └─────────────────────────────────────┘    │  │
  │  │  ┌─────────────────────────────────────┐    │  │
  │  │  │ windows-x86_64│ ubuntu-latest       │    │  │
  │  │  │ target: x86_64-pc-windows-gnu       │    │  │
  │  │  └─────────────────────────────────────┘    │  │
  │  └─────────────────────────────────────────────┘  │
  │            │                                      │
  │            ▼                                      │
  │  ┌─────────────────────────────────────────────┐  │
  │  │  3. Package                                 │  │
  │  │  ├── .tar.gz (Linux binaries)               │  │
  │  │  ├── .zip (Windows binary)                  │  │
  │  │  ├── .deb (dpkg-deb)                        │  │
  │  │  ├── .rpm (rpmbuild)                        │  │
  │  │  ├── .exe (NSIS installer, Windows)         │  │
  │  │  └── SHA256SUMS.txt (checksums)             │  │
  │  └─────────────────────────────────────────────┘  │
  │            │                                      │
  │            ▼                                      │
  │  ┌─────────────────────────────────────────────┐  │
  │  │  4. Sign (via cosign + GitHub OIDC)          │  │
  │  │  ├── SHA256SUMS.txt                          │  │
  │  │  └── SHA256SUMS.txt.bundle (sigstore bundle) │  │
  │  └─────────────────────────────────────────────┘  │
  │            │                                      │
  │            ▼                                      │
  │  ┌─────────────────────────────────────────────┐  │
  │  │  5. Upload to R2 (aws s3 cp)                │  │
  │  │  ├── v1.2.3/solys-linux-x86_64.tar.gz       │  │
  │  │  ├── latest/solys-linux-x86_64.tar.gz       │  │
  │  │  └── canary/solys-linux-x86_64.tar.gz       │  │
  │  └─────────────────────────────────────────────┘  │
  │            │                                      │
  │            ▼                                      │
  │  ┌─────────────────────────────────────────────┐  │
  │  │  6. Update versions.json manifest           │  │
  │  └─────────────────────────────────────────────┘  │
  └──────────────────────────────────────────────────┘
            │
            ▼
  ┌──────────────────────────────────────────────────┐
  │       Cloudflare CDN (get.esluce.com)             │
  │                                                   │
  │  R2 Bucket (solys-releases) → Custom Domain       │
  │  Cache TTL: 1 hour (binaries), 5 min (manifest)  │
  │                                                   │
  │  ┌────────────────────────────────────────────┐   │
  │  │ get.esluce.com/                             │   │
  │  │ ├── latest/  → (symlink to latest version) │   │
  │  │ ├── v1.2.3/  → (version-pinned)            │   │
  │  │ ├── canary/  → (bleeding edge)             │   │
  │  │ └── versions.json                          │   │
  │  └────────────────────────────────────────────┘   │
  └──────────────────────────────────────────────────┘
            │
            ▼
  ┌──────────────────────────────────────────────────┐
  │  User Install (install.sh / PowerShell)           │
  │                                                   │
  │  1. curl -fsSL get.esluce.com/latest/version      │
  │  2. Download binary for platform                  │
  │  3. Download SHA256SUMS + SHA256SUMS.txt.bundle   │
  │  4. cosign verify-blob SHA256SUMS.txt ...         │
  │  5. sha256sum --check SHA256SUMS.txt              │
  │  6. chmod +x && install to PATH                   │
  └──────────────────────────────────────────────────┘
```

### Recommended GitHub Actions Workflow Structure

```
.github/workflows/
├── release.yml      # Tag push → build, sign, upload (stable)
├── canary.yml       # Push to main → build, sign, upload (canary)
└── ci.yml           # PR to main → build check only (no upload)
```

### Workflow: release.yml (Stable Release)

```yaml
# Multi-job workflow:
# Job 1: validate — extract version from git tag
# Job 2: build (matrix) — build 3 targets in parallel
# Job 3: package — collect artifacts, generate checksums
# Job 4: sign — cosign sign-blob on SHA256SUMS
# Job 5: upload — aws s3 cp to R2 (version + latest paths)
# Job 6: manifest — update versions.json
```

### Workflow: canary.yml (Canary Build)

```yaml
# Same build matrix as release, but uploads to canary/ path
# Overwrites existing canary artifacts (no versioning)
```

### Recommended R2 Bucket Layout

```
Bucket: solys-releases (or escluse-releases)

/v1.2.3/
├── solys-linux-x86_64.tar.gz
├── solys-linux-aarch64.tar.gz
├── solys-windows-x86_64.zip
├── solys_amd64.deb            # Debian x86_64
├── solys_arm64.deb            # Debian ARM64
├── solys.x86_64.rpm           # RPM x86_64
├── solys.aarch64.rpm          # RPM ARM64
├── escluse-agent-setup.exe    # NSIS Windows installer
├── SHA256SUMS.txt             # All checksums
├── SHA256SUMS.txt.bundle      # Cosign signature bundle
└── version.json               # Metadata for this version

/latest/
├── (copies/symlinks to v1.2.3/ content)

/canary/
├── solys-linux-x86_64.tar.gz
├── solys-linux-aarch64.tar.gz
├── solys-windows-x86_64.zip
├── SHA256SUMS.txt
├── SHA256SUMS.txt.bundle
└── version.json

versions.json
```

### versions.json Format

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

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-compilation toolchain setup | Manual apt-get + rustup target add | `dtolnay/rust-toolchain` + `taiki-e/setup-cross-toolchain-action` | Handles target triples, linker config, env vars |
| Cosign installation | Download and verify cosign manually | `sigstore/cosign-installer@v4.1.0` | Verifies cosign integrity during install; handles version management |
| S3-to-R2 upload script | Custom TypeScript/Go uploader | `aws s3 cp` CLI (pre-installed) | R2 is S3-compatible; `aws s3` CLI just works with `--endpoint-url` |
| Debian packaging (from scratch) | Manual `dpkg-deb` control file generation | System `dpkg-deb` with a `debian/` template dir | Standard tooling; template can be committed to repo |
| RPM packaging (from scratch) | Manual `rpmbuild` spec file | System `rpmbuild` with a spec file template | Standard tooling; spec template committed to repo |

**Key insight:** Every problem in this phase has a well-understood solution. The complexity is in wiring them together correctly, not in building anything novel.

---

## Common Pitfalls

### Pitfall 1: Cross-compile target mismatch (D-02)
**What goes wrong:** Building for `x86_64-pc-windows-msvc` on Linux fails because MSVC toolchain isn't available.
**Why it happens:** mingw-w64 provides the GNU toolchain for Windows, not MSVC. The `msvc` target requires Windows + Visual Studio Build Tools.
**How to avoid:** Use `x86_64-pc-windows-gnu` target (GNU/MinGW) when cross-compiling from Linux. If MSVC is absolutely required, use a `windows-latest` runner with native build.
**Warning signs:** `.cargo/config.toml` tries to set a linker that doesn't exist for MSVC.

### Pitfall 2: OIDC credentials not propagated to `aws` CLI
**What goes wrong:** `aws s3 cp` command fails with "Unable to locate credentials" despite OIDC being configured for the workflow.
**Why it happens:** GitHub OIDC generates a JWT but the `aws` CLI doesn't natively understand it. You need to either (a) use `aws-actions/configure-aws-credentials` with the OIDC role, or (b) for R2, generate R2 API tokens and store as secrets.
**How to avoid:** Simplest approach for R2 is to use R2 API tokens (access key + secret) stored as GitHub secrets. Full OIDC to R2 requires a Cloudflare Worker acting as STS. **D-10 decision** requires OIDC, which means this needs investigation — likely a hybrid: OIDC for trust, R2 tokens for access.
**Warning signs:** Any `403` or credential errors from `aws s3 cp` commands.

### Pitfall 3: Rust path dependencies break in CI
**What goes wrong:** `cargo build` fails because `agent-proto`, `agent-config`, etc. path dependencies (`agent-core/crates/`) can't be resolved.
**Why it happens:** The working directory must be the workspace root, or the `--manifest-path` must point to `agent/solys/Cargo.toml`.
**How to avoid:** Always run `cargo build --manifest-path agent/solys/Cargo.toml` in CI, or set working-directory. The checkout is at repo root, so relative paths resolve correctly.
**Warning signs:** Error: "failed to get `agent-proto` as a dependency" — path doesn't exist relative to build context.

### Pitfall 4: ARM64 runner not available for private repos
**What goes wrong:** `runs-on: ubuntu-24.04-arm` fails with "No runner matching the specified runs-on."
**Why it happens:** As of May 2026, ARM64 standard runners are available in all repositories (both public and private) [VERIFIED: GitHub blog, Jan 29, 2026]. However, migration from Arm-managed to GitHub-managed images was ongoing until June 12, 2026.
**How to avoid:** Use `ubuntu-24.04-arm` — it's available. If issues persist, fallback to cross-compiling aarch64 on x86 runner using `cargo-zigbuild`.
**Warning signs:** CI job hangs or fails with runner provisioning errors.

### Pitfall 5: R2 `latest/` symlink semantics
**What goes wrong:** The `latest/` directory doesn't automatically point to the newest version.
**Why it happens:** R2/S3 doesn't support filesystem symlinks or directory-level redirects. What you see as "latest/solys-linux-x86_64.tar.gz" must be an actual copy of the file, uploaded fresh on each release.
**How to avoid:** Upload the same artifact files to both `v1.2.3/` and `latest/` paths in the same workflow step. Use `aws s3 cp` with `--copy-props none` to avoid copying metadata.
**Warning signs:** Users download old versions from `latest/` after a new release.

---

## Code Examples

### Pattern 1: Release Workflow — Build Matrix (skeleton)

```yaml
# Source: Pattern derived from dtolnay/rust-toolchain docs, taiki-e/setup-cross-toolchain-action docs
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
            pkg_arch_amd64: amd64
            pkg_arch_arm64: ""
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-24.04-arm
            ext: ""
            artifact: solys-linux-aarch64.tar.gz
            pkg_arch_amd64: ""
            pkg_arch_arm64: arm64
          - target: x86_64-pc-windows-gnu
            os: ubuntu-latest
            ext: ".exe"
            artifact: solys-windows-x86_64.zip
            pkg_arch_amd64: ""
            pkg_arch_arm64: ""

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

### Pattern 2: Cosign Keyless Signing

```yaml
# Source: sigstore/cosign-installer docs, verified against GitHub release assets
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

# User verification command:
# cosign verify-blob SHA256SUMS.txt \
#   --bundle SHA256SUMS.txt.bundle \
#   --certificate-identity-regexp "https://github.com/escluse/escluse/.github/workflows/release.yml@refs/tags/v" \
#   --certificate-oidc-issuer "https://token.actions.githubusercontent.com"
```

### Pattern 3: Upload to R2 via aws s3 cp

```yaml
# Source: Cloudflare R2 docs — S3-compatible API
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

### Pattern 4: Install Script (install.sh) Skeleton

```bash
#!/usr/bin/env bash
# Source: Pattern from anchore/syft install.sh, basecamp/hey-cli install.sh
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

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Cross-compile ARM from x86 via `cross` tool | Native ARM64 GitHub runner (`ubuntu-24.04-arm`) | Jan 2026 | Eliminates cross-compile complexity for aarch64; native performance |
| Cosign v2 (separate `.sig` + `.pem` files) | Cosign v3 (`.sigstore.json` bundle format) | Oct 2025 | Single bundle file instead of two; default protobuf format |
| `actions-rs/cargo` (deprecated) | `dtolnay/rust-toolchain` + raw `cargo` | 2023 | Active maintenance; simpler interface |

**Deprecated/outdated:**
- `actions-rs/cargo` v1 — The most popular Rust GitHub Action is no longer maintained. Use `dtolnay/rust-toolchain` + direct `cargo` commands instead. [VERIFIED: github.com/actions-rs]
- COSIGN_EXPERIMENTAL=1 — No longer needed for keyless signing in cosign v2+; keyless is now default.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `x86_64-pc-windows-gnu` target (NOT `msvc`) is the correct cross-compile target from Linux | Standard Stack | If MSVC is truly required, need `windows-latest` runner instead of cross-compile |
| A2 | R2 API tokens (access key + secret) are acceptable despite D-10 preference for OIDC | Common Pitfalls | If OIDC-only is strictly required, need a Cloudflare Worker for STS-like token exchange |
| A3 | The `aws` CLI is pre-installed on `ubuntu-24.04-arm` runner | Code Examples | May need manual install on ARM runner (`pip3 install awscli`) |

---

## Open Questions (RESOLVED)

All questions below have been resolved during planning. See plan implementations for details.

1. **R2 OIDC authentication (D-10 constraint)** — RESOLVED
   - Resolution: Use R2 API tokens (Access Key ID + Secret Access Key) stored as GitHub Actions secrets, scoped to the bucket. User confirmed this approach. D-10 updated in CONTEXT.md accordingly.

2. **System package generation complexity** — RESOLVED
   - Resolution: Hand-write minimal `debian/control`, RPM `.spec`, and NSIS `.nsi` templates. The plan generates `.deb` via `dpkg-deb`, `.rpm` via `rpmbuild`, and Windows `.exe` installer via `makensis`. User confirmed NSIS `.exe` for Windows (not `.msi`). D-04 updated in CONTEXT.md to reflect this.

3. **Existing NSIS installer integration** — RESOLVED
   - Resolution: Build NSIS `.exe` installer in CI by installing `makensis` on the Ubuntu runner. The existing `escluse-agent.nsi` is referenced and enhanced in the plan.

4. **Memory/disk constraints on ARM64 runner** — RESOLVED
   - Resolution: Accepted. If OOM occurs on `ubuntu-24.04-arm` runner, fall back to `lto = "thin"` for CI builds. Not blocking for initial implementation.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| GitHub Actions | Pipeline executor | ✅ | — | — |
| `ubuntu-latest` runner | Linux x86_64 build | ✅ | 24.04 LTS | — |
| `ubuntu-24.04-arm` runner | Linux aarch64 build | ✅ | 24.04 LTS (GitHub-managed) | Cross-compile with cargo-zigbuild |
| `dtolnay/rust-toolchain` | Rust setup | ✅ | @stable | — |
| `sigstore/cosign-installer` | Cosign install | ✅ | v4.1.0 | Manual cosign download |
| `aws` CLI | R2 upload | ✅ (ubuntu-latest) | v2 | `pip3 install awscli` for ARM |
| Cloudflare R2 | Artifact storage | Depends on infra | — | S3 (if available) |
| `makensis` / NSIS | Windows installer build | ❌ (not preinstalled) | — | `sudo apt-get install nsis` |
| `rpmbuild` | RPM packaging | ❌ (not preinstalled) | — | `sudo apt-get install rpm` |
| `dpkg-deb` | DEB packaging | ✅ (preinstalled) | — | — |

**Missing dependencies with no fallback:**
- None — all dependencies have viable install or fallback strategies.

**Missing dependencies with fallback:**
- `makensis` (NSIS) — install via `sudo apt-get install nsis`
- `rpmbuild` (RPM) — install via `sudo apt-get install rpm`
- `aws` CLI on ARM runner — install via `pip3 install awscli`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | Yes | R2 API tokens scoped to bucket-level write |
| V5 Input Validation | No | — |
| V6 Cryptography | Yes | SHA256 for checksums; cosign with Fulcio + Rekor for signing |
| V7 Error Handling | No | — |
| V8 Data Protection | Yes | Binary signing ensures supply chain integrity |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tampered binary download | Tampering | SHA256SUMS verification + cosign signature on checksums |
| Compromised CI credentials | Elevation of Privilege | Short-lived R2 tokens; GitHub OIDC eliminates long-lived secrets |
| Replay of old signed artifacts | Spoofing | Rekor transparency log records all signing events |
| Dependency confusion in CI | Tampering | `cargo build --locked` pins dependency versions via Cargo.lock |

---

## Sources

### Primary (HIGH confidence)
- [sigstore/cosign-installer GitHub](https://github.com/sigstore/cosign-installer) — verified v4.1.0, cosign v3.0.6
- [sigstore/cosign README](https://github.com/sigstore/cosign) — keyless blob signing flow
- [Cloudflare R2 S3 API docs](https://developers.cloudflare.com/r2/examples/authenticate-r2-auth-tokens/) — S3-compatible upload pattern
- [dtolnay/rust-toolchain GitHub](https://github.com/dtolnay/rust-toolchain) — stable Rust + targets in CI
- [taiki-e/setup-cross-toolchain-action](https://github.com/taiki-e/setup-cross-toolchain-action) — cross-compilation toolchain setup
- [cargo-zigbuild crates.io](https://crates.io/crates/cargo-zigbuild) — v0.22.3, cross-compile with zig

### Secondary (MEDIUM confidence)
- [GitHub ARM64 runner announcement](https://github.blog/changelog/2026-01-29-arm64-standard-runners-are-now-available-in-private-repositories/) — standard ARM64 runners available in all repos, Jan 2026
- [ARM64 runner pricing](https://github.com/github/docs/blob/main/content/billing/reference/actions-runner-pricing.md) — $0.005/min for 2-core ARM Linux
- [GitHub Actions Ubuntu 24.04 ARM image](https://github.blog/changelog/2024-06-24-github-actions-ubuntu-24-04-image-now-available-for-arm64-runners/) — ubuntu-24.04-arm available
- [Anchor Syft install.sh](https://github.com/anchore/syft/blob/main/install.sh) — pattern for cosign-signed install scripts
- [basecamp/hey-cli install.sh](https://github.com/basecamp/hey-cli/blob/main/scripts/install.sh) — pattern for SHA256 + cosign verification

### Tertiary (LOW confidence)
- Cross-platform Rust pipeline blog posts (multiple medium articles) — consistent with primary sources, no contradictions

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all tools verified via GitHub/registry
- Architecture: HIGH — pattern derived from verified tool behavior
- Pitfalls: HIGH — verified via documentation and community knowledge
- Security: HIGH — R2 OIDC resolved: API tokens (confirmed by user)

**Research date:** 2026-05-27
**Valid until:** 2026-06-27 (30 days; tools are stable, CI patterns change slowly)
