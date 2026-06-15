---
phase: 72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te
reviewed: 2026-06-12T14:30:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - api/migrations/20260612000001_add_bedrock_game_type.sql
  - api/src/domain/server/entities/game_type.rs
  - api/src/application/use_cases/create_server_use_case.rs
  - api/src/infrastructure/executors/agent_server_executor.rs
  - api/src/presentation/handlers/server_handlers.rs
  - src/agent_connection.rs
  - src/handlers/runtime.rs
  - app/src/features/server/CreateServerModal.jsx
findings:
  critical: 3
  warning: 4
  info: 3
  total: 10
status: issues_found
---

# Phase 72: Code Review Report — Bedrock Game Type

**Reviewed:** 2026-06-12T14:30:00Z  
**Depth:** standard  
**Files Reviewed:** 8  
**Status:** issues_found  

## Summary

This phase adds support for Minecraft Bedrock Edition servers. While the migration, entity, use-case, and frontend changes are generally sound, there are **three critical data-flow bugs** in the agent communication pipeline that silently break Bedrock (and potentially Java) server provisioning. The core problem is a **field-name mismatch in the `DeployConfig` struct** between the backend and the agent, causing environment variables (EULA, GAMEMODE, DIFFICULTY, MEMORY) to be silently dropped in transit. Additionally, the frontend-only handlers (`start_server`, `wake_server`) in `server_handlers.rs` hardcode `loader="PAPER"` and always send an RCON port, which causes Bedrock containers to use TCP instead of UDP and receive unnecessary port mappings.

The agent-side `runtime.rs` also reads env vars from the wrong JSON key (`env_vars` instead of `env`), creating a second break point even if the first mismatch were fixed.

---

## Critical Issues

### CR-01: DeployConfig field name mismatch — env_vars vs env (agent-side struct) {#cr-01}

**File:** `src/agent_connection.rs:251-271`  
**Related:** `agent_server_executor.rs:83` → `node_protocol.rs:367` → `agent_connection.rs:773`

**Issue:**  
The backend serializes `DeployConfig` with field name `env_vars` (defined in `api/src/presentation/ws/node_protocol.rs:367`):
```rust
pub struct DeployConfig {
    // ...
    pub env_vars: HashMap<String, String>,
    // ...
}
```

But the agent deserializes it with field name `env` (defined in `src/agent_connection.rs:264`):
```rust
pub struct DeployConfig {
    // ...
    pub env: Option<HashMap<String, String>>,
    // ...
}
```

Since serde uses the Rust field name (no `#[serde(rename)]`), the JSON field `env_vars` sent by the backend does **not** match the agent's `env` field. All environment variables are silently dropped during deserialization with no error or warning.

**Impact:**  
The EULA=TRUE variable (required for any Minecraft server to start), MEMORY, GAMEMODE, and DIFFICULTY environment variables never reach the container. This breaks:
- All Java servers (no EULA acceptance → container refuses to start)
- All Bedrock servers (no EULA, GAMEMODE, or DIFFICULTY)
- Memory limits not applied via env

**Fix:**  
Rename the field on the agent side to `env_vars` to match the backend, and change the type to `HashMap<String, String>` (non-optional) to match:

```rust
// src/agent_connection.rs
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DeployConfig {
    pub image: String,
    #[serde(default)]
    pub game_port: Option<u16>,
    #[serde(default)]
    pub rcon_port: Option<u16>,
    #[serde(default)]
    pub ram_mb: Option<u32>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,  // was: env: Option<HashMap<String, String>>
    #[serde(default)]
    pub volume_path: Option<String>,
    #[serde(default)]
    pub memory_limit: Option<u64>,
    #[serde(default)]
    pub cpu_limit: Option<u64>,
}
```

---

### CR-02: payload key mismatch — `payload["env"]` vs `payload["env_vars"]` in agent runtime {#cr-02}

**File:** `src/agent_connection.rs:773-774` → `src/handlers/runtime.rs:259`

**Issue:**  
Even if CR-01 were fixed, there is a **second** mismatch in the agent's own code. The agent connection code writes env vars into `payload["env"]` (line 773-774):

```rust
// agent_connection.rs:773
if let Some(env) = config.env {
    payload["env"] = serde_json::json!(env);
}
```

But the `handle_start` function in `runtime.rs` reads from `payload["env_vars"]` (line 259):

```rust
// runtime.rs:259
if let Some(env_obj) = payload.get("env_vars") {
```

These two keys **must match**, otherwise environment variables are dropped at the second hop even if CR-01 is fixed.

**Impact:**  
Environment variables (EULA, MEMORY, etc.) never reach the container during `create+start` flow (`handle_start` without `container_id`). Containers start without EULA acceptance → Minecraft server fails to boot.

**Fix:**  
Change the agent connection code to write to `"env_vars"` instead of `"env"`:

