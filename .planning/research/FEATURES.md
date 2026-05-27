# Feature Landscape

**Domain:** Game Server Hosting Platform
**Researched:** 2026-04-09

## Executive Summary

Game server hosting platforms enable users to deploy, manage, and monitor game servers on cloud infrastructure. Research across leading platforms (Pterodactyl, AMP, Multicraft, TCAdmin, Nodecraft, Gameye) reveals a clear feature taxonomy: table stakes are server lifecycle management and basic monitoring; differentiators center on automation, integrations, and operational tooling; anti-features are scope-conscious decisions to avoid bloat in early phases.

## Table Stakes

Features users expect. Missing = product feels broken or incomplete. These are non-negotiable for any viable offering.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Server Deployment** | Core value proposition - users must be able to create a game server | High | Requires game egg definitions, Docker image management, port allocation, resource provisioning |
| **Start/Stop/Restart** | Basic server lifecycle control | Low | Simple process control, state transitions |
| **Server Status** | Users need to know if their server is online | Low | Node agent polling, basic health checks |
| **RCON Console** | Standard admin interface for game servers | Medium | Protocol support (Source RCON, Minecraft RCON), authentication |
| **File Manager** | Server configuration and mod management | High | SFTP/FTP, browser-based editor, file system isolation |
| **Resource Allocation** | CPU, RAM, storage limits per server | Medium | Container limits, quota enforcement |
| **Network Configuration** | Port allocation, IP assignment | Medium | Auto-allocation, custom port mapping |

**Key insight:** Pterodactyl (open-source, 200+ game eggs) and AMP (CubeCoders, commercial) define table stakes. Users switching from these expect parity on core functionality.

## Differentiators

Features that set products apart. Not expected universally, but highly valued and create competitive moats.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **One-Click Mod Manager** | Reduce friction for popular games (Minecraft, Palworld, 7D2D) | High | Registry of modpacks, dependency resolution, Steam workshop integration |
| **Automated Backups** | Data safety without manual intervention | Medium | Scheduled snapshots, retention policies, one-click restore |
| **Real-Time Console** | In-browser command execution with streaming output | Medium | WebSocket-based, ANSI color support |
| **Discord/Telegram Integration** | Notifications for events (start, stop, crash, backups) | Low | Webhook-based, event subscription |
| **Scheduled Tasks** | Automation for restarts, backups, commands | Medium | Cron-based, game-specific templates |
| **Crash Recovery** | Auto-restart on crash without user intervention | Low | Process monitoring, failure detection |
| **Resource Graphs** | Historical CPU, RAM, network usage | Medium | Time-series data, retention |
| **Multi-User Collaboration** | Team access with role-based permissions | Medium | Sub-users, role definitions |
| **DDoS Protection** | Mitigation layer (often third-party) | High | Partnership required, cost overhead |
| **Player Analytics** | Session data, peak times, player list | Medium | Game-specific protocol support |
| **Template System** | Reusable server configurations | Low | Preset configs for game variants |

**Key insight:** Nodecraft and commercial hosts differentiate heavily on automation (scheduled tasks, backups, crash recovery) and integrations (Discord, Telegram). Self-hosted panels (Pterodactyl) lag here without plugins.

## Anti-Features

Features to explicitly NOT build. Deliberate scope decisions based on PROJECT.md constraints.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Custom Billing Integration** | Use existing Stripe integration per PROJECT.md | Leverage Stripe for all billing |
| **Multi-Cloud Provider Support** | Scope complexity, single provider focus first | Build one provider well first |
| **White-Label/Reseller** | Adds multi-tenancy complexity early | Focus on direct end-users |
| **Built-In Game Servers** | Requires licensing, maintain images | Use Docker images, user-provided |
| **Plugin Marketplace** | Curation burden, liability concerns | Document manual install process |
| **Server Monitoring Agents** | Premature for early phases | Use existing node agent |
| **Mobile App** | DX overhead, low priority | Responsive web first |

## Feature Dependencies

```
Server Deployment
├── Game Egg Definition (req)
├── Docker Image (req)
├── Port Allocation (req)
├── Resource Quota (req)
└── Node Selection (req)

File Manager
├── SFTP Service (req)
├── File System Isolation (req)
└── Permission Model (req)

RCON Console
├── RCON Protocol Support (req)
├── Authentication (req)
└── WebSocket/Streaming (opt) → Real-Time Console

Automated Backups
├── Snapshot Infrastructure (req)
├── Retention Policy (req)
└── Restore UI (req)

Mod Manager
├── Game-Specific Registry (req)
├── Dependency Resolution (opt)
└── Steam Workshop API (opt for games that support)
```

## MVP Recommendation

Prioritize in this order:

### Phase 1 (MVP - align with Active requirements in PROJECT.md)
1. **Server Deployment** - Core value, enables everything else
2. **Start/Stop/Restart** - Table stakes, simple to implement
3. **Server Status** - Required for basic visibility
4. **RCON** - Admin access, per PROJECT.md Active requirement
5. **File Manager** - SFTP access, per PROJECT.md Active requirement

### Phase 2 (Post-MVP)
1. **Automated Backups** - High value, moderate complexity
2. **Real-Time Console** - Differentiator, builds on RCON infrastructure
3. **Crash Recovery** - Automation baseline

### Phase 3 (Enhancement)
1. **Scheduled Tasks** - Automation suite
2. **Discord Integration** - Community expectation
3. **Resource Graphs** - Operational visibility
4. **Mod Manager** - Game-specific value

## Sources

- Pterodactyl Panel Features (mintlify.com/pterodactyl) - HIGH confidence
- Nodecraft Automation Features (nodecraft.com/features/automation) - HIGH confidence
- AMP Game Server Control Panel (cubecoders.com/AMP) - HIGH confidence
- Self-Hosted Game Server Panels 2026 (selfhosting.sh) - MEDIUM confidence
- Gameye vs GameFabric comparison (gameye.com) - MEDIUM confidence

---

**Confidence Assessment:**
- Table stakes: HIGH (validated against multiple established platforms)
- Differentiators: MEDIUM (pattern consistent across commercial hosts, specific implementations vary)
- Anti-features: HIGH (direct alignment with PROJECT.md constraints)
- Dependencies: MEDIUM (logical flow, implementation may reveal edge cases)