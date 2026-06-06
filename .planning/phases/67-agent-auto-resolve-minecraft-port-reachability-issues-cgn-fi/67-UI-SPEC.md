# Phase 67: Agent auto-resolve Minecraft port reachability issues - UI Design Contract

**Generated:** 2026-06-07
**Source:** CONTEXT.md D-15, D-16, D-17 (locked UI decisions from discuss-phase)
**Mode:** inline-synth (yolo mode + execute-end-to-end instruction; normally gsd-ui-researcher is spawned)

---

## Frontend Additions

### 1. Per-Server Connectivity Badge (Servers list)

**Location:** `app/src/pages/servers/ServerManagerPage.jsx` (server list row)

**Visual:**
```
┌──────────────────────────────────────────────┐
│ Minecraft Vanilla · v1.20.4                 │
│ ●  Reachable   192.168.1.10:25565           │
└──────────────────────────────────────────────┘
```

**States:**
- `Reachable` → green dot, label "Reachable", plain text
- `Unreachable` → red dot, label "Unreachable", clickable (opens diagnostic modal)
- `Unknown` → grey dot, label "Unknown" (probing in progress or no data)

**Data source:** `servers.connectivity_status` column (text: `reachable` | `unreachable` | `unknown`)

---

### 2. Server Details — Connectivity Section

**Location:** `app/src/pages/servers/ServerDetailsPage.jsx` (new `<ConnectivitySection>` component)

**Layout (single card, opt-in):**
```
┌─────────────────────────────────────────────────────────────┐
│ Connectivity                                          [⟳]   │
├─────────────────────────────────────────────────────────────┤
│  Status:   ● Reachable    Mode: Direct                      │
│  Last probe: 2026-06-07 10:24:00 (3 minutes ago)            │
│                                                             │
│  ── Diagnostics ──                                          │
│  Public IP:    47.129.171.64                                │
│  Local IP:     192.168.1.10                                 │
│  Port:         25565 (bound)                                │
│  Firewall:     active (ufw)                                 │
│  CGN suspect:  no                                           │
│  UPnP:         available                                    │
│  Tailscale:    not detected                                 │
│                                                             │
│  ── Auto-fix Attempts (audit log) ──                        │
│  [2026-06-07 10:23:45Z] firewall.open_port: OK              │
│  [2026-06-07 10:23:50Z] upnp.add_mapping: OK                │
│  [2026-06-07 10:24:00Z] connectivity.probe: REACHABLE       │
│                                                             │
│  ── Detected Fallback Options ──                            │
│  · Tailscale  (not installed)                               │
│  · Cloudflare Tunnel (not installed) — experimental         │
│  · Esluce Relay (coming soon)                               │
│                                                             │
│  [ Reachable ]   [ View full audit log ]                    │
└─────────────────────────────────────────────────────────────┘
```

**Components:**
- Status badge (matches list badge)
- Mode pill: `Direct` | `Relay` | `Offline` (Relay placeholder greyed, Relay infrastructure deferred to Phase 68)
- Diagnostics grid (raw facts from agent)
- Audit log list (timestamped, paginated, max 50 visible)
- "Reachable" button → POST `/api/v1/servers/:id/connectivity/probe` (cooldown: 30s per-server, stored in Redis)
- "View full audit log" → modal with full history

**Reachable button cooldown:** 30s per-server; disable button + show countdown when active.

---

### 3. Failure Report Modal

**Trigger:** Server status is `Unreachable` (or user clicks badge)

**Content (per D-13 hybrid failure report):**
```
UNREACHABLE

Primary Cause: CGNAT_DETECTED

Attempts Performed:
  ✓ Port Binding Fixed
  ✓ Firewall Rule Added
  ✓ UPnP Mapping Attempted
  ✗ Reachability Probe Failed

Available Options:
  • Enable Tailscale
  • Configure Manual Port Forwarding
  • Join Esluce Relay Waitlist

Connection Mode: Offline (Awaiting User Action)
```

**Sections:**
1. **Error code** + **root cause** (e.g. `CGNAT_DETECTED`)
2. **Auto-fix attempts** — checklist of what was tried with timestamps
3. **Scenario-specific guidance** — different text for CGNAT / firewall / router / unknown
4. **Detected fallback options** — Tailscale (if `tailscale` CLI detected), Cloudflare (if `cloudflared` detected, marked "experimental"), future Relay (always shown as coming soon)
5. **Persistent banner** until user dismisses

---

### 4. In-App Notifications (D-16)

**Trigger:** Reachable → Unreachable transition

**Notification shape:**
```
┌────────────────────────────────────────────────┐
│ ⚠  Server "My Minecraft" is unreachable       │
│    Auto-fix attempts in progress...            │
│    [View]                                      │
└────────────────────────────────────────────────┘
```

**Delivery:** Reuse existing in-app notification transport (Phase 25 / 39 alert infrastructure).

**Channel config:** Reuse existing `discord_webhook_url` column on server + email transport. Per-server channel opt-in/out.

---

### 5. Dashboard Banner

**Trigger:** Any server in `Unreachable` state

**Persistence:** Stays visible until user clicks [Dismiss] OR all servers return to Reachable

```
┌──────────────────────────────────────────────────────────────────┐
│ ⚠ 1 server is unreachable. [View] [Dismiss]                     │
└──────────────────────────────────────────────────────────────────┘
```

---

## Design Constraints (from D-15)

- **Minimal UI change to existing list pages** — badge only, no other modifications to ServerManagerPage
- **New Connectivity section is opt-in** — collapsed by default; expand to inspect
- **No new design system components** — reuse existing Card / List / Button / Badge / Modal primitives
- **Manual "Reachable" button** in section footer — same probe pipeline as automatic triggers

## Reuse Map

| Existing component | Used in |
|--------------------|---------|
| `Card` | Connectivity section |
| `Badge` | Status indicators |
| `Button` | Reachable / View full audit log |
| `Modal` | Full audit log / Failure report |
| `List` | Audit log entries |
| In-app notification system | Reachability transition alerts |
| `discord_webhook_url` column | Discord alerts |
| Email transport (Phase 25) | Email alerts |

## Out of Scope (Frontend)

- Custom Relay mode selection UI (deferred to Phase 68)
- IPv6-specific UI (deferred until IPv6 reachability becomes user-reported issue)
- Real-time WebSocket push of probe results (poll every 30s is sufficient for MVP)
- Mobile-app-specific layouts (responsive web sufficient per REQUIREMENTS)
