# Phase 61: Create DEVELOPMENT.md - Setup Local Dev Environment - Research

**Researched:** 2026-05-31
**Domain:** Developer documentation, local dev environment setup
**Confidence:** HIGH

## Summary

This phase delivers a `DEVELOPMENT.md` entry point at the repo root plus 5 sub-files under `docs/dev/` that document the complete workflow for setting up Escluse's local development environment. The project uses a **hybrid approach**: Docker Compose for infrastructure services (PostgreSQL 16, Redis 7) and local Rust/Node.js toolchains for active development of API, Worker, Agent, and Frontend.

**Critical architectural discovery:** The project is a **meta-repo** — `api/`, `app/`, `landing-page-escluse/`, `docs/`, `agent/agent-core/`, `agent/solys/`, `packages/`, and `migration/` are all **independent git repositories** (not git submodules), each with their own remote on `github.com/esclusehq`. The parent repo (`esclusehq/escluse`) orchestrates the infrastructure (`docker-compose.yml`, `gateway/`, root-level files). Developers must clone these sub-repos separately into the correct directories. This is the single most important detail the planner must communicate in the setup workflow.

**Primary recommendation:** Follow the 13 user decisions (D-01 through D-13) exactly. Create the root `DEVELOPMENT.md` as a quick-start entry point with TOC links, and populate `docs/dev/01-prerequisites.md` through `docs/dev/05-troubleshooting.md` with the content mapped out in the discussion phase. The end-to-end workflow must cover sub-repo cloning explicitly, as this is the most likely friction point for new developers.

**No requirement IDs** were provided for this phase — it is a developer experience / documentation phase with no functional requirements to map.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Hybrid Docker infra + local dev tools
- **D-02:** Prerequisites table with OS-specific install commands
- **D-03:** Use existing docker-compose.yml, document env overrides
- **D-04:** Document individual service run commands (cargo run -p api, worker, web-agent, npm run dev)
- **D-05:** Inline copy-paste .env content for both api/.env and app/.env
- **D-06:** Local Supabase via Supabase CLI
- **D-07:** Optional service keys listed as "optional" with note
- **D-08:** Configuration docs as profiles per service (PostgreSQL, Redis, Supabase, Stripe, Resend, Discord)
- **D-09:** Commands grouped by service (API, Worker, Agent, Frontend)
- **D-10:** Shell snippets only — no Makefile or Justfile
- **D-11:** Include test and lint commands
- **D-12:** Include end-to-end workflow example
- **D-13:** Multi-file structure: root DEVELOPMENT.md + docs/dev/01-prerequisites.md through docs/dev/05-troubleshooting.md

### The Agent's Discretion
- Exact wording and formatting of shell commands
- Specific troubleshooting entries and their solutions
- Layout of the prerequisites table
- Markdown formatting and code block styles
- Whether to include badges or status indicators in DEVELOPMENT.md

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope.
</user_constraints>

---

## Architectural Responsibility Map

This phase is purely documentation — it does not add runtime code. The "capabilities" are documentation sections, mapped to documentation structure decisions:

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Prerequisites table | `DEVELOPMENT.md` (root) | `docs/dev/01-prerequisites.md` | D-13 specifies root entry + sub-file |
| Setup instructions | `docs/dev/02-setup.md` | — | Detailed setup workflow in sub-file |
| Configuration reference | `docs/dev/03-configuration.md` | — | Per-service config profiles |
| Command reference | `docs/dev/04-commands.md` | — | Grouped by service |
| Troubleshooting | `docs/dev/05-troubleshooting.md` | — | 3-5 common issues |
| Quick start / TOC | `DEVELOPMENT.md` (root) | — | Entry point linking to sub-files |

No runtime tiers are involved — this is a doc-only phase.

---

## Standard Stack

### Documentation Format
| Aspect | Standard | Why |
|--------|----------|-----|
| Format | Markdown (GFM) | Universal, renders on GitHub and VitePress |
| Code blocks | Fenced with language tags (bash, toml, dockerfile) | Syntax highlighting |
| Tables | GFM pipe tables | Prerequisites, env vars |
| Admonitions | None / plain | D-10 shell snippets only — no build tools |

### Existing Docs Infrastructure
| Tool | Version | Purpose |
|------|---------|---------|
| VitePress | latest (deployed at docs.esluce.com) | Documentation website |
| GitHub-Flavored Markdown | — | All `.md` files render on GitHub |

