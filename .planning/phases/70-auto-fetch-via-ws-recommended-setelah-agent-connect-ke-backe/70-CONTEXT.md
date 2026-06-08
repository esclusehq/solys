# Phase 70: Auto-fetch relay config via WS (recommended) - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning

<domain>
## Phase Boundary

After agent authenticates to backend via WebSocket with `AGENT_API_KEY`, backend pushes `relay_token` + per-server relay config directly through the same WS connection — no manual env var configuration needed.

**Before Phase 70:** Agent requires `AGENT_RELAY_TOKEN`, `AGENT_RELAY_SERVER_ID`, `AGENT_RELAY_SUBDOMAIN`, etc. as env vars (manual out-of-band setup).

**After Phase 70:** Agent ships with zero relay env vars. User inputs only `AGENT_API_KEY`. Backend sends all relay configuration through WS after registration.

**Out of scope:**
- Multi-region relay failover (deferred from Phase 68)
- Bedrock Edition UDP support (architecturally different)
- Relay gateway itself (Phase 68-69 already deploy it)
- Changes to relay tunnel protocol (yamux/WS bridge stays as-is)
</domain>

<decisions>
## Implementation Decisions

### Agent State Management
- **D-01 (Config storage split):** Split into two sources:
  - **GlobalRelayConfig** (`OnceCell`) — immutable, from env/TOML: `gateway_url`, `region`, `dns_api_token`, `dns_zone_id`. Rarely changes.
  - **RelaySessionState** (`RwLock`) — dynamic, from WS push: `relay_token` + `servers: Vec<ServerRelayConfig>`. Replaced atomically on every push.
  - After Phase 70, `AGENT_RELAY_TOKEN` and `AGENT_RELAY_SERVER_ID` env vars are no longer needed.
  - **Backward compat:** If `AGENT_RELAY_TOKEN` still set, use as fallback (agent can operate without WS push).
- **D-02 (Replace semantics):** Full state replace on every `RelayConfigSync` push. Agent replaces entire `RelaySessionState.servers` vec — no incremental diff on the storage layer.
- **D-03 (Startup flow):** Wait for WS push. If no `AGENT_RELAY_TOKEN` env var, `relay_client` does not start at bootstrap. It starts only after first `RelayConfigSync` arrives from backend via WS. For backward compat: if env var is set, start immediately (existing behavior).
- **D-04 (Hot update):** Diff-based hot update. When new config arrives:
  1. Acquire write lock on `RelaySessionState`
  2. Cancel tunnels for servers no longer in the new list
  3. Start tunnels for new servers
  4. Update tunnels for existing servers if config changed (stagger jitter per Phase 69 D-17)
  - Atomic under the RwLock write guard.

### Push Timing & Message Design
- **D-05 (Push timing):** Backend sends `RelayConfigSync` immediately after `RegisterAck`, in the same Register handler flow (node_ws_handler.rs:99-299). Same code path for first connect and reconnect.
- **D-06 (Message type):** Single `NodeMessage::RelayConfigSync` variant. Not split into separate token/server messages.
- **D-07 (Message shape):**
  ```rust
  #[serde(rename = "relay_config_sync")]
  RelayConfigSync {
      relay_token: String,
      gateway_url: String,
      region: String,
      servers: Vec<ServerRelayInfo>,
  }

  ServerRelayInfo {
      server_id: Uuid,
      subdomain: String,
      local_mc_addr: String,   // "127.0.0.1:<port>"
      public_port: u16,
  }
  ```
  Backend owns authoritative `gateway_url` and `region` — can override env defaults.

### Server Lifecycle Sync
- **D-08 (Server create while connected):** Fresh `RelayConfigSync` (full state replace). Agent diffs and starts tunnel for the new server. Same message type as initial config — no separate `relay.connect` task for this path.
- **D-09 (Server delete while connected):** Existing `relay.disconnect` task (Phase 69 D-08). Targeted, proven. No full sync needed for deletes.

