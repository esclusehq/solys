# Architecture: Escluse Discord Community Bot

**Researched:** 2026-06-18
**Stack:** TypeScript + Bun + Discord.js (bot), Rust + Axum (API), PostgreSQL, Redis
**Confidence:** HIGH — verified across multiple production Discord bot architectures

---

## 1. System Overview

The Escluse bot follows a **two-process architecture** with a shared data layer:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Docker Host                              │
│                                                                  │
│  ┌──────────────────────┐    ┌──────────────────────┐           │
│  │   Bot Process         │    │   API Service        │           │
│  │   (TypeScript/Bun)    │    │   (Rust/Axum)        │           │
│  │                       │    │                      │           │
│  │   Discord Gateway ◄───┼────┤   HTTP :8080         │           │
│  │   Discord.js v14      │    │   REST API           │           │
│  │   Command Handler     │    │   Web Dashboard      │           │
│  │   Event Handler       │    │   Admin Endpoints    │           │
│  │   Component Handler   │    │                      │           │
│  └──────────┬────────────┘    └──────────┬───────────┘           │
│             │                            │                       │
│             │        ┌───────────────────┼──────────┐           │
│             │        │                   │          │           │
│        ┌────▼────────▼──────┐    ┌──────▼───────────▼────┐      │
│        │   PostgreSQL        │    │   Redis                │      │
│        │   (Shared DB)       │    │   (Cache/Pub-Sub)      │      │
│        │                     │    │                        │      │
│        │   - guild_settings  │    │   - guild_settings     │      │
│        │   - users           │    │   - rate limits        │      │
│        │   - tickets         │    │   - cooldowns          │      │
│        │   - show cases      │    │   - pub/sub events     │      │
│        │   - referrals       │    │   - sessions           │      │
│        │   - commands        │    │   - job queues         │      │
│        └─────────────────────┘    └────────────────────────┘      │
└──────────────────────────────────────────────────────────────────┘
```

**Why two processes?** This is the dominant pattern for Discord bots with web dashboards (verified across SufBotV5, Turbo-Gravity, Kohaku, Rostra, and others). Separation allows:
- Independent scaling (API can be replicated, bot is shard-bound)
- Language-native tooling (Bun for Discord.js speed, Rust for API performance)
- Zero-downtime updates (restart bot without interrupting API, or vice versa)
- Clear responsibility boundaries (no single process doing everything)

---

## 2. Component Boundaries

### 2.1 Bot Process (TypeScript + Bun + Discord.js)

**Responsibility:** All Discord interactions — slash commands, event handling, button/modals/menus, gateway connection.

**Never does:** Direct database queries (goes through API), complex background processing, web serving.

```
escluse-bot/bot/
├── src/
│   ├── index.ts                  # Entry point: creates client, registers handlers, logs in
│   ├── config.ts                 # Env-based config, loaded at startup
│   ├── client.ts                 # Discord.js Client setup (intents, partials, presence)
│   │
│   ├── handlers/                 # Auto-loading infrastructure
│   │   ├── command-handler.ts    # Scans commands/, registers with Discord REST API
│   │   ├── event-handler.ts      # Scans events/, binds listeners to client
│   │   └── component-handler.ts  # Routes button/modal/select interactions by customId
│   │
│   ├── commands/                 # Slash command files, one per command
│   │   ├── _shared/              # Shared utilities for commands
│   │   │   ├── embeds.ts         # Embed builders
│   │   │   ├── permissions.ts    # Permission checks
│   │   │   └── cooldowns.ts      # Cooldown manager
│   │   ├── escluse/
│   │   │   ├── docs.ts           # /escluse docs
│   │   │   ├── pricing.ts        # /escluse pricing
│   │   │   ├── what-is.ts        # /escluse what-is
│   │   │   └── roadmap.ts        # /escluse roadmap
│   │   ├── ticket/
│   │   │   ├── create.ts         # /ticket create
│   │   │   └── close.ts          # /ticket close
│   │   ├── showcase/
│   │   │   └── submit.ts         # /showcase submit
│   │   ├── admin/
│   │   │   ├── channel.ts        # /admin channel (create/edit/delete)
│   │   │   ├── role.ts           # /admin role
│   │   │   └── settings.ts       # /admin settings
│   │   └── info/
│   │       ├── faq.ts            # /faq
│   │       ├── feedback.ts       # /feedback
│   │       └── ping.ts           # /ping
│   │
│   ├── events/                   # Discord event listeners
│   │   ├── ready.ts              # client.on('ready')
│   │   ├── interactionCreate.ts  # client.on('interactionCreate') — main router
│   │   ├── guildMemberAdd.ts     # client.on('guildMemberAdd') — welcome
│   │   ├── messageCreate.ts      # client.on('messageCreate') — if needed
│   │   └── voiceStateUpdate.ts   # client.on('voiceStateUpdate') — if needed
│   │
│   ├── components/               # Non-command interactions (buttons, modals, selects)
│   │   ├── buttons/
│   │   │   ├── ticket-claim.ts   # customId: "ticket:claim"
│   │   │   ├── ticket-close.ts   # customId: "ticket:close"
│   │   │   └── role-select.ts    # customId: "role:select"
│   │   ├── modals/
│   │   │   ├── bug-report.ts     # customId: "bug:report"
│   │   │   └── feedback.ts       # customId: "feedback:submit"
│   │   └── selects/
│   │       └── interest-picker.ts # customId: "interest:pick"
│   │
│   ├── api/                      # API client layer (talks to Axum backend)
│   │   ├── client.ts             # HTTP client (fetch wrapper, base URL, auth)
│   │   ├── guild-settings.ts     # GET/PUT /api/guilds/:id/settings
│   │   ├── tickets.ts            # CRUD /api/tickets
│   │   ├── users.ts              # GET /api/users/:id
│   │   ├── showcase.ts           # POST /api/showcase
│   │   └── referrals.ts          # POST /api/referrals
│   │
│   ├── services/                 # Business logic (keeps commands thin)
│   │   ├── welcome.ts            # Welcome message + interest selection
│   │   ├── role-manager.ts       # Role assignment logic
│   │   ├── ticket-system.ts      # Ticket lifecycle
│   │   ├── showcase-submission.ts # Showcase validation + posting
│   │   └── announcement.ts       # Release/changelog formatting
│   │
│   └── utils/                    # Shared utilities
│       ├── logger.ts             # Structured logging (pino or similar)
│       ├── formatters.ts         # Date, number, string formatters
│       └── constants.ts          # Emoji IDs, channel IDs, role IDs
│
├── tsconfig.json
├── package.json
└── Dockerfile
```

### 2.2 API Service (Rust + Axum)

**Responsibility:** REST API for data persistence, Discord webhook receiver, admin dashboard backend, authentication for web panel.

**Never does:** Connects to Discord gateway, sends messages to Discord channels (delegates to bot via Redis pub/sub).

```
escluse-bot/api/
├── Cargo.toml
├── src/
│   ├── main.rs                   # Entry point: load config, init DB pool, start server
│   ├── config.rs                 # Environment/config loading
│   ├── state.rs                  # AppState (PgPool, Redis connection, config)
│   ├── error.rs                  # Centralized error types -> HTTP responses
│   │
│   ├── routes/
│   │   ├── mod.rs                # Router composition (all route groups merged)
│   │   ├── health.rs             # GET /health, GET /ready
│   │   ├── guilds.rs             # /api/guilds/:id/settings
│   │   ├── tickets.rs            # /api/tickets CRUD
│   │   ├── users.rs              # /api/users
│   │   ├── showcase.rs           # /api/showcase
│   │   ├── referrals.rs          # /api/referrals
│   │   ├── feedback.rs           # /api/feedback, /api/polls
│   │   ├── admin.rs              # /api/admin/* (protected)
│   │   ├── webhook.rs            # POST /api/webhooks/discord (from Discord)
│   │   └── auth.rs               # /api/auth/discord (OAuth2 for web panel)
│   │
│   ├── handlers/                 # Thin HTTP handlers (extract params, call service)
│   │   ├── guild_handlers.rs
│   │   ├── ticket_handlers.rs
│   │   ├── user_handlers.rs
│   │   └── ...
│   │
│   ├── services/                 # Business logic layer
│   │   ├── guild_service.rs
│   │   ├── ticket_service.rs
│   │   ├── referral_service.rs
│   │   └── ...
│   │
│   ├── repository/               # Data access layer (SQLx queries)
│   │   ├── guild_repo.rs
│   │   ├── ticket_repo.rs
│   │   ├── user_repo.rs
│   │   └── ...
│   │
│   ├── models/                   # Domain types / DTOs
│   │   ├── guild.rs
│   │   ├── ticket.rs
│   │   ├── user.rs
│   │   └── ...
│   │
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs               # JWT verification / Discord OAuth session
│   │   ├── rate_limit.rs         # Tower rate limiting (tower-governor)
│   │   └── request_id.rs         # Request tracing
│   │
│   └── redis/                    # Redis client helpers
│       ├── mod.rs
│       ├── cache.rs              # Cache read/write helpers
│       └── pubsub.rs             # Pub/sub publisher (send actions to bot)
│
└── migrations/                   # SQLx migrations
    ├── 001_create_guild_settings.sql
    ├── 002_create_tickets.sql
    ├── 003_create_users.sql
    └── ...
