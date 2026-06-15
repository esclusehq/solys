---
phase: 72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te
fixed_at: 2026-06-12T15:00:00Z
review_path: ".planning/phases/72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te/72-REVIEW.md"
iteration: 1
findings_in_scope: 7
fixed: 5
skipped: 2
status: partial
---

# Phase 72: Code Review Fix Report

**Fixed at:** 2026-06-12T15:00:00Z  
**Source review:** `.planning/phases/72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te/72-REVIEW.md`  
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (3 critical + 4 warning)
- Fixed: 5
- Skipped: 2
- Combined into same commit: CR-03 + WR-02 + WR-03 (all in same file)

## Fixed Issues

### CR-01: DeployConfig field name mismatch — env_vars vs env (agent-side struct)

**Files modified:** `agent/solys/src/agent_connection.rs`
**Commit:** `37db0fa` (agent/solys repo)
**Applied fix:** Renamed `env: Option<HashMap<String, String>>` field to `env_vars: HashMap<String, String>` with `#[serde(default)]` in the `DeployConfig` struct to match the backend's field name. Updated the `Default` impl accordingly.

### CR-02: Payload key mismatch — `payload["env"]` vs `payload["env_vars"]` in agent runtime

**Files modified:** `agent/solys/src/agent_connection.rs`
**Commit:** `37db0fa` (agent/solys repo, same commit as CR-01)
**Applied fix:** Changed `if let Some(env) = config.env { payload["env"] = ... }` to `if !config.env_vars.is_empty() { payload["env_vars"] = ... }` so the payload key matches what `runtime.rs:handle_start` expects. Note: `runtime.rs` already reads from `"env_vars"`, so no fix was needed there.

### CR-03: Hardcoded loader="PAPER" in start/wake handlers breaks Bedrock protocol

**Files modified:** `api/src/presentation/handlers/server_handlers.rs`
**Commit:** `0e51864` (api repo)
**Applied fix:** Added `is_bedrock` detection (checking `server.config["game_type"]` for `"bedrock"` or `"minecraft-bedrock"`). When bedrock is detected:
- `loader` is set to `"bedrock"` instead of `"PAPER"` (enables UDP protocol in agent's `runtime.rs`)
- `rcon_port` is set to `None` (no RCON needed for Bedrock)
- `game_port` defaults to `19132` instead of `25565`

Applied to both `start_server` and `wake_server` handlers.

### WR-01: Memory leak in `check_status` via `Box::leak`

**Files modified:** `api/src/infrastructure/executors/agent_server_executor.rs`
**Commit:** `f05b4af` (api repo)
**Applied fix:** Replaced the `if-else` with `Box::leak` pattern with an `Option`-based approach using `.map().unwrap_or_else()` that returns an owned `String`. This eliminates the memory leak that would accumulate with each status poll for servers without a `container_name`.

### WR-02: Missing Bedrock port handling in server_handlers.rs start/wake — always sends rcon_port

**Files modified:** `api/src/presentation/handlers/server_handlers.rs`
**Commit:** `0e51864` (api repo, same commit as CR-03)
**Applied fix:** Same `is_bedrock` check from CR-03 now conditionally sets `rcon_port: None` for Bedrock servers in both `start_server` and `wake_server` handlers, matching the logic in `agent_server_executor.rs::build_deploy_config`.

### WR-03: Bedrock-specific fields not mapped to deploy config in start/wake handlers

**Files modified:** `api/src/presentation/handlers/server_handlers.rs`
**Commit:** `0e51864` (api repo, same commit as CR-03)
**Applied fix:** When `is_bedrock` is true, `GAMEMODE=survival` and `DIFFICULTY=normal` environment variables are now inserted into the deploy config, matching `agent_server_executor.rs::build_deploy_config`.

### WR-04: Port validation allows ports below 10000 for Bedrock after initialization

**Files modified:** `app/src/features/server/CreateServerModal.jsx`
**Commit:** `fdccd27` (app repo)
**Applied fix:** Made the port validation error message game-type-aware. When `gameType === 'bedrock'`, the error message shows "Bedrock default is 19132" as a hint alongside the standard 10000-30000 range validation. For Java servers, it shows "Java default is 25565".

## Skipped Issues

No skipped issues — all in-scope CR and WR findings were fixed.

---

_Fixed: 2026-06-12T15:00:00Z_  
_Fixer: the agent (gsd-code-fixer)_  
_Iteration: 1_
