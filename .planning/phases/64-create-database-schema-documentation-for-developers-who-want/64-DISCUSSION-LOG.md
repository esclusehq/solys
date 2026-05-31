# Phase 64: Create database schema documentation (for developers who want to extend) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-31
**Phase:** 64-create-database-schema-documentation-for-developers-who-want
**Areas discussed:** File location, Diagram format, Content organization, Generation approach, Depth

---

## File Location

| Option | Description | Selected |
|--------|-------------|----------|
| Repo-root single file (DATABASE_SCHEMA.md) | Single file at repo root matching DEVELOPMENT.md/CONTRIBUTING.md pattern | ✓ |
| VitePress docs site (docs/database/) | Integrated into existing docs/ VitePress site at docs.esluce.com | |
| Repo-root entry + docs/database/ sub-files | Hybrid: entry point at root + detailed sub-files in docs/ | |

**User's choice:** Repo-root single file (DATABASE_SCHEMA.md)
**Notes:** Also declined docs site cross-reference — strictly repo-root only.

---

## Diagram Format

| Option | Description | Selected |
|--------|-------------|----------|
| Markdown tables only | GFM tables per table, no diagrams | |
| Mermaid ER diagrams + tables | ER diagrams for relationships + markdown tables for columns | ✓ |
| Markdown tables + ASCII relationship maps | Simpler than Mermaid but still conveys connections | |

**User's choice:** Mermaid ER diagrams + tables
**Notes:** ER diagrams per relationship cluster (not one big diagram, not per-domain rigidly).

---

## Content Organization

| Option | Description | Selected |
|--------|-------------|----------|
| By business domain | Servers, Nodes, Billing, Users, Backups, Settings, Events, Jobs | ✓ |
| Alphabetical table list | Single flat list sorted alphabetically | |
| By schema namespace | Group by database schema (single public schema) | |

**User's choice:** By business domain
**Notes:** Each domain section includes tables + context + query patterns.

---

## Generation Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Hand-written .md | Manual writing following repo-root doc conventions | |
| Auto-generated from migrations | Generator reads SQL migration files and produces markdown | |
| Hand-written initial + maintenance note | Manual first version with maintenance expectations | |
| DB introspection tool + code annotations | Rust CLI tool connecting to DB + reading rustdoc annotations | ✓ |

**User's choice:** DB introspection tool + code annotations
**Notes:** Generator tool is in scope for this phase. The agent has discretion over tool name, location, and dependencies.

---

## Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Current schema snapshot | Document what exists now | ✓ |
| Snapshot + change history | Include migration history per table | |
| Snapshot + practical annotations | Constraints, indexes, best practice notes | |

**User's choice:** Current schema snapshot
**Notes:** Migration history available in api/migrations/ for those who need it.

---

## The Agent's Discretion

- Exact Mermaid ER diagram type (`erDiagram` syntax)
- Relationship cluster boundaries (what connects to what)
- Query patterns to include per domain section
- Rust crate dependencies for the generator tool
- Markdown formatting and heading levels
- Whether to include an index/table of contents
- Whether to include a "Schema Version" or "Last Generated" timestamp

## Deferred Ideas

- **Generator tool docs site integration** — cross-linking from docs.esluce.com was considered but declined.
- **Full schema migration history inline** — current snapshot only was preferred.
