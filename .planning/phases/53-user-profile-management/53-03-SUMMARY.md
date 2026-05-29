---
phase: 53-user-profile-management
plan: 03
subsystem: backend-api
tags: ["rust", "axum", "handlers", "auth"]
key-files:
  created: []
  modified:
    - api/src/presentation/handlers/auth_handlers.rs
metrics:
  files_created: 0
  files_modified: 1
  total_tasks: 3
  completed_tasks: 3
completed_at: "2026-05-30T01:44:00Z"
---

## 53-03 Summary: Backend HTTP Handlers & Routes

### Tasks Executed

| # | Task | Status | Commit |
|---|------|--------|--------|
| 1 | Extend /auth/me response + add update_profile handler+route | ✓ | 67cb76a (api) |
| 2 | Add login_history, account deletion, cancel-delete, transfer handlers + routes | ✓ | 67cb76a (api) |
| 3 | Add login history tracking (HeaderMap extraction) to login() and oauth() | ✓ | 67cb76a (api) |

### Deviations
- Changed `login_history` handler from `query_as::<_, serde_json::Value>` to `sqlx::query` + manual `Row::get` mapping since `serde_json::Value` does not implement `sqlx::FromRow`
- Added imports for `ServerRepository`, `SqlxServerRepository`, `Server` for the transfer_ownership handler

### Self-Check
**PASSED** — All 6 endpoints added/updated:
- `/auth/me` (GET): returns display_name, avatar_url, scheduled_deletion_at, deletion_scheduled
- `/auth/profile` (PUT): updates display_name and avatar_url
- `/auth/login-history` (GET): returns paginated (100) login history
- `/auth/account/delete` (POST): re-auth + "DELETE" confirm → schedule_deletion
- `/auth/account/cancel-delete` (POST): clears scheduled_deletion_at
- `/auth/account/transfer` (POST): transfers server ownership by email
- login() and oauth(): insert login_history with IP, user-agent, and OAuth provider
- `cargo check` passes
