# Troubleshooting

Common issues encountered during local development and their solutions.

## 1. Empty `api/` or `app/` directories

- **Symptom:** Build errors like "file not found" when running commands in `api/` or `app/`
- **Cause:** The parent repo was cloned but sub-repos were not
- **Solution:** Run the clone commands from [setup guide](02-setup.md#cloning-repositories). These directories are independent git repos, not submodules.

## 2. `cargo run -p api` fails with "can't find crate"

- **Symptom:** `error: package ID specification 'api' did not match any packages`
- **Cause:** No root Cargo workspace. Each service has its own Cargo.toml.
- **Solution:** `cd api` first, then run `cargo run` (without `-p api` flag). Commands must be run from within each service directory.

## 3. Port already allocated (5432 or 6379)

- **Symptom:** `docker compose up postgres redis` fails with `port is already allocated`
- **Cause:** PostgreSQL or Redis is already running natively on the host, or another Docker container is using the port
- **Solution:**
  - Stop local services: `sudo systemctl stop postgresql` (Linux) or `brew services stop postgresql` (macOS)
  - Check what's using the port: `sudo lsof -i :5432` or `sudo lsof -i :6379`
  - Or configure different ports in `docker-compose.yml` (e.g., `"5433:5432"`)

## 4. `supabase` command not found

- **Symptom:** `supabase: command not found` after following setup
- **Cause:** Supabase CLI not installed or not in PATH
- **Solution:** Refer to the prerequisites table in [01-prerequisites.md](01-prerequisites.md) for OS-specific install commands. Quick install:
  - Linux: `npm install -g supabase`
  - macOS: `brew install supabase/tap/supabase`
  - Windows: `npm install -g supabase`

## 5. Database migration fails

- **Symptom:** `sqlx migrate run` returns connection error or permission denied
- **Cause:** PostgreSQL not running, wrong credentials, or database doesn't exist
- **Solution:**
  - Verify PostgreSQL is running: `docker compose ps`
  - Ensure environment variables use the correct connection string:
    ```bash
    DATABASE_URL="postgresql://server:dev_password@localhost:5432/backend_db" sqlx migrate run
    ```
  - The database `backend_db` is created automatically by the PostgreSQL container via the `POSTGRES_DB` environment variable in `docker-compose.yml`
