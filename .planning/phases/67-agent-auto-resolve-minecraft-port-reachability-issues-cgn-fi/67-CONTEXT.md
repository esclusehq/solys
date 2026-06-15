# Phase 67: Agent auto-resolve Minecraft port reachability issues (CGN/firewall/Docker port exposure) - Context

**Gathered:** 2026-06-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Make the Esluce agent and backend automatically detect and resolve Minecraft game port reachability issues at the agent node. The system must:

1. **Probe** — from the Esluce backend (public internet) to `(public_ip, game_port)` after every server start, on IP/firewall-change events, and on low-frequency periodic interval
2. **Classify** failures into 4 MVP modes: PORT_NOT_BOUND, HOST_FIREWALL_BLOCKED, CGNAT_DETECTED, UPnP_UNAVAILABLE
3. **Auto-fix** the safe-to-fix modes (Docker binding, server's host firewall port, UPnP mapping) and re-probe
4. **Surface** connectivity status (Reachable / Unreachable / Unknown) on every server with a per-server Connectivity section showing diagnostics, auto-fix attempts, and a full audit log
5. **Alert** users on Reachable → Unreachable transitions via in-app notification, optional email/Discord webhook, and persistent dashboard banner
6. **Detect-only** Tailscale and Cloudflare Tunnel as experimental fallback diagnostics (no auto-install, no auto-config)
7. **Log** every auto-fix action in a per-server audit trail with exact commands and timestamps

**Out of scope for Phase 67** (deferred to follow-up phases):
- The custom Esluce relay infrastructure (`relay.esluce.com`, outbound tunnel, player routing) — agent side only this phase
- Auto-install / auto-configure of Tailscale or Cloudflare Tunnel
- ISP_BLOCKED and PROTOCOL_MISMATCH failure modes (deferred)
- Direct UPnP/Tailscale configuration by the agent (detect only)
</domain>

<decisions>
## Implementation Decisions

### Reachability Probe Mechanism
- **D-01 (Probe model):** **Hybrid** — Backend initiates the actual probe from the public internet (TCP + Minecraft protocol). Agent sends raw local network diagnostics (local_ip, public_ip, port, port_bound, firewall_active, default_gateway, local_subnet, is_cgn_suspect, online_mode) as context for the backend's failure classification. Self-probing from the agent cannot prove the port is reachable from outside.
- **D-02 (Probe triggers):** **On startup + event-triggered + low-frequency periodic** — Backend probes (a) automatically ~5–10s after each successful `server.start` task, (b) on events that change connectivity (public IP change from existing `DnsWatcher`, host firewall change detected by agent heartbeat), and (c) on low-frequency periodic interval (e.g. every 5 min) to catch silent state changes.
- **D-03 (Probe depth):** **Minecraft protocol-aware** — Java Edition: TCP connect → Minecraft handshake packet (`next_state=1`) → status request → parse JSON response (version, players, max, sample). Bedrock Edition: RakNet unconnected ping. Game-aware checks catch the "port is open but wrong service" edge case.
- **D-04 (Diagnostics reporting):** **Raw facts + heuristics from agent, final classification from backend** — Agent sends raw fields (`port_bound`, `firewall_active`, `default_gateway`, `local_subnet`, `detected_public_ip`, `is_cgn_suspect`); backend stores and uses them to render the diagnostic panel and produce the final root_cause label. Keeps agent simple and lets backend correlate with probe results.

### Failure Classification & Auto-Fix
- **D-05 (Auto-fix policy):** **Safe-to-fix gate** — `Failure Detected → Can agent safely fix? → YES → Auto Fix → Re-Probe Backend → Reachable?`. Auto-fixable: Docker port binding misconfig, host firewall port for the specific server, UPnP port mapping on the IGD, and (later) relay/tunnel enablement. Never auto-fix: changes to firewall rules other than the managed server's port, deletion of user firewall rules, aggressive router config beyond standard UPnP, arbitrary root commands, silent 3rd-party installs.
- **D-06 (MVP failure modes — 4):** `PORT_NOT_BOUND` (container not bound to host port), `HOST_FIREWALL_BLOCKED` (host firewall denying inbound), `CGNAT_DETECTED` (no real public IP), `UPnP_UNAVAILABLE` (no IGD or UPnP disabled). `ISP_BLOCKED` and `PROTOCOL_MISMATCH` are deferred. Agent sends raw facts + heuristics; backend produces the final classification shown to the user.
- **D-07 (Detection method):** **Standard `which()` checks + dedicated Rust crates** — `which('ufw') / which('firewalld') / which('iptables') / which('tailscale') / which('cloudflared')` for tool presence; `upnp-rs` crate for UPnP IGD queries; standard library / `local-ip-address` for network interface detection. Linux-first, cross-platform best-effort. No direct iptables netlink — use the standard CLIs.
- **D-08 (UPnP behavior):** **Auto + manual fallback + retry** — Agent uses `upnp-rs` to attempt `AddPortMapping` on the IGD with a short lease (e.g. 1 hour, renewed by background task). If UPnP is not available or the add fails, fall back to a user-side wizard showing the gateway IP, router admin URL, and copy-pasteable port-forwarding instructions. Backend re-probes after either path.
- **D-09 (Host firewall fix):** **Opt-in once at agent install** — Agent auto-fixes only the server's specific port and protocol; every added rule carries an `esluse:<server-id>` comment/tag for later cleanup; user rules are never deleted or modified; Escluse rules are cleaned up automatically when the server is deleted.

### NAT-Traversal Fallback Stack
- **D-10 (Phase scope):** **Agent-side only in Phase 67** — Custom Esluce relay (`relay.esluce.com`, agent outbound tunnel, player→relay routing) is **deferred to a follow-up phase**. The Direct/Relay mode selection logic and the placeholder for future relay status are in scope; the relay infrastructure itself is not.
- **D-11 (Tailscale):** **Detect only** — Agent checks `which('tailscale')` and `tailscale status` (parse JSON). If Tailscale is up on the node, surface the Tailscale IP (`100.x.x.x` or FQDN like `node.tail-xxxx.ts.net`) as a fallback connectable address in the diagnostic panel. No install, no auth key, no per-server Tailscale config. Reuses the existing `enable_tailscale` and `tailscale_auth_key` columns.
- **D-12 (Cloudflare Tunnel):** **Detect + experimental diagnostics only** — Agent checks `which('cloudflared')` and inspects running tunnels. Show tunnel status and endpoint in the dashboard as "Experimental" — **not** a primary fallback path. Rationale: Cloudflare Tunnel is optimized for HTTP/HTTPS; Minecraft Java and Bedrock use TCP/UDP game traffic, which Cloudflare Tunnel handles less predictably. Reuses Phase 51 Cloudflare credentials.
- **D-13 (Last-resort UX):** **Hybrid failure report** — When all auto-fix attempts are exhausted: (1) clear failure report (error code, root cause, diagnostics, auto-fix attempts already performed with audit log); (2) situation-specific guidance per detected scenario (CGNAT, firewall, router, unknown); (3) list of available fallback options (Tailscale if detected, Cloudflare Tunnel if detected, future Esluce Relay); (4) **keep monitoring** — re-run reachability checks periodically and **auto-switch back to Direct Mode** when connectivity is restored.
- **D-14 (Fallback order):** **Direct Mode → UPnP auto-fix → Firewall auto-fix → Tailscale/Cloudflare (detect-only) → Future Esluce Relay**. Cloudflare Tunnel is not in the primary path; it appears only as a detected experimental option in the diagnostic panel.

### User-Facing Diagnostic Surface
- **D-15 (Status display):** **Per-server status badge + Connectivity section** — Add a connectivity badge (Reachable / Unreachable / Unknown) to the Servers list and Server Details page. Add a new "Connectivity" section in Server Details with: current mode (Direct / Relay / Offline), probe results, auto-fix attempts (audit log), detected fallback options, and the manual "Reachable" test button. Minimal UI change to existing list pages; new section is opt-in.
- **D-16 (Notifications):** **Alerts + dashboard banners** — Real-time in-app notification when a server transitions Reachable → Unreachable. Optional email/Discord webhook per existing alert infrastructure (reuses `discord_webhook_url` and any per-server alert config). Persistent dashboard banner shown for any Unreachable server until the user dismisses.
- **D-17 (Action transparency):** **Full visibility + audit log** — Every auto-fix action is logged in a per-server audit trail visible in the Connectivity section. Exact commands and timestamps: e.g. "Added iptables rule for port 25565 with comment esluse:server-uuid @ 2026-06-07T10:23:45Z". Includes both successful actions and attempted-but-failed actions.
- **D-18 (Manual probe trigger):** **"Reachable" button on demand** — User can click "Reachable" in the Connectivity section to immediately trigger a fresh reachability probe. Useful after manual network changes (router reboot, ISP change, etc.). Same probe pipeline as automatic triggers.

### The Agent's Discretion
- Exact `is_cgn_suspect` heuristic (e.g. RFC1918 + gateway IP check, or compare agent's external IP to gateway)
- UPnP mapping lease duration and renewal strategy
- Exact iptables / ufw / firewalld command syntax per distro family
- Periodic probe interval (recommended 5 min, may be different)
- Probe result retention period and history depth in the audit log
- Discord/email alert template wording
- "Reachable" button cooldown (avoid user spamming the backend probe)
- How to extract the Minecraft server version / motd from the probe response (for display)
- Specific shape of the diagnostic panel UI components
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and prior roadmap context
- `.planning/ROADMAP.md` § Phase 67 — Goal statement: "Agent auto-resolve Minecraft port reachability issues (CGN/firewall/Docker port exposure)"
- `.planning/PROJECT.md` — Project overview, core value, multi-node agent model
- `.planning/REQUIREMENTS.md` — None directly added for Phase 67; relates to DEPLOY-01..05 (server lifecycle) and RCON-01..02
- `.planning/STATE.md` — Phase 67 added 2026-06-07

### Existing agent networking code (integration points)
- `src/handlers/runtime.rs:107-174` — `handle_create` builds `HostConfig` with `port_bindings` and `network_mode: "bridge"` — this is the Docker port binding logic the agent already auto-fixes when re-creating containers
- `src/handlers/runtime.rs:120-135` — Current port binding shape: `host_ip: Some("0.0.0.0".to_string())` (IPv4 only), `host_port`, `exposed_ports`. No reachability validation
- `src/handlers/dns.rs` — Existing `CloudflareDnsConfig` struct, `DNS_CONFIG` static, Cloudflare API client. Reused for fallback diagnostics only
- `src/handlers/dns_watch.rs:132-155` — `detect_public_ip()` (uses `api.ipify.org`, `checkip.amazonaws.com`, `icanhazip.com`, `ifconfig.me/ip`). Reused to feed `public_ip` to backend probe
- `src/handlers/mod.rs:118-166` — `execute_single` task dispatch. New task types (e.g. `connectivity.probe`, `firewall.open`, `upnp.add_mapping`) hook in here
- `src/handlers/mod.rs:186-294` — `get_task_config` for per-task timeouts/retries
- `src/main.rs:282-293` — DNS watcher startup pattern; new connectivity watcher follows the same pattern

### Agent protocol and task types
- `agent/agent-core/crates/agent-proto/src/task.rs` — `Task`, `TaskResult`, `TaskPriority`, `TaskStatus`, `TaskError` types. All new task payloads serialize as `serde_json::Value`
- `agent/agent-core/crates/agent-runtime/src/detector.rs` — `RuntimeDetector` (Docker/Podman detection). Existing `bollard` Docker client reused for container inspection

### Backend WebSocket node protocol and persistence
- `api/src/presentation/handlers/node_ws_handler.rs:237-266` — Heartbeat handler (in-memory cache + `node_metrics` table writes). New connectivity reports piggyback on heartbeat OR use a dedicated message type
- `api/src/presentation/handlers/node_ws_handler.rs:268-288` — `CommandResponse` handler (the only path that writes to `servers` today). New `connectivity.report` message type follows this pattern
- `api/src/presentation/ws/node_protocol.rs` — WebSocket message types (Register, Heartbeat, CommandResponse). Add new message types: `ConnectivityReport`, `ConnectivityProbeRequest`, `ConnectivityProbeResult`
- `api/migrations/` — 69+ SQL migration files. New table for `connectivity_audit_log` (per-server, append-only); new columns on `servers` for `connectivity_status` and `connectivity_mode` and `last_probe_at`

### Existing server model and Tailscale scaffolding
- `api/src/domain/server/model.rs:8-31` — OLD `Server` struct used by dashboard endpoint (currently returns `endpoints: serde_json::Value` that is always `[]`)
- `api/src/domain/entities/server.rs:8-75` — NEW `Server` struct with `mc_version`, `host`, `port`, `public_host` (currently bypassed by dashboard handler)
- `migration/20260307000001_add_enhanced_server_features.sql:9-10` — Existing `enable_tailscale BOOLEAN` + `tailscale_auth_key TEXT` columns on `servers` table. Reused for Tailscale detection diagnostics (no write of these from Phase 67)
- `migration/20260501000001_fix_legacy_not_null_columns.sql:14` — `enable_tailscale SET DEFAULT false`

### Prior phase decisions (carried forward)
- `.planning/phases/51-automasi-dns-cloudflare/51-CONTEXT.md` — Phase 51 Cloudflare DNS automation; credentials/zone stored in DB. Reused only for fallback diagnostics
- `.planning/phases/13-fix-minecraft-server-startup-network-error-dns-resolution-fa/13-01-SUMMARY.md` — Established `HostConfig.network_mode = "bridge"` as the canonical fix for Docker DNS resolution
- `.planning/phases/14-fix-minecraft-server-startup-network-error-container-cannot-/14-01-PLAN.md` — Verified bridge network code in `runtime.rs:140`
- `.planning/phases/65-buat-installer-script-auto-install-docker-sebelum-install-so/65-CONTEXT.md` — D-13..D-16: install-time consent, config generation, post-install service enable. Phase 67's "opt-in once at agent install" firewall consent (D-09) follows the same pattern

### Existing alert infrastructure (for D-16 notifications)
- `api/src/domain/billing/webhooks.rs`, `api/src/infrastructure/billing/lemon_squeezy_service.rs` — Webhook emission pattern. Reused for connectivity alerts
- `api/migrations/` (search for `discord_webhook_url`) — Existing `discord_webhook_url` field on server config. Reused for connectivity Discord alerts
- `.planning/phases/52-improve-api-docs/52-03-PLAN.md:771` — Existing error code pattern (`NODE_OFFLINE` etc.). New connectivity error codes follow the same style

### Strategy and debugging context
- `STRATEGI.md:23, 47, 143` — "No port forwarding" is a Tier 1 differentiator. Phase 67 implements the auto-resolution arm of this strategy
- `.planning/debug/server-details-wrong-address-version-status.md:139-145` — Option C mentions `tailscale ip -4` as a Tailscale IP detection pattern. Reused in D-11
- `.planning/debug/minecraft-dns-network-error.md:55-58` — Root cause of "Network is unreachable" was missing `HostConfig.network_mode`. Re-confirmed bridge network in D-04 raw diagnostics shape

### Codebase maps (tech context)
- `.planning/codebase/STACK.md` — Tech stack versions (Bollard 0.18, reqwest 0.12, tokio 1, sqlx 0.7, PostgreSQL 16)
- `.planning/codebase/STRUCTURE.md` — Directory layout: `src/handlers/`, `agent-core/crates/agent-proto/`, `api/src/presentation/`, `api/migrations/`
- `.planning/codebase/INTEGRATIONS.md` — Cloudflare DNS, PostgreSQL, Redis, WebSocket. No existing NAT-traversal integration beyond DNS
- `.planning/codebase/ARCHITECTURE.md` — Microservices with node agents; agent → backend WebSocket outbound
- `.planning/codebase/CONCERNS.md` — Known fragile areas: WebSocket connection management, server executor trait implementations. New connectivity probe should be careful with WS state
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`RuntimeDetector` (`agent/agent-core/crates/agent-runtime/src/detector.rs`)** — Already detects Docker/Podman and exposes a `bollard::Docker` client. Phase 67 reuses this for container inspection (`docker.inspect_container`) when classifying `PORT_NOT_BOUND`.
- **`detect_public_ip()` (`src/handlers/dns_watch.rs:132-155`)** — Fetches public IP from 4 fallback providers. Phase 67 reuses this exact function to populate the `public_ip` field in the diagnostics payload.
- **`CloudflareDnsConfig` + Cloudflare API client (`src/handlers/dns.rs`)** — Existing Cloudflare client. Phase 67 reuses only for the **experimental** Cloudflare Tunnel diagnostics (D-12). Does NOT add Cloudflare Tunnel as a primary fallback.
- **`execute_single` task dispatch (`src/handlers/mod.rs:118-166`)** — Central task dispatcher. New task types hook in here: `connectivity.diagnostics` (agent → backend, raw facts), `connectivity.ack` (backend → agent, on probe result), `firewall.open_port`, `firewall.close_port`, `upnp.add_mapping`, `upnp.remove_mapping`.
- **`audit::log_task_received/completed/failed` (`src/audit.rs`, called from `src/handlers/mod.rs:36-70`)** — Existing per-task audit pattern. Phase 67's per-server `connectivity_audit_log` table mirrors this for auto-fix actions.
- **`DnsWatcher` (`src/handlers/dns_watch.rs:18-80`)** — Background watcher pattern (tokio::spawn with interval ticker). Phase 67's `ConnectivityMonitor` (agent-side, for periodic CGN/firewall checks) follows this same pattern.

### Established Patterns
- **Agent → backend via WebSocket** — Outbound connection, never inbound. New `ConnectivityReport` WS message type follows the existing `Heartbeat` pattern.
- **Per-server / per-task task config** — `get_task_config` in `src/handlers/mod.rs:186` already maps task_type → timeout/retries. New connectivity tasks add entries here.
- **Bollard for container ops** — `bollard::Docker` client used in `runtime.rs:108` for create/start/stop. Reuse for `inspect_container` to read container network bindings.
- **Cloudflare credentials stored in backend DB** — Phase 51's `dns_config` table. Phase 67's experimental Cloudflare Tunnel detection reads from the same source (no new credential storage).
- **Tailscale columns already on `servers` table** — `enable_tailscale BOOLEAN`, `tailscale_auth_key TEXT`. Phase 67 only **reads** these for the diagnostic display (no auto-config).
- **Bridge network mode is the established Docker networking** — `HostConfig.network_mode: Some("bridge".to_string())` in `runtime.rs:140`. Phase 67's `PORT_NOT_BOUND` reclassification logic verifies the container is actually attached to a network and the port binding is non-empty.

### Integration Points
- **New WS message types** in `api/src/presentation/ws/node_protocol.rs`: `ConnectivityReport` (agent → backend, raw diagnostics), `ConnectivityProbeRequest` (backend → agent, trigger immediate local re-check), `ConnectivityProbeResult` (backend → backend, internal). Dispatched in `api/src/presentation/handlers/node_ws_handler.rs` (add handler cases alongside existing `Heartbeat` at lines 237-266).
- **New task types** dispatched in `src/handlers/mod.rs:118-166`: `connectivity.diagnostics`, `firewall.open_port`, `firewall.close_port`, `upnp.add_mapping`, `upnp.remove_mapping`. Each gets a `get_task_config` entry in `src/handlers/mod.rs:186`.
- **New migration** for `connectivity_audit_log` (append-only, indexed by `server_id`, `created_at`).
- **New migration** to add `connectivity_status` (text), `connectivity_mode` (text: direct/relay/offline), `last_probe_at` (timestamptz) to `servers` table.
- **New endpoint** `POST /api/v1/servers/:id/connectivity/probe` (manual "Reachable" button) — reuses the existing per-server handler pattern in `api/src/presentation/handlers/server_handlers.rs`.
- **Frontend additions** in `app/src/pages/servers/`: add connectivity badge to list, new `ConnectivitySection.jsx` component in Server Details page, audit log list with timestamps.
- **Existing alert webhooks** (`discord_webhook_url` + email transport) — Phase 67's Reachable → Unreachable alert reuses the same dispatch path.
- **Bollard client re-usage** — `docker.inspect_container(id, None).await` in the agent to verify container port bindings are actually mapped to host (not just requested).

### Creative Options
- The `is_cgn_suspect` heuristic could be: agent's external IP is in a known CGN range (e.g. 100.64.0.0/10, RFC 6598), OR the agent's external IP differs from the router's apparent public IP, OR the local subnet is a 100.64.0.0/10 (carrier-grade). Backend can refine the final label.
- The `firewall.open_port` task can use `iptables -I INPUT -p tcp --dport <port> -m comment --comment "esluse:<server-id>" -j ACCEPT` (with `ufw`/`firewalld` wrappers) and persist via `iptables-save` / `ufw` built-in persistence. Cleanup on server delete runs the inverse with comment-match (`-m comment --comment "esluse:<server-id>" -D`).
- The `upnp.add_mapping` task can use `upnp-rs` (IGDv2 preferred, IGDv1 fallback) and store the mapping ID for later `upnp.remove_mapping` on cleanup. Background renewal task re-extends the lease every ~50% of the lease duration.
- The periodic connectivity check on the agent side can re-evaluate `firewall_active` and `is_cgn_suspect` only (no full re-probe from the agent — the probe is the backend's job). Agent re-emits `ConnectivityReport` only when the diagnostic fields actually change, to avoid heartbeat bloat.
- The diagnostic panel can show a JSON-like structured view of all raw agent facts + backend classifications + a "Reachable" button — minimal new UI components needed.
</code_context>

<specifics>
## Specific Ideas

### The user wants the system to answer the actual user question
"Apakah teman saya di luar jaringan rumah bisa masuk ke server Minecraft saya?" — the only way to answer this definitively is a probe from outside the user's network. The user explicitly rejected agent self-probe as the primary mechanism for this reason.

### Direct Mode vs Relay Mode (long-term vision)
```
Server Start
  ↓
Reachability Check
  ↓
Port Reachable?
├─ Ya  → Direct Mode (free, no relay infrastructure cost)
└─ Tidak → Relay Mode (deferred to follow-up phase)
```

Phase 67 implements the **Direct Mode** path end-to-end. The mode selection logic and the placeholder for future relay status are in scope; the actual relay infrastructure (`relay.esluce.com` deployment, agent outbound tunnel protocol, player→relay routing) is deferred.

### Failure report UX (illustrative)
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

### Probe cadence (suggested — agent's discretion to tune)
- After successful `server.start`: 5–10s delay, then probe
- After public IP change (existing `DnsWatcher`): immediate probe
- After firewall change (agent detects via `which` + status check delta): immediate probe
- Periodic fallback: every 5 minutes

### Per-server audit log shape
```
[2026-06-07T10:23:45Z] connectivity.diagnostics: public_ip=47.129.171.64 local_ip=192.168.1.10 port=25565 port_bound=true firewall_active=true is_cgn_suspect=false upnp_available=true
[2026-06-07T10:23:50Z] firewall.open_port: iptables -I INPUT -p tcp --dport 25565 -m comment --comment "esluse:server-uuid" -j ACCEPT → OK
[2026-06-07T10:24:00Z] connectivity.probe: backend → 47.129.171.64:25565 → TCP_OK, HANDSHAKE_OK, STATUS_OK (version=1.20.4, players=0/20)
[2026-06-07T10:24:00Z] connectivity.status: REACHABLE (mode=direct)
```

### Agent's opt-in consent at install (extends Phase 65)
The firewall auto-fix consent (D-09) follows Phase 65's install-time consent pattern. The installer adds one new prompt:
```
[?] Allow Escluse to auto-manage host firewall for game server ports? [Y/n]
  → Adds/removes scoped iptables/ufw/firewalld rules with 'esluse:<id>' comments
  → Never modifies or deletes your existing rules
  → Auto-cleanup on server delete
```
Stored as a `firewall_auto_manage: bool` field in the agent config.
</specifics>

<deferred>
## Deferred Ideas

### Custom Esluce Relay (relay.esluce.com)
The full Direct + Relay dual-mode system the user described — including the actual `relay.esluce.com` infrastructure deployment, agent outbound tunnel protocol, and player→relay routing — is a significant new backend infrastructure component. Deferred to a follow-up phase. Phase 67 includes only the agent-side Direct Mode path + the placeholder for future relay mode selection.

### Auto-install / Auto-configure Tailscale
The user explicitly chose "detect only" for Tailscale in Phase 67 (D-11). Auto-install via `tailscale_auth_key` and per-server Tailscale interface binding belong in a follow-up phase.

### Auto-install / Auto-configure Cloudflare Tunnel
Deferred along with Tailscale. Cloudflare Tunnel stays as a detected-but-experimental diagnostic (D-12) — never a primary fallback.

### ISP_BLOCKED and PROTOCOL_MISMATCH failure modes
4 MVP failure modes ship in Phase 67 (D-06). `ISP_BLOCKED` (some ISPs block common game ports like 25565) and `PROTOCOL_MISMATCH` (port open but wrong service listening) are deferred — they can be added incrementally without breaking the existing 4.

### Re-architecting the two competing Server models
`.planning/debug/server-details-wrong-address-version-status.md` documented two `Server` structs (OLD `model.rs` and NEW `entities/server.rs`) that have diverged. This is unrelated to Phase 67's scope and should be addressed in a dedicated refactor phase.

### IPv6 dual-stack port binding
Current `host_ip: Some("0.0.0.0")` is IPv4-only. Adding `::` for IPv6 binding would require also probing via IPv6 — separate concern from CGN/firewall/Docker exposure. Defer until IPv6 reachability becomes a user-reported issue.

### Frontend Address/Version/Status data pipeline fix
`.planning/debug/server-details-wrong-address-version-status.md` documents a separate backend data pipeline issue (agent doesn't read `VERSION` env var, `endpoints` column never populated). Option B/C from that debug doc is a related but separate fix. Defer to a dedicated session.
</deferred>

---

*Phase: 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi*
*Context gathered: 2026-06-07*
