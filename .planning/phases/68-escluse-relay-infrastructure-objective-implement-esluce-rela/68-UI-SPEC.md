# Phase 68: Escluse Relay Infrastructure - UI Design Contract

**Generated:** 2026-06-07
**Source:** CONTEXT.md D-12, D-14, D-15, D-22, D-23, D-24 + ROADMAP requirement 6 (Dashboard Integration)
**Mode:** inline-synth (yolo mode + execute-end-to-end; normally gsd-ui-researcher is spawned)

---

## Frontend Additions

### 1. Extend Phase 67 ConnectivitySection — Tunnel Health Block

**Location:** `app/src/pages/servers/ServerDetailsPage.jsx` (extend existing `<ConnectivitySection>`)

**New block to append (after existing "Auto-fix Attempts" block):**
```
┌─────────────────────────────────────────────────────────────┐
│  ── Relay Tunnel ──                                          │
│  Status:          ● Connected (since 2026-06-07 10:24:00)    │
│  Latency:         23ms (relay round-trip)                    │
│  Last heartbeat:  4 seconds ago                              │
│  Connection:      2h 14m                                     │
│  Relay endpoint:  ap-southeast-1 (sg-1)                     │
│                                                             │
│  ── Public Addresses ──                                      │
│  ● Always-on (relay):                                        │
│      mc.play.esluce.net    [Copy]  [QR]                      │
│                                                             │
│  ● Direct (only when probe-verified):                        │
│      mc.play.esluce.com    [Copy]  [QR]                      │
│                                                             │
│  ── Mode Override ──                                         │
│  ● Auto (let Esluce pick the best)                           │
│  ○ Force Relay (always use relay)                            │
│  ○ Force Direct (only use direct, fail if unavailable)       │
│                                                             │
│  [ Reachable ]   [ Show full audit log ]                     │
└─────────────────────────────────────────────────────────────┘
```

**States:**
- **Connected** (green dot) — tunnel up, heartbeat fresh (<30s)
- **Connecting** (yellow dot) — agent is dialing relay, no tunnel yet
- **Disconnected** (red dot) — last heartbeat > 30s OR explicit disconnect

**Data source:** `servers.relay_status` (text: connected/connecting/disconnected), `servers.last_tunnel_connected_at` (timestamptz), tunnel health from `connectivity_report` WebSocket messages (Phase 67 extension)

---

### 2. Reachable Button — Mode Aware Behavior

**Same button as Phase 67**, but behavior adapts to current mode:
- If mode=Auto: triggers full re-probe (Direct probe + tunnel health check)
- If mode=Force Relay: triggers tunnel health check only (Direct probe skipped)
- If mode=Force Direct: triggers Direct probe only (tunnel check skipped if not yet established)

**Cooldown:** 30s per-server (reuses Phase 67 Redis key `probe:cooldown:<server_id>`)

---

### 3. Mode Override Dropdown

**Trigger:** User clicks the "Mode Override" radio group in the connectivity section.

**Behavior:**
- POST `/api/v1/servers/:id/connectivity/mode-override` with body `{ "mode": "auto" | "force_relay" | "force_direct" }`
- Backend persists to `servers.connectivity_mode_override` column
- Backend sends `ModeOverrideChange` WebSocket message to the agent (new `NodeMessage` variant from D-13)
- Agent re-evaluates mode on receipt, may flip immediately
- UI updates the radio group optimistically; rolls back on API error

**Default value:** `auto` (Phase 68 default per D-12). Pinned overrides stored as text in DB.

---

### 4. Invite Friends Modal

**Trigger:** User clicks "Invite friends to <server-name>" button on Server Details page (new button in Server Details header).

**Content (single card, opt-in):**
```
┌─────────────────────────────────────────────────────────────┐
│  Invite friends to "My Minecraft"                       [×] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Always-on address (works for everyone, all network types): │
│                                                             │
│  ┌─────────────────────────────────────────┐ ┌──┐          │
│  │ mc.play.esluce.net                      │ │QR│          │
│  └─────────────────────────────────────────┘ └──┘          │
│                                                             │
│  Direct address (only shown when port forwarding works):    │
│                                                             │
│  ┌─────────────────────────────────────────┐ ┌──┐          │
│  │ mc.play.esluce.com                      │ │QR│          │
│  └─────────────────────────────────────────┘ └──┘          │
│                                                             │
│  Both addresses route to the same server. Share whichever   │
│  your friends can connect to — relay works in all cases.    │
│                                                             │
│  [ Copy both as text ]  [ Email invite ]                    │
└─────────────────────────────────────────────────────────────┘
```

