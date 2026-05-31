# Phase 62: Create CONTRIBUTING.md - Cara kontribusi - Research

**Researched:** 2026-05-31
**Domain:** Contributor documentation, open-source contribution workflow
**Confidence:** HIGH

## Summary

This phase delivers a `CONTRIBUTING.md` file at the repo root that tells contributors how to submit code, report bugs, request features, and follow project conventions. The project already has `DEVELOPMENT.md` (Phase 61) covering local setup — this phase must **complement** it, not duplicate it.

**Critical architectural discovery:** Esluce is a **meta-repo** — `api/`, `app/`, `docs/`, `agent/solys/`, `agent/agent-core/`, `landing-page-escluse/`, `packages/`, `migration/`, `gateway/` are all **independent git repositories** under `github.com/esclusehq`. CONTRIBUTING.md must explain that contributions happen across multiple repos, each with its own issue tracker, branches, and PR process. This is the single most important conceptual hurdle for new contributors.

**No existing CONTRIBUTING.md, no CODE_OF_CONDUCT.md, no issue/PR templates exist** in the parent repo. The phase has an opportunity (in the agent's discretion) to create these companion files alongside CONTRIBUTING.md.

**Primary recommendation:** Create `CONTRIBUTING.md` at the repo root as the canonical contributor guide. Structure it with sections covering: Code of Conduct, How to Report Bugs / Request Features, Submitting Changes (PR workflow), Commit Message Conventions, Coding Standards, Testing Expectations, and the Meta-Repo Contribution Model. Reference `DEVELOPMENT.md` for local setup — do not repeat it.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

No CONTEXT.md exists for Phase 62 yet — this is a unfinalized phase. The research must identify what decisions need to be made.

### The Agent's Discretion (recommended for discussion)
- Whether to create CODE_OF_CONDUCT.md alongside CONTRIBUTING.md
- Whether to create GitHub issue/PR templates alongside CONTRIBUTING.md
- Whether to include Indonesian-language sections (project has Indonesian contributions like PUSH_COMMIT.md)
- Whether to list a contact email / Discord server for contributors
- Exact formatting and section ordering of CONTRIBUTING.md
- Which specific coding conventions to inline vs. reference from .planning/codebase/

### Deferred Ideas (OUT OF SCOPE)
- None currently identified.
</user_constraints>

---

## Phase Requirements

No requirement IDs provided for this phase — it is a developer experience / documentation phase with no functional requirements to map.

| ID | Description | Research Support |
|----|-------------|------------------|
| — | — | — |

---

## Architectural Responsibility Map

This phase is purely documentation — it does not add runtime code. The "capabilities" are documentation sections:

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Contribution workflow | `CONTRIBUTING.md` (root) | — | Canonical location for project-level contribution guide |
| Code of Conduct | `CODE_OF_CONDUCT.md` (root, optional) | — | Companion file, linked from CONTRIBUTING.md |
| Issue/PR templates | `.github/ISSUE_TEMPLATE/` (optional) | — | GitHub-native issue/PR template system |

---

## Standard Stack

### Documentation Format
| Aspect | Standard | Why |
|--------|----------|-----|
| Format | Markdown (GFM) | Universal, renders on GitHub |
| Code blocks | Fenced with language tags | Syntax highlighting for commands |
| Tables | GFM pipe tables | Label/priority tables, repo mapping |
| Admonitions | `> **Note:**` / `> **Warning:**` | Standard GitHub-flavored blockquotes |
| Badges | Shields.io | Optional, status indicators |

### Existing Documentation Infrastructure
| Tool | Version | Purpose |
|------|---------|---------|
| DEVELOPMENT.md | — | Local dev setup (Phase 61) — CONTRIBUTING.md MUST reference this |
| dev/* | — | 5 sub-files: prerequisites, setup, config, commands, troubleshooting |
| PUSH_COMMIT.md | — | Internal commit/push workflow for maintainers |
| SEMVER.md | — | Versioning rules |
| DEPLOY.md | — | Deployment guide |
| .planning/codebase/* | — | Technical documentation (ARCHITECTURE.md, CONVENTIONS.md, TESTING.md, etc.) |

### Key Decision: Where CONTRIBUTING.md Lives
- **Location:** `CONTRIBUTING.md` at repo root (`/home/rhnbztnl/Downloads/Berguna/Projects/escluse/CONTRIBUTING.md`)
- **NOT** in `docs/` (that's a separate git sub-repo: `esclusehq/escluse-docs`)
- **NOT** in `docs/dev/` or `dev/` (those are for local setup, Phase 61)
- GitHub automatically links to `CONTRIBUTING.md` when users open issues/PRs (repo-level config)

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Root CONTRIBUTING.md | docs/contributing.md | GitHub doesn't auto-link to docs/ subdirectory; docs/ is separate repo |
| Single file | CONTRIBUTING.md + CODE_OF_CONDUCT.md + issue templates | Single file is simpler, but CoC and templates are standard practice |

---

## Architecture Patterns

### System Architecture Diagram

```
                     ┌─────────────────────────────────────┐
                     │         GitHub Organization          │
                     │         github.com/esclusehq         │
                     └─────────────────────────────────────┘
                                      │
         ┌────────────┬───────────────┼───────────────┬────────────────────┐
         ▼            ▼               ▼               ▼                    ▼
  ┌────────────┐ ┌────────┐ ┌───────────────┐ ┌───────────────┐  ┌────────────────┐
  │escluse/    │ │ solys/ │ │ agent-core/   │ │ escluse-cloud │  │ escluse-dashb. │
  │(parent)   │ │(agent) │ │(shared crates) │ │ (api+worker)  │  │ (frontend app) │
  │ infra +   │ │ Rust   │ │ 12-crate ws    │ │ Rust/Axum     │  │ React + Vite   │
  │ CONTRIBUT │ │ agent  │ │                │ │ backend       │  │                │
  └────────────┘ └────────┘ └───────────────┘ └───────────────┘  └────────────────┘

  ┌───────────────┐ ┌───────────────────┐ ┌─────────────────┐
  │ escluse-docs  │ │ escluse-landing-pg│ │ escluse-sdk     │
  │ VitePress     │ │ Marketing site    │ │ Node.js + Python│
  └───────────────┘ └───────────────────┘ └─────────────────┘
```

**Contribution flow:**
1. Contributor finds a bug/feature in one of the repos
2. Opens issue in the **specific repo** (e.g., `esclusehq/escluse-cloud` for backend bugs)
3. Forks the repo or branches within it
4. Makes changes following project conventions
5. Opens PR against the repo's `master` branch
6. Maintainer reviews, CI checks pass (if configured)
7. Changes merged + changelog updated + deployed

### Recommended File Structure

```
escluse/                              # Parent repo root
├── CONTRIBUTING.md                   # THIS PHASE — contributor guide
├── CODE_OF_CONDUCT.md                # OPTIONAL — if created alongside
├── .github/
│   ├── ISSUE_TEMPLATE/               # OPTIONAL — issue templates
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── config.yml
│   └── PULL_REQUEST_TEMPLATE.md      # OPTIONAL — PR template
├── DEVELOPMENT.md                    # Existing — local dev setup (Phase 61)
├── dev/                              # Existing — setup sub-files (Phase 61)
└── PUSH_COMMIT.md                    # Existing — internal maintainer workflow
```

### Pattern 1: Conventional Commits (already enforced by project)
**What:** Commit messages use `type(scope): description` format
**When to use:** Every commit across all repos
**Scopes from codebase:**
- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation
- `refactor:` — Code restructuring
- `chore:` — Maintenance, dependency updates
**Example:** `feat(sdk): add WebSocket support for real-time events` [VERIFIED: PUSH_COMMIT.md]

### Pattern 2: Meta-Repo Contribution Model
**What:** Contributors must understand they're contributing to one of 9+ independent repos, not a monorepo
**When to use:** Any new contribution
**The repos:**
| Repo | Directory | What It Does | Language |
|------|-----------|-------------|----------|
| `esclusehq/escluse` | `./` | Parent, infra, gateway config | Docker, Caddy |
| `esclusehq/escluse-cloud` | `api/` + `worker/` | Backend API + worker | Rust/Axum |
| `esclusehq/escluse-dashboard` | `app/` | Frontend | React 19 |
| `esclusehq/solys` | `agent/solys/` | Web Agent | Rust |
| `esclusehq/agent-core` | `agent/agent-core/` | Shared crates | Rust |
| `esclusehq/escluse-docs` | `docs/` | Documentation site | VitePress |
| `esclusehq/escluse-landing-page` | `landing-page-escluse/` | Marketing site | Vite |
| `esclusehq/escluse-sdk` | `packages/` | SDK | Node.js, Python |
| `esclusehq/escluse-infra` | `gateway/` + `docker-compose.yml` | Infra config | Docker |

[VERIFIED: PUSH_COMMIT.md, DEPLOY.md]

### Anti-Patterns to Avoid
- **CONTRIBUTING.md duplicating DEVELOPMENT.md:** Local setup is already documented. CONTRIBUTING.md should link to DEVELOPMENT.md and focus on the contribution lifecycle.
- **Treating this as a monorepo:** New contributors will naturally assume `api/` and `app/` are part of one repo. CONTRIBUTING.md must explicitly explain the meta-repo structure.
- **Missing Code of Conduct:** Open-source projects without a CoC discourage new contributors. Strongly recommend adding one (standard `CONTRIBUTOR_COVENANT` via GitHub template).
- **Overspecifying review process:** The project uses GSD workflow with maintainers doing atomic commits. Describe a realistic PR review process, not an aspirational one.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Code of Conduct | Custom CoC | GitHub's standard `CONTRIBUTOR_COVENANT` template | Industry standard, legally vetted |
| Issue templates | Custom template system | GitHub issue templates (`.github/ISSUE_TEMPLATE/`) | Native GH integration, no code needed |
| PR template | Custom PR format | GitHub PR template (`.github/PULL_REQUEST_TEMPLATE.md`) | Shows automatically on PR creation |

---

## Runtime State Inventory

> **SKIPPED** — This is a greenfield documentation phase. No rename, refactor, or migration is involved. No existing CONTRIBUTING.md exists.

---

## Common Pitfalls

### Pitfall 1: CONTRIBUTING.md Duplicates DEVELOPMENT.md
**What goes wrong:** The contributing guide repeats local setup steps already in DEVELOPMENT.md, leading to two sources of truth that drift apart.
**Why it happens:** Natural impulse to include everything a contributor needs.
**How to avoid:** CONTRIBUTING.md should have one line: "See [DEVELOPMENT.md](DEVELOPMENT.md) for local environment setup." Then focus entirely on the contribution workflow (issues, PRs, commits, review).
**Warning signs:** CONTRIBUTING.md sections like "Prerequisites" or "Clone the repo" that mirror DEVELOPMENT.md.

### Pitfall 2: Confusing Single-Repo vs. Meta-Repo Contribution
**What goes wrong:** A contributor opens a PR on `esclusehq/escluse` (the parent repo) expecting it to be the backend or frontend repo. OR they contribute to the wrong repo entirely.
**Why it happens:** The parent repo is named `escluse`, and all sub-repos are cloned into it. The directory structure looks like a monorepo.
**How to avoid:** Include a explicit "Which Repo to Contribute To" section with the repo mapping table. Explain that each service is its own GitHub repo.
**Warning signs:** PRs on the parent repo that should be on individual service repos.

### Pitfall 3: Forgetting Changelog Updates
**What goes wrong:** Feature is merged, but the changelog (in `landing-page-escluse/src/pages/Changelog.tsx`) isn't updated. Next deploy goes out without release notes.
**Why it happens:** Changelog lives in a separate repo (`escluse-landing-page`) from the code changes. Easy to forget.
**How to avoid:** Include changelog update as a step in the PR checklist. Mention that changelog entries go in `landing-page-escluse/src/pages/Changelog.tsx` with the SemVer format documented in `SEMVER.md`.
**Warning signs:** PR description doesn't mention changelog update.

### Pitfall 4: No Code Review Process Defined
**What goes wrong:** Contributors submit PRs and wait indefinitely for review. No reviewer is assigned, no SLA is set.
**Why it happens:** Single-developer project with no documented review expectations.
**How to avoid:** Be honest about the review process in CONTRIBUTING.md. State expected review timelines (or lack thereof for a small team). Suggest contributors ping on community channels.
**Warning signs:** "Code review" section absent from CONTRIBUTING.md.

### Pitfall 5: No CI/Gating Mentioned
**What goes wrong:** Contributor submits PR with failing tests or broken builds because no CI is configured or the CI status is unclear.
**Why it happens:** The project has no `.github/workflows/` visible in the parent repo (though CI exists via Phase 50's release automation).
**How to avoid:** Document what CI checks exist (if any) per repo. Be transparent about CI coverage gaps (e.g., "Frontend tests not yet configured").
**Warning signs:** PR template doesn't include a CI checklist.

---

## Code Examples

### Repo Contribution Table (recommended for CONTRIBUTING.md)
```markdown
| Area | GitHub Repo | Directory in Parent |
|------|-------------|---------------------|
| Backend API + Worker | `esclusehq/escluse-cloud` | `api/`, `worker/` |
| Frontend Dashboard | `esclusehq/escluse-dashboard` | `app/` |
| Web Agent | `esclusehq/solys` | `agent/solys/` |
| Shared Agent Crates | `esclusehq/agent-core` | `agent/agent-core/` |
| Documentation | `esclusehq/escluse-docs` | `docs/` |
| Infrastructure | `esclusehq/escluse-infra` | `gateway/`, root files |
| Landing Page | `esclusehq/escluse-landing-page` | `landing-page-escluse/` |
| SDK | `esclusehq/escluse-sdk` | `packages/` |
| Orchestration (this repo) | `esclusehq/escluse` | `./` |
```

### Commit Message Format (from PUSH_COMMIT.md)
```text
type(scope): description

Types:
  feat    - New feature
  fix     - Bug fix
  docs    - Documentation
  refactor - Code refactoring
  chore   - Maintenance, dependency updates

Examples:
  feat(sdk): add WebSocket support for real-time events
  fix(changelog): update v0.2.0 release notes
  docs: update installation steps in README.md
```

### Changelog Entry Format (from SEMVER.md)
```typescript
{
  version: '0.1.0',
  date: '2026-05-17',
  type: 'minor',  // 'major' | 'minor' | 'patch' | 'initial'
  changes: {
    added: ['New feature 1', 'New feature 2'],
    improved: ['Improvement 1'],
    fixed: ['Bug fix 1'],
    removed: ['Removed feature'],
    security: ['Security fix']
  }
}
```

### Issue Label Schema (from PUSH_COMMIT.md)
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

### Verified Test Commands (from codebase)
```bash
# Backend API
cd api
cargo test                    # Run all tests
cargo test -- --nocapture     # Show output
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

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No CONTRIBUTING.md | Structured contributor guide | Phase 62 | Formalizes contribution process |
| No CODE_OF_CONDUCT.md | Optional addition alongside | Phase 62 (recommended) | Standards-compliant open source |
| No issue/PR templates | Optional template creation | Phase 62 (recommended) | Consistent issue reporting |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The project welcomes external contributors beyond the core team | Summary | If project is internal-only, CONTRIBUTING.md may be unnecessary |
| A2 | CONTRIBUTING.md should be in English | Standard Stack | If primary language is Indonesian, bilingual or Indonesian version may be preferred |
| A3 | No CODE_OF_CONDUCT.md, issue templates, or PR templates exist | Summary | Confirmed by glob searches — no such files found |
| A4 | The project uses `master` as the primary branch | Common Pitfalls | All PUSH_COMMIT.md commands reference `git push origin master` |

---

## Open Questions (RESOLVED)

1. **[Language: English vs. Indonesian]**
   - **RESOLVED:** English primary with a `## Bahasa Indonesia` section at the end. Matches project's split-language pattern (English for external docs, Indonesian for internal workflow).
   - Recommendation: English primary + Bahasa Indonesia note adopted in plan.

2. **[PR Review Expectations]**
   - **RESOLVED:** Describe current workflow honestly with acknowledgment of small team size. State expected response time and recommend communication channels.
   - Recommendation: Adopted in plan — realistic review timelines documented.

3. **[Companion Files]**
   - **RESOLVED:** Create CODE_OF_CONDUCT.md (Contributor Covenant) and basic PULL_REQUEST_TEMPLATE.md alongside CONTRIBUTING.md. Issue templates deferred.
   - Recommendation: Implemented in plan Tasks 2 and 3.

4. **[CI/CD Coverage in CONTRIBUTING.md]**
   - **RESOLVED:** Document what CI exists and what doesn't. Be transparent about gaps.
   - Recommendation: Adopted in plan — CI gaps documented as area for future improvement.

---

## Environment Availability

> **SKIPPED** — This is a pure documentation phase. No tools or services need to be available. The machine environment does not affect the creation of markdown files.

---

## Validation Architecture

> **SKIPPED** — `workflow.nyquist_validation` is explicitly set to `false` in `.planning/config.json`. No test infrastructure or validation coverage is required for this phase.

---

## Security Domain

> **SKIPPED** — This is a pure documentation phase. No runtime code, no authentication/authorization, no data handling. Security considerations are limited to standard open-source contribution practices (don't commit secrets, verify signed commits if required).

---

## Sources

### Primary (HIGH confidence) — Verified from codebase
- `PUSH_COMMIT.md` — Commit conventions, repo mapping, issue labels, GitHub Projects workflow
- `SEMVER.md` — Versioning rules, changelog format, commit types
- `DEVELOPMENT.md` — Existing local setup docs (must not duplicate)
- `dev/*` — 5 setup sub-files from Phase 61
- `DEPLOY.md` — Deployment workflow, repo URLs
- `.planning/codebase/CONVENTIONS.md` — Naming, code style, imports
- `.planning/codebase/TESTING.md` — Test framework details, commands
- `.planning/codebase/STACK.md` — Technology stack versions
- `.planning/codebase/STRUCTURE.md` — Directory layout
- `.planning/codebase/ARCHITECTURE.md` — Service architecture
- `.planning/codebase/INTEGRATIONS.md` — External integrations
- `.github/profile/README.md` — Organization profile
- Git log — Verified conventional commit format in practice

### Secondary (MEDIUM confidence)
- No `.github/ISSUE_TEMPLATE/` directory — verified no existing issue templates
- No `.github/workflows/` in parent repo — CI/CD workflows exist per-service but not centralized
- No `CODE_OF_CONDUCT.md` — verified by glob search
- No `.github/PULL_REQUEST_TEMPLATE.md` — verified by glob search

### Tertiary (LOW confidence)
- None — all findings verified against codebase inspection

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Verified codebase files, commit history, existing docs
- Architecture: HIGH — Verified repo structure, git analysis, planning files
- Pitfalls: HIGH — Based on discovered meta-repo structure and comparison with DEVELOPMENT.md

**Research date:** 2026-05-31
**Valid until:** 2026-07-01 (stable documentation phase — conventions unlikely to change frequently)
