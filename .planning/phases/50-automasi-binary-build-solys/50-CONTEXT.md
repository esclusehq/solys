# Phase 50: Automasi build binary untuk agent/solys - Context

**Gathered:** 2026-05-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Automated CI/CD pipeline that builds the Solys agent binaries across platforms on GitHub push, uploads artifacts to R2, and serves them via Cloudflare CDN at get.esluce.com. Includes installer generation and version management.

</domain>

<decisions>
## Implementation Decisions

### Build Target Matrix (D-01 to D-04)

- **D-01:** Platforms — Linux x86_64, Linux aarch64 (ARM64), Windows x86_64
- **D-02:** Windows build — cross-compile from Linux runner using mingw-w64 (x86_64-pc-windows-msvc target, as done in Phase 46)
- **D-03:** ARM build — use native GitHub ARM64 runner for Linux aarch64 (not cross-compile from x86)
- **D-04:** Packaging — raw binary tarballs + system packages (.deb, .rpm for Linux; .msi for Windows)

### Release Versioning & CDN (D-05 to D-07)

- **D-05:** Version scheme — semver via git tags (push `v1.2.3` → triggers release build)
- **D-06:** CDN layout — hybrid approach:
  - Per-version directories: `get.esluce.com/v1.2.3/solys-linux-x86_64.tar.gz`
  - Latest redirect: `get.esluce.com/latest/` points to most recent version
  - Manifest: `get.esluce.com/versions.json` lists all available versions
- **D-07:** Upload schedule — two channels:
  - Push to main → build + upload to `canary` path (bleeding edge)
  - Tagged semver release → build + upload to `latest` + version-pinned path

### Pipeline Triggers & Security (D-08 to D-10)

- **D-08:** Triggers — push to main (canary build), pull requests to main (CI check only), semver tags (release build). Also support manual workflow dispatch
- **D-09:** Binary integrity — SHA256 checksums file per release + signing with Sigstore/cosign (keyless signing via GitHub OIDC)
- **D-10:** Credentials — GitHub OIDC for authenticating to Cloudflare R2 (no long-lived secrets stored in CI)

### the agent's Discretion
- Installer update mechanism (install.sh / PowerShell script) — user did not discuss this area; agent may choose standard approach (fetch latest binary + verify checksum + cosign signature)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Agent Binary
- `agent/solys/Cargo.toml` — the Solys agent project and its dependencies
- `agent/solys/README.md` — existing agent documentation
- `agent/solys/CHANGELOG.md` — version history

### Existing Infrastructure
- `docker-compose.yml` — current deployment setup (reference for how agent runs)
- `gateway/Caddyfile.prod` — existing Caddy reverse proxy config for domains

### CI/CD
- `.github/` — directory where GitHub Actions workflows will be created (currently empty)

### Prior Phase Context
- `.planning/phases/46-multi-platform/46-CONTEXT.md` — Windows build target decisions (Phase 46 locked x86_64-pc-windows-msvc target)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `agent/solys/` — the complete Solys agent binary project (what gets built)
- `agent-core/` — workspace of shared Rust crates (dependencies of the binary)
- `agent/solys/docker/` — existing Docker-related files (may inform container builds)

### Established Patterns
- Rust cross-compilation via Cargo with target triples
- Phase 46 established mingw-w64 cross-compiler for Windows builds
- Existing .gitignore suggests Rust build output patterns

### Integration Points
- GitHub Actions will integrate with Cloudflare R2 (S3-compatible API)
- Cloudflare CDN will serve binaries at get.esluce.com
- Installer scripts (install.sh) will reference CDN URLs for downloads

</code_context>

<specifics>
## Specific Ideas

Pipeline flow: Push to GitHub → GitHub Actions → Build binaries (Linux x86_64, Linux aarch64, Windows x86_64) + system packages → Upload to R2 → Cloudflare CDN → get.esluce.com → Users install/update

Two channels: canary (main branch, bleeding edge) and stable (tagged releases)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 50-Automasi build binary untuk agent/solys*
*Context gathered: 2026-05-27*
