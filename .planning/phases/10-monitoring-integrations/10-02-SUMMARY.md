---
phase: 10-monitoring-integrations
plan: "02"
type: execute
wave: 1
autonomous: true
subsystem: notifications
tags: [backend, discord, webhooks]
dependency_graph:
  requires: []
  provides:
    - api: Discord webhook notifications on server events
    - app: Settings modal for webhook configuration
  affects:
    - api: discord_client.rs, webhook_service.rs, server_event_notifier.rs
    - app: ServerDetailsPage.jsx
tech_stack:
  added:
    - send_server_event method in DiscordClient
  patterns:
    - Rich Discord embeds for server lifecycle events
    - Event-driven notifications via EventBus
key_files:
  created:
    - api/src/application/services/server_event_notifier.rs
  modified:
    - api/src/infrastructure/external_services/discord_client.rs
    - api/src/application/services/webhook_service.rs
    - api/src/application/services/mod.rs
    - api/src/bootstrap/container.rs
    - app/src/pages/servers/ServerDetailsPage.jsx
decisions:
  - "Reuse existing DiscordClient infrastructure"
  - "Wire notifications via StatusChanged event in EventBus"
  - "Settings modal per-server for webhook URL"
metrics:
  duration: ~3 min
  completed_date: "2026-04-09"
  tasks: 4
  files: 6
---

# Phase 10 Plan 02: Discord Webhook Notifications

## Summary

Added per-server Discord webhook notifications for server lifecycle events. Users can configure a Discord webhook URL per server and receive rich embed notifications when their server starts, stops, or crashes.

## Implementation

### Task 1: Add Server Event Notification Methods to DiscordClient

- Extended `discord_client.rs` with `send_server_event` method
- Event types supported:
  - `server.started` → 🟢 green embed
  - `server.stopped` → 🔴 red embed
  - `server.crashed` → 💥 red embed (urgent)
  - `backup.complete` → 💾 blue embed
  - `restore.complete` → ♻️ green embed

### Task 2: Create ServerEventNotifier Service

- Created `server_event_notifier.rs` service
- Provides helper methods for each event type
- Checks for webhook URL before sending
- Handles errors gracefully (logs but doesn't fail)

### Task 3: Wire Event Notifications to Lifecycle Handlers

- Added StatusChanged event handling to `webhook_service.rs`
- When server status changes to running/stopped/crashed:
  - Look up server's discord_webhook_url
  - Send appropriate Discord notification
- Connected to existing EventBus infrastructure

### Task 4: Add Webhook URL Field to Server Settings UI

- Added Settings button to ServerDetailsPage
- Creates modal with Discord Webhook URL input
- On save: PATCH /servers/:id with discord_webhook_url
- Added useNavigate import for delete redirect

## Verification

- [x] DiscordClient has send_server_event method
- [x] ServerEventNotifier service created
- [x] StatusChanged events trigger notifications
- [x] Server settings modal saves webhook URL
- [x] Notifications use rich embed format

## Known Stubs

None - webhook URL persisted to server entity.

## Threat Flags

None - webhook URL is user-provided, notifications only sent on server events.

---

**Commit:** cc806b6

**Verified by:** Self-check passed