```

### 2.3 Database (PostgreSQL) — Shared

**Responsibility:** Persistent storage of all bot-managed data. Shared between bot (via API) and API service.

**Never does:** Holds ephemeral/rate-limit data (that's Redis's job).

**Schema outline:**

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `guild_settings` | Per-server configuration | guild_id (PK), prefix, ticket_category_id, welcome_channel_id, ... |
| `users` | User profiles/roles | user_id (PK), discord_id, interests[], joined_at, referral_code |
| `tickets` | Support ticket records | id (PK), guild_id, author_id, status, type, created_at, closed_at |
| `ticket_messages` | Ticket message log | id (PK), ticket_id, author_id, content, timestamp |
| `showcases` | Server showcase submissions | id (PK), author_id, name, game, description, image_urls[], status |
| `referrals` | Referral tracking | id (PK), referrer_id, referee_id, code, claimed_at |
| `polls` | Community polls | id (PK), guild_id, question, options[], votes, created_by |
| `bug_reports` | Structured bug reports | id (PK), user_id, version, os, steps, logs, status |
| `feedback` | General feedback | id (PK), user_id, category, message, created_at |
| `achievements` | Community achievements/roles | id (PK), user_id, achievement_type, awarded_at |
| `command_usage` | Analytics (optional) | id, command_name, user_id, guild_id, timestamp |

### 2.4 Redis — Shared (Cache + Pub/Sub)

**Responsibility:** Low-latency caching, rate limiting, cooldowns, inter-process communication, job queues.

| Namespace | Purpose | TTL | Example Key |
|-----------|---------|-----|-------------|
| `guild:{id}:settings` | Guild settings cache | 5 min | `guild:12345:settings` |
| `cooldown:cmd:{uid}:{cmd}` | Command cooldowns | per-command | `cooldown:cmd:98765:docs` |
| `ratelimit:{uid}` | Rate limit counters | 1 min | `ratelimit:98765` |
| `bus:bot:action` | API→Bot command channel | N/A (pub/sub) | Published by API, consumed by bot |
| `bus:api:event` | Bot→API event channel | N/A (pub/sub) | Published by bot, consumed by API |
| `queue:tickets` | Ticket creation jobs | Until processed | Bull/BullMQ queue |

**Critical: Use Redis Streams (not Pub/Sub) for inter-process communication** — based on Discord's own post-mortem showing Pub/Sub without backpressure causes OOM at scale (verified: Discord 2020 outage analysis, Day 16: Pub/Sub Primitives article). Redis Streams with consumer groups provide:
- Message persistence (survives reconnects)
- Backpressure (slow consumers don't crash)
- Acknowledgment guarantees
- Re-delivery on failure

---

## 3. Data Flow

### 3.1 Slash Command Flow (Most Common Path)

```
User types /escluse docs query:"What is Escluse?"
       │
       ▼
