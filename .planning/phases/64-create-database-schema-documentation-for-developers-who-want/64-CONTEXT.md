# Phase 64: Create database schema documentation (for developers who want to extend) - Context

**Gathered:** 2026-05-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a `DATABASE_SCHEMA.md` at repo root documenting the PostgreSQL schema — tables, columns, relationships, constraints, indexes, and key query patterns — so developers extending the platform understand how data is structured without reading 69 migration files.

Includes a Rust CLI generator tool (`tools/db-schema-gen/`) that introspects the live database via `information_schema` and reads rustdoc annotations from Rust entity structs to produce the markdown document.

The result is a developer-facing reference at repo root, complementing `DEVELOPMENT.md` (local setup), `CONTRIBUTING.md` (contribution workflow), and `ARCHITECTURE.md` (module-level architecture). It covers all database tables organized by business domain.

</domain>

<decisions>
## Implementation Decisions

### File Location
- **D-01:** Single `DATABASE_SCHEMA.md` at repo root. No docs site integration — repo-root only.

### Diagram Format
- **D-02:** **Mermaid ER diagrams** per relationship cluster + **markdown tables** per table (column name, type, constraints, description).
- **D-03:** ER diagrams grouped by natural FK relationship clusters (the agent identifies the clusters from the schema), not one big diagram nor per-domain rigidly.

### Content Organization
- **D-04:** Tables grouped **by business domain** — Servers, Nodes, Billing/Subscriptions, Users/Auth, Backups, Settings/Config, Events/Logs, Jobs, and any others the agent identifies.
- **D-05:** Each domain section includes: brief domain description, ER diagram for relationship cluster, then markdown tables per table.
- **D-06:** Each domain section includes **common query patterns** and **design rationale** — why the schema is structured as it is.

### Depth
- **D-07:** **Current schema snapshot only.** Document what exists now. Migration history is available in `api/migrations/` for those who need it.

### Generation Approach
- **D-08:** **Rust CLI tool** (`tools/db-schema-gen/`) that:
  - Connects to PostgreSQL and introspects the live schema via `information_schema`
  - Reads rustdoc annotations from Rust entity structs in `api/src/domain/` for narrative descriptions and query patterns
  - Generates `DATABASE_SCHEMA.md` with Mermaid ER diagrams and markdown tables
- **D-09:** The generated `DATABASE_SCHEMA.md` is committed to the repo. Developers regenerate when schema changes.
- **D-10:** Tool name, location (`tools/db-schema-gen/` is presumed), and Rust dependencies are the agent's discretion.

### The Agent's Discretion
- Exact Mermaid ER diagram type (`erDiagram` syntax)
- Relationship cluster boundaries (what connects to what)
- Query patterns to include per domain section
- Rust crate dependencies for the generator tool
- Markdown formatting and heading levels
- Whether to include an index/table of contents
- Whether to include a "Schema Version" or "Last Generated" timestamp

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Schema Source of Truth
- `api/migrations/` — 69 SQL migration files. Source of truth for current schema shape and evolution history.
- `api/src/domain/` — Rust entity structs with rustdoc annotations. Generator reads these for narrative content.

### Codebase Maps (Patterns & Conventions)
- `.planning/codebase/STRUCTURE.md` — Directory layout showing where entity definitions, migrations, and domain modules live.
- `.planning/codebase/CONVENTIONS.md` — Coding conventions, documentation style for repo-root .md files.
- `.planning/codebase/STACK.md` — Tech stack versions (PostgreSQL version, SQLx version).

### Prior Phase Conventions
- `.planning/phases/63-create-architecture-md-technical-documentation-module-level/63-CONTEXT.md` — D-01 (Mermaid.js inline diagrams), single-file repo-root pattern.
- `.planning/phases/61-create-development-md-setup-local-dev-environment/61-CONTEXT.md` — Repo-root doc conventions, formatting patterns.
- `.planning/phases/62-create-contributing-md-cara-kontribusi/62-CONTEXT.md` — Meta-repo documentation patterns.

### Existing Repo-Root Docs (Format Reference)
- `DEVELOPMENT.md` — Existing doc pattern: ATX headings, GFM tables, fenced code blocks.
- `CONTRIBUTING.md` — Existing doc pattern: section-based single file with cross-references.

### Phase Goal
- `.planning/ROADMAP.md` § Phase 64 — "Create database schema documentation (for developers who want to extend)"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `api/migrations/` — 69 timestamped SQL migration files with full schema DDL. Primary data source.
- `api/src/domain/` — Rust entity structs (Server, Node, Backup, etc.) with type definitions matching the schema.
- Existing repo-root docs (`DEVELOPMENT.md`, `CONTRIBUTING.md`) — Format reference for generated output.
- Mermaid.js renderer — GitHub renders Mermaid natively, no build step needed.

### Established Patterns
- Repo-root entry point .md files follow consistent format: ATX headings, GFM tables, fenced code blocks, callout admonitions.
- Phase 63 established Mermaid.js inline diagrams (D-01) — Phase 64 extends to Mermaid ER diagrams.
- Rust tools/clis live in the workspace Cargo.toml or as workspace members (existing pattern: Cargo workspace at root).

### Integration Points
- `DATABASE_SCHEMA.md` sits alongside `DEVELOPMENT.md`, `CONTRIBUTING.md`, and planned `ARCHITECTURE.md` at repo root.
- The generator tool reads from `api/migrations/` and `api/src/domain/entities/` — needs access to the API crate's entity types.
- Generated document should be consistent with the existing repo-root doc style for a cohesive developer experience.

</code_context>

<specifics>
## Specific Ideas

DATABASE_SCHEMA.md at repo root with Mermaid ER diagrams per relationship cluster and markdown column-level tables. Generated via a Rust CLI tool that introspects the live DB and reads rustdoc annotations from entity structs. Tables organized by business domain (servers, nodes, billing, users, backups, settings, events, jobs) with query patterns and design rationale.

The generator tool is a Cargo workspace member under `tools/db-schema-gen/`. It connects to a target PostgreSQL database, queries `information_schema`, and produces the complete DATABASE_SCHEMA.md.

</specifics>

<deferred>
## Deferred Ideas

- **Generator tool docs site integration** — cross-linking DATABASE_SCHEMA.md from docs.esluce.com was considered but declined. Repo-root only.
- **Full schema migration history inline** — current snapshot only was preferred; migration history remains in `api/migrations/`.

</deferred>

---

*Phase: 64-create-database-schema-documentation-for-developers-who-want*
*Context gathered: 2026-05-31*
