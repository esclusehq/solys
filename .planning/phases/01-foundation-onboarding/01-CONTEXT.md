# Phase 1: Foundation & Onboarding - Context

**Gathered:** 2026-06-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Project scaffolding, bot client foundation, and all onboarding functionality — welcome system, interest selection, auto-role assignment, role-based permission system, and full server settings configuration. This phase establishes the entire technical infrastructure (TypeScript/Bun bot, Rust/Axum API, PostgreSQL, Redis) and delivers the member onboarding flow.

</domain>

<decisions>
## Implementation Decisions

### Welcome Format
- **D-01:** Multi-message sequence — bot sends a sequence of messages (teks sambutan, embed info, follow-up) rather than a single embed
- **D-02:** Messages sent to a designated welcome channel when a member joins (not via DM)
- **D-03:** Full content — welcome greeting, important links, invitation to select interests, server rules, server info
- **D-04:** Bilingual — language configurable per server (Indonesia or English)

### Interest Selection UX
- **D-05:** Select menu dropdown (not buttons) — compact, scrollable list of interests
- **D-06:** Multiple selection — user can pick more than one interest
- **D-07:** Triggered by a button or command (not auto-displayed with welcome) — a "Pilih Interest" button or `/interest` command
- **D-08:** Interest list configurable by admin via Discord command (not hardcoded)

### Permission Model
- **D-09:** Per-command granularity — each command can be individually permitted/denied per role
- **D-10:** Configuration via both Discord command and admin web panel (Discord command first, web panel later)
- **D-11:** Restrictive by default — new commands are admin-only until explicitly granted to other roles
- **D-12:** Custom database storage (not Discord native permissions) — full flexibility, survives server reinstall

### Server Settings
- **D-13:** Separate tables per category (not a single JSON column) — normalized schema
- **D-14:** Configuration via Discord command first (web panel deferred)
- **D-15:** Full server management scope: welcome channel, welcome message, interest list, role mappings, log channel, bot language, channel management, role management, and all server configuration
- **D-16:** Automatic validation with error embed — checks channel/role existence on input

### Auto-Role Timing
- **D-17:** Dual system — default role assigned on join, interest-specific roles assigned after interest selection
- **D-18:** No grace period — default role is permanent even if user never selects interests (can select anytime)
- **D-19:** Append only — bot never removes manually-assigned roles; only adds new ones
- **D-20:** Logging to designated log channel whenever roles are assigned

### Project Structure
- **D-21:** Workspace monorepo layout within `escluse-bot/`
- **D-22:** Turborepo for build orchestration
- **D-23:** Each service manages its own database migrations (bot: Drizzle ORM, api: sqlx)
- **D-24:** Each service has its own `.env` file

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/PROJECT.md` — Project vision, core value, constraints (tech stack: TypeScript/Bun/Discord.js, Rust/Axum, PostgreSQL/Drizzle, Redis)
- `.planning/REQUIREMENTS.md` — v1 requirements: ONB-01 (welcome), ONB-02 (interest selection), ONB-03 (auto-role), PERM-01 (permissions), ADMN-03 (server settings)
- `.planning/ROADMAP.md` §Phase 1 — Phase goal, success criteria, dependency info

### Research
- `.planning/research/SUMMARY.md` — Synthesis of all research findings
- `.planning/research/STACK.md` — Full stack recommendations with versions
- `.planning/research/ARCHITECTURE.md` — Two-process architecture, data flow, build order

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Docker Compose patterns** from parent escluse project: `postgres:16-alpine`, `redis:7-alpine`, health checks, network-based service discovery
- **Rust workspace** structure from existing escluse Cargo.toml — Cargo workspace with members/exclude patterns

### Established Patterns
- **Docker Compose** with named networks and health checks (from escluse root)
- **Environment variable** configuration via `.env` files

### Integration Points
- No integration with existing escluse backend — standalone for v1
- Docker network topology should use separate network from main escluse stack

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches from research recommendations.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 1-Foundation & Onboarding*
*Context gathered: 2026-06-18*