┌──────────────────────────────┐
│  Discord Gateway              │
│  Sends INTERACTION_CREATE     │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│  Bot: events/interactionCreate.ts│
│  ┌─ isChatInputCommand()? ─────┐
│  │  YES: Look up command in    │
│  │  client.commands Map        │
│  │  by interaction.commandName │
│  └──────────────────────────┘ │
│                               │
│  Check cooldowns (Redis)      │
│  Check permissions (in-memory)│
│                               │
│  Execute command.execute()    │
│    ┌────────────────────────┐ │
│    │  Command may:          │ │
│    │  1. Reply directly     │ │
│    │     (simple response)  │ │
│    │  2. Call API service   │ │
│    │     (CRUD operations)  │ │
│    │  3. Send follow-up     │ │
│    │  4. Show modal         │ │
│    └────────────────────────┘ │
└──────────────────────────────┘
           │
           ▼
User sees response in Discord
```

**Key timing constraint:** Discord requires a response within 3 seconds of the interaction. For slow operations:
1. Call `interaction.deferReply()` immediately to get 15 minutes
2. Do the work
3. Call `interaction.editReply()` with the result

### 3.2 Component Interaction Flow (Buttons, Modals, Selects)

```
User clicks a button with customId "ticket:claim"
       │
       ▼
┌──────────────────────────────┐
│  interactionCreate.ts checks: │
│  ┌─ isButton()? ─────────────┐
│  │  YES: Extract customId    │
│  │  Parse "ticket:claim"     │
│  │  Route to components/     │
│  │  buttons/ticket-claim.ts  │
│  └──────────────────────────┘ │
│                               │
│  Handler:                     │
│  - Parse context from IDs     │
│  - Call API to update status  │
│  - Update the embed/message   │
│  - Reply ephemeral confirm    │
└──────────────────────────────┘
```

**Interaction routing strategy** (recommended by Discord.js guide):
- **One-time interactions:** Use `awaitMessageComponent()` on the message — simplest pattern for confirmations
- **Multi-collect interactions:** Use `InteractionCollector` on the message — for polls, multi-step wizards
- **Permanent handlers:** Route by `customId` prefix in `interactionCreate.ts` — for persistent buttons (ticket systems, role selectors)

### 3.3 Event-Driven Flow (Welcome / Onboarding)

```
New member joins server
       │
       ▼
┌──────────────────────────────┐
│  Discord Gateway              │
│  Sends GUILD_MEMBER_ADD      │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│  Bot: events/guildMemberAdd.ts│
│  - Fetch guild settings from │
│    Redis cache (or API)      │
│  - Send welcome DM           │
│  - Post in welcome channel   │
│  - Show interest selection   │
│    (buttons/select menu)     │
│  - Assign default role       │
└──────────────────────────────┘
```

### 3.4 API→Bot Communication Flow (Admin actions via Dashboard)

```
Admin changes guild settings via web dashboard
       │
       ▼
