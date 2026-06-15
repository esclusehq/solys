### Phase 08: Operations Integration

**Goal:** RCON console and SFTP file management for server administration — establish the console interface, file browser, transfer behavior, and security model
**Requirements**: RCON-01, RCON-02, FILE-01, FILE-02, FILE-03
**Depends on:** Phase 07
**Plans:** 3/3 plans complete

**Wave 1**
- [x] 08-01-PLAN.md — Console page: add /console route + xterm.js Terminal integration with server selector
- [x] 08-02-PLAN.md — RCON API client + file route and path security verification
- [x] 08-03-PLAN.md — SFTP wiring: agent dispatch + API endpoints + frontend FileManager integration

### Phase 47: Docs Website

**Goal:** Create a documentation website for Esluce in the docs/ folder
**Requirements**: None
**Depends on:** Phase 46
**Plans:** 0 plans

Plans:
- [x] 47-01-PLAN.md — Docs website with VitePress (complete 2026-05-10)
### Phase 48: Add About Us, Legal, and Contact pages to landing page

**Goal:** Add About Us, Legal (Terms, Privacy), and Contact pages to the landing page
**Requirements**: None
**Depends on:** Phase 47
**Plans:** 1 plan

Plans:
- [x] 48-01-PLAN.md — Add About Us, Legal, and Contact pages (complete 2026-05-15)

### Phase 49: Fix login functionality in landing page

**Goal:** Configure Supabase OAuth and backend API authentication for the landing page login/signup flows
**Requirements**: REQ-49-001 to REQ-49-006
**Depends on:** Phase 48
**Plans:** 4 plans

Plans:
- [x] 49-01-PLAN.md — Configure Supabase credentials and API URL (complete)
- [x] 49-02-PLAN.md — Configure OAuth providers in Supabase (complete)
- [x] 49-03-PLAN.md — Test end-to-end authentication flow (complete)
- [x] 49-04-PLAN.md — Redirect to landing page after login (complete 2026-05-15)

### Phase 50: Automasi build binary untuk agent/solys

**Goal:** Build pipeline otomatis: Push ke GitHub → GitHub Actions → Build binaries → Upload ke R2 → Cloudflare CDN → get.esluce.com → Users install/update
**Requirements**: None
**Depends on:** Phase 49
**Plans:** 2/2 plans complete

Plans:
- [x] 50-01-PLAN.md — CI/CD core workflows (release.yml, canary.yml, ci.yml) + packaging infrastructure
- [x] 50-02-PLAN.md — Installer scripts (install.sh, install.ps1)

### Phase 51: Automasi DNS berbasis Cloudflare API

**Goal:** Agent dapat menghubungkan domain ke IP client secara otomatis via Cloudflare API agar Minecraft server bisa online ke public
**Requirements**: None
**Depends on:** Phase 50
**Plans:** 3/3 plans complete

Plans:
- [x] 51-01-PLAN.md — Backend Foundation: DB seed, CloudflareConfig entity, settings API, WebSocket protocol extension
- [x] 51-02-PLAN.md — Agent DNS Implementation: Cloudflare DNS handler, IP watcher, auto-refresh (DDNS-like)
- [x] 51-03-PLAN.md — Frontend UI: Cloudflare settings tab with API token config, auto-refresh toggle, test connection

### Phase 52: Improve API Documentation

**Goal:** Enhance API docs di https://docs.esluce.com/api/overview dengan detailed descriptions, request/response examples, auth guide, rate limiting, error codes, dan SDK guides untuk Node.js dan Python
**Requirements**: None
**Depends on:** Phase 47 (Docs Website)
**Plans:** 8/8 plans complete

Plans:
**Wave 1**
- [x] 52-01-PLAN.md — VitePress Infrastructure: data loader, Vue components, theme registration, CSS
- [x] 52-02-PLAN.md — Sidebar Configuration: full API Reference navigation tree

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 52-03-PLAN.md — Core Docs: overview restructure, auth guide, error catalog, changelog
- [x] 52-04-PLAN.md — Servers Group: CRUD, operations, console, properties, cron tasks
- [x] 52-05-PLAN.md — Servers Extended: files, backups, plugins, git, build, deploy, profiling
- [x] 52-06-PLAN.md — Nodes + Billing: all node endpoints + billing with sub-pages
- [x] 52-07-PLAN.md — Remaining Resources: webhooks, alerts, settings, templates, agents, jobs, usage, runtimes, deploy
- [x] 52-08-PLAN.md — SDK Guides: Node.js + Python quickstarts

**Cross-cutting constraints:**
- D-05: Field-level schema tables on every page
- D-06: curl + Node.js SDK + Python SDK examples per endpoint
- D-04: HTTP interface only, no proprietary implementation

### Phase 53: User Profile Management

