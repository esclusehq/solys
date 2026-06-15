# Phase 72: Menambahkan Type Minecraft Bedrock — Pattern Map

**Mapped:** 2026-06-12
**Files analyzed:** 7 (new/modified)
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `api/migrations/20260612000001_add_bedrock_game_type.sql` | migration | CRUD | `api/migrations/20260409000001_create_game_types_table.sql` | exact |
| `api/src/domain/server/entities/game_type.rs` | model | CRUD | Existing file (same file) — add bedrock match arm | exact |
| `api/src/infrastructure/executors/agent_server_executor.rs` | controller/service | request-response | Existing file (same file) — modify `build_deploy_config()` | exact |
| `src/handlers/runtime.rs` | handler | event-driven (agent task) | Existing file (same file) — port binding dispatch | exact |
| `src/agent_connection.rs` | utility/middleware | event-driven | Existing file (same file) — port map key | exact |
| `app/src/features/server/CreateServerModal.jsx` | component | request-response | Existing file (same file) — add bedrock options | exact |

## Pattern Assignments

### `api/migrations/20260612000001_add_bedrock_game_type.sql` (migration, CRUD)

**Analog:** `api/migrations/20260409000001_create_game_types_table.sql` (lines 21-28)

**Pattern:** SQL INSERT with `ON CONFLICT DO NOTHING` guard. Uses specific Bedrock Docker image, UDP port 19132, no RCON capability.

```sql
-- Analog pattern (lines 21-28 of 20260409000001_create_game_types_table.sql):
-- Each game type has: identifier, display_name, description, docker_image, default_ports, default_env, capabilities, sort_order
INSERT INTO game_types (identifier, display_name, description, docker_image, default_ports, default_env, capabilities, sort_order) VALUES
    ('minecraft', 'Minecraft', 'Minecraft Java Edition...', 'itzg/minecraft-server:latest', '{"game": 25565, "rcon": 25575}', '{"EULA": "TRUE", "MODE": "survival"}', '{"rcon": true, "mods": true, "backup": true}', 1),
    ...
ON CONFLICT (identifier) DO NOTHING;
```

**Target pattern:**
```sql
INSERT INTO game_types (identifier, display_name, description, docker_image, default_ports, default_env, capabilities, sort_order) VALUES
    ('bedrock', 'Minecraft Bedrock', 'Minecraft Bedrock Edition dedicated server', 'itzg/minecraft-bedrock-server:latest', '{"game": 19132}', '{"EULA": "TRUE", "GAMEMODE": "survival", "DIFFICULTY": "normal"}', '{"rcon": false, "backup": true}', 6)
ON CONFLICT (identifier) DO NOTHING;
```

---

### `api/src/domain/server/entities/game_type.rs` (model, CRUD)

**Analog:** Same file, lines 27-63 — `fallback()` method pattern-matches on identifier. Need to add `"bedrock"` arm.

**Import pattern** (lines 1-3):
```rust
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
```

**Core pattern** — fallback dispatch (lines 27-45):
```rust
impl GameType {
    pub fn fallback(identifier: &str) -> Self {
        let (image, ports) = match identifier {
            "minecraft" => (
                "itzg/minecraft-server:latest".to_string(),
                serde_json::json!({"game": 25565, "rcon": 25575}),
            ),
            "palworld" => (
                "ghcr.io/axllent/minecraft-palworld:latest".to_string(),
                serde_json::json!({"game": 8211, "rcon": 25575}),
            ),
            "valheim" => (
                "lloesche/valheim-server:latest".to_string(),
                serde_json::json!({"game": 2456, "rcon": 2457}),
            ),
            // ADD: "bedrock" arm
            _ => (
                "itzg/minecraft-server:latest".to_string(),
                serde_json::json!({"game": 25565, "rcon": 25575}),
            ),
        };
```