┌──────────────────────────────┐
│  API Service (Axum)          │
│  - PUT /api/guilds/:id/settings│
│  - Update PostgreSQL          │
│  - Invalidate Redis cache     │
│  - Publish to Redis Stream    │
│    "bus:bot:action"           │
│    { type: "settings_update", │
│      guildId, settings }      │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│  Bot Process                  │
│  - Subscribed to Redis Stream │
│    "bus:bot:action"           │
│  - Receives settings_update   │
│  - Updates in-memory cache    │
│  - Applies changes in guild   │
│    (rename channels, etc.)    │
└──────────────────────────────┘
```

### 3.5 Bot→API Communication Flow (Analytics / Logging)

```
User executes a command
       │
       ▼
┌──────────────────────────────┐
│  Bot Process                  │
│  - Execute command            │
│  - Publish to Redis Stream    │
│    "bus:api:event"            │
│    { type: "command_executed", │
│      command: "docs",         │
│      userId, guildId, ts }    │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│  API Service                  │
│  - Consumes from stream       │
│  - Writes to command_usage    │
│    table (async, no blocking) │
│  - Updates analytics          │
└──────────────────────────────┘
```

---

## 4. Discord.js Patterns

### 4.1 Slash Command Template (Per-command file pattern)

```typescript
// bot/src/commands/escluse/docs.ts
import {
  SlashCommandBuilder,
  ChatInputCommandInteraction,
  EmbedBuilder,
} from 'discord.js';
import { Command } from '../../handlers/command-handler';
import { apiClient } from '../../api/client';

export default {
  data: new SlashCommandBuilder()
    .setName('escluse')
    .setDescription('Escluse platform information')
    .addSubcommand((sub) =>
      sub
        .setName('docs')
        .setDescription('Search documentation')
        .addStringOption((opt) =>
          opt
            .setName('query')
            .setDescription('Search query')
            .setRequired(true)
            .setAutocomplete(true) // Enable autocomplete
        )
    ),

  // Cooldown in seconds (0 = disabled)
  cooldown: 3,

  // Required permissions (optional)
  requiredPermissions: [],

  // Autocomplete handler (optional)
  async autocomplete(interaction) {
    const query = interaction.options.getFocused();
    const results = await apiClient.searchDocs(query);
    await interaction.respond(
      results.map((r) => ({ name: r.title, value: r.slug }))
    );
  },

  async execute(interaction: ChatInputCommandInteraction) {
    const subcommand = interaction.options.getSubcommand();

    if (subcommand === 'docs') {
      const query = interaction.options.getString('query', true);

      // Defer for slow operations
      await interaction.deferReply({ ephemeral: true });

      const result = await apiClient.searchDocs(query);
      const embed = new EmbedBuilder()
        .setTitle(`📖 ${result.title}`)
        .setDescription(result.content)
        .setColor(0x5865f2);

      await interaction.editReply({ embeds: [embed] });
    }
  },
} satisfies Command;
```

### 4.2 Event Handler Pattern

```typescript
// bot/src/events/ready.ts
import { Events, Client } from 'discord.js';

export default {
  name: Events.ClientReady,
  once: true, // Only fire once

  async execute(client: Client) {
    client.logger.info(`Logged in as ${client.user?.tag}`);
    client.logger.info(
      `Serving ${client.guilds.cache.size} guilds`
    );

    // Set bot presence
    client.user?.setPresence({
      activities: [
        {
          name: '/escluse help',
          type: 3, // 3 = WATCHING (ActivityType.Watching)
        },
      ],
      status: 'online',
    });
  },
};
```

### 4.3 Interaction Router Pattern

```typescript
// bot/src/events/interactionCreate.ts
import { Events, BaseInteraction } from 'discord.js';
import { handleCommand } from '../handlers/command-handler';
import { handleComponent } from '../handlers/component-handler';

export default {
  name: Events.InteractionCreate,

  async execute(interaction: BaseInteraction) {
    try {
      // Slash commands & context menus
      if (interaction.isChatInputCommand() || interaction.isContextMenuCommand()) {
        await handleCommand(interaction);
        return;
      }

      // Message components (buttons, selects)
      if (interaction.isMessageComponent()) {
        await handleComponent(interaction);
        return;
      }

      // Modals
      if (interaction.isModalSubmit()) {
        await handleComponent(interaction); // Reuse component handler
        return;
      }

      // Autocomplete
      if (interaction.isAutocomplete()) {
        const command = interaction.client.commands.get(interaction.commandName);
        if (command?.autocomplete) {
          await command.autocomplete(interaction);
        }
        return;
      }
    } catch (error) {
      logger.error('Unhandled interaction error', { error, interactionId: interaction.id });

      if (!interaction.replied && !interaction.deferred) {
        await interaction.reply({
          content: 'An error occurred while processing your request.',
          ephemeral: true,
        });
      }
    }
  },
};
```

### 4.4 Component Handler Pattern (customId routing)

```typescript
// bot/src/handlers/component-handler.ts
import {
  ButtonInteraction,
  StringSelectMenuInteraction,
  ModalSubmitInteraction,
} from 'discord.js';

