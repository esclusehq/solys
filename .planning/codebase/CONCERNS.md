# Codebase Concerns

**Analysis Date:** 2026-04-08

---

## Tech Debt

### Lemon Squeezy Webhook Signature Not Verified
- **Issue:** Webhook payload signature is not validated, allowing potential spoofing attacks
- **Files:** `api/src/infrastructure/billing/lemon_squeezy_service.rs` (line 174)
- **Impact:** Production payment webhooks can be forged; attackers could trigger subscription changes without valid signatures
- **Fix approach:** Implement Ed25519 signature verification using the `x-signature` header and the merchant signing secret

### Multiple Payment Providers Partially Implemented
- **Issue:** Both Lemon Squeezy and Stripe services exist, but only Lemon Squeezy is actively used
- **Files:** `api/src/infrastructure/billing/lemon_squeezy_service.rs`, `api/src/infrastructure/billing/stripe_service.rs`
- **Impact:** Maintenance overhead for unused code; potential confusion about which provider is active
- **Fix approach:** Consolidate to single payment provider or fully implement both with clear activation mechanism

### Inconsistent Error Handling with Panics
- **Issue:** Extensive use of `.unwrap()` and `.expect()` throughout handlers can cause runtime panics instead of proper error responses
- **Files:**
  - `api/src/presentation/handlers/ws_handler.rs` (line 28)
  - `api/src/presentation/handlers/terminal_handlers.rs` (lines 34, 93, 108, 131)
  - `api/src/presentation/handlers/profiling_handlers.rs` (lines 437, 441)
  - `api/src/presentation/handlers/docker_log_handler.rs` (line 48)
  - `api/src/presentation/handlers/build_handlers.rs` (lines 251, 331)
  - `api/src/domain/webhook/service.rs` (lines 36, 158)
  - `api/src/domain/rbac/middleware.rs` (line 48)
  - `api/src/domain/usage/service.rs` (lines 231, 232, 350, 353)
  - `api/src/bootstrap/mod.rs` (line 38)
  - `api/src/domain/auth/middleware.rs` (line 101)
  - `api/src/domain/billing/webhooks.rs` (line 193)
- **Impact:** Unhandled edge cases cause service crashes instead of graceful error responses
- **Fix approach:** Replace panic-inducing code with proper Result handling and meaningful error responses

---

## Known Bugs

### Cookie Parsing in WebSocket Handler Assumes Format
- **Symptoms:** WebSocket connections fail if cookie format differs from expected "access_token=TOKEN"
- **Files:** `api/src/presentation/handlers/ws_handler.rs` (line 28)
- **Trigger:** Any cookie that doesn't start with "access_token=" causes unwrap panic
- **Workaround:** Ensure client sends properly formatted cookies

### DateTime Parsing Without Validation
- **Symptoms:** Usage tracking fails silently when date parsing returns None
- **Files:** `api/src/domain/usage/service.rs` (lines 231, 232, 350, 353)
- **Trigger:** Invalid date formats passed to usage service
- **Workaround:** Validate date inputs before parsing

---

## Security Considerations

### Hardcoded Database Credentials in Docker Compose
- **Risk:** Production database passwords visible in docker-compose.yml
- **Files:** `docker-compose.yml` (lines 7, 8, 24, 43, 44)
- **Current mitigation:** None - passwords are in plaintext
- **Recommendations:** 
  - Use Docker secrets or environment variables from secure vault
  - Remove inline credentials from docker-compose.yml
  - Add `.env` to `.gitignore` and document external secret management

### Secrets in .env.example
- **Risk:** Example file contains actual JWT_SECRET and placeholder database URLs that could be mistaken for production values
- **Files:** `api/.env.example`
- **Current mitigation:** None
- **Recommendations:** Replace all example secrets with placeholder strings like "CHANGE_ME" or "your-secret-here"

### Supabase Keys in Dockerfile Build Args
- **Risk:** Supabase anon key embedded in docker build arguments
- **Files:** `docker-compose.yml` (line 62)
- **Current mitigation:** Using anon key (client-side safe), but still exposes project ID
- **Recommendations:** Use build-time secrets for any sensitive values

### WebSocket Token Extraction from Cookies
- **Risk:** Token parsing doesn't validate JWT structure before extracting
- **Files:** `api/src/presentation/handlers/ws_handler.rs` (lines 21-28)
- **Current mitigation:** Token validated after extraction, but malformed cookies could cause errors
- **Recommendations:** Add safer parsing with Option/Result instead of unwrap

---

## Performance Bottlenecks

### Synchronous SSH Session Creation
- **Problem:** SSH executor creates sessions synchronously, blocking on network I/O
- **Files:** `api/src/infrastructure/executors/ssh_server_executor.rs` (line 27)
- **Cause:** Blocking session initialization in async context
- **Improvement path:** Use async SSH library (e.g., ssh2 async wrapper) or run in blocking thread pool

### Missing Database Query Optimization
- **Problem:** No visible query optimization (no eager loading hints visible in repositories)
- **Files:** `api/src/infrastructure/repositories/*.rs`
- **Cause:** N+1 query patterns likely in relationships
- **Improvement path:** Add sqlx::Result::option_multi or explicit joins for related data

