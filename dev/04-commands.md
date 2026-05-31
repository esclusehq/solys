# Development Commands

Common commands for developing and testing each Esluce service. All commands assume you are in the specified service directory.

> **Important:** Each Rust service (`api/`, `worker/`, `agent/solys/`) has its own `Cargo.toml`. There is NO root Cargo workspace. Always `cd` into the service directory before running Cargo commands.

## API Backend (`api/`)

```bash
# Run the backend (starts on http://localhost:3000)
cd api
cargo run

# Run tests
cargo test

# Run linter
cargo clippy

# Run database migrations
DATABASE_URL="postgresql://server:dev_password@localhost:5432/backend_db" \
  sqlx migrate run

# Revert last migration
DATABASE_URL="postgresql://server:dev_password@localhost:5432/backend_db" \
  sqlx migrate revert

# Build in release mode (slow, optimized)
cargo build --release
```

## Worker (`worker/`)

```bash
# Run the background job processor
cd worker
cargo run

# Run tests
cargo test

# Run linter
cargo clippy

# Build
cargo build
```

## Web Agent (`agent/solys/`)

```bash
# Run the node agent (connects to backend via WebSocket)
cd agent/solys
cargo run

# Run tests
cargo test

# Run linter
cargo clippy
```

> **Note:** The web agent connects to the backend API. Ensure the API is running first.

## Agent Core (`agent/agent-core/`)

```bash
# Build all shared crates
cd agent/agent-core
cargo build

# Run tests for all crates in the workspace
cargo test

# Run linter
cargo clippy
```

> **Note:** Agent Core is a shared library workspace (12 crates). There is no runnable binary — build and test only.

## Frontend (`app/`)

```bash
# Install dependencies (first time or after pulling changes)
cd app
npm install

# Start the Vite dev server (http://localhost:5173, proxies /api/v1 -> :3000)
npm run dev

# Build for production
npm run build

# Run linter
npm run lint
```

> **Note:** Frontend tests are not yet configured.

## End-to-End Workflow

A single copy-paste block showing the complete flow from clone to running everything:

```bash
# ============================================================
# Esluce Local Development — End-to-End Setup
# ============================================================

# 1. Clone the parent repo
git clone https://github.com/esclusehq/escluse.git
cd escluse

# 2. Clone all sub-repos
git clone https://github.com/esclusehq/escluse-cloud.git api
git clone https://github.com/esclusehq/escluse-dashboard.git app
git clone https://github.com/esclusehq/escluse-docs.git docs
mkdir -p agent
git clone https://github.com/esclusehq/solys.git agent/solys
git clone https://github.com/esclusehq/agent-core.git agent/agent-core
git clone https://github.com/esclusehq/escluse-landing-page.git landing-page-escluse
git clone https://github.com/esclusehq/escluse-sdk.git packages

# 3. Verify tools installed (see dev/01-prerequisites.md if missing)
docker --version && node --version && rustc --version && supabase --version

# 4. Start infrastructure services (PostgreSQL + Redis)
docker compose up postgres redis -d

# 5. Start local Supabase
supabase start

# 6. Configure .env files (copy-paste from dev/03-configuration.md)
cp api/.env.example api/.env
cp app/.env.example app/.env
# Edit api/.env and app/.env with dev values from configuration guide

# 7. Run database migrations
cd api
DATABASE_URL="postgresql://server:dev_password@localhost:5432/backend_db" \
  sqlx migrate run
cd ..

# 8a. Start backend API (Terminal 1)
cd api && cargo run

# 8b. Start frontend (Terminal 2)
cd app && npm run dev

# 8c. Start worker (Terminal 3)
cd worker && cargo run

# 9. Verify everything is running
#    API:       curl http://localhost:3000/health
#    Frontend:  http://localhost:5173
#    Supabase:  http://localhost:54321

# 10. Run tests
cd api && cargo test
cd worker && cargo test
```

> **Note:** Steps 8a-8c require three separate terminal windows (or a terminal multiplexer like `tmux`/`screen`), as each service runs in the foreground.

For more detailed instructions on each step, refer to [Setup Guide](02-setup.md) and [Configuration](03-configuration.md).
