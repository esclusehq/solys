# Phase 4: Node Agent Communication - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable bidirectional communication between API and node agents via WebSocket. This phase establishes the connection protocol, authentication, task distribution, and reconnection handling.

**Success criteria:**
1. Node agents can connect to API via WebSocket
2. Node agents can register and authenticate
3. Task distribution works from API to agents
4. Agent state machine handles reconnects properly (no race conditions)
</domain>

<decisions>
## Implementation Decisions

### Connection Protocol (D-12)
- **D-12:** JSON + ping/pong
- JSON messages with type field for message routing
- WebSocket ping/pong for heartbeat/keepalive
- Reference: `api/src/presentation/ws/node_protocol.rs`

### Node Authentication (D-13)
- **D-13:** API key authentication
- API key passed on WebSocket connection handshake
- Keys stored in database, rotated periodically
- Reference: `api/src/presentation/handlers/node_ws_handler.rs`

### Task Distribution (D-14)
- **D-14:** Redis queue + WebSocket
- Tasks queued in Redis, workers pull from queue
- Async response via WebSocket after task completion
- Reference: `api/src/infrastructure/cache/queue.rs`

### Reconnection Handling (D-15)
- **D-15:** State machine approach
- Track states: disconnected → connecting → authenticating → connected → ready
- Prevents race conditions during reconnect
- Reference: `api/src/presentation/ws/node_connection_manager.rs`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Protocol
- `api/src/presentation/ws/node_protocol.rs` — Message types and serialization

### Connection
- `api/src/presentation/handlers/node_ws_handler.rs` — WebSocket handler
- `api/src/presentation/ws/node_connection_manager.rs` — Connection state management

### Authentication
- `api/src/domain/node/repository.rs` — Node repository
- `api/src/presentation/routes/node_routes.rs` — Node routes including /api/ws/node

### Task Queue
- `api/src/infrastructure/cache/queue.rs` — Job queue implementation

### Agent
- `web-agent/src/agent_connection.rs` — Agent WebSocket client

</canonical_refs>

<specifics>
## Specific Ideas

- Existing WebSocket handler already processes node connections
- NodeConnectionManager tracks active connections
- API key validation happens at connection time
- Task distribution uses existing job queue

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 04-node-agent-communication*
*Context gathered: 2026-04-09*
