# Walking Skeleton: Escluse Community Bot

## Purpose

This document records the foundational architectural decisions established during Phase 1 (Walking Skeleton). Subsequent phases build on these decisions without renegotiating them.

---

## Technology Choices

| Component | Choice | Version | Rationale |
|-----------|--------|---------|-----------|
| Bot runtime | Bun + TypeScript | Bun 1.3.x, TS 5.x | 4x faster startup than Node, native TS execution, built-in test runner |
| Discord library | discord.js | 14.x (v14.26.4+) | Mature ecosystem, first-class slash commands, TypeScript-native |
| API framework | Rust + Axum | Axum 0.8.x, Rust 1.80+ | Consistent with escluse monorepo, Tower middleware, type-safe extractors |
| Database ORM (bot) | Drizzle ORM | 0.45.x | ~7.4kb bundle, no codegen, native bun-sql adapter, SQL-transparent |
| Database driver (API) | SQLx | 0.8.x | Compile-time checked SQL, async-native, best Rust PostgreSQL client |
| Primary DB | PostgreSQL | 16-alpine (Docker) | Battle-tested, JSONB support, existing escluse patterns |
| Cache / IPC | Redis | 7-alpine (Docker) | Caching, rate limiting, inter-process communication |
| Redis client (bot) | ioredis | 5.10.x | Auto-pipelining, Cluster support, battle-tested |
| Containerization | Docker Compose | 2.x | Matches escluse infra patterns, local dev orchestration |
| Build orchestration | Turborepo | latest | Workspace monorepo management |

---

## Architecture Decisions

### AD-01: Two-Process Architecture
- **Bot process** (TypeScript/Bun/Discord.js) — All Discord interactions only
- **API process** (Rust/Axum) — Data persistence + webhooks
- **Communication:** Bot → API via REST, API → Bot will use Redis Streams
- **Rationale:** Independent scaling, language-native tooling, zero-downtime updates

### AD-02: Bot Never Does Direct DB Queries
- Bot always calls API service for persistence
- API owns the data layer (single auth boundary, schema encapsulation)
- API client in bot uses `fetch()` wrapper (no raw DB credentials in bot process)

### AD-03: Slash Command Registration via Standalone Script
- `scripts/deploy-commands.ts` runs separately (CI/CD or manual)
- Guild-scoped commands (instant) for development via `DEV_GUILD_ID`
- Global commands for production (takes up to 1 hour to propagate)
- **Never** register commands in the `ready` event (causes rate-limit issues)

### AD-04: Privileged Gateway Intents from Day One
- `GuildMembers` intent — required for `guildMemberAdd` event (onboarding)
- `MessageContent` intent — required for future message-based features
- Enabled in both code (`client.ts`) and Discord Developer Portal

### AD-05: Multi-Guild Schema Design
- Every guild-scoped table uses `PRIMARY KEY (guild_id, ...)` composite keys
- Prevents data collision when joining multiple Discord servers
- Retrofit would require downtime migration — done correctly from day one

### AD-06: File-Per-Command Structure
- Each command = one file in `bot/src/commands/<category>/<name>.ts`
- Auto-loaded by `command-handler.ts` (scans commands/ directory)
- Component interactions routed by `customId` prefix matching (O(1) dispatch)

### AD-07: Drizzle ORM for Bot-Side Schema
- Schema defined once in `bot/src/db/schema.ts`
- Drizzle Kit generates migrations
- Bot-only (API uses sqlx migrations independently)
- Per D-23: each service manages its own migrations

### AD-08: Append-Only Role Assignment
- Bot never removes manually-assigned roles
- Only adds new roles (default role on join, interest roles after selection)
- All role assignments logged to configured log channel

### AD-09: Restrictive-by-Default Permissions
- New commands default to admin-only
- Explicitly granted to other roles via permission configuration
- Custom DB storage (not Discord native permissions) per D-12

### AD-10: Inter-Process Communication
- Bot → API: REST HTTP (fetch)
- API → Bot: Deferred (Redis Streams in future phases)
- Direct DB access disabled from bot process

