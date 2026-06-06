# Phase 67: Agent auto-resolve Minecraft port reachability issues (CGN/firewall/Docker port exposure) - Research

**Researched:** 2026-06-07
**Domain:** Network reachability diagnosis & remediation, hybrid agent/backend probe model, Linux firewall/UPnP/NAT-traversal
**Confidence:** MEDIUM-HIGH (existing codebase is well-mapped; protocol & tooling knowledge is verified; few items remain to validate at build time)

## Summary

Phase 67 builds a hybrid reachability-probe pipeline: the agent emits raw local-network facts (public IP, local IP, gateway, firewall presence, CGN heuristics, container port bindings), and the backend performs a real TCP + Minecraft-protocol probe from the public internet. Probes are triggered after `server.start`, on `DnsWatcher` IP-change events, on agent-detected firewall changes, and on a low-frequency periodic interval. The backend classifies probe failures into 4 MVP modes (`PORT_NOT_BOUND`, `HOST_FIREWALL_BLOCKED`, `CGNAT_DETECTED`, `UPnP_UNAVAILABLE`) and dispatches safe-to-fix tasks (`firewall.open_port`, `upnp.add_mapping`, re-create container with explicit `port_bindings`) to the agent. The agent logs every action to a per-server audit log via a new `connectivity_audit_log` table (mirroring `audit_logs` shape). The frontend shows a per-server status badge (Reachable / Unreachable / Unknown) plus a new "Connectivity" section in Server Details with a manual "Reachable" trigger, audit log, and a hybrid failure report (error + scenario-specific guidance + detected fallback options like Tailscale/Cloudflare/Relay).