// Map of customId prefixes to handlers
const componentRegistry = new Map<string, ComponentHandler>();

export interface ComponentHandler {
  customId: string; // Full match or prefix
  execute: (interaction: any) => Promise<void>;
}

export function registerComponent(handler: ComponentHandler) {
  componentRegistry.set(handler.customId, handler);
}

export async function handleComponent(
  interaction: ButtonInteraction | StringSelectMenuInteraction | ModalSubmitInteraction
) {
  // Match customId by exact match, then by prefix
  const id = interaction.customId;
  const handler =
    componentRegistry.get(id) ??
    // Check prefix matches (e.g., "ticket:claim:123" matches "ticket:claim")
    [...componentRegistry.entries()]
      .find(([key]) => id.startsWith(key + ':') || id.startsWith(key + '_'))
      ?.[1];

  if (handler) {
    await handler.execute(interaction);
  } else {
    await interaction.reply({
      content: 'Unknown component interaction.',
      ephemeral: true,
    });
  }
}
```

### 4.5 Command Registration / Deployment

```typescript
// Separate script run during CI/CD or manually
// bot/scripts/deploy-commands.ts
import { REST, Routes, SlashCommandBuilder } from 'discord.js';
import { loadCommands } from '../src/handlers/command-handler';

async function deploy() {
  const commands = await loadCommands();
  const commandData = commands.map((cmd) => cmd.data.toJSON());

  const rest = new REST({ version: '10' }).setToken(process.env.DISCORD_TOKEN!);

  if (process.env.DEV_GUILD_ID) {
    // Guild commands (instant, for development)
    await rest.put(
      Routes.applicationGuildCommands(
        process.env.DISCORD_CLIENT_ID!,
        process.env.DEV_GUILD_ID
      ),
      { body: commandData }
    );
  } else {
    // Global commands (takes up to 1 hour to propagate)
    await rest.put(
      Routes.applicationCommands(process.env.DISCORD_CLIENT_ID!),
      { body: commandData }
    );
  }
}

deploy();
```

---

## 5. Axum API Architecture

### 5.1 Application State

```rust
// api/src/state.rs
use sqlx::PgPool;
use redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub config: Arc<Config>,
}

// Implements FromRef for extractor use
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
```

### 5.2 Route Composition

```rust
// api/src/main.rs
use axum::{Router, middleware};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use crate::routes;

async fn main() {
    // ... init tracing, load config, create pool ...

    let state = AppState { db, redis, config };

    let app = Router::new()
        // Public routes
        .route("/health", get(routes::health::health_check))
        .route("/ready", get(routes::health::readiness))

        // API v1 — protected routes
        .nest(
            "/api/v1",
            Router::new()
                .nest("/guilds", routes::guilds::router())
                .nest("/tickets", routes::tickets::router())
                .nest("/users", routes::users::router())
                .nest("/showcase", routes::showcase::router())
                .nest("/referrals", routes::referrals::router())
                .nest("/feedback", routes::feedback::router())
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    crate::middleware::auth::require_auth,
                ))
        )

        // Admin routes (stricter auth)
        .nest(
            "/api/admin",
            routes::admin::router()
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    crate::middleware::auth::require_admin,
                ))
        )

        // Webhook endpoints (Discord outgoing webhooks)
        .route("/api/webhooks/discord", post(routes::webhook::handle_discord))

        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()) // Tighten in production
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 5.3 Handler Pattern (Thin Handler → Service → Repository)

```rust
// api/src/handlers/ticket_handlers.rs
use axum::{extract::State, Json};
use crate::{services::ticket_service, models::ticket::*, state::AppState};

/// Thin handler — extracts request data, delegates to service
pub async fn create_ticket(
    State(state): State<AppState>,
    Json(req): Json<CreateTicketRequest>,
) -> Result<Json<TicketResponse>, AppError> {
    let ticket = ticket_service::create_ticket(&state.db, req).await?;
    Ok(Json(ticket.into()))
}

pub async fn list_tickets(
    State(state): State<AppState>,
    Query(params): Query<TicketListParams>,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let tickets = ticket_service::list_tickets(
        &state.db,
        params.guild_id,
        params.status,
    ).await?;
    Ok(Json(tickets.into_iter().map(Into::into).collect()))
}
```

### 5.4 Axum Middleware Stack

