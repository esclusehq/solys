# Phase 5: Server Deployment - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can create and provision game servers on nodes. This phase configures game types, port allocation, resource limits, and deployment configuration storage.

**Requirements:** DEPLOY-01, DEPLOY-02 (partial)

**Success criteria:**
1. User can select game type (Minecraft, Palworld, etc.) when deploying
2. Docker container created with appropriate game image
3. Ports allocated and configured correctly
4. Resource limits (CPU, RAM) applied to container
</domain>

<decisions>
## Implementation Decisions

### Game Type Configuration (D-16)
- **D-16:** Database-driven with code fallback
- Game types stored in database (add new games without redeploy)
- Fields: docker_image, default_ports, default_env, startup_command, capabilities
- Code-based fallback for default templates and schema validation

### Port Allocation Strategy (D-17)
- **D-17:** Dynamic pool allocation
- System allocates ports from pool, database tracks allocated ports
- Prevents conflicts automatically

### Resource Limits (D-18)
- **D-18:** Plan-based limits
- User picks from plans: 2GB, 4GB, 8GB, 16GB with fixed CPU ratios
- Simplifies UX, prevents over-provisioning

### Deployment Config Storage (D-19)
- **D-19:** Hybrid approach - separate table + snapshot
- deployment_configs table for source of truth (templates)
- servers.deployment_snapshot for immutable runtime config
- Snapshot created at deployment time, ensures consistency

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Server Model
- `api/src/domain/server/model.rs` — Server entity with game_type, ram_allocation
- `api/src/application/dto/server_dtos.rs` — CreateServerRequest DTO

### Executors
- `api/src/infrastructure/executors/podman_server_executor.rs` — Docker image mapping
- `api/src/infrastructure/executors/agent_server_executor.rs` — Agent-based deployment

### Use Cases
- `api/src/application/use_cases/create_server_use_case.rs` — Server creation logic
- `api/src/infrastructure/factories/simple_executor_factory.rs` — Executor selection

### Repository
- `api/src/infrastructure/repositories/postgres_server_repository.rs` — Server persistence

</canonical_refs>

<specifics>
## Specific Ideas

- Existing code maps game types to Docker images (minecraft → itzg/minecraft-server)
- Default network: devnode-minecraft
- Resource allocation stored in server.ram_allocation

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 05-server-deployment*
*Context gathered: 2026-04-09*
