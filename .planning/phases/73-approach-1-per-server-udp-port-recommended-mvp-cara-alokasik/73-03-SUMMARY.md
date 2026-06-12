# 73-03 SUMMARY — Gateway UdpSocket + Bedrock Forwarding Loop

**Status:** ✅ Complete

## Tasks

### Task 1: Create `opt/relay/src/udp.rs` with UdpPortRegistry
- Created `UdpPortRegistry` with `DashMap<u16, UdpPortEntry>` for concurrent port management
- `bind_port()` — binds `UdpSocket` on port, inserts into registry
- `start_grace_period()` — spawns tokio task that frees port after configurable duration (default 30s)
- `cancel_grace()` — aborts grace task on reconnect
- `free_port()` — immediate teardown (no grace, for deleted servers)
- `spawn_session()` — spawns per-port `run_udp_player_session` forwarding loop
- `abort_session()` — kills the session task
- `run_udp_player_session()` — deferred session: waits for first datagram, opens yamux stream, forwards bidirectionally with TLV framing

### Task 2: Config + State + Module Wiring
- Added `UdpConfig` to `config.rs` with `port_start` (19132), `port_end` (19231), `grace_period_secs` (30)
- Added `pub udp_registry: UdpPortRegistry` to `AppState` in `state.rs`
- Added `mod udp;` to `main.rs`
- Added `[workspace]` to relay's `Cargo.toml` to avoid workspace conflict with parent solys repo

### Task 3: Loader Field + Bedrock Dispatch in tunnel.rs
- Added `pub loader: Option<String>` to `TunnelConnect` with `#[serde(default)]`
- After authorize step: `is_bedrock` check → `state.udp_registry.bind_port()`
- Bedrock servers skip subdomain registry registration (insert into `by_server_id` only)
- After handle creation: `state.udp_registry.spawn_session()` for Bedrock
- On cleanup: Bedrock calls `abort_session()` + `start_grace_period()` instead of `registry.unregister()`
- `TunnelHandle` gets `udp_port: Option<u16>` field for Bedrock disconnect detection
- `TunnelDisconnect` handler checks `udp_port` → uses grace period for Bedrock

## Verification
- `cargo check` passes for `opt/relay` (no errors, only pre-existing warnings)
- `cargo check` passes for `agent/solys` (no errors)
- `cargo check` passes for `api` (no errors)