**Goal:** Halaman profile untuk users setelah OAuth login - view/update profile info (email, avatar, name), update display name, change password (email accounts), view login history, delete account option
**Requirements**: None
**Depends on:** Phase 49 (Fix login functionality)
**Plans:** 6/6 plans complete

**Wave 1**
- [x] 53-01-PLAN.md — Database migrations + Supabase Storage bucket setup

**Wave 2**
- [x] 53-02-PLAN.md — Extend User model/repository with new profile fields

**Wave 3**
- [x] 53-03-PLAN.md — Backend handlers + routes for profile, login history, account deletion

**Wave 4**
- [x] 53-04-PLAN.md — Frontend infra: authStore extensions + useProfile hook
- [x] 53-06-PLAN.md — Deletion cleanup cron job background service

**Wave 5**
- [x] 53-05-PLAN.md — Frontend profile tab components + sidebar user info

### Phase 54: Email Verification Flow

**Goal:** Verifikasi email untuk users yang signup dengan email - send verification email on registration, resend verification option, require verified email for sensitive actions
**Requirements**: None
**Depends on:** Phase 49 (Fix login functionality)
**Plans:** 6/6 plans complete

**Wave 1**
- [x] 54-01-PLAN.md — Backend Foundation: migration, model, VerifiedUser extractor, resend + OAuth auto-verify + email change endpoint
- [x] 54-02-PLAN.md — Frontend Auth Infrastructure: authStore + API client extensions

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 54-03-PLAN.md — Frontend Verification Components: VerifyEmailPage fix, Banner (D-07 fix: button enabled during cooldown, toast on click), Dialog
- [x] 54-05-PLAN.md — Backend Gating: VerifiedUser on billing, subscription, webhook, and server handlers (4 files)

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 54-04-PLAN.md — Frontend Integration: Banner in App.jsx layout, VerifiedRoute wrapper, email change form in SettingsPage

**Wave 4** *(blocked on Wave 1 completion, no code changes)*
- [x] 54-06-PLAN.md — Deferred Gating Strategy: Document for D-08 categories without existing handlers (Identity & Access, Integration Extensions)

### Phase 55: Scheduled Backups

**Goal:** Backup otomatis untuk data server secara berkala dan terjadwal - configurable backup intervals, retention policies, backup storage location
**Requirements**: None
**Depends on:** Phase 51 (Automasi DNS)
**Plans:** 4/4 plans complete

Plans:
- [x] 55-01-PLAN.md — Worker cron evaluation + backup dispatch via Redis queue
- [x] 55-02-PLAN.md — Agent backup execution with agent-backup crate (compress + upload)
- [x] 55-03-PLAN.md — API backup config CRUD + S3 profiles + migrations
- [x] 55-04-PLAN.md — Frontend backup config panel + S3 profile management UI

### Phase 56: Auto Online & Sleep Recovery

**Goal:** Server dapat kembali aktif otomatis setelah offline atau sleep - auto restart on crash, sleep mode detection, automatic wake-up mechanisms
**Requirements**: None
**Depends on:** Phase 51 (Automasi DNS)
**Plans:** 3/4 plans executed

**Wave 1**
- [x] 56-01-PLAN.md — DB Migration + Domain Entities (add auto_wake, sleep_timeout_minutes, restart backoff fields)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 56-02-PLAN.md — DTOs + Use Cases + Sleep/Wake API Endpoints
- [x] 56-03-PLAN.md — Monitoring Service: Sleep Detection + Auto-Restart Backoff

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 56-04-PLAN.md — Frontend UI: Status Badge, Action Button, Sleep Config Panel

### Phase 57: Auto Restart Policies

**Goal:** Restart server secara otomatis saat crash atau tidak merespons - crash detection, configurable restart rules, max restart attempts, cooldown periods
**Requirements**: None
**Depends on:** Phase 56 (Auto Online & Sleep Recovery)
**Plans:** 4 plans

**Wave 1**
- [ ] 57-01-PLAN.md — Data Layer: migration + entity fields + both repositories

**Wave 2** *(blocked on Wave 1 completion)*
- [ ] 57-02-PLAN.md — Backend API: DTOs, use cases, handlers, global defaults storage
- [ ] 57-03-PLAN.md — MonitoringService: RCON health check, unresponsive detection, events

**Wave 3** *(blocked on Wave 2 completion)*
- [ ] 57-04-PLAN.md — Frontend UI: Restart Policy section + global defaults settings tab

### Phase 58: Server, Plugin, and Modpack Templates

**Goal:** Templates untuk server, plugin, dan modpack untuk mempermudah deployment dan konfigurasi server secara instan - pre-configured game templates, plugin bundles, modpack configurations
**Requirements**: None
**Depends on:** Phase 51 (Automasi DNS)
**Plans:** 5/5 plans complete

