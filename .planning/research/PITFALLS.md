# Domain Pitfalls: Discord Community Bot

**Domain:** Discord community management bot (TypeScript + Bun + Discord.js, Rust Axum API, PostgreSQL + Redis)
**Researched:** 2026-06-18
**Overall confidence:** HIGH (official Discord API docs + discord.js guide + community experience)

---

## Critical Pitfalls

Mistakes that cause rewrites, downtime, or data loss.

### Pitfall 1: Slash Command Registration — Global vs Guild Confusion

**What goes wrong:**
Commands don't appear, appear inconsistently, or the bot crashes on startup because command registration is run on every `ready` event instead of as a separate deployment step.

**Why it happens:**
- Developers run command registration inside `client.on('ready', ...)` which fires every time the bot reconnects (which can be frequent)
- Mixing guild commands (instant, for testing) with global commands (up to 1-hour cache) without understanding the propagation delay
- Not using the standalone `REST` API for deployment, instead relying on `client.application.commands.set()` through the gateway

**Consequences:**
- Guild commands disappear when global commands overwrite them (or vice versa)
- Commands show "Application did not respond" because the registration expected guild-scoped but got global
- Discord rate-limits your app for hitting command creation too frequently (daily limit on command creations)
- Changes to command definitions don't appear for up to 1 hour (global commands are cached by Discord)

**Prevention:**
- **Never register commands in the `ready` event.** Use a standalone deployment script run separately (e.g., `bun run deploy-commands.ts`)
- Use **guild commands** during active development — they update instantly — and **global commands** only for stable releases
- Keep a separate `deploy-commands.ts` that uses the standalone `REST` client (not the full `Client`):
  ```typescript
  import { REST, Routes } from 'discord.js';
  const rest = new REST({ version: '10' }).setToken(token);
  await rest.put(Routes.applicationGuildCommands(clientId, guildId), { body: commands });
  ```
- Track the "guild ID" for dev in env vars; use `applicationCommands` (global) for production

**Detection:**
- Commands are missing from the slash command menu
- `DiscordAPIError: 400 Bad Request` when registering
- Commands show "The application did not respond" after the interaction is triggered

**Phase mapping:** This must be correct from Phase 1 (bot setup). Fixing it later means a painful migration when switching from guild to global.

---

### Pitfall 2: Interaction Reply Deadline — The 3-Second Window

**What goes wrong:**
The bot receives an interaction (slash command, button click, etc.) but fails to respond within 3 seconds. Discord shows "The application did not respond", the interaction token expires, and the user is left with a broken experience.

**Why it happens:**
- Command handler performs long operations before replying (DB queries, API calls, file processing)
- Developer forgot to call `interaction.deferReply()` for operations that take >3 seconds
- Unhandled promise rejection crashes the handler before it can respond

**Consequences:**
- Token expires after 3 seconds (initial reply) or 15 minutes (after deferral)
- `interaction.editReply()` silently fails if called after expiration
- The ephemeral flag **cannot be changed after deferral** — setting `ephemeral: true` on `editReply()` is silently ignored
- Followup messages fail because the token is dead

**Prevention:**
- Pattern: `interaction.deferReply({ ephemeral: true })` immediately for any command that does async work
- Wrap all handler logic in try/catch that calls `interaction.editReply()` or `interaction.followUp()` on error
- Use a global error handler on `interactionCreate` to catch unhandled rejections:
  ```typescript
  client.on(Events.InteractionCreate, async (interaction) => {
    try { /* handle */ }
    catch (error) {
      if (interaction.isRepliable()) {
        await interaction.reply({ content: 'Something went wrong.', ephemeral: true });
      }
    }
  });
  ```

**Detection:**
- "The application did not respond" error in the Discord client
- `Interaction has already been acknowledged.` errors in console when double-acking
- `Cannot edit reply before acknowledging` errors

**Phase mapping:** Phase 2 (core command handler). Establish the pattern early — retrofitting it later is tedious but not a rewrite.

---

### Pitfall 3: Privileged Gateway Intents Not Enabled

**What goes wrong:**
The bot connects but guild members are unavailable, message content is empty, or presence data is missing. The bot silently receives `undefined` for critical fields.

