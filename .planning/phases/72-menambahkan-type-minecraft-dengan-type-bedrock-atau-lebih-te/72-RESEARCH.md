# Phase 72: Menambahkan Type Minecraft Bedrock — Research

**Researched:** 2026-06-12
**Domain:** Minecraft Bedrock Edition server type integration
**Confidence:** HIGH (codebase-verified, multiple sources)

## Summary

This phase adds **Minecraft Bedrock Edition** as a first-class deployable server type to the Escluse platform. Currently, the codebase has *partial* Bedrock support scattered across files from Phase 36 and the template system, but the execution pipeline (API handler → agent executor → agent runtime) is hardcoded for Minecraft Java Edition only.

**Key gap:** Four layers must be modified to make Bedrock deployment work end-to-end:
1. **Database** — `game_types` table has no Bedrock row (5 rows: minecraft, palworld, valheim, fabric, forge)
2. **API** — Agent executor hardcodes Java Docker image; handler doesn't map `game_type: "bedrock"` to correct `mc_loader`
3. **Agent** — Runtime hardcodes TCP-only port binding; connection hardcodes `"25565"` as port map key
4. **Frontend** — CreateServerModal doesn't show Bedrock as game type; always shows Java-specific fields (JVM opts, version, server type)

**Already working:** Template fallback (model.rs has bedrock with `itzg/minecraft-bedrock-server:latest`), connectivity probe (`probe_bedrock_edition` UDP RakNet), server entity (`mc_loader` supports `"bedrock"`).

**Primary recommendation:** Implement Bedrock in 4 sequential plans: (1) DB + API handler, (2) Agent runtime UDP support, (3) Frontend UI, (4) End-to-end verification.

<user_constraints>
## User Constraints (from CONTEXT.md)

No CONTEXT.md exists for this phase. No locked decisions recorded.

**Phase description:** "menambahkan type minecraft, dengan type Bedrock atau lebih tepatnya minecraft bedrock"
**Depends on:** Phase 71 (landing page plan subscription flow)

The phase goal is clear: add Minecraft Bedrock Edition as a server type. There is no further specification beyond the UI-SPEC.md that was previously drafted.
</user_constraints>

<phase_requirements>
## Phase Requirements

No REQUIREMENTS.md exists for this phase. Requirements derived from codebase analysis and UI-SPEC.md:

| Requirement | Description | Research Support |
|-------------|-------------|------------------|
| REQ-01 | Bedrock game type available in UI | CreateServerModal fallback options need `"bedrock"` active entry |
| REQ-02 | API creates servers with correct Docker image | `agent_server_executor.rs` must switch image based on `mc_loader` |
| REQ-03 | Agent creates containers with UDP port binding | `runtime.rs` lines 124/274 must use `/udp` for Bedrock |
| REQ-04 | Agent reports correct ports for Bedrock | `agent_connection.rs` line 769 must use dynamic port, not hardcoded `"25565"` |
| REQ-05 | DB has Bedrock `game_types` row | Migration needed (currently 5 rows, no bedrock) |
| REQ-06 | Connectivity probing works for Bedrock | Already works ✅ via `probe_bedrock_edition` UDP RakNet |
| REQ-07 | Bedrock-specific form fields shown in UI | CreateServerModal needs conditional rendering block |
| REQ-08 | Relay tunnel handles Bedrock ports | `local_mc_addr` uses `server.port` generically, tunnel protocol is TCP-only |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Game type definitions | Database (`game_types` table) | API code fallback (`GameType::fallback`) | Phase 05 established DB-driven with code fallback |
| Docker image selection | API (`agent_server_executor.rs`) | Template config | API maps `mc_loader` to correct image at deploy time |
| Port binding protocol | Agent (`runtime.rs`) | — | Bollard Docker API needs explicit `/tcp` or `/udp` suffix |
| Server creation UI | Frontend (`CreateServerModal.jsx`) | — | Game type selection + conditional field rendering |
| Server connectivity probe | API (`connectivity_service.rs`) | — | Already handles Bedrock via `probe_bedrock_edition()` |
| Relay tunnel | API (`relay_service`) + Agent | — | Uses `server.port` for `local_mc_addr`; tunnel is TCP-only |
| Template fallbacks | API (`template/model.rs`) | DB templates | Already has bedrock template ✅ |

