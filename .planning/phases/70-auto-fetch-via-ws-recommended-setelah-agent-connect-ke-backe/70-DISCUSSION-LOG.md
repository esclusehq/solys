# Phase 70: Auto-fetch relay config via WS - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-09
**Phase:** 70-auto-fetch-via-ws-recommended-setelah-agent-connect-ke-backe
**Areas discussed:** Agent State Management, Push Timing & Message Design, Server Lifecycle Sync, Reconnection & Error Handling

---

## Agent State Management

| Option | Description | Selected |
|--------|-------------|----------|
| RwLock (Recommended) | Replace OnceCell with RwLock<Option<RelayConfig>>. Standard async pattern. | |
| OnceCell + overlay — clean split | Env/TOML → GlobalRelayConfig (OnceCell), WS push → RelaySessionState (RwLock). Natural separation: static config from env, dynamic from WS. | ✓ |

**User's choice:** OnceCell + overlay, but clean. Natural split:
- `GlobalRelayConfig` (OnceCell): `gateway_url`, `region`, `dns_api_token`, `dns_zone_id` from env/TOML
- `RelaySessionState` (RwLock): `relay_token` + `servers: Vec<ServerRelayConfig>` from WS push
- After Phase 70, `AGENT_RELAY_TOKEN` and `AGENT_RELAY_SERVER_ID` env vars no longer needed
- Backward compat: if `AGENT_RELAY_TOKEN` still set, use as fallback

### Replace semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Replace all (Recommended) | WS push contains ALL servers. Agent replaces entire vec. Clean — no orphans. | ✓ |
| Incremental per-server | Add/update/remove individual entries. Granular but error-prone. | |
| Full state on reconnect, incremental during session | Combines both approaches. | |

**User's choice:** Replace all (full state)

### Startup flow

| Option | Description | Selected |
|--------|-------------|----------|
| Wait for WS push (Recommended) | If AGENT_RELAY_TOKEN not set, relay_client starts only after first WS push. | ✓ |
| Start in waiting state, request config | relay_client starts but enters 'waiting for config' state, sends request. | |

**User's choice:** Wait for WS push

### Hot update

| Option | Description | Selected |
|--------|-------------|----------|
| Diff-based hot update (Recommended) | Cancel removed, start new, update existing. Atomic under RwLock write. | ✓ |
| Full restart on every push | Cancel ALL, restart all with jitter. Brief connectivity loss. | |

**User's choice:** Diff-based hot update

---

## Push Timing & Message Design

### Push timing

| Option | Description | Selected |
|--------|-------------|----------|
| RelayConfigSync after RegisterAck (Recommended) | Single message immediately after RegisterAck. Simple, clear lifecycle. | ✓ |
| Separate async push with ack | Queued behind registration, agent acknowledges. More explicit handshake. | |
| On-demand request from agent | Agent fetches config. Adds round-trip delay. | |

**User's choice:** RelayConfigSync after RegisterAck

### Message design

| Option | Description | Selected |
|--------|-------------|----------|
| Single RelayConfigSync message (Recommended) | One variant: relay_token + Vec<ServerRelayInfo>. Pushed on RegisterAck and reconnect. | ✓ |
| Split: RelayToken + RelayServerList | Two messages. Separates concerns but more complex. | |
| Per-server messages (multiple) | Backend pushes one message per server. Agent accumulates. | |

**User's choice:** Single RelayConfigSync message

### Message fields

| Option | Description | Selected |
|--------|-------------|----------|
| Include gateway_url + region (Recommended) | Backend sends gateway_url + region alongside relay_token + servers. Backend can override env defaults. | ✓ |
| Minimal — only token + server list | Gateway URL and region stay in env. Lighter but less flexible. | |

**User's choice:** Include gateway_url + region

---

## Server Lifecycle Sync

### Server create while connected

| Option | Description | Selected |
|--------|-------------|----------|
| Fresh RelayConfigSync on create (Recommended) | Full state replace. Agent diffs and starts tunnel for new server. Same message type. | ✓ |
| Existing relay.connect task | More granular but two mechanisms for same thing. | |
| Sync on create, disconnect on delete | RelayConfigSync for create, relay.disconnect for delete. | |

**User's choice:** Fresh RelayConfigSync on create

### Server delete while connected

| Option | Description | Selected |
|--------|-------------|----------|
| relay.disconnect (Recommended) | Existing task type. Clean, targeted, proven since Phase 69. | ✓ |
| RelayConfigSync (full state replace) | Full sync without deleted server. Heavier. | |

**User's choice:** relay.disconnect (existing task)

---

## Reconnection & Error Handling

### Reconnect flow

| Option | Description | Selected |
|--------|-------------|----------|
| In Register handler after ack (Recommended) | Same code path as initial connect. Register → RegisterAck → RelayConfigSync. | ✓ |
| Separate replay message | Decoupled from Register. More explicit but needs reconnect tracking. | |
| Async push (fire and forget) | Agent might process other messages before config arrives. | |

**User's choice:** In Register handler after ack

### Push failure handling

| Option | Description | Selected |
|--------|-------------|----------|
| Log and retry on next connect (Recommended) | Non-critical. Agent with existing config continues working. | ✓ |
| Retry 3x, then resync on heartbeat | Retries with backoff, then triggers on heartbeat. | |
| Critical — block tunnel startup | Agent cannot start tunnels until config received. | |

**User's choice:** Log and retry on next connect

---

## Deferred Ideas

None — discussion stayed within phase scope.
