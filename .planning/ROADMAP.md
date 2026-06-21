# Roadmap: Escluse Community Bot

## Overview

The Escluse Community Bot transforms the Escluse Discord server into a self-service information hub, onboarding assistant, support platform, and community engagement engine. Starting from project scaffolding through four progressive phases, the bot ships as a two-process system (TypeScript/Bun for Discord interaction, Rust/Axum for the API layer) with PostgreSQL + Redis persistence. Each phase delivers an independently valuable capability — onboarding first, then information access, then support and administration tooling, and finally community differentiators that make Escluse unique.

## Phases

- [ ] **Phase 1: Foundation & Onboarding** - Project scaffolding, bot client, welcome system, interest selection, auto-role assignment, permission system
- [ ] **Phase 2: Information Hub** - Documentation search, pricing info, FAQ, release announcements, maintenance notices
- [ ] **Phase 3: Support & Administration** - Structured bug reports, channel management, role management, member logging
- [ ] **Phase 4: Community Features** - Feature voting/polls, community surveys, GitHub integration, referral tracking, contribution recognition, special roles

## Phase Details

### Phase 1: Foundation & Onboarding

**Goal**: New members get automatically welcomed and assigned roles based on interests; server staff can configure behavior via settings.
**Mode**: mvp
**Depends on**: Nothing (first phase)
**Requirements**: ONB-01, ONB-02, ONB-03, PERM-01, ADMN-03
**Success Criteria** (what must be TRUE):

  1. New member joins server → receives a welcome embed with getting-started guides, links, and resources
  2. New member can select interests via interactive buttons/select menu → gets auto-assigned matching roles
  3. Admin can configure welcome message content, interest options, and role mappings via server settings command
  4. All bot commands respect role-based permissions (configurable per role and per command)

**Plans**: 3 plans in 2 waves
**UI hint**: yes

Plans:

- [ ] 01-01: Walking Skeleton — Scaffold monorepo, Docker Compose (postgres/redis/api/bot), bot client with command handlers, Axum API with DB, /ping command
- [ ] 01-02: Settings + Permissions — Normalized settings tables, `/admin settings` command, role-based permission system, command gate
- [ ] 01-03: Welcome + Interest Onboarding — Multi-message bilingual welcome, interest select menu, dual auto-role system, role logging

### Phase 2: Information Hub

**Goal**: Users can find information about Escluse without asking the founder — self-service knowledge.
**Mode**: mvp
**Depends on**: Phase 1
**Requirements**: INFO-01, INFO-02, INFO-04, UPDT-01, UPDT-03
**Success Criteria** (what must be TRUE):

  1. User can search documentation via `/escluse docs <query>` and get relevant search results with links
  2. User can view pricing, features, and "what is Escluse" information via `/escluse pricing` and `/escluse what-is`
  3. User can get quick answers from FAQ via a slash command
  4. Admin can publish release announcements and changelog updates to a designated announcements channel
  5. Admin can post maintenance notices visible to the server community

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] 02-01: [Plan description — TBD during phase planning]

### Phase 3: Support & Administration

**Goal**: Users can submit structured bug reports; staff can manage server channels, roles, and member activity.
**Mode**: mvp
**Depends on**: Phase 2
**Requirements**: SUPP-02, ADMN-01, ADMN-02, ADMN-04
**Success Criteria** (what must be TRUE):

  1. User can submit a structured bug report (version, OS, steps, logs) via a modal form
  2. Admin can create, edit, and delete channels via slash commands
  3. Admin can manage roles (create, assign, remove) via slash commands
  4. Staff can view member join/leave logs and event history via a logging channel

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] 03-01: [Plan description — TBD during phase planning]

### Phase 4: Community Features

