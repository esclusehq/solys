# Technology Stack

**Project:** Escluse Community Bot
**Researched:** 2026-06-18
**Overall confidence:** HIGH — all versions verified against official registries (npm, crates.io) and documentation as of June 2026.

---

## Recommended Stack

### Core Bot Runtime (TypeScript/Bun)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Bun** | `1.3.x` (latest stable `1.3.14`) | JavaScript/TypeScript runtime | 4x faster startup than Node.js, native TypeScript execution (no ts-node), built-in test runner, 99%+ npm compat. Anthropic-backed (acquired Dec 2025). Built-in `bun-sql` PostgreSQL driver available. |
| **discord.js** | `14.x` (latest stable `14.26.4`) | Discord API client library | Most mature, 100% Discord API coverage, first-class slash commands, embed/modals/buttons builders, TypeScript-native types, active maintenance. v15 in pre-release — **do not use v15 for production yet**. |
| **TypeScript** | `5.x` (latest `5.7`) | Type safety | Bun runs `.ts` directly with zero config. Strict mode recommended. |

#### Why not alternatives
| Alternative | Why Not |
|-------------|---------|
| **Node.js** | 2-3x slower startup, requires ts-node/tsx for TypeScript, no built-in test runner. Your stack decision is already Bun — validated as production-ready for Discord bots in 2026. |
| **discord.js v15 dev** | Pre-release state, breaking API changes mid-stream. Wait for stable. v14 is the current production line. |
| **Eris** | Less maintained, smaller ecosystem, fewer community resources. |
| **Discordeno** | Smaller community, fewer third-party integrations. No reason to switch from discord.js dominance. |

### API Service (Rust/Axum)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Rust** | `1.80+` (MSRV) | Systems-level backend language | Aligns with existing Escluse monorepo Rust infrastructure. Memory safety without GC, zero-cost abstractions. |
| **Axum** | `0.8.x` (latest `0.8.9`) | HTTP web framework | Modular, Tower-based middleware, type-safe extractors, WebSocket support built-in, maintained by Tokio team. No macros needed for routing. |
| **Tokio** | `1.x` (latest `1.43`) | Async runtime | Industry-standard Rust async runtime. Axum runs on Tokio. Enable `full` features. |
| **tower-http** | `0.6.x` (latest `0.6.8`) | HTTP middleware (CORS, tracing, compression) | Axum-native middleware via Tower Service trait. Drop-in CORS, request logging, gzip. |
| **serde** | `1.x` | Serialization/deserialization | Standard for JSON handling with `serde_json`. Required by Axum extractors. |
| **sqlx** | `0.8.x` | Async PostgreSQL driver | Compile-time checked SQL queries, migrations, connection pooling. Best-in-class Rust PostgreSQL client. |
| **redis-rs** | `0.27.x` | Redis client for Rust | Async Redis with `tokio-comp` feature. Multiplexed connections. |

#### Why not alternatives
| Alternative | Why Not |
|-------------|---------|
| **Actix-web** | Excellent performance but Axum has better ergonomics, Tower middleware ecosystem, and Tokio team backing. Actix's actor model adds unnecessary complexity for an API service. |
| **Rocket** | Macro-heavy, less flexible, smaller ecosystem. |
| **SeaORM** | Overkill for the API service's role. Prefer `sqlx` with query-as-you-go pattern — the service mostly proxies/serves bot data, not complex ORM operations. |
| **prisma-client-rust** | Prisma is JS-first, Rust binding is immature. |

### Database

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **PostgreSQL** | `16+` (latest `17` available) | Primary data store | Already chosen. Reliable, battle-tested, JSONB for flexible config storage. Use `16-alpine` Docker image. |
| **Drizzle ORM** | `0.45.x` (stable); `1.0.0-beta` available | TypeScript ORM for bot → DB | **The pick over Prisma.** ~7.4kb gzipped, zero dependencies, SQL-like TypeScript queries, full type inference from schema. Works flawlessly with Bun (`drizzle-orm/bun-sql` adapter). Drizzle Kit handles migrations. No codegen step — schema IS the source of truth. |
| **Bun SQL** (native) | Built into Bun | PostgreSQL driver (via `drizzle-orm/bun-sql`) | Bun's native `bun:sql` module is the fastest way to connect Bun → PG. Drizzle wraps it. No `pg` package needed. |