## Standard Stack

### Core Docker Images
| Image | Version | Purpose |
|-------|---------|---------|
| `itzg/minecraft-bedrock-server` | `latest` | Minecraft Bedrock Edition dedicated server [VERIFIED: Docker Hub] |
| `itzg/minecraft-server` | `latest` | Minecraft Java Edition server (existing, unchanged) |

### Key Differences: Bedrock vs Java Edition
| Aspect | Java (itzg/minecraft-server) | Bedrock (itzg/minecraft-bedrock-server) |
|--------|------------------------------|----------------------------------------|
| Default port | 25565 TCP | 19132 UDP |
| RCON | Yes (25575 TCP) | **No** (uses stdin for console) |
| Server type (`TYPE`) | VANILLA, PAPER, SPIGOT, FORGE, FABRIC | **Single binary** (no type variants) |
| JVM / RAM config | `-Xmx`, `MEMORY` env, JVM options | **Not applicable** (C++ binary) |
| Key env vars | `EULA`, `TYPE`, `VERSION`, `MEMORY`, `MAX_PLAYERS`, `ONLINE_MODE` | `EULA`, `VERSION`, `GAMEMODE`, `DIFFICULTY`, `LEVEL_NAME`, `LEVEL_SEED`, `MAX_PLAYERS`, `ALLOW_CHEATS` |
| Port env var | None (container default 25565) | `SERVER_PORT` (default 19132) |
| Version format | Java version (1.20.4, 1.21, etc.) | Bedrock version (1.21.44, LATEST, PREVIOUS) |

### Bedrock Environment Variables (itzg/minecraft-bedrock-server)
[VERIFIED: hub.docker.com/r/itzg/minecraft-bedrock-server]

Container-level:
- `EULA` (required: `TRUE`)
- `VERSION` (default: `LATEST`)
- `UID` / `GID` (optional)

Server properties (mapped via env vars to `server.properties`):
- `SERVER_NAME`, `SERVER_PORT`, `SERVER_PORT_V6`
- `GAMEMODE` (survival, creative, adventure)
- `DIFFICULTY` (peaceful, easy, normal, hard)
- `LEVEL_NAME`, `LEVEL_SEED`, `LEVEL_TYPE`
- `MAX_PLAYERS`, `ONLINE_MODE`, `WHITE_LIST`
- `ALLOW_CHEATS`, `VIEW_DISTANCE`, `TICK_DISTANCE`
- `PLAYER_IDLE_TIMEOUT`, `MAX_THREADS`
- `DEFAULT_PLAYER_PERMISSION_LEVEL`
- `TEXTUREPACK_REQUIRED`
- `SERVER_AUTHORITATIVE_MOVEMENT`

## Architecture Patterns

### System Architecture Diagram

```
[Frontend: CreateServerModal.jsx]
  │ select game_type="bedrock"
  │ POST /api/v1/servers { game_type: "bedrock", ... }
  ▼
[API: server_handlers.rs — create_server handler]
  │ Server::new() → config.game_type = "bedrock"
  ▼
[API: agent_server_executor.rs — build_deploy_config()]
  │ checks server.mc_loader → "bedrock"
  │ image = "itzg/minecraft-bedrock-server:latest"
  │ rcon_port = None (Bedrock has no RCON)
  │ env_vars: EULA=TRUE, GAMEMODE=survival, ...
  ├──→ NodeMessage::ExecuteCommand with DeployConfig
  ▼
[Agent: agent_connection.rs → payload builder]
  │ reads deploy_config
  │ ports = { game_port: [port] }  (NOT hardcoded 25565)
  ▼
[Agent: handlers/runtime.rs — handle_create/handle_start]
  │ reads deploy_config.loader → "bedrock"
  │ port_key = format!("{}/udp", container_port)  (NOT /tcp)
  │ env_vars from deploy_config passed to container
  ▼
[Docker: itzg/minecraft-bedrock-server]
  │ Bedrock server runs on UDP 19132
  ▼
[API: connectivity_service.rs — periodic probe]
  │ mc_loader == "bedrock" → probe_bedrock_edition()
  │ UDP RakNet ping on port 19132
```

### Current Pain Points (verified in code)