**Why it happens:**
- Developer forgets to enable `GuildMembers`, `MessageContent`, or `GuildPresences` in the Discord Developer Portal under "Privileged Gateway Intents"
- Discord made these opt-in (privacy by design) and they are **off by default**
- Verified bots (>100 guilds) must request these intents through Discord's verification process, which can take weeks

**Consequences:**
- `message.content` is empty string — the bot cannot read message content even with `GuildMessages` intent
- `guild.members.cache` contains only the bot itself — cannot look up roles, nicknames, or user data
- Member join/leave events never fire — onboarding features silently fail
- Bot crashes with `[DISALLOWED_INTENTS]` error at startup if intents are requested but not enabled in the portal

**Prevention:**
- Enable intents in **both** code and the Developer Portal:
  ```typescript
  const client = new Client({
    intents: [
      GatewayIntentBits.Guilds,
      GatewayIntentBits.GuildMembers,      // privileged
      GatewayIntentBits.MessageContent,     // privileged
      GatewayIntentBits.GuildPresences,     // privileged
    ]
  });
  ```
- For verified bots, apply for privileged intents early in the verification process
- If you don't need all intents, don't request them — reduces verification friction
- Minimum for this project: `Guilds`, `GuildMembers` (for roles/onboarding), `MessageContent` (if needed for prefix commands or content analysis)

**Detection:**
- `[DISALLOWED_INTENTS]` error at startup
- `message.content === ''` for all messages
- `guild.members.cache.size === 1` (only the bot itself)
- `guildMemberAdd`/`guildMemberRemove` events never fire

**Phase mapping:** Phase 1 (bot setup). Must be correct from day one. Missing intents after rollout means a bot restart and possible verification delay.

---

### Pitfall 4: Token Management & Security

**What goes wrong:**
Bot token is leaked to GitHub, logs, or error messages. A compromised token lets anyone take full control of the bot — read messages, ban members, delete channels, impersonate the bot across every server it's in.

**Why it happens:**
- Token hardcoded in source code (`client.login('ODE2...')`)
- `.env` file committed to version control
- Token logged in console.error output or caught in stack traces
- Token passed in CI/CD env vars that are printed in build logs
- Sharing screenshots or config files that contain the token

**Consequences:**
- Token is discovered by automated GitHub scrapers within minutes of a push
- Attacker takes full control of the bot, potentially destroying months of community work
- If the bot has Administrator permission, the attacker controls every server the bot is in
- Regenerating the token breaks the running bot (requires restart and redeployment)
- Git history contains the old token — must use `git filter-branch` or BFG to purge

**Prevention:**
- **Never hardcode tokens.** Use environment variables exclusively:
  ```typescript
  client.login(process.env.DISCORD_TOKEN!);
  ```
- Add `.env` to `.gitignore` **before** the first commit
- Use a secret manager (Docker secrets, HashiCorp Vault, platform env vars) in production
- Validate `process.env.DISCORD_TOKEN` exists at startup — fail fast with a clear message if missing
- On the Rust (Axum) side: never accept tokens from user input; use env vars for any Discord API interaction
- Set up secret scanning (GitGuardian, GitHub secret scanning) on the repository
- Rotate tokens periodically and after any suspected compromise

**Detection:**
- GitHub security alert about an exposed token
- Bot acting unexpectedly (sending messages, kicking members without command)
- Unauthorized API calls in Discord audit log

**Phase mapping:** Phase 1 (project setup). Token security is foundational. Cannot be retrofitted if already committed to git.

---

### Pitfall 5: Database Schema Design for Multi-Guild Data

**What goes wrong:**
The database schema assumes single-guild operation (flat tables without `guild_id`), leading to data collision when the bot joins a second server. Settings, roles, and user data from different guilds mix together.

**Why it happens:**
- Developer builds and tests on one guild, never considers multi-guild data isolation
- Tables like `settings` or `roles` lack a `guild_id` foreign key
- Indexes are created with `UNIQUE` constraints that don't include `guild_id`, causing constraint violations

**Consequences:**
- Guild A's settings overwrite Guild B's settings
- A user's XP/roles from one guild appear in another (data leak)
- UNIQUE constraint violations crash the bot
- Schema migration to add `guild_id` requires downtime and data migration