#### Why Drizzle ORM over Prisma
| Criterion | Drizzle ORM | Prisma |
|-----------|-------------|--------|
| Bundle size | ~7.4kb min+gzip | ~25MB+ (engine binary) |
| TypeScript inference | From schema file directly | Generated client (codegen step) |
| Bun compatibility | Native `bun-sql` adapter | Works but heavy, engine binary compat issues |
| SQL transparency | You write TypeScript that reads like SQL | Prisma's DSL obscures SQL |
| Migrations | `drizzle-kit` (fast) | `prisma migrate` (slow, often needs `db push` debug) |
| Dev speed | No codegen, edit schema → use immediately | Must run `prisma generate` after every schema change |

### Cache / Real-time

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Redis** | `7.x` (latest `7.4`) | Caching, rate limiting, pub/sub, session store | Already chosen. Redis 7.x supports sharded Pub/Sub (Redis 7.0+) — critical for cross-process communication if bot shards later. |
| **ioredis** | `5.10.x` (latest `5.10.1`) | Node.js/Bun Redis client | Full-featured: Cluster, Sentinel, Streams, Pub/Sub, Lua scripting, auto-pipelining. 100% TypeScript. Bun-compatible. |

#### Why ioredis over node-redis
ioredis is the mature, widely-deployed choice (8k+ dependents, used by Alibaba). `node-redis` is newer and officially recommended by Redis Inc., but ioredis has superior auto-pipelining, better Cluster support, and a larger community. For a Discord bot that may eventually need sharded Pub/Sub across processes, ioredis's Cluster support is important.

### Infrastructure / DevOps

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Docker** | `27.x` | Containerization | Matches existing Escluse infra patterns. Multi-stage builds for both TypeScript bot and Rust API. |
| **Docker Compose** | `2.x` | Local dev orchestration | Spin up bot + API + PostgreSQL + Redis with single `docker compose up` |
| **Dockerfile** (Bot) | Bun-based | Bun's official Docker image: `oven/bun:1.3-alpine` | ~150MB image, fast builds |
| **Dockerfile** (API) | rust:1.80-slim-bookworm | Multi-stage Rust build | ~80MB final image |

### Optional / Nice-to-Have

| Library | Version | Purpose | When to Add |
|---------|---------|---------|-------------|
| **zlib-sync** | (npm) | WebSocket compression for discord.js | Always — reduces gateway bandwidth ~50% |
| **bufferutil** | (npm) | Faster WebSocket frame processing (C++ addon) | When bot >200 guilds; pure-JS fallback exists |
| **chalk** | `5.x` (ESM) | Colored console output | Dev phase only |
| **pino** | `9.x` | Structured logging (bot side) | When you need log shipping; Bun's `console.log` is fine for now |
| **tracing** | `0.1.x` | Structured logging (Rust side) | Use `tracing-subscriber` with JSON output for production |
| **utoipa** | `5.x` | OpenAPI docs generation for Axum | When API endpoints grow >10; skip for v1 |
| **redis-rs** (Rust) | `0.27.x` | Rust-side Redis access for API service | When API service needs to read/write cache directly |

---

## Version Verification

All versions verified as of June 2026:

