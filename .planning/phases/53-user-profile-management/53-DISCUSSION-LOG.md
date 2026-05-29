# Phase 53: User Profile Management - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-29
**Phase:** 53-user-profile-management
**Areas discussed:** Avatar upload, Login history, Delete account, Display name

---

## Avatar Upload

| Option | Description | Selected |
|--------|-------------|----------|
| Supabase Storage (Recommended) | Reuse existing @supabase/supabase-js integration. Built-in buckets, CDN, no extra infra. Consistent with auth provider. | ✓ |
| Backend S3 | Use existing aws-sdk-s3 in the Rust backend. More control, but needs new upload endpoint + signed URLs. | |
| External (Gravatar) | No storage needed — use Gravatar by email hash. Simplest, but users lose control over image. | |

**User's choice:** Supabase Storage (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Click-to-upload (Recommended) | Click avatar → file picker → auto-upload. Simple, one-click. No crop step. | |
| Click + crop dialog | Click avatar → file picker → crop/edit dialog → confirm → upload. More polished but needs a crop component. | |
| Click + drag-drop | Click avatar → file picker OR drag image onto avatar area → auto-upload. Support both methods. | ✓ |

**User's choice:** Click + drag-drop
**Notes:** Support both interaction modes

---

| Option | Description | Selected |
|--------|-------------|----------|
| 2MB max, JPG/PNG/WebP (Recommended) | Standard limits. Enforce client-side before upload. Display friendly error if exceeded. | ✓ |
| 5MB max, JPG/PNG/WebP/GIF | Allow animated avatars (GIF). Larger limit for higher quality. | |
| 1MB max, JPG/PNG only | Strict limits to minimize storage costs. Server-side resize to 256x256. | |

**User's choice:** 2MB max, JPG/PNG/WebP (Recommended)
**Notes:** Client-side enforcement before upload

---

## Login History

| Option | Description | Selected |
|--------|-------------|----------|
| Supabase Auth events (Recommended) | Query via supabase-js or auth admin API. No backend work needed. | |
| Custom backend tracking | Log each auth request in a new login_history table. Needs new migration + endpoints. | |
| Both — Supabase + custom fields (Recommended) | Use Supabase events as base, enrich with custom data (user-agent, device) via backend middleware. | ✓ |

**User's choice:** Both — Supabase + custom fields (Recommended)
**Notes:** Enrich with user-agent and device info

---

| Option | Description | Selected |
|--------|-------------|----------|
| Timestamp, IP, device/browser, provider | Standard fields. Clean and informative. | |
| Timestamp, IP, device, location, provider | Add IP geolocation via a lookup service. More informative but adds external API dependency. | |
| All fields (Recommended) | Full detail including session ID for linking to active sessions. Allows remote session termination. | ✓ |

**User's choice:** All fields (Recommended)
**Notes:** Session ID enables future remote session termination

---

| Option | Description | Selected |
|--------|-------------|----------|
| 30 days | Keep last 30 days. Clean old entries via cron job. Minimal storage cost. | |
| 90 days (Recommended) | Reasonable for most users. Allows reviewing recent months of activity. | ✓ |
| Unlimited (user can clear manually) | Keep all history. Add a 'Clear history' button. Needs more storage but gives user full control. | |

**User's choice:** 90 days (Recommended)
**Notes:** Scheduled cleanup

---

## Delete Account

| Option | Description | Selected |
|--------|-------------|----------|
| Soft delete with grace period (Recommended) | Mark account as deleted, keep data for N days. User can cancel within grace period. After grace, hard delete. Safer. | ✓ |
| Immediate hard delete | Delete all user data immediately. No undo. Simpler but risky. | |
| Soft delete only | Deactivate account permanently. Data retained for compliance/audit. No hard delete. | |

**User's choice:** Soft delete with grace period (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| 7 days | Standard grace period. Enough time to change mind. Data fully restorable. | |
| 14 days (Recommended) | Two weeks. Covers most cases where users regret deletion. | ✓ |
| 30 days | Generous grace period. Aligns with billing cycles. | |

**User's choice:** 14 days (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Stop servers, warn about data loss (Recommended) | Gracefully stop running servers. Warn user their server data will be purged after grace period. Cancel subscriptions. | |
| Immediately delete all resources | Stop and delete everything immediately. No restoration. Cleanest but most destructive. | |
| Transfer ownership option | Allow user to transfer servers to another account before deletion deadline. Complex but user-friendly. | ✓ |

**User's choice:** Transfer ownership option
**Notes:** Allow server transfer before deletion deadline

---

| Option | Description | Selected |
|--------|-------------|----------|
| Re-authentication + type 'DELETE' (Recommended) | User must re-enter password AND type 'DELETE' to confirm. Prevents accidental deletion. | ✓ |
| Re-authentication only | Just re-enter password to confirm. Simpler but higher risk of accidents. | |
| Re-auth + type 'DELETE' + checkbox | Re-enter password, type DELETE, plus checkbox acknowledging all consequences. Most thorough. | |

**User's choice:** Re-authentication + type 'DELETE' (Recommended)
**Notes:** Prevents accidental deletion

---

## Display Name

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — separate display name (Recommended) | Display name is independent from email/OAuth identifier. User can change it freely. Shows in sidebar, server cards, headers. | ✓ |
| No — use name from OAuth/email | Reuse existing 'name' field. Simpler, less fields. User edits existing name field. | |

**User's choice:** Yes — separate display name (Recommended)
**Notes:** Independent field from login identifier

---

| Option | Description | Selected |
|--------|-------------|----------|
| Sidebar + header + server cards | Show in sidebar user info area, top header/nav, and on server cards/list items. Most visible. | |
| Sidebar only (Recommended) | Show in sidebar user area at bottom. Simple, doesn't clutter other views. | ✓ |
| Sidebar + settings header | Show in sidebar and at top of settings page. Keep server cards clean. | |

**User's choice:** Sidebar only (Recommended)
**Notes:** Keep it simple, not cluttering other views

---

## the agent's Discretion

- Specific API endpoint design for profile CRUD
- Login history table schema in Postgres
- Avatar storage bucket naming and access policy
- UI layout of the delete account section within the profile tab
- Transfer ownership UX flow

## Deferred Ideas

None — discussion stayed within phase scope
