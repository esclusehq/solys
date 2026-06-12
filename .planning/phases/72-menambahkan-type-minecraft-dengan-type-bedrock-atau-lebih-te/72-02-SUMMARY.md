---
phase: 72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te
plan: 02
subsystem: agent
tags: runtime, port-binding, udp, bedrock, docker, bollard

requires:
  - phase: 72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te
    provides: Research context (RESEARCH.md, PATTERNS.md)

provides:
  - Dynamic port map key in agent_connection.rs (uses game_port instead of hardcoded "25565")
  - loader field forwarded from DeployConfig to runtime task payload
  - UDP protocol dispatch in runtime.rs handle_create (typed payload)
  - UDP protocol dispatch in runtime.rs handle_start (serde_json::Value payload)

affects:
  - 72-01 (agent executor image selection — needs loader to reach agent)
  - 72-03 (frontend Bedrock UI — will trigger this code path)

tech-stack:
  added: []
  patterns:
    - Dynamic Protocol Dispatch: check loader field, select "udp" or "tcp" suffix
    - Port Map Key from Game Port: use game_port as map key, not hardcoded literal
    - loader Field Forwarding: pass DeployConfig.loader through task payload JSON

key-files:
  created: []
  modified:
    - src/agent_connection.rs — Dynamic port key + loader forwarding
    - src/handlers/runtime.rs — UDP protocol dispatch in handle_create and handle_start

key-decisions:
  - "handle_create uses typed payload (payload.loader.as_deref()) for bedrock detection"
  - "handle_start uses serde_json::Value payload (payload.get('loader')) for bedrock detection"
  - "Both handlers default to TCP when loader is missing (backward compatible)"

requirements-completed: [REQ-03, REQ-04]

duration: 7 min
completed: 2026-06-12
---

# Phase 72: Menambahkan Type Minecraft Bedrock — Plan 02 Summary

**Agent-layer UDP port binding for Bedrock containers: dynamic port map key in agent_connection.rs plus loader-based protocol dispatch (UDP/TCP) in runtime.rs handle_create and handle_start**

## Performance

- **Duration:** 7 min
- **Started:** 2026-06-12T12:31:20Z
- **Completed:** 2026-06-12T12:38:32Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Replaced hardcoded `"25565"` port map key with `port.to_string()` in `agent_connection.rs` deploy_config → payload mapping — Bedrock port 19132 now produces `{"19132": ["19132"]}` instead of `{"25565": ["19132"]}`
- `loader` field from `DeployConfig` is now forwarded to the runtime task payload — enables `runtime.rs` `handle_start` (which reads `payload.get("loader")`) to detect Bedrock servers
- `handle_create` in `runtime.rs` now checks `payload.loader.as_deref()` for `"bedrock"` (case-insensitive) — selects `"udp"` protocol suffix for Bedrock, `"tcp"` otherwise
- `handle_start` in `runtime.rs` now checks `payload.get("loader")` for `"bedrock"` (case-insensitive) — same UDP/TCP protocol selection
- Backward compatible: missing or non-Bedrock loader defaults to TCP (existing Java servers unaffected)
- Agent project compiles (`cargo build`) and all existing tests pass (`cargo test`)

## Task Commits

Each task was committed atomically:

1. **Task 1: Forward loader to payload and use dynamic port key in agent_connection.rs** - `a50c80f` (feat)
2. **Task 2: Add UDP protocol dispatch in runtime.rs handle_create and handle_start** - `f9a0353` (feat)

**Plan metadata:** (committed below)

## Files Created/Modified

- `src/agent_connection.rs` — Dynamic port map key using `port.to_string()`; `loader` field forwarded to task payload JSON (21 insertions, 16 deletions)
- `src/handlers/runtime.rs` — `handle_create` uses `payload.loader.as_deref()` for bedrock detection; `handle_start` uses `payload.get("loader")` for bedrock detection; both select UDP/TCP protocol suffix dynamically (12 insertions, 2 deletions)

## Decisions Made

- **handle_create typed vs handle_start untyped:** `handle_create` receives a typed `ServerCreatePayload` struct (which already has `loader: Option<String>`), so it uses `payload.loader.as_deref()`. `handle_start` receives `serde_json::Value` from deserialized task payload, so it uses `payload.get("loader").and_then(|v| v.as_str())`. Both produce the same protocol selection logic.
- **Defaults to TCP:** Both handlers default to TCP when `loader` is missing (`unwrap_or(false)` then `if is_bedrock { "udp" } else { "tcp" }`). This ensures zero behavioral change for existing Java servers that don't forward a loader field.

## Deviations from Plan

None - plan executed exactly as written.

## Threat Surface Scan

No new security-relevant surface introduced beyond what the plan's `<threat_model>` covers:
- T-72-05 (loader field injection): Mitigated — loader drives protocol selection only, worst case is wrong port binding
- T-72-06 (protocol spoofing): Mitigated — loader is server-controlled, not user input
- T-72-07 (malformed port key): Mitigated — format! produces valid Docker port spec, Docker daemon rejects invalid ports

## Known Stubs

No stubs found — both modified files have fully wired implementations.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Agent runtime can now create/start containers with UDP port bindings for Bedrock servers
- `loader` field flows from DeployConfig through agent_connection.rs payload to runtime.rs handlers
- Ready for Plan 72-01 (DB migration + API executor changes) — the agent is ready to receive and handle the "bedrock" loader
- Ready for Plan 72-03 (Frontend Bedrock UI) — independent parallel workstream

## Self-Check: PASSED

- ✅ `src/agent_connection.rs` has no hardcoded `"25565"` in deploy_config ports mapping
- ✅ `src/agent_connection.rs` forwards `loader` via `if let Some(loader) = &config.loader { payload["loader"] = ... }`
- ✅ `src/handlers/runtime.rs` `handle_create` uses `payload.loader.as_deref()` for bedrock detection
- ✅ `src/handlers/runtime.rs` `handle_start` uses `payload.get("loader")` for bedrock detection
- ✅ Both handlers use `let protocol = if is_bedrock { "udp" } else { "tcp" };` — no hardcoded "tcp"
- ✅ No `format!("{}/tcp", ...)` remaining in runtime.rs
- ✅ `cargo build` compiles successfully (only pre-existing warnings)
- ✅ `cargo test` — 1 passed, 0 failed
- ✅ Both task commits exist in git log
- ✅ SUMMARY.md exists at plan directory
- ✅ Deviations: none
- ✅ All plan-level `<success_criteria>` satisfied

## Verification Results

### cargo build
```
Finished `dev` profile [unoptimized + debuginfo] target(s)
```
No new warnings — only 20 pre-existing warnings (dead code, unused imports).

### cargo test
```
running 1 test
test state::tests::test_get_state_path ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
```

### grep checks
- ✅ `agent_connection.rs`: `payload["ports"] = serde_json::json!({ port.to_string(): [port.to_string()] })` — dynamic key
- ✅ `agent_connection.rs`: `if let Some(loader) = &config.loader { payload["loader"] = ... }` — loader forwarded
- ✅ `runtime.rs`: `payload.loader.as_deref()` — handle_create bedrock detection
- ✅ `runtime.rs`: `payload.get("loader")` — handle_start bedrock detection
- ✅ `runtime.rs` x2: `let protocol = if is_bedrock { "udp" } else { "tcp" };` — protocol selection
- ✅ No `format!("{}/tcp")` remaining in runtime.rs

---

*Phase: 72-menambahkan-type-minecraft-dengan-type-bedrock-atau-lebih-te*
*Completed: 2026-06-12*
