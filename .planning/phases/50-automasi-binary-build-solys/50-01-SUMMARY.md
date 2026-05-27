---
phase: 50-automasi-binary-build-solys
plan: 01
subsystem: ci-cd
tags: github-actions, rust, cross-compilation, cosign, r2, debian, rpm, nsis

requires: []
provides:
  - Release workflow (release.yml) — 6 jobs, 3-platform build matrix, cosign signing, R2 upload
  - Canary workflow (canary.yml) — push-to-main build, sign, upload to canary/ path
  - CI workflow (ci.yml) — PR build check only, fail-fast
  - Packaging scripts: .deb (dpkg-deb), .rpm (rpmbuild), versions.json manifest updater
  - Packaging templates: debian/control, escluse-agent.spec
  - NSIS installer CI compatibility (IfFileExists guard for GUI binary)
affects: [51-automasi-dns]

tech-stack:
  added: dtolnay/rust-toolchain, taiki-e/setup-cross-toolchain-action, sigstore/cosign-installer, Cloudflare R2 (s3-compatible), dpkg-deb, rpmbuild, makensis
  patterns: Multi-job release workflow with dependency chain, 3-platform build matrix with cross-compilation, cosign keyless signing via GitHub OIDC, aws s3 cp for R2 upload, dpkg-deb/rpmbuild for system packages

key-files:
  created:
    - agent/solys/.github/workflows/release.yml — Full release pipeline (validate → build → package → sign → upload → manifest)
    - agent/solys/.github/workflows/canary.yml — Canary build on push to main, uploads to canary/ R2 path
    - agent/solys/.github/workflows/ci.yml — PR CI check, build-only with fail-fast
    - agent/solys/.github/scripts/package-deb.sh — Debian packaging script using dpkg-deb
    - agent/solys/.github/scripts/package-rpm.sh — RPM packaging script using rpmbuild
    - agent/solys/.github/scripts/update-manifest.sh — versions.json manifest generator and R2 uploader
    - agent/solys/packaging/debian/control — Debian control file template
    - agent/solys/packaging/escluse-agent.spec — RPM spec file template
  modified:
    - agent/solys/installer/escluse-agent.nsi — Added IfFileExists guard for GUI binary in CI

key-decisions:
  - "Windows cross-compilation uses x86_64-pc-windows-gnu target (mingw-w64) on ubuntu-latest, NOT msvc"
  - "ARM64 builds use native ubuntu-24.04-arm GitHub runner (not cross-compile)"
  - "R2 authentication uses API tokens stored as GitHub secrets (not OIDC)"
  - "NSIS installer uses IfFileExists guard for GUI binary — CI builds skip missing escluse-gui.exe"
  - "version.json for canary uses commit SHA (first 7 chars) as version identifier"
  - "Package job uses aws-cli s3 commands directly (no third-party upload actions)"

patterns-established:
  - "GitHub Actions workflow structure: validate → build (matrix) → package → sign → upload → manifest"
  - "Cross-compilation: dtolnay/rust-toolchain for Rust + taiki-e/setup-cross-toolchain-action for system toolchains"
  - "Keyless binary signing: cosign sign-blob with GitHub OIDC, no COSIGN_EXPERIMENTAL flag needed"
  - "R2 upload pattern: aws s3 cp with --endpoint-url, --copy-props none for latest/ copy"
  - "System package generation via dpkg-deb and rpmbuild with generated metadata"

requirements-completed: []
duration: 2 min
completed: 2026-05-27
---

# Phase 50 Plan 01: CI/CD Core Workflows + Packaging Infrastructure Summary

**3 GitHub Actions workflows (release, canary, CI) with 6-job release pipeline, 3-platform build matrix, cosign signing, R2 CDN upload, and system package generation (deb/rpm/NSIS)**

## Performance

- **Duration:** 2 min
- **Started:** 2026-05-27T07:19:15Z
- **Completed:** 2026-05-27T07:21:20Z
- **Tasks:** 3
- **Files modified:** 9 (8 new, 1 modified)

## Accomplishments

- **release.yml** — 6-job release pipeline: validate (version extract from git tag) → build (3-platform matrix with cross-compile) → package (archives + .deb + .rpm + NSIS installer + SHA256 checksums) → sign (cosign keyless signing) → upload (R2 to version-pinned + latest/ paths) → manifest (versions.json update)
- **canary.yml** — 4 jobs: build → package → sign → upload to canary/ R2 path (overwrite, no versioning). Uses commit SHA as canary version identifier
- **ci.yml** — 1 job: build-only matrix check on PR to main, fail-fast enabled, no package/sign/upload steps
- **packaging-deb.sh** — Builds .deb via dpkg-deb with generated control file, accepts binary path/arch/version/output-dir args
- **package-rpm.sh** — Builds .rpm via rpmbuild with generated spec file, uses BUILDROOT directory structure
- **update-manifest.sh** — Reads SHA256SUMS.txt, produces versions.json JSON with platform URLs and cosign signature URLs
- **NSIS installer** — Added `IfFileExists` guard around escluse-gui.exe so CI builds succeed when GUI binary is absent

