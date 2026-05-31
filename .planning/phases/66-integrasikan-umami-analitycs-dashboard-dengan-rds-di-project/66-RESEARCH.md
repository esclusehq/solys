# Phase 66: Integrasikan Umami Analytics Dashboard dengan RDS — Research

**Researched:** 2026-05-31
**Domain:** Self-hosted web analytics deployment (Umami v3.1.0 + RDS PostgreSQL + Caddy reverse proxy + tracking script integration)
**Confidence:** HIGH

## Summary

This phase deploys a self-hosted Umami Analytics instance for tracking all esluce.com subdomains. The deployment follows a three-part architecture: (1) a dedicated RDS PostgreSQL instance stores analytics data, (2) an Umami Docker container serves the dashboard and tracking script, and (3) Caddy terminates TLS at `analytics.esluce.com`. Both frontends (landing page and React app) are SPAs — Umami's tracking script automatically handles client-side navigation via the History API, requiring no additional JavaScript.

**Primary recommendation:** Run Umami as a standalone Docker Compose stack on EC2 with Umami + Caddy containers, connecting to an external RDS PostgreSQL instance. Add the tracking script as a `<script>` tag in both `index.html` files for the SPA frontends.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Analytics dashboard hosting | EC2 Docker | — | Umami is a Node.js app served via Docker; Caddy handles TLS |
| Analytics data storage | RDS PostgreSQL | — | D-02: dedicated RDS instance; distinct from main app's local Postgres |
| TLS termination | Caddy (on EC2) | — | D-04: matches existing Caddy stack; auto-provisions Let's Encrypt certs |
| Tracking script delivery | EC2 Docker (Umami) | — | Umami serves `/script.js` directly from its container; Caddy proxies |
| Tracking script injection | Browser / Client | Frontend build | Script tag in `index.html` is client-side; website IDs are build-time VITE env vars |
| DNS resolution | Cloudflare | — | A record: `analytics.esluce.com` → EC2 IP |
| Page view detection | Browser (Umami script) | — | Script monitors History API automatically for SPA navigation |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Umami | v3.1.0 | Self-hosted web analytics | Privacy-focused GA alternative; Docker image from GHCR |
| PostgreSQL | 16 (RDS) | Analytics data store | Umami v3+ supports PostgreSQL only (MySQL dropped); RDS matches managed infra pattern |
| Caddy | 2 (latest) | TLS termination, reverse proxy | D-04; matches existing project Caddy stack; auto-SSL |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|--------------|
| `@danielgtmn/umami-react` | latest | React component for Umami tracking | Optional convenience wrapper; zero deps, React 19 support |
| `@giof/react-umami` | v2.6.12 | React component for Umami tracking | Alternative; has env-var auto-detection, dry-run/testing support |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `@danielgtmn/umami-react` / `@giof/react-umami` | Plain `<script>` tag | Script tag is simpler (no npm dep); both React SPAs track SPA nav automatically. No npm package needed — raw `<script>` tag suffices. |

**Installation (EC2):**
```bash
# Umami is Docker-only deployment on EC2
# Pull image:
docker pull ghcr.io/umami-software/umami:v3.1.0

# Or use postgresql-latest tag (auto-update friendly):
docker pull ghcr.io/umami-software/umami:postgresql-latest
```

**Version verification:**
```bash
# Verified: npm registry for GHCR tag availability
# Umami v3.1.0 — released 2026-04-16 (latest stable)
# Docker images at: ghcr.io/umami-software/umami
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│  Cloudflare DNS                                          │
│  analytics.esluce.com A → EC2 IP                         │
└──────────┬────────────────────────────────────────────┘
           │
           ▼
┌────────────────────── EC2 Instance ─────────────────────┐
│                                                          │
│  ┌──────────┐    port 443     ┌──────────────────────┐  │
│  │ Browser  │ ──────────────► │   Caddy (container)  │  │
│  │ (visits  │                 │   TLS termination    │  │
│  │  site)   │                 │   auto Let's Encrypt │  │
│  └──────────┘                 └──────┬───────────────┘  │
│                                      │ proxy_pass        │
│  ┌───────────────────┐              │                   │
│  │ Umami (container) │ ◄────────────┘                   │
│  │                   │                                   │
│  │ - Dashboard UI   │   port 3000 (internal)            │
│  │ - Tracking API   │                                    │
│  │ - /script.js     │                                    │
│  │ - /api/send      │                                    │
│  └────────┬─────────┘                                    │
└───────────┼──────────────────────────────────────────────┘
            │ DATABASE_URL (SSL)
            ▼
┌────────────────────── RDS PostgreSQL ────────────────────┐
│  umami database (dedicated instance)                      │
│  - Tables auto-created via Prisma migration on 1st boot   │
│  - Security group: allow 5432 from EC2 SG                 │
└──────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Frontend properties (tracking script injection)          │
│                                                           │
│  esluce.com ────────► landing-page-escluse/index.html    │
│                        <script src="…/script.js"          │
│                               data-website-id="ABCD" />  │
│                                                           │
│  app.esluce.com ────► app/index.html                     │
│                        <script src="…/script.js"          │
│                               data-website-id="EFGH" />  │
│                                                           │
│  (Umami auto-tracks SPA navigation via History API)       │
└──────────────────────────────────────────────────────────┘
```

