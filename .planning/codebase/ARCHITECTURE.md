# Architecture

**Analysis Date:** 2026-04-08

## Pattern Overview

**Overall:** Distributed Microservices Architecture with Agent-Based Node Management

**Key Characteristics:**
- **Main API Server**: Rust Axum-based REST API handling business logic, authentication, and orchestration
- **Background Worker**: Separate Rust service processing async jobs from Redis queue
- **Node Agents**: Lightweight Rust agents deployed on compute nodes, connected via WebSocket
- **React Frontend**: Single-page application consuming REST APIs
- **Shared Crates**: Workspace of reusable libraries for agent functionality

## Layers

### API Backend (`api/`)

**Purpose:** Central orchestration point for all operations

**Location:** `api/src/`

**Contains:**
- **Domain Layer** (`domain/`): Business entities, repository traits, services, RBAC, billing models
- **Application Layer** (`application/`): Use cases, DTOs, background services (monitoring, webhook, backup)
- **Infrastructure Layer** (`infrastructure/`): Concrete implementations - Postgres repositories, executor factories, external service clients
- **Presentation Layer** (`presentation/`): HTTP routes, handlers, middleware, WebSocket management
- **Shared** (`shared/`): Constants, utilities, error types

**Depends on:** PostgreSQL (data), Redis (caching/queue), external services (billing, email, S3)

**Used by:** Frontend (REST), Node Agents (WebSocket), Webhooks (external callbacks)

**Architecture Pattern:** Clean Architecture with dependency injection via `AppContainer`

```rust
// api/src/bootstrap/container.rs - Dependency injection example
pub struct AppContainer {
    pub pool: PgPool,
    pub create_server_use_case: Arc<CreateServerUseCase<dyn ServerRepository>>,
    pub start_server_use_case: Arc<StartServerUseCase<dyn ServerRepository, dyn ExecutorFactory>>,
    // ... many more use cases and services
}
```

### Worker Service (`worker/`)

**Purpose:** Async job processing for long-running operations

**Location:** `worker/src/main.rs`

**Contains:**
- Job processor consuming Redis queues
- Webhook emission for external notifications

**Depends on:** Redis, backend API

### Web Agent (`web-agent/`)

**Purpose:** Node-resident agent handling container operations

**Location:** `web-agent/src/main.rs`

**Contains:**
- WebSocket client connecting to backend
- Task handlers (runtime, backup, rcon, metrics, ssh, sftp)
- Agent connection management with reconnection logic

**Depends on:** Backend WebSocket, Podman/Docker (via Bollard)

**Used by:** Backend for direct server operations on nodes

### Frontend (`app/`)

**Purpose:** User-facing web interface

**Location:** `app/src/`

**Contains:**
- React 19 SPA with React Router
- Zustand for state management
- Monaco Editor for IDE
- Supabase for auth

**Depends on:** Backend REST API, Supabase

### Agent-Core (`agent-core/`)

**Purpose:** Shared Rust crates for agent functionality

**Location:** `agent-core/crates/`

**Contains:**
- `agent-proto`: Task/result definitions, protocol messages
- `agent-config`: Configuration loading and validation
- `agent-runtime`: Runtime detection (Docker/Podman)
- `agent-ssh`: SSH client and connection pooling
- `agent-backup`: Backup compression utilities
- `agent-rcon`: RCON client implementation
- `agent-health`: Health monitoring, circuit breakers, retry logic
- `agent-metrics`: Metrics collection
- `agent-task`: Task queue and dispatcher
- `agent-security`: JWT, rate limiting, audit
- `agent-event`: Event handling
- `agent-capability`: Capability registry

## Data Flow

### Server Creation Flow:

1. **User Action**: Frontend sends POST to `/api/servers`
2. **API Handler**: Routes to `create_server_use_case`
3. **Domain**: Creates server entity in Postgres
4. **Executor Factory**: Selects appropriate executor (AgentServerExecutor)
5. **Node Client**: Sends task via WebSocket to target node
6. **Web Agent**: Receives task, pulls image, creates container via Bollard
7. **Result**: Web agent sends TaskResult back via WebSocket
8. **API**: Updates server status in database, returns to frontend

### Agent Registration Flow:

1. **Web Agent**: Connects to `/api/ws/node` via WebSocket
2. **Registration**: Sends `Register` message with capabilities
3. **Backend**: Creates Node record in Postgres, assigns UUID
4. **Ack**: Sends `RegisterAck` with node_id
5. **Heartbeat**: Agent sends periodic heartbeats, backend tracks health

### Background Job Flow:

1. **Trigger**: API enqueues job to Redis (e.g., backup, alert evaluation)
2. **Worker**: Polls Redis, processes job
3. **Result**: Updates database, emits events, triggers webhooks

## Key Abstractions

**Repository Pattern:**
```rust
// domain/repositories/server_repository.rs
pub trait ServerRepository: Send + Sync {
    async fn create(&self, server: Server) -> Result<Server>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Server>>;
    // ...
}
```

**Executor Factory:**
```rust
// domain/factories/mod.rs
pub trait ExecutorFactory: Send + Sync {
    fn create_server_executor(&self, node_id: Uuid) -> Box<dyn ServerExecutor>;
}
```

**Task Dispatch (Agent-Core):**
```rust
// agent-core/crates/agent-task/src/dispatcher.rs
pub trait TaskDispatcher: Send + Sync {
    fn dispatch(&self, task: Task) -> impl Future<Output = Result<TaskResult>>;
}
```

## Entry Points

**API Backend:**
- Location: `api/src/main.rs`
- Triggers: Docker compose starts container, binds to port 3000
- Responsibilities: HTTP server, WebSocket server, background services startup

**Worker:**
- Location: `worker/src/main.rs`
- Triggers: Docker compose starts container
- Responsibilities: Job queue processing, webhook emission

**Web Agent:**
- Location: `web-agent/src/main.rs`
- Triggers: Run binary directly or via container
- Responsibilities: Agent connection, task execution, metrics reporting

**Frontend:**
- Location: `app/src/main.jsx`
- Triggers: Browser loads index.html
- Responsibilities: SPA rendering, API communication

## Error Handling

**Strategy:** Centralized error types with application-specific codes

**Patterns:**
- Custom error types in `shared/errors/`
- `anyhow::Result` for fallible operations
- Task retry logic with exponential backoff in agent handlers
- Circuit breaker pattern for external services (`agent-health` crate)

## Cross-Cutting Concerns

**Logging:** `tracing` crate with `tracing-subscriber` for structured logging

**Validation:** Config validation in `agent-config` crate, request validation in handlers

**Authentication:** JWT-based auth in `domain/auth/`, RBAC middleware in `domain/rbac/`

**Caching:** Redis for session caching, rate limiting

---

*Architecture analysis: 2026-04-08*