| File | Line(s) | Issue | Severity |
|------|---------|-------|----------|
| `api/migrations/20260409000001_create_game_types_table.sql` | 22-27 | No Bedrock row in `INSERT INTO game_types` | HIGH |
| `api/src/domain/server/model.rs` | 70-72 | `default_image()` returns `itzg/minecraft-server:latest` — no Bedrock mapping | MEDIUM |
| `api/src/infrastructure/executors/agent_server_executor.rs` | 71 | `image: "docker.io/itzg/minecraft-server".to_string()` — hardcoded Java | HIGH |
| `api/src/infrastructure/executors/agent_server_executor.rs` | 72-73 | Always sets `rcon_port` — Bedrock has no RCON | HIGH |
| `src/handlers/runtime.rs` | 124, 274 | `format!("{}/tcp", container_port)` — hardcoded TCP | HIGH |
| `src/agent_connection.rs` | 769 | `payload["ports"] = json!({ "25565": [...] })` — hardcoded key | HIGH |
| `api/src/application/use_cases/create_server_use_case.rs` | 45 | `mc_loader: req.mc_loader.unwrap_or("PAPER")` — Java default | MEDIUM |
| `app/src/features/server/CreateServerModal.jsx` | 399-402 | No bedrock option in fallback game type list | HIGH |
| `app/src/features/server/CreateServerModal.jsx` | 475-633 | Java fields always shown when `gameType === 'minecraft'` | HIGH |
| `app/src/features/server/CreateServerModal.jsx` | 328 | `resetForm` sets `jvmOpts`, `serverType` — needs Bedrock reset too | LOW |

### Already Working for Bedrock

| File | Line(s) | What Works |
|------|---------|------------|
| `api/src/domain/server/template/model.rs` | 179-197 | `Template::fallback()` includes bedrock with `itzg/minecraft-bedrock-server:latest`, port 19132 |
| `api/src/application/services/connectivity_service.rs` | 260-261 | `probe_bedrock_edition()` called when `mc_loader == "bedrock"` |
| `api/src/domain/entities/server.rs` | 28-29 | `mc_loader: String`, doc comment lists `"bedrock"` as valid value |
| `app/src/pages/templates/TemplateCreatePage.jsx` | 129, 140 | Already includes `"bedrock"` option with variants `vanilla, pocketmine, nukkit, powernukkitx` |

### Recommended Project Structure (Changes)

```
api/migrations/
└── 20260612000001_add_bedrock_game_type.sql     # ADD: bedrock game_types row

api/src/
├── application/
│   ├── use_cases/
│   │   └── create_server_use_case.rs             # MODIFY: map game_type to mc_loader
│   └── dto/
│       └── server_dtos.rs                       # VERIFY: field mappings
├── presentation/
│   └── handlers/
│       └── server_handlers.rs                   # MODIFY: set image/mc_loader from game_type
├── infrastructure/
│   └── executors/
│       └── agent_server_executor.rs              # MODIFY: dynamic image + no rcon for bedrock
├── domain/
│   ├── server/
│   │   └── template/model.rs                    # VERIFY: bedrock template (already exists)
│   └── server/
│       └── model.rs                             # VERIFY: CreateServerRequest game_type field

src/ (agent/solys)
├── handlers/
│   └── runtime.rs                               # MODIFY: dynamic TCP/UDP port binding
└── agent_connection.rs                          # MODIFY: dynamic port map key

app/src/
└── features/server/
    └── CreateServerModal.jsx                     # MODIFY: add Bedrock game type + Bedrock fields
```

### Pattern 1: Game Type → Docker Image Dispatch
**What:** Route server creation to the correct Docker image based on `mc_loader`
**Where:** `agent_server_executor.rs:67-84` → `build_deploy_config()`

```rust
// Current (Java-only hardcoded):
let image = "docker.io/itzg/minecraft-server".to_string();

// Proposed (dynamic dispatch):
let is_bedrock = server.mc_loader.eq_ignore_ascii_case("bedrock");
let image = if is_bedrock {
    "docker.io/itzg/minecraft-bedrock-server"
} else {
    "docker.io/itzg/minecraft-server"
};
let rcon_port = if is_bedrock { None } else { Some((server.port + 10) as u16) };
```

