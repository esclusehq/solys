# Domain Pitfalls

**Domain:** Game Server Hosting Platform  
**Researched:** 2026-04-09  
**Project Context:** Adding features to existing platform (brownfield)

---

## Critical Pitfalls

Mistakes that cause data loss, security breaches, or require significant rewrites.

### Pitfall 1: Unverified Webhook Payloads (Billing)

**What goes wrong:** Payment provider webhooks (Lemon Squeezy) processed without signature verification allow attackers to forge subscription events, granting free access or manipulating billing state.

**Why it happens:** In existing codebase: `lemon_squeezy_service.rs` line 174 lacks Ed25519 signature verification. The `x-signature` header is ignored.

**Consequences:**
- Attackers trigger subscription changes without payment
- Revenue loss from forged "paid" events
- Service state desynchronization with payment provider

**Prevention:**
- Implement Ed25519 signature verification using merchant signing secret
- Reject all webhooks with missing or invalid signatures
- Add webhook replay protection (idempotency keys)

**Warning signs:**
- Subscription state mismatches between platform and payment provider
- Unusually high "successful" payments without corresponding transactions

**Phase to address:** This is existing technical debt from CONCERNS.md. Should be addressed in an early infrastructure phase before adding subscription features.

---

### Pitfall 2: Container Memory Limits Too Low

**What goes wrong:** Game servers (especially Java-based like Minecraft) exceed allocated memory, get OOM-killed, causing world corruption and player data loss.

**Why it happens:** Default container memory limits don't account for JVM heap + OS overhead + world data. A "4GB" limit often means only ~2.5GB usable heap.

**Consequences:**
- Sudden server crashes during gameplay
- Save file corruption from improper shutdown
- World rollback required (data loss)
- Player churn from unreliable hosting

**Prevention:**
- Calculate memory as: game RAM + 1GB overhead (JVM metadata, OS, cache)
- Set container memory limits with headroom, not exact allocation
- Implement memory monitoring with alerts before OOM
- Configure proper shutdown hooks for graceful terminations

**Warning signs:**
- Container restart logs showing exit code 137 (OOMKilled)
- JVM garbage collection pauses
- Players reporting sudden crashes during load

**Phase to address:** Core hosting phase - server allocation and container configuration.

---

### Pitfall 3: Hardcoded Secrets in Configuration

**What goes wrong:** Production credentials committed to version control or embedded in Docker Compose, exposing database passwords and API keys.

**Why it happens:** In existing codebase: `docker-compose.yml` lines 7, 8, 24, 43, 44 contain hardcoded database passwords. `.env.example` contains actual JWT secrets.

**Consequences:**
- Credential exposure in repository history (cannot be removed)
- Production breaches if repository is compromised
- Compliance violations (GDPR, PCI-DSS)

**Prevention:**
- Remove all secrets from docker-compose.yml
- Use Docker secrets or environment variables from vault
- Replace `.env.example` with placeholder strings ("CHANGE_ME")
- Implement secret rotation capability

**Warning signs:**
- Git history contains "password", "secret", "key" in config files
- No secrets management solution visible in infrastructure

**Phase to address:** Immediately - security critical. Should be fixed before any production deployment.

---

### Pitfall 4: Race Conditions in WebSocket Connection State

**What goes wrong:** Node connection manager has complex state that can become inconsistent when connections drop and reconnect rapidly, leaving orphaned or duplicate connection states.

**Why it happens:** In existing codebase: `node_connection_manager.rs` and `node_protocol.rs` manage WebSocket state without atomic operations or proper reconnection logic.

**Consequences:**
- Commands sent to disconnected nodes silently fail
- Node appears "online" but isn't responding
- Duplicate message delivery or message loss
- User-facing failures to start/stop servers

**Prevention:**
- Implement connection state machine with proper transitions
- Add reconnection logic with exponential backoff
- Use atomic reference counting for connection state
- Add integration tests for connection race conditions

**Warning signs:**
- Intermittent "node not responding" errors
- Commands succeed but don't actually execute
- WebSocket disconnection logs without cleanup

**Phase to address:** Node communication phase - requires comprehensive testing before adding features that depend on reliable node state.

---

## Moderate Pitfalls

Mistakes that cause degraded user experience or require significant work to fix.

### Pitfall 5: Synchronous SSH Blocking in Async Context

**What goes wrong:** SSH session creation blocks the async runtime, causing thread starvation and slow response times under concurrent load.

**Why it happens:** In existing codebase: `ssh_server_executor.rs` line 27 creates SSH sessions synchronously in async handlers.

**Consequences:**
- Request timeouts under load
- API workers blocked waiting for SSH
- Poor performance as concurrent requests increase

**Prevention:**
- Use async SSH library (ssh2 async wrapper) or run in blocking thread pool
- Implement connection pooling for SSH sessions
- Add request timeouts at application level

**Phase to address:** Infrastructure optimization phase - before scaling beyond single instance.

---

### Pitfall 6: Panics Instead of Proper Error Responses

**What goes wrong:** Extensive `.unwrap()` and `.expect()` throughout handlers cause service crashes instead of graceful error responses.

**Why it happens:** Multiple files in existing codebase use panic-inducing patterns: `ws_handler.rs`, `terminal_handlers.rs`, `profiling_handlers.rs`, `docker_log_handler.rs`, etc.

**Consequences:**
- Single request error crashes entire service
- Poor user experience (500 errors instead of proper messages)
- Difficult debugging (no context in panic logs)

**Prevention:**
- Replace all unwrap/expect with proper Result handling
- Implement error type hierarchy (not Box<dyn Error>)
- Add middleware for graceful error responses

**Warning signs:**
- Frequent service restarts in logs
- Error messages containing "panicked at"

**Phase to address:** Early infrastructure phase - foundational for adding features reliably.

