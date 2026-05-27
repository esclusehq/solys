# Discord Bot API Documentation

Comprehensive guide to Discord Bot capabilities and implementation.

---

## Overview

Discord API allows developers to create bots that can interact with Discord servers programmatically. Bots can perform moderation, automation, gaming, and utility functions.

## Prerequisites

- Discord Account
- [Discord Developer Portal](https://discord.com/developers/applications) access
- Bot Token
- Server with Administrator permissions (for full capabilities)

---

## Capabilities by Category

### Messages

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Send Message | Kirim text/embed ke channel | `POST /channels/{id}/messages` |
| Edit Message | Edit pesan yang sudah dikirim | `PATCH /channels/{id}/messages/{id}` |
| Delete Message | Hapus pesan | `DELETE /channels/{id}/messages/{id}` |
| Send Embed | Kirim rich embed message | `POST /channels/{id}/messages` (with `embed`) |
| Reply to Message | Balas pesan dengan mention | `POST /channels/{id}/messages` (with `message_reference`) |
| Pin Message | Pin pesan ke channel | `PUT /channels/{id}/pins/{message_id}` |
| Add Reaction | Tambahkan emoji reaction | `PUT /channels/{id}/messages/{id}/reactions/{emoji}/@me` |
| Fetch Message History | Ambil pesan sebelumnya | `GET /channels/{id}/messages` |
| Bulk Delete | Hapus banyak pesan (server_perms needed) | `POST /channels/{id}/messages/bulk-delete` |

### Channels

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Text Channel | Buat channel text baru | `POST /guilds/{id}/channels` |
| Create Voice Channel | Buat channel voice baru | `POST /guilds/{id}/channels` |
| Create Category | Buat channel category | `POST /guilds/{id}/channels` |
| Edit Channel | Ubah channel settings | `PATCH /channels/{id}` |
| Delete Channel | Hapus channel | `DELETE /channels/{id}` |
| Move Channel | Pindahkan channel ke category lain | `PATCH /channels/{id}` (with `parent_id`) |
| Set Channel Permissions | Atur permission per channel | `PUT /channels/{id}/permissions/{id}` |
| Create Webhook | Buat webhook untuk automate posting | `POST /channels/{id}/webhooks` |

### Members & Users

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Get Member Info | Ambil info user di server | `GET /guilds/{id}/members/{id}` |
| List Members | Daftar semua member (limit 1000) | `GET /guilds/{id}/members` |
| Kick Member | Kick user dari server | `DELETE /guilds/{id}/members/{id}` |
| Ban Member | Ban user dari server | `PUT /guilds/{id}/bans/{id}` |
| Unban Member | Unban user | `DELETE /guilds/{id}/bans/{id}` |
| Timeout Member | Timeout user sementara | `PATCH /guilds/{id}/members/{id}` |
| Modify Member | Ubah nick, roles, dll | `PATCH /guilds/{id}/members/{id}` |
| Add Role | Berikan role ke member | `PUT /guilds/{id}/members/{id}/roles/{role_id}` |
| Remove Role | Hapus role dari member | `DELETE /guilds/{id}/members/{id}/roles/{role_id}` |
| Fetch User Info | Ambil info user (global) | `GET /users/{id}` |

### Roles

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Role | Buat role baru | `POST /guilds/{id}/roles` |
| Edit Role | Ubah role settings | `PATCH /guilds/{id}/roles/{id}` |
| Delete Role | Hapus role | `DELETE /guilds/{id}/roles/{id}` |
| Reorder Roles | Urutkan roles | `PATCH /guilds/{id}/roles` |
| Set Role Permissions | Atur permissions role | `PATCH /guilds/{id}/roles/{id}` (with `permissions`) |
| Set Role Color | Ubah warna role | `PATCH /guilds/{id}/roles/{id}` (with `color`) |

### Server (Guild)

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Get Server Info | Ambil info server | `GET /guilds/{id}` |
| Get Server Channels | Daftar semua channel | `GET /guilds/{id}/channels` |
| Get Server Roles | Daftar semua roles | `GET /guilds/{id}/roles` |
| Get Server Members | Jumlah member | `GET /guilds/{id}/member-count` |
| Edit Server Settings | Ubah server settings | `PATCH /guilds/{id}` |
| Get Server Emoji | Daftar emoji server | `GET /guilds/{id}/emojis` |
| Upload Emoji | Tambah emoji custom | `POST /guilds/{id}/emojis` |
| Delete Emoji | Hapus emoji | `DELETE /guilds/{id}/emojis/{id}` |

### Voice

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Move Member to Voice | Pindah user ke voice channel | `PATCH /guilds/{id}/members/{id}` (with `channel_id`) |
| Disconnect Member | Putuskan koneksi voice | `PATCH /guilds/{id}/members/{id}` (with `channel_id`: null) |
| Get Voice States | Cek siapa di voice | `GET /guilds/{id}/voice-states` |

### Interactions

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Slash Command | Buat command baru | `POST /applications/{id}/commands` |
| Handle Button Click | Respond button interaction | `POST /interactions/{id}/{token}` |
| Handle Modal Submit | Respond modal submission | `POST /interactions/{id}/{token}` |
| Create Message Component | Kirim button/select menu | `POST /channels/{id}/messages` (with `components`) |
| Defer Response | Tunda response (lebih dari 3 detik) | `POST /interactions/{id}/{token}/callback` |

### Webhooks

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Webhook | Buat webhook | `POST /channels/{id}/webhooks` |
| Send Webhook Message | Kirim pesan via webhook | `POST /webhooks/{id}/{token}` |
| Edit Webhook Message | Edit webhook message | `PATCH /webhooks/{id}/{token}/messages/{id}` |
| Delete Webhook Message | Hapus webhook message | `DELETE /webhooks/{id}/{token}/messages/{id}` |

### Threads

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Thread | Buat thread dari message | `POST /channels/{id}/messages/{id}/threads` |
| Create Public Thread | Buat thread publik | `POST /channels/{id}/threads` |
| Join Thread | Join ke thread | `PUT /channels/{id}/thread-members/@me` |
| Leave Thread | Leave thread | `DELETE /channels/{id}/thread-members/@me` |
| Archive Thread | Archive thread | `PATCH /channels/{id}` (with `archived`: true) |

### Scheduled Events

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Event | Buat scheduled event | `POST /guilds/{id}/scheduled-events` |
| Edit Event | Edit scheduled event | `PATCH /guilds/{id}/scheduled-events/{id}` |
| Delete Event | Hapus scheduled event | `DELETE /guilds/{id}/scheduled-events/{id}` |
| Get Event Subscribers | Daftar yang subscribe | `GET /guilds/{id}/scheduled-events/{id}/users` |

### Stage Channels

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Stage | Buat stage channel | `POST /guilds/{id}/channels` (type: 13) |
| Start Stage | Start stage (host) | `POST /guilds/{id}/stage-instances` |
| End Stage | End stage | `DELETE /guilds/{id}/stage-instances/{id}` |

### Invite Management

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Create Invite | Buat invite link | `POST /channels/{id}/invites` |
| Delete Invite | Hapus invite | `DELETE /invites/{code}` |
| Get Server Invites | Daftar semua invite | `GET /guilds/{id}/invites` |

### Stickers

| Capability | Description | Endpoint/Method |
|------------|-------------|-----------------|
| Get Server Stickers | Daftar sticker | `GET /guilds/{id}/stickers` |
| Upload Sticker | Tambah sticker | `POST /guilds/{id}/stickers` |
| Delete Sticker | Hapus sticker | `DELETE /guilds/{id}/stickers/{id}` |

---

## Common Use Cases

### Moderation Bot

| Feature | Implementation |
|---------|-----------------|
| Auto-moderation | Listen `on_message` → check content → delete/warn |
| Welcome messages | Event `on_member_join` → send DM/channel message |
| Logging | Listen events → log to audit channel |
| Auto-role | `on_member_join` → add role |
| Slowmode | `PATCH /channels/{id}` (with `rate_limit_per_user`) |

### Utility Bot

| Feature | Implementation |
|---------|-----------------|
| Level system | Track messages → store XP → level up |
| Music bot | Join voice → play audio from URL |
| Reminders | `setTimeout` / scheduler → send reminder |
| Polls | Create embed → add reactions → count |

### Game Bot

| Feature | Implementation |
|---------|-----------------|
| Game server status | Check server → post status to channel |
| Game matching | Queue system → match players |
| Tournament brackets | Create/delete channels → manage rounds |

---

## Implementation Examples

### Python (discord.py)

```python
import discord
from discord.ext import commands

intents = discord.Intents.default()
intents.message_content = True

bot = commands.Bot(command_prefix="!", intents=intents)

@bot.event
async def on_message(message):
    if message.content == "ping":
        await message.channel.send("pong")

bot.run("YOUR_TOKEN")
```

### Node.js (discord.js)

```javascript
const { Client, GatewayIntentBits, EmbedBuilder } = require('discord.js');

const client = new Client({
    intents: [GatewayIntentBits.Guilds, GatewayIntentBits.GuildMessages]
});

client.on('messageCreate', message => {
    if (message.content === 'ping') {
        message.reply('pong');
    }
});

client.login('YOUR_TOKEN');
```

### TypeScript (Recommended)

**Why TypeScript over plain Node.js:**

| Aspect | Node.js (JS) | TypeScript | Reason |
|--------|-------------|------------|--------|
| **Type Safety** | ❌ None | ✅ Full | Catch errors before runtime |
| **IntelliSense** | ⚠️ Terbatas | ✅ Excellent | Better autocomplete for Discord objects |
| **Debugging** | Sulit | Mudah | Clear error messages with line numbers |
| **Code Quality** | Error-prone | Reliable | Prevent common mistakes |
| **Refactoring** | Sulit | Mudah | Safe rename and changes |
| **Library** | Same | Same | Uses discord.js |

**TypeScript Example:**

```typescript
import { Client, GatewayIntentBits, Message, Guild, CommandInteraction } from 'discord.js';

const client = new Client({
    intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.MessageContent
    ]
});

client.on('messageCreate', async (message: Message) => {
    if (message.content === 'ping') {
        await message.reply('pong');
    }
});

client.on('interactionCreate', async (interaction: CommandInteraction) => {
    if (!interaction.isCommand()) return;

    const { commandName } = interaction;

    if (commandName === 'status') {
        await interaction.reply({
            content: 'Bot is running!',
            ephemeral: true
        });
    }
});

client.login(process.env.DISCORD_TOKEN);
```

**Project Setup:**

```bash
# Initialize project
npm init -y

# Install TypeScript and Discord.js
npm install discord.js
npm install -D typescript @types/node ts-node

# Initialize TypeScript
npx tsc --init
```

**tsconfig.json:**

```json
{
    "compilerOptions": {
        "target": "ES2020",
        "module": "commonjs",
        "strict": true,
        "esModuleInterop": true,
        "skipLibCheck": true,
        "outDir": "./dist"
    }
}
```

---

## Recommended Stack for Escluse

**Technology Stack: TypeScript + discord.js**

**Reasons to Choose TypeScript:**

| Reason | Explanation |
|--------|-------------|
| **Type Safety** | Discord objects are complex; TypeScript prevents mistakes with Message, Guild, User types |
| **Better DX** | IntelliSense shows all available methods and properties |
| **Maintainable** | Easier to refactor as project grows |
| **Industry Standard** | Most production Discord bots use TypeScript |
| **Consistency** | Matches your frontend stack (React/TypeScript) |

**Performance Comparison:**

| Metric | Node.js | TypeScript |
|--------|---------|------------|
| Startup Time | ~100ms | ~200ms |
| Runtime Performance | Same | Same |
| Development Speed | Faster | Slightly slower |
| Production Stability | Good | Excellent |

**When to Use What:**

| Use Case | Recommendation |
|----------|----------------|
| Learning/Beginner | Node.js (faster to start) |
| Production Bot | TypeScript |
| Quick Prototype | Node.js |
| Team Project | TypeScript |
| Escluse Community Bot | TypeScript |

---

## Permissions Required

| Permission | Capability |
|------------|------------|
| `MANAGE_MESSAGES` | Delete messages, manage channels |
| `MANAGE_ROLES` | Create/edit/delete roles |
| `KICK_MEMBERS` | Kick members |
| `BAN_MEMBERS` | Ban members |
| `MANAGE_CHANNELS` | Create/edit/delete channels |
| `MOVE_MEMBERS` | Move voice members |
| `MUTE_MEMBERS` | Mute in voice |
| `DEAFEN_MEMBERS` | Deafen in voice |
| `ADMINISTRATOR` | Full access |

---

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| Send Message | 5 req/5 sec (global), 2 req/sec (channel) |
| Create Channel | 2 req/10 min |
| Kick/Ban | 10 req/10 sec |
| General API | 50 req/sec (with bot) |

---

## Additional Resources

- [Discord Developer Portal](https://discord.com/developers/applications)
- [Discord.js Documentation](https://discord.js.guide)
- [discord.py Documentation](https://discordpy.readthedocs.io)
- [Discord API Documentation](https://discord.com/developers/docs)

---

*Last updated: 2026-05-18*
*Part of Escluse Developer Documentation*