**For `server_handlers.rs`** — `mc_loader` should be derived from `game_type`:
```rust
// In create_server handler, after building config:
let image = match payload.game_type.as_deref() {
    Some("bedrock") | Some("minecraft-bedrock") => "itzg/minecraft-bedrock-server:latest",
    _ => "itzg/minecraft-server:latest",  // default_image()
};
```

### Pattern 2: Port Protocol Dispatch
**What:** Choose TCP vs UDP port binding based on `mc_loader` / loader
**Where:** `src/handlers/runtime.rs` lines 120-134 and ~270

```rust
// Current (TCP only):
let port_key = format!("{}/tcp", container_port);

// Proposed (dynamic):
let is_bedrock = loaders
    .and_then(|l| l.as_str())
    .map(|l| l.eq_ignore_ascii_case("bedrock"))
    .unwrap_or(false);
let protocol = if is_bedrock { "udp" } else { "tcp" };
let port_key = format!("{}/{}", container_port, protocol);
```

### Pattern 3: Dynamic Port Map Key
**What:** Use `game_port` from deploy config instead of hardcoded `"25565"`
**Where:** `src/agent_connection.rs` line 769

```rust
// Current (hardcoded):
payload["ports"] = serde_json::json!({ "25565": [port.to_string()] });

// Proposed (dynamic):
let port_key = format!("{}", port); // Use game_port directly
payload["ports"] = serde_json::json!({ port_key: [port.to_string()] });
```

### Anti-Patterns to Avoid
- **Hardcoding Java image for all servers:** Current `build_deploy_config` always uses `itzg/minecraft-server`. Breaks Bedrock.
- **Hardcoding TCP port bindings:** Agent creates all containers with `/tcp`. Bedrock uses UDP only → client timeout.
- **Hardcoding `"25565"` port map key:** Agent connection maps `{ "25565": [port] }`. Should use actual `game_port`.
- **Showing Java-specific fields for non-Java types:** CreateServerModal always renders JVM opts, version dropdown, RAM options when `gameType === 'minecraft'`. For Bedrock, hide these.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Bedrock server binary | Custom download script | `itzg/minecraft-bedrock-server` Docker image | Handles version selection, EULA, auto-updates |
| UDP RakNet ping | Custom implementation | Already built (`probe_bedrock_edition`) | `connectivity_service.rs` line 477 |
| Bedrock config | Manual `server.properties` | ENV vars from Docker image | All mapped via image env vars |

## Common Pitfalls

### Pitfall 1: UDP vs TCP Port Binding
**What goes wrong:** Agent creates container with `19132/tcp` instead of `19132/udp`. Bedrock clients timeout with "Unable to connect to world".
**Root cause:** `runtime.rs` lines 124 and 274 use `format!("{}/tcp", container_port)` unconditionally. Bollard Docker API requires explicit `/udp` suffix.
**How to avoid:** Read `loader` from deploy config payload. If `"bedrock"`, use `/udp` instead of `/tcp`.
**Warning signs:** Container starts successfully but clients fail to connect; `docker port` shows `19132/tcp` instead of `19132/udp`.

### Pitfall 2: Wrong Docker Image
**What goes wrong:** Agent downloads and runs `itzg/minecraft-server` (Java) for Bedrock servers.
**Root cause:** `agent_server_executor.rs:71` hardcodes `"docker.io/itzg/minecraft-server"`. The `mc_loader` field is set but never checked for image selection.
**How to avoid:** Check `server.mc_loader` and select image dynamically in `build_deploy_config()`.
**Warning signs:** Container logs show Java JVM startup; Bedrock clients get "Outdated server" protocol errors.

### Pitfall 3: RCON for Bedrock
**What goes wrong:** Backend sends `rcon_port` to agent which tries to connect to RCON on a Bedrock container that has no RCON.
**Root cause:** `agent_server_executor.rs:73` always sets `rcon_port`.
**How to avoid:** Set `rcon_port: None` when `mc_loader == "bedrock"`. Capabilities JSON should include `{"rcon": false}`.
**Impact:** Medium — unused port binding but harmless. Console/terminal for Bedrock must use Docker `exec` stdin, not RCON.

