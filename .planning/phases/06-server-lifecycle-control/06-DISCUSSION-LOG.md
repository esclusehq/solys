# Phase 6: Server Lifecycle Control - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 6-Server Lifecycle Control
**Areas discussed:** Server state management, Delete confirmation, Stop behavior, Restart strategy

---

## Server State Management

| Option | Description | Selected |
|--------|-------------|----------|
| Optimistic with async | DB status updated immediately, async operation in background | ✓ |
| Synchronous updates | Wait for operation to complete before updating status | |
| Agent-driven status | Status stays pending until agent reports completion | |

**User's choice:** Optimistic with async (Recommended)
**Notes:** Fast feedback to users.

---

## Delete Confirmation

| Option | Description | Selected |
|--------|-------------|----------|
| UI confirm + soft delete | UI shows confirmation dialog, soft delete in DB, cleanup after delay | ✓ |
| Immediate delete | No confirmation, immediate hard delete | |
| Type-to-confirm | Requires typing server name to confirm | |

**User's choice:** UI confirm + soft delete (Recommended)
**Notes:** Prevents accidental deletes, allows recovery.

---

## Stop Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Graceful with timeout | Send stop signal, wait 30s grace period, force kill if needed | ✓ |
| Force stop | Immediate kill, no grace period | |
| Fire and forget | Send stop signal and return immediately | |

**User's choice:** Graceful with timeout (Recommended)
**Notes:** Prevents data corruption.

---

## Restart Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Stop then start | Stop, wait for cleanup, then start - preserves container | ✓ |
| Destroy and recreate | Destroy and recreate container from scratch | |
| In-container restart | Send restart signal via RCON, no container lifecycle change | |

**User's choice:** Stop then start (Recommended)
**Notes:** Preserves container, faster than recreate.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
