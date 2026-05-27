---
status: awaiting_human_verify
trigger: "GET /api/servers/:id/logs returns null/empty instead of actual log content"
created: 2026-04-09T13:30:00Z
updated: 2026-04-09T14:00:00Z
---

## Current Focus

hypothesis: Root cause found - Docker client not available in web-agent
test: Apply fix to create fresh Docker client on each logs request
expecting: Logs will now be returned correctly from container
next_action: User needs to rebuild and test the web-agent

## Root Cause

The user confirmed: "Debug shows command sent to agent, but response is output=null from agent"

**Root cause:** The `RuntimeDetector` passed to `handle_logs` has a `docker_client` that is either:
1. None (not initialized)
2. A stale/closed connection

The code at line 465 was: `let docker = runtime.docker().context("Docker client not available")?;`

This would either return an error or potentially return a client that couldn't communicate with Docker.

## Fix Applied

**File:** `web-agent/src/handlers/runtime.rs`

**Change:** Modified `handle_logs` function to always create a fresh Docker client on each request instead of relying on the cached client from RuntimeDetector:

```rust
// OLD (broken):
let docker = runtime.docker().context("Docker client not available")?;

// NEW (fixed):
let docker = bollard::Docker::connect_with_local_defaults()
    .context("Failed to connect to Docker. Is Docker running?")?;
```

This ensures each logs request creates a fresh connection to the Docker socket, avoiding stale connection issues.

## Files Changed

- web-agent/src/handlers/runtime.rs - Modified handle_logs to create fresh Docker client

## Verification Steps

1. Rebuild the web-agent: `cargo build` in web-agent folder
2. Restart the web-agent with Docker running
3. Call the logs endpoint: `curl http://localhost:3000/api/v1/servers/{server_id}/logs/100`
4. Verify logs are now returned

**Tell me:** Does the fix work now?