**Target addition** (insert before `_ =>` catch-all):
```rust
"bedrock" => (
    "itzg/minecraft-bedrock-server:latest".to_string(),
    serde_json::json!({"game": 19132}),
),
```

**Note:** Capabilities for bedrock in this function (line 57: `serde_json::json!({"rcon": true, "backup": true})`) should also handle Bedrock — but the function always uses the same capabilities JSON for all types. Changing `"rcon": true` to dynamic is optional since this is a fallback only.

---

### `api/src/infrastructure/executors/agent_server_executor.rs` (controller/service, request-response)

**Analog:** Same file, lines 67-84 — `build_deploy_config()` method. Hardcodes Java image and always sets rcon_port.

**Import pattern** (lines 1-11):
```rust
use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;
use crate::domain::entities::server::Server;
use crate::domain::entities::server_metrics::ServerMetrics;
use crate::domain::server_executor::ServerExecutor;
use crate::infrastructure::node_client::NodeClient;
use crate::presentation::ws::node_protocol::{CommandParams, DeployConfig};
```

**Core pattern** — current hardcoded Java (lines 67-84):
```rust
fn build_deploy_config(&self, server: &Server) -> DeployConfig {
    let rcon_port = (server.port + 10) as u16;
    
    DeployConfig {
        image: "docker.io/itzg/minecraft-server".to_string(),   // ❌ Hardcoded
        game_port: Some(server.port as u16),
        rcon_port: Some(rcon_port),                              // ❌ Bedrock has no RCON
        ram_mb: Some(parse_ram_allocation(&server.ram_allocation)),
        version: Some(if server.mc_version.is_empty() { "LATEST".to_string() } else { server.mc_version.clone() }),
        loader: Some(if server.mc_loader.is_empty() { "PAPER".to_string() } else { server.mc_loader.clone() }),
        env_vars: {
            let mut vars = HashMap::new();
            vars.insert("EULA".to_string(), "TRUE".to_string());
            vars
        },
        volume_path: Some(server.server_path.clone().unwrap_or_else(|| "/data".to_string())),
    }
}
```

**Target pattern** — dynamic dispatch:
```rust
fn build_deploy_config(&self, server: &Server) -> DeployConfig {
    let is_bedrock = server.mc_loader.eq_ignore_ascii_case("bedrock");
    let image = if is_bedrock {
        "docker.io/itzg/minecraft-bedrock-server"
    } else {
        "docker.io/itzg/minecraft-server"
    };
    let rcon_port = if is_bedrock { None } else { Some((server.port + 10) as u16) };
    
    let mut env_vars = HashMap::new();
    env_vars.insert("EULA".to_string(), "TRUE".to_string());
    if is_bedrock {
        env_vars.insert("GAMEMODE".to_string(), "survival".to_string());
        env_vars.insert("DIFFICULTY".to_string(), "normal".to_string());
    }
    
    DeployConfig {
        image: image.to_string(),
        game_port: Some(server.port as u16),
        rcon_port,
        ram_mb: Some(parse_ram_allocation(&server.ram_allocation)),
        version: Some(if server.mc_version.is_empty() { "LATEST".to_string() } else { server.mc_version.clone() }),
        loader: Some(if server.mc_loader.is_empty() { "PAPER".to_string() } else { server.mc_loader.clone() }),
        env_vars,
        volume_path: Some(server.server_path.clone().unwrap_or_else(|| "/data".to_string())),
    }
}
```

**Error handling pattern** (lines 43-65) — `check_node_online()` and `validate_node_connection()`:
```rust
async fn check_node_online(&self, node_id: Uuid) -> Result<()> {
    if let Some(node) = self.node_client.get_node(&node_id).await {
        if node.is_offline() {
            return Err(anyhow::anyhow!("Node {} is offline", node.name));
        }
    }
    Ok(())
}
```

---

### `src/handlers/runtime.rs` (handler, event-driven)

**Analog:** Same file — two locations with hardcoded `{}/tcp` port binding: `handle_create` (line 124) and `handle_start` (line 274).