### Deployment Pattern: Standalone Docker Compose on EC2

Umami runs as a separate Docker Compose stack on the same EC2 instance, NOT merged into the existing project `docker-compose.yml`. This keeps concerns separate and follows the project's pattern of independent services.

**Recommended project structure on EC2:**
```
/opt/umami/
├── docker-compose.yml    # Umami + Caddy containers
├── Caddyfile             # Caddy config for analytics.esluce.com
├── .env                  # DATABASE_URL, APP_SECRET, etc.
└── backups/              # pg_dump backups
```

### Pattern 1: Umami + Caddy Docker Compose (external RDS)
**What:** Umami container connected to external RDS, with Caddy for TLS. No bundled PostgreSQL.
**When to use:** When D-02 mandates dedicated RDS (not local Postgres container).
**Example:**
```yaml
# /opt/umami/docker-compose.yml
services:
  umami:
    image: ghcr.io/umami-software/umami:v3.1.0
    container_name: umami
    restart: unless-stopped
    expose:
      - "3000"   # internal only; Caddy proxies
    environment:
      DATABASE_URL: "${DATABASE_URL}"
      APP_SECRET: "${APP_SECRET}"
      DISABLE_TELEMETRY: "1"
      DISABLE_UPDATES: "1"
      TRACKER_SCRIPT_NAME: "script.js"
    healthcheck:
      test: ["CMD-SHELL", "wget --spider -q http://localhost:3000/api/heartbeat || exit 1"]
      interval: 30s
      timeout: 5s
      retries: 3
    networks:
      - umami-net

  caddy:
    image: caddy:2
    container_name: umami-caddy
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
      - "443:443/udp"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - umami
    networks:
      - umami-net

networks:
  umami-net:
    driver: bridge

volumes:
  caddy_data:
  caddy_config:
```

### Pattern 2: Caddy Reverse Proxy for analytics.esluce.com
**What:** Caddy block matching existing project Caddyfile style.
**When to use:** D-04; must follow gateway/Caddyfile.prod conventions.
**Example:**
```caddy
# Caddyfile (/opt/umami/Caddyfile)
analytics.esluce.com {
    encode zstd gzip

    reverse_proxy umami:3000 {
        health_uri /api/heartbeat
        health_interval 30s
    }

    header {
        X-Frame-Options DENY
        X-Content-Type-Options nosniff
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        -Server
    }

    log {
        output file /var/log/caddy/analytics.log
    }
}
```

Note: This Caddyfile runs **inside** the Umami Docker Compose stack, NOT in the project's gateway Caddy. This keeps the Umami deployment self-contained and doesn't require changes to the existing gateway infrastructure.

### Pattern 3: React SPA Tracking (script tag)
**What:** Add Umami tracking script to both frontend `index.html` files. Umami automatically detects SPA navigation via History API (`pushState`/`replaceState` + `popstate`).
**When to use:** Both `landing-page-escluse/` and `app/` are React SPAs with React Router.
**Example (landing-page-escluse/index.html):**
```html
<head>
  <script
    defer
    src="https://analytics.esluce.com/script.js"
    data-website-id="LANDING_PAGE_WEBSITE_ID"
    data-domains="esluce.com"
  ></script>
</head>
```

**Example (app/index.html):**
```html
<head>
  <script
    defer
    src="https://analytics.esluce.com/script.js"
    data-website-id="APP_WEBSITE_ID"
    data-domains="app.esluce.com"
  ></script>
</head>
```

