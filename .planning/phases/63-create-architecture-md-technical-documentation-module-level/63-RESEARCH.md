# Phase 63: Create ARCHITECTURE.md — Research

**Date:** 2026-05-31
**Status:** Research complete

## Verified Architecture Details (from live codebase)

### API Backend (`api/src/`)
- **Repository traits confirmed:** 10+ trait files in `api/src/domain/repositories/` (server, node, backup, billing, etc.)
- **Use cases confirmed:** 18 use case files in `api/src/application/use_cases/` (create/start/stop/delete server, send_command, port_allocation, etc.)
- **Architecture:** Clean Architecture with AppContainer DI — domain/application/infrastructure/presentation layers verified
- **Dependencies:** Axum v0.7, sqlx v0.7, tokio v1, redis v0.25, jsonwebtoken v9, ssh2 v0.9.5, rcon v0.6

### Worker (`worker/src/`)
- Entry point at `worker/src/main.rs`
- Contains: `config.rs`, `queue.rs`, `agent/`, `webhook/`

### Web Agent (`agent/solys/`)
- Entry point at `agent/solys/src/main.rs`
- Contains: `agent_connection.rs`, `handlers/` (runtime, backup, rcon, metrics, ssh, sftp), `api/` (internal HTTP), `task_state.rs`

### Agent-Core (`agent/agent-core/crates/`)
- **12 crates confirmed:** agent-proto, agent-config, agent-runtime, agent-ssh, agent-backup, agent-rcon, agent-health, agent-metrics, agent-task, agent-security, agent-event, agent-capability

### Frontend (`app/src/`)
- Entry point: `app/src/main.jsx`
- Structure: `pages/`, `components/`, `hooks/`, `store/` (Zustand), `lib/`, `api/`, `context/`, `types/`, `features/`

### Directory Layout Note
- Agent-core is at `agent/agent-core/crates/` (not repo root `agent-core/` as internal doc suggests)
- `migrations/` is a symlink: `migrations -> api/migrations`

## Mermaid Diagram Strategy

### Suitable Diagram Types per Flow
- **System Context Overview:** `graph TD` (block diagram with directional arrows) — shows services as boxes, arrows for data flow
- **Server Creation Flow:** `sequenceDiagram` — multi-step async flow across services
- **Agent Registration Flow:** `sequenceDiagram` — WebSocket handshake + message exchange
- **Background Job Flow:** `sequenceDiagram` — queue-based async job flow
- **WebSocket Lifecycle:** `stateDiagram-v2` or `sequenceDiagram` — connection states and transitions
- **Key Abstractions:** `classDiagram` — trait/struct relationships for Repository, ExecutorFactory, TaskDispatcher

### Mermaid Constraints
- GitHub-flavored Markdown renders ` ```mermaid ` natively
- Sequence diagrams support `participant`, `->>`, `-->>`, `Note over`, `loop`, `alt`, `opt`, `par`
- Block diagrams support `subgraph`, `node[text]`, click handlers (optional)
- State diagrams support `[*]` for initial/terminal states
- Class diagrams support `<<interface>>`, `+` (public), `#` (protected), `-` (private)

## Existing Doc Format Patterns

### DEVELOPMENT.md (Phase 61) — 103 lines
- ATX heading: `# Escluse — Local Development`
- Shields.io badge row (5 badges)
- GFM pipe tables with alignment (`| --- | --- | --- |`)
- Fenced code blocks (`bash`, no language tags for some)
- `> **Note:**` and `> **Warning:**` callouts
- Bullet list doc index: `- **[link](path)** — description`
- ASCII tree with `├──` / `└──` branches, `#` inline comments

### CONTRIBUTING.md (Phase 62) — 155 lines
- Same heading pattern: `# Escluse — Contributing Guide`
- Shields.io badge row (3 badges)
- `## Table of Contents` with anchor links
- Horizontal rule `---` after badges
- GFM tables with pipe syntax
- `> **Warning:**` callout
- Per-task code blocks

## Key Differences: Internal vs Public ARCHITECTURE.md

| Aspect | Internal (.planning/codebase/) | Public (repo root) |
|--------|-------------------------------|-------------------|
| Audience | Maintainers, planning team | External developers, contributors |
| Diagrams | ASCII text descriptions | Mermaid.js inline fenced code blocks |
| Module depth | Brief descriptions per layer | Expanded with key files, entry points, structs |
| Tech stack | No version info | Inline version reference or STACK.md link |
| Cross-references | None to other docs | Links to DEVELOPMENT.md, CONTRIBUTING.md |
| Meta-repo context | Assumes monorepo-view | Explains meta-repo, references repo mapping |
| Planning references | Analysis date, internal notes | Clean — no internal planning artifacts |
| Format | Plain markdown | Badge row, TOC, admonitions, consistent with other docs |

## Research Findings Summary
1. Internal architecture doc is accurate but needs adaptation for public audience
2. Mermaid.js diagrams are GitHub-native — no build step required
3. Single-file format at repo root follows Phase 62 pattern
4. Agent-core location is `agent/agent-core/crates/` (12 crates confirmed)
5. API uses Clean Architecture with 18+ use cases and 10+ repository traits
6. Existing docs use consistent ATX heading + badge + table + code block conventions
7. Phase 63 PLAN.md should follow Phase 62's structure (objective, context, tasks, verification, threat model, success criteria)
