# Phase 54: Email Verification Flow - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-30
**Phase:** 54-email-verification
**Areas discussed:** Post-registration UX, Resend verification, Sensitive action gating, OAuth and verification

---

## Post-registration UX

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-login + banner | Keep current auto-login flow, show prominent banner | |
| Redirect to pending page | Show dedicated "Check your email" page after signup | |
| Hybrid | Auto-login + persistent global banner + feature-blocking dialog | ✓ |

**User's choice:** Hybrid — auto-login then dashboard with persistent global banner. Banner includes [Resend Email] and [Change Email] links. When accessing gated features, a modal dialog blocks with "Email verification required".

**Notes:**
- Banner lives globally on all pages, not just dashboard
- Feature-blocking dialog appears on: API Keys, Billing, Team Invites, Webhooks, Project Creation, Server Creation, Server Deployment, Node Registration, Instance Creation, Environment Creation, External Integrations, SDK Credentials, Subscription Changes, Invoice Management, Payment Methods
- After successful verification: success page → 2-3s auto-redirect → returnTo if exists, else dashboard. Manual button fallback

---

## Resend Verification

| Option | Description | Selected |
|--------|-------------|----------|
| 60-second cooldown | User can resend after 60 seconds | ✓ |
| 2-minute cooldown | Standard security cooldown | |
| 5-minute cooldown | More restrictive | |
| Unlimited | No hard cap on resends | |
| Rate-limited (5/hour) | 5 per hour cap | |
| Hard cap (5 total) | 5 total, then expired | ✓ |
| Contact support | Message when cap reached | ✓ |
| Re-register option | Start fresh with new account | |
| Admin override | Manual verification from admin panel | |
| Countdown on button | Button shows "Resend (55s)" disabled | |
| Button hides during cooldown | Button disappears, reappears later | |
| Toast + tooltip | Toast on send, tooltip on hover during cooldown | ✓ |

**User's choice:** 60-second cooldown, hard cap of 5 total resends, then contact support. Toast notification on send + tooltip with remaining time on hover during cooldown.

---

## Sensitive Action Gating

| Option | Description | Selected |
|--------|-------------|----------|
| All 5 | API Keys, Billing, Team Invites, Webhooks, Project Creation | |
| Minimal | Billing + Webhooks only | |
| Full resource-based | Gating based on resource abuse potential, not feature importance | ✓ |

**User's choice:** Gate based on potential for resource abuse and cost. 4 categories: Identity & Access (API Keys, PATs, OAuth Apps, Team Invites, Team Creation), Resource Creation (Project, Server, Deployment, Node, Instance, Environment), Integration (Webhooks, External Integrations, SDK Credentials), Financial (Billing, Subscription, Invoice, Payment Methods).

Ungated: Login, Dashboard, Profile Settings, Security Settings, Login History, Documentation, Pricing, Community/Discord, Account Preferences.

Enforcement: Backend + frontend. Backend uses custom `VerifiedUser` extractor (like `AuthUser`). Enhanced 403 error with resend data.

---

## OAuth and Verification

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, auto-verified | Set email_verified_at on OAuth registration | |
| No, require separate verification | OAuth users must verify separately | |
| Auto-verify, re-verify on email change | Auto-verify OAuth, pending-email on change | ✓ |
| Mark unverified, gate features | New email unverified, blocks all gated features | |
| Partial: gate only billing | Only billing re-requires verification | |
| Keep verified, no re-verification | Email change doesn't affect status | |
| Pending-email pattern | Keep old verified, store pending_email, switch after verify | ✓ |

**User's choice:** OAuth users auto-verified. Email changes use pending-email pattern: keep old email verified, store `pending_email`, send verification to new email, continue using features, switch only after new email verified.

---

## the agent's Discretion

- Specific UI design of the global banner (colors, position, dismissibility)
- Specific UI design of the feature-blocking dialog
- Backend endpoint design for resend
- Migration design for `pending_email` column
- Route tree placement for gating middleware/extractor
- Frontend component structure

## Deferred Ideas

None — discussion stayed within phase scope
