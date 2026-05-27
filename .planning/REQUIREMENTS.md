# Requirements: Esluce

**Version:** 1.0
**Date:** 2026-04-09

## v1 Requirements

### Server Lifecycle (DEPLOY-01 to DEPLOY-05)

- [ ] **DEPLOY-01**: User can deploy a game server to a selected node with specified game type
- [x] **DEPLOY-02**: User can start a deployed game server
- [x] **DEPLOY-03**: User can stop a running game server
- [x] **DEPLOY-04**: User can restart a running game server
- [x] **DEPLOY-05**: User can delete a game server

### Server Status (STATUS-01 to STATUS-02)

- [x] **STATUS-01**: User can view current server status (online/offline/starting/stopping)
- [x] **STATUS-02**: User can view server resource usage (CPU, RAM, disk)

### RCON Access (RCON-01 to RCON-02)

- [ ] **RCON-01**: User can connect to server via RCON protocol
- [ ] **RCON-02**: User can execute console commands via RCON

### File Management (FILE-01 to FILE-03)

- [ ] **FILE-01**: User can browse server files via SFTP
- [ ] **FILE-02**: User can upload files to server
- [ ] **FILE-03**: User can download files from server

### Authentication (AUTH-01)

- [ ] **AUTH-01**: User can authenticate via existing Supabase auth

## v2 Requirements (Deferred)

- **BACKUP-01**: User can schedule automated backups — scheduled for Phase 2
- **BACKUP-02**: User can restore from backup — scheduled for Phase 2
- **CONSOLE-01**: User can view real-time console output — builds on RCON infrastructure
- **DISCORD-01**: User can receive Discord notifications — post-MVP integration
- **SCHEDULED-01**: User can schedule cron tasks — post-MVP automation

## Out of Scope

- [Custom billing integration] — Use existing Stripe integration
- [Multiple cloud providers] — Focus on single provider first
- [White-label/reseller] — Multi-tenancy scope deferred
- [Mobile app] — Responsive web sufficient for initial release
- [DDoS protection] — Partnership required, not in initial scope

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| DEPLOY-01 | 5 | - |
| DEPLOY-02 | 6 | - |
| DEPLOY-03 | 6 | - |
| DEPLOY-04 | 6 | - |
| DEPLOY-05 | 6 | - |
| STATUS-01 | 7 | - |
| STATUS-02 | 7 | - |
| RCON-01 | 8 | - |
| RCON-02 | 8 | - |
| FILE-01 | 8 | - |
| FILE-02 | 8 | - |
| FILE-03 | 8 | - |
| AUTH-01 | 3 | - |

---

*Last updated: 2026-04-09*