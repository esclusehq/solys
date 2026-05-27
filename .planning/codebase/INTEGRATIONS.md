# External Integrations

**Analysis Date:** 2026-04-08

## APIs & External Services

**Game Server Management:**
- **SSH/SFTP** - Direct connection to game server infrastructure
  - Library: `ssh2` v0.9.5 (backend), `agent-ssh` crate (agent)
  - Used for: Remote command execution, file transfers, config management

- **RCON (Remote Console)** - Game server admin protocol
  - Library: `rcon` v0.6 (backend), `agent-rcon` crate (agent)
  - Used for: In-game commands, server control, player management

**Cloud Storage:**
- **AWS S3** - Backup storage
  - SDK: `aws-sdk-s3` v1.124.0 (backend), `rusoto_s3` v0.48 (agent)
  - Used for: Server backup files, archived data
  - Environment: AWS credentials via configuration

**WebSockets:**
- **Tokio-Tungstenite** - WebSocket client for agent communication
  - Library: `tokio-tungstenite` v0.26
  - Used for: Real-time agent <-> backend communication

## Data Storage

**PostgreSQL:**
- **Version:** 16 (Alpine image)
- **Connection:** `postgresql://server:[password]@postgres:5432/backend_db`
- **Client:** `sqlx` v0.7 with `runtime-tokio-rustls`
- **Migrations:** Located in `api/migrations/`
- **Used by:** Backend API, Worker

**Redis:**
- **Version:** 7 (Alpine image)
- **Connection:** `redis://:[password]@redis:6379`
- **Client:** `redis` v0.25 with async tokio runtime
- **Features used:** Connection pooling, async operations
- **Used by:** Backend API (caching, sessions), Worker (job queue)
- **Password:** Required (set in docker-compose.yml)

**Local File Storage:**
- **Type:** Local filesystem
- **Purpose:** Server logs, temporary files
- **Note:** S3 used for persistent backups

## Authentication & Identity

**Supabase (Auth & Database):**
- **Service:** Supabase (managed auth service)
- **Client:** `@supabase/supabase-js` v2.100.0
- **URL:** `https://ucroffwfbnihmhlwhzba.supabase.co`
- **Anon Key:** `sb_publishable_zStE4AUkPWCnBeLPa5DV1Q_dHuAOQnM`
- **Used for:** User authentication in frontend
- **Environment Variables:**
  - `VITE_SUPABASE_URL`
  - `VITE_SUPABASE_ANON_KEY`

**Custom JWT Authentication:**
- **Library:** `jsonwebtoken` v9
- **Used for:** Backend API authentication (session tokens)
- **Password Hashing:** `bcrypt` v0.15, `argon2` v0.5

**Webhooks:**
- **Discord Webhooks** - Alert notifications
  - Database field: `discord_webhook_url`
  - Stored in server configuration

## Monitoring & Observability

**Logging:**
- **Framework:** `tracing` v0.1 (Rust), console (frontend)
- **Subscriber:** `tracing-subscriber` with env-filter
- **Logs destination:** Files and stdout (container logs)
- **Log files:**
  - `api/backend.log`
  - `app/frontend.log`

**Health Monitoring:**
- **Internal:** Agent health checks via `agent-health` crate
- **Database:** PostgreSQL health checks (`pg_isready`)
- **Redis:** Redis health checks (`redis-cli ping`)

## CI/CD & Deployment

**Containerization:**
- **Docker** - All services containerized
- **Docker Compose** - Local development orchestration

**Reverse Proxy:**
- **Caddy** v2 - Production reverse proxy
  - Configuration: `gateway/Caddyfile.prod`
  - Features: Automatic HTTPS, security headers, SSL termination
  - Domains: esluce.com, app.esluce.com, api.esluce.com

**Static File Server:**
- **Nginx** (Alpine) - Frontend production serving
  - Configuration: `app/nginx.conf`
  - Served from container

**Build:**
- **Frontend:** Vite build (Node.js 20)
- **Backend:** Cargo release build (Rust)

## Environment Configuration

**Required Environment Variables:**

**Backend (`api/.env`):**
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `RUST_LOG` - Logging level
- `SERVER_HOST` - Bind address
- `SERVER_PORT` - Port number
- AWS credentials (for S3 backup)

**Frontend (`app/.env`):**
- `VITE_API_URL` - Backend API URL (e.g., `https://api.esluce.com`)
- `VITE_SUPABASE_URL` - Supabase project URL
- `VITE_SUPABASE_ANON_KEY` - Supabase anon key

**Worker (`worker/.env`):**
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `WORKER_ID` - Worker identifier
- `WORKER_CONCURRENCY` - Max concurrent jobs

**Landing Page (`landing-page-escluse/.env`):**
- Environment configuration present (details in .env file)

## Webhooks & Callbacks

**Outgoing Webhooks:**
- Discord webhook notifications for server alerts
- Webhook configuration stored in database

**Incoming Webhooks:**
- Stripe webhooks (billing) - Commented out in code
- Agent registration callbacks
- Custom webhook endpoints via `webhook_handlers.rs`

---

*Integration audit: 2026-04-08*
