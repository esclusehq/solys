# Phase 69: Multiple RelayClient instances (satu per server) dan Tiap server butuh subdomain unik biar gateway bisa route lewat Handshake parser - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-09
**Phase:** 69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv
**Areas discussed:** Per-server tunnel instantiation, Task dispatch with server_id, Subdomain generation strategy, Per-server tunnel config source, Resource isolation & limits, Gateway routing evolution

---

## Per-server tunnel instantiation

| Option | Description | Selected |
|--------|-------------|----------|
| HashMap (Mutex) | Arc<Mutex<HashMap<ServerId, PerServerRuntime>>> — simple, proven Rust pattern | |
| RwLock<HashMap> | tokio::sync::RwLock<HashMap> — concurrent reads, exclusive writes | ✓ |
| Actor with channel | Actor pattern (tokio::spawn + channel) — cleaner isolation, more boilerplate | |

**User's choice:** RwLock<HashMap>
**Notes:** Concurrent reads for heartbeats and player streams, exclusive writes for add/remove.

| Option | Description | Selected |
|--------|-------------|----------|
| Via task dispatch | Backend pushes relay.connect/relay.disconnect tasks with server_id | ✓ |
| Agent-side reconciliation | Agent watches server list changes and reconciles | |
| Local state file | Agent reads from local file or DB | |

**User's choice:** Via task dispatch
**Notes:** Matches existing Phase 68 task dispatch pattern.

| Option | Description | Selected |
|--------|-------------|----------|
| Full per-server struct | All fields per-server: CancelToken, JoinHandle, ControlTx, BytesCounter, TunnelStart, ServerRelayConfig | ✓ |
| Hybrid: global + dispatch | Keep global Runtime, add per-server dispatch layer | |

**User's choice:** Full per-server struct

| Option | Description | Selected |
|--------|-------------|----------|
| Drop map → cascade | Drop HashMap, which drops all CancelTokens, aborts all loops | |
| Explicit per-server cancel | Iterate and cancel each token individually | ✓ (modified) |

**User's choice:** Parent CancellationToken with child_token() per server. parent.cancel() cascades to all children, then clear HashMap. Tokio's CancellationToken.drop() does NOT auto-cancel — it only decrements ref count. Explicit parent.cancel() required.

---

## Task dispatch with server_id

| Option | Description | Selected |
|--------|-------------|----------|
| server_id in task.payload | Current task.payload is opaque JSON — add server_id | ✓ |
| Separate task types | New task types per server: relay.connect.{server_id} | |
| Dedicated header field | Backend sends server_id in a separate task.headers field | |

**User's choice:** server_id in task.payload

| Option | Description | Selected |
|--------|-------------|----------|
| Replace existing | Cancel old tunnel, start new one (Phase 68 D-21) | ✓ |
| Idempotent (no-op) | Keep existing tunnel | |
| Return error | Only one task should be in-flight per server | |

**User's choice:** Replace existing

| Option | Description | Selected |
|--------|-------------|----------|
| Backend pushes all active | Backend pushes relay.connect for every server on reconnect | ✓ |
| Backend sends list, agent decides | Backend sends server list, agent decides which to connect | |
| Agent pulls from local state | Agent requests tunnels for known servers | |

**User's choice:** Backend pushes all active servers

| Option | Description | Selected |
|--------|-------------|----------|
| Backend sends disconnect | Backend sends relay.disconnect when server deleted | ✓ |
| Agent-side reconciliation cleanup | Agent detects server absence and self-cleans | |
| Let gateway time it out | Tunnel stays open until heartbeat times out (30s) | |

**User's choice:** Backend sends disconnect

---

## Subdomain generation strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Short hex hash | Short hex hash of server UUID (e.g., a3f8b) | ✓ |
| Full UUID | Full UUID — unambiguous but long | |
| Server name | Human-readable but may conflict or change | |

**User's choice:** Short hex hash

| Option | Description | Selected |
|--------|-------------|----------|
| On server create | Backend generates on server create, stores in servers.subdomain | ✓ |
| On first tunnel connect | Agent generates and reports back | |
| On gateway accept | Gateway assigns when tunnel connects | |

