# Phase 5: Server Deployment - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 5-Server Deployment
**Areas discussed:** Game type configuration, Port allocation strategy, Resource limits, Deployment config storage

---

## Game Type Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Code-based | Game types map to Docker images in code, easily extensible | |
| Database-driven | Database table with game configs, no code changes for new games | ✓ |
| Config file based | JSON/YAML config files, loaded at startup | |

**User's choice:** Database-driven with code fallback
**Notes:** Game types stored in database for SaaS flexibility + code fallback for safety.

---

## Port Allocation Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Dynamic pool | Dynamic port allocation from pool, database tracks allocated ports | ✓ |
| User-specified | User specifies port, system validates availability | |
| Fixed per game | Fixed port per game type, no allocation needed | |

**User's choice:** Dynamic pool (Recommended)
**Notes:** Prevents port conflicts automatically.

---

## Resource Limits

| Option | Description | Selected |
|--------|-------------|----------|
| Plan-based limits | User picks from plans: 2GB, 4GB, 8GB, 16GB; fixed CPU ratios | ✓ |
| User-specified values | User specifies exact CPU cores and RAM in MB/GB | |
| Unlimited | No limits, containers use host resources unrestricted | |

**User's choice:** Plan-based limits (Recommended)
**Notes:** Simplifies UX, prevents over-provisioning.

---

## Deployment Config Storage

| Option | Description | Selected |
|--------|-------------|----------|
| Server environment JSON | Store in server.environment JSON field, sent to agent on deployment | |
| Separate table | Separate deployment_config table with foreign key to servers | |
| Image-based | Config embedded in container image, no runtime config | ✓ |

**User's choice:** Hybrid - separate table + snapshot
**Notes:** deployment_configs for templates, servers.deployment_snapshot for runtime immutability.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
