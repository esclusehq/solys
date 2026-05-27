# Phase 4: Node Agent Communication - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 4-Node Agent Communication
**Areas discussed:** Connection protocol, Node authentication, Task distribution, Reconnection handling

---

## Connection Protocol

| Option | Description | Selected |
|--------|-------------|----------|
| JSON + ping/pong | JSON messages with type field, WebSocket ping/pong for heartbeat | ✓ |
| Binary protobuf | Binary protobuf messages for efficiency | |
| Text protocol | Plain text commands with simple parsing | |

**User's choice:** JSON + ping/pong (Recommended)
**Notes:** Aligns with existing JSON message handling.

---

## Node Authentication

| Option | Description | Selected |
|--------|-------------|----------|
| API key | API key passed on connection, stored in database, rotated periodically | ✓ |
| JWT tokens | JWT tokens with expiration, refresh mechanism | |
| Static credentials | Username/password per node stored in config | |

**User's choice:** API key (Recommended)
**Notes:** Simple and effective for machine-to-machine auth.

---

## Task Distribution

| Option | Description | Selected |
|--------|-------------|----------|
| Redis queue + WS | Tasks queued in Redis, workers pull from queue, async response via WebSocket | ✓ |
| Direct WS messaging | Direct WebSocket messages for each task, blocking until complete | |
| HTTP + WS hybrid | HTTP endpoints for task submission, WebSocket for status updates | |

**User's choice:** Redis queue + WS (Recommended)
**Notes:** Leverages existing Redis infrastructure.

---

## Reconnection Handling

| Option | Description | Selected |
|--------|-------------|----------|
| State machine | State machine tracks: disconnected → connecting → authenticating → connected → ready | ✓ |
| Simple backoff | Simple reconnect with backoff, no state tracking | |
| Agent-managed | Let agent handle reconnection, API just tracks last seen | |

**User's choice:** State machine (Recommended)
**Notes:** Prevents race conditions during reconnect.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
