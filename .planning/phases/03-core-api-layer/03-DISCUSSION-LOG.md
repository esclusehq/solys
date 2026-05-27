# Phase 3: Core API Layer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 3-Core API Layer
**Areas discussed:** Authentication approach, API response patterns, Authorization model, API route structure

---

## Authentication Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Supabase + JWT tokens | Frontend uses Supabase Auth, backend validates JWT tokens from Supabase | ✓ |
| Backend-native auth | Backend handles password hashing and session management | |
| Hybrid approach | Mix of Supabase for frontend and custom JWT for API | |

**User's choice:** Supabase + JWT tokens (Recommended)
**Notes:** Aligns with existing JwtService implementation.

---

## API Response Patterns

| Option | Description | Selected |
|--------|-------------|----------|
| Consistent wrapper | Consistent ApiResponse<T> wrapper with status, data, error fields | ✓ |
| Raw types | Return raw types, let handlers decide on response format | |
| Mixed approach | Use different response types per endpoint based on complexity | |

**User's choice:** Consistent wrapper (Recommended)
**Notes:** Standardizes error handling across API.

---

## Authorization Model

| Option | Description | Selected |
|--------|-------------|----------|
| User ownership | User can only access their own servers; admin can access all | ✓ |
| Team-based access | Allow server sharing between users with roles | |
| Open access | Any authenticated user can access any server | |

**User's choice:** User ownership (Recommended)
**Notes:** Enforces tenant isolation.

---

## API Route Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Resource-based | Group by resource: /api/servers, /api/nodes, /api/users | ✓ |
| Feature-based | Group by feature: /api/servers/*, /api/billing/*, /api/admin/* | |
| Action-based | Flat structure: /api/list-servers, /api/get-server, /api/create-server | |

**User's choice:** Resource-based (Recommended)
**Notes:** Follows RESTful conventions.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
