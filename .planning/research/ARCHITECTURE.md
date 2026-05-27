# Research: Game Server Hosting Architecture

**Project:** Esluce — Game Server Hosting Platform  
**Researched:** 2026-04-09  
**Mode:** Ecosystem (Architecture Patterns)  
**Confidence:** HIGH

## Executive Summary

Game server hosting platforms require a distributed agent-based architecture where a central orchestration layer manages lightweight node agents deployed on compute infrastructure. The existing codebase architecture aligns with industry patterns: a central API (Rust/Axum) handles business logic, background workers process async operations, and node-resident agents handle container lifecycle via WebSocket communication.

The recommended architecture follows a **Hub-and-Spoke Model** where the API backend acts as the hub, coordinating spoke agents on each compute node. This pattern is proven in managed game hosting (e.g., Hathora, PlayFab) and handles the unique constraints of game server workloads: long-running processes, per-instance resource isolation, and low-latency control plane communication.

## Recommended Architecture

### Overall Pattern: Distributed Agent-Based with Central Orchestration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (React SPA)                           │
│                           https://app.escluse.io                           │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │ REST API + WebSocket
┌──────────────────────────────────▼──────────────────────────────────────────┐
│                              API BACKEND (Rust/Axum)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Domain     │  │ Application │  │Infrastructure│  │  Presentation     │ │
│  │  Layer      │  │  Layer      │  │   Layer     │  │  (HTTP/WS)         │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                    │
    ┌─────────▼──────────┐  ┌──────▼──────┐  ┌─────────▼──────────┐
    │  PostgreSQL       │  │   Redis     │  │  External         │
    │  (Persistence)   │  │  (Queue)    │  │  Services         │
    └───────────────────┘  └─────────────┘  │  (S3, Email,      │
                                          │   Stripe, etc.)  │
                                          └──────────────────┘
              
              ═══════════════════════════════════════════════════════════════
                              NODE LAYER (N nodes)
              ═══════════════════════════════════════════════════════════════
              
    ┌───────────────────────┐   ┌───────────────────────┐   ┌───────────────────────┐
    │      WEB AGENT 1      │   │      WEB AGENT 2      │   │      WEB AGENT N      │
    │  ┌─────────────────┐  │   │  ┌─────────────────┐  │   │  ┌─────────────────┐  │
    │  │ Task Dispatcher │  │   │  │ Task Dispatcher │  │   │  │ Task Dispatcher │  │
    │  │ Runtime Manager │  │   │  │ Runtime Manager │  │   │  │ Runtime Manager │  │
    │  │ RCON Handler    │  │   │  │ RCON Handler    │  │   │  │ RCON Handler    │  │
    │  │ SFTP Handler   │  │   │  │ SFTP Handler   │  │   │  │ SFTP Handler   │  │
    │  │ Backup Handler│  │   │  │ Backup Handler│  │   │  │ Backup Handler│  │
    │  │ Metrics       │  │   │  │ Metrics       │  │   │  │ Metrics       │  │ 
    │  └─────────────────┘  │   │  └─────────────────┘  │   │  └─────────────────┘  │
    │         │             │   │         │             │   │         │             │
    │    ┌────▼────┐       │   │    ┌────▼────┐       │   │    ┌────▼────┐       │
    │    │Podman   │       │   │    │Podman   │       │   │    │Podman   │       │
    │    │Docker  │       │   │    │Docker  │       │   │    │Docker  │       │
    │    └────────┘       │   │    └────────┘       │   │    └────────┘       │
    └───────────────────────┘   └───────────────────────┘   └───────────────────────┘
              │                         │                         │
              ▼                         ▼                         ▼
    ┌───────────────────────┐   ┌───────────────────────┐   ┌───────────────────────┐
    │  GAME CONTAINERS       │   │  GAME CONTAINERS       │   │  GAME CONTAINERS     │
    │  (Minecraft,         │   │  (Minecraft,         │   │  (Minecraft,        │
    │   Palworld, etc.)    │   │   Palworld, etc.)    │   │   Palworld, etc.)   │
    └───────────────────────┘   └───────────────────────┘   └───────────────────────┘
