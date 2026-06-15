---
phase: 72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te
plan: 01
subsystem: api
tags: bedrock, game-types, migration, docker, agent-executor

# Dependency graph
requires:
  - phase: 05
    provides: game_types table, fallback pattern, agent executor pattern
provides:
  - Bedrock game_types row in migration SQL (itzg/minecraft-bedrock-server, UDP 19132, no RCON)
  - GameType::fallback("bedrock") returning bedrock image and game-only ports
  - create_server_use_case mapping game_type="bedrock" → mc_loader="bedrock"
  - AgentServerExecutor dynamic image/RCON/env dispatch based on mc_loader
  - server_handlers mc_loader reference in config when game_type is bedrock/minecraft-bedrock
affects:
  - 72-02 (agent runtime UDP port binding — depends on deploy_config with loader field)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dynamic image dispatch: mc_loader.eq_ignore_ascii_case('bedrock') → bedrock vs java image"
    - "Game type → mc_loader mapping in use case: game_type override prevents conflicting data"
    - "UDP-only port config: rcon_port = None for bedrock (no RCON)"
    - "Bedrock env vars: GAMEMODE=survival, DIFFICULTY=normal as default stack"

key-files:
  created:
    - api/migrations/20260612000001_add_bedrock_game_type.sql
  modified:
    - api/src/domain/server/entities/game_type.rs
    - api/src/application/use_cases/create_server_use_case.rs
    - api/src/infrastructure/executors/agent_server_executor.rs
    - api/src/presentation/handlers/server_handlers.rs

key-decisions:
  - "Bedrock capabilities in fallback() keep rcon: true (shared across all fallback arms — fallback is only invoked on DB query failure; DB row has correct rcon: false)"
  - "handler mc_loader mapping uses match (not if-else) for future extensibility"
  - "server.mc_loader.eq_ignore_ascii_case('bedrock') for case-insensitive comparison"
  - "env_vars for bedrock include GAMEMODE and DIFFICULTY defaults per RESEARCH.md standard stack"
  - "Sort order 6 for bedrock (after forge which is 5)"

requirements-completed: [REQ-02, REQ-05]

# Metrics
duration: 5 min
completed: 2026-06-12
---

# Phase 72: Menambahkan Type Minecraft Bedrock — Plan 01 Summary

**Bedrock game type migration, GameType fallback arm, game_type→mc_loader mapping in use case, dynamic image/RCON/env dispatch in agent executor, and handler mc_loader reference**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-12T12:24:29Z
- **Completed:** 2026-06-12T12:29:32Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Created migration `20260612000001_add_bedrock_game_type.sql` inserting bedrock row with `itzg/minecraft-bedrock-server:latest`, UDP port 19132 only, `rcon: false`, sort_order 6
- Added `"bedrock"` match arm to `GameType::fallback()` returning bedrock image and UDP-only `{"game": 19132}` ports
- Modified `create_server_use_case.rs` to force `mc_loader: "bedrock"` when `req.game_type == "bedrock"`, preventing conflicting `mc_loader: "PAPER"` default (Pitfall 6 mitigation)
- Replaced hardcoded Java image in `AgentServerExecutor::build_deploy_config()` with dynamic dispatch: checks `server.mc_loader.eq_ignore_ascii_case("bedrock")` to select correct image, disable RCON, and add bedrock-specific env vars
- Added mc_loader mapping in `server_handlers.rs` create_server handler: `game_type = "bedrock"` or `"minecraft-bedrock"` sets `config["mc_loader"] = "bedrock"` for executor dispatch
- All changes compile successfully (`cargo check` — only pre-existing warnings)

## Task Commits

Each task was committed atomically to the `api/` sub-repo:

1. **Task 1: Create migration to add bedrock game_types row** — `api@8f0df56` (feat)
2. **Task 2: Add bedrock arm to GameType fallback() and map game_type→mc_loader** — `api@ae1e387` (feat)
3. **Task 3: Dynamic image dispatch in executor + mc_loader mapping in handler** — `api@c15f318` (feat)