**User's choice:** On server create

| Option | Description | Selected |
|--------|-------------|----------|
| Backend creates DNS | Backend manages Route 53 records per server | |
| Agent manages DNS | Agent pushes DNS records via Route 53 API | |
| Wildcard only | Wildcard record covers all subdomains | ✓ |

**User's choice:** Wildcard *.play.esluce.net → NLB. No per-record DNS needed.

| Option | Description | Selected |
|--------|-------------|----------|
| Parse Handshake packet | Gateway parses Minecraft Handshake server address field | ✓ |
| TLS SNI field | Gateway uses TLS SNI from player's TCP connection | |
| Proxy protocol header | Uses proxy_protocol header from NLB | |

**User's choice:** Parse Handshake packet

---

## Per-server tunnel config source

| Option | Description | Selected |
|--------|-------------|----------|
| In relay.connect payload | Backend includes full config in task payload | ✓ |
| Separate config message | Backend pushes relay.config then relay.connect | |
| Local config file | Agent caches in state.json | |

**User's choice:** In relay.connect payload

| Option | Description | Selected |
|--------|-------------|----------|
| Backend push on reconnect | Agent trusts backend push on restart | |
| Cache in state.json | Agent caches for immediate re-establish | |
| No persistence — all in task | Agent stores nothing. Backend re-pushes on reconnect. | ✓ |

**User's choice:** No persistence — backend pushes on reconnect

| Option | Description | Selected |
|--------|-------------|----------|
| Global shared + per-server task | Shared fields (env) + per-server fields (task.payload) | ✓ |
| All in task payload | Backend sends everything — even shared fields | |
| Env for infra, task for per-server | Env for stable infra config, task for dynamic config | |

**User's choice:** Global shared (env) + per-server fields in task.payload

---

## Resource isolation & limits

| Option | Description | Selected |
|--------|-------------|----------|
| No limit | Let OS/memory handle it. Tens of tunnels negligible. | ✓ |
| Configurable cap | e.g., max 10 tunnels per agent | |
| Backend-driven throttling | Agent reports load, backend stops pushing | |

**User's choice:** No hard limit

| Option | Description | Selected |
|--------|-------------|----------|
| Stagger with jitter start | Random startup delay 0-10s per tunnel | ✓ |
| Aggregated heartbeat | One combined heartbeat with all server statuses | |
| No staggering | Heartbeats fire independently | |

**User's choice:** Stagger with jitter start

| Option | Description | Selected |
|--------|-------------|----------|
| Per-server accounting | Each PerServerRuntime has its own AtomicU64 | ✓ |
| Both per-server and global | Per-server + global aggregate | |
| Global only | Keep global counter, ignore per-server | |

**User's choice:** Per-server accounting

---

## Gateway routing evolution

| Option | Description | Selected |
|--------|-------------|----------|
| Direct per-tunnel routing | Each tunnel = one server, player routes directly | |
| Keep yamux multiplexing | Keep yamux within each per-server tunnel | ✓ |
| Server-specific WS path | Gateway exposes /tunnel/{server_id} per server | |

**User's choice:** Keep yamux multiplexing. Each tunnel = 1 server, but still need yamux for multiple concurrent player connections per server.

**User's architecture comparison:**
```
Phase 68 (single):         Phase 69 (multi-server):
WS per agent:      1       N (1 per server)
yamux session:     1       N (1 per server)
Handshake routing:  subdomain → server_id (same)
Player → tunnel:   yamux stream in global session   yamux stream in per-server session
```

**User's technical breakdown for gateway changes:**
- auth.rs: One relay_token must authorize multiple server_ids (all servers owned by the node)
- tunnel.rs: Handle N WS from same agent IP without conflict
- registry.rs: No change — already HashMap<ServerId, TunnelHandle>
- player.rs: No change — already subdomain → server_id

---

## the agent's Discretion

- Exact short-hex length
- Heartbeat jitter formula and startup delay distribution
- PerServerRuntime struct field ordering
- HashMap eviction policy
- Per-server tunnel state transition logging
- Gateway auth layer per-server WS handling

## Deferred Ideas

None — discussion stayed within phase scope.