**VITE env var approach** (optional, for build-time injection):
```typescript
// In index.html, injected at build time:
<script
  defer
  src="%VITE_UMAMI_SCRIPT_URL%/script.js"
  data-website-id="%VITE_UMAMI_WEBSITE_ID%"
></script>
```

### Anti-Patterns to Avoid
- **Merging Umami into existing docker-compose.yml:** Umami has its own database requirement and lifecycle. Keep it as a separate stack at `/opt/umami/`.
- **Using a React npm package for tracking:** Umami's script tag works out of the box with SPAs. Adding `@giof/react-umami` or similar is unnecessary complexity.
- **Exposing Umami port 3000 publicly:** Always bind to internal Docker network and proxy through Caddy. Binding to `127.0.0.1` only is an even stronger pattern.
- **Hardcoding website IDs:** Website IDs are created inside Umami after first login. The plan must account for creating websites, getting their IDs, then injecting into frontend builds.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Web analytics | Custom analytics solution | Umami v3.1.0 | Open source (MIT), 37K+ GitHub stars, 2KB tracking script, no cookies, GDPR-compliant, SPA auto-detection |
| TLS certificate management | Manual cert generation/renewal | Caddy 2 | Auto-provisions and renews Let's Encrypt certificates; matches existing project stack |
| Database migration for Umami's schema | Writing schema SQL | Umami built-in (Prisma) | Umami runs Prisma migrations automatically on container startup — no manual migration scripts needed |

**Key insight:** Umami handles all analytics complexity (data collection, storage aggregation, dashboard rendering) with a single container. The tracking script is 2KB, sets no cookies, and works with SPAs automatically. There is zero justification for building custom analytics infrastructure.

## Runtime State Inventory

> Omit — this is a greenfield deployment, not a rename/refactor phase.

## Common Pitfalls

### Pitfall 1: RDS Connection Failures on First Boot
**What goes wrong:** Umami container fails to start because it can't connect to RDS.
**Why it happens:** (a) Security group doesn't allow inbound 5432 from EC2, (b) `sslmode=require` not set in DATABASE_URL, (c) database/role doesn't exist yet.
**How to avoid:** Before launching Umami, (1) create the RDS security group rule allowing postgres access from the EC2 security group, (2) create the `umami` database and role on RDS manually, (3) use `postgresql://umami:password@rds-endpoint:5432/umami?sslmode=require`.
**Warning signs:** `docker logs umami` shows `Error: Can't reach database server` or connection timeouts.

### Pitfall 2: Tracker Script Blocked by Ad Blockers
**What goes wrong:** `/script.js` is a known Umami tracking URL, flagged by uBlock Origin and similar.
**Why it happens:** Default tracking script URL patterns are in ad blocker filter lists.
**How to avoid:** Set `TRACKER_SCRIPT_NAME` to an innocuous name like `analytics.js`, `stats.js`, or a path like `/assets/tracker.js`. Also set `COLLECT_API_ENDPOINT` to rename `/api/send` (default) to something like `/api/collect`.
**Warning signs:** Network tab shows script loaded, but no POST to `/api/send` — ad blocker silently intercepts.

### Pitfall 3: Forgetting to Create Umami Website After Login
**What goes wrong:** Tracking script loads successfully, but no data appears.
**Why it happens:** The default admin account exists, but no "website" has been created in the Umami dashboard. The tracking script needs a valid `data-website-id` that matches a website defined in Umami.
**How to avoid:** After first login (admin/umami — must change password immediately), go to Settings → Add Website for each subdomain. Then copy the generated website IDs into the tracking scripts.
**Warning signs:** Umami dashboard shows 0 visitors despite script loading successfully.

### Pitfall 4: SPA Double Pageview Tracking
**What goes wrong:** Each SPA navigation triggers two pageview events.
**Why it happens:** Umami auto-tracks via History API, but some app code also calls `umami.track()` in `useEffect` on route change.
**How to avoid:** Do NOT call `umami.track()` manually for page views in the React app. Umami's script already watches `pushState`/`replaceState` and `popstate`. Only use `umami.track()` for custom events (button clicks, form submissions, etc.).
**Warning signs:** Analytics dashboard shows roughly 2x expected page views for single-page-app sites.