---

### Pitfall 7: Single Instance Architecture Assumptions

**What goes wrong:** Platform designed for single API instance; horizontal scaling requires session affinity or Redis-backed sessions.

**Why it happens:** In-memory rate limiting, session storage, and connection state assume single instance.

**Consequences:**
- Cannot scale horizontally without issues
- Deployment requires rolling updates with downtime
- Rate limiting ineffective in distributed setup

**Prevention:**
- Use Redis for session storage instead of in-memory
- Implement stateless JWT validation
- Move rate limiting to Redis for distributed deployments

**Phase to address:** Scaling phase - before adding multi-instance deployment capability.

---

### Pitfall 8: Game Save Data Loss on Container Termination

**What goes wrong:** Game server state (worlds, player data, configs) stored in container's writable layer; container termination destroys data.

**Why it happens:** Game servers write to working directory; containers don't persist data unless volumes are properly configured.

**Consequences:**
- Complete server reset on crash/restart
- Player progress loss
- Configurations lost

**Prevention:**
- Use named Docker volumes for game data (not container filesystem)
- Implement backup before any destructive operation
- Document volume mount requirements per game

**Warning signs:**
- "world reset" complaints from users
- No volume configuration visible in server creation

**Phase to address:** Server provisioning phase - data persistence is core to the product value.

---

### Pitfall 9: Missing Backup Strategy

**What goes wrong:** No automated backups of game server data; user data depends entirely on single point of failure.

**Why it happens:** Backup infrastructure not implemented; relies on user manual backups (which never happen).

**Consequences:**
- Complete data loss on hardware failure
- No recovery from user error (accidental deletion)
- Unrecoverable from bugs corrupting save data

**Prevention:**
- Implement automated backup schedule (daily/weekly)
- Store backups in separate location from primary data
- Test restore procedures regularly
- Allow user-triggered backups before major changes

**Warning signs:**
- No backup-related configuration in server creation
- No "restore" functionality visible

**Phase to address:** Data management phase - before user trust can be established.

---

## Minor Pitfalls

Issues that cause friction but are lower priority.

### Pitfall 10: Inconsistent Game Server Process Management

**What goes wrong:** Different game types require different startup, shutdown, and monitoring approaches; treating all games identically causes failures.

**Why it happens:** Minecraft, Palworld, Valheim have different process requirements, ports, and resource needs.

**Consequences:**
- Some games fail to start
- Incorrect resource allocation
- No monitoring of game-specific metrics

**Prevention:**
- Create game-type profiles with specific configurations
- Validate game compatibility before deployment
- Implement game-specific health checks

**Phase to address:** Game support phase - as new game types are added.

---

### Pitfall 11: Missing Health Checks for Dependencies

**Problem:** Only basic liveness checks; no deep health for PostgreSQL, Redis, or node connectivity.

**Why it happens:** Health endpoints don't verify dependent services.

**Consequences:**
- Container orchestrator marks healthy when API is up but database is down
- Traffic routed to degraded instances
- Poor debugging of partial failures

**Prevention:**
- Add liveness (process) and readiness (dependencies) probes
- Check PostgreSQL, Redis, and node connectivity in health endpoints
- Return 503 when degraded to allow load balancer to route elsewhere

**Phase to address:** Infrastructure phase - before reliable operations.

---

### Pitfall 12: No Request Correlation IDs

**Problem:** No request ID propagation across service layers; difficult to trace issues across microservices.

**Why it happens:** Distributed tracing not implemented.

**Consequences:**
- Cannot correlate logs across API, worker, and agent
- Production debugging takes hours
- Cannot reproduce user issues from logs

**Prevention:**
- Add request ID header (X-Request-ID) generation and propagation
- Include correlation ID in all log statements
- Implement structured logging with correlation context

**Phase to address:** Observability phase - before production debugging is needed.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Server deployment | OOM kills, data loss | Proper memory limits, volumes from start |
| Node communication | Race conditions, timeouts | Test connection logic before adding features |
| Billing | Webhook spoofing | Verify signatures before processing |
| Data management | No backups, no recovery | Implement backup system early |
| Scaling | Single instance limits | Design for distributed from phase 1 |
| Error handling | Service crashes | Replace panics with proper errors early |

---

## Integration with Existing Concerns

The following items from `.planning/codebase/CONCERNS.md` map to pitfalls above:

| Concern | Related Pitfall | Phase |
|---------|-----------------|-------|
| Lemon Squeezy webhook signature not verified | Pitfall 1: Webhook security | Immediate (security) |
| Hardcoded database credentials | Pitfall 3: Hardcoded secrets | Immediate (security) |
| Inconsistent error handling with panics | Pitfall 6: Panics | Early infrastructure |
| WebSocket connection management fragile | Pitfall 4: Connection state | Node communication |
| Synchronous SSH session creation | Pitfall 5: Blocking async | Infrastructure optimization |
| Single backend instance assumption | Pitfall 7: Single instance | Scaling phase |
| Missing proper error type hierarchy | Pitfall 6: Panics | Early infrastructure |

---

## Sources

- [Pterodactyl Panel Issue: Backup crashes](https://github.com/pterodactyl/panel/issues/3135) - backup reliability
- [Kubernetes OOMKilled errors](https://medium.com/@pillarslive/why-pods-get-oomkilled-in-kubernetes-and-why-memory-limits-wont-save-you-5891b8f338ea) - memory management
- [Docker in Production mistakes](https://zeonedge.com/sw/blog/docker-production-mistakes-2026) - container best practices
- [Game server containerization](https://gameteam.io/blog/docker-minecraft-server-container-orchestration/) - game-specific considerations
- Existing codebase analysis from `.planning/codebase/CONCERNS.md`

---

*Research complete: 2026-04-09*