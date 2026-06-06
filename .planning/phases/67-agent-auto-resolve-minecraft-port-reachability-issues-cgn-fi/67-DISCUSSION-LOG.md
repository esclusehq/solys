# Phase 67: Agent auto-resolve Minecraft port reachability issues (CGN/firewall/Docker port exposure) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-07
**Phase:** 67-agent-auto-resolve-minecraft-port-reachability-issues-cgn-fi
**Areas discussed:** Reachability probe mechanism, Failure classification & auto-fix, NAT-traversal fallback stack, User-facing diagnostic surface

---

## Reachability probe mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Agent self-probes | Agent runs the probe from a known external endpoint (curl-style, via our own API or third-party TCP/HTTP probe service) | |
| Backend probes on-demand | Backend (Escluse API) initiates a probe from its own infrastructure on agent's behalf | ✓ (primary) |
| User self-verifies | Agent reports local network state and the user runs the test via a dashboard 'Test Connection' button | |

**User's choice:** Hybrid (1+2). Agent sends raw diagnostics (`local_ip`, `public_ip`, `port`, `bound`, `online_mode`); backend performs the actual TCP + Minecraft protocol probe from the public internet. Self-probe from the agent is rejected because it cannot prove the port is reachable from outside the user's network.

**Notes:** "Yang ingin diketahui pengguna sebenarnya adalah: 'Apakah teman saya di luar jaringan rumah bisa masuk ke server Minecraft saya?' — dan hanya probe dari luar (backend) yang bisa menjawab pertanyaan itu dengan pasti."

### Probe timing

| Option | Description | Selected |
|--------|-------------|----------|
| On container start + periodic | Probe after each server.start + every 5 min background | |
| On-demand only via button | Probe only when user clicks 'Test Connection' | |
| On container start, no periodic | Probe once after each server.start, rely on Phase 51 DnsWatcher for re-trigger | |
| On startup + event-triggered + low-frequency periodic | Probe on startup, on IP/firewall change, and low-frequency periodic | ✓ |

**User's choice:** "Jadi pilihannya paling dekat dengan opsi 1, tetapi aku akan mengubahnya menjadi: 'On startup + event-triggered + low-frequency periodic verification'."

### Probe depth

| Option | Description | Selected |
|--------|-------------|----------|
| Minecraft protocol-aware | TCP + Java handshake + status request; Bedrock RakNet unconnected ping | ✓ |
| TCP connect only | Just verify TCP port accepts connections | |
| Game-aware but not full handshake | TCP connect + read first bytes (Server List Ping banner / RakNet) | |

**User's choice:** Minecraft protocol-aware. Catches "port is open but wrong service" edge cases.

### Diagnostics reporting

| Option | Description | Selected |
|--------|-------------|----------|
| Report only local state | Agent reports: bound, firewall_rules, public_ip, local_ip, port, online_mode. Backend uses ONLY to enrich probe failure | |
| Pre-classify and report root cause | Agent does local diagnosis and sends a pre-classified root_cause field | |
| Report raw diagnostics, backend classifies | Agent reports raw data: port_bound, firewall_active, default_gateway, local_subnet, detected_public_ip, is_cgn_suspect. Backend stores and classifies | ✓ |

**User's choice:** "Agent sebaiknya lebih fokus mengirim fakta mentah + beberapa heuristic, sedangkan backend tetap menjadi pihak yang menghasilkan klasifikasi akhir yang dilihat pengguna. Ini akan jauh lebih fleksibel saat Escluse berkembang."

---

## Failure classification & auto-fix

### Auto-fix scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: Docker only | Agent only auto-fixes Docker issues. For firewall and CGN, only diagnostics | |
| Moderate: Docker + firewall | Agent auto-fixes Docker + host firewall. For CGN, setup wizard for Tailscale/Cloudflare Tunnel | |
| Maximum: aggressive auto-fix | Docker + firewall + UPnP query + Cloudflare Tunnel try-install | |
| Safe-to-fix gate (user's hybrid) | Failure → Can agent safely fix? → YES → Auto Fix → Re-Probe → Reachable? | ✓ |

**User's choice:** Hybrid safe-to-fix gate. Auto-fixable: firewall (server's port), port mapping (UPnP), relay/tunnel enablement. NOT auto-fixable: changes to firewall rules other than the managed server's port, deletion of user firewall rules, aggressive router config beyond standard UPnP, arbitrary root commands, silent 3rd-party installs.

**Notes:** User provided a full failure-handling flow:
```
Failure Detected
  ↓
Can agent safely fix?
  ↓
  YES
  ↓
Auto Fix
  ↓
Re-Probe Backend
  ↓
Reachable?
```

### Failure modes