**Import pattern** (lines 1-14):
```rust
use std::collections::HashMap;
use std::time::Duration;
use agent_proto::Task;
use agent_runtime::RuntimeDetector;
use anyhow::{Context, Result};
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions, LogsOptions};
use bollard::image::CreateImageOptions;
use bollard::models::HostConfig;
use futures_util::StreamExt;
use serde::Deserialize;
use tracing::{error, info, warn};
```

**ServerCreatePayload struct** (lines 16-29) — note `loader` field exists:
```rust
#[derive(Debug, Deserialize)]
pub struct ServerCreatePayload {
    pub server_id: uuid::Uuid,
    pub image: String,
    pub name: String,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<HashMap<String, Vec<String>>>,
    pub container_port: Option<u32>,
    pub memory_limit: Option<i64>,
    pub cpu_limit: Option<i64>,
    pub volume_path: Option<String>,
    pub version: Option<String>,
    pub loader: Option<String>,                              // ✅ Already exists
}
```

**Fix #1 — `handle_create` port binding** (lines 120-134, current hardcoded TCP):
```rust
// Current (line 124):
let port_key = format!("{}/tcp", container_port);

// Target — dynamic protocol dispatch:
let is_bedrock = payload.loader
    .as_deref()
    .map(|l| l.eq_ignore_ascii_case("bedrock"))
    .unwrap_or(false);
let protocol = if is_bedrock { "udp" } else { "tcp" };
let port_key = format!("{}/{}", container_port, protocol);
```

**Fix #2 — `handle_start` port binding** (lines 271-279, current hardcoded TCP):
```rust
// Current (line 274):
let port_key = format!("{}/tcp", container_port);

// Target — dynamic protocol dispatch (same pattern):
let is_bedrock = payload.get("loader")
    .and_then(|v| v.as_str())
    .map(|l| l.eq_ignore_ascii_case("bedrock"))
    .unwrap_or(false);
let protocol = if is_bedrock { "udp" } else { "tcp" };
let port_key = format!("{}/{}", container_port, protocol);
```

**Error handling pattern** (lines 79-105) — `ensure_image_exists()` retry:
```rust
async fn ensure_image_exists(docker: &bollard::Docker, image: &str, max_retries: u32, retry_delay_secs: u64) -> Result<()> {
    if check_image_exists(docker, image).await {
        return Ok(());
    }
    for attempt in 1..=max_retries {
        match pull_image_with_timeout(docker, image, 300).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < max_retries => {
                warn!(..., "Pull failed, retrying...");
                tokio::time::sleep(Duration::from_secs(retry_delay_secs)).await;
            }
            Err(e) => return Err(anyhow::anyhow!("...")),
        }
    }
    Err(anyhow::anyhow!("Failed to pull image after {} retries", max_retries))
}
```

**Note:** The payload for `handle_start` (line 242-266) reads `loader` from the JSON payload via `payload.get("loader")`. Need to ensure `loader` is passed from the deploy_config. Currently `agent_connection.rs` does NOT forward `loader` — this is a gap to fix.

---

### `src/agent_connection.rs` (middleware/utility, event-driven)

**Analog:** Same file, lines 766-781 — deploy_config mapping to payload JSON. `loader` is NOT currently forwarded.

**DeployConfig struct** (lines 251-271, agent-side definition):
```rust
#[derive(Deserialize)]
pub struct DeployConfig {
    pub image: String,
    pub game_port: Option<u16>,
    pub rcon_port: Option<u16>,
    pub ram_mb: Option<u32>,
    pub version: Option<String>,
    pub loader: Option<String>,     // ✅ Field exists but not mapped to payload
    pub env: Option<HashMap<String, String>>,
    pub volume_path: Option<String>,
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<u64>,
}
```