| Component | Latest Verified | Source | Confidence |
|-----------|----------------|--------|------------|
| Bun | 1.3.14 | `bun.sh`, `endoflife.date/bun`, npm | HIGH |
| discord.js | 14.26.4 | `npmjs.com/package/discord.js` | HIGH |
| TypeScript | 5.7+ | `npmjs.com/package/typescript` | HIGH |
| Axum | 0.8.9 | `crates.io/crates/axum`, GitHub releases | HIGH |
| Tokio | 1.43 | `crates.io/crates/tokio` | HIGH |
| tower-http | 0.6.8 | `crates.io/crates/tower-http` | HIGH |
| sqlx | 0.8.x | `crates.io/crates/sqlx` | MEDIUM (exact patch version not checked) |
| Drizzle ORM | 0.45.x (stable) | `orm.drizzle.team`, npm | HIGH |
| PostgreSQL | 17 | PostgreSQL official | HIGH |
| Redis | 7.4 | `redis.io` | HIGH |
| ioredis | 5.10.1 | `npmjs.com/package/ioredis` | HIGH |

---

## Installation

### Bot (TypeScript/Bun)

```bash
# Create project with Bun
bun init -y

# Core dependencies
bun add discord.js@^14.26.4
bun add drizzle-orm
bun add ioredis@^5.10.1

# Dev dependencies
bun add -D drizzle-kit
bun add -D @types/node  # if needed for Node.js compatibility types

# Optional performance
bun add zlib-sync
bun add bufferutil
```

### API Service (Rust/Axum)

```toml
# Cargo.toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
redis = { version = "0.27", features = ["tokio-comp"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
reqwest = { version = "0.12", features = ["json"] }  # For outgoing webhooks/API calls
```

### Docker

```dockerfile
# Bot Dockerfile
FROM oven/bun:1.3-alpine AS bot
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile
COPY . .
CMD ["bun", "run", "src/index.ts"]

# API Dockerfile
FROM rust:1.80-slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/escluse-api /app/escluse-api
CMD ["/app/escluse-api"]
```

---

## Database Schema Patterns (Discord Bot)

### Core Tables (PostgreSQL via Drizzle ORM)

```typescript
// src/db/schema.ts — recommended structure for a Discord community bot

import { pgTable, serial, bigint, varchar, text, 
         timestamp, boolean, jsonb, integer } from 'drizzle-orm/pg-core';

// Guild (server) configuration — one row per Discord server
export const guilds = pgTable('guilds', {
  id: bigint('id', { mode: 'bigint' }).primaryKey(),       // Discord guild ID
  name: varchar('name', { length: 100 }).notNull(),
  prefix: varchar('prefix', { length: 10 }).default('/'),
  settings: jsonb('settings').$type<{
    welcomeChannelId?: string;
    ticketCategoryId?: string;
    logChannelId?: string;
    memberRoleId?: string;
  }>(),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
});

// User data
export const users = pgTable('users', {
  id: bigint('id', { mode: 'bigint' }).primaryKey(),       // Discord user ID
  username: varchar('username', { length: 32 }).notNull(),
  globalName: varchar('global_name', { length: 100 }),
  selectedInterests: varchar('selected_interests').array(), // 'self-hosting', 'minecraft', etc.
  referralCode: varchar('referral_code', { length: 20 }).unique(),
  referredBy: bigint('referred_by', { mode: 'bigint' }),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
});

// Interests/Roles mapping
export const interests = pgTable('interests', {
  id: serial('id').primaryKey(),
  guildId: bigint('guild_id', { mode: 'bigint' }).notNull().references(() => guilds.id),
  interestKey: varchar('interest_key', { length: 50 }).notNull(), // 'self-hosting', 'minecraft'
  displayName: varchar('display_name', { length: 100 }).notNull(),
  roleId: bigint('role_id', { mode: 'bigint' }),
  createdAt: timestamp('created_at').defaultNow().notNull(),
});

// Support tickets
export const tickets = pgTable('tickets', {
  id: serial('id').primaryKey(),
  guildId: bigint('guild_id', { mode: 'bigint' }).notNull(),
  userId: bigint('user_id', { mode: 'bigint' }).notNull(),
  channelId: bigint('channel_id', { mode: 'bigint' }),
  category: varchar('category', { length: 50 }).notNull(), // 'support', 'bug-report'
  status: varchar('status', { length: 20 }).default('open').notNull(), // 'open', 'closed'
  subject: varchar('subject', { length: 200 }),
  metadata: jsonb('metadata').$type<{
    version?: string;
    os?: string;
    steps?: string;
    logs?: string;
  }>(),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  closedAt: timestamp('closed_at'),
});

// Server showcases
export const showcases = pgTable('showcases', {
  id: serial('id').primaryKey(),
  guildId: bigint('guild_id', { mode: 'bigint' }).notNull(),
  userId: bigint('user_id', { mode: 'bigint' }).notNull(),
  name: varchar('name', { length: 100 }).notNull(),
  description: text('description'),
  game: varchar('game', { length: 100 }),
  screenshots: varchar('screenshots').array(),
  approved: boolean('approved').default(false),
  createdAt: timestamp('created_at').defaultNow().notNull(),
});

// Release announcements / changelog cache
export const releases = pgTable('releases', {
  id: serial('id').primaryKey(),
  version: varchar('version', { length: 20 }).notNull().unique(),
  title: varchar('title', { length: 200 }).notNull(),
  body: text('body'),
  publishedAt: timestamp('published_at').notNull(),
  createdAt: timestamp('created_at').defaultNow().notNull(),
});

// Feature votes/polls
export const polls = pgTable('polls', {
  id: serial('id').primaryKey(),
  guildId: bigint('guild_id', { mode: 'bigint' }).notNull(),
  title: varchar('title', { length: 200 }).notNull(),
  options: jsonb('options').$type<string[]>(),
  messageId: bigint('message_id', { mode: 'bigint' }),
  expiresAt: timestamp('expires_at'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
});

// poll votes
export const pollVotes = pgTable('poll_votes', {
  id: serial('id').primaryKey(),
  pollId: integer('poll_id').notNull().references(() => polls.id),
  userId: bigint('user_id', { mode: 'bigint' }).notNull(),
  optionIndex: integer('option_index').notNull(),
  createdAt: timestamp('created_at').defaultNow().notNull(),
});
```

