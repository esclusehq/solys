# Phase 10: Monitoring & Integrations - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Resource visualization and third-party notifications. This phase establishes historical resource graphs, Discord notifications, cron task scheduling, and notification events.

**Success criteria:**
1. User can view historical resource graphs (CPU, RAM, network)
2. User can configure Discord webhook notifications
3. User can schedule cron tasks for server automation
4. Notifications sent for start/stop/crash events
</domain>

<decisions>
## Implementation Decisions

### Historical Resource Graphs (D-36)
- **D-36:** Recharts line charts
- Use Recharts library in React frontend
- Store 24h of data points in server_metrics table
- Reference: Frontend chart components

### Discord Notifications (D-37)
- **D-37:** Per-server webhooks
- Store Discord webhook URL per server (discord_webhook_url field)
- Use DiscordClient for sending notifications
- Embed format for rich messages
- Reference: `api/src/infrastructure/external_services/discord_client.rs`

### Cron Task Scheduling (D-38)
- **D-38:** UI-based scheduling
- UI time picker for scheduling
- Tasks stored in database, scheduler service executes
- Reference: Server settings UI

### Notification Events (D-39)
- **D-39:** Event-based notifications
- Events: server start, server stop, server crash, backup complete, restore complete
- Sent when discord_webhook_url is configured
- Reference: `api/src/domain/webhook/service.rs`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Discord
- `api/src/infrastructure/external_services/discord_client.rs` — Discord client
- `api/src/application/services/webhook_service.rs` — Webhook service
- Server model: discord_webhook_url field

### Metrics
- `api/src/domain/entities/server_metrics.rs` — Metrics entity
- `api/src/infrastructure/repositories/postgres_metrics_repository.rs` — Metrics repository

### Webhooks
- `api/src/domain/webhook/service.rs` — Webhook delivery
- `api/src/presentation/handlers/webhook_handlers.rs` — Webhook CRUD

### Frontend
- `app/src/pages/servers/ServerDetailsPage.jsx` — Metrics display

</canonical_refs>

<specifics>
## Specific Ideas

- discord_webhook_url field already exists on Server
- DiscordClient exists in infrastructure
- WebhookService handles delivery with retry
- Server metrics stored with 24h retention (from Phase 7)

</specifics>

<deferred>
## Deferred Ideas

None — all decisions captured.

</deferred>

---

*Phase: 10-monitoring-integrations*
*Context gathered: 2026-04-09*
