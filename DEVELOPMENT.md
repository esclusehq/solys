# Escluse — Local Development

Esluce is a game server management platform. This guide covers setting up the full development environment.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![React](https://img.shields.io/badge/react-19-blue)
![Node](https://img.shields.io/badge/node-20%2B-green)
![PostgreSQL](https://img.shields.io/badge/postgresql-16-336791)
![Redis](https://img.shields.io/badge/redis-7-red)

## Prerequisites

| Tool            | Minimum Version | Verification Command    |
|-----------------|-----------------|-------------------------|
| Docker          | 24+             | `docker --version`      |
| Docker Compose  | v2              | `docker compose version`|
| Node.js         | 20+             | `node --version`        |
| npm             | 10+             | `npm --version`         |
| Rust            | 1.70+           | `rustc --version`       |
| Cargo           | 1.70+           | `cargo --version`       |
| rustup          | latest          | `rustup --version`      |
| Supabase CLI    | latest          | `supabase --version`    |

> **Note:** See [01-prerequisites.md](dev/01-prerequisites.md) for OS-specific install commands.

## Quick Start

1. **Clone the parent repo:**
   ```bash
   git clone https://github.com/esclusehq/escluse.git && cd escluse
   ```

2. **Clone all sub-repos** (these are independent git repos, not submodules):
   See the [repo table in the Setup Guide](dev/02-setup.md#cloning-repositories) for the full list of clone commands.

3. **Install prerequisites** (toolchain, Docker, etc.):
   See the [prerequisites guide](dev/01-prerequisites.md) for OS-specific instructions.

4. **Start infrastructure services:**
   ```bash
   docker compose up postgres redis -d
   ```

5. **Start local Supabase:**
   ```bash
   supabase start
   ```
   Follow the [setup guide](dev/02-setup.md#local-supabase) for details.

6. **Configure environment files:**
   Copy-paste the dev-friendly values from the [configuration guide](dev/03-configuration.md) into `api/.env` and `app/.env`.

7. **Run database migrations:**
   ```bash
   cd api && DATABASE_URL="postgresql://server:dev_password@localhost:5432/backend_db" sqlx migrate run
   ```

8. **Start backend API** (Terminal 1):
   ```bash
   cd api && cargo run
   ```

9. **Start frontend** (Terminal 2):
   ```bash
   cd app && npm install && npm run dev
   ```

10. **Verify everything is running:**
    - API: [http://localhost:3000/health](http://localhost:3000/health)
    - Frontend: [http://localhost:5173](http://localhost:5173)

## Repository Structure

```
escluse/
├── api/              # Backend API (Rust/Axum, port 3000) — separate repo
├── app/              # Frontend (React 19 + Vite, port 5173) — separate repo
├── worker/           # Background job processor (Rust) — separate repo
├── agent/            # Agent crates
│   ├── solys/        #   Web Agent (Rust + Bollard) — separate repo
│   └── agent-core/   #   Shared Rust workspace (12 crates) — separate repo
├── docs/             # Documentation site (VitePress) — separate repo
├── dev/              # Local development setup guides — part of parent repo
├── landing-page-escluse/  # Marketing site — separate repo
├── packages/         # SDK packages — separate repo
├── gateway/          # Caddy reverse proxy config — part of parent repo
├── docker-compose.yml
└── DEVELOPMENT.md    # You are here
```

> **Warning:** `api/`, `app/`, `docs/`, `agent/solys/`, `agent/agent-core/`, `landing-page-escluse/`, `packages/`, and `migration/` are **independent git repositories** cloned into these directories — they are NOT git submodules. You must clone them separately (see [Setup Guide](dev/02-setup.md#cloning-repositories)).

## Documentation Index

- **[dev/01-prerequisites.md](dev/01-prerequisites.md)** — OS-specific tool install commands for Linux, macOS, and Windows
- **[dev/02-setup.md](dev/02-setup.md)** — Full setup: clone repos, Docker infra, Supabase, .env configuration
- **[dev/03-configuration.md](dev/03-configuration.md)** — Per-service configuration reference with copy-paste .env values
- **[dev/04-commands.md](dev/04-commands.md)** — Commands grouped by service + complete end-to-end workflow
- **[dev/05-troubleshooting.md](dev/05-troubleshooting.md)** — Common issues and solutions

## Next Steps

Once your environment is running, check out the [Architecture Overview](dev/02-setup.md#quick-architecture-overview) and [Contributing Guide](../CONTRIBUTING.md) (when available).
