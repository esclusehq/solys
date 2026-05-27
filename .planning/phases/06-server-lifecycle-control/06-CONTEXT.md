# Phase 6: Server Lifecycle Control - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Full server lifecycle management - start, stop, restart, delete. This phase establishes state management, delete confirmation, stop behavior, and restart strategy.

**Requirements:** DEPLOY-02, DEPLOY-03, DEPLOY-04, DEPLOY-05

**Success criteria:**
1. User can start a deployed game server
2. User can stop a running game server
3. User can restart a running game server
4. User can delete a game server (with confirmation)
</domain>

<decisions>
## Implementation Decisions

### Server State Management (D-20)
- **D-20:** Optimistic with async
- DB status updated immediately, async operation in background
- Status changes reflect user intent quickly
- Reference: `api/src/application/use_cases/start_server_use_case.rs`

### Delete Confirmation (D-21)
- **D-21:** UI confirm + soft delete
- UI shows confirmation dialog before delete
- Soft delete in DB (marked deleted), cleanup after delay
- Reference: `api/src/application/use_cases/delete_server_use_case.rs`

### Stop Behavior (D-22)
- **D-22:** Graceful with timeout
- Send stop signal, wait 30s grace period, force kill if needed
- Prevents data corruption from abrupt stops
- Reference: `api/src/infrastructure/executors/podman_server_executor.rs`

### Restart Strategy (D-23)
- **D-23:** Stop then start
- Stop, wait for cleanup, then start - preserves container
- Faster than destroy/recreate, maintains state
- Reference: `api/src/infrastructure/executors/podman_server_executor.rs`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Use Cases
- `api/src/application/use_cases/start_server_use_case.rs`
- `api/src/application/use_cases/stop_server_use_case.rs`
- `api/src/application/use_cases/delete_server_use_case.rs`

### Executors
- `api/src/infrastructure/executors/podman_server_executor.rs`
- `api/src/infrastructure/executors/agent_server_executor.rs`
- `api/src/domain/server_executor.rs` — trait definition

### Handlers
- `api/src/presentation/handlers/server_handlers.rs`

### Server Model
- `api/src/domain/server/model.rs` — status field

</canonical_refs>

<specifics>
## Specific Ideas

- Existing use cases: StartServerUseCase, StopServerUseCase, DeleteServerUseCase
- Status field in Server model: starting, running, stopping, stopped, error
- Routes: /:id/start, /:id/stop, /:id/restart, /:id (DELETE)

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 06-server-lifecycle-control*
*Context gathered: 2026-04-09*