```rust
// api/src/middleware/mod.rs
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, SetRequestIdLayer, PropagateRequestIdLayer},
    trace::TraceLayer,
    cors::CorsLayer,
    timeout::TimeoutLayer,
    compression::CompressionLayer,
};

/// Standard middleware applied to the entire app
pub fn standard_layers<S>() -> ServiceBuilder<S> {
    ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            MakeRequestUuid,
        ))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http())
}

/// Applied to authenticated routes
pub fn auth_layer() -> AuthMiddlewareLayer {
    // JWT verification via FromRequestParts extractor
    // Implementation detail depends on auth strategy
}
```

### 5.5 Database Layer (SQLx)

```rust
// api/src/repository/ticket_repo.rs
use sqlx::PgPool;
use crate::models::ticket::{Ticket, TicketStatus};

pub async fn create_ticket(
    pool: &PgPool,
    guild_id: &str,
    author_id: &str,
    ticket_type: &str,
) -> Result<Ticket, sqlx::Error> {
    sqlx::query_as!(
        Ticket,
        r#"
        INSERT INTO tickets (guild_id, author_id, type, status)
        VALUES ($1, $2, $3, 'open')
        RETURNING id, guild_id, author_id, type, status as "status: TicketStatus",
                  created_at, closed_at
        "#,
        guild_id,
        author_id,
        ticket_type
    )
    .fetch_one(pool)
    .await
}
```

---

## 6. Inter-Process Communication (Bot ↔ API)

### 6.1 Communication Methods

| Method | Direction | Use Case | Technology |
|--------|-----------|----------|------------|
| REST API | Bot → API | CRUD operations, data queries | HTTP fetch from bot → Axum |
| Redis Streams | API → Bot | Send actions to Discord | Redis Streams consumer groups |
| Redis Streams | Bot → API | Analytics, event logging | Redis Streams consumer groups |
| Direct | Either | Emergency/real-time | `@discordjs/brokers` (PubSubRedisBroker) |

### 6.2 Redis Stream Protocol

```typescript
// API publishes action to bot
// Redis Stream: "bus:bot:actions"

interface BotAction {
  type: 'settings_update'
  | 'announcement'
  | 'role_sync'
  | 'reload_commands'
  | 'send_message'
  | 'create_channel';

  guildId: string;
  payload: Record<string, unknown>;
  timestamp: number;
  idempotencyKey: string; // Prevent double-processing
}

// Bot publishes event to API
// Redis Stream: "bus:api:events"

interface ApiEvent {
  type: 'command_executed'
  | 'user_joined'
  | 'ticket_created'
  | 'button_clicked'
  | 'error';

  guildId?: string;
  userId?: string;
  payload: Record<string, unknown>;
  timestamp: number;
}
```

### 6.3 Why Not Direct gRPC or WebSocket?

For the Escluse bot scale (community bot, not Discord-scale), Redis Streams hit the sweet spot:
- Already have Redis in the stack (zero new infrastructure)
- Messages persist if consumer is down (streams survive)
- Exactly-once processing via consumer groups
- Pub/Sub within same tech stack
- Verified pattern: Rostra, 1Git2Clone/serenity-discord-bot, and Ovencord hybrid-sharding all use Redis for this

---

## 7. Sharding Strategy

### 7.1 When Sharding is Needed

| Guild Count | Sharding Required? | Strategy |
|-------------|-------------------|----------|
| < 250 | No | Single process, no sharding |
| 250–2,500 | Discord mandates at 250 | Native Discord.js sharding |
| 2,500+ | Yes | Hybrid sharding (multiple shards per process) |

**For Escluse (early stage):** Start without sharding. Add `Discord.js` built-in sharding when approaching 250 guilds. Do not prematurely optimize.

### 7.2 Adding Sharding Later

```typescript
// When needed, switch to sharding manager
// bot/src/index.ts (sharded version)
import { ShardingManager } from 'discord.js';

const manager = new ShardingManager('./bot.js', {
  token: process.env.DISCORD_TOKEN,
  totalShards: 'auto', // Discord recommends auto
  respawn: true,
});

manager.on('shardCreate', (shard) => {
  logger.info(`Launched shard ${shard.id}`);
});

manager.spawn();
```

**For Bun runtime** (if Discord.js sharding manager has compatibility issues, use hybrid-sharding):
- `@ovencord/hybrid-sharding` — Bun-native, Redis heartbeats, zero-downtime rolling restarts
- `galactic.ts` — Cross-machine sharding, Docker-native
- `@lacunahub/letsfrag` — Distributed, Redis-based automatic shard rebalancing

**Recommendation:** Start with no sharding. If Bun/Discord.js sharding has issues, `@ovencord/hybrid-sharding` is the closest Bun-native match.

---

## 8. Deployment Architecture

### 8.1 Docker Compose Topology

The bot follows the existing escluse monorepo's Docker patterns (postgres:16-alpine, redis:7-alpine, app-network bridge):