```rust
// agent_connection.rs:773-774
if let Some(env) = config.env_vars {  // name change if CR-01 is applied
    payload["env_vars"] = serde_json::json!(env);
}
```

---

### CR-03: Hardcoded loader="PAPER" in start/wake handlers breaks Bedrock protocol {#cr-03}

**File:** `api/src/presentation/handlers/server_handlers.rs:893, 1370`

**Issue:**  
The `start_server` and `wake_server` handlers hardcode `loader: Some("PAPER".to_string())` when building the `DeployConfig` for agent-mode servers:

```rust
// Line 888-896
let deploy_config = crate::presentation::ws::node_protocol::DeployConfig {
    image: server.image.clone(),
    game_port: Some(server.port.unwrap_or(25565) as u16),
    rcon_port: Some((server.port.unwrap_or(25565) + 10) as u16),
    ram_mb: Some(ram_mb),
    version: Some(mc_version),
    loader: Some("PAPER".to_string()),   // ← always PAPER, even for Bedrock
    env_vars,
    volume_path: Some("/data".to_string()),
};
```

The agent's `handle_start` function in `runtime.rs` uses the loader to determine the network protocol:

```rust
// runtime.rs:279-283
let is_bedrock = payload.get("loader")
    .and_then(|v| v.as_str())
    .map(|l| l.eq_ignore_ascii_case("bedrock"))
    .unwrap_or(false);
let protocol = if is_bedrock { "udp" } else { "tcp" };
```

Since the handler always sends `"PAPER"`, `is_bedrock` is always `false`, and the port binding uses `tcp` instead of `udp`. Bedrock servers **require UDP** and will fail to accept connections over TCP.

Additionally, `rcon_port` is always set (even for Bedrock), though the `agent_server_executor.rs` `build_deploy_config` correctly sets it to `None` for Bedrock. The handlers bypass the executor's logic.

**Impact:**  
Bedrock servers started via the `POST /:id/start` or `POST /:id/wake` endpoints will have TCP port mappings instead of UDP. Clients cannot connect.

**Fix:**  
Check the server's game type or `mc_loader` before setting the loader:

```rust
// server_handlers.rs ~line 892-893
let is_bedrock = server.config
    .get("game_type")
    .and_then(|v| v.as_str())
    .map(|g| g == "bedrock" || g == "minecraft-bedrock")
    .unwrap_or(false);

let deploy_config = DeployConfig {
    image: server.image.clone(),
    game_port: Some(server.port.unwrap_or(if is_bedrock { 19132 } else { 25565 }) as u16),
    rcon_port: if is_bedrock { None } else { Some((server.port.unwrap_or(25565) + 10) as u16) },
    ram_mb: Some(ram_mb),
    version: Some(mc_version),
    loader: Some(if is_bedrock { "bedrock".to_string() } else { "PAPER".to_string() }),
    env_vars,
    volume_path: Some("/data".to_string()),
};
```

---

## Warnings

### WR-01: Memory leak in `check_status` via `Box::leak` {#wr-01}

**File:** `api/src/infrastructure/executors/agent_server_executor.rs:267-271`

**Issue:**  
When `server.container_name` is `None`, the fallback creates a `Box<str>` and leaks it:

```rust
let container_name = if let Some(cn) = server.container_name.as_ref() {
    cn.trim_start_matches('/')
} else {
    let fallback = format!("mc-{}", server.id);
    Box::leak(fallback.into_boxed_str())  // <-- memory leak
};
```

This function (`check_status`) is called every time server status is polled (frequently). The leaked memory (`&'static str`) accumulates with each call for servers without a `container_name` set. Over time this can exhaust memory.

**Fix:**  
Return a `String` from an `Option`-based approach instead of leaking:

```rust
let container_name = server.container_name.as_ref()
    .map(|cn| cn.trim_start_matches('/').to_string())
    .unwrap_or_else(|| format!("mc-{}", server.id));
```

(Note: `container_name` would need to change from `&str` to `String` in the usage below, or use `Cow<'_, str>`.)

---

### WR-02: Missing Bedrock port handling in server_handlers.rs start/wake — always sends rcon_port {#wr-02}

**File:** `api/src/presentation/handlers/server_handlers.rs:890, 1367`

**Issue:**  
The `start_server` and `wake_server` handlers always compute an RCON port:

```rust
rcon_port: Some((server.port.unwrap_or(25565) + 10) as u16),
```

Unlike the `agent_server_executor.rs` `build_deploy_config` (line 74) which correctly sets `rcon_port` to `None` for Bedrock servers, these handlers always send an RCON port. While less severe than CR-03, this creates unnecessary port mappings for Bedrock containers and may cause port conflicts.

**Fix:**  
Apply the same Bedrock check as WR-01 above and conditionally set `rcon_port: None` for Bedrock.

---

### WR-03: Bedrock-specific fields not mapped to deploy config in start/wake handlers {#wr-03}

**File:** `api/src/presentation/handlers/server_handlers.rs:884-896, 1360-1373`