```

## Component Boundaries

Each component has defined interfaces and responsibilities. Communication follows the patterns below.

### Component: API Backend

| Responsibility | What It Owns | What It Can't Own |
|--------------|--------------|------------------|
| Business Logic | Domain entities, validation rules, RBAC policies | Node state, container runtime |
| Orchestration | Server lifecycle state machine | Direct container operations |
| API Contract | REST endpoints, WebSocket protocol | Agent implementation |
| Persistence | Repository interfaces | Database engine |

**Boundaries:**
- REST API consumed by: Frontend, external webhooks
- WebSocket consumed by: Node Agents
- Database accessed via: Repository traits (PostgreSQL)
- Queue accessed via: Redis client

### Component: Background Worker

| Responsibility | What It Owns | What It Can't Own |
|--------------|--------------|------------------|
| Async Jobs | Job processing logic | API authorization |
| Webhooks | External notifications | Internal state |
| Scheduled Tasks | Backup scheduling, metric collection | Real-time control |

**Boundaries:**
- Reads from: Redis queue (produced by API)
- Writes to: Database (via API or direct)
- Emits to: External webhooks

### Component: Web Agent (Per Node)

| Responsibility | What It Owns | What It Can't Own |
|--------------|--------------|------------------|
| Container Lifecycle | Docker/Podman operations | Server provisioning |
| Task Execution | Task handlers | Business rules |
| Node Health | Local health checks | Global state |
| Metrics Collection | Local resource metrics | Aggregation |

**Boundaries:**
- Communicates via: WebSocket to API Backend
- Accesses: Docker/Podman API (Bollard)
- Owns: Local container state

### Component: Frontend

| Responsibility | What It Owns | What It Can't Own |
|--------------|--------------|------------------|
| User Interface | React components, routing | Server operations |
| State Management | Client-side state | Persistent state |
| Real-time Updates | WebSocket connection | Backend logic |

**Boundaries:**
- Calls: REST API, WebSocket
- Owns: Browser state

## Data Flow

### Primary Flow: Server Deployment

```
USER                  FRONTEND              API BACKEND           NODE AGENT           CONTAINER
  │                      │                     │                     │                    │
  │ 1. Select game       │                     │                     │                    │
  │    + node           │                     │                     │                    │
  │────────────────────>│                     │                     │                    │
  │                     │ 2. POST /api/servers │                     │                    │
  │                     │─────────────────────>│                     │                    │
  │                     │                     │ 3. Validate request │                    │
  │                     │                     │    + create Server  │                    │
  │                     │                     │    entity in DB    │                    │
  │                     │                     │─────────────────────>│                    │
  │                     │                     │ 4. WebSocket: Task  │                    │
  │                     │                     │    (CreateServer)  │                    │
  │                     │                     │─────────────────────>│                    │
  │                     │                     │                     │ 5. Pull image     │
  │                     │                     │                     │───────────────────>│
  │                     │                     │                     │ 6. Create         │
  │                     │                     │                     │    container      │
  │                     │                     │                     │<──────────────────│
  │                     │                     │                     │ 7. TaskResult    │
  │                     │                     │                     │<─────────────────│
  │                     │                     │ 8. Update status  │                    │
  │                     │                     │─────────────────────>│                    │
  │                     │ 9. 201 Created      │                     │                    │
  │                     │<────────────────────│                     │                    │
  │ 10. Server ready    │                     │                     │                    │
  │<────────────────────│                     │                     │                    │