```yaml
# escluse-bot/docker-compose.yml
services:
  # Reuse existing escluse postgres by connecting to same network
  # OR start isolated for standalone mode

  bot:
    build:
      context: ./bot
      dockerfile: Dockerfile
    container_name: escluse_bot
    environment:
      DISCORD_TOKEN: ${DISCORD_TOKEN}
      DISCORD_CLIENT_ID: ${DISCORD_CLIENT_ID}
      DEV_GUILD_ID: ${DEV_GUILD_ID:-}
      API_URL: http://api:8080
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      LOG_LEVEL: info
    depends_on:
      redis:
        condition: service_healthy
    networks:
      - bot-network
    restart: unless-stopped

  api:
    build:
      context: ./api
      dockerfile: Dockerfile
    container_name: escluse_bot_api
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgresql://${DB_USER}:${DB_PASSWORD}@postgres:5432/${DB_NAME}
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      DISCORD_TOKEN: ${DISCORD_TOKEN} # For webhook verification
      JWT_SECRET: ${JWT_SECRET}
      RUST_LOG: info
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - bot-network
    restart: unless-stopped

  postgres:
    image: postgres:16-alpine
    container_name: escluse_bot_postgres
    environment:
      POSTGRES_USER: ${DB_USER:-bot_user}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: ${DB_NAME:-escluse_bot}
    ports:
      - "5433:5432"  # Offset from main escluse postgres
    volumes:
      - bot_postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER:-bot_user}"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - bot-network
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    container_name: escluse_bot_redis
    command: redis-server --requirepass ${REDIS_PASSWORD}
    ports:
      - "6380:6379"  # Offset from main escluse redis
    volumes:
      - bot_redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "--raw", "incr", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - bot-network
    restart: unless-stopped

networks:
  bot-network:
    driver: bridge

volumes:
  bot_postgres_data:
  bot_redis_data:
```

### 8.2 Docker Build Files

**Bot Dockerfile (multi-stage):**
```dockerfile
# Stage 1: Install dependencies
FROM oven/bun:1.2 AS deps
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Stage 2: Build & run
FROM oven/bun:1.2 AS runner
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN bun build ./src/index.ts --outdir ./dist

CMD ["bun", "dist/index.js"]
```

**API Dockerfile (multi-stage):**
```dockerfile
# Stage 1: Build
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src
COPY src/ ./src/
RUN cargo build --release

# Stage 2: Run
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/escluse-bot-api /usr/local/bin/
COPY migrations/ /app/migrations/
CMD ["escluse-bot-api"]
```

### 8.3 Caddy Integration

For production, route through the existing escluse gateway (matches existing `gateway/Caddyfile.prod` pattern):

```
# In existing escluse gateway/Caddyfile.prod or a new Caddyfile
bot-api.esluce.com {
    reverse_proxy escluse_bot_api:8080
    encode zstd gzip
}
```

---

## 9. Build Order (Phase Dependencies)

The architecture dictates a clear build sequence based on dependencies:

```
Phase 1: Project Scaffolding (no dependencies)
  - Initialize bun project (bot/)
  - Initialize cargo project (api/)
  - Docker Compose with postgres + redis
  - CI/CD pipeline (lint, typecheck, build)

Phase 2: Database Schema (depends on Phase 1)
  - SQLx migrations for all initial tables
  - Rust models + repository layer

Phase 3: API Core (depends on Phase 2)
  - AppState, config, error handling
  - Health/readiness endpoints
  - Axum server startup with graceful shutdown
  - Middleware stack (tracing, CORS, request ID)

Phase 4: Bot Core (depends on Phase 1)
  - Discord.js client setup (intents, partials)
  - Command handler (auto-load, register)
  - Event handler (auto-load)
  - Interaction router
  - `interactionCreate` event
  - `/ping` command (verification)

Phase 5: Bot-API Communication (depends on Phase 3 + Phase 4)
  - Redis client in both processes
  - Bot API client layer (HTTP to Axum)
  - Redis Streams for pub/sub
  - `@discordjs/brokers` or custom stream consumer

Phase 6: Feature Delivery (depends on Phase 5)
  - ONB: Welcome + interest selection
  - INFO: Documentation search, pricing, roadmap
  - SUPP: Ticket system
  - SHOW: Showcase submission
  - FDBK: Feedback and polls
  - RECG: Referrals and achievements
  - ADMN: Admin commands (channel, role, settings)

Phase 7: Production Hardening (depends on Phase 6)
  - Rate limiting (Redis-backed)
  - Cooldown system
  - Permission verification
  - Error tracking (Sentry)
  - Structured logging
  - Graceful shutdown
```

### Dependency Graph

```
Phase 1 (Scaffold)
   ├── Phase 2 (DB Schema)
   │    └── Phase 3 (API Core)
   │         └── Phase 5 (Bot-API Comm)
   └── Phase 4 (Bot Core)
        └── Phase 5
             └── Phase 6 (Features)
                  └── Phase 7 (Hardening)
```

---

## 10. Anti-Patterns to Avoid

### 10.1 Bot Doing Direct Database Queries
**Why bad:** Couples bot to DB schema, DB credentials in bot process, no access control, harder to migrate.

**Instead:** Bot always calls API service for persistence. API owns the data layer.

