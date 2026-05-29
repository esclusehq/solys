# Phase 52: Improve API Documentation - Context

**Gathered:** 2026-05-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Enhance the existing API documentation at https://docs.esluce.com/api/ with comprehensive, developer-friendly docs covering all public REST API endpoints. Includes detailed descriptions, field-level request/response schemas, curl + SDK code examples per endpoint, dedicated auth guide with step-by-step flows, global error code catalog, and SDK quickstart guides for Node.js and Python.

</domain>

<decisions>
## Implementation Decisions

### Doc Source of Truth (D-01 to D-04)

- **D-01:** Hybrid approach — manually written markdown pages for narrative and examples, with auto-generated schema tables pulled from OpenAPI spec
- **D-02:** Schema tables fetched at build time from the API's `/openapi.json` endpoint via a VitePress plugin/shortcode — always in sync on deploy
- **D-03:** Cover ALL public API endpoints — auth, servers, nodes, billing, settings (S3, Cloudflare), alerts, templates, plugins, deployments, WebSocket, file management, git, build, runtimes, cron tasks
- **D-04:** Docs are public — they describe the HTTP interface only, not proprietary backend implementation. Standard practice (Stripe, GitHub, Twilio model). Internal/admin-only endpoints excluded or labeled "Internal"

### Content Depth Per Endpoint (D-05 to D-07)

- **D-05:** Comprehensive detail per endpoint — HTTP method + path, description, field-level schema tables (name, type, required, description, default, possible values), full JSON request/response examples
- **D-06:** Code examples in three formats: curl (raw HTTP), Node.js SDK, Python SDK
- **D-07:** Error codes documented globally in a dedicated `/api/errors` reference page + inline "Possible errors" section per endpoint linking to the global catalog

### Auth Guide (D-08 to D-10)

- **D-08:** Dedicated `/api/auth` page (separate from overview)
- **D-09:** Endpoint reference + step-by-step flow guides: login with email/password, OAuth (Google/GitHub), token refresh, email verification, password reset, MFA
- **D-10:** Document both user authentication (Supabase JWT) and node API key auth

### SDK Guides (D-11)

- **D-11:** Quickstart guides and basic examples on docs.esluce.com at `/api/sdks/node` and `/api/sdks/python`. Full API reference and advanced usage lives in each SDK's GitHub repo

### the agent's Discretion
- VitePress plugin design for OpenAPI schema shortcode — implementer chooses the technical approach
- Specific endpoint grouping on the sidebar/navigation
- Error code ID scheme and categorization
- Whether to include a "Try it" / Swagger UI embed on the docs site (already available at api.esluce.com/docs and /redoc)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing API Docs
- `docs/api/overview.md` — current API overview page (will be restructured)
- `docs/api/servers.md` — current servers API page (reference for depth)
- `docs/api/nodes.md` — current nodes API page
- `docs/api/billing.md` — current billing API page

### API Backend (source of truth for endpoints)
- `api/src/presentation/routes/api_routes.rs` — all API v1 route definitions
- `api/src/presentation/routes/server_routes.rs` — additional server routes (files, git, build, terminal)
- `api/src/presentation/routes/openapi_routes.rs` — OpenAPI/Swagger/Redoc endpoints
- `api/src/presentation/handlers/auth_handlers.rs` — auth endpoint handlers

### Existing Infrastructure
- `docs/.vitepress/config.mts` — VitePress configuration (sidebar, nav)
- `docs/index.md` — docs site landing page
- `docs/Dockerfile` — docs deployment setup

### OpenAPI / SDK
- `https://api.esluce.com/openapi.json` — live OpenAPI spec (auto-generated from Utoipa)
- `https://github.com/escluse/sdk-node` — Node.js SDK repo
- `https://github.com/escluse/sdk-python` — Python SDK repo
- `https://github.com/escluse/escluse` — main monorepo (agent-core, shared proto types)

### Prior Phase Context
- `.planning/phases/47-docs-website/47-CONTEXT.md` — docs site setup decisions (VitePress)
- `.planning/codebase/CONVENTIONS.md` — existing code conventions
- `.planning/codebase/STRUCTURE.md` — codebase directory layout

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `docs/api/overview.md`, `servers.md`, `nodes.md`, `billing.md` — existing base pages to expand
- `docs/.vitepress/` — existing VitePress config with sidebar, search, theme
- `api/src/presentation/routes/openapi_routes.rs` — Utoipa OpenAPI generation with Swagger UI + Redoc

### Established Patterns
- VitePress markdown pages with Vue components for interactive features
- RESTful API design with consistent response format (`{ data, status, message }`)
- Auth via Bearer JWT tokens from Supabase

### Integration Points
- New docs pages go in `docs/api/` directory
- Sidebar config in `docs/.vitepress/config.mts` for navigation
- Auto-generated schema tables need a custom VitePress component that fetches `/openapi.json`

</code_context>

<specifics>
## Specific Ideas

Docs should be public — describing the HTTP interface, not proprietary backend code. Internal/admin-only endpoints should be excluded or marked.

Pipeline flow: Developer reads docs → finds endpoint → sees curl example → uses SDK → handles errors via catalog → authenticates via auth guide.

</specifics>

<deferred>
None — discussion stayed within phase scope

</deferred>

---

*Phase: 52-improve-api-docs*
*Context gathered: 2026-05-29*
