# Codebase Structure

**Analysis Date:** 2026-04-08

## Directory Layout

```
escluse-deploy/
├── api/                    # Main backend API (Rust/Axum)
│   ├── src/
│   │   ├── domain/        # Business logic, entities, repository traits
│   │   ├── application/   # Use cases, services, DTOs
│   │   ├── infrastructure/# Concrete implementations
│   │   ├── presentation/  # HTTP routes, handlers, middleware
│   │   ├── shared/        # Constants, utils, errors
│   │   ├── config/        # Configuration
│   │   ├── bootstrap/     # App initialization, DI container
│   │   └── main.rs        # Entry point
│   ├── migrations/        # Database migrations
│   ├── Cargo.toml
│   └── Dockerfile
│
├── worker/                 # Background job processor
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── config.rs     # Configuration
│   │   ├── queue.rs      # Job processing
│   │   ├── agent/        # Agent communication
│   │   └── webhook/      # Webhook emission
│   └── Cargo.toml
│
├── web-agent/             # Node agent (runs on compute nodes)
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── agent/        # Agent core logic
│   │   ├── agent_connection.rs  # WebSocket client
│   │   ├── handlers/     # Task handlers (runtime, backup, rcon, etc.)
│   │   ├── task_state.rs # Task tracking
│   │   ├── api/          # Internal HTTP server
│   │   └── ...
│   ├── Cargo.toml
│   └── compose/           # Docker compose for agent
│
├── agent-core/            # Shared Rust crates workspace
│   ├── Cargo.toml
│   └── crates/
│       ├── agent-proto/   # Task/Result definitions
│       ├── agent-config/  # Config loading/validation
│       ├── agent-runtime/ # Docker/Podman detection
│       ├── agent-ssh/     # SSH client
│       ├── agent-rcon/    # RCON client
│       ├── agent-backup/  # Backup utilities
│       ├── agent-health/  # Health monitoring
│       ├── agent-metrics/ # Metrics collection
│       ├── agent-task/    # Task queue/dispatcher
│       ├── agent-security/# JWT, rate limiting, audit
│       ├── agent-event/   # Event handling
│       └── agent-capability/ # Capability registry
│
├── app/                   # Frontend (React SPA)
│   ├── src/
│   │   ├── main.jsx      # Entry point
│   │   ├── app/          # App component, routing
│   │   ├── pages/        # Page components (auth, dashboard, servers, billing, settings)
│   │   ├── components/   # Reusable components (Sidebar, IDE, FileManager, etc.)
│   │   ├── hooks/        # Custom React hooks (useServers, useNodes, useWebSocket, etc.)
│   │   ├── store/        # Zustand stores (authStore, serverStore, uiStore)
│   │   ├── lib/          # API client, Supabase client
│   │   ├── api/          # API client utilities
│   │   ├── context/      # React context providers
│   │   ├── types/        # TypeScript type definitions
│   │   └── features/     # Feature-specific components
│   ├── index.html
│   ├── vite.config.js
│   ├── package.json
│   └── Dockerfile
│
├── landing-page-escluse/  # Marketing landing page (separate repo)
├── gateway/               # Caddy reverse proxy config
├── migrations -> api/migrations  # Symlink to API migrations
├── docker-compose.yml     # Full stack compose
├── docker-compose-root.yml
├── deploy.sh              # Deployment script
└── sync.sh                # Sync script
```

## Directory Purposes

### API Backend (`api/`)

**Purpose:** Central REST API and WebSocket server

**Key directories:**
- `src/domain/`: Business entities (`entities/`), repository traits (`repositories/`), domain services (`auth/`, `billing/`, `rbac/`, `webhook/`)
- `src/application/use_cases/`: Request handlers (create_server, start_server, stop_server, etc.)
- `src/application/services/`: Background services (monitoring_service, webhook_service, backup_service, backup_scheduler, node_health_service)
- `src/infrastructure/repositories/`: Postgres implementations of repository traits
- `src/infrastructure/executors/`: Server executors (podman, ssh, rcon, mock)
- `src/infrastructure/billing/`: Stripe, Lemon Squeezy integration
- `src/infrastructure/external_services/`: Modrinth (plugins), Discord webhooks
- `src/presentation/routes/`: HTTP route definitions (server_routes, node_routes, api_routes, openapi_routes)
- `src/presentation/handlers/`: Request handlers for each resource
- `src/presentation/middleware/`: Auth, RBAC, rate limiting, CORS