| Option | Description | Selected |
|--------|-------------|----------|
| All 6 modes | PORT_NOT_BOUND, HOST_FIREWALL_BLOCKED, CGNAT_DETECTED, ISP_BLOCKED, UPnP_UNAVAILABLE, PROTOCOL_MISMATCH | |
| Core 3 only | PORT_NOT_BOUND, HOST_FIREWALL_BLOCKED, CGNAT_DETECTED | |
| Core 3 + UPnP | PORT_NOT_BOUND, HOST_FIREWALL_BLOCKED, CGNAT_DETECTED, UPnP_UNAVAILABLE | ✓ |

**User's choice:** "✅ Opsi 3 sebagai MVP. Tetapi dengan sedikit modifikasi: agent sebaiknya lebih fokus mengirim fakta mentah + beberapa heuristic, sedangkan backend tetap menjadi pihak yang menghasilkan klasifikasi akhir yang dilihat pengguna."

### Detection method

| Option | Description | Selected |
|--------|-------------|----------|
| Standard, no shelling out | `which()` checks for ufw/firewalld/iptables/tailscale/cloudflared; cross-platform best-effort | ✓ (partial) |
| UPnP + Linux tooling only | Dedicated Rust crates (upnp-rs, network-interface). Linux-first, Windows separate. Direct iptables netlink | ✓ (partial) |
| CLI-based, all distros | Shell out to platform-specific commands and parse output | |

**User's choice:** "Kombinasi 1 + sedikit 2." — `which()` checks for CLIs + dedicated Rust crates (upnp-rs) for UPnP and network interfaces. Linux-first, cross-platform best-effort.

### UPnP behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Auto + manual + retry | upnp-rs AddPortMapping on IGD; fall back to user-side wizard with gateway IP + manual instructions | ✓ |
| Auto only | Try UPnP only; if fails, user must manually forward ports | |
| Manual + guidance only | Never modifies router; just detects UPnP availability | |

**User's choice:** Auto + manual + retry. Both paths result in backend re-probe.

### Firewall fix