**Key insight:** The docs site at `docs/` is a **separate sub-repo** (`esclusehq/escluse-docs`). DEVELOPMENT.md and docs/dev/*.md live in the **parent repo** (`esclusehq/escluse`), not in the docs sub-repo. They will be viewed primarily on GitHub, not on the VitePress docs site. Format as plain GitHub Markdown.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Root DEVELOPMENT.md | All under docs/dev/ | D-13 chose root entry point for discoverability |
| Shell snippets | Makefile / Justfile | D-10 chose raw shell commands, no build tool |
| Multi-file | Single file | D-13 chose split for maintainability |

---

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                 PARENT REPO (esclusehq/escluse)          │
│  docker-compose.yml, gateway/, root .md files, .planning │
└─────────────────────┬───────────────────────────────────┘
                      │ separate git repos (not submodules)
                      │ (clone independently then arrange)
     ┌────────────────┼──────────────────┬──────────────────┐
     ▼                ▼                  ▼                  ▼
┌──────────┐   ┌──────────┐   ┌──────────────┐   ┌───────────┐
│  api/    │   │  app/    │   │  worker/     │   │  docs/    │
│(backend) │   │(frontend)│   │(background)  │   │(VitePress)│
│ Rust/Axum│   │ React 19 │   │ Rust service │   │ API docs  │
│ Port 3000│   │Vite 5173 │   │ (api subdir) │   │doc site   │
└──────────┘   └──────────┘   └──────────────┘   └───────────┘

     ┌──────────────────┐   ┌───────────────────────┐
     │  agent/solys/    │   │  agent/agent-core/    │
     │  (Web Agent)     │   │  (shared Rust crates) │
     │  Rust + Bollard  │   │  12 crate workspace   │
     └──────────────────┘   └───────────────────────┘

     ┌──────────────────┐   ┌───────────────────────┐
     │ landing-page/    │   │  packages/            │
     │ Vite + TypeScript│   │  SDK (Node, Python)   │
     └──────────────────┘   └───────────────────────┘

              ▼
┌──────────────────────────────────┐
│   DOCKER COMPOSE INFRASTRUCTURE  │
│  ┌──────────┐  ┌──────────┐     │
│  │PostgreSQL│  │  Redis 7 │     │
│  │  16-alp  │  │(password)│     │
│  └──────────┘  └──────────┘     │
│  Port 5432       Port 6379      │
└──────────────────────────────────┘
```

**Data flow for local development:**
1. Developer runs `docker compose up postgres redis` → infra starts
2. Developer runs `cargo run -p api` in `api/` → Rust backend on :3000
3. Developer runs `npm run dev` in `app/` → Vite dev server on :5173, proxies `/api/v1` → :3000
4. Developer runs `cargo run -p worker` in `worker/` → background job processor
5. Developer runs agent binary in `agent/solys/` → connects to backend via WebSocket
6. All services connect to PostgreSQL (:5432) and Redis (:6379)

### Recommended File Structure (per D-13)
```
escluse/                              # Parent repo root
├── DEVELOPMENT.md                    # Entry point / Quick start / TOC
├── docs/
│   └── dev/
│       ├── 01-prerequisites.md        # Tool prerequisites per OS
│       ├── 02-setup.md                # Clone, Docker infra, Supabase, .env
│       ├── 03-configuration.md        # Per-service config profiles
│       ├── 04-commands.md             # Commands grouped by service + e2e workflow
│       └── 05-troubleshooting.md      # 3-5 common issues
```

### Anti-Patterns to Avoid
- **Assuming Cargo workspace at root:** There is no root `Cargo.toml`. Each Rust service (`api/`, `worker/`, `agent/solys/`) has its own `Cargo.toml`. Commands like `cargo run -p api` work from the `api/` directory. `cargo test` also runs per-directory.
- **Forgetting sub-repo structure:** New developers will not know api/ and app/ are separate repos. The clone workflow must explicitly list each repo + target directory.
- **Hardcoding Docker host credentials:** The docker-compose.yml contains production credentials (PostgreSQL password, Redis password). Dev documentation must provide different dev-friendly credentials.
- **Confusing DEVELOPMENT.md with docs site:** The docs site (VitePress at docs/) is a separate repo for API documentation. DEVELOPMENT.md lives in the parent repo and is GitHub-rendered, NOT part of the VitePress docs.
- **Omitting Supabase CLI from prerequisites:** D-06 requires local Supabase. The Supabase CLI is NOT installed on this machine and must be included in the prerequisites table with install commands.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Infrastructure setup | Manual install of PostgreSQL/Redis | `docker compose up postgres redis` | D-01: Docker handles service dependencies, fast for contributors |
| Local Supabase | Manual Supabase project config | `supabase init && supabase start` | D-06: Local CLI avoids cloud project dependency |
| Config documentation | One-off custom format | Inline copy-paste .env content | D-05: Ready to use, no interpretation needed |
| Build automation | Makefile / Justfile | Raw shell commands | D-10: Simpler, more transparent |
| Dev commands reference | One-line "see docker-compose" | Per-service grouped commands | D-09 + D-04: Debugging requires individual service control |

**Key insight:** This is a documentation phase. The "don't hand-roll" principle applies to the documentation approach itself — use standard Markdown patterns (tables for prerequisites, fenced code blocks for commands, ordered lists for workflows) rather than inventing custom formats.

---

## Runtime State Inventory

> **SKIPPED** — This is a greenfield documentation phase. No rename, refactor, or migration is involved. No existing DEVELOPMENT.md or docs/dev/ files exist to migrate.

---

## Common Pitfalls

### Pitfall 1: Sub-Repo Clone Confusion
**What goes wrong:** Developer clones only `esclusehq/escluse`, runs `docker compose up`, then gets build errors because `api/` and `app/` directories are empty.
**Why it happens:** The parent repo `.gitignore` lists `api/`, `app/`, `agent/agent-core/`, `agent/solys/`, `landing-page-escluse/`, `packages/`, `migration/` under "Submodules / Embedded repos (tracked separately)" — they are NOT git submodules with `.gitmodules` entries. They are standalone repos cloned into those directories.
**How to avoid:** Include explicit clone + directory setup steps in the end-to-end workflow. List every sub-repo with its GitHub URL and target directory in a table.
**Warning signs:** Empty `api/src/` or `app/src/` after cloning parent repo.

### Pitfall 2: Missing Supabase CLI
**What goes wrong:** Developer follows setup steps, hits `supabase init` command, gets "command not found".
**Why it happens:** Supabase CLI is a separate install not commonly present — it must be installed via npm (`npm install -g supabase`), brew, apt, or direct binary download.
**How to avoid:** Explicit Supabase CLI install commands in the prerequisites table for each OS.
**Warning signs:** The environment check confirmed Supabase CLI is NOT installed on this machine.

### Pitfall 3: Hardcoded Production Credentials
**What goes wrong:** Developer copies docker-compose.yml production credentials (PostgreSQL password, Redis password, JWT_SECRET) into their .env files, then later accidentally commits them.
**Why it happens:** docker-compose.yml contains real-looking passwords and a JWT_SECRET. The documentation must provide dev-friendly alternatives.
**How to avoid:** D-05 mandates inline copy-paste .env content with **dev-friendly values** (e.g., localhost URLs, simple passwords like `dev_password`). Document that these are for local dev only.
**Warning signs:** Developer uses `dU4J99CaxFSQti9jqucZ` (production Redis password) in local .env.

### Pitfall 4: Cargo Workspace Confusion
**What goes wrong:** Developer runs `cargo run -p api` from the repo root and gets "can't find crate `api`".
**Why it happens:** Each Rust service has its own `Cargo.toml` — there is NO root workspace Cargo.toml. Commands must be run from within each service's directory.
**How to avoid:** Prefix every Cargo command with the explicit `cd api && cargo run ...` or `cd worker && cargo run ...`. Document clearly that each Rust service is standalone.
**Warning signs:** `cargo run -p api` fails from repo root.

### Pitfall 5: Port Conflicts
**What goes wrong:** PostgreSQL port 5432 or Redis port 6379 are already in use on the developer's machine.
**Why it happens:** Common issue — developer may already have these services running natively.
**How to avoid:** Include port conflict troubleshooting in the troubleshooting file. Document how to stop local services or change Docker port mappings.
**Warning signs:** `docker compose up` fails with "port is already allocated".

---

## Code Examples

### Verifying Tools (from Prerequisites section)
```bash
# Docker
docker --version
docker compose version

# Node.js (v20+)
node --version

# Rust toolchain (via rustup)
rustup show
cargo --version

# Supabase CLI
supabase --version
```

### Docker Compose Infrastructure
```bash
# Start only infrastructure services (recommended for local dev)
docker compose up postgres redis -d

# Verify they're running
docker compose ps

# Stop infrastructure
docker compose down
```

### API Backend (Rust/Axum, port 3000)
```bash
cd api
cp .env.example .env       # Then edit .env with dev values
cargo run                   # Starts on http://localhost:3000
cargo test                  # Run tests
cargo clippy                # Lint check
```

### Frontend (React 19 + Vite, port 5173)
```bash
cd app
cp .env.example .env       # Then edit .env with dev values
npm install
npm run dev                 # Starts on http://localhost:5173
                            # Proxies /api/v1 -> http://localhost:3000
```

### Worker (Background Job Processor)
```bash
cd worker
cp .env.example .env       # Then edit .env with dev values
cargo run                   # Starts job processor
```

### Solys Agent (Web Agent)
```bash
cd agent/solys
# Build and run the agent
cargo run                   # Connects to backend via WebSocket
```

### Supabase Local Setup
```bash
# Initialize Supabase project
supabase init

# Start local Supabase (uses Docker containers)
supabase start

# After starting, Supabase CLI outputs:
# - API URL: http://localhost:54321
# - anon key: (copy this to app/.env)
# - service_role key: (copy this to api/.env)
```

### SQLx Migrations
```bash
cd api
# Run pending migrations
DATABASE_URL="postgresql://server:password@localhost:5432/backend_db" \
  sqlx migrate run

# Revert last migration
DATABASE_URL="postgresql://server:password@localhost:5432/backend_db" \
  sqlx migrate revert
```

### Complete End-to-End Workflow
```bash
# 1. Clone parent repo
git clone https://github.com/esclusehq/escluse.git
cd escluse

# 2. Clone sub-repos (each is a separate git repo)
git clone https://github.com/esclusehq/escluse-cloud.git api
git clone https://github.com/esclusehq/escluse-dashboard.git app
git clone https://github.com/esclusehq/escluse-docs.git docs
git clone https://github.com/esclusehq/solys.git agent/solys
git clone https://github.com/esclusehq/agent-core.git agent/agent-core
git clone https://github.com/esclusehq/escluse-landing-page.git landing-page-escluse
git clone https://github.com/esclusehq/escluse-sdk.git packages

# 3. Verify tools (see prerequisites)
docker --version
node --version
rustc --version
supabase --version

# 4. Start infrastructure
docker compose up postgres redis -d

# 5. Set up Supabase local
supabase start

# 6. Configure .env files (copy-paste from docs/dev/03-configuration.md)
#    api/.env and app/.env

# 7. Run database migrations
cd api
DATABASE_URL="postgresql://server:dev_password@localhost:5432/backend_db" \
  sqlx migrate run
cd ..

# 8. Start backend API (terminal 1)
cd api && cargo run

# 9. Start frontend (terminal 2)
cd app && npm run dev

# 10. Start worker (terminal 3)
cd worker && cargo run

# 11. Verify everything works
#     API: http://localhost:3000/health
#     Frontend: http://localhost:5173
#     Docs: http://localhost:5173/docs (via VitePress)

# 12. Run tests
cd api && cargo test
cd app && npm test       # (if configured)
cd worker && cargo test
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Full Docker Compose (all services) | Hybrid: Docker infra + local dev tools | Phase 61 discussion (D-01) | Faster edit-run cycle for contributors |
| Single-file docs | Multi-file under docs/dev/ | Phase 61 discussion (D-13) | Maintainable, findable documentation |
| .env.example reference only | Inline copy-paste values | Phase 61 discussion (D-05) | Developer doesn't need to interpret .env.example |

---

## Assumptions Log

No claims tagged `[ASSUMED]` in this research. All findings were verified against the codebase, CONTEXT.md decisions, and existing project files.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | None — all claims verified against codebase and CONTEXT.md decisions | — | — |

---

## Open Questions

1. **[Frontend tests]**
   - What we know: `app/package.json` has no `test` script. The codebase has no test files for the frontend.
   - What's unclear: Should DEVELOPMENT.md include `npm test` in the frontend command section (D-11 says include test commands)? There is no test framework configured.
   - Recommendation: Document `npm test` if it exists after checking, or note "Frontend tests not yet configured" in the commands section. This is within the agent's discretion.

2. **[Landing page setup]**
   - What we know: `landing-page-escluse/` is a separate sub-repo with its own Vite config.
   - What's unclear: Should the DEVELOPMENT.md cover landing page setup or is it out of scope? The phase boundary says "how to run all services (API, Worker, Web Agent, Frontend)" — landing page is separate.
   - Recommendation: Mention landing page briefly in the repo overview table, but don't include its setup in the primary workflow unless specifically needed.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Docker | Infrastructure (PostgreSQL, Redis) | ✓ | 29.3.0 | — |
| Docker Compose | Container orchestration | ✓ | v5.1.1 | — |
| Node.js | Frontend (Vite, npm) | ✓ | v22.22.2 | — |
| npm | Frontend dependencies | ✓ | 10.9.7 | — |
| Rust (rustc) | API, Worker, Agent build | ✓ | 1.95.0 | — |
| Cargo | Rust package manager | ✓ | 1.95.0 | — |
| rustup | Rust toolchain manager | ✓ | 1.29.0 | — |
| Supabase CLI | Local Supabase Auth | **✗** | — | Must install via `npm install -g supabase` or `brew install supabase` |

**Missing dependencies with no fallback:**
- None — Docker available for PostgreSQL/Redis, Rust/Node toolchains available for dev

**Missing dependencies with fallback:**
- **Supabase CLI** — Not installed. Prerequisites table MUST include install commands for each OS. Without it, local Supabase auth won't work. The app can still run backend/frontend for non-auth development if the developer uses a cloud Supabase project instead.

---

## Validation Architecture

> **SKIPPED** — `workflow.nyquist_validation` is explicitly set to `false` in `.planning/config.json`. No test infrastructure or validation coverage is required for this phase.

---

## Security Domain

> **SKIPPED** — This is a pure documentation phase. No runtime code, no authentication/authorization, no data handling. The DEVELOPMENT.md content warns developers not to commit real credentials (a standard security practice for setup docs), but no security controls or ASVS categories apply.

---

## Sources

### Primary (HIGH confidence) — Verified from codebase
- `docker-compose.yml` — Full stack definition, port mappings, credentials
- `api/Cargo.toml` — Rust dependencies, Axum 0.7, sqlx 0.7
- `app/package.json` — React 19.2.4, Vite 7.3.1, zustand 5
- `app/vite.config.js` — Dev server port 5173, proxy :3000 for API
- `worker/.env.example` — Worker env reference (19 lines)
- `PUSH_COMMIT.md` — Sub-repo mapping table (9 repos, GitHub URLs)
- `DEPLOY.md` — Sub-repo URL mapping (5 main repos)
- `docs/.vitepress/config.js` — VitePress configuration
- `.planning/codebase/STACK.md` — Tech stack versions
- `.planning/codebase/STRUCTURE.md` — Directory layout
- `.planning/codebase/ARCHITECTURE.md` — Service architecture
- `.planning/codebase/INTEGRATIONS.md` — External service connectivity
- `.planning/codebase/TESTING.md` — Test infrastructure
- `.planning/codebase/CONCERNS.md` — Known issues
- `.gitignore` — Confirms sub-repos listed as "tracked separately"
- `61-CONTEXT.md` — All 13 user decisions
- `61-DISCUSSION-LOG.md` — Alternatives considered

### Secondary (MEDIUM confidence)
- Verified sub-repo structure: `api`, `app`, `landing-page-escluse`, `docs`, `agent/agent-core`, `agent/solys`, `packages`, `migration` each have their own `.git` directory with independent remotes
- No `.gitmodules` file — confirms these are NOT standard git submodules
- Confirmed: `gateway/` and `docker-compose.yml` are part of parent repo (no .git)

### Tertiary (LOW confidence)
- None — all findings verified against codebase inspection

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Verified against codebase files, package.json, Cargo.toml, docker-compose.yml
- Architecture: HIGH — Verified directory structure, sub-repo analysis, git checks, docs config
- Pitfalls: HIGH — Based on discovered sub-repo structure, missing Supabase CLI, and common Docker/Rust pitfalls

**Research date:** 2026-05-31
**Valid until:** 2026-07-01 (stable documentation phase — file structure format unlikely to change frequently)
