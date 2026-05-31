# Phase 63: Create ARCHITECTURE.md - technical documentation (module-level) - Context

**Gathered:** 2026-05-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver an `ARCHITECTURE.md` at repo root documenting the module-level architecture of the Esluce platform — service boundaries, data flow, component relationships, and key abstractions. A developer-facing reference for those building on or extending the platform.

The result is a technical architecture reference at repo root, complementing `DEVELOPMENT.md` (local setup) and `CONTRIBUTING.md` (contribution workflow). It covers how services connect, data flows between them, and the module-level structure of each service.

</domain>

<decisions>
## Implementation Decisions

### Diagrams — Format and Scope
- **D-01:** Use **Mermaid.js** inline fenced code blocks (````mermaid`) for all architecture diagrams. GitHub renders these natively — no build step or external tool needed.
- **D-02:** Include **all major data flows** as Mermaid sequence/flow diagrams: system context overview, server creation flow, agent registration flow, background job (backup) flow, WebSocket lifecycle, and any other significant cross-service interactions.
- **D-03:** Embed diagrams as inline ` ```mermaid ` code blocks directly in `ARCHITECTURE.md`. Do not pre-render to images.

### Structure (Agent's Discretion)
- **D-04:** Single `ARCHITECTURE.md` at repo root (following the `CONTRIBUTING.md` pattern from Phase 62), not a multi-file split under `docs/architecture/`. The phase title specifies a single file. If the document grows long, consider a table of contents with anchor links to sections.

### Depth (Agent's Discretion)
- **D-05:** Detailed reference — include module descriptions, entry points, key files, and code structure for each service (API, Worker, Web Agent, Frontend, Agent-Core). Match the depth level of the existing `.planning/codebase/ARCHITECTURE.md` but adapted for public consumption.

### Source Material (Agent's Discretion)
- **D-06:** Build from `.planning/codebase/ARCHITECTURE.md` as primary source material. Adapt for public audience — remove internal planning references, annotate with Mermaid diagrams, and expand module-level coverage with the meta-repo mapping from Phase 62.

### The Agent's Discretion
- Exact section ordering and headings
- Which specific Mermaid diagram types to use per flow (sequence for flows, block diagram for system overview, classDiagram for key abstractions)
- Module-level depth per service (some may get more detail based on complexity)
- How to reference the meta-repo mapping from CONTRIBUTING.md (link vs inline table)
- Whether to include a tech stack version table or reference `.planning/codebase/STACK.md`
- Whether to include an "Architecture Decisions" section linking to CONTEXT.md decisions from prior phases
- Markdown formatting and styling details

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary Source Material
- `.planning/codebase/ARCHITECTURE.md` — Internal architecture doc: service layers, data flows, key abstractions, entry points. Primary source material for public ARCHITECTURE.md.
- `.planning/codebase/STRUCTURE.md` — Directory layout and module organization for all services.
- `.planning/codebase/CONVENTIONS.md` — Coding conventions (naming, imports, style) across all services.
- `.planning/codebase/STACK.md` — Tech stack with exact versions (Rust edition 2021, React 19, Axum 0.7, etc.).
- `.planning/codebase/INTEGRATIONS.md` — External integrations (PostgreSQL, Redis, Discord, Stripe, etc.).
- `.planning/codebase/TESTING.md` — Test framework details and commands.

### Prior Phase Patterns (Doc Structure)
- `.planning/ROADMAP.md` § Phase 63 — Phase goal: "Create ARCHITECTURE.md - technical documentation (module-level)"
- `.planning/phases/61-create-development-md-setup-local-dev-environment/61-CONTEXT.md` — D-13: root entry + sub-files pattern (for reference — Phase 63 uses single file per D-04)
- `.planning/phases/62-create-contributing-md-cara-kontribusi/62-CONTEXT.md` (if exists) — CONTRIBUTING.md format, meta-repo mapping, companion files pattern

### Meta-Repo Architecture
- `.planning/phases/62-create-contributing-md-cara-kontribusi/62-RESEARCH.md` — Meta-repo structure, repo mapping table, contribution model (the ARCHITECTURE.md should reference or link to this)

### Phase Goal
- `.planning/ROADMAP.md` § Phase 63 — "Create ARCHITECTURE.md - technical documentation (module-level)"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.planning/codebase/ARCHITECTURE.md` — 207-line internal architecture doc with service descriptions, data flow narrative, key abstractions, entry points. Direct source material.
- `.planning/codebase/STRUCTURE.md` — 229-line directory layout with directory purposes and key file locations per service.
- `.planning/codebase/CONVENTIONS.md` — 246-line coding conventions doc with import organization patterns and module design descriptions.
- `DEVELOPMENT.md` — Existing repo-root doc (Phase 61). Reference pattern for formatting, tone, badges, and structure.
- `CONTRIBUTING.md` — Existing repo-root doc (Phase 62). Reference pattern for single-file architecture docs with sections and cross-references.

### Established Patterns
- Repo-root entry point docs follow a consistent format: ATX headings, Shields.io badges, GFM tables, fenced code blocks, callout admonitions (DEVELOPMENT.md, CONTRIBUTING.md pattern).
- `.planning/codebase/` maps use detailed markdown with ASCII diagrams, section headers, and code examples.
- Meta-repo architecture documentation pattern established in Phase 62 (repo mapping table, per-repo descriptions).

### Integration Points
- `ARCHITECTURE.md` links to `DEVELOPMENT.md` for local setup reference.
- `ARCHITECTURE.md` links to `CONTRIBUTING.md` for the meta-repo contribution model.
- `.planning/codebase/ARCHITECTURE.md` serves as source — the public ARCHITECTURE.md should be a superset adapted for external developers.
- The existing docs site at `docs/` (VitePress) could cross-reference sections from ARCHITECTURE.md.

</code_context>

<specifics>
## Specific Ideas

ARCHITECTURE.md at repo root with Mermaid.js diagrams covering all major data flows. Build from the existing internal architecture doc. Single file with detailed module-level coverage.

Section ideas (agent discretion to finalize):
1. System Overview — high-level architecture diagram + description
2. Service Architecture — per-service breakdown (API, Worker, Web Agent, Frontend, Agent-Core)
3. Data Flows — server creation, agent registration, background jobs, WebSocket lifecycle (Mermaid sequence diagrams)
4. Key Abstractions — repository pattern, executor factory, task dispatch
5. Technology Stack — reference to STACK.md or inline version table
6. Module Reference — per-service module structure with key files

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 63-create-architecture-md-technical-documentation-module-level*
*Context gathered: 2026-05-31*
