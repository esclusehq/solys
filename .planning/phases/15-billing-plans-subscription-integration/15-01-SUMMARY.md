---
phase: 15-billing-plans-subscription-integration
plan: 01
subsystem: billing
tags: [webhook, security, lemon-squeezy]
dependency_graph:
  requires: []
  provides: [webhook-signature-verification]
  affects: [billing-webhook-handler]
tech_stack:
  - Rust (HMAC-SHA256)
  - subtle crate (constant-time comparison)
key_files:
  created: []
  modified:
    - api/src/infrastructure/billing/lemon_squeezy_service.rs
    - api/src/presentation/handlers/billing_handlers.rs
    - api/Cargo.toml
decisions:
  - "Use constant-time comparison to prevent timing attacks"
  - "Inline signature verification in webhook handler"
metrics:
  tasks: 3
  duration: "~3 min"
  files: 3
---

# Plan 15-01: Webhook Security & Subscription Event Processing

**One-liner:** HMAC-SHA256 webhook signature verification with custom_data user identification

## Completed Tasks

### Task 1: Add HMAC-SHA256 signature verification
- **Status:** ✓ Complete
- **Files:** api/Cargo.toml, api/src/presentation/handlers/billing_handlers.rs

Added subtle crate for constant-time comparison. Webhook handler now verifies X-Signature header before processing JSON.

### Task 2: Integrate signature verification
- **Status:** ✓ Complete
- **Files:** api/src/presentation/handlers/billing_handlers.rs

Webhook handler returns 400 with INVALID_SIGNATURE error if signature doesn't match.

### Task 3: Fix subscription events to use custom_data user_id
- **Status:** ✓ Complete
- **Files:** api/src/presentation/handlers/billing_handlers.rs

Extract user_id from webhook payload's custom_data. Map Lemon Squeezy variant IDs (1490734=Starter, 1517243=Pro) to internal plan IDs.

## Deviances from Plan

None - plan executed exactly as written.

## Auth Gates

None - all work completed without authentication gates.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: auth_bypass | billing_handlers.rs | Webhook endpoint unauthenticated, but signature verification protects |

## Self-Check: PASSED

- [x] Signature verification implemented with HMAC-SHA256
- [x] Invalid signatures return 400
- [x] User identified via custom_data
- [x] Plan mapping from variant IDs