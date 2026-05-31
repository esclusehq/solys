# Configuration

Environment variable configuration for each service. Use these values for local development.

> **Warning:** The values below are for local development only. Never commit real credentials or use production keys in `.env` files.

## Backend API (`api/.env`)

```bash
# REQUIRED: Database
DATABASE_URL=postgresql://server:dev_password@localhost:5432/backend_db

# REQUIRED: Redis
REDIS_URL=redis://:dev_password@localhost:6379

# REQUIRED: Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# REQUIRED: Security
JWT_SECRET=dev-jwt-secret-key-min-32-characters-long

# REQUIRED: Environment
ENVIRONMENT=development
RUST_LOG=debug

# OPTIONAL: Advanced Settings

# JWT Settings
JWT_ACCESS_TOKEN_EXPIRY=15
JWT_REFRESH_TOKEN_EXPIRY=7

# Security
BCRYPT_COST=12
API_KEY_LENGTH=32

# Rate Limiting
RATE_LIMIT_PER_MINUTE=60
RATE_LIMIT_PER_HOUR=1000

# Worker
WORKER_ID=worker-01
WORKER_POLL_INTERVAL_MS=1000

# Redis Pool
REDIS_POOL_SIZE=10

# OPTIONAL: Third-Party Services (leave empty if not using)

# Stripe (for payments)
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=
STRIPE_PUBLISHABLE_KEY=

# Resend (for emails)
RESEND_API_KEY=
EMAIL_FROM=

# Solys Agent
SOLYS_DEFAULT_URL=http://localhost:3001

# App URL
APP_URL=http://localhost:3000
```

## Frontend (`app/.env`)

```bash
VITE_API_URL=http://localhost:3000
VITE_SUPABASE_URL=http://localhost:54321
VITE_SUPABASE_ANON_KEY=<copy from supabase start output>
```

> **Note:** The `VITE_SUPABASE_ANON_KEY` is printed by `supabase start`. Replace `<...>` with the actual value.

## Service Configuration Profiles

#### PostgreSQL

-   Host: `localhost:5432`
-   Database: `backend_db`
-   User: `server`
-   Password: `dev_password`
-   Connection string: `postgresql://server:dev_password@localhost:5432/backend_db`
-   Running via: `docker compose up postgres -d`
-   Admin tool: `psql postgresql://server:dev_password@localhost:5432/backend_db`

#### Redis

-   Host: `localhost:6379`
-   Password: `dev_password`
-   Connection string: `redis://:dev_password@localhost:6379`
-   Running via: `docker compose up redis -d`
-   CLI: `redis-cli -a dev_password`

#### Supabase (Auth)

-   API URL: `http://localhost:54321`
-   anon key: From `supabase start` output
-   service_role key: From `supabase start` output
-   Running via: `supabase start`
-   Dashboard: `http://localhost:54323`

#### Stripe (Optional)

-   Status: **Optional** — App runs without it
-   `STRIPE_SECRET_KEY` — from Stripe Dashboard
-   `STRIPE_WEBHOOK_SECRET` — from Stripe Dashboard webhook endpoint config
-   `STRIPE_PUBLISHABLE_KEY` — from Stripe Dashboard
-   For local testing, use Stripe test mode keys

#### Resend / Email (Optional)

-   Status: **Optional** — App runs without it
-   `RESEND_API_KEY` — from Resend Dashboard
-   `EMAIL_FROM` — e.g., `noreply@esluce.com`
-   For local testing, use Resend test mode

#### Discord Webhooks (Optional)

-   Status: **Optional** — App runs without it
-   Configured per-server in the dashboard
-   For local testing, create a Discord webhook URL in a test channel

For a full reference of all environment variables, see `api/.env.example` and `app/.env.example` in their respective directories.
