# Phase 62: Create CONTRIBUTING.md - Cara kontribusi - Pattern Map

**Mapped:** 2026-05-31
**Files analyzed:** 6 (1 primary + 5 optional)
**Analogs found:** 4 / 6 (2 optional items have no direct analog — use GitHub defaults)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `CONTRIBUTING.md` | documentation (guide) | static | `DEVELOPMENT.md` | exact — same role, same location pattern (repo root entry point) |
| `CODE_OF_CONDUCT.md` | documentation (policy) | static | `DEVELOPMENT.md` | partial — both are repo-root policy-style docs, but CoC uses standard template |
| `.github/ISSUE_TEMPLATE/bug_report.md` | config (template) | static | `PUSH_COMMIT.md` (issue label schema) | partial — same project conventions reference |
| `.github/ISSUE_TEMPLATE/feature_request.md` | config (template) | static | `PUSH_COMMIT.md` (issue label schema) | partial — same project conventions reference |
| `.github/ISSUE_TEMPLATE/config.yml` | config (YAML) | static | *(no analog — GitHub-native YAML config)* | none |
| `.github/PULL_REQUEST_TEMPLATE.md` | config (template) | static | `PUSH_COMMIT.md` (PR checklist section) | partial — checklist format reference |

## Pattern Assignments

### `CONTRIBUTING.md` (documentation — guide, static)

**Analog:** `DEVELOPMENT.md` (repo root, 103 lines)

