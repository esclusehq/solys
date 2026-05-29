# Phase 52: Improve API Documentation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-29
**Phase:** 52-improve-api-docs
**Areas discussed:** Doc Source of Truth, Content depth per endpoint, Auth guide format, SDK guide placement

---

## Doc Source of Truth

| Option | Description | Selected |
|--------|-------------|----------|
| Manual markdown | Keep writing markdown by hand — full control | |
| Auto-generated from OpenAPI | Generate docs from Utoipa annotations | |
| Hybrid | Base structure from OpenAPI + manually written narrative/examples | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| Generated base + manual insertions | Redoc/swagger-ui embedded in VitePress page | |
| Manual pages + auto schema tables | Hand-written markdown that pulls in specific OpenAPI schemas | ✓ |
| You decide | Let the agent/researcher figure out the best hybrid tooling | |

| Option | Description | Selected |
|--------|-------------|----------|
| Build-time auto-fetch | VitePress plugin fetches /openapi.json at build time | ✓ |
| Manual copy + review flag | Copy schemas manually, add review-required flag | |

| Option | Description | Selected |
|--------|-------------|----------|
| All endpoints | Document every public API endpoint | ✓ |
| Core endpoints only | Focus on the 4 existing categories | |

**User's choice:** Hybrid approach: manual markdown pages + auto schema tables from OpenAPI at build time. Cover all endpoints.

---

## Content Depth Per Endpoint

| Option | Description | Selected |
|--------|-------------|----------|
| Standard | Description + JSON example + params listed | |
| Comprehensive | Standard + field-level schemas + curl/JS/Python examples + error codes | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| Curl + JS + Python | Three examples per endpoint | |
| Curl + Node.js SDK + Python SDK | Examples using official SDKs | ✓ |
| Curl + JS + Python + SDK | All four formats | |

| Option | Description | Selected |
|--------|-------------|----------|
| Global reference page + inline per endpoint | Dedicated errors page + inline sections | ✓ |
| Per-endpoint only | Each endpoint documents its own errors | |

**User's choice:** Comprehensive depth with field-level schemas, curl + Node.js SDK + Python SDK examples per endpoint, global error code page + inline per-endpoint errors.

---

## Auth Guide Format

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated auth page | Full /api/auth page with all endpoints and flows | ✓ |
| Expanded overview section | Keep auth in overview but expand it | |

| Option | Description | Selected |
|--------|-------------|----------|
| Endpoints + flow guides | Reference + step-by-step auth flows | ✓ |
| Endpoints only | Reference-style only | |

| Option | Description | Selected |
|--------|-------------|----------|
| Both user + node auth | Document user JWT + node API keys | ✓ |
| User auth only | Focus on Supabase JWT auth | |

**User's choice:** Dedicated auth page with endpoint reference + flow guides. Cover both user JWT and node API key auth.

---

## SDK Guide Placement

| Option | Description | Selected |
|--------|-------------|----------|
| On docs.esluce.com | Full SDK docs on the docs site | |
| In SDK GitHub repos | SDK docs in their own repos | |
| Quickstarts on docs + full ref in repos | Getting-started on docs site, full API ref in repos | ✓ |

**User's choice:** Quickstarts on docs.esluce.com, full reference in GitHub repos.

---

## the agent's Discretion

- VitePress plugin design for OpenAPI schema shortcode
- Specific endpoint grouping on the sidebar/navigation
- Error code ID scheme and categorization

## Deferred Ideas

None
