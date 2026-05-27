# Phase 1: Security Foundation - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix critical security vulnerabilities in existing codebase before any feature work. This includes secret management, webhook verification, error handling improvements, and configuration management.
</domain>

<decisions>
## Implementation Decisions

### Webhook Verification (D-01)
- **D-01:** Use HMAC SHA-256 verification for Lemon Squeezy webhook payloads
- Implementation: Verify `X-Signature` header using HMAC with `LEMON_SQUEEZY_WEBHOOK_SECRET`
- Reference: `api/src/domain/billing/webhooks.rs`

### Error Handling Strategy (D-02)
- **D-02:** Replace all `.unwrap()` calls with proper Result types and error propagation
- Scope: Full codebase review, not just critical paths
- Pattern: Use `anyhow::Result` or custom error types with `?` operator

### Configuration Management (D-03)
- **D-03:** Use config crate with validation on load, fail fast on missing required values
- Implementation: Use `config` crate with validation functions
- Required fields: Database URL, JWT secret, Redis URL
- Optional with defaults: API keys, feature flags

### Secret Management Approach
- Use environment variables for all secrets
- No hardcoded secrets in codebase
- Reference: `api/src/config/app_config.rs`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Configuration
- `api/src/config/app_config.rs` — Current config loading logic
- `api/src/bootstrap/container.rs` — AppContainer dependency injection

### Webhooks
- `api/src/domain/billing/webhooks.rs` — Webhook handler implementation

### Error Handling
- `api/src/shared/errors/` — Existing error types (if any)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `api/src/config/app_config.rs` — Existing config loading (to be enhanced)
- `api/src/domain/billing/webhooks.rs` — Webhook handler (needs verification)

### Established Patterns
- Uses `anyhow::Result` for error handling
- Uses `config` crate for configuration (not yet fully utilized)
- Environment variable pattern already in place

### Integration Points
- Webhook verification needs to integrate with billing handlers
- Config validation needs to run at startup before other services
- Error handling improvements need to span all presentation handlers

</code_context>

<specifics>
## Specific Ideas

No additional specifics — decisions captured above provide clear direction for planning.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-security-foundation*
*Context gathered: 2026-04-09*