### Indexing Strategy
- **Guild ID index** on all FK columns
- **Composite index** on `tickets(guild_id, status)` — fast open ticket queries
- **Unique constraint** on `users.referral_code`
- **GIN index** on `users.selected_interests` (array column)
- **Partial index** on `showcases(approved)` where `approved = true`

---

## Redis Usage Patterns

### Caching Layer
```
Key Pattern: escluse:cache:{type}:{id}
Examples:
  escluse:cache:guild:123456789  → guild settings (TTL: 300s)
  escluse:cache:user:987654321   → user data (TTL: 300s)
  escluse:cache:docs:search:{q}  → docs search results (TTL: 600s)
  escluse:rate:cmd:ping:user:987654321 → rate limit counter
```

### Rate Limiting
```
Key Pattern: escluse:rate:{resource}:{scope}:{id}
TTL matches rate limit window (1s-1h)
Use INCR + EXPIRE pattern, or Lua script for atomicity
```

### Pub/Sub (for future sharding)
```
Channel: escluse:events:global — cross-process events
Channel: escluse:events:guild:{guildId} — guild-scoped events

Message format (JSON):
{
  "type": "GUILD_CONFIG_UPDATE" | "CACHE_INVALIDATE",
  "guildId": "123456789",
  "timestamp": 1234567890
}
```

### Session Store (if needed)
```
Key Pattern: escluse:sess:{sessionId}
String value (JSON-encoded session data with TTL)
```

### What NOT to use Redis for
- **Persistent storage** — use PostgreSQL for anything that must survive restarts
- **Large documents** — Redis memory is expensive; keep values < 10KB
- **Relational queries** — Redis is key-value, don't fight it

---

## Key Architectural Decisions

