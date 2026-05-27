# Phase 40: BACKEND ↔ AGENT STABILITY - Research

**Researched:** 2026-05-03
**Status:** Complete

## Domain Context

Phase 40 focuses on backend handling of WebSocket agent connections to eliminate "node not connected" issues. This builds on Phase 39 (agent hardening) by implementing the server-side logic.

## Existing Codebase Analysis

### 1. NodeConnectionManager (`api/src/presentation/ws/node_connection_manager.rs`)

**Current functionality:**
- Stores connections in `Arc<RwLock<HashMap<Uuid, NodeSender>>>`
- `is_connected(node_id)` checks if sender exists in map
- `add_connection()` and `remove_connection()` manage WebSocket channels
- `get_sender()` with sender closed detection and cleanup
- Command queuing when node is offline

**Pattern to extend:** Already has `is_connected()` — can add `connection_health(node_id)` that returns detailed status.

### 2. NodeHealthService (`api/src/application/services/node_health_service.rs`)

**Current functionality:**
- `check_node_health()` evaluates: is_online, connection_health, metrics_health
- `evaluate()` method on NodeHealth determines status
- Uses `last_seen` from node database

**Pattern to extend:** Add configurable interval check, add DEGRADED status evaluation.

### 3. NodeHealth Entity (`api/src/domain/entities/node_health.rs`)

**Current status enum:**
```rust
pub enum NodeHealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}
```

**Current evaluation:**
- Critical if offline or heartbeat >120s
- Warning if heartbeat >60s
- Checks connection_health and metrics_health

**Required changes:** Extend to include OFFLINE and DEGRADED per D-06.

### 4. Node Entity (`api/src/domain/entities/node.rs`)

**Relevant fields:**
- `status`: String ("online", "offline")
- `last_seen`: Option<DateTime<Utc>>
- `metadata`: JSONB for custom settings

**Pattern:** Use `metadata` to store `heartbeat_interval` per node (D-01).

### 5. WebSocket Handler (`api/src/presentation/handlers/node_ws_handler.rs`)

**On heartbeat receipt:**
- Updates `last_seen` via `node_repository.update_last_seen()`
- Stores container status from heartbeat payload
- Stores metrics from heartbeat

**Pattern to extend:** Handle heartbeat interval from agent, calculate delays.

### 6. MonitoringService (`api/src/application/services/monitoring_service.rs`)

**Current loop:**
- `check_all_servers()` iterates all servers
- No check for node status before polling

**Required changes:** Add node status check, skip offline nodes per D-07.

## Implementation Patterns

### Heartbeat Interval Pattern (D-01, D-02)

```rust
// In node.metadata JSONB:
// {"heartbeat_interval": 10}

// Default: 10 seconds
fn get_heartbeat_interval(node: &Node) -> u32 {
    node.metadata
        .get("heartbeat_interval")
        .and_then(|v| v.as_i64())
        .map(|v| v as u32)
        .unwrap_or(10)
}
```

### OFFLINE Detection Pattern (D-04)

Threshold: 3x interval (30s for default 10s)

```rust
fn is_offline(node: &Node, interval_seconds: u32) -> bool {
    if let Some(last_seen) = node.last_seen {
        let age = Utc::now().signed_duration_since(last_seen).num_seconds();
        age > (interval_seconds * 3) as i64
    } else {
        true
    }
}
```

### DEGRADED Detection Pattern (D-05)

Triggers:
- Heartbeat late (>50% interval): e.g., >5s for 10s interval
- Metrics stale (>2x interval)
- High CPU/RAM threshold (configurable)
- Reconnecting attempts active

### Server Monitoring Skip Pattern (D-07)

```rust
// In MonitoringService::check_all_servers
async fn check_all_servers(&self) -> Result<()> {
    let nodes = self.node_repository.list().await?;
    
    for node in nodes {
        // Skip offline nodes per D-07
        if node.status == "offline" {
            tracing::debug!("Skipping servers on offline node {}", node.id);
            continue;
        }
        // Check server status...
    }
}
```

### Reconnection Sync Pattern (D-10)

When agent reconnects:
1. ConnectionManager adds new sender
2. State sync: send cached commands from queue
3. Resume monitoring for servers on node

## Technical Decisions

### 1. Status Enum Strategy

**Option A:** Extend existing `NodeHealthStatus`
- Pros: Single source of truth
- Cons: Breaks existing consumers expecting 4 states

**Option B:** Create separate NodeConnectionStatus for connection
- Pros: Clean separation
- Cons: Duplication

**Recommendation:** Option A - extend but add new variants:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeHealthStatus {
    Online,      // NEW: Connected and healthy
    Offline,     // NEW: No heartbeat (3x interval)
    Degraded,    // NEW: Late/stale/high load
    Healthy,     // Existing - kept for backward compat
    Warning,
    Critical,
    Unknown,
}
```

Or simpler per D-06: `ONLINE, OFFLINE, DEGRADED`

### 2. Heartbeat Interval Storage

- Per-node setting in `node.metadata` JSONB
- Default 10 seconds
- Agent sends interval in heartbeat payload (optional override)

### 3. Offline Detection

- Use `last_seen` timestamp from database
- Calculate age: `now - last_seen`
- Compare against threshold: `interval * 3`

### 4. Monitoring Skip

- In MonitoringService loop, check node status before checking servers
- Don't mark offline nodes as unhealthy (that's expected state)

## Validation Architecture

### Test Cases

1. **Heartbeat received**: Update last_seen, set status to online
2. **No heartbeat 3x interval**: Mark node offline
3. **Heartbeat late (>50%)**: Mark node degraded
4. **Node offline + server check**: Skip, don't count as error
5. **Agent reconnect**: Resume monitoring, sync commands

### Acceptance Criteria (what must be TRUE)

1. Node shows OFFLINE after 30s without heartbeat (default 10s interval)
2. Node shows DEGRADED when heartbeat arrives >5s late
3. MonitoringService skips servers on offline nodes without errors
4. Node returns to ONLINE on successful reconnect
5. Command queue syncs on reconnection

## Integration Points

| Component | File | Required Changes |
|-----------|------|------------------|
| NodeConnectionManager | `node_connection_manager.rs` | Add connection status tracking |
| NodeHealthService | `node_health_service.rs` | Add interval config, degraded logic |
| NodeHealth | `node_health.rs` | Add OFFLINE/DEGRADED states |
| MonitoringService | `monitoring_service.rs` | Add node status skip |
| Node entity | `node.rs` | Add metadata helper |
| NodeWsHandler | `node_ws_handler.rs` | Handle interval, sync |

## Common Pitfalls

1. **Race condition on reconnect**: Must clear old sender before adding new
2. **Command duplication**: Queue commands sent during offline
3. **Status stuck**: Reset degraded after successful heartbeat
4. **Monitoring skip false positive**: Only skip when node explicitly offline

---

*Research complete: 2026-05-03*