```

### Secondary Flow: Server Control (Start/Stop/Restart)

```
USER                  FRONTEND              API BACKEND           NODE AGENT           CONTAINER
  │                      │                     │                     │                    │
  │ 1. Click "Start"     │                     │                     │                    │
  │─────────────────────>│                     │                     │                    │
  │                     │ 2. POST /servers    │                     │                    │
  │                     │    /{id}/start      │                     │                    │
  │                     │─────────────────────>│                     │                    │
  │                     │                     │ 3. Update status   │                    │
  │                     │                     │    to STARTING     │                    │
  │                     │                     │─────────────────────>│                    │
  │                     │                     │ 4. WS: Task(Start) │                    │
  │                     │                     │─────────────────────>│                    │
  │                     │                     │                     │ 5. docker start  │
  │                     │                     │                     │──────────────────>│
  │                     │                     │                     │ 6. Result       │
  │                     │                     │                     │<─────────────────│
  │                     │                     │ 7. Update to RUNNING│                    │
  │                     │                     │─────────────────────>│                    │
  │                     │ 8. 200 OK           │                     │                    │
  │                     │<────────────────────│                     │                    │
  │ 9. Status updated   │                     │                     │                    │
  │<────────────────────│                     │                     │                    │
```

### Tertiary Flow: Metrics Collection

```
NODE AGENT            API BACKEND           DATABASE          FRONTEND
  │                      │                     │                  │
  │ 1. Collect local     │                     │                  │
  │    metrics (CPU,    │                     │                  │
  │    memory, etc.)   │                     │                  │
  │───────────────────>│                     │                  │
  │ 2. WebSocket:      │                     │                  │
  │    MetricsReport  │                     │                  │
  │───────────────────>│                     │                  │
  │                     │ 3. Update metrics │                  │
  │                     │    in DB         │                  │
  │                     │─────────────────>│                  │
  │                     │                     │ 4. GET /metrics  │
  │                     │<─────────────────│                  │
  │                     │ 5. Return JSON   │                  │
  │                     │<─────────────────│                  │
  │                     │                  │ 6. SSE/WebSocket │
  │                     │                  │    (optional)    │
  │                     │                  │<────────────────│
  │ 7. Real-time       │                     │                  │
  │    display       │                     │                  │
