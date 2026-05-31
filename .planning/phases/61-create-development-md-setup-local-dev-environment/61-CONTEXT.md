# Phase 61: Create DEVELOPMENT.md - Setup local dev environment - Context

**Gathered:** 2026-05-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a DEVELOPMENT.md entry point and docs/dev/ sub-files for developers to set up the project locally. Documents the full workflow: prerequisites, environment setup, configuration, common commands, and troubleshooting.

The result is a developer-facing reference, not a contribution guide (that's Phase 62). It covers how to run all services (API, Worker, Web Agent, Frontend) locally for development.

</domain>

<decisions>
## Implementation Decisions

### D-01: Setup Approach — Hybrid Docker Infra + Local Dev Tools
**Decision:** Document a hybrid approach:
- Docker (docker compose up) for infrastructure services (PostgreSQL 16, Redis 7)
- Local Rust toolchain + Node.js for active development of API, Worker, Agent, and Frontend

Rationale: Fastest path for contributors. Docker handles service dependencies; local tools provide fast edit-run cycles for each service.

### D-02: Tool Prerequisites — Table with OS-Specific Commands
**Decision:** Include a prerequisites table listing exact versions:
- Docker & Docker Compose
- Rust toolchain (via rustup)
- Node.js 20+ (via nvm or direct install)
- Supabase CLI (for local Supabase)

Include copy-paste install commands for Linux (apt), macOS (brew), and Windows (winget).

### D-03: Docker Configuration — Document Env Overrides
**Decision:** Use existing `docker-compose.yml` as-is. Document the environment variable overrides needed for local dev (ports, volumes, etc.) rather than creating a separate dev compose file.

### D-04: Document Individual Services, Not Just Full-Stack
**Decision:** Include shell commands to run each service separately for debugging:
- `cargo run -p api` — Backend API (port 3000)
- `cargo run -p worker` — Background job processor
- `cargo run -p web-agent` — Node agent
- `npm run dev` — Frontend (Vite dev server)

### D-05: .env Configuration — Inline Values in Doc
**Decision:** Provide complete copy-paste-ready .env content for both `api/.env` and `app/.env` directly in the documentation, with dev-friendly values (localhost URLs, demo passwords).

### D-06: Supabase — Local via Supabase CLI
**Decision:** Document Supabase local setup using `supabase init` + `supabase start` commands, not a cloud project.

### D-07: Optional Service Keys — Listed with "Optional" Note
**Decision:** Document Stripe, Resend, Discord webhook, and other optional keys as "optional" and note the app runs without them for basic development.

### D-08: Configuration Docs — Profile Per Service
**Decision:** Organize configuration documentation as individual profiles per external service:
- PostgreSQL
- Redis
- Supabase (Auth)
- Stripe (optional)
- Resend/Email (optional)
- Discord Webhooks (optional)

### D-09: Command Organization — Grouped by Service
**Decision:** Commands grouped under service headers:
- **API** (cargo build/run/test/clippy, sqlx migrate)
- **Worker** (cargo build/run/test)
- **Web Agent** (cargo build/run/test)
- **Frontend** (npm install/dev/build/test/lint)

### D-10: Shell Snippets Only — No Build File
**Decision:** Document raw shell commands in code blocks. Do not create a Makefile or Justfile.

### D-11: Include Test and Lint Commands
**Decision:** Include `cargo test`, `cargo clippy`, `npm test`, `npm run lint` commands alongside build/run commands.

### D-12: Include End-to-End Workflow Example
**Decision:** Include a complete workflow from `git clone` through first PR, with all intermediate steps (install prerequisites → docker compose up → configure .env → run migrations → start services → run tests).

### D-13: Document Structure — Root Entry + Multi-File Under docs/dev/
**Decision:**
- `DEVELOPMENT.md` (repo root) — Entry point with quick start and table of contents
- `docs/dev/01-prerequisites.md` — Prerequisites table per OS
- `docs/dev/02-setup.md` — Setup steps (Docker infra, local tools, Supabase)
- `docs/dev/03-configuration.md` — Configuration profiles per service
- `docs/dev/04-commands.md` — Commands grouped by service + end-to-end workflow
- `docs/dev/05-troubleshooting.md` — 3-5 common issues with solutions

### The Agent's Discretion
- Exact wording and formatting of shell commands
- Specific troubleshooting entries and their solutions
- Layout of the prerequisites table
- Markdown formatting and code block styles
- Whether to include badges or status indicators in DEVELOPMENT.md

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Configuration & Deployment
- `docker-compose.yml` — Full stack Docker Compose (PostgreSQL, Redis, backend, frontend, gateway)
- `api/.env.example` — API environment variable reference
- `app/.env.example` — Frontend environment variable reference

### Codebase Structure & Tech Stack
- `.planning/codebase/STACK.md` — Tech stack with exact versions (Rust edition 2021, React 19, Axum 0.7, etc.)
- `.planning/codebase/STRUCTURE.md` — Directory layout and where to add new code
- `.planning/codebase/CONVENTIONS.md` — Coding conventions (naming, imports, style)
- `.planning/codebase/ARCHITECTURE.md` — Service architecture and patterns

### Phase 62 (Next in Chain)
- `.planning/ROADMAP.md` § Phase 62 — Create CONTRIBUTING.md (depends on Phase 61)

### Phase Goal
- `.planning/ROADMAP.md` § Phase 61 — "Create DEVELOPMENT.md - Setup local dev environment"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `docker-compose.yml` — Infrastructure definition for PostgreSQL, Redis, backend, frontend, gateway. Directly usable for dev setup.
- `api/.env.example` — 64-line env reference with all keys. Source for dev-friendly .env documentation.
- `app/.env.example` — 3-line Supabase + API URL config.
- `api/migrations/` — SQLx migration files; `sqlx migrate run` command opens the database.

### Established Patterns
- Documentation lives in `docs/` directory (VitePress docs site exists at `docs/`).
- `.env.example` files live alongside each service's entry point.
- Docker Compose for infrastructure, Cargo for Rust services, npm/Vite for frontend.

### Integration Points
- `DEVELOPMENT.md` links to `docs/dev/*.md` sub-files.
- After this phase completes, Phase 62 (CONTRIBUTING.md) will reference DEVELOPMENT.md for setup instructions.
- The docs site at `docs/` could cross-reference DEVELOPMENT.md sections.

</code_context>

<specifics>
## Specific Ideas

The DEVELOPMENT.md structure should follow the convention of a README-style entry point at repo root with detailed sub-pages under docs/dev/. Developers should be able to go from zero to running the full stack in under 10 minutes by following the documented path.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 61-create-development-md-setup-local-dev-environment*
*Context gathered: 2026-05-31*