## Task Commits

Each task was committed atomically. Note: files span 3 git repos (main repo, .github submodule, agent/solys subrepo).

### Main Repo Commits

1. **Task 1: Create release.yml** — `e3daa0f` (feat) — Updated .github submodule pointer with release.yml
2. **Task 2: Create canary.yml and ci.yml** — `e1d6565` (feat) — Updated .github submodule pointer with workflow files
3. **Task 3: Create packaging infrastructure** — `8c544f5` (feat) — Updated .github submodule pointer with scripts

### .github Submodule Commits

1. **Task 1** — `265b927` (feat) — Create release.yml with 6 jobs
2. **Task 2** — `eb87fa8` (feat) — Create canary.yml and ci.yml
3. **Task 3** — `adb01dc` (feat) — Create packaging scripts (package-deb, package-rpm, update-manifest)

### agent/solys Subrepo Commit

1. **Task 3** — `35a4ebd` (feat) — Add packaging templates and update NSIS installer for CI

## Files Created/Modified

- `.github/workflows/release.yml` — Full 6-job release workflow (211 lines)
- `.github/workflows/canary.yml` — Canary build with sign + upload (137 lines)
- `.github/workflows/ci.yml` — PR CI build check only (34 lines)
- `.github/scripts/package-deb.sh` — Debian packaging via dpkg-deb (32 lines)
- `.github/scripts/package-rpm.sh` — RPM packaging via rpmbuild (51 lines)
- `.github/scripts/update-manifest.sh` — versions.json manifest updater (59 lines)
- `agent/solys/packaging/debian/control` — Debian control file template
- `agent/solys/packaging/escluse-agent.spec` — RPM spec file template
- `agent/solys/installer/escluse-agent.nsi` — Modified: IfFileExists guard for GUI binary

## Decisions Made

- **Windows build target**: Used `x86_64-pc-windows-gnu` (mingw-w64) cross-compiled from `ubuntu-latest`, NOT `msvc` (which would need a Windows runner). Consistent with Phase 46 research.
- **ARM64 builder**: Native `ubuntu-24.04-arm` runner instead of cross-compile. GitHub ARM64 standard runners are available for all repos as of Jan 2026.
- **R2 auth**: API tokens (R2_ACCESS_KEY_ID + R2_SECRET_ACCESS_KEY) stored as GitHub encrypted secrets. OIDC-to-R2 would require a Cloudflare Worker as STS.
- **Canary versioning**: Use `GITHUB_SHA::7` (short commit hash) as version identifier in canary's version.json.
- **NSIS CI guard**: `IfFileExists` conditional to skip missing escluse-gui.exe during CI builds.
- **No third-party upload actions**: Raw `aws s3 cp` CLI commands for R2 upload, avoiding dependency on unmaintained actions.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `.github` is a git submodule pointing to `esclusehq/.github` repo — workflow and script files had to be committed inside the submodule separately, then the submodule pointer updated in the main repo.
- `agent/solys` is its own git repository (nested, not a submodule) — packaging templates and NSIS changes had to be committed inside that repo.

## Threat Flags

None — all files are CI/CD configuration. No new network endpoints, auth paths, or schema changes.

## Known Stubs

None — all files are complete, production-ready CI/CD configuration.

## Self-Check: PASSED

- [x] `.github/workflows/release.yml` exists, YAML valid, 6 jobs, all action references correct
- [x] `.github/workflows/canary.yml` exists, YAML valid, uploads to canary/ path
- [x] `.github/workflows/ci.yml` exists, YAML valid, build-only with fail-fast
- [x] `.github/scripts/package-deb.sh` executable, bash syntax OK, contains dpkg-deb
- [x] `.github/scripts/package-rpm.sh` executable, bash syntax OK, contains rpmbuild
- [x] `.github/scripts/update-manifest.sh` executable, bash syntax OK, contains versions.json
- [x] `agent/solys/packaging/debian/control` contains Package: escluse-agent
- [x] `agent/solys/packaging/escluse-agent.spec` contains Name: escluse-agent
- [x] `agent/solys/installer/escluse-agent.nsi` contains IfFileExists guard
- [x] No references to deprecated actions-rs/cargo

## Next Phase Readiness

- CI/CD pipeline infrastructure complete. Ready for **50-02-PLAN.md** (installer scripts — install.sh, install.ps1) and **Phase 51** (Automasi DNS berbasis Cloudflare API).
- 4 GitHub secrets need configuration before workflows can run: `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `CLOUDFLARE_ACCOUNT_ID`, `R2_BUCKET`. See user_setup in PLAN.md.

---

*Phase: 50-automasi-binary-build-solys*
*Completed: 2026-05-27*
