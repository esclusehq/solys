---
phase: 50-automasi-binary-build-solys
plan: 02
subsystem: ci-cd
tags: installer, bash, powershell, windows, linux, macos, sha256, cosign

requires:
  - phase: 50-automasi-binary-build-solys
    plan: 01
    provides: CI/CD workflow that builds and signs binaries
provides:
  - install.sh — Linux/macOS one-command installer with platform detection, SHA256 verification, cosign verification
  - install.ps1 — Windows PowerShell installer with platform detection, SHA256 verification, PATH update
affects: [51-automasi-dns]

tech-stack:
  added: bash (install.sh), powershell (install.ps1)
  patterns: One-command installer pattern with curl|bash, SHA256 checksum verification before install, optional cosign signature verification

key-files:
  created:
    - install.sh — Linux/macOS installer script (156 lines, bash syntax verified)
    - install.ps1 — Windows PowerShell installer script (222 lines)
  modified: []

key-decisions:
  - "install.sh uses `uname -s` with darwin→linux mapping (macOS uses same linux binaries)"
  - "install.sh exits with clear error on Windows (redirects users to install.ps1)"
  - "install.ps1 installs to ProgramFiles\Escluse with LOCALAPPDATA fallback if not writable"
  - "Cosign verification is best-effort in both scripts — non-fatal if cosign not available or verification fails"

requirements-completed: []
duration: 1 min
completed: 2026-05-27
---

# Phase 50 Plan 02: Installer Scripts Summary

**Production-grade one-command installer scripts (install.sh for Linux/macOS, install.ps1 for Windows) with platform detection, SHA256 checksum verification, optional cosign signature verification, and system PATH installation**

## Performance

- **Duration:** 1 min
- **Started:** 2026-05-27T07:24:16Z
- **Completed:** 2026-05-27T07:25:42Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- **install.sh** (156 lines) — Linux/macOS standalone installer with platform detection (`uname -s`/`uname -m`), optional version pinning, HTTPS download from `get.esluce.com`, SHA256 checksum verification, optional cosign signature verification, `sudo install -m 755` to `/usr/local/bin`, `ESCLUSE_BIN_DIR` env var override. Passes `bash -n` syntax check.
- **install.ps1** (222 lines) — Windows PowerShell installer with `-Version` parameter, architecture detection via `PROCESSOR_ARCHITECTURE`, HTTPS download via `Invoke-WebRequest`, SHA256 verification via `Get-FileHash`, optional cosign verification, installation to `ProgramFiles\Escluse` with `LOCALAPPDATA` fallback, User PATH update via `SetEnvironmentVariable`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create install.sh** — `8500685` (feat) — Linux/macOS installer with platform detection, SHA256 + cosign verification
2. **Task 2: Create install.ps1** — `1ec9e31` (feat) — Windows PowerShell installer with SHA256, PATH update

## Files Created/Modified

- `install.sh` — Linux/macOS installer script (156 lines)
- `install.ps1` — Windows PowerShell installer script (222 lines)

## Decisions Made

- **macOS binary mapping**: `darwin` OS maps to `linux` in install.sh (macOS uses same linux binaries for now; exits with clear message if unsupported in the future)
- **Windows detection in install.sh**: install.sh detects `mingw*|cygwin*` and exits with a message directing users to install.ps1 instead of attempting Windows installation via bash
- **Cosign best-effort**: Both scripts attempt cosign verification but treat failures as non-fatal (warn, not fail) — cosign may not be installed on user machines
- **Install location priority**: install.ps1 tries `ProgramFiles\Escluse` first, falls back to `LOCALAPPDATA\Escluse` if ProgramFiles is not writable (common on locked-down enterprise machines)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## Threat Flags

None — installer scripts are client-side utility scripts. All downloads use HTTPS exclusively. SHA256 verification protects against binary tampering during transit.

## Known Stubs

None

## Self-Check: PASSED

- [x] `install.sh` exists at repository root, executable, passes `bash -n`
- [x] `install.ps1` exists at repository root
- [x] Both scripts use HTTPS URLs only (0 HTTP URLs found)
- [x] Both scripts verify SHA256 checksums before installation
- [x] install.sh supports `ESCLUSE_BIN_DIR` override
- [x] install.ps1 updates User PATH environment variable
- [x] Both scripts accept optional version parameter for pinned-version installs
- [x] install.sh has cosign verify-blob for optional signature verification
- [x] install.ps1 has Get-FileHash for SHA256 verification
- [x] install.sh has platform detection with `uname -s` and `uname -m`
- [x] install.ps1 has platform detection with `PROCESSOR_ARCHITECTURE`

## Next Phase Readiness

- Phase 50 (automasi-binary-build-solys) complete — both plans executed:
  - Plan 01: CI/CD workflows (release, canary, CI) with build matrix, packaging, cosign signing, R2 upload
  - Plan 02: Installer scripts for Linux/macOS (install.sh) and Windows (install.ps1)
- Ready for **Phase 51** (Automasi DNS berbasis Cloudflare API)
- 4 GitHub secrets still need configuration before CI/CD can run: `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `CLOUDFLARE_ACCOUNT_ID`, `R2_BUCKET`

---

*Phase: 50-automasi-binary-build-solys*
*Completed: 2026-05-27*
