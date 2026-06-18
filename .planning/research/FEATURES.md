# Feature Landscape

**Domain:** Discord community management bot (open-source infrastructure product ecosystem)
**Researched:** 2026-06-18
**Confidence:** HIGH (verified across 20+ live bots including MEE6, Dyno, Carl-bot, YAGPDB, ProBot, VibeBot, PeakBot, Ticket Tool, community analyses, and GitHub implementations)

---

## Table of Contents

1. [Feature Categorization](#-feature-categorization)
2. [Table Stakes (Must-Have)](#-table-stakes)
3. [Differentiators (Competitive Advantage)](#-differentiators)
4. [Anti-Features (Explicitly NOT Build)](#-anti-features)
5. [Feature Dependency Map](#-feature-dependency-map)
6. [MVP Recommendation](#-mvp-recommendation)
7. [Phase Mapping](#-phase-mapping)
8. [Sources](#-sources)

---

## 📊 Feature Categorization

Every feature is rated on two axes:

- **Expectation**: Table Stake (must have) → Differentiator (competitive edge) → Anti-Feature (do not build)
- **Complexity**: Low (days) → Medium (weeks) → High (months) — assumes TypeScript/Bun/Discord.js stack

---

## ✅ Table Stakes

Features the market expects. Omitting any will make the bot feel incomplete compared to Carl-bot, Dyno, YAGPDB, etc. These are the price of entry.

### ---- ONBOARDING ----

### 1. Welcome Messages (table stake, LOW complexity)

**What:** Automated greeting when a user joins the server, posted to a designated welcome channel.

**Why it's table stake:** Every major bot (MEE6, Dyno, Carl-bot, ProBot, VibeBot, PeakBot, YAGPDB) ships this. It's the first thing a new member sees.

**Implementation approach:** Discord.js `guildMemberAdd` event → embed builder with placeholders (`{user}`, `{server}`, `{memberCount}`, `{createdAt}`). Store channel config in PostgreSQL.

**Complexity:** Low. 2-3 days including placeholder system.

**Dependencies:** None. First feature to build.

**Source:** [VibeBot Welcome docs](https://www.vibebot.gg/features/welcome), [Discord Community Onboarding FAQ](https://support.discord.com/hc/en-us/articles/11074987197975-Community-Onboarding-FAQ)

### 2. Auto-Role Assignment (table stake, LOW complexity)

**What:** Automatically assign 1-2 roles to new members on join (e.g., "Member", "New Arrival").

**Why it's table stake:** Every bot does it. Required for server gating and permission management.

**Implementation approach:** On `guildMemberAdd`, check DB for auto-role config for that guild → `member.roles.add()`. Support delayed assignment (e.g., after 24h remove "New" role).

**Complexity:** Low. 1-2 days.

**Dependencies:** Welcome Messages (shares event handling infrastructure).

**Source:** [Auto Role Bot docs](https://docs.discordlabs.org/auto-role-bot/), [Discord RoleLogic guides](https://rolelogic.faizo.net/docs/guides/common-scenarios)

### 3. Interest/Role Selection (table stake, MEDIUM complexity)

**What:** Members choose interests via buttons/dropdowns → auto-assigned roles that gate channel access. The project's ONB-02/ONB-03 (Self-Hosting, Minecraft, Game Server, Development, Exploring Alternatives).

**Why it's table stake:** Discord's native onboarding + every major bot offers this. Carl-bot built its reputation on "reaction roles." In 2026, button-based role selection is the norm.

**Implementation approach:** Embed + buttons (discord.js `ActionRowBuilder` with `ButtonBuilder`). On click, toggle the corresponding role. Store role-channel mappings in PostgreSQL. Use Discord's native onboarding as a complementary channel.

**Complexity:** Medium. 4-5 days (role toggle logic, permission checks, debounce).

**Dependencies:** Auto-Role (role infrastructure).

**Source:** [Carl-bot reaction roles](https://www.ionos.com/digitalguide/online-marketing/social-media/discord-bots/), [Discord Onboarding FAQ](https://support.discord.com/hc/en-us/articles/11074987197975-Community-Onboarding-FAQ)

### 4. Role-Based Permissions (table stake, MEDIUM complexity)

**What:** Every command and feature respects role hierarchy. Admins configure who can use which commands, who can access which channels.

**Why it's table stake:** Without this, the bot is unusable on any server larger than a friend group. This is the PERM requirement.

**Implementation approach:** Permission resolver middleware in Discord.js command handler. Check user roles against stored permission config for each command. Cache in Redis for performance. Command-level permission guards with fallthrough.

**Complexity:** Medium. 4-6 days (depends on scope — per-command, per-category, or per-feature).

**Dependencies:** Everything. Must be built early and wired into command handler architecture.

**Source:** [Discord.js permission system docs](https://discordjs.guide/slash-commands/permissions.html), [TicketsBot permission model](https://ticketsbot.org/blog/best-discord-ticket-bot-features-2025)

### ---- INFORMATION ----

### 5. Documentation Search Command (table stake, LOW complexity)

**What:** `/escluse docs <query>` — search documentation from Discord. Returns rich embed with relevant docs.

**Why it's table stake:** Information hub is a core promise. Without it, the bot is just another mod bot. This is the INFO-01 requirement.

**Implementation approach:** Pull docs from a knowledge base (markdown files or API). Use full-text search (PostgreSQL `tsvector` or a search index). Return top 3 results as embeds with title, snippet, link.

**Complexity:** Low. 3-4 days (including knowledge base setup).

**Dependencies:** None standalone. Can be docs-first before building FAQ system.

**Source:** [eesel AI approach](https://www.donkey.support/blog/best-discord-ticket-bots-2026), [CommunityOne Spark AI](https://blog.communityone.io/best-discord-bots/)

### 6. FAQ System (table stake, LOW-MEDIUM complexity)

**What:** Quick-answer commands for common questions. `/esclue what-is`, `/escluse pricing`, etc. INFO-04.

**Why it's table stake:** Every product bot has this. Reduces repetitive questions.

**Implementation approach:** Key-value FAQ database. Slash commands that return predefined embeds. Support for categories and search. Can be extended to "smart FAQ" later.

**Complexity:** Low-Medium. 2-3 days.

**Dependencies:** Documentation Search (shares knowledge base infrastructure).

### ---- SUPPORT ----

### 7. Ticket System (table stake, HIGH complexity)

**What:** Users open private support tickets via buttons/commands. Staff manage, claim, close tickets. Includes transcripts, categories, auto-close.

**Why it's table stake:** This is the single most requested feature in the project requirements (SUPP-01). Dedicated ticket bots are a whole category. In 2026, tickets with transcripts, claim system, and auto-close are the minimum.

**Implementation approach:** Create private channel per ticket. Button panel → modal for category selection → create channel → notify staff. Track state in PostgreSQL. Auto-close after inactivity. Web dashboard for transcripts (nice-to-have v1.1).

**Critical sub-features (all table stake for a ticket system in 2026):**
- Ticket categories (Support, Bug Report, Appeal, etc.) with different routing
- Claim system (prevent double-reply)
- Auto-transcripts on close (HTML or link)
- Auto-close on inactivity (configurable)
- Staff-only channel for ticket discussion
- Button + slash command creation methods

**Complexity:** High. 2-3 weeks for a production-quality system.

**Dependencies:** Role-Based Permissions (staff roles), Channel Management (ADMN-01 for creating ticket channels).

**Source:** [Mava ticket comparison](https://www.mava.app/blog/discord-ticket-bots-compared), [PeakBot ticket analysis](https://peakbot.pro/blog/best-discord-ticket-bots-2026), [TicketBot features guide](https://ticketsbot.org/blog/best-discord-ticket-bot-features-2025)

### 8. Structured Bug Reports (table stake, MEDIUM complexity)

**What:** When a ticket is created for "Bug Report," the bot prompts for version, OS, steps to reproduce, logs. SUPP-02.

**Why it's table stake:** Reduces back-and-forth. Standard in modern ticket bots via custom forms.

**Implementation approach:** Modal form (Discord.js `ModalBuilder`) that fires on ticket type selection. Store structured data in the ticket's first message as an embed. Include in transcript.

**Complexity:** Medium. 3-5 days.

**Dependencies:** Ticket System (SUPP-01).

**Source:** [SCNX feature comparison](https://docs.scnx.xyz/docs/support-bot/feature-comparison/), [Ticket Tool custom forms](https://www.mava.app/blog/discord-ticket-bots-compared)

### ---- UPDATES ----

### 9. Release Announcements (table stake, LOW-MEDIUM complexity)

**What:** Post release notes, changelog, and version updates to an announcements channel. UPDT-01.

**Why it's table stake:** Announcements are a core Discord community feature. Without it, the info hub is incomplete.

**Implementation approach:** Slash command for manual posting + webhook endpoint for automated posting from CI/CD. Rich embed with version, highlights, download/links. Support for scheduled announcements.

**Complexity:** Low-Medium. 2-4 days.

**Dependencies:** None (but benefits from Role-Based Permissions for who can post).

### 10. Maintenance Notices (table stake, LOW complexity)

**What:** Scheduled/posted maintenance windows. UPDT-03.

**Why it's table stake:** Required for any SaaS product community. Sets professional expectations.

**Implementation approach:** Similar to announcements but with "downtime start/end" fields. Can share infrastructure with announcement system.

**Complexity:** Low. 1-2 days.

**Dependencies:** Release Announcements (shares infrastructure).

### ---- ADMINISTRATION ----

### 11. Channel Management (table stake, MEDIUM complexity)

**What:** Create, edit, delete channels via bot commands. ADMN-01.

**Why it's table stake:** Admin convenience feature. Present in Dyno, Carl-bot, YAGPDB.

**Implementation approach:** Discord.js `GuildChannelManager` methods. Create voice/text/category with permission overwrites. Must respect role hierarchy (can't edit channels above bot's role).

**Complexity:** Medium. 3-5 days (permission boundary checking adds complexity).

**Dependencies:** Role-Based Permissions.

### 12. Role Management (table stake, MEDIUM complexity)

**What:** Create, assign, remove roles via commands. ADMN-02.

**Why it's table stake:** Essential for permission systems. Present in every admin bot.

**Implementation approach:** Discord.js `GuildRoleManager`. Must check hierarchy (can't manage roles above bot's highest role). Bulk operations with rate-limit awareness.

**Complexity:** Medium. 3-5 days.

**Dependencies:** Role-Based Permissions, Channel Management (shares infrastructure).

### 13. Server Settings Configuration (table stake, MEDIUM complexity)

**What:** Per-guild configuration for all bot features. ADMN-03.

**Why it's table stake:** Without this, every feature default is hardcoded. Web dashboard or config commands are standard in 2026.

**Implementation approach:** Config CRUD commands (`/config get`, `/config set`, `/config reset`). Store in PostgreSQL per guild. Support for autocomplete on config keys. Web dashboard is a differentiator — for v1, slash commands are sufficient.

**Complexity:** Medium. 4-6 days (config schema design, validation, persistence).

**Dependencies:** Everything. Must be extensible.

**Source:** [Invite tracker config approach](https://github.com/rafalohaki/invite-tracker), [PeakBot all-in-one config](https://peakbot.pro/blog/best-discord-bot-2026-ranked)

### 14. Member Logging (table stake, MEDIUM complexity)

**What:** Log member joins, leaves, role changes, message edits/deletes. ADMN-04.

**Why it's table stake:** Carl-bot and Dyno are renowned for their logging. Audit trail is expected for security-conscious communities.

**Implementation approach:** `guildMemberAdd/Remove`, `guildMemberUpdate`, `messageUpdate/Delete` event listeners. Log to configurable channel. Support filter presets (minimal, standard, verbose).

**Complexity:** Medium. 4-6 days (many event types, rate-limit awareness for high-volume servers).

**Dependencies:** Server Settings (configurable log channel).

**Source:** [Carl-bot logging reputation](https://www.vibebot.gg/blog/best-discord-moderation-bots), [Dyno action logging](https://peakbot.pro/blog/best-discord-bot-2026-ranked)

---

## 🌟 Differentiators

Features that set Escluse apart. Not expected by general bot standards, but align with the project's specific value proposition: a developer-focused infrastructure community.

### 15. GitHub Issue & Release Integration (HIGH value, MEDIUM complexity)

**What:** Real-time GitHub events pushed to Discord — issues opened/closed, PRs merged, releases published, CI status. DEV-01.

**Why it's a differentiator:** Most bots only do webhook relay (GitHub → Discord via built-in webhooks). Escluse can go further: store repo-channel mappings, provide `/github` commands for on-demand status, allow per-repo event filtering, and integrate GitHub OAuth for contribution tracking.

**Implementation approach:** Express server to receive GitHub webhooks → HMAC-SHA256 verification → route to configured Discord channel with rich embed. Store repo-channel mappings in PostgreSQL. Support `/github repos list`, `/github setup <repo>`.

**Sub-features:**
- Push/commit notifications with diff stats
- PR open/merge/close notifications
- Issue open/close/reopen notifications
- Release published with changelog
- GitHub Actions workflow status
- "Good first issue" alerts for community contributors
- Per-repo event filtering (releases only, PRs only, etc.)

**Complexity:** Medium. 1-2 weeks.

**Dependencies:** Rust Axum API (for webhook endpoint), Role-Based Permissions.

**Source:** [repo-relay](https://github.com/blamechris/repo-relay), [patchy](https://github.com/aftab-s/patchy), [GitBot](https://github.com/TheCodingDad-TisonK/GitBot), [GitTrack](https://gittrack.me/)

### 16. Developer Portal / SDK Resources (HIGH value, MEDIUM complexity)

**What:** Centralized developer hub commands — SDK docs, API reference, contribution guide, quickstart templates. DEV-02.

**Why it's a differentiator:** No general-purpose bot offers this. It's specific to product ecosystems. This transforms the bot from "community manager" into "developer platform companion."

**Implementation approach:** Command group (`/escluse dev`) with subcommands: `docs`, `sdk`, `quickstart`, `contribute`, `api-status`. Pull content from documentation knowledge base. `/escluse dev quickstart` could even scaffold a starter project.

**Complexity:** Medium. 1-2 weeks (depends on amount of content).

**Dependencies:** Documentation Search (shares knowledge base), GitHub Integration (for `/dev quickstart` scaffolding).

### 17. Community Showcase System (HIGH value, MEDIUM-HIGH complexity)

**What:** Users submit their projects/servers via `/showcase submit`. Submissions get a modal (name, description, screenshots, tags) → posted to showcase channel with reactions/voting. SHOW-01, SHOW-02.

**Why it's a differentiator:** Dedicated showcase systems are rare in general bots. Poddy and jam-bot have this, but it's not common.

**Implementation approach:** `/showcase submit` opens modal → validates → posts embed to showcase channel with gallery fields. Support upvotes/starring. Optional integration with external showcase website. Moderation queue for approvals.

**Sub-features from research:**
- `/showcase-project` modal with name, description, GitHub URL, live URL, tags [jam-bot pattern]
- Embed posted to showcase channel
- Voting via buttons (upvote/downvote with one-vote-per-user enforcement)
- Weekly/monthly winner selection [Poddy pattern]
- Optional: POST to external showcase API for website syndication

**Complexity:** Medium-High. 2-3 weeks.

**Dependencies:** Role-Based Permissions (moderation queue), Channel Management (configurable showcase channel).

**Source:** [jam-bot showcase](https://github.com/wespreadjam/jam-discord-bot), [Poddy events system](https://runpod-poddy.mintlify.app/features/community), [RIT project-showcase-bot](https://github.com/RITct/project-showcase-bot)

### 18. Feedback & Polling System (MEDIUM value, MEDIUM complexity)

**What:** Structured polls, feature voting, community surveys with governance features. FDBK-01, FDBK-02.

**Why differentiator vs table stake:** Basic polls are built into Discord natively as of 2024. A custom system justifies itself only if it offers more:
- Anonymous voting (hide who voted for what)
- Governance guardrails (quorum, pass thresholds, role-restricted voting)
- Scheduled polls with auto-close
- Ranked-choice voting (RCV) for complex decisions
- Poll analytics (participation rates, trends over time)
- Integration with feature request tracking

**Approach:** Use Discord's native poll API for simple polls. Build custom system for advanced polls. Store in PostgreSQL.

**Complexity:** Medium. 1-2 weeks for advanced system.

**Dependencies:** None (can use native polls for v1).

**Source:** [Discord Poll API](https://docs.discord.com/developers/resources/poll), [pollaroid](https://github.com/RoiDayan1/discord-pollaroid), [gfh-bot governance features](https://github.com/dillon1000/gfh-bot)

### 19. Recognition & Contribution System (HIGH value, HIGH complexity)

**What:** Track contributions (referrals, community help, GitHub contributions), assign achievement roles, leaderboards. RECG-01, RECG-02, RECG-03.

**Why it's a differentiator:** While MEE6/Arcane have leveling/XP, Escluse's recognition is *contribution-based* not *chat-activity-based*. This aligns with a developer community where value comes from helping, building, and referring — not from chatting.

**Sub-features:**
- **Referral Tracking:** Personal invite links per member. Track joins, validate retention (7-day check), prevent abuse (account age, rejoin detection). Leaderboards (week/month/all-time). Configurable reward thresholds.
- **Contribution Recognition:** "Community Helper" role for ticket resolution participation. "Contributor" role for GitHub PRs. "Early Supporter" role for founding members.
- **Special Roles:** Auto-assign based on contribution thresholds. Manual grant by admins for special cases.
- **Leaderboards:** Top contributors, top referrers, top GitHub contributors. Weekly, monthly, all-time views.

**Complexity:** High. 3-4 weeks for full system.

**Dependencies:** GitHub Integration (for contribution tracking), Ticket System (for helper recognition), Role Management.

**Source:** [referral-bot](https://github.com/Katania91/discord-referral-bot), [invite-tracker](https://github.com/rafalohaki/invite-tracker), [achievements.bot](https://achievements.bot/), [discord-github-roles](https://github.com/KasperiP/discord-github-roles)

### 20. Roadmap Command (MEDIUM value, LOW complexity)

**What:** `/escluse roadmap` — display the product roadmap as a rich embed. INFO-03.

**Why differentiator:** Most bots don't know their parent product's roadmap. This ties the bot directly to the Escluse product.

**Implementation approach:** Read from a markdown file or API endpoint. Display as paginated embeds with status indicators (Planned, In Progress, Shipped).

**Complexity:** Low. 1-2 days.

**Dependencies:** Documentation/Knowledge Base.

### 21. Dev Milestone Notifications (MEDIUM value, LOW-MEDIUM complexity)

**What:** Post development milestone completions (e.g., "Beta 2 released", "v1.0 feature complete") to announcements. UPDT-02.

**Why differentiator:** Most bots don't have milestone tracking. Combined with GitHub integration, this becomes an automated changelog bridge.

**Implementation approach:** Manual posting command + optional webhook from GitHub milestones. Rich embed with milestone name, description, linked issues/PRs.

**Complexity:** Low-Medium. 2-4 days.

**Dependencies:** GitHub Integration (for automation), Release Announcements (shares infrastructure).

---

## ❌ Anti-Features

Features to explicitly NOT build. The project already lists moderation as v2 and AI assistant as long-term. Here are additional anti-features based on market analysis.

### 1. Economy / Currency System

**Why avoid:** Every general bot has it (MEE6, Arcane, Tatsu). It's the most feature-crept system in Discord bots — once you add coins, users demand shops, gambling, transfers, banks. It does not serve an infrastructure product community. It doesn't help users with Escluse.

**What to do instead:** Recognition badges and contribution points (non-tradeable, non-spendable). These serve the same engagement goal without the economy bloat.

**Source:** General consensus across bot analysis — economy systems are the #1 regretted feature by bot developers due to support burden and scope creep.

### 2. Music Playback

**Why avoid:** Requires voice connections, YouTube API (rate-limited and legally gray), audio streaming infrastructure. Separate bot category. Huge support burden. Does not align with Escluse's value prop.

**What to do instead:** Leave music to specialized bots (Pancake, FredBoat, etc.). Users can run multiple bots.

**Source:** [VibeBot blog](https://www.vibebot.gg/blog/best-discord-bots-2026) — "You might still want a dedicated music bot."

### 3. Games / Fun Commands

**Why avoid:** Trivia, slots, 8ball, etc. These drive general engagement but dilute the Escluse brand. A community bot for a developer infrastructure product should feel professional, not like a game server.

**What to do instead:** Community showcase voting and polls are the "fun" features. Keep it product-focused.

### 4. Chat Moderation (Warn, Mute, Kick, Ban, Purge)

**Why avoid:** Already scoped as v2 by the project. This is correct — moderation is a deep, high-liability feature. False positives, appeals, mod log, and custom automod rules are a full-time project.

**What to do instead:** Use Discord's native AutoMod (which is surprisingly good as of 2024+) for baseline. The project should ship with AutoMod enabled.

**Source:** [Discord AutoMod vs bots guide](https://www.discord.cab/discord-automod-vs-mee6-carl-bot-red-practical-scale-guide/) — "Start with native AutoMod for baseline. Add bot moderation only when you truly need it."

### 5. Intelligent AI Assistant (v2 / long-term vision)

**Why avoid now:** The project already lists this as long-term. Correct call. AI assistants require:
- LLM API costs
- Context management (what docs to retrieve, what the bot knows)
- Prompt injection hardening
- User trust (must be clear when AI vs human)

**What to do instead (v1):** Structured FAQ + documentation search commands. This gives users answers without the AI complexity. Add AI as a v2 differentiator.

### 6. Web Dashboard (v1.1 / v2)

**Why avoid in v1:** A web dashboard multiplies the surface area — auth (Discord OAuth), frontend framework decision, API design, deployment complexity. Mature bots like Dyno, Carl-bot, VibeBot, and MEE6 all invested heavily in dashboards. For v1, slash commands + config commands are sufficient.

**What to do instead:** Focus on excellent slash command UX. Save the dashboard for when the community reaches a scale where config commands become tedious.

### 7. External Platform Integrations (YouTube, Twitch, Reddit, Twitter/X)

**Why avoid:** These are table stakes for general *streamer/gaming* bots (MEE6, ProBot, Streamcord). Escluse is an infrastructure product community — these integrations don't serve the audience.

**What to do instead:** GitHub integration + documentation search. These are the integrations that matter for Escluse's audience.

---

## 🧩 Feature Dependency Map

```
v0 (Foundation - Phase 1)
├── Role-Based Permissions (PERM-01) ───── depends on ─── nothing (foundation)
├── Server Settings Config (ADMN-03) ───── depends on ─── nothing (foundation)

v1 Core (Phase 2)
├── Welcome Messages ────────────────────── depends on ─── Server Settings
├── Auto-Role ───────────────────────────── depends on ─── Welcome Messages (shares guildMemberAdd)
├── Interest/Role Selection ─────────────── depends on ─── Auto-Role (role infrastructure)
├── Member Logging ──────────────────────── depends on ─── Server Settings
├── FAQ + Docs Search ───────────────────── depends on ─── Role-Based Permissions

v1 Support (Phase 3)
├── Channel Management ──────────────────── depends on ─── Role-Based Permissions
├── Role Management ─────────────────────── depends on ─── Role-Based Permissions
├── Ticket System ───────────────────────── depends on ─── Channel Management, Role-Based Permissions
│   └── Bug Report Forms ───────────────── depends on ─── Ticket System

v1 Updates (Phase 4)
├── Release Announcements ───────────────── depends on ─── Role-Based Permissions
├── Maintenance Notices ─────────────────── depends on ─── Release Announcements (shares infra)
├── Dev Milestone Notifications ─────────── depends on ─── Release Announcements
├── Roadmap Command ─────────────────────── depends on ─── FAQ/Docs (shares content infra)

v2 Differentiators (Phase 5)
├── GitHub Integration ──────────────────── depends on ─── Rust API service, Role-Based Permissions
├── Developer Portal ────────────────────── depends on ─── FAQ/Docs, GitHub Integration
├── Feedback & Polls ────────────────────── depends on ─── Server Settings
├── Community Showcase ──────────────────── depends on ─── Channel Management, Role-Based Permissions
├── Recognition System ──────────────────── depends on ─── GitHub Integration, Role Management, Ticket System
│   ├── Referral Tracking ───────────────── depends on ─── Recognition System (shared DB)
│   ├── Contribution Recognition ────────── depends on ─── GitHub Integration + Ticket System
│   └── Special Roles ───────────────────── depends on ─── Role Management
```

---

## 🎯 MVP Recommendation

### Phase 1 (Foundation) — "Ship a Working Bot"
**Goal:** Get a bot in the server that handles the basics.

| Order | Feature | Complexity | Why First |
|-------|---------|------------|-----------|
| 1 | Role-Based Permissions | Medium | Everything depends on this |
| 2 | Server Settings Config | Medium | Every feature needs config |
| 3 | Welcome Messages | Low | Visible value, quick win |
| 4 | Auto-Role + Interest Selection | Medium | Core onboarding, drives engagement |

**Result by end of Phase 1:** New members get welcomed, choose interests, get appropriate roles.

### Phase 2 (Info Hub) — "Answer Questions Without the Founder"
| Order | Feature | Complexity | Why This Phase |
|-------|---------|------------|----------------|
| 5 | FAQ + Docs Search | Low-Medium | Reduces DMs to founder, 1st info hub feature |
| 6 | Roadmap Command | Low | Shows product vision, community-facing value |
| 7 | Release Announcements | Low-Medium | Establishes an announcements channel pattern |
| 8 | Maintenance Notices | Low | Professionalism signal |

### Phase 3 (Support) — "Handle Support at Scale"
| Order | Feature | Complexity | Why This Phase |
|-------|---------|------------|----------------|
| 9 | Channel Management | Medium | Needed for ticket channel creation |
| 10 | Role Management | Medium | Needed for ticket staff roles |
| 11 | Ticket System | High | Core support feature, biggest build |
| 12 | Bug Report Forms | Medium | Attached to ticket system |

### Phase 4 (Community) — "Build Community Engagement"
| Order | Feature | Complexity | Why This Phase |
|-------|---------|------------|----------------|
| 13 | Member Logging | Medium | Audit trail, trust |
| 14 | Feedback & Polls | Medium | Community voice, engagement |
| 15 | Community Showcase | Med-High | Let users show what they built |
| 16 | Recognition System | High | Drive growth, reward contributors |
| 17 | GitHub Integration | Medium | Bridge to dev workflow |
| 18 | Developer Portal | Medium | Developer hub, product docs |

---

## 📝 Additional Notes

### Complexity Level Definitions

| Level | Person-Days | Example Features | Risk Factors |
|-------|-------------|------------------|--------------|
| **Low** | 1-4 days | Welcome messages, FAQ, Roadmap command | None — well-worn patterns |
| **Medium** | 4-10 days | Role selection, GitHub integration, Channel management | Rate limits, edge cases, permission boundaries |
| **High** | 10-20+ days | Ticket system, Recognition system, Showcase system | State management, multi-user races, abuse prevention |

### Key Architectural Decisions Impacting Features

1. **Standalone vs. connected to Escluse backend (decision: standalone v1)**
   - Docs/FAQ data can be stored in PostgreSQL directly (markdown files imported at build or edit time)
   - GitHub webhooks go to the bot's Rust API, not the main Escluse API
   - Showcase submissions are Discord-internal for v1

2. **Database schema design matters early**
   - The config system (Phase 1) needs to be extensible enough to support ticket categories (Phase 3), poll configurations (Phase 4), showcase channel IDs (Phase 4+)
   - Use a polymorphic config pattern: `(guild_id, config_key, config_value)` or JSONB column

3. **Redis caching**
   - Role hierarchies (expensive to fetch from Discord API each time)
   - Permission resolution (check on every command execution)
   - Active ticket state (avoid querying PostgreSQL on every ticket interaction)

---

## 📚 Sources

### Bot Comparisons & Ecosystem Analysis
- [IONOS: Best Discord Bots 2026 Guide](https://www.ionos.com/digitalguide/online-marketing/social-media/discord-bots/) — HIGH confidence
- [VibeBot: Best Discord Bots 2026](https://www.vibebot.gg/blog/best-discord-bots-2026) — HIGH confidence
- [PeakBot: Best Discord Bot 2026 Ranking](https://peakbot.pro/blog/best-discord-bot-2026-ranked) — HIGH confidence
- [CommunityOne: Best Discord Bots 2026 Guide](https://blog.communityone.io/best-discord-bots/) — HIGH confidence
- [BuildMyDiscord: MEE6 Alternative Analysis](https://buildmydiscord.com/en/blog/mee6-alternative-ai-discord-bots) — MEDIUM confidence (vendor blog)

### Onboarding & Roles
- [Discord Community Onboarding FAQ](https://support.discord.com/hc/en-us/articles/11074987197975-Community-Onboarding-FAQ) — HIGH confidence (official Discord)
- [RoleLogic: Discord Role Automation Guide](https://rolelogic.faizo.net/docs/guides/common-scenarios) — MEDIUM confidence
- [VibeBot Welcome Features](https://www.vibebot.gg/features/welcome) — HIGH confidence
- [Memvers: Discord Server Setup Guide 2026](https://memvers.com/blog/discord-server-setup-guide-2026) — MEDIUM confidence

### Ticket Systems
- [Mava: Discord Ticket Bots Compared 2026](https://www.mava.app/blog/discord-ticket-bots-compared) — HIGH confidence
- [PeakBot: Best Discord Ticket Bots 2026](https://peakbot.pro/blog/best-discord-ticket-bots-2026) — HIGH confidence
- [TicketBot: 5 Features Every Ticket Bot Needs](https://ticketsbot.org/blog/best-discord-ticket-bot-features-2025) — MEDIUM confidence (vendor blog)
- [SCNX: Modmail vs Ticket System Comparison](https://docs.scnx.xyz/docs/support-bot/feature-comparison/) — HIGH confidence

### GitHub Integration
- [repo-relay: GitHub-Discord Integration Bot](https://github.com/blamechris/repo-relay) — HIGH confidence (open source code)
- [patchy: GitHub Webhook Discord Bot](https://github.com/aftab-s/patchy) — HIGH confidence (open source code)
- [GitBot: GitHub Events to Discord](https://github.com/TheCodingDad-TisonK/GitBot) — HIGH confidence (open source code)
- [GitTrack: GitHub Notifications](https://gittrack.me/) — MEDIUM confidence

### Polls & Feedback
- [Discord Poll API Documentation](https://docs.discord.com/developers/resources/poll) — HIGH confidence (official Discord)
- [pollaroid: Discord Poll Bot](https://github.com/RoiDayan1/discord-pollaroid) — HIGH confidence (open source code)
- [gfh-bot: Polling with Governance](https://github.com/dillon1000/gfh-bot) — HIGH confidence (open source code)

### Showcase & Submissions
- [jam-bot: Showcase Submissions](https://github.com/wespreadjam/jam-discord-bot) — HIGH confidence (open source code)
- [Poddy: Events & Voting System](https://runpod-poddy.mintlify.app/features/community) — MEDIUM confidence
- [RIT Project Showcase Bot](https://github.com/RITct/project-showcase-bot) — MEDIUM confidence

### Recognition & Referrals
- [discord-referral-bot](https://github.com/Katania91/discord-referral-bot) — HIGH confidence (open source code)
- [invite-tracker (Bun + TypeScript)](https://github.com/rafalohaki/invite-tracker) — HIGH confidence (open source code)
- [Achievements Bot](https://achievements.bot/) — MEDIUM confidence
- [discord-github-roles](https://github.com/KasperiP/discord-github-roles) — HIGH confidence (open source code)

### Moderation & Anti-Feature Guidance
- [Discord AutoMod vs MEE6/Carl-bot/Red Guide](https://www.discord.cab/discord-automod-vs-mee6-carl-bot-red-practical-scale-guide/) — HIGH confidence
- [VibeBot: Best Discord Moderation Bots 2026](https://www.vibebot.gg/blog/best-discord-moderation-bots) — HIGH confidence
