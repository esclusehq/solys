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
**Plans:** 6/8 plans executed

Plans:
**Wave 1**
- [x] 52-01-PLAN.md — VitePress Infrastructure: data loader, Vue components, theme registration, CSS
- [x] 52-02-PLAN.md — Sidebar Configuration: full API Reference navigation tree

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 52-03-PLAN.md — Core Docs: overview restructure, auth guide, error catalog, changelog
- [x] 52-04-PLAN.md — Servers Group: CRUD, operations, console, properties, cron tasks
- [x] 52-05-PLAN.md — Servers Extended: files, backups, plugins, git, build, deploy, profiling
- [x] 52-06-PLAN.md — Nodes + Billing: all node endpoints + billing with sub-pages
- [ ] 52-07-PLAN.md — Remaining Resources: webhooks, alerts, settings, templates, agents, jobs, usage, runtimes, deploy
- [ ] 52-08-PLAN.md — SDK Guides: Node.js + Python quickstarts

**Cross-cutting constraints:**
- D-05: Field-level schema tables on every page
- D-06: curl + Node.js SDK + Python SDK examples per endpoint
- D-04: HTTP interface only, no proprietary implementation

### Phase 53: User Profile Management

**Goal:** Halaman profile untuk users setelah OAuth login - view/update profile info (email, avatar, name), update display name, change password (email accounts), view login history, delete account option
**Requirements**: None
**Depends on:** Phase 49 (Fix login functionality)
**Plans:** 0 plans

### Phase 54: Email Verification Flow

**Goal:** Verifikasi email untuk users yang signup dengan email - send verification email on registration, resend verification option, require verified email for sensitive actions
**Requirements**: None
**Depends on:** Phase 49 (Fix login functionality)
**Plans:** 0 plans

### Phase 55: Scheduled Backups

**Goal:** Backup otomatis untuk data server secara berkala dan terjadwal - configurable backup intervals, retention policies, backup storage location
**Requirements**: None
**Depends on:** Phase 51 (Automasi DNS)
**Plans:** 0 plans

### Phase 56: Auto Online & Sleep Recovery

**Goal:** Server dapat kembali aktif otomatis setelah offline atau sleep - auto restart on crash, sleep mode detection, automatic wake-up mechanisms
**Requirements**: None
**Depends on:** Phase 51 (Automasi DNS)
**Plans:** 0 plans

### Phase 57: Auto Restart Policies

**Goal:** Restart server secara otomatis saat crash atau tidak merespons - crash detection, configurable restart rules, max restart attempts, cooldown periods
**Requirements**: None
**Depends on:** Phase 56 (Auto Online & Sleep Recovery)
**Plans:** 0 plans

### Phase 58: Server, Plugin, and Modpack Templates

**Goal:** Templates untuk server, plugin, dan modpack untuk mempermudah deployment dan konfigurasi server secara instan - pre-configured game templates, plugin bundles, modpack configurations
**Requirements**: None
**Depends on:** Phase 51 (Automasi DNS)
**Plans:** 0 plans

### Phase 59: Server Scheduling

**Goal:** Mengatur start, stop, restart, dan sleep server secara otomatis berdasarkan jadwal - scheduled start/stop, recurring schedules, timezone support
**Requirements**: None
**Depends on:** Phase 57 (Auto Restart Policies)
**Plans:** 0 plans

### Phase 60: Crash Detection

**Goal:** Mendeteksi server yang berhenti atau crash secara otomatis dan menjalankan recovery.

**Depends on:** Phase 57 (Auto Restart Policies)
**Plans:** 0 plans

### Phase 61: Create DEVELOPMENT.md - Setup local dev environment

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 60
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 61 to break down)

### Phase 62: Create CONTRIBUTING.md - Cara kontribusi

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 61
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 62 to break down)

### Phase 63: Create ARCHITECTURE.md - technical documentation (module-level)

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 62
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 63 to break down)

### Phase 64: Create database schema documentation (for developers who want to extend)

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 63
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 64 to break down)

### Phase 65: Buat installer script auto-install Docker sebelum install Solys agent

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 64
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 65 to break down)
