# Phase 3: Core API Layer - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

User authentication and server management API endpoints. This phase establishes the core API surface for users to authenticate, list their servers, and view server details.

**Requirements addressed:** AUTH-01, DEPLOY-01 (partial), STATUS-01 (partial)

**Success criteria:**
1. User can authenticate via Supabase auth integration
2. User can list all their game servers
3. User can view server details
4. API returns proper HTTP status codes and error messages
</domain>

<decisions>
## Implementation Decisions

### Authentication Approach (D-08)
- **D-08:** Supabase + JWT tokens
- Frontend uses Supabase Auth for user login
- Backend validates JWT tokens from Supabase
- Existing implementation: `api/src/domain/auth/service.rs` with JwtService
- Token validation via Bearer header in middleware

### API Response Patterns (D-09)
- **D-09:** Consistent ApiResponse<T> wrapper
- Use existing ApiResponse<T> wrapper with status, data, error fields
- Standardize error responses across all handlers
- Reference: `api/src/presentation/responses/api_response.rs`

### Authorization Model (D-10)
- **D-10:** User ownership model
- User can only access their own servers
- Admin can access all servers (role-based)
- Server ownership enforced via user_id column in servers table
- Reference: `api/src/domain/rbac/middleware.rs`

### API Route Structure (D-11)
- **D-11:** Resource-based organization
- Group by resource: `/api/servers`, `/api/nodes`, `/api/users`
- RESTful patterns: GET/POST/PUT/DELETE per resource
- Reference: `api/src/presentation/routes/`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Authentication
- `api/src/domain/auth/service.rs` — JwtService implementation
- `api/src/domain/auth/middleware.rs` — Token validation middleware
- `app/src/lib/supabase.js` — Frontend Supabase client

### API Responses
- `api/src/presentation/responses/api_response.rs` — ApiResponse wrapper

### Authorization
- `api/src/domain/rbac/middleware.rs` — RBAC middleware
- `api/src/presentation/middleware/auth.rs` — Auth middleware

### Routes
- `api/src/presentation/routes/server_routes.rs` — Server routes
- `api/src/presentation/routes/api_routes.rs` — Route aggregation

### Handlers
- `api/src/presentation/handlers/auth_handlers.rs` — Auth endpoints
- `api/src/presentation/handlers/server_handlers.rs` — Server list/get endpoints

</canonical_refs>

<specifics>
## Specific Ideas

- Existing auth handlers already implement login/register
- Server list and get endpoints already exist
- Need to verify Supabase integration is properly wired
- Need to ensure user_id filtering is applied to list queries

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 03-core-api-layer*
*Context gathered: 2026-04-09*
