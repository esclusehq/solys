---
phase: 66-integrasikan-umami-analitycs-dashboard-dengan-rds-di-project
plan: 01
subsystem: infra
tags: umami, analytics, docker, caddy, rds, postgres, deployment, ec2

requires:
  - phase: 65
    provides: Docker installer script for EC2 (pre-requisite)

provides:
  - Standalone Umami Docker Compose stack (Umami 3.1.0 + Caddy 2) at opt/umami/
  - Caddyfile for analytics.esluce.com with reverse proxy, security headers, TLS
  - .env.example template with DATABASE_URL (sslmode=require), APP_SECRET, tracker config
  - DEPLOYMENT.md guide for manual RDS + DNS + EC2 setup

affects:
  - Phase 66-02 (frontend tracking script injection into landing-page-escluse and app)
  - Frontend deployment (landing-page-escluse/index.html, app/index.html)

tech-stack:
  added:
    - Umami v3.1.0 (ghcr.io/umami-software/umami) — self-hosted analytics
    - Caddy 2 — reverse proxy and TLS termination
  patterns:
    - Standalone Docker Compose stack at /opt/umami/ (not merged into main docker-compose.yml)
    - Caddy security headers matching existing gateway/Caddyfile.prod conventions
    - TRACKER_SCRIPT_NAME and COLLECT_API_ENDPOINT renamed for ad blocker evasion

key-files:
  created:
    - opt/umami/docker-compose.yml — Umami + Caddy stack with health checks
    - opt/umami/Caddyfile — analytics.esluce.com reverse proxy config
    - opt/umami/.env.example — environment variable template
    - DEPLOYMENT.md — complete manual deployment guide

key-decisions:
  - "Umami runs as standalone Docker Compose on EC2 (not merged into main docker-compose.yml)"
  - "RDS PostgreSQL with sslmode=require for encrypted connection"
  - "TRACKER_SCRIPT_NAME=analytics.js, COLLECT_API_ENDPOINT=/api/collect (ad blocker evasion)"
  - "Caddy auto-provisions Let's Encrypt TLS (no manual certificate steps)"
  - "Separate Caddyfile inside /opt/umami stack (not modifying existing gateway/Caddyfile.prod)"
  - "Security headers follow existing Caddyfile.prod patterns (X-Frame-Options, HSTS, -Server)"

requirements-completed: []

duration: 2 min
completed: 2026-05-31
---

# Phase 66: Integrasikan Umami Analytics Dashboard — Plan 1 Summary

**Umami Docker stack (v3.1.0 + Caddy 2) with RDS PostgreSQL deployment configuration and step-by-step manual deployment guide for analytics.esluce.com**

## Performance

- **Duration:** 2 min
- **Started:** 2026-05-31T11:54:42Z
- **Completed:** 2026-05-31T11:56:16Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Created `opt/umami/docker-compose.yml` with Umami v3.1.0 and Caddy 2 services, health check on `/api/heartbeat`, dedicated `umami-net` bridge network, and caddy_data/caddy_config volumes
- Created `opt/umami/Caddyfile` for `analytics.esluce.com` with `reverse_proxy umami:3000`, security headers (X-Frame-Options, X-Content-Type-Options, HSTS, -Server), `encode zstd gzip` compression, and JSON access logs
- Created `opt/umami/.env.example` with DATABASE_URL (sslmode=require), APP_SECRET generation instructions, TRACKER_SCRIPT_NAME=analytics.js, and COLLECT_API_ENDPOINT=/api/collect
- Created `DEPLOYMENT.md` (235 lines) with complete 6-step manual deployment guide covering RDS provisioning, database user setup, DNS configuration, EC2 deployment, first login/configuration, and tracking script injection
- All STRIDE threat model mitigations implemented across the configuration files (T-66-01 through T-66-06)
- Ad blocker evasion configured via renamed tracker script and collect endpoint

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Umami Docker Compose stack and .env template** - `b009e25` (feat)
2. **Task 2: Create Caddyfile for analytics.esluce.com** - `87ab681` (feat)
3. **Task 3: Create deployment documentation (DEPLOYMENT.md)** - `613d78e` (feat)

## Files Created/Modified

- `opt/umami/docker-compose.yml` — Umami + Caddy container stack (64 lines)
- `opt/umami/Caddyfile` — Reverse proxy config for analytics.esluce.com (21 lines)
- `opt/umami/.env.example` — Environment variable template (13 lines)
- `DEPLOYMENT.md` — Step-by-step deployment guide (235 lines)

## Decisions Made

- **Standalone Docker stack:** Umami runs as its own Compose stack at `/opt/umami/` rather than being merged into the main project `docker-compose.yml`, keeping concerns separate per established patterns
- **Ad blocker evasion:** TRACKER_SCRIPT_NAME set to `analytics.js` and COLLECT_API_ENDPOINT to `/api/collect` instead of default paths that appear on ad blocker filter lists
- **Caddy security headers:** Follow the existing `gateway/Caddyfile.prod` patterns for consistency: X-Frame-Options DENY, X-Content-Type-Options nosniff, Strict-Transport-Security with includeSubDomains, -Server directive
- **Separate Caddyfile:** The Umami stack runs its own Caddy instance (not the existing gateway Caddy) to keep the deployment self-contained
- **sslmode=require:** All DATABASE_URL examples use `sslmode=require` to enforce TLS encryption for the Umami→RDS connection

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

None — all threat surface is within the scope of the plan's threat model. No new security-relevant endpoints, auth paths, file access patterns, or schema changes introduced beyond what was modeled.

## Known Stubs

- `opt/umami/.env.example` contains intentional placeholder values (`replace-with-openssl-rand-hex-32-output`, `YOUR_RDS_PASSWORD`, `xxxxxxx` region placeholder) — these are expected for a template/env example file that deployers will customize with actual values.

## Issues Encountered

None.

## User Setup Required

**External services require manual configuration.** The following manual steps are documented in DEPLOYMENT.md:

- **AWS RDS PostgreSQL** — Create RDS instance, configure security group (port 5432 from EC2 SG), create database user
- **Cloudflare DNS** — Add A record analytics.esluce.com pointing to EC2 IP
- **EC2 Docker Deployment** — Copy files, generate APP_SECRET via openssl, run `docker compose up -d`
- **Umami Dashboard** — First login (admin/umami), change password, create websites for each subdomain
- **Frontend Integration** — Inject tracking scripts with website IDs into landing-page-escluse/index.html and app/index.html

## Next Phase Readiness

- All infrastructure configuration files created and committed
- Step-by-step deployment documentation ready for manual execution
- Ready for Plan 66-02 (inject Umami tracking scripts into frontend properties)
- Files at `opt/umami/` are source artifacts — deployer copies them to `/opt/umami/` on EC2 during deployment

## Self-Check: PASSED

- ✅ All 4 created files exist at expected paths
- ✅ All 3 task commits found in git log with `feat(66-01):` prefix
- ✅ SUMMARY.md exists at plan directory
- ✅ Each file passes its acceptance criteria verification

---

*Phase: 66-integrasikan-umami-analitycs-dashboard-dengan-rds-di-project*
*Completed: 2026-05-31*
