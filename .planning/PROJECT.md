# Project: Esluce

**What this is:** Self-hosted game server hosting platform. Users deploy game servers (Minecraft, Palworld, etc.) on their own infrastructure (local machine/VPS) and manage them via a centralized web control panel.

## Core Value

One thing that must work: Users can connect their own nodes (local/VPS) to the platform and deploy game servers with minimal configuration, managed via a web control panel.

## Context

**Current state:** Brownfield project - existing codebase mapped with:
- Rust microservices (API, Worker, Web Agent)
- React frontend with Monaco Editor
- 12 shared Rust crates (agent-core)
- Users connect their own nodes (VPS/local machines) via Web Agent

**Key constraints:**
- Users provide their own infrastructure (nodes)
- PostgreSQL for persistence
- Redis for caching/queue
- WebSocket for node communication
- Docker/Podman for containerization

## Requirements

### Validated

(None yet - to be determined)

### Active

- [ ] User can connect their own node (VPS/local machine) to the platform
- [ ] User can deploy game server to their connected node
- [ ] User can start/stop/restart game server
- [ ] User can view server status and metrics
- [ ] User can access server via RCON
- [ ] User can manage server files (SFTP)

### Out of Scope

- [Providing infrastructure] — Users provide their own VPS/nodes
- [Multiple cloud providers] — Not applicable (self-hosted model)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Architecture | Microservices with node agents | Established in existing codebase |

---

*Last updated: 2026-04-09 after initialization*

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state