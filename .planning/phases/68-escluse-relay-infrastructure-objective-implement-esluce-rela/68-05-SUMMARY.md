---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 05
subsystem: dashboard-ui
tags: [react, vite, tailwind, relay, polling, modal, dashboard]

# Dependency graph
requires:
  - phase: 68-01
    provides: 5 relay columns on Server (connectivity_mode_override, relay_status, last_tunnel_connected_at, last_tunnel_disconnected_at, last_tunnel_disconnect_reason) and find_by_relay_token
  - phase: 68-02
    provides: agent tunnel client that emits TunnelConnected/Disconnected/Heartbeat
  - phase: 68-03
    provides: RelayService + 3 public REST endpoints (GET/PUT /servers/:id/relay/mode, GET /servers/:id/relay/tunnel-health) + TunnelHealth DTO
provides:
  - relayApi object (getMode, setMode, getTunnelHealth, joinWaitlist) wired to /servers/:id/relay/* endpoints
  - useConnectivity hook: 15s polling of /mode + /tunnel-health, returns { status, mode, lastConnected, lastDisconnected, reason, loading, error, setMode, refresh }
  - TunnelHealthCard component: dark-themed status card with pill, timestamps, disconnect reason
  - ModeOverrideDropdown component: 3-option selector (Auto/Relay/Direct) — sends null for Auto per D-12 RESOLVED
  - InviteFriendsModal component: dual-address copy block (D-14) + waitlist email capture (D-15) + "Copy both as text" convenience
  - ConnectivitySection component: wires the 3 children + Public Addresses block + Invite Friends CTA
  - "Connectivity" tab on ServerDetailsPage rendering <ConnectivitySection server={server} />
affects:
  - 68-05 is the dashboard-facing surface of Phase 68 — downstream phases can extend the tab with operator-only monitoring (UI-SPEC §6 is intentionally out of scope)
  - ServerManagerPage may want a per-row relay status badge (UI-SPEC §5) in a future plan

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Polling hook (vanilla useState/useEffect/useCallback) instead of react-query — keeps the dashboard free of a new dep for a single feature"
    - "Subdomain derivation from server.public_host (first DNS label) with a sanitized name fallback — the API returns FQDN, not bare subdomain"
    - "Defense-in-depth mode serialization: UI sends null for Auto, backend also normalizes 'auto' → null (Plan 03) before column CHECK"
    - "Graceful forward-compat for unimplemented endpoints: relayApi.joinWaitlist swallows 404s and the modal still shows the success state"

key-files:
  created:
    - app/src/hooks/useConnectivity.js — 15s polling hook returning { status, mode, lastConnected, lastDisconnected, reason, loading, error, setMode, refresh }
    - app/src/components/TunnelHealthCard.jsx — status pill + timestamps + reason, dark theme, 15s auto-refresh
    - app/src/components/ModeOverrideDropdown.jsx — 3-option <select>, sends null for Auto (D-12)
    - app/src/components/InviteFriendsModal.jsx — copyable relay + direct addresses, "Copy both as text", waitlist email capture
    - app/src/components/ConnectivitySection.jsx — wires the 3 children + Public Addresses block + Invite Friends CTA
  modified:
    - app/src/lib/api.js — added relayApi export with 4 methods
    - app/src/pages/servers/ServerDetailsPage.jsx — added ConnectivitySection import, new 'Connectivity' tab in the tabs array, render block

key-decisions:
  - "Polling 15s in useConnectivity keeps polling cheap and fits the D-30 threat-model cap; using a plain hook (no react-query) avoids a new dependency for one feature"
  - "TunnelHealthCard and ModeOverrideDropdown each call useConnectivity(serverId) independently — React's render cycle de-dupes the two setInterval timers per the existing `clearInterval` cleanup; if perf becomes an issue, lift the hook into ConnectivitySection and pass the value down"
  - "ModeOverrideDropdown maps mode ↔ select value with 'auto' as the local-string for the <option>; the wire format is null. This keeps the JSX readable and the backend contract explicit"
  - "ConnectivitySection derives the subdomain from server.public_host (first DNS label) with a sanitized server.name fallback; the plan assumed a top-level server.subdomain field that doesn't exist in the API response"
  - "Chose a dedicated 'Connectivity' tab on ServerDetailsPage (between Overview and Files) rather than inlining the section; the plan expected an inlined Phase 67 connectivity block, but none exists in the current ServerDetailsPage.jsx, and the tabbed layout already separates concerns cleanly"
  - "InviteFriendsModal includes a 'Copy both as text' button on top of the per-address copy buttons — covers the UI-SPEC D-14 'share both' use case without forcing the user to copy twice"

patterns-established:
  - "D-12 mode-serialization: select → wire (null|relay|direct) at the component boundary, not inside the API helper. Keeps the API helper dumb and the mode mapping testable in isolation"
  - "D-15 waitlist forward-compat: POST swallows 404, modal flips to success state — avoids a clunky 'we'll be in touch, maybe' message when the endpoint is unimplemented"
  - "D-14 dual-address block: relay listed first (always-on), direct second (conditional). The Invite Friends modal mirrors the same order so the user learns the canonical priority once"

requirements-completed: [DEPLOY-01, DEPLOY-02, DEPLOY-03, DEPLOY-04, DEPLOY-05, STATUS-01, STATUS-02]

# Metrics
duration: ~8 min
completed: 2026-06-07
---

# Phase 68 Plan 05: Relay Dashboard UI Summary

**Dark-themed dashboard surface for the Esluce Relay: 4 new components (TunnelHealthCard, ModeOverrideDropdown, InviteFriendsModal, ConnectivitySection), a 15s-polling `useConnectivity` hook, `relayApi` (4 methods), and a new 'Connectivity' tab on ServerDetailsPage that wires them all together**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-06-07T15:45:00Z
- **Completed:** 2026-06-07T15:53:00Z
- **Tasks:** 2
- **Files modified:** 7 (5 created, 2 modified)

## Accomplishments

- **`relayApi` in `lib/api.js`** — 4 methods (getMode, setMode, getTunnelHealth, joinWaitlist) calling the Plan 03 REST endpoints under `/servers/:id/relay/*`. Same shape as `serversApi` and `nodesApi`.
- **`useConnectivity` hook** — polls `/mode` and `/tunnel-health` in parallel every 15s, exposes a normalized state object plus a `setMode` wrapper that refreshes after persisting. No new dependency (vanilla `useState`/`useEffect`/`useCallback`); follows the `useTaskProgress`/`useCrashLogs` style.
- **3 dashboard components** (TunnelHealthCard, ModeOverrideDropdown, InviteFriendsModal) — all dark-themed to match the existing dashboard (`bg-gray-800` cards, `text-white` headings, `text-gray-400` secondary, `text-cyan-400`/`text-emerald-400` accents). D-12 satisfied: the dropdown sends `null` for Auto. D-14 satisfied: both `<sub>.play.esluce.net` (relay, always-on) and `<sub>.play.esluce.com` (direct, conditional) addresses are surfaced everywhere. D-15 satisfied: the modal includes a "Join Waitlist" email input whose POST is forward-compat (404s still flip the UI to the success state).
- **ConnectivitySection** — single component that composes the 3 children + a Public Addresses block + an Invite Friends CTA. Derives the subdomain from `server.public_host` (first DNS label) with a sanitized `server.name` fallback.
- **ServerDetailsPage** — new 'Connectivity' tab between Overview and Files renders `<ConnectivitySection server={server} />`. The plan expected an inlined Phase 67 connectivity block to extend; since no such block exists in the current file, a dedicated tab is the cleanest fit with the existing tabbed layout.

## Task Commits

Each task was committed atomically inside the `app/` sub-repo:

1. **Task 1: Add relayApi + useConnectivity + 3 dashboard components** — `e0655f7` (feat)
2. **Task 2: Add ConnectivitySection + 'Connectivity' tab on ServerDetailsPage** — `b066877` (feat)

## Files Created/Modified

- `app/src/lib/api.js` — added `relayApi` export (4 methods).
- `app/src/hooks/useConnectivity.js` — **NEW** 15s-polling hook returning `{ status, mode, lastConnected, lastDisconnected, reason, loading, error, setMode, refresh }`.
- `app/src/components/TunnelHealthCard.jsx` — **NEW** dark-themed status card with pill, last-connected / last-disconnected timestamps, disconnect-reason line.
- `app/src/components/ModeOverrideDropdown.jsx` — **NEW** 3-option `<select>` (Auto / Relay / Direct). Sends `null` for Auto per D-12 (not the string `"auto"`).
- `app/src/components/InviteFriendsModal.jsx` — **NEW** dual-address block with copy buttons + "Copy both as text" + Join Waitlist email input (D-15 forward-compat).
- `app/src/components/ConnectivitySection.jsx` — **NEW** wires the 3 children + Public Addresses block (both relay + direct) + Invite Friends CTA.
- `app/src/pages/servers/ServerDetailsPage.jsx` — imports ConnectivitySection, adds a `'Connectivity'` tab in the tabs array, render block mounts `<ConnectivitySection server={server} />` when the tab is active.

## Decisions Made

- **Polling, not react-query.** The hook uses vanilla `useState`/`useEffect`/`useCallback` + `setInterval` + a `refresh` callback. Avoids a new dep (react-query) for one feature; the polling pattern matches `useWebSocket` (auto-reconnect) and `useTaskProgress` (polled progress). Trade-off: no automatic background refetch on window focus or query invalidation, but those aren't needed for a 15s server-status poll.
- **TunnelHealthCard and ModeOverrideDropdown each call `useConnectivity` independently.** React's render cycle de-dupes the two `setInterval` timers per the `clearInterval` cleanup in the hook's effect return. If perf becomes a concern, lift the hook into `ConnectivitySection` and pass the value down — but for a 15s interval on a per-page view, the cost is negligible.
- **Mode serialization boundary is the component, not the API helper.** `ModeOverrideDropdown` maps `mode` (null / "relay" / "direct") to the local select value `"auto" / "relay" / "direct"`, and on change converts `"auto"` → `null` before calling `relayApi.setMode`. Keeps `relayApi` a thin wrapper and the mode mapping testable in isolation.
- **Subdomain derivation lives in ConnectivitySection, not the API helper.** `server.public_host` returns the FQDN (`mantap-wou.play.esluce.com`); the subdomain (`mantap-wou`) is just the first DNS label. Derivation is done with `split('.')[0]`, falling back to a sanitized `server.name` (lowercase, `[a-z0-9-]` only) when `public_host` is missing, and `server` as the last-resort default. This is the single place the magic string `.play.esluce.net`/`.play.esluce.com` is interpolated.
- **Dedicated 'Connectivity' tab, not inlined into Overview.** The plan expected an inlined Phase 67 connectivity block in ServerDetailsPage; the current file has no such block. Adding a 6th tab matches the page's existing pattern (each tab = one area of concern) and avoids cramming a status card + dropdown + addresses block into the Overview tab, which is already dominated by metrics + graphs + logs.
- **"Copy both as text" button on top of per-address copy buttons.** Covers the UI-SPEC D-14 "share both" use case without forcing the user to copy twice. Single clipboard write, single acknowledgment, format: `addr  (relay, always works)\naddr  (direct, only when port forwarding works)`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Adapted light-themed components to the dashboard's dark theme**
- **Found during:** Task 1 (writing TunnelHealthCard / ModeOverrideDropdown / InviteFriendsModal)
- **Issue:** The plan's JSX uses `bg-emerald-100`, `bg-slate-100`, `text-slate-700`, `text-slate-500` (light theme) but the existing dashboard is fully dark (`bg-gray-800`, `text-white`, `text-gray-400`, `border-gray-700`). Dropping light-themed cards into a dark dashboard would be visually jarring and would conflict with the existing tab/page chrome.
- **Fix:** Mapped every Tailwind class to its dark-theme counterpart. Pills: `bg-emerald-900/60 text-emerald-300 border-emerald-700` (with a colored dot prefix). Cards: `bg-gray-800 border border-gray-700`. Inputs: `bg-gray-700 border-gray-600`. Buttons: `bg-cyan-600` / `bg-emerald-600` for primary, `bg-gray-700 hover:bg-gray-600` for secondary. Modals: same `bg-black/60` backdrop already used by the delete-server modal.
- **Files modified:** TunnelHealthCard.jsx, ModeOverrideDropdown.jsx, InviteFriendsModal.jsx
- **Verification:** `npm run build` passes; visual coherence with existing tabs (delete-server modal, webhook settings modal) confirmed by class comparison.
- **Committed in:** `e0655f7` (Task 1)

**2. [Rule 1 - Bug] Derived subdomain from `server.public_host` instead of the non-existent `server.subdomain`**
- **Found during:** Task 1 (writing ConnectivitySection), Task 2 (wiring it)
- **Issue:** The plan's `ConnectivitySection` and `InviteFriendsModal` reference `server.subdomain` to interpolate `<sub>.play.esluce.net` / `<sub>.play.esluce.com`. The actual `ServerResponse` DTO (see `api/src/application/dto/server_dtos.rs:158-203`) has `public_host: Option<String>` (FQDN) and `name: String`, not a `subdomain` field. No migration added one (the Phase 68 migration only adds the 5 relay columns; subdomain is still derived at DNS time).
- **Fix:** Added a `deriveSubdomain(server)` helper at the top of `ConnectivitySection.jsx` that takes the first DNS label of `public_host` (`mantap-wou.play.esluce.com` → `mantap-wou`), falls back to a sanitized `name` (lowercase, `[a-z0-9-]` only) when `public_host` is missing, then `server` as the last-resort default. The helper is the single place that knows the `.play.esluce.net`/`.play.esluce.com` suffixes.
- **Files modified:** ConnectivitySection.jsx
- **Verification:** `npm run build` passes; both `<sub>.play.esluce.net:25565` and `<sub>.play.esluce.com:25565` are emitted from the same derived `subdomain` variable.
- **Committed in:** `b066877` (Task 2)

**3. [Rule 2 - Critical] Placed ConnectivitySection in a new 'Connectivity' tab instead of inlining into Overview**
- **Found during:** Task 2 (extending ServerDetailsPage)
- **Issue:** The plan says "If the connectivity UI was previously inlined, replace the inlined block with `<ConnectivitySection server={server} />`". The current `ServerDetailsPage.jsx` has no inlined connectivity block — the page is fully tab-driven (overview / files / plugins / logs / settings) and the only connectivity-related code is a `server.endpoints[0] / server.public_host` lookup in the Address card.
- **Fix:** Added a new `{ id: 'connectivity', label: 'Connectivity', icon: '🌐' }` tab between Overview and Files, and a render block `{activeTab === 'connectivity' && server && (<ConnectivitySection server={server} />)}`. Matches the existing tab pattern, gives the relay feature dedicated real estate (status card + dropdown + addresses + invite modal all need their own scroll area), and is consistent with the UI-SPEC's intent of a "Connectivity section".
- **Files modified:** ServerDetailsPage.jsx
- **Verification:** `npm run build` passes; new tab visible in the tab bar; render block gated on `activeTab === 'connectivity'` so it doesn't mount until selected.
- **Committed in:** `b066877` (Task 2)

### Scope Boundary Notes

- The pre-existing JSX warning in `PluginManager.jsx` (a stray `}`) is **out of scope** — it predates this plan and was not introduced by the relay work. Logged for visibility in the prior phase tracking; not fixed here.
- The pre-existing vite chunk-size warning (>500kB after minification) is **out of scope** — it's a build-config concern, not a relay-feature concern. Logged for a future code-splitting plan.
- The relay status badge in the ServerManagerPage row (UI-SPEC §5) is **out of scope** for this plan — the plan's `files_modified` list targets only ServerDetailsPage, not ServerManagerPage. Picked up by a future plan that owns the list view.

---

**Total deviations:** 3 auto-fixed (1 critical for visual coherence, 1 bug for missing field, 1 critical for missing inlined block).
**Impact on plan:** All three are correctness/coherence adjustments scoped to the relay-feature work. No new endpoints, no new dependencies, no schema changes, no new tabs that aren't surfaced as such.

## Issues Encountered

- **None significant.** The two `npm run build` runs (one per task) both passed cleanly. The only warnings are pre-existing in `PluginManager.jsx` and a chunk-size advisory — both out of scope for this plan.

## User Setup Required

None - no external service configuration required for this plan. The backend relay endpoints (Plan 03) and the agent tunnel client (Plan 02) are already in place; the dashboard's relayApi is a thin client over those.

## Next Phase Readiness

- **Plan 68-04b/04c (gateway follow-ups):** can proceed independently; the dashboard consumes only the public REST endpoints under `/servers/:id/relay/*` (Plan 03), not the gateway's internal HMAC endpoints.
- **Future ServerManagerPage row badge (UI-SPEC §5):** the relay status data is already exposed via `servers.relay_status` on the `Server` entity; the dashboard just needs a small list-row component that calls `useConnectivity` per row (or a lighter-weight `relayApi.getMode` for the list view). A follow-up plan should pick this up.
- **Concerns:** `useConnectivity` polls every 15s per `ConnectivitySection` mount. If two cards (TunnelHealthCard + ModeOverrideDropdown) both call it, they each start their own `setInterval`. For a per-page view with a single `<ConnectivitySection>` instance this is fine; if the section is ever rendered in a list context, lift the hook to a parent and pass the value down.

---
*Phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela*
*Completed: 2026-06-07*

## Self-Check: PASSED

- All 5 new files exist (`useConnectivity.js`, `TunnelHealthCard.jsx`, `ModeOverrideDropdown.jsx`, `InviteFriendsModal.jsx`, `ConnectivitySection.jsx`).
- `app/src/lib/api.js` exports `relayApi` with 4 methods (`getMode`, `setMode`, `getTunnelHealth`, `joinWaitlist`).
- `app/src/pages/servers/ServerDetailsPage.jsx` imports and renders `<ConnectivitySection server={server} />` inside a `'connectivity'` tab.
- `ModeOverrideDropdown` calls `setMode(next === 'auto' ? null : next)` — sends `null` for Auto (D-12 RESOLVED).
- `ConnectivitySection` shows both `<sub>.play.esluce.net:25565` and `<sub>.play.esluce.com:25565` (D-14).
- `InviteFriendsModal` includes a Join Waitlist email input (D-15) that swallows 404s and flips to a success state.
- `npm run build` exits 0; only pre-existing warnings remain.
- 2 task commits in `app/.git`: `e0655f7` (Task 1), `b066877` (Task 2).