**Fix — deploy config to payload mapping** (lines 766-781, current with hardcoded `"25565"`):
```rust
// Current (lines 766-781):
if let Some(config) = deploy_config {
    payload["image"] = serde_json::json!(config.image);
    if let Some(port) = config.game_port {
        payload["ports"] = serde_json::json!({ "25565": [port.to_string()] });  // ❌ hardcoded key
        payload["container_port"] = serde_json::json!(port);
    }
    if let Some(env) = config.env {
        payload["env"] = serde_json::json!(env);
    }
    if let Some(mem) = config.ram_mb {
        payload["memory_limit"] = serde_json::json!(mem * 1024 * 1024);
    }
    if let Some(cpu) = config.cpu_limit {
        payload["cpu_limit"] = serde_json::json!(cpu);
    }
}

// Target:
if let Some(config) = deploy_config {
    payload["image"] = serde_json::json!(config.image);
    if let Some(port) = config.game_port {
        // Use dynamic port key instead of hardcoded "25565"
        payload["ports"] = serde_json::json!({ port.to_string(): [port.to_string()] });
        payload["container_port"] = serde_json::json!(port);
    }
    if let Some(env) = config.env {
        payload["env"] = serde_json::json!(env);
    }
    if let Some(mem) = config.ram_mb {
        payload["memory_limit"] = serde_json::json!(mem * 1024 * 1024);
    }
    if let Some(cpu) = config.cpu_limit {
        payload["cpu_limit"] = serde_json::json!(cpu);
    }
    // ADD: forward loader to payload so runtime.rs can read it
    if let Some(loader) = &config.loader {
        payload["loader"] = serde_json::json!(loader);
    }
}
```

---

### `app/src/features/server/CreateServerModal.jsx` (component, request-response)

**Analog:** Same file — multiple sections need bedrock modifications.

**Import pattern** (lines 1-5):
```jsx
import { useState, useEffect } from 'react'
import { useServerStore } from '../../store/serverStore'
import { useUIStore } from '../../store/uiStore'
import { serversApi, nodesApi, api } from '../../lib/api'
import { useModpackTemplates } from '../../hooks/useModpackTemplates'
```

**State variables** (lines 120-148):
```jsx
const [name, setName] = useState('')
const [gameType, setGameType] = useState('minecraft')
const [mcVersion, setMcVersion] = useState('26.2')
const [ram, setRam] = useState('4')
const [maxRam, setMaxRam] = useState('6')
const [maxPlayers, setMaxPlayers] = useState('20')
const [port, setPort] = useState('25565')        // Default port for Minecraft Java
const [onlineMode, setOnlineMode] = useState('true')
const [worldSeed, setWorldSeed] = useState('')
const [difficulty, setDifficulty] = useState('normal')
const [op, setOp] = useState('')
const [serverType, setServerType] = useState('paper')
const [jvmOpts, setJvmOpts] = useState('')
// ADD: Bedrock-specific state vars:
// const [gameMode, setGameMode] = useState('survival')
// const [allowCheats, setAllowCheats] = useState('false')
// const [levelName, setLevelName] = useState('')
```

**Fix #1 — Game type dropdown** (lines 397-403, fallback options):
```jsx
{/* Current fallback options (lines 397-403): */}
<>
  <option value="minecraft">Minecraft</option>
  <option value="palworld" disabled>Palworld (Coming Soon)</option>
  <option value="rust" disabled>Rust (Coming Soon)</option>
  <option value="valheim" disabled>Valheim (Coming Soon)</option>
</>

{/* Target: */}
<>
  <option value="minecraft">Minecraft</option>
  <option value="bedrock">Minecraft Bedrock</option>       {/* ADD */}
  <option value="palworld" disabled>Palworld (Coming Soon)</option>
  <option value="rust" disabled>Rust (Coming Soon)</option>
  <option value="valheim" disabled>Valheim (Coming Soon)</option>
</>
```