**Key files:**
- `src/main.rs`: Entry point, starts Axum server
- `src/bootstrap/mod.rs`: App builder (config, DB, services, routes)
- `src/bootstrap/container.rs`: Dependency injection container (`AppContainer`)

### Web Agent (`web-agent/`)

**Purpose:** Node-resident agent executing server operations

**Key directories:**
- `src/handlers/`: Task execution (runtime.rs, backup.rs, rcon.rs, metrics.rs, ssh.rs, sftp.rs)
- `src/agent/`: Agent result sending
- `src/api/`: Internal HTTP server (health checks)

**Key files:**
- `src/main.rs`: Entry point, loads config, connects to backend
- `src/agent_connection.rs`: WebSocket client, message handling
- `src/handlers/mod.rs`: Task dispatch, retry logic, timeout handling

### Agent-Core (`agent-core/crates/`)

**Purpose:** Reusable libraries for agent functionality

**Each crate:** Self-contained with `lib.rs` defining public API

### Frontend (`app/src/`)

**Purpose:** User-facing web application

**Key directories:**
- `pages/`: Full page components organized by feature
- `components/`: Reusable UI components
- `hooks/`: Custom React hooks for data fetching
- `store/`: Zustand state management
- `lib/`: API client, Supabase client

## Key File Locations

**Entry Points:**
- `api/src/main.rs`: Backend API server (port 3000)
- `worker/src/main.rs`: Background job processor
- `web-agent/src/main.rs`: Node agent
- `app/src/main.jsx`: Frontend React entry
- `app/index.html`: Frontend HTML shell

**Configuration:**
- `api/src/config/`: AppConfig types
- `api/.env`: Backend environment variables
- `app/.env`: Frontend environment variables
- `web-agent/compose/.env`: Agent configuration
- `docker-compose.yml`: Full stack configuration

**Core Logic:**
- `api/src/domain/server/model.rs`: Server entity
- `api/src/application/use_cases/start_server_use_case.rs`: Server start logic
- `web-agent/src/handlers/runtime.rs`: Container operations (create, start, stop)

**Testing:**
- `api/tests/`: Integration tests

## Naming Conventions

**Rust:**
- Files: `snake_case.rs` (e.g., `server_routes.rs`, `task_queue.rs`)
- Modules: `snake_case` (e.g., `mod handlers`)
- Types/Enums: `PascalCase` (e.g., `ServerStatus`, `TaskResult`)
- Functions/Variables: `snake_case` (e.g., `create_server`, `pool_size`)
- Traits: `PascalCase` ending in `Trait` (e.g., `ServerRepository`)

**TypeScript/React:**
- Files: `PascalCase.tsx` for components (e.g., `LoginPage.tsx`), `camelCase.ts` for utilities
- Components: `PascalCase` (e.g., `Sidebar`, `ToastContainer`)
- Hooks: `camelCase` starting with `use` (e.g., `useServers`, `useWebSocket`)
- Stores: `camelCase` (e.g., `authStore.js`, `serverStore.js`)

## Where to Add New Code

**New API Endpoint:**
- Handler: `api/src/presentation/handlers/`
- Route: `api/src/presentation/routes/`
- Use Case: `api/src/application/use_cases/`
- Repository Trait: `api/src/domain/repositories/`
- Repository Impl: `api/src/infrastructure/repositories/`

**New Agent Task Type:**
- Task definition: `agent-core/crates/agent-proto/src/task.rs`
- Handler: `web-agent/src/handlers/`
- Retry logic: `web-agent/src/handlers/mod.rs` (`get_task_config`)

**New Frontend Page:**
- Page component: `app/src/pages/`
- Hook for data: `app/src/hooks/`
- Store (if needed): `app/src/store/`

**New Background Service:**
- Service: `api/src/application/services/`
- Init: `api/src/bootstrap/container.rs`
- Start: `api/src/bootstrap/mod.rs` (spawn async task)

## Special Directories

**migrations/**: Database schema migrations (PostgreSQL SQL files)
- Generated: Yes (via SQLx or manual)
- Committed: Yes
- Pattern: `YYYYMMDDHHMMSS_description.sql`

**node_modules/**: npm dependencies
- Generated: Yes
- Committed: No (in .gitignore)

**target/**: Rust build artifacts
- Generated: Yes
- Committed: No

**dist/**: Frontend build output
- Generated: Yes
- Committed: No

**web-agent/target/**: Agent build output
- Generated: Yes
- Committed: No

---

*Structure analysis: 2026-04-08*