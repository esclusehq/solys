# Phase 2: Infrastructure Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 2-Infrastructure Foundation
**Areas discussed:** PostgreSQL schema strategy, Redis configuration, WebSocket infrastructure, Repository layer design

---

## PostgreSQL Schema Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| sqlx with .sql files | Use sqlx macros for compile-time schema checking, requires all SQL in .sql files | ✓ |
| sqlx runtime queries | Runtime queries, less safe but more flexible during development | |
| Diesel ORM | Use an ORM like diesel for schema management | |

**User's choice:** sqlx with .sql files (Recommended)
**Notes:** Existing migrations already follow this pattern.

---

## Redis Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Multi-purpose | Use for sessions, rate limiting, and node status caching | ✓ |
| Queue only | Only for job queue, other data in PostgreSQL | |
| Full query caching | Comprehensive caching of all query results | |

**User's choice:** Multi-purpose (Recommended)
**Notes:** Aligns with existing rate limiting implementation.

---

## WebSocket Infrastructure

| Option | Description | Selected |
|--------|-------------|----------|
| Axum native | Built-in Axum WebSocket with tokio-tungstenite for node agents | ✓ |
| Message broker based | Use a separate message broker for all WebSocket communication | |
| Raw tungstenite | Custom implementation with raw tokio-tungstenite | |

**User's choice:** Axum native (Recommended)
**Notes:** Existing handlers already use Axum native approach.

---

## Repository Layer Design

| Option | Description | Selected |
|--------|-------------|----------|
| Async traits | Async traits with concrete implementations, dependency injection via container | ✓ |
| Macros-based | Generate repository implementations from traits using macros | |
| Concrete only | Direct struct implementations without traits | |

**User's choice:** Async traits (Recommended)
**Notes:** Follows existing pattern in domain/repositories/.

---

## Agent's Discretion

No areas delegated to agent discretion — all decisions explicitly made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