### Reconnection & Error Handling
- **D-10 (Reconnect flow):** Same as initial connect — Register handler → `RegisterAck` → `RelayConfigSync`. No separate reconnect detection needed. The Register handler already handles re-registration (node_id lookup in lines 115-199 of node_ws_handler.rs).
- **D-11 (Push failure):** Non-critical — log and retry on next WS connect. Agent with existing config continues working. No retry loop, no blocking. If agent has no config (fresh install), `relay_client` simply doesn't start until config arrives.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and prior roadmap context
- `.planning/ROADMAP.md` § Phase 70 — Goal: "Auto-fetch via WS — setelah agent connect ke backend pakai AGENT_API_KEY, backend kirim relay_token + server_ids langsung via WebSocket"
- `.planning/STATE.md` — Project state and progress

### Phase 69 carryforward (RelayClient per-server architecture)
- `.planning/phases/69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv/69-CONTEXT.md` — D-01 (HashMap storage), D-02 (add/remove via task dispatch), D-03 (PerServerRuntime struct), D-05/D-06/D-07/D-08 (task dispatch with server_id), D-13/D-14 (config delivery in task payload, no local persistence), D-17 (heartbeat staggering)
- `.planning/phases/69-multiple-relayclient-instances-satu-per-server-dan-tiap-serv/69-CONTEXT.md` § `<canonical_refs>` — Full file-level references to relay_client.rs, relay_session.rs, relay.rs, state.rs, opt/relay/src/auth.rs, opt/relay/src/tunnel.rs

### Phase 68 carryforward (relay infrastructure fundamentals)
- `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-CONTEXT.md` — D-09 (relay_token on nodes table), D-10 (server_id ownership validation), D-04 (tunnel reconnection), D-09/D-10/D-11 (auth layer)
- `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-CONTEXT.md` § `<canonical_refs>` — Full file-level references to node_protocol.rs, node_ws_handler.rs, relay_client.rs, state.rs, node.rs

### Backend WS protocol (integration points)
- `api/src/presentation/ws/node_protocol.rs` — NodeMessage enum. Phase 70 adds `RelayConfigSync` variant (lines 231-233 slot, after `TunnelCloseAck`)
- `api/src/presentation/handlers/node_ws_handler.rs:99-299` — Register handler. Phase 70 adds `RelayConfigSync` push after line 298 (after DNS config replay)
- `api/src/domain/entities/node.rs:50-53` — `relay_token: Option<Uuid>` on Node struct. Backend reads this to populate `RelayConfigSync.relay_token`

### Agent relay code (to be modified)
- `src/state.rs:137-186` — `RelayConfig` struct (OnceCell). Phase 70 splits into `GlobalRelayConfig` (OnceCell) + `RelaySessionState` (RwLock)
- `src/main.rs:397-426` — `bootstrap_relay_client()` reading env vars. Phase 70 makes this conditional (skip if no AGENT_RELAY_TOKEN)
- `src/handlers/relay_client.rs` — Per-server tunnel lifecycle. Phase 70 adds `apply_relay_config()` that diffs and hot-updates running tunnels
- `src/handlers/mod.rs:118-166` — Task dispatch. Phase 70 may add a handler for `NodeMessage::RelayConfigSync` from backend WS