### Redis Connection Pool May Be Undersized
- **Problem:** Default pool size of 10 may not handle concurrent requests under load
- **Files:** `api/.env.example` (line 45)
- **Cause:** Static pool size doesn't auto-scale
- **Improvement path:** Implement dynamic connection pooling or increase default based on load testing

---

## Fragile Areas

### WebSocket Connection Management
- **Files:** `api/src/presentation/ws/node_connection_manager.rs`, `api/src/presentation/ws/node_protocol.rs`
- **Why fragile:** Complex state management for node connections; connection drops can leave orphaned state
- **Safe modification:** Test thoroughly with connection race conditions; add reconnection logic
- **Test coverage:** Appears limited - no integration tests for WebSocket handling

### Server Executor Trait Implementation
- **Files:** `api/src/infrastructure/executors/*.rs`
- **Why fragile:** Multiple executor implementations (podman, ssh, mock, agent, rcon) with different failure modes
- **Safe modification:** Add integration tests for each executor type; standardize error types
- **Test coverage:** Minimal - only mock executor visible in test files

### Billing Webhook Processing
- **Files:** `api/src/domain/billing/webhooks.rs`, `api/src/infrastructure/billing/lemon_squeezy_service.rs`
- **Why fragile:** Critical payment logic with minimal signature validation and complex parsing
- **Safe modification:** Add idempotency checks, signature verification, and comprehensive error handling
- **Test coverage:** Unknown - no visible tests for billing webhook edge cases

---

## Scaling Limits

### Single Backend Instance Assumption
- **Current capacity:** Designed for single API instance
- **Limit:** Horizontal scaling requires session affinity or Redis-backed sessions
- **Scaling path:** Use Redis for session storage instead of in-memory; add stateless JWT validation

### PostgreSQL Connection Limits
- **Current capacity:** Not configured explicitly
- **Limit:** Default PostgreSQL max_connections (typically 100) will limit concurrent requests
- **Scaling path:** Configure connection pooling (e.g., deadpool) with explicit pool size; consider read replicas

### In-Memory Rate Limiting
- **Current capacity:** Per-instance rate limiting
- **Limit:** Distributed deployments would need shared rate limit state
- **Scaling path:** Move rate limiting to Redis for distributed deployments

---

## Dependencies at Risk

### Rust Crate Version Pins Not Visible
- **Risk:** Cargo.toml doesn't show exact versions, relying on Cargo.lock
- **Impact:** Dependency updates require careful review
- **Migration plan:** Review Cargo.lock regularly; consider pinning critical security-sensitive crates

### External Service Dependencies
- **Service:** Lemon Squeezy (billing), Stripe (partial), Discord (webhooks), Modrinth (plugins)
- **Impact:** Service downtime or API changes could break functionality
- **Migration plan:** Add circuit breakers for external services; implement fallback behaviors

---

## Missing Critical Features

### Proper Error Type Hierarchy
- **Problem:** Using Box<dyn Error> and string errors inconsistently across domain
- **Blocks:** Structured error logging, error code extraction for clients, automated error recovery
- **Priority:** High

### Health Check Endpoints for All Services
- **Problem:** Only basic liveness checks visible; no deep health for dependencies
- **Blocks:** Container orchestration health monitoring, degraded mode detection
- **Priority:** Medium

### Request Tracing/Correlation IDs
- **Problem:** No visible request ID propagation across service layers
- **Blocks:** Distributed request tracing, debugging production issues
- **Priority:** Medium

---

## Test Coverage Gaps

### Billing Service Tests
- **What's not tested:** Webhook signature verification, subscription state transitions, invoice generation
- **Files:** `api/src/infrastructure/billing/`
- **Risk:** Payment processing bugs could go undetected until production
- **Priority:** High

### WebSocket Handler Tests
- **What's not tested:** Connection upgrades, token validation, message routing
- **Files:** `api/src/presentation/handlers/ws_handler.rs`, `api/src/presentation/ws/`
- **Risk:** Real-time features break silently
- **Priority:** High

### Executor Integration Tests
- **What's not tested:** Podman container lifecycle, SSH command execution, RCON interactions
- **Files:** `api/src/infrastructure/executors/`
- **Risk:** Server management operations fail in production
- **Priority:** High

### Frontend Error Boundaries
- **What's not tested:** React error boundaries, API error state handling, WebSocket reconnection
- **Files:** `app/src/`
- **Risk:** Poor user experience on errors; no graceful degradation
- **Priority:** Medium

---

## Documentation Gaps

### Missing Security Documentation
- **What's missing:** Authentication flow, RBAC model, API key lifecycle, tenant isolation approach
- **Files:** None exist
- **Impact:** Hard to audit security model; onboarding security review difficult

### Deployment Documentation
- **What's missing:** Production deployment steps, backup procedures, monitoring setup
- **Files:** `deploy.sh` exists but not documented
- **Impact:** Operations rely on tribal knowledge

### API Documentation
- **What's missing:** OpenAPI spec lacks descriptions, examples, and authentication requirements
- **Files:** `api/openapi.json`
- **Impact:** API consumers have difficulty integrating

---

*Concerns audit: 2026-04-08*