### Pitfall 4: Port Map Key is Hardcoded `"25565"`
**What goes wrong:** `agent_connection.rs:769` always stores ports under key `"25565"`, but Bedrock uses port 19132. Docker still works (port is correct value), but the stored config is confusing.
**Root cause:** The JSON map `{ "25565": [port] }` uses a literal string key instead of the actual `game_port`.
**How to avoid:** Use the `game_port` value as the map key: `json!({ port.to_string(): [port.to_string()] })`.
**Impact:** Low — Docker creates correct port binding regardless of JSON key, but metrics/config views may display wrong port info.

### Pitfall 5: Relay Tunnel TCP Assumption
**What goes wrong:** Relay tunnel uses yamux over WebSocket (TCP). Bedrock's UDP traffic cannot be forwarded through a TCP tunnel without additional proxying.
**Root cause:** Phase 68 relay infrastructure assumes TCP (Minecraft Java uses TCP).
**How to avoid:** For Bedrock, default to direct mode (no relay). Document that relay support for UDP/Bedrock is a future enhancement. Alternatively, use a UDP-to-TCP proxy (like `udp-over-tcp` or a SOCKS5 UDP associate) in the relay client.
**Impact:** MEDIUM — Bedrock servers will work in direct mode but won't have relay failover. This is acceptable for MVP.

### Pitfall 6: `mc_loader` Default is `"PAPER"`
**What goes wrong:** `create_server_use_case.rs:45` defaults `mc_loader` to `"PAPER"` when not provided. If `game_type: "bedrock"` comes from frontend but `mc_loader` isn't mapped, the server gets PAPER → Java image.
**Root cause:** The use case doesn't derive `mc_loader` from `game_type`.
**How to avoid:** In the use case (or handler), add mapping: if `game_type` is `"bedrock"` or `"minecraft-bedrock"`, set `mc_loader` to `"bedrock"`.

## Code Examples

### Current Agent Executor (Must Fix)
```rust
// Source: api/src/infrastructure/executors/agent_server_executor.rs:67-84
fn build_deploy_config(&self, server: &Server) -> DeployConfig {
    let rcon_port = (server.port + 10) as u16;
    DeployConfig {
        image: "docker.io/itzg/minecraft-server".to_string(), // ❌ Hardcoded Java
        game_port: Some(server.port as u16),
        rcon_port: Some(rcon_port),                           // ❌ Bedrock has no RCON
        loader: Some(...),
        ...
    }
}
```

### Current Agent Runtime Port Binding (Must Fix)
```rust
// Source: src/handlers/runtime.rs:124
let port_key = format!("{}/tcp", container_port); // ❌ Always TCP

// Source: src/agent_connection.rs:769
payload["ports"] = serde_json::json!({ "25565": [port.to_string()] }); // ❌ Hardcoded key
```

### Target Agent Executor (Dynamic)
```rust
fn build_deploy_config(&self, server: &Server) -> DeployConfig {
    let is_bedrock = server.mc_loader.eq_ignore_ascii_case("bedrock");
    let image = if is_bedrock {
        "docker.io/itzg/minecraft-bedrock-server"
    } else {
        "docker.io/itzg/minecraft-server"
    };
    let rcon_port = if is_bedrock { None } else { Some((server.port + 10) as u16) };
    DeployConfig {
        image: image.to_string(),
        game_port: Some(server.port as u16),
        rcon_port,
        ram_mb: Some(parse_ram_allocation(&server.ram_allocation)),
        version: Some(...),
        loader: Some(server.mc_loader.clone()),
        env_vars: {
            let mut vars = HashMap::new();
            vars.insert("EULA".to_string(), "TRUE".to_string());
            if is_bedrock {
                vars.insert("GAMEMODE".to_string(), "survival".to_string());
                vars.insert("DIFFICULTY".to_string(), "normal".to_string());
            }
            vars
        },
        volume_path: Some(...),
    }
}
```