### Codebase maps (tech context)
- `.planning/codebase/STACK.md` — Tech stack: Rust 2021, tokio 1, yamux 0.13, tokio-tungstenite 0.26
- `.planning/codebase/STRUCTURE.md` — Directory layout
- `.planning/codebase/ARCHITECTURE.md` — Microservices: agent → backend WS (control) + agent → relay WS (data)
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`node_ws_handler.rs:248-298`** — DNS config replay pattern: after RegisterAck, backend reads persisted config and pushes to agent. Phase 70 reuses this exact pattern for `RelayConfigSync`.
- **`NodeMessage::DnsConfig`** (node_protocol.rs:138-156) — Existing full-state-push message. Phase 70's `RelayConfigSync` follows the same shape: backend-pushed, full state replace.
- **`RelayConfig` struct** (state.rs:137-172) — Current env-var based config. Phase 70 splits: `gateway_url`, `region`, `dns_` fields stay in `OnceCell<GlobalRelayConfig>`; `token`, `server_id`, `subdomain`, `public_port`, `local_mc_addr`, `agent_public_ip` move to `RwLock<RelaySessionState>`.
- **`PerServerRuntime`** (relay_client.rs, Phase 69) — Per-server tunnel struct. Phase 70's `apply_relay_config()` uses this to diff and hot-update active tunnels.

### Established Patterns
- **Full-state push after Register** — Already proven by the DNS config replay (node_ws_handler.rs:248-298). Phase 70 extends the same post-registration window with a relay config push.
- **Diff-based hot update** — Phase 69 D-06 already defines "replace existing tunnel atomically" semantics. Phase 70 extends to batch diff.
- **OnceCell for immutable config, RwLock for mutable state** — Standard Rust pattern throughout the codebase.

### Integration Points
- **Add `RelayConfigSync` variant** to `NodeMessage` enum in `node_protocol.rs` (after line 233, after `TunnelCloseAck`)
- **Add handler case** in `node_ws_handler.rs` Register block (after line 298, after DNS config replay): read `relay_token` from node entity, collect server relay configs, serialize, push via `manager.send_to_node()`
- **Modify `state.rs`**: Keep `RELAY_CONFIG` OnceCell for global fields. Add `RELAY_SESSION_STATE: RwLock<Option<RelaySessionState>>`
- **Modify `relay_client.rs`**: Add `apply_relay_config()` function that diffs current tunnels against new config, starts/stops/updates per-server tunnels under RwLock write guard
- **Modify `main.rs:397-426`**: Make `bootstrap_relay_client()` conditional: if no `AGENT_RELAY_TOKEN` env var, skip startup (relay starts when WS push arrives)
</code_context>

<specifics>
## Specific Ideas

### Natural split from user discussion
```
Sumber           Data                                    Storage
Env var / TOML   gateway_url, region, dns_api_token,      OnceCell<GlobalRelayConfig>
                 dns_zone_id
WS push          relay_token (per-node) +                 RwLock<RelaySessionState>
                 daftar (server_id, subdomain,
                 local_mc_addr) per-server
```

After Phase 70, `AGENT_RELAY_TOKEN` dan `AGENT_RELAY_SERVER_ID` tidak perlu lagi — itu yang di-fetch via WS. Global config tetap dari env (jarang berubah), per-server config dari WS (dinamis).

### Register + RelayConfigSync flow
```
Agent connects to backend WS with AGENT_API_KEY
  ↓
Backend validates API key, sets authenticated_node_id
  ↓
Agent sends Register message (name, ip, capabilities, ...)
  ↓
Backend processes Register (lines 99-299):
  → Find/create node
  → Update node info
  → Send RegisterAck
  → Replay DNS config (existing)
  → [NEW] Push RelayConfigSync:
      relay_token (from node.relay_token)
      gateway_url (from env or default)
      region (from env or default)
      servers: [{server_id, subdomain, local_mc_addr, public_port}, ...]
  ↓
Agent receives RelayConfigSync:
  → Write lock RelaySessionState
  → Diff against current running tunnels
  → Cancel removed, start new, update existing
  → Release lock
```

### RelayConfigSync message shape
```rust
#[serde(rename = "relay_config_sync")]
RelayConfigSync {
    relay_token: String,
    gateway_url: String,
    region: String,
    servers: Vec<ServerRelayInfo>,
}
```

No separate message types for different config categories — single message, full state replace.
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. Phase 70 is a focused enhancement of Phase 68-69's config delivery mechanism.
</deferred>

---

*Phase: 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe*
*Context gathered: 2026-06-09*
