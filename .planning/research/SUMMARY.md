# Project Research Summary

**Project:** Esluce — Game Server Hosting Platform
**Domain:** Game Server Hosting / Infrastructure-as-a-Service
**Researched:** 2026-04-09
**Confidence:** HIGH

## Executive Summary

Escluse is a game server hosting platform enabling users to deploy, manage, and monitor game servers (Minecraft, Palworld, Valheim, etc.) on cloud infrastructure. Research across industry leaders (Pterodactyl, AMP, Nodecraft) confirms the existing Rust + React stack is well-aligned with 2026 best practices — Rust provides superior performance characteristics for sustained game server connections compared to PHP/Go alternatives, and the codebase already implements the Hub-and-Spoke architecture pattern proven in production platforms.

The roadmap should prioritize in this order: **immediate security fixes** (hardcoded secrets, webhook verification), then **infrastructure foundation** (database, Redis, WebSocket), followed by **core server operations** (deployment, lifecycle, container management), and finally **operations automation** (backups, monitoring, integrations). This sequencing avoids critical pitfalls identified in the existing codebase — especially container memory limits that cause OOM kills, WebSocket race conditions, and missing backup infrastructure — while delivering table-stakes features that make the product viable.

Key risks center on operational reliability: game server data persistence requires careful volume configuration, memory limits must account for JVM + OS overhead, and container lifecycle management needs robust state machines to prevent race conditions. These are addressable with proper architectural patterns documented in this research.

## Key Findings

### Recommended Stack

The existing stack is industry-aligned. Minor version upgrades are recommended but not critical blockers.

**Core technologies:**
- **Rust 1.80+** (backend): Current minimum should be 1.80 for improved async performance. Provides memory safety without GC overhead — critical for sustained game server connections.
- **Axum 0.8.x** (API framework): Upgrade from 0.7.x. Best Rust web framework in 2026 due to Tower ecosystem integration.
- **PostgreSQL 16.x** + **Redis 7.x** (persistence): Standard for game server metadata, user data, and async queue — unchanged.
- **bollard 0.20.x** (containers): Pure Rust Docker API. Upgrade from v0.18. Essential for game server container lifecycle.
- **React 19.x** + **Vite 6+** (frontend): Modern stack, no changes needed.
- **Argon2** (passwords): Migrate from bcrypt. Better GPU resistance than bcrypt. Use for new passwords, maintain bcrypt for legacy.

**Confidence:** HIGH — stack choices are industry standard, validated against Pterodactyl, AMP, and Hathora patterns.

### Expected Features

**Must have (table stakes):**
- **Server Deployment** — Core value proposition. Requires game definitions, Docker images, port allocation, resource provisioning.
- **Start/Stop/Restart** — Basic lifecycle control.
- **Server Status** — Node agent polling for online/offline/starting states.
- **RCON Console** — Standard admin interface (Source RCON, Minecraft RCON).
- **File Manager** — SFTP access, browser-based config editing.
- **Resource Allocation** — CPU, RAM, storage limits per container.

**Should have (differentiators):**
- **Automated Backups** — Scheduled snapshots with retention policies. High value, moderate complexity.
- **Real-Time Console** — WebSocket streaming, ANSI color support.
- **Crash Recovery** — Auto-restart on crash without user intervention.
- **Discord/Telegram Integration** — Webhook notifications for start/stop/crash events.
- **Resource Graphs** — Historical CPU, RAM, network usage.

**Defer (v2+):**
- Plugin marketplace, white-label/reseller, mobile app, multi-cloud support.

### Architecture Approach

The architecture follows the **Hub-and-Spoke Model** — API backend as the hub coordinating spoke agents on each compute node. This pattern is proven in managed game hosting (Hathora, PlayFab) and handles the unique constraints of game server workloads: long-running processes, per-instance resource isolation, and low-latency control plane communication.

**Major components:**

1. **API Backend (Rust/Axum)** — Business logic, REST endpoints, orchestration state machine. Handles user authentication, server provisioning, and acts as the control plane.
2. **Web Agent (Per Node)** — Lightweight agent handling container lifecycle, task execution, metrics collection. Communicates via WebSocket to API backend.
3. **Background Worker** — Async job processing (backup scheduling, webhook delivery, scheduled tasks). Reads from Redis queue, emits to external services.
4. **Frontend (React SPA)** — User interface for server management, real-time console, metrics visualization.

**Data flow:** Frontend → REST API → WebSocket → Node Agent → Docker/Podman → Game Container. Each component has clear boundaries documented in ARCHITECTURE.md.

**Confidence:** HIGH — architecture matches existing codebase and industry patterns.

### Critical Pitfalls

1. **Unverified Webhook Payloads (Lemon Squeezy)** — Existing code ignores Ed25519 signature verification. Attackers can forge subscription events. **Immediate fix required before any billing features.**

2. **Container Memory Limits Too Low** — Game servers (especially Java/Minecraft) exceed allocated memory, get OOM-killed, causing world corruption. Calculate as: game RAM + 1GB overhead minimum.

3. **Hardcoded Secrets** — Database passwords in docker-compose.yml, JWT secrets in .env.example. **Immediate fix required — remove all secrets before production.**

4. **WebSocket Race Conditions** — Node connection manager state becomes inconsistent on rapid reconnect. Requires proper state machine with atomic operations before depending on reliable node state.

**Moderate pitfalls include:** synchronous SSH blocking async runtime, extensive `.unwrap()` causing panics, single instance assumptions, and missing backup strategy. Full details in PITFALLS.md.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Security Foundation
**Rationale:** Existing codebase has critical security issues (hardcoded secrets, unverified webhooks) that prevent production deployment. Fix these before any feature work.
**Delivers:** Clean configuration management, webhook signature verification, proper error handling replacing panics.
**Must address:** Pitfall 1 (webhook security) and Pitfall 3 (hardcoded secrets) from CONCERNS.md mapping.
**Uses:** No new stack elements — infrastructure configuration changes.