| Option | Description | Selected |
|--------|-------------|----------|
| Conservative: opt-in per server | Narrowly-scoped rule with comment; opt-in flag in server config | |
| Aggressive: no opt-in | Auto-add rule as soon as blocked; comment only | |
| Informational only | Never modifies firewall; provides commands to copy-paste | |
| Hybrid (user's spec) | One-time consent at install; only server's port; tagged rule; never delete user rules; cleanup on server delete | ✓ |

**User's choice:** "Auto-fix firewall = YA, tetapi: Memerlukan persetujuan satu kali saat instalasi agent. Hanya membuka port yang dikelola Escluse. Menambahkan komentar/tag pada setiap rule. Tidak pernah menghapus atau mengubah rule milik pengguna. Membersihkan rule Escluse saat server dihapus."

---

## NAT-traversal fallback stack

### Fallback stack scope

| Option | Description | Selected |
|--------|-------------|----------|
| Tailscale + Cloudflare Tunnel | Both fallbacks integrated | |
| Tailscale only | Lean MVP, defer Cloudflare Tunnel | |
| Custom Esluce relay | Months of work, separate phase | |
| Cloudflare Tunnel only | Reuse Phase 51 credentials | |
| Direct Mode + Relay Mode (user's design) | Reachability check chooses between Direct (free) and Relay (deferred infra) | ✓ |

**User's choice:** Direct + Relay dual-mode. Phase 67 = agent-side Direct Mode + mode selection logic. Custom relay infrastructure (`relay.esluce.com`, outbound tunnel, player routing) is deferred to a follow-up phase.

**Notes:** "Direct Mode" is what most users get for free when port forwarding works. "Relay Mode" is the fallback when CGN/port-forwarding fails — agent makes outbound connection, players connect to relay. The actual relay infrastructure is too large to fit in this phase.

### Tailscale integration

| Option | Description | Selected |
|--------|-------------|----------|
| Per-server Tailscale enable | Agent checks if Tailscale is up; offers to share Tailscale IP as fallback address; reuses enable_tailscale column | |
| Auto-install + authkey | Agent auto-installs Tailscale using tailscale_auth_key; binds Minecraft container to Tailscale interface | |
| Detect only, no install | Agent only DETECTS Tailscale; recommends manual setup | ✓ |

**User's choice:** Detect only. Use existing `enable_tailscale` column; surface Tailscale IP as fallback address when detected.

### Cloudflare Tunnel integration

| Option | Description | Selected |
|--------|-------------|----------|
| Detect only, manual setup | Agent detects cloudflared; registers as candidate fallback. Reuses Phase 51 credentials. | |
| Auto-install via cloudflared | Auto-install cloudflared (with consent), creates quick tunnel, registers trycloudflare URL | |
| Skip Cloudflare Tunnel entirely | Rely on Tailscale + future Esluce relay | |
| Detect + Experimental (user's spec) | Detect cloudflared and active tunnels; show diagnostics; no auto-install; not a primary fallback; marked as Experimental | ✓ |

**User's choice:** Detect + Experimental. Rationale: "Cloudflare Tunnel is optimized for HTTP/HTTPS workloads. Minecraft Java and Bedrock use TCP/UDP game traffic, making Cloudflare Tunnel less predictable than Direct Mode or a future Esluce Relay."

### Last-resort UX

| Option | Description | Selected |
|--------|-------------|----------|
| Clear error + manual instructions | Failure report + situation-specific guide | |
| Suggest Tailscale/Cloudflare | Last resort: suggest Tailscale with deep link | |
| Suggest Esluce Relay (waitlist) | 'Join Waitlist' button | |
| Hybrid (user's spec) | Failure report + situation-specific guidance + available fallback options + keep monitoring (auto-switch back to Direct) | ✓ |

**User's choice:** Hybrid. When all auto-fix attempts fail, show:
1. Clear failure report (error code, root cause, diagnostics, attempts performed)
2. Situation-specific guidance per detected scenario
3. Available fallback options (Tailscale/Cloudflare/Relay)
4. Keep monitoring — re-run periodic checks, auto-switch back to Direct Mode if connectivity restored

---

## User-facing diagnostic surface

### Status display

| Option | Description | Selected |
|--------|-------------|----------|
| Per-server badge only | Badge in Servers list and Server Details | |
| Per-server badge + diagnostic panel | Badge + new 'Connectivity' section in Server Details | ✓ |
| Global dashboard widget + per-server | Aggregate widget on main dashboard + per-server badge | |

**User's choice:** Per-server badge + new Connectivity section in Server Details page.

### Notifications

| Option | Description | Selected |
|--------|-------------|----------|
| Alerts + dashboard banners | Real-time in-app + email/Discord webhook + persistent banner | ✓ |
| Dashboard banner only | Persistent banner for any Unreachable server | |
| Status badge changes only | No active notifications, user must check badge | |

**User's choice:** Real-time in-app alerts on Reachable → Unreachable transition + optional email/Discord webhook (reuses existing alert infrastructure) + persistent dashboard banner.

### Action transparency

| Option | Description | Selected |
|--------|-------------|----------|
| Full visibility + audit log | Every auto-fix action logged with exact command and timestamp | ✓ |
| High-level summary only | Auto-fix actions summarized at high level (no exact commands) | |
| Silent | No audit trail; user only sees final result | |

**User's choice:** Full visibility + audit log. "Added iptables rule for port 25565 with comment esluse:server-uuid @ 2026-06-07T10:23:45Z".

### Manual probe trigger

| Option | Description | Selected |
|--------|-------------|----------|
| Reachable on demand | User can click 'Reachable' button in Connectivity section | ✓ |
| No manual trigger | No manual trigger; user waits for periodic checks | |

**User's choice:** "Reachable" button in Connectivity section to trigger a fresh reachability probe immediately. Useful after manual network changes.

---

## The Agent's Discretion

The following are agent's discretion (deferred to planner/executor):
- Exact `is_cgn_suspect` heuristic (e.g. RFC1918 + gateway IP check, or compare agent's external IP to gateway)
- UPnP mapping lease duration and renewal strategy
- Exact iptables / ufw / firewalld command syntax per distro family
- Periodic probe interval (recommended 5 min, may be different)
- Probe result retention period and history depth in the audit log
- Discord/email alert template wording
- "Reachable" button cooldown (avoid user spamming the backend probe)
- How to extract the Minecraft server version / motd from the probe response (for display)
- Specific shape of the diagnostic panel UI components

## Deferred Ideas

- **Custom Esluce Relay infrastructure** (`relay.esluce.com`, agent outbound tunnel, player routing) — deferred to follow-up phase. Phase 67 = agent-side only.
- **Auto-install / auto-configure Tailscale** — explicit user choice (D-11)
- **Auto-install / auto-configure Cloudflare Tunnel** — explicit user choice (D-12)
- **ISP_BLOCKED and PROTOCOL_MISMATCH failure modes** — deferred from 6 → 4 MVP
- **Two competing Server models refactor** (`.planning/debug/server-details-wrong-address-version-status.md`) — separate concern
- **IPv6 dual-stack port binding** — current `host_ip: 0.0.0.0` is IPv4-only; defer until users report IPv6 issues
- **Frontend Address/Version/Status data pipeline fix** (debug doc Option B/C) — related but separate fix