**Goal**: Community members can participate in polls, vote on features, get recognized for contributions, and follow development via GitHub.
**Mode**: mvp
**Depends on**: Phase 3
**Requirements**: FDBK-01, FDBK-02, DEV-01, RECG-01, RECG-02, RECG-03
**Success Criteria** (what must be TRUE):

  1. User can vote on feature requests and participate in game support polls
  2. User can respond to community surveys and template requests
  3. GitHub events (issues, PRs, releases) automatically appear in a designated Discord channel
  4. User can generate referral links and track successful referrals
  5. User receives role-based recognition (Early Supporter, Beta Tester, Contributor, Community Helper) based on contributions and contributions are tracked

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] 04-01: [Plan description — TBD during phase planning]

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & Onboarding | 0/3 | Not started | - |
| 2. Information Hub | 0/TBD | Not started | - |
| 3. Support & Administration | 0/TBD | Not started | - |
| 4. Community Features | 0/TBD | Not started | - |

### Phase 88: Update UI/UX console header and terminal layout

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 87
**Plans:** 2/2 plans complete

Plans:

- [x] TBD (run /gsd-plan-phase 88 to break down) (completed 2026-06-18)

### Phase 89: audit seluruh copy dan CTA di landing page saya dari perspektif hukum/legalitas bisnis digital. 

Cakupan audit meliputi seluruh teks yang ada di menu:

- Product: Features, Pricing, Supported Games, Changelog

- Resources: Documentation, API Reference, Community Forum

- Company: About Us, Legal, Contact

Tolong kelompokkan analisis kamu menjadi 3 kategori:

1. "KATA/CTA YANG AMAN" (Boleh digunakan tanpa syarat)

2. "KATA/CTA BERISIKO" (Boleh digunakan tapi butuh disclaimer/catatan hukum)

3. "KATA/CTA YANG HARUS DIHINDARI" (Secara hukum ilegal, menjebak, atau melanggar hak cipta pihak ketiga, terutama di bagian Supported Games dan Pricing).

**Goal:** Landing page copy and CTAs audited against hybrid legal framework (UU ITE, UU PDP, Consumer Protection, UU Merek, GDPR, FTC) with findings classified into Safe/Risky/Must Avoid tiers — producing an actionable compliance review report with specific legal citations, risk assessments, and concrete fix suggestions for every legal exposure.

**Requirements**: None (audit/report phase — no feature requirements apply)
**Depends on:** Phase 88
**Plans:** 1/1 plans complete

Plans:

- [x] 89-01-PLAN.md — Read all 16 source files, classify copy elements against hybrid legal framework, compile 89-AUDIT-REPORT.md with 3-tier classification (Safe/Risky/Must Avoid), fix suggestions, legal citations, and future recommendations. Per D-01 to D-10 locked decisions.

### Phase 90: implementasi hasil analisis legal landing page untuk halaman About Us, Legal, dan Contact sesuai kerangka hybrid (UU ITE, UU PDP, UU Perlindungan Konsumen, UU Merek, GDPR, FTC)

**Goal:** Implement compliance fixes for About Us, Legal, and Contact pages based on audit findings across hybrid legal framework (UU ITE, UU PDP, UU Perlindungan Konsumen, UU Merek, GDPR, FTC). About Us must display PT status, NIB, address, contact info. Legal page must include Privacy Policy (8 mandatory clauses), Terms of Service, Cookie Policy. Contact page must include data protection notice (data controller identity, purpose, retention, rights, complaint channel).
**Requirements**: None (implementation phase based on 89-AUDIT-REPORT.md findings)
**Depends on:** Phase 89
**Plans:** 3 plans in 1 wave

Plans:

- [x] 90-01-PLAN.md — Entity disclosure (AboutUs.tsx), Legal page removal (Legal.tsx delete + App.tsx route/nav/copyright), Footer copyright updates (both footers) (completed 2026-06-21)
- [x] 90-02-PLAN.md — Contact page data protection microcopy (Contact.tsx), Terms of Service date sync (TermsOfService.tsx) (completed 2026-06-21)
- [ ] 90-03-PLAN.md — Privacy Policy full compliance: enriched Cookies + 5 new sections (Data Retention, International Data Transfers, Breach Notification, Dispute Resolution, Contact and Complaints) + date sync

**Cross-cutting constraints:**

- npm run build succeeds with no TypeScript errors