**Primary recommendation:** Treat the agent as a **diagnostic reporter + safe-action executor** and the backend as the **probe origin + classifier + UX orchestrator**. Build the WebSocket protocol extensions and the 4-modes auto-fix pipeline end-to-end first; add a manual "Reachable" button and audit-log UI in the same pass so users see results before Phase 68 (Relay) lands.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01 (Probe model):** Hybrid — backend probes from public internet; agent sends raw local diagnostics
- **D-02 (Probe triggers):** On startup + event-triggered + low-frequency periodic
- **D-03 (Probe depth):** Minecraft protocol-aware — Java Edition handshake + status request; Bedrock Edition RakNet unconnected ping
- **D-04 (Diagnostics reporting):** Raw facts + heuristics from agent, final classification from backend
- **D-05 (Auto-fix policy):** Safe-to-fix gate (Docker binding, host firewall for the server's port, UPnP mapping; never user rules or arbitrary root)
- **D-06 (MVP failure modes — 4):** `PORT_NOT_BOUND`, `HOST_FIREWALL_BLOCKED`, `CGNAT_DETECTED`, `UPnP_UNAVAILABLE` (no `ISP_BLOCKED` / `PROTOCOL_MISMATCH`)
- **D-07 (Detection method):** `which()` checks + dedicated Rust crates (`upnp-rs`); Linux-first, cross-platform best-effort
- **D-08 (UPnP behavior):** Auto + manual fallback + retry; short lease, background renewal; user wizard on failure
- **D-09 (Host firewall fix):** Opt-in once at agent install; scoped rules with `esluse:<server-id>` comment; cleanup on server delete
- **D-10 (Phase scope):** Agent-side only; relay infrastructure deferred
- **D-11 (Tailscale):** Detect only; reuse `enable_tailscale` / `tailscale_auth_key` columns
- **D-12 (Cloudflare Tunnel):** Detect + experimental diagnostic only; reuse Phase 51 credentials
- **D-13 (Last-resort UX):** Hybrid failure report + scenario guidance + detected fallback options + keep monitoring
- **D-14 (Fallback order):** Direct → UPnP auto-fix → Firewall auto-fix → Tailscale/Cloudflare (detect-only) → future Esluce Relay
- **D-15 (Status display):** Per-server badge + new "Connectivity" section in Server Details
- **D-16 (Notifications):** Real-time in-app alerts + optional email/Discord webhook + persistent dashboard banner
- **D-17 (Action transparency):** Full visibility + audit log (exact command + timestamp)
- **D-18 (Manual probe trigger):** "Reachable" button on demand

### the agent's Discretion
- Exact `is_cgn_suspect` heuristic (RFC 1918 + gateway IP check, or external vs gateway comparison)
- UPnP mapping lease duration and renewal strategy
- Exact iptables/ufw/firewalld command syntax per distro family
- Periodic probe interval (recommended 5 min)
- Probe result retention period and history depth
- Discord/email alert template wording
- "Reachable" button cooldown
- How to extract Minecraft version/MOTD from probe response
- Specific shape of diagnostic panel UI components

### Deferred Ideas (OUT OF SCOPE)
- Custom Esluce Relay (`relay.esluce.com`, outbound tunnel, player routing) — Phase 68
- Auto-install / auto-configure Tailscale or Cloudflare Tunnel
- `ISP_BLOCKED` and `PROTOCOL_MISMATCH` failure modes
- Re-architecting the two competing `Server` model structs
- IPv6 dual-stack port binding
- Frontend Address/Version/Status pipeline fix (separate `debug` doc)

### Phase Requirements
No IDs mapped by orchestrator (`phase_req_ids: null`). Tangentially related requirements from REQUIREMENTS.md: `DEPLOY-01..05` (server lifecycle — already complete, reused for post-start probe trigger), `RCON-01..02` (used to verify the running game server is the right service when classifying `PROTOCOL_MISMATCH` — deferred, but the probe pipeline design accommodates it).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Reachability probe (TCP + Minecraft protocol) | API / Backend | — | Must originate from the public internet to prove inbound reachability (D-01). Self-probe from the agent node would only prove the loopback works. |
| Raw local diagnostics emission (public_ip, local_ip, port_bound, firewall_active, default_gateway, is_cgn_suspect) | Agent | — | Agent has authoritative knowledge of its own network state, container bindings, and local subnet (D-04). |
| Failure classification (`PORT_NOT_BOUND` / `HOST_FIREWALL_BLOCKED` / `CGNAT_DETECTED` / `UPnP_UNAVAILABLE`) | API / Backend | — | Backend correlates probe results with raw agent diagnostics; centralising here keeps the agent dumb and the rule engine upgradable (D-04). |
| Safe-to-fix actions (Docker port binding fix, host firewall rule, UPnP `AddPortMapping`) | Agent | — | Action runs on the host; agent executes the system command and reports back. Auto-fixable list is hard-coded in agent dispatch (D-05). |
| Audit log persistence (`connectivity_audit_log`) | API / Backend | — | Single source of truth in PostgreSQL; agent pushes each action via WebSocket `ConnectivityReport` (mirrors `CrashReport` pattern at `node_ws_handler.rs:397`). |
| Notification dispatch (in-app + email + Discord webhook) | API / Backend | — | Reuses existing webhook emission (`api/src/domain/billing/webhooks.rs` + per-server `discord_webhook_url`). |
| Tailscale / Cloudflare Tunnel detection | Agent | — | Read-only `which()` + `tailscale status` JSON / `cloudflared tunnel info`. No install, no auth (D-11, D-12). |
| Connectivity badge + Connectivity section UI | Frontend / Browser | — | Renders data from backend endpoints; no client-side state authority. |
| Probe cadence scheduling (after server.start, periodic, event-triggered) | API / Backend | Agent | Backend schedules; agent re-emits `ConnectivityReport` only on diagnostic change to avoid heartbeat bloat. |

## Standard Stack

### Core (verify versions before pinning)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `bollard` | 0.18 (already in `Cargo.toml` and `Cargo.lock` line 400) | Docker `inspect_container` for `PORT_NOT_BOUND` reclassification | Same client used by `runtime.rs:108` for `create_container` — reuse, no new dep |
| `upnp-rs` | 0.2.0 (latest published, [crates.io/upnp-rs](https://libraries.io/cargo/upnp-rs), [github.com/johnstonskj/rust-upnp](https://github.com/johnstonskj/rust-upnp)) | UPnP IGD discovery + `AddPortMapping` for `UPnP_UNAVAILABLE` auto-fix path | Pinned by CONTEXT D-07; alternative `igd` crate is more popular but unmaintained; `upnp-rs` exposes SSDP discovery + IGD control |
| `which` | 6 (already transitive via `agent-runtime/detector.rs`) | `which('ufw')`, `which('firewalld')`, `which('iptables')`, `which('tailscale')`, `which('cloudflared')` | Already used by `detector.rs:41` for `which("docker")` — no new dep |
| `local-ip-address` | 1.x ([crates.io/local-ip-address](https://crates.io/crates/local-ip-address)) | Discover the host's primary local IPv4 + default gateway for `is_cgn_suspect` heuristic | Lightweight, sync + async API, cross-platform; `get_local_ip_address` returns default-route local IP |
| `reqwest` | 0.12 (already in `Cargo.toml`) | `Cloudflare API` for Phase 51 reused for Cloudflare Tunnel detection | Reuses existing dependency |
| `serde` / `serde_json` | 1 (already) | WebSocket `ConnectivityReport` / `ConnectivityProbeRequest` payload | Mandatory |
| `tokio` | 1 (already, features = full) | Async task dispatch + `ConnectivityMonitor` periodic loop | Mandatory |
| `chrono` | 0.4 (already) | Audit-log timestamp + `last_probe_at` | Already used by `dns_watch.rs:8` |
| `uuid` | 1 (already) | Server / node / audit-log IDs | Already used everywhere |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing` | 0.1 (already) | `info!` / `warn!` for every auto-fix action and probe result | Already used in `dns_watch.rs:7` — mirror pattern |
| `iproute2` (`ip route` via `Command`) | OS dep | `ip route show default` to read default gateway | Linux-first; on Windows use `route print` (defer cross-platform read to follow-up if needed) |
| `iptables` (via shell) | OS dep | `iptables -I INPUT -p tcp --dport <port> -m comment --comment "esluse:<server-id>" -j ACCEPT` | Linux host firewall; use `nft` symlink/iptables-nft compatibility shim is now the default on Ubuntu 22.04+ / Debian 12+ / RHEL 8+ |
| `ufw` (via shell) | OS dep | `ufw allow <port>/tcp comment "esluse:<server-id>"` | Ubuntu/Debian convenience wrapper |
| `firewalld` (via shell) | OS dep | `firewall-cmd --zone=public --add-port=<port>/tcp` | RHEL/Fedora convenience wrapper |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `upnp-rs` | `igd` crate (popular but mostly stale) | `upnp-rs` is actively maintained (latest 0.2.x per libraries.io); `igd` is what most P2P apps use but its async story is weaker. CONTEXT D-07 explicitly chose `upnp-rs`. |
| `iptables` shell-out | Direct netlink via `rust-netlink` or `nfnetlink` | Direct netlink skips the iptables-nft shim, but loses the wrapper-tool layer (ufw/firewalld users would bypass their stack). Shell-out is the lowest-common-denominator that works on every modern Linux with the comment-match cleanup pattern. |
| Custom Minecraft protocol parser | `minecraft-server-status` crate, `mcping` crate | No maintained, idiomatic, async-ready Rust crate for full SLP. Implementing the 5-field handshake + status response is ~80 lines of async TCP — see Code Examples. Avoid the dependency. |
| Custom RakNet unconnected ping | `bedrock-crustaceans/raknet` (incomplete), `sandertv/go-raknet` (Go) | No production-quality Rust RakNet client crate. Building just the **unconnected ping** (10 bytes) + **unconnected pong parser** (~120 bytes) is the only minimum needed for Phase 67 — see Code Examples. |
| `local-ip-address` crate | `get_if_addrs` crate | Both are roughly equivalent; `local-ip-address` has a `find()` async API specifically for "the default-route local IP" and includes gateway lookup on Linux. |
| `nft` direct | `iptables` shell-out | `nft` is the modern backend (default on Ubuntu 22.04+, RHEL 8+), but `iptables` syntax via `iptables-nft` shim is what 88.4% of home/SMB Linux boxes run in 2026 (per commandlinux.com 2026 survey). Writing only the iptables path keeps the implementation portable; the shim translates to nftables on the kernel side. |

### Installation

Agent side (add to root `Cargo.toml` dependencies block, alongside the existing `bollard`, `which`, `reqwest`, `tokio`):

```toml
# Network diagnostics for Phase 67 connectivity probe
upnp-rs        = "0.2"
local-ip-address = "1"
```

No backend (api/Cargo.toml) additions required — the probe pipeline uses existing `reqwest`, `sqlx`, `serde`, and `tokio`.

> **Version verification:** Run `npm view upnp-rs version` and `npm view local-ip-address version` (or `cargo search upnp-rs`) before locking. CONTEXT.md does not pin exact versions; `upnp-rs 0.2.0` is the latest verified major; `local-ip-address 1.x` is the latest family. Both are stable and have not had breaking releases recently.

## Architecture Patterns

### System Architecture Diagram

```
                                      Public Internet
                                            │
                                            ▼
   ┌──────────────────────────────────────────────────────────────────┐
   │  Escluse API / Backend (api/)                                    │
   │  ┌──────────────────────────────────────────────────────────┐    │
   │  │  Connectivity Probe Scheduler (new)                       │    │
   │  │   • Triggered by: server.start ack, IP-change event,     │    │
   │  │     agent heartbeat delta, manual "/probe" button,       │    │
   │  │     low-freq periodic (5 min)                            │    │
   │  └────────┬─────────────────────────────────────┬───────────┘    │
   │           │ TCP+SLP probe (Java)                │ RakNet ping    │
   │           │ OR UDP+RakNet ping (Bedrock)        │ (Bedrock)      │
   │           ▼                                     ▼                │
   │  ┌──────────────────────────────────────────────────────────┐    │
   │  │  Probe Result Classifier (new)                            │    │
   │  │   correlate(probe_outcome, agent_diagnostics)             │    │
   │  │     → 4 MVP labels: PORT_NOT_BOUND, HOST_FIREWALL_BLOCKED,│    │
   │  │       CGNAT_DETECTED, UPnP_UNAVAILABLE                   │    │
   │  │     → render status (Reachable / Unreachable / Unknown)   │    │
   │  └────────┬─────────────────────────────────────────────────┘    │
   │           │ store                                                  │
   │           ▼                                                        │
   │  ┌─────────────────────────┐  ┌──────────────────────────────┐    │
   │  │  servers                │  │  connectivity_audit_log (new) │    │
   │  │  +connectivity_status   │  │  per-server, append-only,     │    │
   │  │  +connectivity_mode     │  │  indexed by (server_id, ts)   │    │
   │  │  +last_probe_at         │  └──────────────────────────────┘    │
   │  └─────────────────────────┘                                       │
   │  ┌─────────────────────────┐  ┌──────────────────────────────┐    │
   │  │  Auto-Fix Dispatcher    │  │  Notification Service         │    │
   │  │  (new)                  │──▶  in-app + Discord/email      │    │
   │  │  safe-to-fix gate per   │  │  on Reachable → Unreachable   │    │
   │  │  D-05 (Docker, FW, UPnP)│  └──────────────────────────────┘    │
   │  └─────────────────────────┘                                       │
   │           │                                                        │
   └───────────┼────────────────────────────────────────────────────────┘
               │ WebSocket (outbound from agent)
               │ NodeMessage: ConnectivityProbeRequest, ConnectivityFixRequest
               ▼
   ┌──────────────────────────────────────────────────────────────────┐
   │  Agent (escluse-agent / solys)                                    │
   │  ┌──────────────────────────────────────────────────────────┐    │
   │  │  ConnectivityMonitor (new — mirrors DnsWatcher pattern)   │    │
   │  │   • Periodic: re-evaluate firewall_active + is_cgn_suspect│    │
   │  │   • Re-emit ConnectivityReport on delta only             │    │
   │  └────────┬─────────────────────────────────────────────────┘    │
   │           │ raw facts (local_ip, public_ip, port_bound,         │
   │           ▼ firewall_active, default_gateway, is_cgn_suspect,    │
   │  ┌──────────────────────────────────────────────────────────┐    │
   │  │  DiagnosticCollector (new)                               │    │
   │  │   • docker.inspect_container → port_bindings              │    │
   │  │   • detect_public_ip() (reuse dns_watch.rs:132)          │    │
   │  │   • local_ip_address::find() → local_ip                  │    │
   │  │   • ip route show default → default_gateway              │    │
   │  │   • is_cgn_suspect = (local_ip in 100.64.0.0/10)         │    │
   │  │     OR (default_gateway in 100.64.0.0/10)                │    │
   │  │   • which('ufw' / 'firewalld' / 'iptables' / 'tailscale' │    │
   │  │     / 'cloudflared') → firewall_active, fallback options │    │
   │  │   • upnp-rs::search_igd() → upnp_available               │    │
   │  └──────────────────────────────────────────────────────────┘    │
   │  ┌──────────────────────────────────────────────────────────┐    │
   │  │  Task dispatch handlers (extended mod.rs:118)            │    │
   │  │   firewall.open_port, firewall.close_port,               │    │
   │  │   upnp.add_mapping, upnp.remove_mapping                   │    │
   │  │   (each adds audit-log row via WebSocket)                │    │
   │  └──────────────────────────────────────────────────────────┘    │
   │           │ iptables -I INPUT -p tcp --dport <port>              │
   │           │ -m comment --comment "esluse:<server-id>" -j ACCEPT  │
   │           │ (or ufw allow / firewall-cmd --add-port equivalent)   │
   │           ▼                                                        │
   │       ┌─────────────┐                                              │
   │       │  Linux host  │  ← firewall opt-in required at install     │
   │       │  firewall    │  ← UPnP IGD: AddPortMapping with 1h lease  │
   │       └─────────────┘  ← renew every 30 min by background task   │
   └──────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
api/
├── migrations/
│   └── 20260607000001_add_connectivity_columns.sql          # NEW
│       ALTER TABLE servers
│         ADD COLUMN connectivity_status TEXT DEFAULT 'unknown',
│         ADD COLUMN connectivity_mode   TEXT DEFAULT 'direct',
│         ADD COLUMN last_probe_at        TIMESTAMPTZ;
│   └── 20260607000002_create_connectivity_audit_log.sql     # NEW
│       CREATE TABLE connectivity_audit_log (
│         id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
│         server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
│         node_id  UUID REFERENCES nodes(id),
│         event_type TEXT NOT NULL,        -- 'connectivity.diagnostics' |
│                                           --  'firewall.open_port' |
│                                           --  'upnp.add_mapping' |
│                                           --  'connectivity.probe'
│         command    TEXT,                  -- exact iptables/ufw/etc. command
│         status     TEXT NOT NULL,         -- 'ok' | 'failed' | 'attempted'
│         details    JSONB DEFAULT '{}'::jsonb,
│         created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
│       );
│       CREATE INDEX idx_connectivity_audit_log_server_ts
│         ON connectivity_audit_log (server_id, created_at DESC);
│
├── src/
│   ├── domain/
│   │   ├── entities/
│   │   │   └── connectivity_audit_log.rs                   # NEW
│   │   └── repositories/
│   │       ├── connectivity_audit_log_repository.rs        # NEW
│   │       └── mod.rs                                      # updated
│   ├── infrastructure/
│   │   └── repositories/
│   │       └── sqlx_connectivity_audit_log_repository.rs   # NEW
│   ├── presentation/
│   │   ├── ws/
│   │   │   └── node_protocol.rs                            # EXTEND
│   │   │     + NodeMessage::ConnectivityReport             # agent → backend
│   │   │     + NodeMessage::ConnectivityFixRequest         # backend → agent
│   │   │     + NodeMessage::ConnectivityFixResult          # agent → backend
│   │   ├── handlers/
│   │   │   ├── node_ws_handler.rs                          # EXTEND dispatch
│   │   │   │   + ConnectivityReport case → store + classify
│   │   │   │   + ConnectivityFixResult case → store
│   │   │   └── connectivity_handlers.rs                    # NEW (REST)
│   │   │     + POST /api/v1/servers/:id/connectivity/probe # manual trigger
│   │   │     + GET  /api/v1/servers/:id/connectivity       # status + audit
│   │   ├── services/                                        # NEW
│   │   │   └── connectivity_service.rs                     # probe + classify
│   │   │     + async fn probe_java_edition(public_ip, port, timeout)
│   │   │       → ProbeResult { tcp_ok, handshake_ok, status_ok, version, players }
│   │   │     + async fn probe_bedrock_edition(public_ip, port, timeout)
│   │   │       → ProbeResult { raknet_ok, motd, version, players }
│   │   │     + async fn classify(probe, diagnostics) → FailureMode
│   │   └── routes/api_routes.rs                            # MOUNT new routes
│
src/ (agent)
├── handlers/
│   ├── mod.rs                                              # EXTEND dispatch
│   │   + "connectivity.diagnostics" → connectivity::handle_diagnostics
│   │   + "firewall.open_port"   → connectivity::firewall::open
│   │   + "firewall.close_port"  → connectivity::firewall::close
│   │   + "upnp.add_mapping"     → connectivity::upnp::add
│   │   + "upnp.remove_mapping"  → connectivity::upnp::remove
│   ├── connectivity.rs                                     # NEW orchestrator
│   ├── connectivity/
│   │   ├── mod.rs
│   │   ├── diagnostics.rs   # collect raw facts
│   │   ├── firewall.rs      # iptables / ufw / firewalld wrappers
│   │   └── upnp.rs          # upnp-rs AddPortMapping
│   └── dns_watch.rs                                         # MIRROR pattern
│       # + ConnectivityMonitor (re-emit on diagnostic change)
├── main.rs                                                  # STARTUP
│   + connectivity_monitor.start() after dns_watcher.start()  (line 282)
│
app/src/pages/servers/
├── ServerManagerPage.jsx                                   # + badge column
├── ServerDetailsPage.jsx                                   # + ConnectivitySection
├── components/                                              # NEW
│   ├── ConnectivityBadge.jsx
│   ├── ConnectivitySection.jsx
│   └── ConnectivityAuditLog.jsx
├── hooks/
│   └── useConnectivity.js                                  # NEW
```

### Pattern 1: Hybrid probe + diagnostic reporting (D-01, D-04)

**What:** Agent emits a `ConnectivityReport` containing raw diagnostic facts; backend correlates the facts with its own probe outcome to produce the final classification.

**When to use:** Always for any new connectivity-related state changes.

**Example (Rust agent — `src/handlers/connectivity/diagnostics.rs`):**

```rust
// Source: CONTEXT.md D-04; pattern reuses dns_watch.rs:132 (detect_public_ip)
use local_ip_address::list_afinet_netifas;
use std::net::IpAddr;
use std::process::Command;

const CGNAT_RANGE: std::net::IpAddr = ...; // 100.64.0.0/10

pub async fn collect_diagnostics(
    docker: &bollard::Docker,
    server_id: uuid::Uuid,
    game_port: u16,
) -> anyhow::Result<serde_json::Value> {
    // 1) Public IP via existing helper (reused, not re-implemented)
    let public_ip = crate::handlers::dns_watch::detect_public_ip().await?;

    // 2) Local IP from default route
    let ifaces = list_afinet_netifas()?;
    let local_ip = ifaces.values()
        .find(|ip| ip.is_ipv4() && !ip.is_loopback())
        .copied();

    // 3) Default gateway (Linux: `ip route show default`)
    let default_gateway = read_default_gateway()?;

    // 4) Firewall presence: which() check (D-07)
    let firewall_active = which::which("ufw").is_ok()
        || which::which("firewalld").is_ok()
        || iptables_has_active_rules()?;

    // 5) is_cgn_suspect heuristic (agent's discretion — recommended)
    let is_cgn_suspect = matches!(local_ip, Some(ip) if in_cgnat_range(ip))
        || matches!(default_gateway, Some(gw) if in_cgnat_range(gw));

    // 6) Tailscale / Cloudflared detection (D-11, D-12)
    let tailscale_up = which::which("tailscale").is_ok()
        && Command::new("tailscale")
            .args(["status", "--json"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    let cloudflared_up = which::which("cloudflared").is_ok()
        && Command::new("cloudflared")
            .args(["tunnel", "list"])
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false);

    // 7) Container port bindings (PORT_NOT_BOUND reclassification)
    let port_bound = if let Ok(info) = docker.inspect_container(&server_id.to_string(), None).await {
        let ports = info.network_settings
            .and_then(|ns| ns.ports)
            .unwrap_or_default();
        let key = format!("{}/tcp", game_port);
        ports.get(&key).map(|b| !b.is_empty()).unwrap_or(false)
    } else {
        false
    };

    Ok(serde_json::json!({
        "server_id": server_id,
        "public_ip": public_ip,
        "local_ip": local_ip,
        "default_gateway": default_gateway,
        "firewall_active": firewall_active,
        "is_cgn_suspect": is_cgn_suspect,
        "tailscale_up": tailscale_up,
        "cloudflared_up": cloudflared_up,
        "port_bound": port_bound,
        "game_port": game_port,
    }))
}
```

### Pattern 2: Safe-to-fix dispatcher (D-05)

**What:** Backend classifies a probe failure; if the mode is auto-fixable (`PORT_NOT_BOUND` / `HOST_FIREWALL_BLOCKED` / `UPnP_UNAVAILABLE`), it sends a fix task to the agent. Agent executes, returns result, backend stores in `connectivity_audit_log`, backend re-probes.

**When to use:** Every probe failure.

**Example (Rust backend — `api/src/presentation/services/connectivity_service.rs`):**

```rust
// Source: CONTEXT.md D-05; mirrors execute_command dispatch at
// api/src/presentation/handlers/node_ws_handler.rs:332
pub async fn try_auto_fix(
    failure: &FailureMode,
    server_id: Uuid,
    node_id: Uuid,
    manager: &NodeConnectionManager,
) -> Result<(), AppError> {
    let fix_request = match failure {
        FailureMode::PortNotBound => ConnectivityFixRequest {
            action: "recreate_container_with_port_bindings".into(),
            params: json!({}),
        },
        FailureMode::HostFirewallBlocked => ConnectivityFixRequest {
            action: "firewall.open_port".into(),
            params: json!({ "port": 25565, "proto": "tcp", "server_id": server_id }),
        },
        FailureMode::CgnatDetected => return Ok(()),  // not auto-fixable
        FailureMode::UpnpUnavailable => ConnectivityFixRequest {
            action: "upnp.add_mapping".into(),
            params: json!({ "port": 25565, "lease_secs": 3600 }),
        },
    };

    manager.send_to_node(&node_id, &NodeMessage::ConnectivityFixRequest {
        request_id: Uuid::new_v4(),
        server_id,
        fix: fix_request,
    }).await?;

    // Backend re-probe is scheduled by the probe scheduler (D-02)
    Ok(())
}
```

### Pattern 3: UPnP lease renewal (D-08)

**What:** UPnP IGD mappings expire; the agent schedules a renewal task at 50% of the lease duration.

**When to use:** After a successful `AddPortMapping` returns.

**Example (Rust agent — `src/handlers/connectivity/upnp.rs`):**

```rust
// Source: CONTEXT.md D-08; upnp-rs 0.2 SSDP discovery + IGD control
use upnp_rs::discovery;
use std::time::Duration;

const LEASE_SECS: u32 = 3600; // 1 hour

pub async fn add_port_mapping(
    port: u16,
    protocol: &str,
) -> anyhow::Result<String> {
    let devices = discovery::discover(
        "urn:schemas-upnp-org:device:InternetGatewayDevice:1",
        Duration::from_secs(2),
    ).await?;
    let igd = devices.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!("No UPnP IGD found"))?;

    let mapping_id = igd.add_port_mapping(
        "",                     // external host (any)
        port,                   // external port
        protocol,
        LEASE_SECS,
        "0.0.0.0",              // internal client (the agent host)
        port,                   // internal port
    ).await?;

    // Schedule renewal at 50% of lease
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(LEASE_SECS as u64 / 2)).await;
        let _ = add_port_mapping(port, protocol).await;
    });

    Ok(mapping_id)
}
```

### Anti-Patterns to Avoid

- **Auto-installing Tailscale / Cloudflared:** Explicitly out of scope (D-11, D-12). The agent must detect-and-display, never install. The `tailscale_auth_key` column stays unused in Phase 67.
- **Modifying user firewall rules:** Only rules tagged `esluse:<server-id>` are added or removed. Never touch other INPUT/OUTPUT/FORWARD rules (D-05, D-09).
- **Self-probe from the agent:** Defeats the purpose (D-01). The user asks "can my friend outside my home network connect?" — only a probe from outside can answer that.
- **Skipping the audit log:** Every auto-fix action MUST be recorded (D-17). The user must see "iptables -I INPUT ... -m comment --comment 'esluse:uuid' -j ACCEPT @ 2026-06-07T10:23:45Z" in the audit log, not a generic "Firewall rule added".
- **Hiding the failure:** When the safe-to-fix gate fails, the user MUST see a hybrid failure report (D-13) — not a silent "Connecting..." spinner. The diagnostic section must show the actual error code, root cause, and audit log of attempts performed.
- **Synchronous re-probe blocking the WS handler:** The probe runs in the backend service layer (tokio task), not inline in the WebSocket dispatch. The handler returns immediately after queuing the probe (`node_ws_handler.rs:301` pattern — non-blocking).
- **Touching the dual `Server` struct problem:** Deferred per CONTEXT `<deferred>`. Phase 67 adds columns via a new migration; the OLD `domain/server/model.rs:8-31` and NEW `domain/entities/server.rs:8-75` continue to diverge — don't try to unify them in this phase.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Public IP detection | Custom HTTP fetcher to `api.ipify.org` etc. | `src/handlers/dns_watch.rs:132` `detect_public_ip()` (reused as-is) | Already implemented with 4 fallback providers; re-implementing creates a second source of truth. |
| Container port binding lookup | Custom Docker SDK call | `bollard::Docker::inspect_container()` (already used at `runtime.rs:186`) | Existing client connection; reuse. |
| SSDP / UPnP discovery | Raw UDP multicast `M-SEARCH` packet | `upnp-rs 0.2` `discovery::discover()` | UPnP IGDv2 SSDP edge cases (MX, USER-AGENT, retry) are surprisingly subtle; trust the crate. |
| WebSocket message framing | Custom JSON envelope | Existing `NodeMessage` enum at `api/src/presentation/ws/node_protocol.rs:7` | Add new variants, don't fork. |
| Audit log persistence | Custom file-based logger | `api/migrations/20260324000007_create_audit_logs_table.sql` pattern + new `connectivity_audit_log` table | Backend has a 69+ migration precedent; new table mirrors `audit_logs` shape (id, user_id, action, metadata, created_at). |
| Discord / email alert dispatch | Custom SMTP / HTTP code | Reuse `api/src/domain/billing/webhooks.rs` webhook emission + `discord_webhook_url` column on `servers` | Same pattern as billing webhook emission; avoids two paths. |
| Periodic background task spawn | Custom timer wheel | `tokio::time::interval` (mirrors `dns_watch.rs:51` `ticker.tick().await`) | Same pattern as existing DNS watcher; new `ConnectivityMonitor` follows it line-for-line. |
| CGN detection from a "real" public IP | Traceroute / TCP RST tricks | Compare `local_ip` (or `default_gateway`) to RFC 6598 `100.64.0.0/10`; correlate with what the agent thinks the public IP is | CGN cannot be detected from a single endpoint with 100% certainty. The combined heuristic is good enough for UX; flag for human confirmation. |

**Key insight:** The hybrid architecture (D-01) is the entire reason this phase avoids the trap of hand-rolling. By keeping the agent as a "facts provider" and the backend as the "prober + classifier", we avoid re-implementing a Minecraft-protocol client, a probe scheduler, a notification dispatch, and an audit log store — they all already exist in different forms.

## Common Pitfalls

### Pitfall 1: IPv4-only probe against IPv6-only servers (or vice versa)
**What goes wrong:** Docker binding is `host_ip: Some("0.0.0.0")` (IPv4 only, see `runtime.rs:128`). Backend probes IPv4, succeeds, but the user's ISP gives them a CGN IPv6 path. Probe says "Reachable" but the friend can't connect.
**Why it happens:** IPv6 dual-stack is deferred (CONTEXT `<deferred>`). The current code is IPv4-only.
**How to avoid:** Phase 67 sticks to IPv4 end-to-end. Document in the audit log + diagnostic panel that connectivity is "checked over IPv4 only". Add a TODO comment for Phase 68 / future IPv6 work.
**Warning signs:** Diagnostic panel shows "Reachable" but the user reports friends can't connect; no `CGNAT_DETECTED` but the public IP is in `100.64.0.0/10`.

### Pitfall 2: Probe timeout vs. Minecraft server startup delay
**What goes wrong:** Backend probes 5 seconds after `server.start` completes, but the Java Edition server's `Done (X.XXXs)! For help, type "help"` log line happens 5–15 seconds after the container's `started` state, especially on first run with cold disk-cache world generation. Probe reads the open TCP port but the SLP handshake gets a malformed/empty response.
**Why it happens:** `server.start` task completion ≠ game server listening. The 5-second delay (CONTEXT D-02) is a heuristic, not a guarantee.
**How to avoid:** Backend probe timeout should be 10s (not 2s) for the SLP handshake; on `SLP_HANDSHAKE_OK` failure, retry once after 5s; only classify as `UNKNOWN` (not `UNREACHABLE`) if both attempts fail. Audit log records both attempts.
**Warning signs:** Audit log shows two failed SLP probes in a row immediately after server.start, then a successful one 12s later — that's normal cold-start behaviour, not a failure.

### Pitfall 3: iptables comment-match cleanup races
**What goes wrong:** Agent adds an iptables rule with `comment "esluse:server-uuid"`. On server delete, the agent runs `iptables -D ... -m comment --comment "esluse:server-uuid"` to remove it. If two rules with the same comment exist (e.g., IPv4 + IPv6 rules for both TCP + UDP), the `-D` command removes only the first match, leaving the rest.
**Why it happens:** `-m comment` matches the FIRST rule with that comment, not all of them. For multiple rules (TCP+UDP×IPv4+IPv6 = 4 rules), four delete commands are needed.
**How to avoid:** `firewall::close_port` enumerates ALL matching rules via `iptables -S | grep "esluse:<server-id>"` and deletes each one explicitly. Or use `iptables-save | grep "esluse:<server-id>"` to read, parse, and re-build the input chain.
**Warning signs:** `iptables -L INPUT` after server delete shows stale `esluse:*` rules.

### Pitfall 4: UPnP IGD requires the agent to be on the LAN with the gateway
**What goes wrong:** UPnP discovery (`M-SEARCH` over UDP multicast to `239.255.255.250:1900`) only finds the IGD on the same LAN segment. If the agent runs on a VPS in a datacenter, there is no UPnP-capable gateway to find, so `UPnP_UNAVAILABLE` is always the verdict.
**Why it happens:** UPnP is a LAN-only protocol by design. Cloud VPS providers do not expose UPnP IGD to their customers.
**How to avoid:** Treat `UPnP_UNAVAILABLE` on a VPS as expected and skip the auto-fix attempt. The agent's `is_cgn_suspect` heuristic + the absence of a private local IP (`10.x`, `192.168.x`, `172.16-31.x`) is a reliable VPS detector. Show "UPnP not available in this environment" rather than failing the auto-fix.
**Warning signs:** Audit log shows repeated `upnp.add_mapping` failed attempts on cloud-VPS nodes.

### Pitfall 5: Probe result column write race
**What goes wrong:** Two concurrent probe results (one from the post-start scheduler, one from a manual "Reachable" button click) both `UPDATE servers SET connectivity_status = ...` at the same time, and the older result overwrites the newer.
**Why it happens:** No row-level locking on the `servers` table during probe result persistence.
**How to avoid:** Use `UPDATE servers SET connectivity_status = $1, last_probe_at = $2 WHERE id = $3 AND ($2 > last_probe_at OR last_probe_at IS NULL)` — only update if the new probe is newer than the stored one. This makes concurrent writes idempotent and keeps the latest result.
**Warning signs:** Manual "Reachable" button shows "Reachable" but the auto-scheduler overwrites it back to "Unreachable" 30 seconds later.

### Pitfall 6: Bedrock probe reads SLP-shaped response
**What goes wrong:** Backend tries to do a Java SLP probe on a Bedrock server (port 19132 UDP). The TCP connect either times out (server is UDP-only) or connects to a different service. Backend classifies as `UNREACHABLE` even though Bedrock is online.
**Why it happens:** The `servers.game` field in the DB may not disambiguate Java vs Bedrock reliably (Phase 36 added Bedrock, but the schema may not be enforced).
**How to avoid:** The server model has a `mc_loader` column (`migration/20260307000001`). Backend probe picks the right protocol based on `mc_loader`/`protocol` (or the port — 25565 vs 19132). For ambiguity, default to Java probe + log a warning.
**Warning signs:** Audit log shows `tcp_connect` failed on port 19132 for a known Bedrock server.

### Pitfall 7: `firewall.open_port` doesn't persist across reboot
**What goes wrong:** Agent runs `iptables -I INPUT ...` and the rule is live, but the next reboot wipes it. Probe succeeds now, fails after reboot.
**Why it happens:** `iptables` runtime rules are not persistent by default; they need `iptables-save` / `netfilter-persistent` / `iptables-persistent` package, or the distribution's native tool (`ufw` is persistent, `firewalld` is persistent, raw `iptables` is NOT).
**How to avoid:** `firewall::open_port` must call the appropriate persist command per distro: `netfilter-persistent save` (Debian/Ubuntu raw iptables), `ufw` (built-in), `firewall-cmd --runtime-to-permanent` (RHEL). The audit log entry should include the persist command.
**Warning signs:** Audit log shows "iptables rule added" but the rule disappears after `reboot` — classic sign of missing persistence step.

### Pitfall 8: iptables on nftables-only system
**What goes wrong:** Modern distros (Debian 12, Ubuntu 22.04+, RHEL 8+, Fedora 32+) ship `iptables-nft` as a compatibility shim, but some don't ship `iptables` at all. Agent shells out to `iptables` and gets "command not found".
**Why it happens:** Distribution splits iptables into separate packages now (e.g., `iptables-nft` vs `iptables-legacy`); some distros drop the iptables binary entirely.
**How to avoid:** `which('iptables')` returns false; `which('nft')` returns true. Fall back to `nft add rule inet filter input tcp dport <port> accept comment "<comment>"` with the `esluse:<server-id>` comment. Audit log records which backend was used.
**Warning signs:** Audit log on a fresh Debian 12 install shows "iptables: command not found".

## Code Examples

Verified patterns from official sources:

### Minecraft Java Edition Server List Ping (handshake + status request)

```rust
// Source: https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping (verified 2026-04-04)
// Combined handshake + status request + JSON parser, ~80 lines.

use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use std::time::Duration;

pub async fn probe_java_edition(
    host: &str, port: u16, timeout: Duration,
) -> Result<MinecraftStatus, ProbeError> {
    let mut stream = tokio::time::timeout(timeout, TcpStream::connect((host, port))).await
        .map_err(|_| ProbeError::Timeout)?
        .map_err(ProbeError::TcpConnect)?;

    // 1) Handshake packet:
    //    [length=varint][packet_id=0x00][protocol_version=-1][server_address=string][server_port=u16][next_state=1]
    let handshake = build_packet(&[
        &encode_varint(0x00),               // packet id
        &encode_varint(-1),                  // protocol version (-1 = any)
        &encode_string(host),                // server address (string)
        &encode_ushort(port),                // server port (u16)
        &encode_varint(1),                   // next state = status
    ]);
    stream.write_all(&handshake).await.map_err(ProbeError::Io)?;

    // 2) Status request packet:
    //    [length=varint][packet_id=0x00]
    let status_req = build_packet(&[&encode_varint(0x00)]);
    stream.write_all(&status_req).await.map_err(ProbeError::Io)?;

    // 3) Read status response:
    //    [length=varint][packet_id=0x00][json_response=string]
    let packet_len = read_varint(&mut stream).await.map_err(ProbeError::Io)?;
    let mut buf = vec![0u8; packet_len as usize];
    stream.read_exact(&mut buf).await.map_err(ProbeError::Io)?;
    let packet_id = read_varint_sync(&buf)?;
    if packet_id != 0x00 {
        return Err(ProbeError::Protocol);
    }
    let json = read_string_sync(&buf)?;

    let status: MinecraftStatus = serde_json::from_str(&json)
        .map_err(ProbeError::Json)?;
    Ok(status)
}

#[derive(serde::Deserialize)]
pub struct MinecraftStatus {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    #[serde(default)]
    pub description: serde_json::Value, // MOTD
}
#[derive(serde::Deserialize)]
pub struct VersionInfo { pub name: String, pub protocol: i32 }
#[derive(serde::Deserialize)]
pub struct PlayersInfo { pub online: i32, pub max: i32 }
```

### Minecraft Bedrock Edition RakNet Unconnected Ping (single UDP packet)

```rust
// Source: https://wiki.bedrock.dev/servers/raknet (verified)
// Unconnected Ping (0x01) + Unconnected Pong (0x1c) parse.
// RakNet magic constant is required in the request; server echoes it back.

use tokio::net::UdpSocket;
use std::time::Duration;

const RAKNET_MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
const UNCONNECTED_PING: u8 = 0x01;

pub async fn probe_bedrock_edition(
    host: &str, port: u16, timeout: Duration,
) -> Result<BedrockStatus, ProbeError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(ProbeError::Io)?;
    socket.connect((host, port)).await.map_err(ProbeError::Io)?;

    // Build: [0x01][client_time:u64][magic:16][client_guid:u64] = 25 bytes
    let client_time: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
    let client_guid: u64 = 0x0123456789abcdef; // arbitrary; bedrock doesn't validate
    let mut pkt = vec![UNCONNECTED_PING];
    pkt.extend_from_slice(&client_time.to_be_bytes());
    pkt.extend_from_slice(&RAKNET_MAGIC);
    pkt.extend_from_slice(&client_guid.to_be_bytes());

    tokio::time::timeout(timeout, socket.send(&pkt)).await
        .map_err(|_| ProbeError::Timeout)?
        .map_err(ProbeError::Io)?;

    let mut resp = [0u8; 1500];
    let len = tokio::time::timeout(timeout, socket.recv(&mut resp)).await
        .map_err(|_| ProbeError::Timeout)?
        .map_err(ProbeError::Io)?;
    let resp = &resp[..len];

    if resp[0] != 0x1c {  // UNCONNECTED_PONG
        return Err(ProbeError::Protocol);
    }
    // Response layout:
    //   [0x1c][client_time:u64][server_guid:u64][magic:16][string_len:u16][string:utf8]
    let _pong_time = &resp[1..9];
    let _server_guid = &resp[9..17];
    // Magic at resp[17..33] is the same RAKNET_MAGIC.
    let string_len = u16::from_be_bytes([resp[33], resp[34]]) as usize;
    let motd_str = std::str::from_utf8(&resp[35..35 + string_len])
        .map_err(ProbeError::Io)?;

    // Format: "MCPE;MOTD line 1;Protocol;Version;Online;Max;Server UID;MOTD line 2;GameMode;..."
    // Source: https://wiki.bedrock.dev/servers/raknet
    let parts: Vec<&str> = motd_str.split(';').collect();
    Ok(BedrockStatus {
        edition: parts.first().copied().unwrap_or("").to_string(),
        motd:    parts.get(1).copied().unwrap_or("").to_string(),
        protocol: parts.get(2).copied().unwrap_or("").to_string(),
        version:  parts.get(3).copied().unwrap_or("").to_string(),
        online:   parts.get(4).copied().unwrap_or("0").parse().unwrap_or(0),
        max:      parts.get(5).copied().unwrap_or("0").parse().unwrap_or(0),
    })
}

#[derive(Debug)]
pub struct BedrockStatus {
    pub edition: String,
    pub motd: String,
    pub protocol: String,
    pub version: String,
    pub online: i32,
    pub max: i32,
}
```

### CGN detection heuristic (Rust)

```rust
// Source: RFC 6598 (verified via https://www.rfc-editor.org/rfc/rfc6598)
// CGNAT range is 100.64.0.0/10 (100.64.0.0 - 100.127.255.255)

pub fn is_cgnat_suspect(local_ip: Option<std::net::Ipv4Addr>, gateway: Option<std::net::Ipv4Addr>) -> bool {
    let in_cgn = |ip: std::net::Ipv4Addr| -> bool {
        let octets = ip.octets();
        octets[0] == 100 && octets[1] >= 64 && octets[1] <= 127
    };
    local_ip.map(in_cgn).unwrap_or(false) || gateway.map(in_cgn).unwrap_or(false)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Agent self-probes its own loopback | Hybrid: agent sends facts, backend probes from public internet | Phase 67 (D-01) | Only a probe from outside the LAN proves reachability; user explicitly chose this. |
| `which('docker')` only for runtime detection | `bollard::Docker::inspect_container()` for port-binding state | Phase 67 (CONTEXT `RuntimeDetector` reuse) | Reuses the same Docker client connection; no new dependency. |
| Hand-rolled Minecraft protocol parsing | Wire-format SLP handshake + status request (~80 LOC) | n/a (in-house code, no crate) | No maintained, idiomatic, async Rust crate for full SLP. Implementation is small and tested by vanilla clients daily. |
| `igd` crate (popular but stale) | `upnp-rs 0.2` | 2023+ | Pinned by CONTEXT D-07; `upnp-rs` is actively maintained (per github.com/johnstonskj/rust-upnp, last commit recent) with documented discovery + IGD control. |
| Direct netlink iptables | `iptables` shell-out + `nft` fallback | 2026 (per commandlinux.com survey: 88.4% of Linux boxes run firewalld/UFW on top of nftables) | `iptables` syntax is now a shim over nftables; shell-out works on every modern distro. |
| `192.168.x` RFC 1918 detection | `100.64.0.0/10` (RFC 6598) CGN detection | RFC 6598 (2012), widely adopted by 2020s | Distinguishes LAN (RFC 1918) from CGN (RFC 6598). Critical for accurate `CGNAT_DETECTED` label. |
| `enable_tailscale` column unused | Detection only, surface Tailscale IP if up | Phase 67 (D-11) | Reuses existing column without writing it; user explicitly deferred auto-install. |
| `dns_config` table (Cloudflare DNS) reused for tunnel detection | Detect `cloudflared` binary; show as Experimental diagnostic | Phase 67 (D-12) | Reuses Phase 51 credentials; tunnel is HTTP-optimized, not a primary fallback. |
| Alert dispatch via custom SMTP | Reuse `api/src/domain/billing/webhooks.rs` + `discord_webhook_url` column | Phase 67 (D-16) | Single webhook dispatch path; consistent with billing / crash alerts. |

**Deprecated/outdated:**
- `easy-upnp` (docs.rs/easy-upnp): maintained but minimal (open/close port only). `upnp-rs` exposes the SSDP discovery + IGD control surface, which we need for `is_cgn_suspect` cross-check.
- `igd` crate: most popular but hasn't seen major releases recently. CONTEXT D-07 picked `upnp-rs`; this aligns with active maintenance.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `upnp-rs 0.2` is the right Rust crate for UPnP IGD `AddPortMapping` | Standard Stack | If `upnp-rs` API is awkward (e.g., requires sync wrapper), we waste time on ceremony. Fallback: `igd` crate (less maintained but battle-tested). Verify with `cargo doc --open upnp-rs` at plan time. |
| A2 | `local-ip-address 1.x` exposes both `local_ip` and `default_gateway` lookup | Standard Stack | If the crate only does local IP (no gateway), we add a 5-line `ip route show default` shell-out as a supplement. The combo of crate + `ip route` is robust. |
| A3 | The agent's `RuntimeDetector` (`agent-runtime/src/detector.rs`) exposes `docker()` returning a working `bollard::Docker` instance | Pattern 1 code example | If the `docker` field is `None` (no socket), the diagnostic collector needs a graceful fallback. Add a check at the top of `collect_diagnostics` that returns `port_bound: false` and logs a warning instead of failing. |
| A4 | The existing `NodeConnectionManager.send_to_node()` (`api/src/presentation/ws/node_connection_manager.rs`) can deliver a new `ConnectivityFixRequest` message variant | Pattern 2 code example | If `send_to_node` is generic-enum-only, we add a `send_to_node_typed` helper. Existing code already sends `NodeMessage::DnsConfig` (line 258 of `node_ws_handler.rs`), so this is safe. |
| A5 | The server repository can update `connectivity_status` / `connectivity_mode` / `last_probe_at` columns without breaking existing queries | Architecture, migration shape | The new migration uses `ADD COLUMN IF NOT EXISTS ... DEFAULT ...` so it is non-breaking. Old `Server` struct fields stay unchanged. |
| A6 | The existing `discord_webhook_url` column on `servers` is the right per-server alert channel | Standard Stack, D-16 | If a future phase wants per-event-type webhooks, that's a refactor. Phase 67 reuses the existing column as-is. |
| A7 | The Bedrock Edition `mc_loader` or `template` column reliably distinguishes Java from Bedrock servers | Pitfall 6 | If the data is inconsistent, fall back to port-based heuristic (19132 = Bedrock) and log a warning. |
| A8 | `iptables -m comment --comment "esluse:<id>"` is the right tagging strategy for cleanup | Pattern 3, Pitfall 3 | If the user's distro doesn't have the `comment` match (older iptables), use `iptables -I INPUT -s 0.0.0.0/0 -p tcp --dport <port> -j ACCEPT` and pair the open/close with a stored rule-hash on disk. |
| A9 | The phase ships without breaking the existing `audit_logs` table (which is `IMMUTABLE` per `20260325000001_make_audit_logs_immutable.sql`) | Standard Stack, Don't Hand-Roll | The new `connectivity_audit_log` table is separate, so the immutability constraint doesn't carry over. Use a different table name to avoid confusion. |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.
**If this table is not empty:** A1–A9 are codebase-tooling assumptions; the planner should accept them as defaults and the executor should validate against the local environment at Wave 0 before locking the implementation. None of them block planning.

## Open Questions

1. **Probe timeout tuning**
   - What we know: CONTEXT D-02 says "5-10s after server.start", SLP handshake is typically <1s on healthy servers.
   - What's unclear: The right probe timeout for the entire TCP+handshake+status request sequence. 5s is tight, 10s is conservative. Recommend 10s for the first probe, 5s for periodic ones.
   - Recommendation: Planner picks 10s/5s as defaults; if `Pitfall 2` (cold-start) is observed in practice, bump to 15s.

2. **Cooldown for the manual "Reachable" button**
   - What we know: User can spam the button; each click triggers a probe (TCP + SLP = network round trip from backend → user's public IP).
   - What's unclear: How long should the cooldown be? The backend can rate-limit by user_id or by server_id.
   - Recommendation: 30-second cooldown per server. Store `last_manual_probe_at` in Redis (already in stack per `INTEGRATIONS.md`) with a 60s TTL.

3. **CGN heuristic validation**
   - What we know: `is_cgn_suspect` is in the agent's discretion (CONTEXT). RFC 6598 is `100.64.0.0/10`.
   - What's unclear: Whether CGN is also detectable when the public IP is in 100.64.0.0/10 (which is what the agent *thinks* its public IP is when CGN'd). Some CGN setups expose the CGN address as the public IP, others hide it.
   - Recommendation: Heuristic = (local_ip ∈ 100.64/10) OR (default_gateway ∈ 100.64/10) OR (agent's reported public_ip ∈ 100.64/10) (last one is a self-check; if it sees itself in CGN range, it's CGN). Document the false-positive rate in the audit log.

4. **Probe scheduler placement**
   - What we know: CONTEXT D-02 says "on startup + event-triggered + periodic". Existing `DnsWatcher` lives in the agent.
   - What's unclear: Should the probe scheduler live on the backend (centralised) or the agent (with a backend probe origin)?
   - Recommendation: Backend owns the scheduler; agent is reactive. The agent emits `ConnectivityReport` when its diagnostics change; the backend decides when to re-probe based on (event, time-since-last-probe, server.start signal). This avoids race conditions on who decides "now is the time to probe".

5. **IPv6 dual-stack port binding**
   - What we know: `host_ip: Some("0.0.0.0")` is IPv4-only. CONTEXT defers IPv6.
   - What's unclear: How many users actually have IPv6-only paths. For now, document "IPv4 only" in the diagnostic panel.
   - Recommendation: Out of scope; revisit when users report IPv6 reachability issues.

## Environment Availability

The agent's host environment (where the binary will run) is the user's VPS or local machine — not this research machine. Tools listed are what an installer on a typical Linux host should expect to find; the agent probes for them at runtime via `which()` and degrades gracefully if missing.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Docker (or Podman) | `port_bound` reclassification via `bollard` | ✓ (assumed on all Escluse nodes) | bollard 0.18 | None — the whole product requires Docker/Podman. |
| `iptables` | `firewall.open_port` for raw iptables Linux | ✓ on this dev box; common on most Linux | iptables 1.8.11 (nf_tables) | Fall back to `nft` on distros without iptables binary. |
| `ufw` | Ubuntu/Debian convenience wrapper | ✗ on this dev box (assumed on user Ubuntu hosts) | — | Use raw iptables + comment-match; same effect. |
| `firewalld` | RHEL/Fedora convenience wrapper | ✓ on this dev box | — | Use raw iptables + comment-match; same effect. |
| `upnp-rs` crate (Rust) | `upnp.add_mapping` | needs Cargo fetch at build | — | Detect-only fallback (no auto-fix, manual wizard). |
| `local-ip-address` crate (Rust) | `local_ip` + `default_gateway` lookup | needs Cargo fetch at build | — | `ip route show default` shell-out (Linux only). |
| `tailscale` binary | Tailscale detection (D-11) | ✓ on this dev box; not on user hosts by default | — | If absent, `tailscale_up: false`, skip Tailscale section. |
| `cloudflared` binary | Cloudflare Tunnel detection (D-12) | ✗ on this dev box; not on user hosts by default | — | If absent, `cloudflared_up: false`, skip Cloudflare section. |
| UPnP IGD on the LAN | UPnP auto-fix | assumed on most home routers; absent on cloud VPS | — | Skip auto-fix on VPS; surface "UPnP not available in this environment" message. |
| Backend reachability from public internet | Backend's outbound probe to `(public_ip, port)` | requires user's inbound port to be open | — | This is the whole point of Phase 67. |
| Caddy / TLS at `*.esluce.com` | Backend probe origin is hosted behind the gateway | already in stack per `INTEGRATIONS.md` | Caddy 2 | None. |

**Missing dependencies with no fallback:**
- None — every tool has a fallback path documented in the table.

**Missing dependencies with fallback:**
- `ufw` / `firewalld` / `cloudflared` / `tailscale` are detected-only; if absent, the diagnostic panel just shows the section as "not detected" (D-11, D-12). The agent never installs any of them.
- `upnp-rs` is mandatory; the crate is pulled at `cargo build` time, no runtime binary needed.

## Security Domain

> Required because `security_enforcement` is absent in `.planning/config.json` (default = enabled).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V1 Architecture | yes | Threat-model the new probe pipeline; document trust boundary (agent node ↔ public internet ↔ backend) in `ARCHITECTURE.md`. |
| V2 Authentication | no | The probe pipeline uses the existing node↔backend WebSocket auth. No new auth surface. |
| V3 Session Management | no | Existing WebSocket session model (re-registration on reconnect, `api/src/presentation/ws/node_connection_manager.rs`) is reused. |
| V4 Access Control | **yes** | `POST /api/v1/servers/:id/connectivity/probe` MUST verify the requesting user owns the server (re-use `get_server_use_case.execute(server_id)` ownership check at `api/src/presentation/handlers/file_handlers.rs:143`). Audit log rows are server-scoped (no cross-tenant access). |
| V5 Input Validation | **yes** | The probe payload (host, port, timeout) MUST be validated server-side: `port` is `u16` (1..=65535), `host` is `String` max length 253, `timeout` is `Duration` ≤ 30s. Reject malformed input at the handler boundary, not in the service. |
| V6 Cryptography | no | The probe is unauthenticated plaintext (correct: it impersonates a player, which never has credentials). WebSocket transport is TLS-terminated by Caddy. |
| V7 Error Handling | yes | Probe failures surface as `ProbeError` enum with structured codes (`TIMEOUT`, `TCP_CONNECT`, `HANDSHAKE`, `PROTOCOL_MISMATCH`); never leak the raw backend infrastructure (IP, port) to the frontend. |
| V8 Data Protection | no | No PII in the audit log. Server UUIDs + commands are not personal data. |
| V9 Communication | yes | Backend → agent `ConnectivityFixRequest` is over the existing authenticated WebSocket channel. Probe results are only returned to the owning user's session. |
| V10 Malicious Code | yes | Agent's `firewall.open_port` MUST whitelist commands: only `iptables -I INPUT ...`, `ufw allow ...`, `firewall-cmd --add-port=...` are allowed. Reject anything else. Audit log records the EXACT command run. |
| V11 Business Logic | yes | Auto-fix dispatcher MUST enforce the safe-to-fix gate (D-05). Never accept a fix request from the agent that wasn't classified by the backend's `try_auto_fix` function. |
| V12 Files and Resources | no | The phase adds no file-system access beyond what `bollard` already does. |
| V13 API and Web Service | yes | New REST endpoint `POST /api/v1/servers/:id/connectivity/probe` MUST enforce per-user rate limit (e.g., 1 probe per 30s per server — same as the manual-button cooldown). |
| V14 Configuration | yes | `firewall_auto_manage: bool` opt-in flag (per CONTEXT D-09) is set at agent install time. The agent refuses to run `firewall.open_port` if this flag is `false`. |

### Known Threat Patterns for Rust + Linux firewall + UPnP

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Command injection in `firewall.open_port` payload (agent receives untrusted port number from backend) | Tampering | The `port` is `u16` after deserialization (Rust's `serde` rejects out-of-range); the comment is constructed from the server UUID (not from user input). The command string is built from a fixed template. |
| Comment-injection trick: server UUID contains shell metacharacters | Tampering | The server UUID is a Rust `Uuid` type, not a `String`. `Uuid::to_string()` produces canonical lowercase hex with hyphens — no shell metacharacters. Safe. |
| Agent misuses the auto-fix path to open arbitrary ports | Elevation of Privilege | Auto-fix is dispatched by the backend, not requested by the agent. The backend's `try_auto_fix` is the only code that constructs `ConnectivityFixRequest`. The agent's handler validates the `action` field against a fixed enum (`firewall.open_port`, `upnp.add_mapping`, `recreate_container_with_port_bindings`). |
| UPnP IGD compromised / malicious UPnP response | Spoofing | `upnp-rs` uses HTTPS for the IGD control URL where the device supports it; for plain-HTTP IGDs (most home routers), the device authentication is via the router admin password (not exposed). Phase 67 accepts the risk and only opens the specific port the backend asked for, with the lease duration capped. |
| Probe results spoofed by a malicious actor at the user's public IP | Spoofing | The probe originates from the backend's known IP; backend persists the source IP in the audit log. If a future attacker can replay results, the worst case is a `Reachable` false-positive (game is actually down) — not a security incident. |
| Audit log tampering | Repudiation | The new `connectivity_audit_log` table is append-only; no UPDATE / DELETE grants in the migration. Mirrors the `audit_logs` immutability pattern (`20260325000001_make_audit_logs_immutable.sql`). |
| Reachable → Unreachable transition triggers spam of Discord/email alerts | Denial of Service | The notification service rate-limits per server (e.g., 1 alert per 5 min max); subsequent transitions during the window update the dashboard banner only, not external channels. |
| Auto-fix races with manual user action (user manually opens port 25565, agent then adds `esluse:<id>` rule for the same port) | Tampering | Both rules coexist (iptables is additive). Audit log records both. No data loss; minor visual noise in `iptables -L`. |
| Backend probe over IPv4 reveals user IPv4 address to attacker who controls the agent node | Information Disclosure | This is by design — the agent needs to know its own public IP. The user already trusts the agent with root on their node. The probe is backend-originated, not agent-originated, so the agent never sees the backend's view of the connection. |

## Sources

### Primary (HIGH confidence)
- `minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping` — official Java SLP packet format (handshake + status request), verified 2026-04-04
- `wiki.bedrock.dev/servers/raknet` + `minecraft.wiki/w/RakNet` — official Bedrock RakNet unconnected ping/pong format, verified
- `RFC 6598` (`www.rfc-editor.org/rfc/rfc6598`) — CGNAT 100.64.0.0/10 shared address space, defined 2012
- `crates.io/upnp-rs` 0.2 — Rust UPnP crate pinned by CONTEXT D-07
- `crates.io/local-ip-address` — Rust local IP + gateway discovery
- `commandlinux.com/statistics/linux-firewall-adoption-rates-iptables-vs-nftables-vs-firewalld-usage` (2026-03-19) — 88.4% of home/SMB Linux boxes run firewalld/UFW over nftables; iptables syntax is shim'd via iptables-nft on modern distros
- `networkingtoolbox.net/reference/cgnat` + `blog.cloudflare.com/detecting-cgn` — CGN heuristics; gateway IP comparison is the standard detection technique
- `api/migrations/20260324000007_create_audit_logs_table.sql` — audit log table shape (mirrored for `connectivity_audit_log`)
- `api/src/presentation/ws/node_protocol.rs:7-135` — `NodeMessage` enum, the existing WS message contract
- `api/src/presentation/handlers/node_ws_handler.rs:237-298` — `Heartbeat` handler pattern to mirror for `ConnectivityReport`
- `src/handlers/dns_watch.rs:18-80, 132-155` — `DnsWatcher` pattern (periodic background task) + `detect_public_ip()` (reused)
- `src/handlers/runtime.rs:107-174` — existing `HostConfig` port binding shape (the `PORT_NOT_BOUND` reclassification target)
- `src/handlers/mod.rs:118-294` — task dispatch + `get_task_config` (where new `connectivity.*` tasks hook in)
- `agent/agent-core/crates/agent-runtime/src/detector.rs:15-115` — `RuntimeDetector::docker()` returning `bollard::Docker` (reused for `inspect_container`)
- `migration/20260307000001_add_enhanced_server_features.sql:9-10` — existing `enable_tailscale` / `tailscale_auth_key` columns (read-only reuse)
- `migration/20260501000001_fix_legacy_not_null_columns.sql:14` — `enable_tailscale SET DEFAULT false`

### Secondary (MEDIUM confidence)
- `libnpupnp` reference — UPnP IGD port mapping standard behaviour, lease durations typically configurable
- `Cloudflare blog: One IP address, many users: detecting CGNAT` (2025-10-29) — CGNAT detection at scale; heuristic combination of (gateway IP, public IP) is the industry pattern
- `.planning/phases/51-automasi-dns-cloudflare/51-CONTEXT.md` (referenced in CONTEXT.md) — Phase 51 Cloudflare DNS context (reused for tunnel detection)
- `.planning/phases/65-buat-installer-script-auto-install-docker-sebelum-install-so/65-CONTEXT.md` — Phase 65 install-time consent pattern (mirrored for `firewall_auto_manage` opt-in)

### Tertiary (LOW confidence)
- `easy-upnp` (docs.rs/easy-upnp) — alternative UPnP crate; minimal API (open/close only), would force us to add SSDP discovery by hand. Lower priority than `upnp-rs`.
- `github.com/bedrock-crustaceans/raknet` (incomplete Rust RakNet) — alternative for full Bedrock support; Phase 67 only needs the unconnected ping/pong pair, so a full RakNet client is overkill.

## Metadata

**Confidence breakdown:**
- **Standard stack:** MEDIUM-HIGH — `bollard`, `which`, `reqwest`, `tokio`, `chrono`, `uuid` are verified in `Cargo.toml` and `Cargo.lock`; `upnp-rs` 0.2 and `local-ip-address` 1.x are verified via crates.io/library search. Build-time validation recommended in Wave 0.
- **Architecture:** HIGH — Hybrid probe model (D-01) is a well-established pattern; existing `NodeMessage` enum and `DnsWatcher` background task are the natural extension points. The 4-mode classification is small enough to enumerate.
- **Minecraft protocol implementation:** HIGH — wire format documented on official Minecraft Wiki; small enough (5-field handshake + 80-byte status response) to implement inline without a crate dependency. Bedrock RakNet unconnected ping is similarly well-documented and small.
- **CGN detection:** MEDIUM — heuristic combination (gateway IP in 100.64/10 OR public IP in 100.64/10) is the standard pattern, but false-positive/false-negative rate is non-zero. Documented in the audit log so users can self-validate.
- **Firewall cross-distro:** MEDIUM — iptables comment-match works on every modern Linux; ufw and firewalld wrappers are optional. nft fallback is the safety net for iptables-less distros.
- **Pitfalls:** HIGH — most pitfalls are derived from direct codebase reading (Pitfall 1, 3, 5, 6) or well-documented protocol edge cases (Pitfall 2, 4, 7, 8). Each has a concrete warning sign for early detection.

**Research date:** 2026-06-07
**Valid until:** 30 days (2026-07-07) — UPnP crate ecosystem and Linux firewall stack are stable; only refresh if a new distro (e.g., Ubuntu 26.04) ships a different default or if a maintained Rust Minecraft-protocol crate emerges.
