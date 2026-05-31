# Phase 66: Integrasikan Umami Analytics Dashboard dengan RDS di Project Esluse - Context

**Gathered:** 2026-05-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Deploy and configure a self-hosted Umami Analytics instance on AWS EC2 with RDS PostgreSQL for tracking website traffic across all esluce.com subdomains (landing page, app, API, docs). Includes Docker deployment, RDS setup, Caddy reverse proxy with SSL, and embedding the Umami tracking script into the Esluce frontend properties.

</domain>

<decisions>
## Implementation Decisions

### Deployment Architecture
- **D-01:** Deploy Umami via Docker on EC2 (official method, easiest updates)
- **D-02:** Use a dedicated new RDS PostgreSQL instance for Umami's database
- **D-03:** Subdomain will be `analytics.esluce.com`

### SSL & Reverse Proxy
- **D-04:** Use Caddy for TLS termination (matches existing project stack)

### Tracking Scope
- **D-05:** Track all subdomains — landing page (`esluce.com`), app (`app.esluce.com`), API, docs

### Automation
- **D-06:** Manual setup with clear documentation (no Terraform/Ansible)

### Agent's Discretion
- EC2 instance type/size (t3.medium or similar recommended)
- Umami version pinning (latest stable)
- RDS instance class and storage allocation
- Tracking script placement and configuration details

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Infrastructure
- `.planning/codebase/STACK.md` — Current tech stack (PostgreSQL 16, Caddy reverse proxy, Docker Compose)
- `.planning/codebase/INTEGRATIONS.md` — Existing external integrations (PostgreSQL, Redis, S3, Supabase)
- `.planning/codebase/ARCHITECTURE.md` — System architecture and deployment patterns
- `gateway/Caddyfile.prod` — Existing Caddy configuration for esluce.com domains

### Frontend Properties to Track
- `landing-page-escluse/` — Landing page source (esluce.com)
- `app/` — React dashboard app (app.esluce.com)

### Umami Documentation
- https://umami.is/docs/ — Official Umami deployment and configuration docs

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Caddy config pattern** — `gateway/Caddyfile.prod` shows existing TLS/reverse proxy setup that can be adapted for analytics.esluce.com
- **Docker Compose** — Existing `docker-compose.yml` shows project's containerization conventions

### Established Patterns
- **Self-hosted infra** — The project already self-hosts its own PostgreSQL, Redis, and Caddy on the server
- **Environment variables** — `.env` files used throughout the project for configuration

### Integration Points
- **Landing page** — `landing-page-escluse/` will need the Umami tracking script added to its HTML
- **React app** — `app/` will need the Umami tracking script added (possibly via react-umami or direct script tag)
- **DNS** — New A record needed: `analytics.esluce.com` → EC2 IP

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard Umami deployment approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 66-integrasikan-umami-analitycs-dashboard-dengan-rds-di-project*
*Context gathered: 2026-05-31*
