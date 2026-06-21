# Escluse Community Bot

## What This Is

A comprehensive Discord community bot for the Escluse ecosystem that serves as an information hub, onboarding assistant, support platform, developer companion, and community management system. It helps users succeed with Escluse while reducing manual work for the founder and supporting community growth — transforming Discord into the central hub of the Escluse ecosystem.

## Core Value

The bot must serve as the central, reliable hub of the Escluse community — allowing users to learn, get support, share projects, follow development, and feel connected even when the founder is offline.

## Requirements

### Validated

<!-- Existing codebase: escluse monorepo with Rust backend, TypeScript frontend, Docker infrastructure -->

- ✓ Escluse core platform (Rust/Cargo) — existing
- ✓ Escluse API service — existing
- ✓ Escluse web app — existing
- ✓ Docker deployment infrastructure — existing

### Active

- [ ] **ONB-01**: Bot automatically welcomes new members with getting-started guides, links, and resources
- [ ] **ONB-02**: Interest selection system (Self-Hosting, Minecraft, Game Server, Development, Exploring Alternatives)
- [ ] **ONB-03**: Automatic role assignment based on selected interests
- [ ] **INFO-01**: Documentation search command (`/escluse docs`)
- [ ] **INFO-02**: Pricing and feature information commands (`/escluse pricing`, `/escluse what-is`)
- [ ] **INFO-03**: Roadmap viewing command (`/escluse roadmap`)
- [ ] **INFO-04**: FAQ with quick answers
- [ ] **SUPP-01**: Support ticket system
- [ ] **SUPP-02**: Structured bug reporting (version, OS, steps, logs)
- [ ] **SUPP-03**: Troubleshooting guides and question channels
- [ ] **UPDT-01**: Release announcements and changelog updates
- [ ] **UPDT-02**: Development milestone notifications
- [ ] **UPDT-03**: Maintenance notices
- [ ] **SHOW-01**: Server showcase submission with name, game, description, screenshots
- [ ] **SHOW-02**: Media sharing and deployment stories
- [ ] **FDBK-01**: Feature voting and game support polls
- [ ] **FDBK-02**: Community surveys and template requests
- [ ] **DEV-01**: GitHub integration (issue tracking, release notifications)
- [ ] **DEV-02**: SDK resources and contribution information
- [ ] **RECG-01**: Referral tracking system
- [ ] **RECG-02**: Contribution recognition and community achievements
- [ ] **RECG-03**: Special roles (Early Supporter, Beta Tester, Contributor, Community Helper)
- [ ] **PERM-01**: Role-based permission system for all commands
- [ ] **ADMN-01**: Channel management (create, edit, delete)
- [ ] **ADMN-02**: Role management
- [ ] **ADMN-03**: Server settings configuration
- [ ] **ADMN-04**: Member logging

### Out of Scope

- Chat moderation (warn, mute, kick, ban, purge) — v2
- Intelligent AI assistant — long-term vision
- Integration with existing escluse backend — standalone first
- Mobile app — web-first via Discord

## Context

- Built as a subdirectory (`escluse-bot`) within the larger escluse monorepo
- Existing escluse ecosystem: Rust backend (Cargo), TypeScript frontend, Docker deployment
- The bot is standalone and does not depend on existing escluse infrastructure initially
- Community is in early stages — bot should help build from ground up

## Constraints

- **Tech Stack (Bot)**: TypeScript + Bun + Discord.js
- **Tech Stack (API)**: Rust (Axum)
- **Database**: PostgreSQL
- **Cache**: Redis
- **Deployment**: Docker (matching existing escluse infra patterns)
- **Standalone**: No dependency on existing escluse backend for v1

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| TypeScript + Bun + Discord.js | Modern runtime, fast dev cycle, rich Discord.js ecosystem | — Pending |
| Rust Axum for API service | Consistent with existing escluse Rust stack, high performance | — Pending |
| PostgreSQL + Redis | Reliable persistence + fast caching layer | — Pending |
| Standalone first | Avoid coupling with existing backend during initial development | — Pending |
| Monorepo subdirectory | Keep code close to existing escluse project | — Pending |

---

*Last updated: 2026-06-18 after initialization*
