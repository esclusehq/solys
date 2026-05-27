---
phase: 44-authentication
plan: "01"
subsystem: agent-config
tags:
  - config
  - authentication
  - api_key
dependency_graph:
  requires: []
  provides:
    - api_key loaded from TOML config file
  affects:
    - agent-core/crates/agent-config/src/loader.rs
    - web-agent/src/agent_connection.rs
tech_stack:
  added: []
  patterns:
    - SecretString wrapper for sensitive config values
    - TOML config file with XDG path discovery
    - Environment variable override precedence
key_files:
  created: []
  modified:
    - agent-core/crates/agent-config/src/loader.rs
decisions:
  - D-01: config.toml is primary source for api_key
  - D-02: ESCLUSE_AGENT_* env vars override TOML values
  - D-04: No additional api_key validation during handshake (trust connection)
metrics:
  duration: ~
  completed: 2026-05-03
  tasks_completed: 3
---

# Phase 44 Plan 1: Authentication - API Key from Config TOML Summary

One-liner: Added config.toml support for api_key loading - agents can now authenticate using file-based credentials in addition to environment variables.

## Overview

Implemented api_key loading from config.toml [server] section, completing the config.toml integration for authentication. Agents can now use file-based credentials instead of only .env files.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add api_key to TOML config loader | 2082a07 | loader.rs |
| 2 | Verify config loading order | 2082a07 | loader.rs |
| 3 | Verify integration with agent_connection | N/A | agent_connection.rs |

## Changes Made

### api_key from TOML config

**File:** `agent-core/crates/agent-config/src/loader.rs`

Added api_key parsing from TOML [server] section:

```rust
// [server] - api_key (D-01: primary source from config.toml)
if let Some(v) = toml_map.get("server").and_then(|t| t.get("api_key")) {
    if let Some(s) = v.as_str() {
        if !s.is_empty() {
            config.api_key = SecretString::new(s.to_string());
        }
    }
}
```

### Config Loading Order (documented)

1. TOML file (config.toml) - D-01: primary source
2. Old-style AGENT_* env vars (backward compatibility)
3. Legacy .env file
4. ESCLUSE_AGENT_* env vars (takes precedence) - D-02: env override

## Verification

- Build passes: `cargo build --package agent-config`
- Test passes: `cargo test --package agent-config -- test_load_from_env`
- Env override works: ESCLUSE_AGENT_API_KEY overrides TOML value

## Success Criteria

- [x] api_key loads from config.toml file
- [x] ESCLUSE_AGENT_API_KEY env var overrides TOML value  
- [x] Config loading order is correct (TOML → env override)
- [x] Register handshake works without additional api_key validation (D-04)

## Example config.toml

```toml
[server]
api_key = "your-api-key-from-dashboard"
backend_url = "wss://app.esluce.com/api/ws/node"
```

## Deviations from Plan

None - plan executed exactly as written.

---