### 10.2 Fat Event Handler (Single Massive `interactionCreate.ts`)
**Why bad:** Unmaintainable at 30+ commands, impossible to test, merge conflicts on every PR.

**Instead:** One file per command + one file per component type, auto-loaded by handlers.

### 10.3 Blocking the Gateway with Synchronous Operations
**Why bad:** Discord Gateway expects heartbeats; blocking the event loop causes disconnects.

**Instead:**
- Always `deferReply()` before slow operations
- Use queue-based processing for heavy tasks (thumbnail generation, batch operations)
- Never `await` long operations in the main event handler path

### 10.4 Hardcoding Channel/Role IDs
**Why bad:** Breaks across servers, requires code changes for every new server.

**Instead:**
- Always read from `guild_settings` table
- Use descriptive config keys, not hardcoded snowflakes
- Use Discord.js's built-in resolvers (`interaction.guild.channels.cache`)

### 10.5 API → Bot HTTP Calls (Instead of Pub/Sub)
**Why bad:** HTTP requires the bot to run a web server, complicates deployment, adds latency.

**Instead:** Use Redis Streams pub/sub. API publishes actions, bot subscribes. No extra HTTP server needed in the bot process.

### 10.6. Ignoring the 3-Second Interaction Window
**Why bad:** Discord rejects unacknowledged interactions after 3 seconds.

**Instead:**
- Simple responses: Reply directly
- Slow responses: `deferReply()` → `editReply()`
- Very slow responses: `deferReply()` → process in background → send followup via webhook

---

## 11. Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Bot language | TypeScript + Bun | Discord.js ecosystem maturity, Bun's speed for I/O-bound operations, fast iteration |
| API language | Rust (Axum) | Consistency with existing escluse Rust stack, type safety, performance for DB operations |
| Two processes | Bot + API | Clear separation of concerns, independent scaling, language-native tooling |
| DB access pattern | Bot → REST API → DB | Single auth boundary, schema encapsulation, access control |
| Inter-process comm | Redis Streams | Already in stack, persistence, backpressure, consumer groups |
| Slash commands | File-per-command | Auto-loaded, testable, merge-friendly, Discord.js standard |
| Component routing | customId prefix matching | Deterministic routing, no DB lookup, O(1) dispatch |
| Sharding | Start without, add later | Premature optimization is harmful; simple approach sufficient for <250 guilds |
| Command deployment | Guild-scoped in dev, global in prod | Instant iteration in dev, production consistency |

---

## 12. Scalability Considerations

| Dimension | At Launch (1-50 servers) | At Growth (250+ servers) | At Scale (2500+ servers) |
|-----------|--------------------------|--------------------------|--------------------------|
| Bot instances | 1 process, no sharding | 2-3 shards (native) | Hybrid sharding (2-4 shards/cluster) |
| API instances | 1 replica | 2 replicas behind load balancer | Horizontal auto-scaling |
| Database | Single PostgreSQL | PostgreSQL + connection pooling (PgBouncer) | Read replicas for analytics |
| Cache | Redis single instance | Redis with persistence | Redis Cluster or ElastiCache |
| Queue | In-process | BullMQ with Redis | BullMQ with Redis Cluster |
| Deployment | Docker Compose on single host | Docker Swarm or Nomad | Kubernetes |

---

## Sources

- Discord.js Official Guide: https://discordjs.guide/ (HIGH confidence)
- Discord.js v14 Docs: https://discord.js.org/docs/packages/discord.js/14.26.4 (HIGH confidence)
- Discord API Documentation: https://docs.discord.com/developers/interactions/receiving-and-responding (HIGH confidence)
- Axum SQLx PostgreSQL Example: https://github.com/tokio-rs/axum/blob/main/examples/sqlx-postgres/src/main.rs (HIGH confidence)
- Discord Bot Architecture @ 2000+ Members: https://moderngrindtech.com/blog/discord-bot-architecture-scale (MEDIUM confidence)
- Multi-Server Bot Architecture: https://space-node.net/blog/discord-multi-server-bot-architecture-2026 (MEDIUM confidence)
- Day 16: Pub/Sub Primitives (Discord Redis analysis): https://javatsc.substack.com/p/day-16-pubsub-primitives-decoupling (MEDIUM confidence)
- Rostra Discord Bot (production reference): https://github.com/AstorisTheBrave/Rostra (MEDIUM confidence)
- Turbo-Gravity Rust/Axum Bot: https://github.com/TurboRx/Turbo-Gravity (MEDIUM confidence)
- SufBotV5 (Bot + API + Dashboard pattern): https://github.com/MRsuffixx/SufBotV5 (MEDIUM confidence)
- Discord.js Brokers (Redis Pub/Sub): https://github.com/discordjs/discord.js/blob/main/packages/brokers/README.md (HIGH confidence)
- Bulletproof Rust Web (Axum patterns): https://github.com/gruberb/bulletproof-rust-web (MEDIUM confidence)
- Ovencord Hybrid Sharding: https://github.com/ovencord/hybrid-sharding (MEDIUM confidence)
