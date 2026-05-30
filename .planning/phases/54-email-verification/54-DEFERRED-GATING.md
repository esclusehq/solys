# Phase 54: Deferred Gating Strategy

**Created:** 2026-05-30  
**Status:** Final  
**Checker Blockers Resolved:** BLOCKER 1 (D-08 partial coverage)

## Gap Analysis

D-08 defines 4 gated feature categories. This document addresses the categories where no backend handler files exist yet, making immediate VerifiedUser gating impossible.

| Category | D-08 Items | Existing Handlers? | Status |
|----------|-----------|-------------------|--------|
| Financial | Billing, Subscription, Invoices, Payment Methods | `billing_handlers.rs`, `subscription_handlers.rs` | ✓ GATED (Plan 05) |
| Resource Creation | Server Creation, Server Deployment, Node Registration, Instance Creation, Environment Creation | `server_handlers.rs` (+ deployment, node handlers without AuthUser) | ✓ GATED (Plan 05 — server_handlers.rs) |
| Integration:Webhooks | Webhook CRUD, test, retry | `webhook_handlers.rs` | ✓ GATED (Plan 05) |
| **Identity & Access** | **API Keys, Personal Access Tokens, OAuth Applications, Team Invites, Team Creation** | **No handler files exist** | **⏳ DEFERRED** |
| **Integration:Extensions** | **External Integrations, SDK Credentials** | **No handler files exist** | **⏳ DEFERRED** |

## Deferred Categories Detail

### Identity & Access
- **Items:** API Keys, Personal Access Tokens, OAuth Applications, Team Invites, Team Creation
- **Why deferred:** These are future features — no handlers exist in the codebase yet
- **Gating requirement:** When built, ALL handlers in these features MUST use `VerifiedUser` instead of `AuthUser`
- **Pattern:** `use crate::domain::auth::middleware::VerifiedUser;` in the handler file, then `auth_user: VerifiedUser` in each handler parameter

### Integration Extensions
- **Items:** External Integrations, SDK Credentials
- **Why deferred:** These are future features — no handlers exist in the codebase yet
- **Gating requirement:** When built, ALL handlers in these features MUST use `VerifiedUser` instead of `AuthUser`
- **Pattern:** Same as Identity & Access — `VerifiedUser` extractor pattern

## Enforcement During Development

### For future PRs / feature phases:
1. Every new handler file for Identity & Access or Integration Extension features MUST:
   - Import `VerifiedUser` from `crate::domain::auth::middleware`
   - Use `auth_user: VerifiedUser` instead of `auth_user: AuthUser` in all handler signatures
   - NOT use `AuthUser` in any handler (verified user enforcement is non-negotiable per D-08, D-10)
2. The code review checklist should include:
   - "Does this new handler use `VerifiedUser` instead of `AuthUser`?"
   - "Is this feature in a D-08 gated category?"
3. If a handler legitimately requires unverified access (e.g., registration, login, password reset), document the exception

### Template for new gated handlers:
```rust
use crate::domain::auth::middleware::VerifiedUser;

pub async fn new_gated_handler(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,  // D-08: gated behind email verification
    Json(payload): Json<RequestPayload>,
) -> Result<axum::response::Response, ApiError> {
    // handler body — only reached if email_verified_at is not null
}
```

## Relationship to Other Phase Plans

- **Plan 05** (`54-05-PLAN.md`): Covers Financial, Resource Creation, and Webhook gating — uses same `VerifiedUser` extractor
- **Plan 01** (`54-01-PLAN.md`): Creates the `VerifiedUser` extractor in `middleware.rs` — the foundation for all gating
- **Plan 04** (`54-04-PLAN.md`): Frontend gating via `VerifiedRoute` — UX layer complementing backend enforcement

## Verification

When a future phase builds Identity & Access or Integration Extension handlers:
- [ ] All new handlers use `VerifiedUser` extractor
- [ ] No `AuthUser` appears in handler signatures for gated features
- [ ] Exception handlers (registration, login, etc.) document why they use `AuthUser`
- [ ] `cargo check` passes