**Fix #2 — `handleGameTypeChange` loads modpacks** (lines 227-230):
```jsx
// Current: loads modpacks only for 'minecraft'
if (selectedGameType === 'minecraft' && isHobbyPlusPlan(userPlan)) {
  loadModpacks(selectedGameType)
}

// Target: same, Bedrock doesn't need modpacks
// No change needed — Bedrock won't trigger this (not 'minecraft')
```

**Fix #3 — After Java fields section** (after line 634), add Bedrock conditional fields:
```jsx
{/* Current — only Java fields at line 475: */}
{gameType === 'minecraft' && (
  <>
    {/* Minecraft Version, RAM, Max RAM, Max Players, Port, Online Mode, World Seed, Difficulty, Op, Server Type, JVM Opts */}
  </>
)}

{/* Target — add Bedrock block after the java block closes: */}
{gameType === 'bedrock' && (
  <>
    <div>
      <label>Max Players</label>
      <select value={maxPlayers} onChange={(e) => setMaxPlayers(e.target.value)} ...>
        {PLAYER_OPTIONS.map(...)}
      </select>
    </div>
    <div>
      <label>Online Mode</label>
      <select value={onlineMode} onChange={(e) => setOnlineMode(e.target.value)} ...>
        <option value="true">True</option>
        <option value="false">False</option>
      </select>
    </div>
    <div>
      <label>Game Mode</label>
      <select value={gameMode} onChange={(e) => setGameMode(e.target.value)} ...>
        <option value="survival">Survival</option>
        <option value="creative">Creative</option>
        <option value="adventure">Adventure</option>
      </select>
    </div>
    <div>
      <label>Difficulty</label>
      <select value={difficulty} onChange={(e) => setDifficulty(e.target.value)} ...>
        <option value="peaceful">Peaceful</option>
        <option value="easy">Easy</option>
        <option value="normal">Normal</option>
        <option value="hard">Hard</option>
      </select>
    </div>
    <div>
      <label>Allow Cheats</label>
      <select value={allowCheats} onChange={(e) => setAllowCheats(e.target.value)} ...>
        <option value="true">True</option>
        <option value="false">False</option>
      </select>
    </div>
    <div>
      <label>Level Name</label>
      <input value={levelName} onChange={(e) => setLevelName(e.target.value)} placeholder="Bedrock Server" ... />
    </div>
    <div>
      <label>World Seed</label>
      <input value={worldSeed} onChange={(e) => setWorldSeed(e.target.value)} placeholder="Leave empty for random" ... />
    </div>
    <div>
      <label>Server Port (UDP)</label>
      <input type="number" value={port} onChange={handlePortChange} placeholder="19132" ... />
      <p className="text-gray-400 text-xs">Bedrock servers use UDP port 19132 by default</p>
    </div>
  </>
)}
```

**Fix #4 — `resetForm`** (lines 315-332) — add bedrock state reset:
```jsx
const resetForm = () => {
  setName('')
  setGameType('minecraft')
  setMcVersion('26.2')
  setRam('4')
  setMaxRam('6')
  setMaxPlayers('20')
  setPort('25565')                     // Could conditionally reset based on last gameType
  setOnlineMode('true')
  setWorldSeed('')
  setDifficulty('normal')
  setOp('')
  setServerType('paper')
  setJvmOpts('')
  setPortError('')
  setNodeId('')
  setSelectedModpack(null)
  // ADD bedrock reset:
  // setGameMode('survival')
  // setAllowCheats('false')
  // setLevelName('')
}
```