### Phase 2: Infrastructure Foundation
**Rationale:** All other components depend on persistence and messaging. Establish core data layer before any API work.
**Delivers:** PostgreSQL schema, Redis configuration, repository implementations, WebSocket infrastructure.
**Dependencies:** Phase 1 complete.
**Avoids:** Pitfall 7 (single instance assumptions) — use Redis for session storage from start.

### Phase 3: Core API
**Rationale:** Frontend needs API to connect to. Establish user-facing endpoints and basic server CRUD.
**Delivers:** Domain layer, REST endpoints, basic authentication, server listing/details.
**Dependencies:** Phase 2 complete.
**Features:** Server Status, Server Listing (table stakes).

### Phase 4: Node Agent & Communication
**Rationale:** Enables container operations — the core value of the platform. Agents depend on API existing.
**Delivers:** WebSocket client, task handlers, runtime manager (Docker/Podman), agent registration.
**Dependencies:** Phase 3 complete.
**Avoids:** Pitfall 4 (WebSocket race conditions) — implement proper state machine.

### Phase 5: Server Deployment
**Rationale:** Core value proposition. Users must be able to create game servers.
**Delivers:** Game egg definitions, Docker image management, port allocation, resource provisioning.
**Dependencies:** Phase 4 complete.
**Avoids:** Pitfall 2 (memory limits) — configure with overhead; Pitfall 8 (data loss) — use named Docker volumes.
**Features:** Server Deployment, Start/Stop/Restart, Resource Allocation, Network Configuration (table stakes).

### Phase 6: Operations Integration
**Rationale:** Complete server management with file access and console.
**Delivers:** RCON handler, SFTP handler, status reporting, metrics collection.
**Dependencies:** Phase 5 complete.
**Features:** RCON Console, File Manager, Server Status enhanced (table stakes).

### Phase 7: Automation & Backups
**Rationale:** Differentiators that reduce user friction and protect data.
**Delivers:** Backup handler, crash recovery, background worker for scheduled tasks.
**Dependencies:** Phase 6 complete.
**Avoids:** Pitfall 9 (missing backup strategy) — implement automated backups.
**Features:** Automated Backups, Crash Recovery (differentiators).

### Phase 8: Monitoring & Integrations
**Rationale:** Operational visibility and community integrations.
**Delivers:** Resource graphs, Discord/Telegram webhooks, metrics dashboard.
**Dependencies:** Phase 7 complete.
**Features:** Resource Graphs, Discord Integration, Scheduled Tasks (differentiators).

### Phase 9: Frontend Polish
**Rationale:** Complete user experience after all backend capabilities exist.
**Delivers:** Real-time console, file browser, metrics visualization.
**Dependencies:** Phases 6-8 complete.

### Phase Ordering Rationale

- **Security first:** Hardcoded secrets and webhook verification block production — fix immediately.
- **Infrastructure before API:** All components depend on PostgreSQL and Redis.
- **Agent after API:** Node agents require API endpoints to communicate with.
- **Operations after agents:** RCON, SFTP, and file management require agent handlers.
- **Automation last:** Backups and scheduled tasks build on operational infrastructure.
- **Frontend last:** Full UI requires backend capabilities to exist.

This ordering maps to Architecture.md build order with security fixes prioritized from Pitfalls.md.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 5 (Server Deployment):** Game-specific configurations (Minecraft eggs, Palworld, Valheim). May need per-game research or game egg registry.
- **Phase 8 (Integrations):** Discord/Telegram API integration patterns, webhook design for notification events.

Phases with standard patterns (skip research-phase):

- **Phase 2 (Infrastructure):** PostgreSQL + Redis is standard — well-documented.
- **Phase 3 (Core API):** REST API patterns well-established with Axum.
- **Phase 4 (Node Agent):** WebSocket + Docker patterns from existing codebase.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Industry standard, validated against Pterodactyl/AMP |
| Features | HIGH | Table stakes from multiple established platforms |
| Architecture | HIGH | Matches existing codebase + industry patterns |
| Pitfalls | HIGH | Directly from existing codebase analysis |

**Overall confidence:** HIGH

### Gaps to Address

- **Game egg definitions:** Research assumes game eggs exist. Phase planning should validate game coverage.
- **Multi-region considerations:** Not covered — needs research if geo-distribution required.
- **Auto-scaling patterns:** Needs deeper research for scale-out strategy beyond Phase 9.
- **DDoS protection:** Partnership required (often third-party) — not in scope for initial phases.

## Sources

### Primary (HIGH confidence)
- Pterodactyl Panel v1.12.0 (mintlify.com/pterodactyl) — Industry reference, 200+ game support
- Axum v0.8.9 Release (github.com/tokio-rs/axum) — Framework verification
- Bollard v0.20.2 Release (github.com/fussybeaver/bollard) — Container library verification
- Existing codebase architecture analysis (2026-04-08)

### Secondary (MEDIUM confidence)
- Nodecraft Automation Features — Differentiator patterns
- AMP Game Server Control Panel (cubecoders.com) — Commercial reference
- Self-Hosted Game Server Panels 2026 (selfhosting.sh) — Industry landscape
- Hathora scalable WebSocket architecture — Pattern reference

### Tertiary (LOW confidence)
- Rust Web Frameworks 2026 comparison — Framework choices may evolve

---

*Research completed: 2026-04-09*
*Ready for roadmap: yes*