**Behavior:**
- "Copy" buttons: copy just the address
- "QR" button: show QR code modal (renders the address as a Minecraft-compatible `minecraft://` URL or plain text)
- "Copy both as text": copies `mc.play.esluce.net  (relay, always works)\nmc.play.esluce.com  (direct, only when port forwarding works)`
- "Email invite": opens mailto: with both addresses pre-filled

**Direct address only shown when:** `servers.connectivity_mode_override = force_direct` OR `(mode = auto AND direct probe passed AND <50ms latency penalty)` per D-12 + D-14.

---

### 5. Servers List — Per-Server Relay Status Badge

**Location:** `app/src/pages/servers/ServerManagerPage.jsx` (server list row, next to Phase 67 reachability badge)

**Visual addition to existing Phase 67 row:**
```
┌──────────────────────────────────────────────────┐
│ Minecraft Vanilla · v1.20.4                     │
│ ●  Reachable   ●  Relay   192.168.1.10:25565    │
└──────────────────────────────────────────────────┘
```

**States:**
- **Relay** (blue dot) — connected via relay
- **Direct** (green dot) — connected via Direct Mode
- **Offline** (red dot) — neither path working
- **Switching** (yellow dot) — mode flip in progress

**Hover:** Tooltip shows "Mode: Relay (since 2026-06-07 10:24:00) · Latency: 23ms"

**Click:** Opens the failure report modal (Phase 67) or the connectivity section in Server Details.

---

### 6. Monitoring Dashboard Tab (NEW — operator-only view)

**Location:** `app/src/admin/relay-monitoring.jsx` (new admin page)

**Content (operator-only, gated by `relay:view` permission):**
```
┌─────────────────────────────────────────────────────────────┐
│  Relay Status:    ● Healthy   (since 2026-06-07 10:24:00)   │
│  Active tunnels:  142                                        │
│  Bandwidth:        234 Mbps in  / 189 Mbps out               │
│  Reconnects (5m):  3                                         │
│  Relay latency:   p50 23ms  p95 47ms  p99 78ms              │
│  Mode distribution:  Relay 89%  Direct 9%  Offline 2%       │
│                                                             │
│  Recent errors:                                              │
│  [10:24:01] handshake_failure: nonce=... node=node-uuid     │
│  [10:23:45] rejected_lookup: server_id=server-uuid          │
│  [10:22:10] timeout: tunnel=tunnel-uuid                      │
│                                                             │
│  [ Open Prometheus dashboard ↗ ]                             │
└─────────────────────────────────────────────────────────────┘
```

**Data source:** `relay_metrics` Prometheus endpoint via backend aggregator.

**Reuses:** Existing admin layout + monitoring card components from Phase 39 / 40.

---

## Design Constraints (from D-14, D-15, D-23)

- **Both addresses shown when applicable** — no hiding the direct address when it's working. Users should know the difference.
- **Primary = relay** — "Copy join address" defaults to the relay one (always works). Direct is secondary.
- **Mode override is opt-in** — default is "Auto". User must explicitly pin to override.
- **No new design system components** — reuse existing Card / List / Button / Badge / Modal / RadioGroup / QRCode primitives
- **Relay status badge is added to existing row** — minimal change to ServerManagerPage
- **Monitoring dashboard is admin-only** — regular users see per-server connectivity only

## Reuse Map

| Existing component | Used in |
|--------------------|---------|
| `Card` (Phase 67 ConnectivitySection) | Tunnel Health block |
| `Badge` (Phase 67 status) | Relay status indicator |
| `RadioGroup` (existing in `app/src/components/forms/`) | Mode override |
| `Button` | Reachable / Copy / QR |
| `Modal` | Invite Friends / QR Code |
| `QRCode` (existing in `app/src/components/qr/`) | Address QR codes |
| In-app notification system (Phase 25) | Mode change confirmations |
| `discord_webhook_url` column (Phase 25) | Per-server relay alerts (D-23) |
| Admin dashboard layout (Phase 39/40) | Monitoring tab |

## Out of Scope (Frontend)

- Custom Relay mode selection UI for end users (operator-only monitoring, per D-15 free tier)
- Bedrock Edition address format (deferred with Bedrock support, Phase 68 D-16 = Java only)
- Per-region relay selection UI (single region, D-17)
- WebRTC fallback for ultra-low-latency (architecturally different, deferred)
- Player-side launcher mod showing both addresses (Pitfall 9 v2)