**Fix #5 — `handleSubmit` sends `game_type` in payload** (lines 276-292):
```jsx
const serverData = {
  name: name.trim(),
  game_type: gameType,                   // ✅ Already sends bedrock
  minecraft_version: mcVersion,
  ram_mb: parseInt(ram) * 1024,
  max_ram_mb: parseInt(maxRam) * 1024,
  max_players: parseInt(maxPlayers),
  port: parseInt(port),
  online_mode: onlineMode === 'true',
  world_seed: worldSeed || undefined,
  difficulty,
  op: op || undefined,
  server_type: serverType,
  jvm_opts: jvmOpts || undefined,
  node_id: nodeId || undefined,
  modpack_template_id: selectedModpack?.id || null,
}
// NOTE: For Bedrock, `minecraft_version`, `ram_mb`, `max_ram_mb`, `server_type`, `jvm_opts`
// are irrelevant. Backend will map `game_type: "bedrock"` to `mc_loader: "bedrock"`.
// Frontend should either exclude these or backend should ignore them for bedrock.
```

**`handlePortChange`** (lines 349-357) — same validation, default port 19132 for Bedrock:
```jsx
// Current default is 25565. When gameType is 'bedrock', you may want to
// set default port via useEffect when gameType changes:
useEffect(() => {
  if (gameType === 'bedrock') {
    setPort('19132')
  } else if (gameType === 'minecraft') {
    setPort('25565')
  }
}, [gameType])
```

## Shared Patterns

### Dynamic Protocol Dispatch (UDP vs TCP)
**Source:** `src/handlers/runtime.rs` lines 120-134 and 271-279
**Apply to:** Both `handle_create` and `handle_start` in `runtime.rs`

The pattern checks `loader` from the deploy config payload to determine protocol:
```rust
let is_bedrock = payload.loader
    .as_deref()
    .map(|l| l.eq_ignore_ascii_case("bedrock"))
    .unwrap_or(false);
let protocol = if is_bedrock { "udp" } else { "tcp" };
let port_key = format!("{}/{}", container_port, protocol);
```

### Dynamic Image Dispatch
**Source:** `api/src/infrastructure/executors/agent_server_executor.rs` lines 67-84
**Apply to:** `build_deploy_config()` method

```rust
let is_bedrock = server.mc_loader.eq_ignore_ascii_case("bedrock");
let image = if is_bedrock {
    "docker.io/itzg/minecraft-bedrock-server"
} else {
    "docker.io/itzg/minecraft-server"
};
let rcon_port = if is_bedrock { None } else { Some((server.port + 10) as u16) };
```

### Port Map Key from Game Port
**Source:** `src/agent_connection.rs` lines 766-781
**Apply to:** `deploy_config` → payload mapping

```rust
// Use game_port as both key and value (dynamic, not hardcoded "25565")
if let Some(port) = config.game_port {
    payload["ports"] = serde_json::json!({ port.to_string(): [port.to_string()] });
}
```

### `loader` Field Forwarding Gap
**Source:** `src/agent_connection.rs` lines 766-781
**Apply to:** deploy_config mapping in agent_connection.rs

The `loader` field exists in both API-side and agent-side `DeployConfig` structs, but is **not forwarded** to the task payload. This must be added:

```rust
if let Some(loader) = &config.loader {
    payload["loader"] = serde_json::json!(loader);
}
```

This enables `runtime.rs` `handle_start` (which reads from raw payload JSON via `payload.get("loader")`) to know whether the container is Bedrock.

### Game Type → `mc_loader` Mapping
**Source:** `api/src/application/use_cases/create_server_use_case.rs` line 45
**Apply to:** Server creation when `game_type` = `"bedrock"`

```rust
// Current default (line 45):
mc_loader: req.mc_loader.clone().unwrap_or_else(|| "PAPER".to_string()),

// Target: derive mc_loader from game_type
let mc_loader = if req.game_type.as_deref() == Some("bedrock") {
    "bedrock".to_string()
} else {
    req.mc_loader.clone().unwrap_or_else(|| "PAPER".to_string())
};
```

## No Analog Found

All files have exact analogs (the files themselves). No new files need external analogs.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| All | — | — | Each file has an existing version to modify in-place |

## Metadata

**Analog search scope:** `api/src/`, `src/`, `app/src/features/server/`, `api/migrations/`
**Files scanned:** 12
**Pattern extraction date:** 2026-06-12
