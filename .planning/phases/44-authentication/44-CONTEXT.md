# Phase 44: AUTHENTICATION (WAJIB) - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Secure the agent - config file based authentication, registration handshake with backend.
</domain>

<decisions>
## Implementation Decisions

### Config Source
- **D-01:** config.toml primary source for api_key
- **D-02:** Environment variables override config (as fallback)
- **D-05:** backend_url default: wss://app.esluce.com/api/ws/node (user tidak perlu isi)

### Handshake
- **D-03:** Keep current Register → RegisterAck flow
- **D-04:** No additional api_key validation during handshake (trust connection)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `agent-core/crates/agent-config/src/schema.rs` — config schema
- `agent-core/crates/agent-config/src/loader.rs` — config loading
- `web-agent/src/agent_connection.rs` — registration handshake

</canonical_refs>

## Existing Code Insights

### Reusable Assets
- api_key already in AgentConfig schema (SecretString type)
- Config loader supports TOML loading (Phase 39)
- Register/RegisterAck already implemented in agent_connection.rs

### Integration Points
- config.toml loaded early in main()
- Registration happens after WebSocket connection established

</code_context>

<specifics>
## Specific Ideas

User currently uses .env file:
```
AGENT_API_KEY=your-api-key-from-dashboard
```

Want to move to config.toml instead.

</specifics>

<deferred>
## Deferred Ideas

None

</deferred>

---

*Phase: 44-authentication*
*Context gathered: 2026-05-03*