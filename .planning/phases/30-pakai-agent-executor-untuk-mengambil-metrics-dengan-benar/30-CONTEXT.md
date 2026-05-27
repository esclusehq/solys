# Phase 30: pakai agent executor untuk mengambil metrics dengan benar - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Mengaktifkan AgentServerExecutor untuk mengambil container metrics (CPU, memory, disk) via WebSocket heartbeat dari agent ke backend, bukan via SSH executor atau RconServerExecutor.

</domain>

<decisions>
## Implementation Decisions

### Executor Type
- **D-01:** Ubah executor_type server dari "minecraft" ke "agent" untuk menggunakan AgentServerExecutor

### Container Naming
- **D-02:** Pattern: `mc-{server_id}` - agent mencari container dengan nama ini saat collect metrics

### Fallback Strategy
- **D-03:** Jika agent/node disconnected, tampilkan "not connected to agent" - bukan return zeros atau skip

### Metrics Source
- **D-04:** Container metrics dikirim via WebSocket heartbeat dari agent (sudah diimplementasi di code sebelumnya)

</decisions>

<canonical_refs>
## Canonical References

### Existing Code
- `api/src/infrastructure/executors/agent_server_executor.rs` — AgentServerExecutor implementation
- `api/src/infrastructure/node_client/agent_client.rs` — NodeClient trait and AgentNodeClient
- `api/src/presentation/ws/node_connection_manager.rs` — Container cache via heartbeat
- `web-agent/src/agent_connection.rs` — Agent heartbeat with container metrics
- `api/src/infrastructure/factories/simple_executor_factory.rs` — Executor factory (line 59: "agent" → AgentServerExecutor)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- AgentServerExecutor.collect_metrics() sudah ada di agent_server_executor.rs:244-282
- ContainerStatus with metrics fields sudah ditambahkan di node_protocol.rs
- heartbeat cache sudah di node_connection_manager.rs

### Integration Points
- Server.executor_type field di database
- MonitoringService menggunakan factory untuk get executor, lalu memanggil collect_metrics()

</code_context>

<specifics>
## Specific Ideas

- Server 'test5' (id: 3f12503d-d797-4303-aa78-b566e4c48e0b) akan diupdate executor_type-nya ke 'agent'

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 30-pakai-agent-executor-untuk-mengambil-metrics-dengan-benar*
*Context gathered: 2026-04-21*