```

## Build Order Dependencies

The architecture suggests a specific build order based on dependencies. This assumes the existing codebase is the starting point.

### Phase 1: Infrastructure Foundation

**Goal:** Establish core persistence and messaging

| Step | Component | Dependencies | Rationale |
|------|-----------|--------------|-----------|
| 1.1 | PostgreSQL Schema | None | Source of truth for all state |
| 1.2 | Redis Configuration | None | Queue and caching foundation |
| 1.3 | Repository Implementations | 1.1 | Data access layer |
| 1.4 | WebSocket Infrastructure | 1.2, 1.3 | Node communication channel |

**Why first:** All other components depend on persistence. API cannot operate without data layer.

### Phase 2: Core API

**Goal:** Enable basic CRUD on servers

| Step | Component | Dependencies | Rationale |
|------|-----------|--------------|-----------|
| 2.1 | Domain Layer | 1.1 | Business entities |
| 2.2 | REST Endpoints | 2.1 | User-facing API |
| 2.3 | Basic Authentication | 2.2 | Security foundation |

**Why second:** Frontend needs API to connect to. No agents yet, but basic operations work.

### Phase 3: Node Agent

**Goal:** Enable container operations on compute nodes

| Step | Component | Dependencies | Rationale |
|------|-----------|--------------|-----------|
| 3.1 | WebSocket Client | 2.4 | Agent connectivity |
| 3.2 | Task Handlers | 2.4 | Operation implementation |
| 3.3 | Runtime Manager | None | Docker/Podman integration |
| 3.4 | Agent Registration | 3.1, 3.2 | Node onboarding |

**Why third:** Agents depend on API existing. Operations flow requires both sides.

### Phase 4: Operations Integration

**Goal:** Connect full lifecycle operations

| Step | Component | Dependencies | Rationale |
|------|-----------|--------------|-----------|
| 4.1 | Create Server Flow | 2.1, 3.3 | Full deployment |
| 4.2 | Start/Stop Flow | 2.1, 3.3 | Full control |
| 4.3 | Status Reporting | 3.2 | Feedback loop |
| 4.4 | Metrics Collection | 1.1, 3.2 | Observability |

**Why fourth:** Depends on both API and agent. Complete operations require integration.

### Phase 5: Advanced Features

**Goal:** Add differentiated capabilities

| Step | Component | Dependencies | Rationale |
|------|-----------|--------------|-----------|
| 5.1 | RCON Handler | 3.2 | Game console access |
| 5.2 | SFTP Handler | 3.2 | File management |
| 5.3 | Backup Handler | 1.1, 3.2 | Data protection |
| 5.4 | Background Worker | 1.2 | Async processing |

**Why fifth:** These enhance core functionality. Build on established flows.

### Phase 6: Frontend Enhancement

**Goal:** Complete user experience

| Step | Component | Dependencies | Rationale |
|------|-----------|--------------|-----------|
| 6.1 | Server Management UI | 2.2, 4.1-4.4 | Full control panel |
| 6.2 | Real-time Status | 4.3 | Live updates |
| 6.3 | Metrics Dashboard | 5.4, 4.4 | Visualization |
| 6.4 | File Browser | 5.2 | UX enhancement |

**Why last:** Frontend depends on all backend capabilities being operational.

## Cross-Cutting Concerns

### Communication Patterns

| Pattern | Use Case | Implementation |
|---------|---------|----------------|
| Request/Response | User actions, CRUD | REST API |
| Bidirectional | Node commands, results | WebSocket |
| Pub/Sub | Events, notifications | Redis pub/sub |
| Polling | Async jobs | Redis queue |

### Scalability Boundaries

| Scale | Architecture Adjustment |
|-------|------------------------|
| 1-10 nodes | Single API instance, single worker |
| 10-50 nodes | API with connection pooling, horizontal worker |
| 50+ nodes | API clustering, Redis sharding, multiple workers |

### Security Boundaries

```
┌────────────────┐    ┌────────────────┐    ┌────────────────┐
│   Public       │    │   internal     │    │   Agent        │
│   Internet     │───>│   Network      │───>│   Network      │
│                │    │   (API)        │    │   (Agents)     │
└────────────────┘    └────────────────┘    └────────────────┘
       │                      │                      │
       v                      v                      v
  Frontend              API Backend             Node Agents
  (HTTPS)              (Auth required)         (mTLS optional)
```

## Industry Patterns Summary

| Pattern | Used By | Why |
|---------|--------|-----|
| Agent-based orchestration | Hathora, PlayFab | Separates control plane from compute |
| WebSocket for node communication | Industry standard | Bidirectional, low-latency |
| Redis for async queue | All major platforms | Reliable job processing |
| Container per game instance | Minecraft, Palworld | Isolation, resource control |

## Key Findings

1. **Existing architecture aligns with industry:** The codebase already implements the Hub-and-Spoke model used by game hosting platforms.

2. **WebSocket is critical:** Agent communication via WebSocket is the standard pattern for real-time node control.

3. **Build order is dependencies-first:** Core infrastructure → API → Agent → Operations → Frontend follows natural dependencies.

4. **Container isolation is fundamental:** Podman/Docker per game instance provides the necessary isolation and resource control.

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Component boundaries | HIGH | Matches existing codebase + industry patterns |
| Data flow | HIGH | Based on existing implementation + standard patterns |
| Build order | MEDIUM | Suggests logical progression, may need adjustment |
| Scalability | HIGH | Pattern proven at scale by industry |

## Gaps to Address During Implementation

- **Phase-specific validation:** Build order may shift based on phase priorities
- **Multi-region considerations:** Not covered here, needs research later
- **Auto-scaling patterns:** Needs deeper research for scale-out strategy

## Sources

- Existing codebase architecture analysis (2026-04-08)
- Hathora scalable WebSocket architecture patterns
- Industry best practices for game server hosting
- CloudPap game server scaling patterns

---

*Research file: ARCHITECTURE.md - informs roadmap phase structure*