**Wave 1**
- [x] 58-01-PLAN.md — Data Layer: migration, entity, repository (D-04/D-12 foundation)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 58-02-PLAN.md — Backend API: DTOs, use cases, handlers, routes, container wiring, CreateServer integration

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 58-03-PLAN.md — External Services: CurseForgeClient, Modrinth/CurseForge API key settings
- [x] 58-04-PLAN.md — Frontend Infrastructure: API client, hooks, components

**Wave 4** *(blocked on Wave 3 completion)*
- [x] 58-05-PLAN.md — Frontend Pages: template library, create form, mod browser, App.jsx routes + sidebar

### Phase 59: Server Scheduling

**Goal:** Mengatur start, stop, restart, dan sleep server secara otomatis berdasarkan jadwal - scheduled start/stop, recurring schedules, timezone support
**Requirements**: None
**Depends on:** Phase 57 (Auto Restart Policies)
**Plans:** 3/3 plans complete

**Wave 1**
- [x] 59-01-PLAN.md — Data Layer: migration (4 new columns), entity + DTO extension, repository updates, handler validation updates
**Wave 2** *(blocked on Wave 1)*
- [x] 59-02-PLAN.md — Worker: cron_eval extension (all task types + timezone), job handlers (start/stop/restart/sleep with D-05/D-06/D-07/D-08), API dispatch endpoint
- [x] 59-03-PLAN.md — Frontend: useScheduledActions hook, API client methods, Scheduled Actions section in ServerDetails Settings tab

### Phase 60: Crash Detection

**Goal:** Mendeteksi server yang berhenti atau crash secara otomatis dan menjalankan recovery.

**Depends on:** Phase 57 (Auto Restart Policies)
**Plans:** 4/4 plans complete

**Wave 1**
- [x] 60-01-PLAN.md — Data Layer: migration (server_crash_logs table), ServerCrashLog entity, PostgresCrashLogRepository
- [x] 60-02-PLAN.md — Agent Protocol + Crash Reporter: CrashReport WS message, agent crash data capture via Bollard events

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 60-03-PLAN.md — Backend Crash Processing: crash_classifier.rs, MonitoringService crash ingestion/recovery/notifications, WS CrashReport handler, REST API endpoints

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 60-04-PLAN.md — Frontend: useCrashLogs hook, API client functions, Crash History section in ServerDetails Settings tab

### Phase 61: Create DEVELOPMENT.md - Setup local dev environment

**Goal:** Create a DEVELOPMENT.md entry point and docs/dev/ sub-files for developers to set up the Esluce project locally, covering prerequisites, clone, Docker infra, local Supabase, .env config, per-service run commands, end-to-end workflow, and troubleshooting.
**Requirements**: None
**Depends on:** Phase 60
**Plans:** 1/1 plans complete

