# Technology Stack

**Analysis Date:** 2026-04-08

## Languages

**Primary:**
- **Rust** (edition 2021) - Backend API, Worker, Agent Core, Web Agent
- **TypeScript/JavaScript** - Frontend application (React), Landing page
- **SQL** - Database migrations (PostgreSQL)

**Secondary:**
- **Shell** - Deployment scripts (bash)
- **Nginx Config** - Frontend server configuration

## Runtime

**Environment:**
- **Node.js** v20 - Frontend build and development
- **Tokio** v1 (Rust) - Async runtime for all Rust services

**Package Managers:**
- **npm** - Frontend dependencies (Node.js)
- **Cargo** - Rust dependencies (with `Cargo.lock`)

## Frameworks

**Backend (Rust):**
- **Axum** v0.7 - Web framework for REST API and WebSocket
- **Tower** v0.4 - HTTP middleware
- **Tower-HTTP** v0.5 - HTTP utilities (CORS, compression, tracing)
- **Hyper** v1 - HTTP/1 client and server
- **Tokio** v1 - Async runtime with full features

**Frontend:**
- **React** v19.2.4 - UI framework
- **React Router** v7.13.0 - Client-side routing
- **Vite** v7.3.1 - Build tool and dev server
- **Tailwind CSS** v4.2.0 - Utility-first CSS framework
- **Zustand** v5.0.12 - State management

**Landing Page:**
- **Vite** - Build tool (shared with frontend)
- **TypeScript** - Type safety

## Key Dependencies

**Backend API (`api/Cargo.toml`):**
- `sqlx` v0.7 - PostgreSQL ORM with async support
- `tokio` v1 - Async runtime
- `serde` / `serde_json` v1 - Serialization
- `ssh2` v0.9.5 - SSH/SFTP connectivity to game servers
- `rcon` v0.6 - RCON protocol for game server management
- `aws-sdk-s3` v1.124.0 - S3 backup storage
- `redis` v0.25 - Redis client for caching/queue
- `jsonwebtoken` v9 - JWT authentication
- `bcrypt` v0.15 / `argon2` v0.5 - Password hashing
- `chrono` v0.4 - Date/time handling
- `uuid` v1 - UUID generation
- `reqwest` v0.12 - HTTP client
- `regex` v1 - Regex pattern matching
- `zip` v8.2.0 / `tar` v0.4.44 / `flate2` v1.1.9 - Archive handling
- `cron` v0.15.0 - Cron scheduling
- `tokio-stream` v0.1.18 - Async streams
- `hyper` v1 with `hyper-util` - HTTP/1 support
- `tower-http` v0.5 - Middleware (CORS, compression)

**Worker (`worker/Cargo.toml`):**
- `tokio` v1 - Async runtime
- `sqlx` v0.7 - Database access
- `redis` v0.25 - Redis client
- `reqwest` v0.12 - HTTP client
- `chrono` v0.4 - Date/time
- `uuid` v1 - UUID generation
- `anyhow` / `thiserror` - Error handling
- `tracing` v0.1 - Logging

**Agent Core (`agent-core/Cargo.toml`):**
- Workspace with 12 crates for modular agent functionality
- `bollard` v0.18 - Docker API client
- `rusoto_s3` v0.48 - S3 backup storage
- `tokio-tungstenite` v0.26 - WebSocket client
- `axum` v0.8 - HTTP server for local API
- `notify` v6 - File system watcher
- SSH, RCON, backup, metrics, health monitoring crates

**Frontend (`app/package.json`):**
- `react` v19.2.4 / `react-dom` v19.2.4
- `@supabase/supabase-js` v2.100.0 - Supabase client
- `react-router-dom` v7.13.0
- `zustand` v5.0.12
- `@monaco-editor/react` v4.7.0 - Code editor
- `@tailwindcss/vite` v4.2.0
- `@vitejs/plugin-react` v5.1.4
- `tailwindcss` v4.2.0

**Landing Page (`landing-page-escluse/package.json`):**
- Basic static site with Vite
- TypeScript configuration present

## Configuration

**Environment:**
- `.env` files for local development
- Docker Compose uses environment variables
- Build args for frontend (Vite environment variables)

**Key Config Files:**
- `docker-compose.yml` - Container orchestration
- `api/Cargo.toml` - Rust dependencies and build config
- `app/package.json` - Node dependencies
- `app/vite.config.ts` - Vite configuration
- `gateway/Caddyfile.prod` - Caddy reverse proxy config

## Platform Requirements

**Development:**
- Node.js v20+
- Rust toolchain (1.70+)
- PostgreSQL (running via Docker)
- Redis (running via Docker)
- Docker & Docker Compose

**Production:**
- Docker containers for all services
- PostgreSQL 16 (Alpine)
- Redis 7 (Alpine)
- Caddy 2 for reverse proxy and SSL
- Nginx for frontend static serving
- AWS S3 for backup storage

---

*Stack analysis: 2026-04-08*
