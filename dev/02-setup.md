# Setup Guide

Step-by-step instructions to set up your Esluce development environment.

## Cloning Repositories

Esluce uses a **meta-repo** architecture. The parent repo (`esclusehq/escluse`) contains infrastructure files (docker-compose.yml, gateway config) and placeholder directories for sub-repos.

1. Clone the parent repo:
   ```bash
   git clone https://github.com/esclusehq/escluse.git && cd escluse
   ```

2. Clone each sub-repo into its target directory:

   | Directory | GitHub URL | Clone Command |
   |-----------|------------|---------------|
   | `api/` | `https://github.com/esclusehq/escluse-cloud.git` | `git clone https://github.com/esclusehq/escluse-cloud.git api` |
   | `app/` | `https://github.com/esclusehq/escluse-dashboard.git` | `git clone https://github.com/esclusehq/escluse-dashboard.git app` |
   | `docs/` | `https://github.com/esclusehq/escluse-docs.git` | `git clone https://github.com/esclusehq/escluse-docs.git docs` |
   | `agent/solys/` | `https://github.com/esclusehq/solys.git` | `mkdir -p agent && git clone https://github.com/esclusehq/solys.git agent/solys` |
   | `agent/agent-core/` | `https://github.com/esclusehq/agent-core.git` | `git clone https://github.com/esclusehq/agent-core.git agent/agent-core` |
   | `landing-page-escluse/` | `https://github.com/esclusehq/escluse-landing-page.git` | `git clone https://github.com/esclusehq/escluse-landing-page.git landing-page-escluse` |
   | `packages/` | `https://github.com/esclusehq/escluse-sdk.git` | `git clone https://github.com/esclusehq/escluse-sdk.git packages` |

   > **Warning:** These are NOT git submodules — there is no `.gitmodules` file. You must clone each repo manually. Empty directories will cause build errors.

## Starting Infrastructure (Docker)

The project uses Docker Compose to run PostgreSQL 16 and Redis 7. Start only the infrastructure services (not the full stack):

```bash
# Start PostgreSQL and Redis in detached mode
docker compose up postgres redis -d

# Verify both services are running and healthy
docker compose ps

# PostgreSQL: port 5432, user 'server', password set in docker-compose.yml
# Redis: port 6379, password set in docker-compose.yml
```

Using dev-friendly credentials for local .env files is documented in [Configuration](03-configuration.md).

To stop infrastructure when done:

```bash
docker compose down
```

## Local Supabase

Esluce uses Supabase for authentication. For local development, use the Supabase CLI to run a local instance:

```bash
# Initialize Supabase project (creates supabase/config.toml)
supabase init

# Start local Supabase (uses Docker under the hood)
supabase start
```

After starting, Supabase CLI outputs:
- API URL: `http://localhost:54321`
- anon key: Copy this to `app/.env` as `VITE_SUPABASE_ANON_KEY`
- service_role key: Copy this to `api/.env` if needed

> **Note:** The first `supabase start` may take 1-2 minutes to pull Docker images.

To stop:

```bash
supabase stop
```

## Configuring Environment Files

Copy the example files, then replace values with the dev-friendly ones from the [Configuration guide](03-configuration.md):

```bash
cp api/.env.example api/.env
cp app/.env.example app/.env
```

Edit both files with your dev credentials (see next section for the exact values).

> **Note:** The `.env` files are in `.gitignore` — your local credentials will never be committed.

## Running Database Migrations

```bash
cd api
DATABASE_URL="postgresql://server:dev_password@localhost:5432/backend_db" \
  sqlx migrate run
cd ..
```

If migrations fail, ensure PostgreSQL is running (`docker compose ps`).

## Quick Architecture Overview

- **API (`api/`)** — Rust/Axum REST API + WebSocket server on port 3000. Handles business logic, authentication, and orchestration.
- **Worker (`worker/`)** — Background job processor consuming Redis queues for long-running operations.
- **Web Agent (`agent/solys/`)** — Node agent connecting to the backend via WebSocket for container operations on compute nodes.
- **Frontend (`app/`)** — React 19 SPA on port 5173, proxies `/api/v1` → `:3000`.