### Why Drizzle ORM > Prisma for this project
1. **No 25MB engine binary** — meant for Serverless/Edge but even in Docker, smaller is better
2. **No codegen step** — edit schema, use immediately. Prisma requires `prisma generate` after every change
3. **Native Bun support** — `drizzle-orm/bun-sql` uses Bun's native `bun:sql` driver directly. Prisma's engine binary has had recurring issues with Bun compatibility
4. **SQL transparency** — Drizzle queries look like SQL so you understand the performance characteristics. Prisma hides SQL behind generated clients, making N+1 and lazy loading easier to accidentally introduce
5. **Smaller bundle** for the bot process itself

### Why Axum > Actix-web for the API service
1. **Tower ecosystem** — middleware is standardized across Tower. CORS, tracing, compression, rate limiting are all `tower-http` layers
2. **Type-safe extractors** — `State`, `Path`, `Query`, `Json` extractors validated at compile time
3. **No actor model complexity** — Axum is function-based, simpler than Actix's actor system
4. **Tokio alignment** — `sqlx` and `redis-rs` both run on Tokio; using Axum keeps the async runtime consistent

### Why ioredis > node-redis
- Better Cluster/Sentinel support
- Auto-pipelining for batch operations
- Lua scripting abstraction
- Larger community (8k+ dependents)
- Both work with Bun; ioredis is more battle-tested

---

## Alternatives Considered (Full List)

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| **Runtime** | Bun 1.3.x | Node.js 22 LTS | Stack decision already made. Bun validated. |
| **Bot Library** | discord.js 14.x | Discord.js v15 (dev) | Pre-release, API instability. Don't use for production. |
| **Bot Library** | discord.js 14.x | Eris | Smaller ecosystem, fewer features. |
| **Bot Library** | discord.js 14.x | Discordeno | Smaller community, immature. |
| **TypeScript ORM** | Drizzle ORM | Prisma | Heavy (25MB engine), codegen step, Bun compat issues. |
| **TypeScript ORM** | Drizzle ORM | TypeORM | Dated API, slower, less TypeScript-native. |
| **TypeScript ORM** | Drizzle ORM | Kysely | Good query builder but no migrations, no schema-first approach. |
| **JS Redis client** | ioredis | node-redis | ioredis has better Cluster support and auto-pipelining. |
| **JS Redis client** | ioredis | redis (npm) | Bun-compatible but less feature-rich. |
| **Rust Framework** | Axum 0.8 | Actix-web | Tower ecosystem, Tokio alignment, better DX. |
| **Rust Framework** | Axum 0.8 | Rocket | Macro-heavy, less flexible, slower release cadence. |
| **Rust Framework** | Axum 0.8 | Warp | Less maintained, Axum absorbed most Warp users. |
| **Rust DB Driver** | sqlx | diesel | diesel is synchronous, not async-native. |
| **Rust DB Driver** | sqlx | SeaORM | Overkill for proxy service; sqlx gives direct SQL control. |

---

## Sources

- **Discord.js v14 (14.26.4)**: https://www.npmjs.com/package/discord.js — HIGH confidence
- **Bun 1.3.x**: https://bun.sh, https://endoflife.date/bun — HIGH confidence
- **Drizzle ORM**: https://orm.drizzle.team/docs/get-started/bun-sql-new — HIGH confidence
- **Axum 0.8.x**: https://crates.io/crates/axum — HIGH confidence
- **ioredis 5.10.x**: https://www.npmjs.com/package/ioredis — HIGH confidence
- **tower-http**: https://crates.io/crates/tower-http — HIGH confidence
- **Bun + Discord.js compat**: https://www.npmjs.com/package/discord.js (lists `bun add discord.js`) — HIGH confidence
- **Bun production readiness**: https://www.froxell.com/blog/bun-runtime-breakdown-nodejs-2026 — MEDIUM confidence (third-party analysis)
- **Axum version history**: https://github.com/tokio-rs/axum/blob/main/axum/CHANGELOG.md — HIGH confidence
- **Discord bot DB schema patterns**: Community patterns from multiple open-source Discord bot projects — MEDIUM confidence