### Pitfall 5: Tracker Script Loaded Before Caddy Certificates Are Ready
**What goes wrong:** On first deploy, the tracking script URL returns a TLS error.
**Why it happens:** Caddy needs to provision Let's Encrypt certificates for `analytics.esluce.com` (step 1), but the frontends are deployed and serving the tracking script tag (step 2). If step 1 and 2 happen simultaneously, or if the DNS record isn't yet propagated, visitors get TLS warnings.
**How to avoid:** Deploy sequence: DNS record → Caddy starts (auto-certs) → verify HTTPS works → THEN deploy frontend tracking code.
**Warning signs:** Browser console shows `ERR_CERT_COMMON_NAME_INVALID` or mixed-content warnings.

## Code Examples

### Environment File for Umami
```bash
# /opt/umami/.env
DATABASE_URL=postgresql://umami:YOUR_RDS_PASSWORD@analytics-db.xxxxxxx.ap-southeast-1.rds.amazonaws.com:5432/umami?sslmode=require
APP_SECRET=generated-via-openssl-rand-hex-32
DISABLE_TELEMETRY=1
DISABLE_UPDATES=1
TRACKER_SCRIPT_NAME=script.js
```

Generate APP_SECRET:
```bash
openssl rand -hex 32
```

### Umami Docker Compose with External RDS (No bundled PostgreSQL)
```yaml
# /opt/umami/docker-compose.yml
services:
  umami:
    image: ghcr.io/umami-software/umami:v3.1.0
    container_name: umami
    restart: unless-stopped
    expose:
      - "3000"
    environment:
      DATABASE_URL: "${DATABASE_URL}"
      APP_SECRET: "${APP_SECRET}"
      DISABLE_TELEMETRY: "${DISABLE_TELEMETRY:-1}"
      DISABLE_UPDATES: "${DISABLE_UPDATES:-1}"
      TRACKER_SCRIPT_NAME: "${TRACKER_SCRIPT_NAME:-script.js}"
    healthcheck:
      test: ["CMD-SHELL", "wget --spider -q http://localhost:3000/api/heartbeat || exit 1"]
      interval: 30s
      timeout: 5s
      retries: 3
    networks:
      - umami-net

  caddy:
    image: caddy:2
    container_name: umami-caddy
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
      - "443:443/udp"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - umami
    networks:
      - umami-net

networks:
  umami-net:
    driver: bridge

volumes:
  caddy_data:
  caddy_config:
```

### Caddyfile for analytics.esluce.com (Self-Contained Stack)
```caddy
# /opt/umami/Caddyfile
analytics.esluce.com {
    encode zstd gzip

    reverse_proxy umami:3000 {
        health_uri /api/heartbeat
        health_interval 30s
        health_timeout 2s
    }

    header {
        X-Frame-Options "DENY"
        X-Content-Type-Options "nosniff"
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        -Server
    }

    log {
        output file /var/log/caddy/analytics-esluce-com.log
        format json
    }
}
```

### Tracking Script Injection — Landing Page
```html
<!-- landing-page-escluse/index.html -->
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="icon" type="image/svg+xml" href="/assets/logo.svg" />
    <title>Escluse - Distributed Infrastructure Platform</title>
    <script
        defer
        src="https://analytics.esluce.com/script.js"
        data-website-id="LANDING_PAGE_WEBSITE_ID"
        data-domains="esluce.com"
    ></script>
</head>
```

### Tracking Script Injection — React App
```html
<!-- app/index.html -->
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="icon" type="image/svg+xml" href="/logo.svg" />
    <title>Escluse — Server Control Platform</title>
    <link
        href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Fira+Code:wght@400;500&display=swap"
        rel="stylesheet">
    <script
        defer
        src="https://analytics.esluce.com/script.js"
        data-website-id="APP_WEBSITE_ID"
        data-domains="app.esluce.com"
    ></script>
</head>
```

### RDS Database Setup (Pre-requisite)
```sql
-- Run on RDS (via psql or pgAdmin)
CREATE DATABASE umami;
CREATE USER umami WITH PASSWORD 'your-strong-password';
GRANT ALL PRIVILEGES ON DATABASE umami TO umami;

-- For Prisma migration (needs schema-level grants):
\c umami
GRANT ALL ON SCHEMA public TO umami;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO umami;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO umami;
```

### Generate Tracking Script with Different Name (Ad Blocker Evasion)
```yaml
# In docker-compose.yml environment:
environment:
  TRACKER_SCRIPT_NAME: "analytics.js"
  COLLECT_API_ENDPOINT: "/api/collect"
```