**Issue:**  
The `agent_server_executor.rs` `build_deploy_config` function (line 78-81) adds Bedrock-specific env vars for servers with `mc_loader == "bedrock"`:

```rust
if is_bedrock {
    env_vars.insert("GAMEMODE".to_string(), "survival".to_string());
    env_vars.insert("DIFFICULTY".to_string(), "normal".to_string());
}
```

But the `start_server` and `wake_server` handlers in `server_handlers.rs` build their own `DeployConfig` with only `EULA` and `MEMORY` variables, bypassing the executor's logic entirely. Bedrock servers started via these handlers will lack `GAMEMODE` and `DIFFICULTY` env vars.

**Fix:**  
Either (a) route through `AgentServerExecutor::build_deploy_config` instead of building `DeployConfig` inline, or (b) replicate the executors logic for bedrock env vars in the handler.

---

### WR-04: Port validation allows ports below 10000 for Bedrock after initialization {#wr-04}

**File:** `app/src/features/server/CreateServerModal.jsx:349-362`

**Issue:**  
The `validatePort` function requires ports between 10000 and 30000. The Bedrock default port is `19132` (set via `useEffect` on line 170), which falls in the valid range so this specific case works. However, the Bedrock section shows a hint "Bedrock servers use UDP port 19132 by default" (line 757) while the actual validation message says "Port must be between 10000 and 30000" regardless of game type. The port input `min`/`max` attributes (lines 561, 753) also show the generic 10000-30000 range.

While this doesn't cause a bug with the default port, the UX is misleading for Bedrock — the hint suggests 19132 is standard, but validation allows any port 10000-30000. A user who changes the port to a "standard" Minecraft port like 25565 would be rejected (below 10000). More importantly, there is **no validation** that the port is not already in use for a different server's UDP port allocation.

**Fix:**  
Consider adding a game-type-aware minimum port or at least updating the validation error message per game type. For Bedrock, the hint should be clearer about the allowed range.

---

## Info

### IN-01: Debug console.log calls in production UI code {#in-01}

**File:** `app/src/features/server/CreateServerModal.jsx:181, 183, 194, 197, 217, 219, 237, 257, 259, 269, 287, 307, 308, 320`

**Issue:**  
Multiple `console.log` and `console.error` calls are left in the production component. While not a functional bug, these expose internal state and server data to browser consoles. Several log calls also include `[CreateServerModal]` prefixes suggesting debugging artifacts.

**Fix:**  
Remove or gate console.log calls behind a debug flag. For essential logging, use a proper logging utility that can be disabled in production.

---

### IN-02: Unused variable binding `_node_id` in handler functions {#in-02}

**File:** `api/src/presentation/handlers/server_handlers.rs:807, 1304`

**Issue:**  
The `start_server` and `wake_server` handlers bind `let _node_id = server.node_id;` which is never used — the node_id is resolved again from scratch in the agent branch:

```rust
let _node_id = server.node_id;  // line 807 — unused
```

**Fix:**  
Remove the unused variable bindings.

---

### IN-03: Duplicate shutdown check in reconnect loop {#in-03}

**File:** `src/agent_connection.rs:1005-1017`

**Issue:**  
The shutdown flag is checked twice consecutively before sleeping during reconnection:

```rust
// Line 1005-1009
if shutdown.load(Ordering::Relaxed) {
    info!("Shutdown requested, exiting");
    break;
}

// ...

// Line 1013-1017
if shutdown.load(Ordering::Relaxed) {
    info!("Shutdown requested, exiting reconnect loop");
    break;
}
```

The second check (lines 1013-1017) is redundant since no significant time passes between the two checks. The check before the `tokio::time::sleep` (lines 1005-1009) is sufficient.

**Fix:**  
Remove the second redundant check at lines 1013-1017.

---

## Notes

- The **migration** (`add_bedrock_game_type.sql`) correctly inserts the Bedrock game type with the right Docker image, UDP port (`19132`), and environment defaults. No issues found.
- The **`game_type.rs`** entity correctly adds the bedrock variant in `fallback()` with UDP port 19132. No issues found.
- The **`create_server_use_case.rs`** correctly forces `mc_loader="bedrock"` for Bedrock game type (line 48-49). This is the single source of truth that is then overridden by the handlers in `server_handlers.rs`.
- The **`agent_server_executor.rs`** `build_deploy_config` has correct Bedrock logic — this function is used during initial `create_server` and `start_server` through `send_command_with_config`. The handlers in `server_handlers.rs` duplicate and break this logic.
- The **frontend `CreateServerModal.jsx`** correctly separates Minecraft Java and Bedrock UI sections, hides version/loader/JVM options for Bedrock, and sets the default port to 19132. Good UX work overall.

---

_Reviewed: 2026-06-12T14:30:00Z_  
_Reviewer: gsd-code-reviewer agent_  
_Depth: standard_