**Key constraint learned from Phase 61:** `docs/` is a sub-repo gitlink — parent-repo files CANNOT live under `docs/`. CONTRIBUTING.md at repo root is the correct location (no conflict since it's at root, not under `docs/`).

**Imports/Header pattern** (DEVELOPMENT.md lines 1-9):
```markdown
# Escluse — Local Development

Esluce is a game server management platform. This guide covers setting up the full development environment.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![React](https://img.shields.io/badge/react-19-blue)
![Node](https://img.shields.io/badge/node-20%2B-green)
![PostgreSQL](https://img.shields.io/badge/postgresql-16-336791)
![Redis](https://img.shields.io/badge/redis-7-red)
```

**Pattern:** ATX heading `# Title — Subtitle`, followed by summary paragraph, then Shields.io badge row. For CONTRIBUTING.md, use badges relevant to contribution (PRs, issues, contributors).

**Quick Start / Section pattern** (DEVELOPMENT.md lines 26-70):
```markdown
## Quick Start

1. **Step title:**
   ```bash
   command
   ```

2. **Step title:**
   See the [link text](path) for details.
```

**Pattern:** Numbered steps with bold titles, optional code blocks per step, cross-references to sub-documents. CONTRIBUTING.md uses the same pattern for PR workflow steps.

**Repository Structure ASCII tree** (DEVELOPMENT.md lines 74-88):
```markdown
## Repository Structure

```
escluse/
├── api/              # Backend API (Rust/Axum, port 3000) — separate repo
├── app/              # Frontend (React 19 + Vite, port 5173) — separate repo
├── worker/           # Background job processor (Rust) — separate repo
├── agent/            # Agent crates
│   ├── solys/        #   Web Agent (Rust + Bollard) — separate repo
│   └── agent-core/   #   Shared Rust workspace (12 crates) — separate repo
├── docs/             # Documentation site (VitePress) — separate repo
├── dev/              # Local development setup guides — part of parent repo
├── landing-page-escluse/  # Marketing site — separate repo
├── packages/         # SDK packages — separate repo
├── gateway/          # Caddy reverse proxy config — part of parent repo
├── docker-compose.yml
└── DEVELOPMENT.md    # You are here
```
```

**Pattern:** `───` lines with `├──` and `└──` for branches. 2-space indent for sub-directories. `#` comments inline. CONTRIBUTING.md should use same tree format for the meta-repo structure.

**Warning callout pattern** (DEVELOPMENT.md lines 91):
```markdown
> **Warning:** `api/`, `app/`, `docs/`, ... are **independent git repositories** cloned into these directories — they are NOT git submodules. You must clone them separately (see [Setup Guide](dev/02-setup.md#cloning-repositories)).
```

**Pattern:** `> **Warning:**` for critical callouts, bold emphasis on key terms with `**`. CONTRIBUTING.md uses same pattern for critical warnings (e.g., "contribute to the correct repo").

**Documentation Index pattern** (DEVELOPMENT.md lines 93-99):
```markdown
## Documentation Index

- **[dev/01-prerequisites.md](dev/01-prerequisites.md)** — OS-specific tool install commands for Linux, macOS, and Windows
- **[dev/02-setup.md](dev/02-setup.md)** — Full setup: clone repos, Docker infra, Supabase, .env configuration
```

**Pattern:** Bullet list with `- **[link](path)** — description`. CONTRIBUTING.md uses the same for referencing sub-documents.

**Note callout pattern** (DEVELOPMENT.md line 24):
```markdown
> **Note:** See [01-prerequisites.md](dev/01-prerequisites.md) for OS-specific install commands.
```

**Pattern:** `> **Note:**` for less-critical information.

---

### `CODE_OF_CONDUCT.md` (documentation — policy, static)

**Analog:** `DEVELOPMENT.md` (partial match — same repo-root doc style)

**Pattern to use:** Standard Contributor Covenant template (industry standard, GitHub-native). No custom writing needed. Link from CONTRIBUTING.md.

**Source for template:** GitHub's built-in CODE_OF_CONDUCT.md generator (Settings → Community Standards or use `gh api`).

**Key pattern:** Single markdown file at repo root with standard sections: Pledge, Standards, Enforcement, Attribution. No custom formatting needed.

---

### `.github/ISSUE_TEMPLATE/bug_report.md` (config — template, static)

**Analog:** `PUSH_COMMIT.md` lines 218-261 (issue label schema)

**Label schema excerpt** (PUSH_COMMIT.md lines 218-234):
```markdown
**Labels yang WAJIB digunakan:**

| Label | Penggunaan |
|-------|-------------|
| `bug` | Bug fix dan perbaikan error |
| `feature` | Feature baru |
| `enhancement` | Perbaikan/optimasi yang bukan bug |
| `documentation` | Perubahan docs (README, CHANGELOG, dll) |
| `chore` | Maintenance, dependency update |
| `refactor` | Code refactoring |

**Priority Labels (opsional):**
| Label | Penggunaan |
|-------|-------------|
| `priority:high` | Urgent, harus selesai secepatnya |
| `priority:medium` | Normal priority |
| `priority:low` | Tidak urgent |
```

**Pattern:** Issue templates must reference these exact labels (`bug`, `feature`, `enhancement`, `documentation`, `chore`, `refactor`, `priority:high/medium/low`). Template frontmatter uses `labels: [bug]` or `labels: [feature]`.

**Template format:** GitHub issue template format with YAML frontmatter:
```yaml
---
name: Bug Report
about: Create a report to help us improve
title: 'fix: '
labels: ['bug']
assignees: ''
---
```

**Body format:** Use GFM headings (`## `) for sections (Describe the bug, To Reproduce, Expected behavior, Environment).

---

### `.github/ISSUE_TEMPLATE/feature_request.md` (config — template, static)

**Analog:** `PUSH_COMMIT.md` lines 218-261 (same label schema)

**Pattern:** Same template format as bug_report.md, with `labels: ['feature']` in frontmatter. Sections: Problem, Solution, Alternatives, Additional context.

---

### `.github/ISSUE_TEMPLATE/config.yml` (config — YAML, static)

**No analog found.** This is a GitHub-native YAML config file for issue template routing. Use standard GitHub format:

```yaml
blank_issues_enabled: false
contact_links:
  - name: Escluse Community
    url: https://github.com/esclusehq/escluse/discussions
    about: Please ask and answer questions here.
```

---

### `.github/PULL_REQUEST_TEMPLATE.md` (config — template, static)

**Analog:** `PUSH_COMMIT.md` lines 108-181 (PR/commit checklist)

**Checklist section excerpt** (PUSH_COMMIT.md lines 126-134):
```markdown
### 2. Checklist Commit

- [ ] Ubah kode/features?
- [ ] CHANGELOG.md diupdate? (WAJIB jika ada perubahan - lihat [SEMVER.md](./SEMVER.md))
- [ ] README.md diupdate? (jika ada perubahan install/setup)
- [ ] Semua file `.md` sudah konsisten dengan perubahan?
- [ ] Tidak ada secrets atau credentials di commit
- [ ] Branch sudah benar (`master` atau feature branch)
```

**Pattern:** GitHub checklist using `- [ ]` / `- [x]` syntax. For PR template, use similar checklist but adapted for PR review (not commit). Include changelog update checklist item referencing SEMVER.md.

**Commit message format** (PUSH_COMMIT.md lines 136-151):
```markdown
### 3. Format Commit Message

```bash
# Format: type(scope): description
#
# Types:
#   feat    - Fitur baru
#   fix     - Bug fix
#   docs    - Dokumentasi (WAJIB setelah setiap fitur/bug fix)
#   refactor - Refactoring kode
#   chore   - Maintenance, dependency update
#
# Contoh:
git commit -m "feat(sdk): add WebSocket support for real-time events"
git commit -m "fix(changelog): update v0.2.0 release notes"
git commit -m "docs: update installation steps in README.md"
```
```

**Pattern:** PR template should include commit message format reference linking back to PUSH_COMMIT.md.

---

## Shared Patterns

### Markdown Formatting Conventions
**Source:** `DEVELOPMENT.md`, `PUSH_COMMIT.md`, `SEMVER.md`, `DEPLOY.md`
**Apply to:** All new documentation files

| Pattern | Standard | Source Example |
|---------|----------|----------------|
| Heading style | ATX (`##`, `###`) | DEVELOPMENT.md line 1 |
| Code blocks | Fenced with language tags (`bash`, `text`, `typescript`) | DEVELOPMENT.md line 29 |
| Tables | GFM pipe tables with alignment dashes | DEVELOPMENT.md line 13-22 |
| Admonitions | `> **Warning:**` and `> **Note:**` | DEVELOPMENT.md line 24, 91 |
| Badges | Shields.io (`![Label](https://img.shields.io/badge/...)`) | DEVELOPMENT.md lines 5-9 |
| Emphasis | Bold `**` for key terms, inline code `` for commands | DEVELOPMENT.md line 91 |
| Internal links | Relative paths (`dev/01-prerequisites.md`) | DEVELOPMENT.md line 24 |
| Line spacing | 1 blank line between sections, 0 blank lines after `---` | All docs |

### Language Conventions
**Source:** Project root docs
**Apply to:** All new documentation files

| Document | Language | Audience |
|----------|----------|----------|
| `DEVELOPMENT.md` | English | External developers |
| `PUSH_COMMIT.md` | Indonesian | Internal maintainer |
| `SEMVER.md` | Indonesian | Internal maintainer |
| `DEPLOY.md` | Indonesian | Internal maintainer |
| `STRATEGI.md` | Indonesian | Internal |
| `CONTRIBUTING.md` | **English recommended** | External contributors (open-source standard) |
| `CODE_OF_CONDUCT.md` | English | Standard template (English by default) |

**Key insight:** The project has a split-language pattern:
- **English** for public-facing developer docs (DEVELOPMENT.md, CONTRIBUTING.md)
- **Indonesian** for internal workflow docs (PUSH_COMMIT.md, SEMVER.md, DEPLOY.md)

CONTRIBUTING.md should follow the English pattern since external contributors are the primary audience. The phase title "Cara kontribusi" is Indonesian, but the file itself should be English with a possible note welcoming Indonesian contributors.

### Meta-Repo Contribution Table Pattern
**Source:** `PUSH_COMMIT.md` lines 14-27, `DEPLOY.md` lines 491-501
**Apply to:** `CONTRIBUTING.md`

**Repo mapping table** (PUSH_COMMIT.md lines 15-27):
```markdown
| Folder Lokal | GitHub Repo | Description |
|--------------|-------------|-------------|
| `agent/solys/` | `esclusehq/solys` | Solys Agent - Rust game server agent |
| `agent/agent-core/` | `esclusehq/agent-core` | Agent Core Framework - Rust crates |
| `app/` | `esclusehq/escluse-dashboard` | React Dashboard - Frontend |
| `api/` | `esclusehq/escluse-cloud` | Rust Backend - API + Migrations |
| ...
```

**Pattern:** 3-column GFM table with `Directory`, `GitHub URL`, `Description`. For CONTRIBUTING.md, adapt columns to: "Area / Feature", "GitHub Repo", "Directory in Parent", "Language". This is the critical pattern for explaining the meta-repo contribution model.

### Conventional Commits Pattern
**Source:** `PUSH_COMMIT.md` lines 136-151, `SEMVER.md` lines 104-121
**Apply to:** `CONTRIBUTING.md` (commit conventions section)

**Commit type table** (SEMVER.md lines 108-115):
```markdown
| Prefix | Arti |
|--------|------|
| `feat:` | Fitur baru |
| `fix:` | Bug fix |
| `refactor:` | Refactoring |
| `docs:` | Dokumentasi |
| `chore:` | Maintenance |
```

**Pattern:** 2-column GFM table for commit types. CONTRIBUTING.md should use English descriptions since it's aimed at external contributors. Reference PUSH_COMMIT.md for full details.

### Changelog Entry Format Pattern
**Source:** `SEMVER.md` lines 60-73
**Apply to:** `CONTRIBUTING.md` (if changelog section included)

```typescript
{
  version: '0.1.0',
  date: '2026-05-17',
  type: 'minor',  // 'major' | 'minor' | 'patch' | 'initial'
  changes: {
    added: ['Fitur baru 1', 'Fitur baru 2'],
    improved: ['Perbaikan 1'],
    fixed: ['Bug fix 1'],
    removed: ['Fitur yang dihapus'],
    security: ['Security fix']
  }
}
```

**Pattern:** Changelog entries go in `landing-page-escluse/src/pages/Changelog.tsx`, not in individual repos. CONTRIBUTING.md must explain this is a separate step.

### GitHub Issue Labels Pattern
**Source:** `PUSH_COMMIT.md` lines 218-234
**Apply to:** `CONTRIBUTING.md` (if issue section included)

```markdown
| Label | Usage |
|-------|-------|
| `bug` | Bug fixes and error corrections |
| `feature` | New feature |
| `enhancement` | Improvements/optimizations (not bugs) |
| `documentation` | Docs changes |
| `chore` | Maintenance, dependency updates |
| `refactor` | Code refactoring |
| `priority:high` | Urgent |
| `priority:medium` | Normal priority |
| `priority:low` | Not urgent |
```

**Pattern:** 2-column GFM table. CONTRIBUTING.md should use English translations (the original is in Indonesian). Reference PUSH_COMMIT.md for the canonical source.

### Verification Command Pattern (Testing)
**Source:** `dev/04-commands.md` lines 9-30, 33-47, 49-61, 82-98
**Apply to:** `CONTRIBUTING.md` (testing expectations section)

```bash
# API Backend
cd api
cargo test                    # Run all tests
cargo clippy                  # Lint

# Worker
cd worker
cargo test
cargo clippy

# Web Agent
cd agent/solys
cargo test
cargo clippy

# Agent Core (shared workspace, 12 crates)
cd agent/agent-core
cargo test --workspace
cargo clippy --workspace

# Frontend
cd app
npm run lint                  # Lint only (no test framework configured)
```

**Pattern:** Group commands by service with `cd <dir>` prefix. Note where tests are not yet configured (frontend). Reference 04-commands.md for the full command reference.

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns or GitHub defaults instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `.github/ISSUE_TEMPLATE/config.yml` | config (YAML) | static | No existing YAML config files in parent repo — use GitHub standard format |
| `CODE_OF_CONDUCT.md` | documentation (policy) | static | No existing CoC — use GitHub's built-in Contributor Covenant template generator |
| `.github/ISSUE_TEMPLATE/bug_report.md` | config (template) | static | No existing issue templates — use GitHub standard markdown template format |
| `.github/ISSUE_TEMPLATE/feature_request.md` | config (template) | static | No existing issue templates — use GitHub standard markdown template format |
| `.github/PULL_REQUEST_TEMPLATE.md` | config (template) | static | No existing PR template — use GitHub standard markdown template format |

For all template files, the planner should use GitHub's built-in template system (`.github/ISSUE_TEMPLATE/` with YAML frontmatter + markdown body). These are GitHub-native features, not custom file formats.

---

## Key Structural Pattern: What CONTRIBUTING.md MUST Reference vs. MUST NOT Duplicate

**Critical architectural constraint from Phase 61:**
- `DEVELOPMENT.md` covers **local setup** (prerequisites, clone, Docker, .env, run commands)
- `CONTRIBUTING.md` covers **contribution workflow** (issues, PRs, commits, review, meta-repo model)
- CONTRIBUTING.md MUST link to DEVELOPMENT.md for setup, NOT repeat it
- CONTRIBUTING.md MUST link to PUSH_COMMIT.md for commit conventions, NOT repeat them in full

**Pattern for cross-referencing** (DEVELOPMENT.md line 103):
```markdown
Once your environment is running, check out the [Architecture Overview](dev/02-setup.md#quick-architecture-overview) and [Contributing Guide](../CONTRIBUTING.md) (when available).
```

CONTRIBUTING.md should have a companion forward-reference:
```markdown
For local development setup, see [DEVELOPMENT.md](DEVELOPMENT.md).
For commit conventions, see [PUSH_COMMIT.md](PUSH_COMMIT.md).
For versioning details, see [SEMVER.md](SEMVER.md).
For deployment, see [DEPLOY.md](DEPLOY.md).
```

---

## Metadata

**Analog search scope:** Repo root, `dev/`, `.github/`, `docs/`
**Files scanned:** `DEVELOPMENT.md`, `dev/01-prerequisites.md`, `dev/02-setup.md`, `dev/03-configuration.md`, `dev/04-commands.md`, `dev/05-troubleshooting.md`, `PUSH_COMMIT.md`, `SEMVER.md`, `DEPLOY.md`, `.github/profile/README.md`, `STRATEGI.md`
**Pattern extraction date:** 2026-05-31