Then the tracking script would change to:
```html
<script defer src="https://analytics.esluce.com/analytics.js" data-website-id="xxx"></script>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Umami v2 (MySQL + PostgreSQL) | Umami v3 (PostgreSQL only) | 2025-12 (v3.0.0) | MySQL support dropped entirely; all new deployments must use PostgreSQL |
| Manual server-side tracking | SPA auto-detection via History API | v2.x | No extra JavaScript needed for React/Vue/Angular apps — script tag alone works |
| Default tracker script name `umami.js` | Configurable via `TRACKER_SCRIPT_NAME` | v1.26.0 | Can rename to evade ad blockers (e.g., `analytics.js`, `stats.js`) |
| GeoIP via MaxMind download | Optional; can skip via `SKIP_BUILD_GEO` | v3.1.0 | Reduces image size and startup time if not needed |

**Deprecated/outdated:**
- **MySQL database:** Not supported in Umami v3+. PostgreSQL is the only option.
- **Default port 3000 directly exposed:** Strongly discouraged; always use a reverse proxy (Caddy/Nginx).
- **`umami.js` as tracker script name:** On every ad blocker filter list. Rename via `TRACKER_SCRIPT_NAME`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Umami auto-creates tables via Prisma migrations on first boot against an empty database. | Code Examples | LOW — official docs confirm this; if wrong, need to pre-run `npx prisma migrate deploy` |
| A2 | Umami default admin credentials are `admin` / `umami`. | Common Pitfalls | LOW — documented by every deployment guide; first action is always to change password |
| A3 | Existing EC2 instance has Docker and Docker Compose installed. | Environment Availability | MEDIUM — depends on Phase 65 outcome (installer script for Docker); if Docker not present, plan must include install step |
| A4 | DNS for `analytics.esluce.com` can be created via Cloudflare API or manually. | Architecture | MEDIUM — Cloudflare integration exists in project (Phase 51), but DNS step may be manual (D-06) |

## Open Questions

1. **Where does this Umami stack get deployed?**
   - What we know: D-01 says "EC2" — assumed same EC2 instance hosting the main esluce app stack.
   - What's unclear: Which EC2 instance (same as main app, or a separate one)? The existing project has services running on a single EC2 via Docker Compose. Umami with its own Caddy + RDS could share the same EC2.
   - Recommendation: Assume **same EC2 instance** unless indicated otherwise. If resources are tight (2GB RAM recommended for Umami + 200MB idle), consider a t3.medium or larger.

2. **Does the existing EC2 already have Docker installed?**
   - What we know: Phase 65 creates an auto-install script for Docker. The phase is not yet complete.
   - What's unclear: Whether Docker is actually installed and available on the target EC2.
   - Recommendation: Add Docker availability check to the plan's prerequisites section.

3. **Who manages the RDS instance?**
   - What we know: D-02 says "dedicated new RDS PostgreSQL instance." D-06 says manual setup.
   - What's unclear: The plan needs to include RDS creation steps (console or CLI). Is the RDS instance already created, or does this phase create it?
   - Recommendation: The plan should include RDS provisioning steps: instance class (db.t3.micro or db.t3.small), storage (20GB gp3), security group rules, and database/user creation.

## Environment Availability

> Note: This phase requires access to an AWS EC2 instance and RDS service. Dependency checks below assume the target machine is the same EC2 running the main esluce stack. If the target environment is not yet ready (Phase 65 not complete), Docker availability may be pending.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Docker | Umami container | ? (depends on Phase 65) | — | Manual Docker install (Phase 65 script) |
| Docker Compose | Docker orchestration | ? (depends on Phase 65) | — | Manual Docker Compose install |
| AWS CLI | RDS management (optional) | ? | — | Use AWS Management Console (D-06 manual approach) |
| `openssl` | Generate APP_SECRET | ✓ | system | `python3 -c "import secrets; print(secrets.token_hex(32))"` |
| `psql` client | Test RDS connection | ? | — | RDS connection can be tested via Docker postgres image |

**Missing dependencies with no fallback:**
- Docker and Docker Compose must be available on the EC2 instance. Phase 65 addresses this; verify before planning.

**Missing dependencies with fallback:**
- AWS CLI: Not strictly needed — RDS can be provisioned via AWS Console (D-06: manual setup with docs).
- `psql` client: Use `docker run --rm postgres:16-alpine psql` as a client container.

## Validation Architecture

> The Plan Checker is configured with `workflow.nyquist_validation: false` in `.planning/config.json`. Validation Architecture section is omitted per configuration.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | Yes | Umami's built-in auth with `APP_SECRET` for JWT signing; default admin password must be changed on first login |
| V3 Session Management | Yes | Umami manages sessions via signed JWT cookies; `APP_SECRET` must be a strong random value |
| V4 Access Control | Partial | Umami has admin vs. view-only roles; dashboard access scoped to `analytics.esluce.com` domain |
| V5 Input Validation | No | Umami handles its own input validation on the tracking endpoint |
| V6 Cryptography | No | No custom crypto; TLS handled by Caddy |

### Known Threat Patterns for {Umami + Caddy + RDS}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Database exposed to internet | Information Disclosure | Security group: allow port 5432 only from EC2 security group, not 0.0.0.0/0 |
| Man-in-the-middle on DB connection | Tampering | `sslmode=require` in DATABASE_URL; RDS enforces TLS by default |
| Tracking script served over HTTP | Spoofing | Caddy auto-redirects HTTP→HTTPS; Strict-Transport-Security header set |
| Brute force on admin login | Elevation of Privilege | Use strong APP_SECRET; consider adding fail2ban or Caddy rate limiting |
| Ad blocker bypass not configured | Denial of Service | Low risk; use `TRACKER_SCRIPT_NAME` to rename tracking script |

## Sources

### Primary (HIGH confidence)
- [Official Umami Docs — Environment Variables](https://umami.is/docs/environment-variables) — Full env reference for v3
- [Official Umami Docs — Tracker Configuration](https://umami.is/docs/tracker-configuration) — All `data-*` attributes
- [Official Umami Docs — Track Single-Page Apps](https://docs.umami.is/docs/guides/track-single-page-apps) — SPA tracking confirmation
- [Umami GitHub — docker-compose.yml](https://github.com/umami-software/umami/blob/master/docker-compose.yml) — Reference compose file with bundled PostgreSQL
- [Umami GitHub — Releases (v3.1.0)](https://github.com/umami-software/umami/releases/tag/v3.1.0) — Latest stable version confirmed
- [Umami Docker Hub](https://hub.docker.com/r/umamisoftware/umami) — Official image tags
- [Project Caddyfile.prod](./gateway/Caddyfile.prod) — Existing Caddy patterns to follow
- [Project docker-compose.yml](./docker-compose.yml) — Existing composition patterns
- [Project STACK.md](./.planning/codebase/STACK.md) — PostgreSQL 16, Caddy 2 confirmed

### Secondary (MEDIUM confidence)
- [Self-hosting Umami guide](https://analytics-alternatives.com/umami-self-hosting-guide-docker-setup/) — Detailed deployment walkthrough (2026-04-05)
- [Umami Self-Hosted Setup Guide](https://darge.com/umami-analytics-self-hosted-complete-vps-setup-guide/) — Step-by-step RDS-like patterns (2026-03-13)
- [Umami with Caddy reverse proxy](https://deepakness.com/blog/self-hosting-umami-analytics/) — Working Caddy setup example (2025-10-20)
- [DeepWiki Umami configuration](https://deepwiki.com/umami-software/umami/2.3-environment-configuration) — Comprehensive env reference (2026-04-25)
- [@danielgtmn/umami-react npm](https://registry.npmjs.org/%40danielgtmn%2Fumami-react) — React Umami wrapper package info (2025-03-27)
- [@giof/react-umami npm](https://registry.npmjs.org/%40giof%2Freact-umami) — Alternative React Umami wrapper (2025-06-26)

### Tertiary (LOW confidence)
- None — all critical claims verified via official sources.

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — Umami image tag, PostgreSQL requirement, Caddy config all verified via official docs and project files
- Architecture: **HIGH** — Docker Compose pattern, Caddy proxy setup, SPA tracking all confirmed by official docs and ecosystem
- Pitfalls: **HIGH** — RDS connection issues, ad blocker bypass, SPA double-tracking all documented in official and community guides
- Environment availability: **MEDIUM** — Docker availability depends on Phase 65 completion; cannot verify without access to target EC2

**Research date:** 2026-05-31
**Valid until:** 2026-07-01 (Umami releases minor updates monthly; major version validity longer)
