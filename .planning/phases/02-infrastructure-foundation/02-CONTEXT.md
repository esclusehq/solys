# Phase 2: Infrastructure Foundation - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish core data layer and messaging infrastructure - PostgreSQL schema, Redis caching/queue, WebSocket server for node agents, and repository layer for data access.

This phase builds on Phase 1 security foundation and enables subsequent phases for API, node communication, and server operations.
</domain>

<decisions>
## Implementation Decisions

### PostgreSQL Schema Strategy (D-04)
- **D-04:** Use sqlx with .sql files for compile-time schema checking
- Implementation: All SQL queries in migration files, use sqlx::query!() macro
- Existing migrations already present in api/migrations/
- Connection pooling via sqlx::postgres::PgPool

### Redis Configuration (D-05)
- **D-05:** Multi-purpose Redis usage for:
  - Session caching
  - Rate limiting (already implemented)
  - Node status caching
  - Job queue (already implemented)
- Implementation: RedisPool already exists in api/src/infrastructure/cache/redis.rs
- Use redis::aio::MultiplexedConnection for efficient pooling

### WebSocket Infrastructure (D-06)
- **D-06:** Use Axum native WebSocket with tokio-tungstenite
- Implementation: Existing handlers at api/src/presentation/handlers/node_ws_handler.rs
- Node connection manager already exists: api/src/presentation/ws/node_connection_manager.rs
- Reconnection logic handled via heartbeat mechanism

### Repository Layer Design (D-07)
- **D-07:** Use async traits with concrete implementations
- Implementation: Traits defined in domain/repositories/, implementations in infrastructure/
- Dependency injection via AppContainer (already exists in api/src/bootstrap/container.rs)
- Use Arc<dyn Repository> for thread-safe references

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Database
- `api/migrations/` — Existing migration files
- `api/src/infrastructure/cache/redis.rs` — Redis pool implementation

### WebSocket
- `api/src/presentation/handlers/node_ws_handler.rs` — Node WebSocket handler
- `api/src/presentation/ws/node_connection_manager.rs` — Connection management
- `api/src/presentation/routes/node_routes.rs` — WebSocket route definition

### Repository
- `api/src/domain/repositories/` — Repository traits
- `api/src/infrastructure/repositories/` — Concrete implementations
- `api/src/bootstrap/container.rs` — Dependency injection container

### Configuration
- `api/src/config/app_config.rs` — Config with validation (from Phase 1)

</canonical_refs>

<specifics>
## Specific Ideas

- Existing PostgreSQL schema is adequate for initial phases
- Redis is already integrated for rate limiting
- WebSocket handler for node agents already exists
- Repository pattern already partially implemented with traits

</specifics>

<deferred>
## Deferred Ideas

None — all infrastructure decisions captured.

</deferred>

---

*Phase: 02-infrastructure-foundation*
*Context gathered: 2026-04-09*