Plans:
- [x] 61-01-PLAN.md — Create root DEVELOPMENT.md + docs/dev/* sub-files (01-prerequisites, 02-setup, 03-configuration, 04-commands, 05-troubleshooting)

### Phase 62: Create CONTRIBUTING.md - Cara kontribusi

**Goal:** Create CONTRIBUTING.md guide, CODE_OF_CONDUCT.md, and PULL_REQUEST_TEMPLATE.md for developers who want to contribute to the Esluce project
**Requirements**: None
**Depends on:** Phase 61
**Plans:** 1 plan

Plans:
- [x] 62-01-PLAN.md — Create CONTRIBUTING.md + CODE_OF_CONDUCT.md + PULL_REQUEST_TEMPLATE.md (complete 2026-05-31)

### Phase 63: Create ARCHITECTURE.md - technical documentation (module-level)

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 62
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 63 to break down)

### Phase 64: Create database schema documentation (for developers who want to extend)

**Goal:** Deliver DATABASE_SCHEMA.md at repo root documenting all PostgreSQL tables, columns, relationships, constraints, and indexes by business domain, generated by a Rust CLI tool (tools/db-schema-gen/) that introspects the live database via information_schema and reads rustdoc annotations from entity structs.
**Requirements**: None
**Depends on:** Phase 63
**Plans:** 3 plans

**Wave 1**
- [ ] 64-01-PLAN.md — Build db-schema-gen Rust CLI generator tool (8 source files)
- [ ] 64-02-PLAN.md — Add rustdoc annotations to all entity structs (27 files)

**Wave 2** *(blocked on Wave 1 completion)*
- [ ] 64-03-PLAN.md — Build generator, run against live DB, commit DATABASE_SCHEMA.md

### Phase 65: Buat installer script auto-install Docker sebelum install Solys agent

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 64
**Plans:** 1/1 plans complete

Plans:
- [x] 65-01-PLAN.md — Add container runtime auto-install to Solys installer (created 2026-05-31)

### Phase 66: integrasikan umami analitycs dashboard dengan RDS di project escluse

**Goal:** Deploy self-hosted Umami Analytics on EC2 + RDS at analytics.esluce.com to track all esluce.com subdomains
**Requirements**: TBD
**Depends on:** Phase 65
**Plans:** 1/2 plans executed

**Wave 1**
- [x] 66-01-PLAN.md — Create Umami Docker stack (docker-compose.yml, Caddyfile, .env.example) + DEPLOYMENT.md
- [ ] 66-02-PLAN.md — Inject Umami tracking scripts into landing-page-escluse/index.html and app/index.html

### Phase 67: Agent auto-resolve Minecraft port reachability issues (CGN/firewall/Docker port exposure)

**Goal:** Make the Esluce agent and backend automatically detect and resolve Minecraft game port reachability issues at the agent node via hybrid (backend-probe + agent-diagnostics) detection, 4-mode classification (PORT_NOT_BOUND, HOST_FIREWALL_BLOCKED, CGNAT_DETECTED, UPnP_UNAVAILABLE), and safe-to-fix auto-remediation with per-server audit log.
**Requirements**: DEPLOY-01..05, RCON-01..02 (lifecycle hooks + audit reuse)
**Depends on:** Phase 66
**Plans:** 3/3 plans complete

**Wave 1** *(foundational — schema + persistence)*:
- [x] 67-01: Schema migrations (servers columns + connectivity_audit_log table) + entity + repository + container wiring + [BLOCKING] sqlx migrate run

**Wave 2** *(agent + backend in parallel, blocked on Wave 1 completion)*:
- [x] 67-02: Agent-side connectivity (Cargo deps, diagnostics/firewall/upnp handlers, ConnectivityMonitor background task, IP-change hook in DnsWatcher)
- [x] 67-03: Backend-side connectivity (WS NodeMessage extension, ConnectivityService probe/classify/dispatch, REST handlers, routes mount, container wiring)

**Cross-cutting constraints:**
- Hybrid probe model: backend probes from public internet, agent sends raw local diagnostics (D-01)
- Auto-fix policy: safe-to-fix gate only — never modify user firewall rules or arbitrary root commands (D-05)
- Audit log: every auto-fix action logged with exact commands + timestamps, append-only `connectivity_audit_log` table (D-17)
- Probe triggers: server.start (~5-10s delay) + IP/firewall change events + 5-min periodic fallback (D-02)
- Per-server connectivity columns on `servers`: `connectivity_status`, `connectivity_mode`, `last_probe_at`
- Manual "Reachable" button: POST `/api/v1/servers/:id/connectivity/probe` with 30s per-server cooldown in Redis

### Phase 68: Escluse Relay Infrastructure

Objective:
Implement Esluce Relay as the primary connectivity path for Minecraft servers, with relay-backed stable DNS on *.play.esluce.net and conditional Direct Mode fast-path on *.play.esluce.com.

Architecture:

Player
↓
[server-name].play.esluce.net   ← always-on, stable (relay)
[server-name].play.esluce.com   ← conditional, best-effort (direct)
↓
Esluce Relay Gateway (relay.esluce.net)   ← primary path
OR
User's public IP (port-forwarded)         ← fast-path when probe-verified
↓
Persistent Tunnel (relay) / Direct TCP (direct)
↓
Esluce Agent
↓
Minecraft Server

Requirements:

1. Relay Gateway Service

* Deploy relay.esluce.net on AWS.
* Accept persistent outbound tunnel connections from agents.
* Maintain active tunnel registry indexed by server_id.
* Support multiple concurrent tunnels.
* Detect stale/disconnected tunnels via heartbeat timeout (30s default).
* Handle Minecraft Java TCP traffic.
* Reject connection requests to server_id with no active tunnel.

2. Agent Tunnel Client

* Agent establishes outbound encrypted tunnel to relay.esluce.net.
* Automatic reconnect with exponential backoff on disconnect.
* Heartbeat every 10s; tunnel marked stale after 3 missed heartbeats.
* Register on connect:
  * server_id
  * node_id
  * agent_version
  * minecraft_port
* Report tunnel health (latency, uptime, bytes transferred) periodically.

3. Routing Layer

* Player connection to <server>.play.esluce.net → relay gateway → lookup active tunnel by server_id → forward TCP.
* Reject (close socket) when no active tunnel exists for requested server_id.
* Support multiple servers simultaneously.
* Connection-level isolation: per-tunnel stream multiplexing.

4. DNS Integration (relay-first)

* Relay endpoint (always-on):
  * `<server>.play.esluce.net` → CNAME/ALIAS → relay.esluce.net → dynamic resolution to active tunnel.
  * Endpoint remains stable across agent restarts, IP changes, ISP changes.
  * DNS zone `esluce.net` delegated to relay infrastructure (e.g., Route 53).
* Direct Mode endpoint (conditional, best-effort):
  * `<server>.play.esluce.com` → A record → user's public IP, only emitted when Direct Mode is probe-verified working.
  * Removed when mode flips to Relay or Direct probe fails.
  * Updated by agent via Cloudflare API on every IP change (existing behavior).
  * DNS zone `esluce.com` stays on Cloudflare, contains only valid A records.
* Agent owns DNS lifecycle for both zones based on current mode and probe results.
* Backward compat: existing `<server>.play.esluce.com` wildcard records for servers predating Phase 68 continue to work.

5. Connectivity Mode Selection (relay-default)

* Relay Mode is the **default and primary** path — works for CGNAT, NAT, port-forward-failed, and non-CGNAT alike.
* Direct Mode is a **fast-path optimization** for non-CGNAT users with working port forwarding.
* Mode flip logic:
  * On agent start: probe Direct Mode reachability (external UDP/TCP probe to A record).
  * Direct probe success + <50ms latency penalty vs relay → emit `<server>.play.esluce.com`, mode = Direct.
  * Direct probe failure OR user is CGNAT-detected → skip `<server>.play.esluce.com`, mode = Relay.
  * Periodic re-probe (every 5 min); flip mode if conditions change.
* Automatic fallback without player-side software. Player can use either address; agent picks the best one.

6. Dashboard Integration

Add Connectivity section.

Connection Mode: Direct / Relay / Offline

Relay Status: Connected / Connecting / Disconnected

Public Addresses (both shown when applicable):
* `<server>.play.esluce.net` — always-on, relay-backed
* `<server>.play.esluce.com` — shown only when Direct Mode is active

Tunnel Health:
* Latency (relay round-trip)
* Last Heartbeat
* Connection Duration
* Mode (Direct / Relay / Offline)
* Direct Probe Status (Pass / Fail / Skipped)

7. Security

* Authenticate tunnel sessions via per-agent token issued at agent registration.
* Validate server_id ownership against backend before tunnel accepted.
* Prevent unauthorized tunnel registration (replay protection, nonce-based handshake).
* TLS 1.3+ for all relay-agent communication.
* Rate limit connection attempts per source IP (100/min default).
* Rate limit tunnel registration per server_id (1 active tunnel max, replacement on reconnect).

8. Monitoring

* Active tunnels
* Relay bandwidth (in/out Mbps)
* Concurrent players per tunnel
* Tunnel disconnects (reconnect rate)
* Relay latency (p50, p95, p99)
* Error rates (handshake failures, timeouts, rejected lookups)
* Mode distribution (% of servers in Direct vs Relay at any time)

9. Initial Scope

* Minecraft Java TCP support only.
* Single AWS relay region (single AZ).
* Single relay gateway deployment (no horizontal scale).
* No load balancing.
* No multi-region routing.
* No UDP support (Minecraft Bedrock deferred).

**Goal:** Implement Esluce Relay as the primary connectivity path on *.play.esluce.net (always-on, stable across restarts and IP changes), with Direct Mode A records on *.play.esluce.com emitted as best-effort fast-path only when probe-verified. Single AWS region, single gateway, Minecraft Java TCP only.

**Depends on:** Phase 67
**Plans:** 7/7 plans complete + 1 gap-closure plan pending

**Success Criteria**:
1. Relay endpoint `<server>.play.esluce.net` is stable across ≥10 agent restarts (DNS lookup returns valid CNAME → active tunnel).
2. A user behind CGNAT can complete the full flow: install agent → create server → receive `<server>.play.esluce.net` → external player connects successfully.
3. A non-CGNAT user with working port forwarding has BOTH addresses active; latency via `play.esluce.com` is measurably lower than via `play.esluce.net` (≥20% p50 improvement).
4. When Direct Mode probe fails, the `play.esluce.com` A record is removed within 60s; no stale records accumulate in the `esluce.com` zone.
5. Mode flip is automatic: agent re-probes every 5 min and updates DNS state without requiring user action.

Plans:
- [x] 68-01-PLAN.md — Schema migration + entities + NodeMessage enum (Wave 1, blocking sqlx migrate)
- [x] 68-02-PLAN.md — Agent tunnel client: relay_client.rs, relay_session.rs, dispatch, bootstrap, D-25 rekeying (24h/100GB), D-13 CNAME cleanup on disconnect (Wave 2)
- [x] 68-03-PLAN.md — Backend RelayService + REST handlers + internal HMAC handlers + WS dispatch + D-23 alert scraper (Wave 2)
- [x] 68-04a-PLAN.md — Relay gateway crate: 13 source files with MC Handshake-parse routing (by_subdomain, NOT by_agent_ip), registry, player, metrics on :9100 (Wave 3)
- [x] 68-04b-PLAN.md — Docker + Caddy + compose (9100:9100 Prometheus exposure, TLS 1.3 enforcement) (Wave 3)
- [x] 68-04c-PLAN.md — DEPLOY.md operator runbook (AWS NLB + Route 53 static wildcard + IAM scoped + EC2 + verify) (Wave 3)
- [x] 68-05-PLAN.md — Dashboard UI: relayApi + useConnectivity + TunnelHealthCard + ModeOverrideDropdown + InviteFriendsModal + ConnectivitySection (Wave 3)
- [ ] 68-gap-01-PLAN.md — Gap closure: gateway's tunnel.rs real yamux server session + auth::authorize + WS Binary frame compatibility (closes 3 of 4 BLOCKERs from VERIFICATION.md; BLOCKER #4 rate-limiter is a verifier false positive)

### Phase 70: Auto-fetch relay config via WS

**Goal:** Auto-fetch via WS — setelah agent connect ke backend pakai AGENT_API_KEY, backend kirim relay_token + server_ids langsung via WebSocket
**Requirements**: None
**Depends on:** Phase 69
**Plans:** 3/3 plans complete

**Wave 1**
- [x] 70-01-PLAN.md — Backend: RelayConfigSync protocol variant + push_relay_config() + wire in Register handler
- [x] 70-02-PLAN.md — Agent: config storage split (GlobalRelayConfig + RelaySessionState) + apply_relay_config() + bootstrap changes

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 70-03-PLAN.md — Agent: BackendMessage::RelayConfigSync handler in agent_connection.rs

### Phase 71: buat agar plan hobby dan pro yang ada di landing page, bisa benar berfungsi untuk berlangganan

**Goal:** Wire the landing page pricing section to the backend billing/subscription system so Hobby and Pro plan buttons create real Lemon Squeezy checkout flows with auth gating, monthly/yearly toggle, auto-checkout after sign-in, welcome modal on the dashboard, and current plan badge for logged-in users.
**Requirements**: REQ-01, REQ-02, REQ-03, REQ-04, REQ-05, REQ-06, REQ-07, REQ-08, REQ-09, REQ-10
**Depends on:** Phase 70
**Plans:** 2/2 plans complete

**Wave 1** *(parallel — no file overlap)*
- [x] 71-01-PLAN.md — Landing Page Pricing & Checkout: billing API module, API-driven PricingSection with toggle, PlanCard, auth-gated checkout, sign-in auto-checkout, OAuth plan context, current plan badge
- [x] 71-02-PLAN.md — Dashboard Welcome Modal: post-checkout welcome modal on /dashboard?checkout=success, checkout=canceled toast, createPortal API method

### Phase 72: menambahkan type minecraft, dengan type Bedrock atau lebih tepatnya minecraft bedrock

**Goal:** Menambahkan Minecraft Bedrock Edition sebagai first-class server type — user dapat memilih Bedrock saat membuat server, API menggunakan Docker image yang benar (`itzg/minecraft-bedrock-server`), agent membuat container dengan UDP port binding, dan server dapat diakses oleh Minecraft Bedrock client
**Requirements**: REQ-01 to REQ-08
**Depends on:** Phase 71
**Plans:** 4/4 plans complete

**Wave 1** *(parallel — no file overlap)*
- [x] 72-01-PLAN.md — DB Migration + API Backend: bedrock game_types row, dynamic image dispatch, game_type→mc_loader mapping
- [x] 72-02-PLAN.md — Agent Runtime: UDP port binding for Bedrock, dynamic port map key, loader field forwarding
- [x] 72-03-PLAN.md — Frontend UI: Bedrock option in CreateServerModal, conditional field rendering

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 72-04-PLAN.md — End-to-End Verification: compilation tests, code presence checks, visual verification, limitation documentation

### Phase 73: Approach 1: Per-Server UDP Port (⭐ Recommended MVP)
Cara: Alokasikan satu port UDP unik per Bedrock server di relay EC2. Player connect langsung ke <relay-ip>:<dedicated-port> — port-nya yang identify server, bukan subdomain.
Apa yang perlu diubah:
Layer	Perubahan
Relay Gateway	Tambah UdpSocket listener di port range (19132-19xxx). Setiap kali agent TunnelConnect dengan loader: "bedrock", gateway buka port UDP baru dan simpan mapping port → server_id. Tiap datagram UDP yang masuk, wrap dengan length-prefix, kirim lewat yamux stream baru ke agent.
Agent	relay_session.rs perlu handle UDP: baca length-prefixed datagram dari yamux, forward ke UDP socket lokal (127.0.0.1:19132), baca balik, kirim balik ke yamux.
Backend API	Port pool perlu support UDP ports. port_pools table tambah kolom protocol (tcp/udp). Alokasi port Bedrock dari pool UDP.
NLB / Infra	Security group tambah rule UDP port range. NLB bisa handle UDP listener. Caddy nggak perlu diubah (UDP langsung ke gateway, bukan lewat Caddy).
Agent → Gateway tunnel	Yang existing (WSS → yamux) tetap untuk TCP control + heartbeat. Bedrock data stream perlu yamux stream terpisah khusus UDP datagram dengan framing length-prefix.

**Goal:** Add UDP relay support for Minecraft Bedrock Edition via per-server dedicated UDP ports on the relay gateway — backend allocates from a new UDP port pool (19132-19231), agent declares `loader: "bedrock"` in TunnelConnect and runs a TLV-framed datagram session, gateway binds UdpSocket per port with 30-second grace period, and dashboard shows Bedrock address (IP:port + DNS SRV record).
**Requirements**: None
**Depends on:** Phase 72
**Plans:** 4/4 plans complete

**Wave 1** *(Backend — no upstream deps)*:
- [x] 73-01-PLAN.md — Backend: UDP port pool seed migration + protocol-aware port allocation + loader field pipeline (ServerRelayInfo → RelayServerConfig)

**Wave 2** *(blocked on Wave 1)*:
- [x] 73-02-PLAN.md — Agent: TunnelConnect loader field + `run_udp_relay_session()` with TLV framing + `drive_inbound_streams` Bedrock dispatch

**Wave 3** *(blocked on Wave 2)*:
- [x] 73-03-PLAN.md — Gateway: UdpPortRegistry + UdpSocket lifecycle + TunnelConnect Bedrock dispatch + config/state extensions

**Wave 4** *(blocked on Wave 1 — can start after Wave 1)*:
- [x] 73-04-PLAN.md — Route 53 SRV record upsert/list/delete/resolve methods + auto-delete SRV on Bedrock server deletion (dashboard UI skipped per decision: SRV fully automatic, no user-facing toggle)

**Cross-cutting constraints:**
- D-01: Backend allocates from port_pools (protocol='udp')
- D-03: Agent declares loader in TunnelConnect; gateway derives protocol from loader
- D-05: 30-second grace period on port release
- D-08: TLV wire format: [1-byte type][4-byte big-endian length][payload]
- D-10: Agent UdpSocket with send_to/recv_from (connectionless)
- D-13: Dashboard shows IP:port + DNS SRV record

### Phase 74: menambahkan nama akun yang login ke Dashboard

**Goal:** Show the logged-in user's account identity (display name + avatar) prominently on the Dashboard — as a persistent top bar header across all pages and as a corrected welcome greeting on the server list page.
**Requirements**: None
**Depends on:** Phase 73
**Plans:** 1/1 plans complete

Plans:
- [x] 74-01-PLAN.md — Create TopBar component, integrate into Layout, fix welcome message display_name

### Phase 75: update UI https://app.esluce.com/servers

**Goal:** Redesign the server list page (`/servers`) with card/table view toggle, sort controls, game type filter, restart button with confirmation, 30s auto-refresh, and status change notifications. No new API endpoints — purely frontend.
**Requirements**: None
**Depends on:** Phase 74
**Plans:** 2 plans

**Wave 1**
- [ ] 75-01-PLAN.md — View toggle (card/table), sort controls, game type filter, table view markup, localStorage persistence, cosmic theme restyling

**Wave 2** *(blocked on Wave 1 completion)*
- [ ] 75-02-PLAN.md — Restart button + confirmation modal, 30s polling, status change detection + toast notifications

### Phase 76: update UI https://app.esluce.com/nodes

**Goal:** Redesign the Nodes management page (`/nodes`) with split-panel layout, table/card view toggle (matching Phase 75 pattern), enriched node card content (uptime + last seen), table view, and visual refresh of health metrics with progress bars and color coding. No new API endpoints.
**Requirements**: None
**Depends on:** Phase 75
**Plans:** 1 plan

Plans:
- [ ] 76-01-PLAN.md — Nodes.jsx: view toggle, enriched cards, table view, health metrics progress bars, toast/confirm replacements, Trash2 icons

### Phase 77: update UI https://app.esluce.com/templates

**Goal:** Replace stacked sections with filter tabs (Featured/Yours/All), enrich cards with version/updated/count info, add sort + category filter, refine form styling. No changes to Mod Browser.
**Requirements**: D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08
**Depends on:** Phase 76
**Plans:** 3 plans

**Wave 0** (backend prerequisite)
- [ ] 77-01-PLAN.md — Backend: add version + usage_count fields to TemplateResponse DTO + SQL

**Wave 1** (blocked on Wave 0)
- [ ] 77-02-PLAN.md — Frontend library: filter tabs, sort, category filter, enriched cards, delete modal, table view, error/empty states

**Wave 2** (blocked on Wave 1)
- [ ] 77-03-PLAN.md — Frontend form: cosmic theme consistency pass (focus rings, checkbox accent, back link, save button)

### Phase 78: update UI https://app.esluce.com/mods

**Goal:** Improve the Mod Browser page at `/mods` with enriched result cards, working Add-to-server flow, version detail modal, enhanced filters, and page-number pagination.
**Requirements**: D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10
**Depends on:** Phase 77
**Plans:** 3 plans

Plans:
- [ ] 78-01-PLAN.md — Rust backend: add missing DTO fields (author, latest_version, date_published) + game-versions endpoint
- [ ] 78-02-PLAN.md — Frontend enrichment: category filter, dynamic versions, pagination, version modal, enriched cards
- [ ] 78-03-PLAN.md — Add-to-Server modal: version picker, server picker, install flow, toast feedback

### Phase 79: update UI https://app.esluce.com/billing

**Goal:** Restyle billing page with cosmic theme (`glass-panel`), add usage progress bars from QuotaService, mount existing UsageHandlers route, and collapse subscription states from 3 to 2.
**Requirements**: D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10
**Depends on:** Phase 78
**Plans:** 2 plans

**Wave 1** *(parallel — no file overlap)*
- [ ] 79-01-PLAN.md — Mount UsageHandlers router at /api/v1/usage (1 line, api_routes.rs)
- [ ] 79-02-PLAN.md — Add usageApi client + full BillingPage cosmic restyle + usage bars

### Phase 80: update UI https://app.esluce.com/settings

**Goal:** Restyle Settings page from flat `bg-gray-700`/`bg-gray-800` to cosmic `glass-panel` theme AND split the 1433-line monolith into a shell + 4 extracted component files

**Requirements**: SPLIT-01, SPLIT-02, SPLIT-03, RESTYLE-01, RESTYLE-02, RESTYLE-03, NO-CHANGE-01
**Depends on:** Phase 79
**Plans:** 3 plans

Plans:
- [ ] 80-01-PLAN.md — File splitting: create 4 component files (ProfileSettings, ApiKeySettings, WebhookSettings, RestartDefaultsSettings), reduce SettingsPage.jsx to shell
- [ ] 80-02-PLAN.md — Cosmic restyle of all new components + SettingsPage shell, Profile tab sub-section grouping with dividers
- [ ] 80-03-PLAN.md — Cosmic restyle of existing CloudflareSettings.jsx and S3ProfileSettings.jsx

### Phase 81: update UI di halaman utama/dashboard app.esluce.com

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 80
**Plans:** 2/2 plans complete

Plans:
- [x] TBD (run /gsd-plan-phase 81 to break down) (completed 2026-06-15)

### Phase 82: Membuat theme dan warna keseluruhan menjadi lebih konsisten, dan membuat toggle light/dark berfungsi dengan benar

**Goal:** Add semantic CSS variables, fix hardcoded colors across all JSX files, implement functional light/dark toggle with system-preference support, and hide stars/glows in light mode
**Requirements**: No numbered requirements — implicit from phase goal
**Depends on:** Phase 81
**Plans:** 3 plans

**Wave 1** *(foundation — CSS vars + toggle)*
- [ ] 82-01-PLAN.md — Foundation: semantic CSS variables in @theme, light theme cleanup, transition CSS, flash-guard script, system-preference detection

**Wave 2** *(blocked on Wave 1)*
- [ ] 82-02-PLAN.md — Visible components: App.jsx sidebar, TopBar, Sidebar, ToastContainer, NotificationCenter, Onboarding, email verification components, InviteFriendsModal

**Wave 3** *(blocked on Wave 2)*
- [ ] 82-03-PLAN.md — Remaining components: auth pages, server detail components, FileManager, LogViewer, skeleton loaders, already-cosmic page vestiges

### Phase 83: buat onboarding untuk mempermudah user membuat server yang di inginkan ketika menekan 'Create your first server' di dashboard utama

**Goal:** Multi-step onboarding wizard (ServerOnboardingWizard) that replaces the `navigate('/servers')` behavior when first-time users click "Create your first server" — wizard guides through Type → Resources → Config → Deploy with progress bar, preserves existing CreateServerModal for experienced users
**Requirements**: ONB-01, ONB-02, ONB-03, ONB-04, ONB-05, ONB-06
**Depends on:** Phase 82
**Plans:** 1 plan

Plans:
- [ ] 83-01-PLAN.md — Create ServerOnboardingWizard component + extract shared constants + wire into DashboardPage