**Prevention:**
- **Every guild-scoped table must have `guild_id` as a composite key or partition:**
  ```sql
  CREATE TABLE guild_settings (
    guild_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value JSONB NOT NULL,
    PRIMARY KEY (guild_id, key)
  );
  ```
- Use PostgreSQL partitioning by `guild_id` for large tables
- Create indexes that include `guild_id`:
  ```sql
  CREATE INDEX idx_guild_settings_lookup ON guild_settings(guild_id);
  ```
- User data tables should distinguish between:
  - **Global user data** (user_id keyed) — applies across all guilds
  - **Per-guild user data** (user_id + guild_id composite key) — guild-specific XP, roles, settings
- Always validate `guild_id` exists when inserting guild-scoped data

**Detection:**
- Settings from one guild appear in another
- Duplicate key errors on insert
- Unexpected permission grants or role assignments

**Phase mapping:** Phase 1 (database schema design). Fixing this later requires a data migration and potential downtime. Schema must be multi-guild from the start.

---

### Pitfall 6: Discord API Rate Limiting — Not Being Bucket-Aware

**What goes wrong:**
Bot hits HTTP 429 rate limits, gets globally blocked, or gets a temporary IP ban from Cloudflare. Bot goes silent for minutes to hours.

**Why it happens:**
- Sending messages too fast in a single channel (limit: 5 per 5 seconds)
- Bulk role operations (10 role modifications per 10 seconds per guild)
- Fetching many guild members concurrently without respecting per-route limits
- Using `guild.members.fetch()` without any caching strategy — hitting the same endpoint repeatedly
- Making 50+ requests per second (global limit for all bots)

**Consequences:**
- `HTTP 429 Too Many Requests` with `Retry-After` header — bot must pause
- Global rate limit: ALL requests blocked for the duration
- Invalid request limit (10,000 per 10 minutes) exceeded → temporary Cloudflare IP ban
- Webhook rate limits (5 per 2 seconds per webhook) — silently drops messages
- The bot appears offline to users

**Prevention:**
- **Discord.js handles per-route rate limits automatically** — but only if using the built-in `REST` client. Don't bypass it with raw `fetch()` calls
- Listen for rate limit events to monitor:
  ```typescript
  client.rest.on('rateLimited', (info) => {
    console.warn(`Rate limited on ${info.route} for ${info.timeToReset}ms`);
  });
  ```
- Cache aggressively with Redis to reduce API calls:
  - Guild member lists → cache, refresh periodically
  - Channel lists → cache, invalidate on channel create/delete events
  - Role data → cache, invalidate on role update events
- Use bulk endpoints where available:
  - `guild.members.fetch()` with no args fetches all members at once (respects rate limit)
  - `channel.bulkDelete()` for message cleanup
- Implement per-command cooldowns using Redis (not in-memory Collection which resets on restart):
  ```typescript
  const key = `cooldown:${commandName}:${userId}`;
  const exists = await redis.exists(key);
  if (exists) return interaction.reply({ content: 'Please wait...', ephemeral: true });
  await redis.set(key, '1', 'EX', cooldownSeconds);
  ```
- Webhook operations: queue messages and batch them (respect 5 per 2 seconds)
- For the Rust Axum API service: implement retry with exponential backoff for any outgoing Discord API calls

**Detection:**
- `DiscordAPIError: 429 Too Many Requests` in logs
- `X-RateLimit-Global: true` header in responses
- Bot stops responding but process is still running
- `Retry-After: XX` headers in responses

**Phase mapping:** Phase 2 (core features). Rate limit handling must be built into the command handler and API client layers from the beginning.

---

### Pitfall 7: Cross-Shard Communication Not Designed For