## Files Created/Modified
- `api/migrations/20260612000001_add_bedrock_game_type.sql` — Created: Bedrock game_types INSERT with correct Docker image, UDP port 19132, no RCON
- `api/src/domain/server/entities/game_type.rs` — Modified: Added `"bedrock"` match arm in `fallback()` before catch-all
- `api/src/application/use_cases/create_server_use_case.rs` — Modified: game_type bedrock check forces mc_loader to "bedrock"
- `api/src/infrastructure/executors/agent_server_executor.rs` — Modified: Dynamic image/RCON/env dispatch in `build_deploy_config()`
- `api/src/presentation/handlers/server_handlers.rs` — Modified: game_type→mc_loader mapping in create_server handler

## Decisions Made
- **fallback() capabilities not dynamic for bedrock:** `GameType::fallback()` keeps `"rcon": true` in shared capabilities for all types. This is acceptable because `fallback()` is only called when DB query fails; the DB row has correct `{"rcon": false, "backup": true}`. Making capabilities dynamic adds unnecessary complexity.
- **match statement in handler (not if-else):** Using `match payload.game_type.as_deref() { Some("bedrock") | Some("minecraft-bedrock") => ... }` makes it easy to add future game types that need mc_loader overrides.
- **eq_ignore_ascii_case for bedrock detection:** Prevents case-sensitivity issues with user-submitted data like "BEDROCK" or "Bedrock".
- **Default Bedrock env vars:** `GAMEMODE=survival`, `DIFFICULTY=normal` per RESEARCH.md standard stack. `ALLOW_CHEATS` is intentionally not set — left for user configuration.
- **Staged approach:** This plan covers only DB + API layers. Agent runtime changes (UDP binding) and frontend updates are in Plans 72-02 and 72-03.

## Deviations from Plan

None - plan executed exactly as written.

### Threat Model Compliance
- **T-72-01** (Tampering - game_type validation): Handler uses tight `match` with exact string comparison. Only "bedrock" and "minecraft-bedrock" trigger bedrock mc_loader. All other values use default path.
- **T-72-02** (Tampering - mc_loader override): Use case uses `as_deref() == Some("bedrock")` — only exact string "bedrock" triggers override. No regex, no substring match.
- **T-72-03** (Information Disclosure): Migration file is committed to git. Game types are public lookup data with no secrets.
- **T-72-04** (Elevation of Privilege): Default env vars are game configuration defaults, not security controls. `ALLOW_CHEATS` is not set by default.

## Issues Encountered
- `api/` directory is a separate embedded git repo (not a submodule), excluded via `.gitignore`. All commits were made to the `api/` sub-repo directly. The parent repo's `.gitignore` excludes `api/`, so no parent-level changes needed for these files.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- DB migration ready for bedrock game_types row (can be applied to RDS)
- API layer dispatches correct image/RCON/env for bedrock servers
- Ready for Plan 72-02 (agent runtime: UDP port binding in runtime.rs + dynamic port map key in agent_connection.rs)
- Ready for Plan 72-03 (frontend: CreateServerModal bedrock option with conditional fields)

## Self-Check: PASSED

- ✅ `api/migrations/20260612000001_add_bedrock_game_type.sql` exists with bedrock row (itzg/minecraft-bedrock-server:latest, UDP 19132, rcon:false, sort_order 6)
- ✅ `api/src/domain/server/entities/game_type.rs` has `"bedrock" =>` match arm before catch-all, ports no rcon key
- ✅ `api/src/application/use_cases/create_server_use_case.rs` checks `game_type.as_deref() == Some("bedrock")` and returns `"bedrock".to_string()` for mc_loader
- ✅ `api/src/infrastructure/executors/agent_server_executor.rs` has `is_bedrock` check, dynamic image, rcon_port=None for bedrock, GAMEMODE/DIFFICULTY env vars
- ✅ `api/src/presentation/handlers/server_handlers.rs` has `Some("bedrock")` match in game_type block and sets `config["mc_loader"]`
- ✅ `cargo check` compiles successfully (api/ sub-repo)
- ✅ All 3 task commits found in api sub-repo git log

---

*Phase: 72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te*
*Completed: 2026-06-12*
