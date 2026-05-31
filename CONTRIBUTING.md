# Escluse — Contributing Guide

Esluce is a game server management platform. We welcome contributions from everyone — whether you're fixing a bug, adding a feature, improving documentation, or helping with maintenance.

![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen)
![Conventional Commits](https://img.shields.io/badge/commits-conventional-blue)
![Contributor Covenant](https://img.shields.io/badge/CoC-v2.1-ff69b4)

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Which Repo to Contribute To](#which-repo-to-contribute-to)
- [How to Report Bugs](#how-to-report-bugs)
- [How to Request Features](#how-to-request-features)
- [Contribution Workflow](#contribution-workflow)
- [Commit Message Conventions](#commit-message-conventions)
- [Coding Standards](#coding-standards)
- [Testing Expectations](#testing-expectations)
- [Changelog](#changelog)
- [Code Review](#code-review)
- [Getting Help](#getting-help)
- [Bahasa Indonesia](#bahasa-indonesia)

## Code of Conduct

This project and everyone participating in it is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior.

## Which Repo to Contribute To

Esluce is organized as a **meta-repo**: the parent repository (`esclusehq/escluse`) contains multiple independent repositories cloned into subdirectories. They are NOT submodules — each is its own GitHub repo with its own issues, branches, and PRs.

> **Warning:** Contributing to the wrong repo is a common mistake. If you're fixing a backend bug, you need to open the PR on `esclusehq/escluse-cloud`, not the parent repo.

| Area | GitHub Repository | Directory | Language |
|------|------------------|-----------|----------|
| Backend API + Worker | `esclusehq/escluse-cloud` | `api/`, `worker/` | Rust (Axum) |
| Frontend Dashboard | `esclusehq/escluse-dashboard` | `app/` | React 19 + TypeScript |
| Game Server Agent | `esclusehq/solys` | `agent/solys/` | Rust |
| Agent Core Crates | `esclusehq/agent-core` | `agent/agent-core/` | Rust (12 crates) |
| Documentation Site | `esclusehq/escluse-docs` | `docs/` | VitePress |
| Infrastructure | `esclusehq/escluse-infra` | `gateway/` | Docker, Caddy |
| Landing Page | `esclusehq/escluse-landing-page` | `landing-page-escluse/` | Vite |
| SDK | `esclusehq/escluse-sdk` | `packages/` | Node.js, Python |
| Orchestration (this repo) | `esclusehq/escluse` | `./` | Config, docs |

For local setup instructions, see [DEVELOPMENT.md](DEVELOPMENT.md).

## How to Report Bugs

Open an issue on the **specific repo** where the bug occurs. Use the Bug Report issue template if available.

Include: steps to reproduce, expected behavior, actual behavior, and environment details (OS, browser, versions).

Labels: `bug`, plus `priority:high` if critical.

## How to Request Features

Open an issue on the relevant repo with a Feature Request template.

Describe: the problem you're solving, your proposed solution, and any alternatives considered. Label: `feature`.

## Contribution Workflow

1. **Find or create an issue** — Search existing issues or create a new one. All changes must be tracked via GitHub Projects with appropriate labels.

2. **Fork or branch** — If you're an external contributor, fork the relevant repo. If you're a team member, create a feature branch from `master`.

3. **Make your changes** — Follow coding standards defined below. Run tests before committing.

4. **Commit using conventional commits** — See [Commit Message Conventions](#commit-message-conventions).

5. **Open a Pull Request** — Open a PR on the **specific repo** against the `master` branch. Fill out the PR template (`.github/PULL_REQUEST_TEMPLATE.md`).

6. **Code review** — A maintainer will review your PR. Response time varies (small team). Feel free to ping on Discord if no response within a week.

7. **Merge** — Once approved and CI passes (where configured), a maintainer will merge your changes.

## Commit Message Conventions

This project uses **Conventional Commits** (enforced across all repos):

| Type | Description | Example |
|------|-------------|---------|
| `feat` | A new feature | `feat(sdk): add WebSocket support for real-time events` |
| `fix` | A bug fix | `fix(changelog): update v0.2.0 release notes` |
| `docs` | Documentation changes | `docs: update installation steps` |
| `refactor` | Code restructuring (no behavior change) | `refactor(api): extract auth middleware` |
| `chore` | Maintenance, deps, tooling | `chore: bump dependencies` |

Format: `type(scope): description` (lowercase, no period at end).

See [PUSH_COMMIT.md](PUSH_COMMIT.md) for the full convention guide (in Indonesian).

## Coding Standards

Each subsystem has its own conventions documented in the `.planning/codebase/` directory.

- Rust: `cargo clippy` must pass. Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Frontend: `npm run lint` must pass. Follow existing component patterns.
- All new public APIs should include documentation comments.
- No secrets, credentials, or tokens in any commit.

## Testing Expectations

Run relevant tests and lints before opening a PR:

```bash
# Backend API
cd api && cargo test && cargo clippy
# Worker
cd worker && cargo test && cargo clippy
# Web Agent
cd agent/solys && cargo test && cargo clippy
# Agent Core (12 crates)
cd agent/agent-core && cargo test --workspace && cargo clippy --workspace
# Frontend
cd app && npm run lint
```

> **Note:** Frontend tests are not yet configured — `npm run lint` is the only CI check available for `app/`.

Full command reference: [dev/04-commands.md](dev/04-commands.md).

## Changelog

Changelog entries are maintained in `landing-page-escluse/src/pages/Changelog.tsx` (separate repo).

Format: version, date, type (major/minor/patch/initial), and change categories (added/improved/fixed/removed/security).

For every feature or fix, include a changelog entry in your PR. See [SEMVER.md](SEMVER.md) for the full convention (in Indonesian).

## Code Review

This is a small team project. Review timelines may vary.

PRs are reviewed based on: correctness, test coverage, convention compliance, and documentation completeness.

Expected response: initial review within 1-3 business days (not guaranteed).

If your PR hasn't received attention within a week, reach out via Discord.

## Getting Help

- Discord: [discord.esluce.com](https://discord.esluce.com)
- Email: dev@esluce.com

Questions about contributing? Open a Discussion on the relevant repo.

## Bahasa Indonesia

Kontributor Indonesia dipersilakan! Dokumentasi internal (PUSH_COMMIT.md, SEMVER.md, STRATEGI.md) tersedia dalam Bahasa Indonesia.

Issue, PR, dan komentar dapat ditulis dalam Bahasa Indonesia atau Inggris.