**What goes wrong:**
Bot grows beyond 2,500 guilds (Discord's mandatory sharding threshold). The in-memory cache is now split across multiple processes. Commands that worked on single shard break because data isn't shared.

**Why it happens:**
- Sharding is not planned for early enough — at 2,500 guilds, Discord refuses connections from non-sharded bots
- In-memory Collections (cooldowns, caches) are per-shard — shard 0 doesn't know what shard 1 cached
- `client.users.cache.get()` only checks the current shard's cache
- `guild.members.fetch()` doesn't work across shards without `broadcastEval`

**Consequences:**
- Emergency sharding implementation under time pressure
- Command cooldowns reset per-shard (user spams across shards)
- Analytics (total guild count, total user count) are wrong unless aggregated
- Database lookups by cached data fail because data isn't in this shard's cache
- Some guilds are "invisible" to the bot's logic

**Prevention:**
- **Start with `ShardingManager` even at small scale** — it's trivial to set up and prevents migration pain:
  ```typescript
  const manager = new ShardingManager('./bot.js', {
    token: process.env.DISCORD_TOKEN,
    totalShards: 'auto', // Discord tells you how many
  });
  manager.spawn();
  ```
- Store all mutable state in **Redis** (not in-memory) from day one:
  - Cooldowns → Redis with TTL
  - Rate limit tracking → Redis
  - Temporary state → Redis
- Use PostgreSQL for persistent data (already planned)
- Use `broadcastEval` for cross-shard aggregation:
  ```typescript
  const results = await client.shard.broadcastEval(c => c.guilds.cache.size);
  const totalGuilds = results.reduce((a, b) => a + b, 0);
  ```
- The Rust Axum API is unaffected (it's a separate service), but must be aware of sharding for any WebSocket event handling

**Phase mapping:** Phase 1 (architecture decision). Even if the bot starts with 1 guild, design for sharding. Redis as shared state is the key enabler. Implement `ShardingManager` before 500 guilds to avoid scramble.

---

### Pitfall 8: Embed Validation — Silent Rejection of Invalid Emebds

**What goes wrong:**
Commands that return rich embeds fail silently or return "Invalid Form Body" errors. The error message is cryptic and doesn't say which field is the problem.

**Why it happens:**
- Embed descriptions exceed 4,096 characters
- Total combined embed text across all embeds exceeds 6,000 characters
- Field values are empty strings (Discord rejects `""`)
- Field name/value exceeds 256/1024 character limit
- More than 25 fields or 10 embeds per message
- More than 10 embeds in a single message

**Consequences:**
- `DiscordAPIError: Invalid Form Body` with no clear indicator of which field overflowed
- Dynamic embeds from DB data fail unpredictably when data length varies
- Bot appears broken to users — command does nothing
- Error is logged but hard to debug because Discord returns `{"code": 50035}` with nested error paths

**Prevention:**
- Build a validation helper that enforces ALL embed limits before sending:
  ```typescript
  const EMBED_LIMITS = {
    title: 256,
    description: 4096,
    fieldName: 256,
    fieldValue: 1024,
    footerText: 2048,
    authorName: 256,
    totalEmbeds: 10,
    totalFields: 25,
    totalChars: 6000,
  };
  ```
- Filter out fields with empty `name` or `value`:
  ```typescript
  embed.fields = fields.filter(f => f.name && f.value);
  ```
- Truncate user-generated content before embedding:
  ```typescript
  const truncate = (s: string, len: number) => s.length > len ? s.slice(0, len - 1) + '…' : s;
  ```
- For the docs search command (`/escluse docs`) and changelog features: paginate results instead of packing everything into one embed
- Use `EmbedsBuilder` from discord.js which validates at build time

**Detection:**
- `DiscordAPIError: 50035` with `Invalid Form Body`
- Error contains `"BASE_TYPE_MAX_LENGTH"` or `"Must be 4096 or fewer in length."`
- Command silently does nothing (if error is swallowed)

**Phase mapping:** Phase 2 (core features). Build the embed validation utility before implementing any embed-generating commands.

---

## Moderate Pitfalls

### Pitfall 9: In-Memory Cache as Primary Data Store

**What goes wrong:**
Bot restarts and loses all in-memory state — cooldowns, temporary bans, ongoing ticket sessions, verification states. Users can immediately bypass restrictions.

**Why it happens:**
- Using discord.js's built-in Collections for cooldowns and temporary state
- Not persisting temporary state to Redis
- Assuming bot uptime is infinite

**Prevention:**
- In-memory Collections are for runtime cache only, not as primary storage
- Use **Redis** for all ephemeral state:
  - Cooldowns → `SET key EX 30` (auto-expire after 30s)
  - Ticket sessions → `SET ticket:{channelId} {data}`
  - Verification state → `SET verify:{userId} {step} EX 600`
  - Rate limit counters → `INCR + EXPIRE`
- Redis is already in the stack — use it from day one
- For the Rust Axum API: Redis for session data, PostgreSQL for persistent data

### Pitfall 10: Guild Member Cache Not Populated

**What goes wrong:**
`guild.members.cache.get(userId)` returns `undefined` for legitimate guild members. Role checks fail, welcome messages don't send.

**Why it happens:**
- `GuildMembers` privileged intent not enabled (see Pitfall 3)
- Member cache only fills lazily — members must be fetched or arrive via events
- Small guilds may not trigger `guildMemberAdd` for existing members
- `cache.get()` returns `undefined` if the member hasn't been explicitly fetched yet

**Prevention:**
- After `client.on('ready', ...)`, eagerly fetch members for primary guild:
  ```typescript
  const guild = client.guilds.cache.get(guildId);
  await guild.members.fetch(); // Fetches ALL members into cache
  ```
- For multi-guild: use `client.guilds.cache.forEach(g => g.members.fetch())` (but respect rate limits!)
- Always have a fallback: if `cache.get()` returns undefined, use `guild.members.fetch(userId)` directly
- For interaction data, use `interaction.member` which is always populated for guild commands

### Pitfall 11: Webhook Payload Limits for Release Notifications

**What goes wrong:**
GitHub release notifications, changelog updates, or long embed descriptions fail with "400 Bad Request" because the payload exceeds Discord's limits.

**Why it happens:**
- Webhook embeds have the same limits as bot embeds (6,000 total chars)
- Long release notes or changelogs overflow the description field
- Multiple embeds in one message exceed 10-embed limit
- GitHub webhook payloads can be very large

**Prevention:**
- Truncate long descriptions with "… Read more: <link>"
- Split content across multiple messages for very long updates (with rate-limit respecting delay)
- Use Discord's `content` field for overflow text (2,000 chars) combined with embeds
- For the Rust Axum API: implement a webhook payload builder that validates limits before sending

### Pitfall 12: Hardcoded Channel/Role/User IDs

**What goes wrong:**
Bot breaks when it joins a new guild because channel/role/user IDs are hardcoded. Commands try to send messages to specific channels that don't exist in the new guild.

**Why it happens:**
- Development patterns like `const WELCOME_CHANNEL = '123456789'`
- Not storing guild-specific configuration in the database
- Assuming all guilds have the same structure

**Prevention:**
- Zero hardcoded Discord IDs in code. Store everything in `guild_settings` table:
  ```sql
  -- guild_settings(guild_id, key, value)
  INSERT INTO guild_settings VALUES ('guild_id', 'welcome_channel', '"123456789"');
  ```
- The Rust Axum API should manage guild configuration endpoints
- Use interaction data (`interaction.channelId`, `interaction.guildId`) instead of hardcoded references
- For commands that need specific channels (like welcome), store the mapping and provide a setup command to configure them

### Pitfall 13: Unhandled Promise Rejections Crashing the Bot

**What goes wrong:**
An unhandled promise rejection (e.g., from a failed Discord API call, database connection issue, or network timeout) crashes the entire Node.js/Bun process. The bot goes offline.

**Why it happens:**
- Missing `.catch()` on async operations
- Async event handlers that throw without try/catch
- Bun, like modern Node.js, terminates the process on unhandled promise rejections (DEP0018)
- Discord API calls failing (network, rate limits) without proper error handling

**Prevention:**
- **With Bun:** register a global handler for safety (though Bun may still terminate):
  ```typescript
  process.on('unhandledRejection', (error) => {
    console.error('Unhandled rejection:', error);
  });
  ```
- Wrap ALL event handlers in try/catch:
  ```typescript
  client.on(Events.InteractionCreate, async (interaction) => {
    try {
      // command logic
    } catch (error) {
      console.error(`Error in ${interaction.commandName}:`, error);
      await interaction.reply({ content: 'An error occurred.', ephemeral: true }).catch(() => {});
    }
  });
  ```
- Bun v1.3+ behavior: check if Bun throws on unhandled rejections or just warns (behavior may differ from Node)
- For the Rust Axum API: Rust's `Result` types make this less of an issue, but ensure `tokio::spawn` tasks have error handling

### Pitfall 14: Bot Going Offline During Deployments

**What goes wrong:**
Deploying a new version of the bot disconnects the WebSocket connection to Discord. The bot is offline for seconds to minutes while it reconnects and re-caches data. Users see the bot as offline.

**Why it happens:**
- Traditional deployment (stop → deploy → start) kills the process
- No zero-downtime strategy for the WebSocket connection
- Bot must re-cache guilds, members, channels on reconnect

**Prevention:**
- Use Docker with rolling updates (zero-downtime deployment)
- The Rust Axum API can be deployed independently (it's stateless HTTP)
- On bot restart, reconnect will happen automatically (discord.js handles this)
- Eagerly re-fetch member cache after reconnect (see Pitfall 10)
- Consider using PM2 or a process manager that supports graceful restart
- For Docker: `HEALTHCHECK` that verifies the WebSocket is connected
- Bot reconnection typically takes 2-5 seconds with a good network connection

---

## Minor Pitfalls

### Pitfall 15: Command Cooldowns Using In-Memory Storage

Relying on discord.js's in-memory `Collection` for cooldowns means cooldowns reset on bot restart. A user can spam commands immediately after a restart. Fix: use Redis with TTL for cooldowns — already in the stack.

### Pitfall 16: Not Handling `Unknown Interaction` Errors

When a user double-clicks a button or a command token expires, Discord returns `Unknown Interaction` (10062). Without handling this, the bot logs a confusing error. Always wrap interaction replies in try/catch and filter out error code 10062.

### Pitfall 17: Single Message Content Intent

If the bot doesn't need to read message content (only uses slash commands), don't enable `MessageContent` intent. This reduces verification friction and respects user privacy. This project primarily uses slash commands, so `MessageContent` may not be needed at all.

### Pitfall 18: Accidental Permission Escalation via Admin Role

Giving the bot's role `Administrator` permission makes it a high-value target. If the token is compromised, every server is compromised. Apply the principle of least privilege — only grant the specific permissions the bot needs (Send Messages, Manage Roles, Read Message History, etc.), not blanket Administrator.

### Pitfall 19: Not Handling Discord API Version Deprecations

Discord occasionally deprecates API versions. Discord.js v14 uses API v10. If discord.js is pinned to an old version and Discord sunsets the API version, the bot will stop working. Keep discord.js updated (currently 14.26.x as of mid-2026).

### Pitfall 20: Missing `GUILD_MEMBERS` Intent for Welcome Messages

The onboarding feature (`ONB-01`) needs `GuildMembers` privileged intent to receive `guildMemberAdd` events when new members join. Without it, the welcome system silently fails. Enable this in the Developer Portal and in code from day one.

### Pitfall 21: Rate Limiting the Rust Axum API's Discord Webhook Calls

If the Rust API sends Discord webhook messages (for release announcements, changelogs), respect the webhook rate limit of 5 requests per 2 seconds per webhook URL. Implement a queue or debounce mechanism in Rust.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|---|---|---|
| **Phase 1: Project setup** | Token committed to git (Pitfall 4) | Add `.env` to `.gitignore` before first commit. Use env vars. |
| **Phase 1: Database schema** | Schema not multi-guild (Pitfall 5) | Every table gets `guild_id` as composite key. |
| **Phase 1: Bot framework** | Intents not enabled (Pitfall 3) | Enable `Guilds`, `GuildMembers` minimum. Check Dev Portal. |
| **Phase 1: Slash commands** | Global vs guild confusion (Pitfall 1) | Guild commands for dev, global for prod. Standalone deploy script. |
| **Phase 2: Command handler** | Missing deferReply (Pitfall 2) | Always `deferReply()` for any async work. |
| **Phase 2: Embed generation** | Embed limits overflow (Pitfall 8) | Validation helper before any embed send. |
| **Phase 2: Caching** | In-memory only (Pitfall 9) | Use Redis for all ephemeral state from day one. |
| **Phase 3: Onboarding** | Hardcoded channel IDs (Pitfall 12) | Store channel config in DB. Setup command for guild admins. |
| **Phase 4: Tickets/support** | Interaction timeout on complex flows (Pitfall 2) | Defer early. Store ticket state in Redis. |
| **Phase 5: GitHub integration** | Webhook payload too large (Pitfall 11) | Truncate + link. Validate before sending. |
| **Phase 6: Realm/Servers** | Data not scoped to guild (Pitfall 5) | `guild_id` on every server/showcase record. |
| **Scale: 2,000+ guilds** | Sharding not implemented (Pitfall 7) | Start with ShardingManager early. Redis as shared state. |
| **Deployment** | Bot goes offline on deploy (Pitfall 14) | Docker rolling updates. Redis persistence for state. |

---

## Sources

| Source | Type | Confidence |
|--------|------|------------|
| [Discord API Rate Limits](https://docs.discord.com/developers/topics/rate-limits) | Official documentation | HIGH |
| [Discord Application Commands](https://docs.discord.com/developers/interactions/application-commands) | Official documentation | HIGH |
| [Discord Permissions](https://docs.discord.com/developers/topics/permissions) | Official documentation | HIGH |
| [discord.js Guide: Common Errors](https://discordjs.guide/popular-topics/errors) | Official guide | HIGH |
| [discord.js Guide: Intents](https://discordjs.guide/popular-topics/intents) | Official guide | HIGH |
| [discord.js Guide: Sharding](https://discordjs.guide/sharding) | Official guide | HIGH |
| [discord.js Guide: Cooldowns](https://discordjs.guide/additional-features/cooldowns) | Official guide | HIGH |
| [discord.js Guide: Command Registration](https://discordjs.guide/legacy/app-creation/deploying-commands) | Official guide | HIGH |
| [discord.js Guide: Updating to v14](https://discordjs.guide/legacy/additional-info/changes-in-v14) | Official guide | HIGH |
| [Bun Node.js Compatibility](https://bun.sh/docs/runtime/nodejs-compat) | Official docs | HIGH |
| [Space-Node: Discord Bot Sharding Guide](https://space-node.net/blog/discord-bot-sharding-scaling-guide-2025) | Community article | MEDIUM |
| [Space-Node: Bot Token Security](https://space-node.net/blog/discord-bot-security-token-protection) | Community article | MEDIUM |
| [Space-Node: Database Options](https://space-node.net/blog/discord-bot-database-options-2026) | Community article | MEDIUM |
| [Hooklistener: Webhook Debugging](https://www.hooklistener.com/guides/discord-webhook-debugging) | Community article | MEDIUM |
| [Discord Embed Limits](https://discord-webhook.com/en/blog/discord-webhook-embed-limits) | Community reference | MEDIUM |
| [Bun Compatibility 2026](https://dev.to/alexcloudstar/bun-compatibility-in-2026-what-actually-works-what-does-not-and-when-to-switch-23eb) | Community article | MEDIUM |
| [Build Discord Bot with Bun 2026](https://www.alexcloudstar.com/blog/build-discord-bot-bun-typescript-2026/) | Community article | MEDIUM |
| [DeepWiki: discord.js sharding](https://deepwiki.com/discordjs/discord.js/5.3-sharding-strategies-and-scaling) | Community reference | MEDIUM |
| [Discord Support: Bot Rate Limiting](https://support-dev.discord.com/hc/en-us/articles/6223003921559-My-Bot-is-Being-Rate-Limited) | Official support | HIGH |
| [Discord Support: Ephemeral Messages FAQ](https://support-apps.discord.com/hc/en-us/articles/26501839512855-Ephemeral-Messages-FAQ) | Official support | HIGH |
| [Friendify: Rate Limit Patterns 2025](https://friendify.net/blog/discord-rate-limit-handling-patterns-2025.html) | Community article | MEDIUM |
| [GitHub: discord.js issue #8248 (Bun)](https://github.com/discordjs/discord.js/issues/8248) | GitHub issue | MEDIUM |
| [GitHub: Bun issue #18023 (discord.js)](https://github.com/oven-sh/bun/issues/18023) | GitHub issue | MEDIUM |