### CreateServerModal: Add Bedrock Option (JSX)
```jsx
// In game type dropdown fallback section (line 397-403):
<>
  <option value="minecraft">Minecraft</option>
  <option value="bedrock">Minecraft Bedrock</option>       {/* ADD */}
  <option value="palworld" disabled>Palworld (Coming Soon)</option>
  <option value="rust" disabled>Rust (Coming Soon)</option>
  <option value="valheim" disabled>Valheim (Coming Soon)</option>
</>

// Bedrock conditional fields (after line 475 Java block):
{gameType === 'bedrock' && (
  <>
    <div> {/* Max Players - reuse existing */}
      <label>Max Players</label>
      <select value={maxPlayers} onChange={...}>
        {PLAYER_OPTIONS.map(...)}
      </select>
    </div>
    <div> {/* Online Mode - reuse existing */}
      <label>Online Mode</label>
      <select value={onlineMode} ...>
        <option value="true">True</option>
        <option value="false">False</option>
      </select>
    </div>
    <div> {/* Game Mode - Bedrock-only */}
      <label>Game Mode</label>
      <select value={gameMode} onChange={...}>
        <option value="survival">Survival</option>
        <option value="creative">Creative</option>
        <option value="adventure">Adventure</option>
      </select>
    </div>
    <div> {/* Difficulty - reuse existing */}
      <label>Difficulty</label>
      <select value={difficulty} ...>
        <option value="peaceful">Peaceful</option>
        <option value="easy">Easy</option>
        <option value="normal">Normal</option>
        <option value="hard">Hard</option>
      </select>
    </div>
    <div> {/* Allow Cheats - Bedrock-only */}
      <label>Allow Cheats</label>
      <select value={allowCheats} onChange={...}>
        <option value="true">True</option>
        <option value="false">False</option>
      </select>
    </div>
    <div> {/* Level Name - Bedrock-only */}
      <label>Level Name</label>
      <input value={levelName} ... placeholder="Bedrock Server" />
    </div>
    <div> {/* World Seed - reuse existing pattern */}
      <label>World Seed</label>
      <input value={worldSeed} ... placeholder="Leave empty for random" />
    </div>
    <div> {/* Port - Bedrock default 19132 */}
      <label>Server Port (UDP)</label>
      <input type="number" value={port} ... placeholder="19132" />
      <p className="text-gray-400 text-xs">Bedrock servers use UDP port 19132 by default</p>
    </div>
  </>
)}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No bedrock support | Partial bedrock template in code fallback | Phase 36 | Template exists, execution pipeline ignores it |
| No bedrock connectivity probe | UDP RakNet probe (`probe_bedrock_edition`) | Phase 67 | Probe works, just needs `mc_loader="bedrock"` |
| Java-only image assumption | Dynamic image per `mc_loader` | This phase | Core change enabling Bedrock deployment |

## Runtime State Inventory

**Not applicable** — this is a feature addition phase (not rename/refactor/migration). No existing runtime state contains the renamed/refactored string.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `itzg/minecraft-bedrock-server` uses `19132/udp` by default | Standard Stack | LOW — verified from Docker Hub docs |
| A2 | Bedrock has no RCON support | Standard Stack | LOW — confirmed from image docs and Bedrock protocol |
| A3 | The `loader` field in `DeployConfig` propagates through agent task payload to `handle_start`/`handle_create` in runtime.rs | Architecture | MEDIUM — must verify `loader` is in task payload JSON that runtime.rs receives |
| A4 | Relay tunnel yamux/WebSocket cannot forward UDP | Architecture | MEDIUM — if relay can do UDP, add relay support; if not, direct mode only for Bedrock |
| A5 | Setting `rcon_port: None` in DeployConfig won't break agent | Code Examples | LOW — agent handles `Option<u16>` already |

## Open Questions (RESOLVED)

1. **Does `loader` from `DeployConfig` reach the agent runtime's handlers?**
   - What we know: `build_deploy_config` sets `loader`, `agent_connection.rs` maps deploy_config fields to payload JSON (line 766-781). But only `image`, `ports`, `container_port`, `env`, `memory_limit`, `cpu_limit` are mapped — `loader` is NOT forwarded.
   - What's unclear: How should `loader` reach `runtime.rs`? Options: (a) Add `loader` to the `execute_command` payload, (b) Use `container_port` value to infer protocol, (c) Add a new payload field `protocol` or `loader`.
   - **Recommendation:** Add `loader` field to the payload mapping in `agent_connection.rs` and extract it in `runtime.rs` `handle_create` and `handle_start`. This is the cleanest approach.

2. **How should Bedrock console/terminal work?**
   - Java servers use RCON via `rcon-cli`. Bedrock has no RCON.
   - The itzg bedrock image supports stdin-based console via `docker exec -i`.
   - **Deferred:** Console support for Bedrock is out of scope for Phase 72. Document as known limitation.

3. **Should Bedrock behavior packs / addons be supported?**
   - Bedrock supports `.mcpack`, `.mcworld`, `.mcaddon`. The image supports `MC_PACK` env var.
   - **Deferred:** Out of scope for Phase 72. Can be passed through `env_vars` in template config if needed.

4. **Does the relay tunnel support UDP?**
   - Currently yamux/WSS is TCP-only. Bedrock's RakNet protocol is UDP.
   - **Phase 72 MVP:** Default Bedrock servers to direct mode. Document relay support as a future enhancement.
   - In `relay_service.rs`, when building relay config for a server, skip Bedrock servers (or set mode to `direct`).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Docker (agent host) | Container creation | ✓ (assumed) | — | — |
| Bollard 0.18 | Agent runtime.rs | ✓ | 0.18 | — |
| UDP socket | Agent runtime + connectivity | ✓ | — | — |
| PostgreSQL | Game types migration | ✓ (RDS) | — | — |
| Node.js | Frontend dev | ✓ | — | — |

## Validation Architecture

> Nyquist validation framework enabled (config.json has no `workflow.nyquist_validation: false`).

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Not detected — project has no test runner manifest in root |
| Config file | None detected |
| Quick run command | `cargo test -p api` (Rust backend) |
| Full suite command | `cargo test` + `npm --prefix app run build` (if app builds) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-01 | Frontend shows Bedrock option | Manual | N/A — visual inspection | ❌ Manual |
| REQ-02 | API creates server with correct image | Integration | `cargo test --test create_server_bedrock` | ❌ Wave 0 |
| REQ-03 | Agent creates container with UDP binding | Integration | `cargo test --test agent_udp_binding` (add test) | ❌ Wave 0 |
| REQ-04 | Agent reports correct ports | Integration | Code review + manual | ❌ Manual |
| REQ-05 | DB has bedrock game_types row | Migration | `cargo test` (sqlx migration check) | ❌ Wave 0 |
| REQ-06 | Connectivity probe works for Bedrock | Existing | Already in connectivity_service test | ✅ |
| REQ-07 | Bedrock-specific form fields shown | Manual | N/A — visual inspection | ❌ Manual |
| REQ-08 | Relay handles Bedrock | Manual | Code review + manual | ❌ Manual |

### Verification Strategy (Per Plan)

**Plan 72-01 (DB + API):**
- [ ] Run `cargo test` — no regressions
- [ ] Verify migration adds bedrock row: `SELECT * FROM game_types WHERE identifier='bedrock'`
- [ ] Verify `build_deploy_config` returns correct image when `mc_loader="bedrock"`
- [ ] Verify `rcon_port` is `None` for bedrock

**Plan 72-02 (Agent):**
- [ ] Verify `runtime.rs` port key uses `"/udp"` when `loader == "bedrock"`
- [ ] Verify `agent_connection.rs` uses dynamic port key instead of `"25565"`
- [ ] Manual: Deploy Bedrock server, check `docker port <container>` shows `19132/udp`

**Plan 72-03 (Frontend):**
- [ ] Visual: CreateServerModal shows "Minecraft Bedrock" as game type option
- [ ] Visual: Selecting Bedrock hides Java-specific fields (version, JVM opts, server type, RAM)
- [ ] Visual: Selecting Bedrock shows Bedrock-specific fields (game mode, allow cheats, level name)
- [ ] Visual: Default port is 19132 for Bedrock
- [ ] API call: `POST /api/v1/servers` with `game_type: "bedrock"` sends correct payload

**Plan 72-04 (E2E):**
- [ ] Create Bedrock server via UI → server appears in dashboard as "Minecraft Bedrock"
- [ ] Server status transitions to "running"
- [ ] `docker inspect` shows UDP port binding 19132
- [ ] Minecraft Bedrock client can connect (manual)
- [ ] Connectivity probe shows "reachable"

### Sampling Rate
- **Per task commit:** `cargo build` (verify compilation)
- **Per wave merge:** `cargo test` + `npm --prefix app run build`
- **Phase gate:** Full manual verification via E2E checklist

### Wave 0 Gaps
- [ ] `tests/create_server_bedrock.rs` — integration test for API handler with bedrock game_type
- [ ] `tests/agent_udp_binding.rs` — verify agent runtime handles `/udp` ports correctly
- [ ] Test fixture/conftest for game_types with bedrock row

## Security Domain

> `security_enforcement` not explicitly set to false in config.json — treating as enabled.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | No new auth paths |
| V3 Session Management | No | No session changes |
| V4 Access Control | No | Uses existing RBAC |
| V5 Input Validation | Yes | Port validation (10000-30000), game_type enum check |
| V6 Cryptography | No | No crypto changes |

### Known Threat Patterns
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Invalid game_type injection | Tampering | Validate `game_type` against allowed values on API handler |
| Port hijack (UDP 19132) | Spoofing | Standard port allocation pool (10000-30000) same as TCP |
| Unsafe env vars in container | Tampering | `EULA=TRUE` already validated; sanitize `ALLOW_CHEATS` |
| No new attack surface — all code paths use existing executor/auth patterns |

## Planning Guidance

This phase needs **at least 4 plans**:

| Plan | Focus | Files |
|------|-------|-------|
| **72-01** | DB migration + API: Add bedrock game_types row, fix agent executor for dynamic image/protocol | `20260612000001_add_bedrock_game_type.sql`, `agent_server_executor.rs`, `server_handlers.rs`, `create_server_use_case.rs` |
| **72-02** | Agent: Add UDP port binding, dynamic port mapping | `src/handlers/runtime.rs`, `src/agent_connection.rs` |
| **72-03** | Frontend: Add Bedrock game type option in CreateServerModal with conditional fields | `CreateServerModal.jsx` |
| **72-04** | End-to-end verification: Manual test plan covering all layers | Test documentation |

### Ordering
72-01 → 72-02 (agent needs API to pass correct deploy_config) → 72-03 (separate, can be parallel with 72-02) → 72-04 (requires all previous)

72-01 and 72-02 are MOST CRITICAL — without them, Bedrock servers fail at runtime (wrong image, TCP binding). 72-03 is frontend-only and independent.

### Dependency on Phase 71
Phase 71 modifies landing page subscription flow. Phase 72's backend/frontend changes do NOT depend on billing UI. However, if Phase 71 modified the server creation API handler or agent executor, those changes could create merge conflicts with 72-01. Coordinate or rebase after Phase 71 is complete.

## Sources

### Primary (HIGH confidence)
- [VERIFIED: Codebase] — `api/src/infrastructure/executors/agent_server_executor.rs:67-84` — hardcoded Java image
- [VERIFIED: Codebase] — `src/handlers/runtime.rs:124,274` — hardcoded TCP port binding
- [VERIFIED: Codebase] — `src/agent_connection.rs:769` — hardcoded `"25565"` port map key
- [VERIFIED: Codebase] — `api/migrations/20260409000001_create_game_types_table.sql` — no bedrock row
- [VERIFIED: Codebase] — `api/src/domain/server/template/model.rs:179-197` — bedrock template exists
- [VERIFIED: Codebase] — `api/src/application/services/connectivity_service.rs:260-261` — bedrock probe works
- [VERIFIED: Codebase] — `api/src/domain/entities/server.rs:28-29` — `mc_loader` supports `"bedrock"`
- [VERIFIED: Codebase] — `app/src/features/server/CreateServerModal.jsx:399-402` — no bedrock in fallback
- [VERIFIED: Codebase] — `app/src/features/server/CreateServerModal.jsx:475-633` — Java fields only
- [VERIFIED: Codebase] — `api/src/application/use_cases/create_server_use_case.rs:45` — mc_loader defaults to PAPER
- [VERIFIED: Docker Hub] — `itzg/minecraft-bedrock-server` image docs
- [VERIFIED: GitHub] — `itzg/docker-minecraft-bedrock-server` README

### Secondary (MEDIUM confidence)
- [CITED: Docker Hub] — Bedrock image environment variables
- [CITED: Codebase review] — Relay tunnel assumes TCP (yamux/WSS)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Docker images verified from Docker Hub + GitHub
- Architecture: HIGH — Full codebase analysis completed, all referenced file paths verified
- Pitfalls: HIGH — All pitfalls identified from codebase review with verified line references
- Frontend details: HIGH — CreateServerModal fully analyzed (675 lines reviewed)

**Research date:** 2026-06-12
**Valid until:** 2026-07-12 (30 days — stable dependencies)