---

## Directory Layout

```
escluse-bot/
├── docker-compose.yml           # Service orchestration
├── .env.example                 # Documented env vars
├── .gitignore                   # .env, node_modules, target, dist
│
├── bot/                         # TypeScript/Bun Discord bot
│   ├── package.json
│   ├── tsconfig.json
│   ├── Dockerfile
│   ├── drizzle.config.ts        # Drizzle Kit config
│   ├── scripts/
│   │   └── deploy-commands.ts   # Standalone slash command registration
│   └── src/
│       ├── index.ts             # Entry point
│       ├── config.ts            # Env-based config
│       ├── client.ts            # Discord.js Client setup
│       ├── db/
│       │   └── schema.ts        # Drizzle ORM schema
│       ├── handlers/
│       │   ├── command-handler.ts
│       │   ├── event-handler.ts
│       │   └── component-handler.ts
│       ├── events/
│       │   ├── ready.ts
│       │   └── interactionCreate.ts
│       ├── commands/
│       │   └── info/
│       │       └── ping.ts
│       └── api/
│           └── client.ts        # HTTP client for API service
│
└── api/                         # Rust/Axum API service
    ├── Cargo.toml
    ├── Dockerfile
    ├── migrations/
    │   └── 001_create_guilds.sql
    └── src/
        ├── main.rs
        ├── config.rs
        ├── state.rs
        ├── error.rs
        ├── routes/
        │   ├── mod.rs
        │   ├── health.rs
        │   └── guilds.rs
        ├── handlers/
        │   └── guild_handlers.rs
        ├── services/
        │   └── guild_service.rs
        ├── repository/
        │   └── guild_repo.rs
        └── models/
            └── guild.rs
```

---

## Development Workflow

1. **Clone repo** — `git clone ... && cd escluse-bot`
2. **Set up Discord app** — Create application in Discord Developer Portal, copy token + client ID
3. **Enable intents** — Toggle SERVER MEMBERS INTENT + MESSAGE CONTENT INTENT in Dev Portal
4. **Copy env** — `cp .env.example .env` and fill in secrets
5. **Start services** — `docker compose up -d` (postgres, redis, api, bot)
6. **Register commands** — `docker compose exec bot bun run scripts/deploy-commands.ts`
7. **Dev iteration** — Edit bot/ or api/ code; `docker compose restart <service>`

---

## Key Env Vars

| Variable | Required | Source |
|----------|----------|--------|
| `DISCORD_TOKEN` | Yes | Discord Developer Portal → Bot → Token |
| `DISCORD_CLIENT_ID` | Yes | Discord Developer Portal → OAuth2 → Client ID |
| `DEV_GUILD_ID` | Dev only | Right-click dev server → Copy ID |
| `DB_USER` | Yes | Default: `bot_user` |
| `DB_PASSWORD` | Yes | Choose a strong password |
| `DB_NAME` | Yes | Default: `escluse_bot` |
| `DATABASE_URL` | Yes (auto-constructed) | `postgresql://${DB_USER}:${DB_PASSWORD}@postgres:5432/${DB_NAME}` |
| `REDIS_PASSWORD` | Yes | Choose a strong password |
| `REDIS_URL` | Yes (auto-constructed) | `redis://:${REDIS_PASSWORD}@redis:6379` |
| `API_URL` | Yes | `http://api:8080` (Docker) or `http://localhost:8080` (dev) |

---

## Constraints & Invariants

- **Bot process NEVER connects to PostgreSQL directly** — all DB access through API
- **All guild-scoped tables include `guild_id` as composite key** — prevents cross-server data collision
- **`hasOwnProperty` check required on all JSON config objects** — Prototype pollution prevention
- **`deferReply()` before any async operation in command handlers** — Prevents 3-second interaction timeout
- **CustomId prefix routing for components** — No DB lookups in component dispatch path
- **`z` (zod) validation on all command option parsing** — Prevents injection and type confusion
- **Env vars at process start, never read from env after init** — Immutable config pattern
