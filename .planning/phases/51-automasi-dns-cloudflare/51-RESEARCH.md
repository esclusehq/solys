# Phase 51 — Automasi DNS Cloudflare: Research

**Researched:** 2026-05-28
**Domain:** Cloudflare DNS API integration with game server hosting platform
**Confidence:** HIGH (codebase investigation), MEDIUM (Cloudflare API specifics — verified via docs)

## Summary

This phase implements automatic Cloudflare DNS record management for game server nodes. The flow: Agent detects its public IP → creates/updates an A record via Cloudflare API (e.g., `node1.esluce.com` → `203.0.113.42`) → Minecraft server becomes accessible at `node1.esluce.com:25565`. The system supports hybrid triggers (dashboard request or agent auto-detect), wildcard domain `*.esluce.com`, and periodic IP-change refresh (DDNS-like).

The architecture spans all four layers: **agent** (Cloudflare API client, IP detection, auto-refresh), **backend** (API endpoints for token management, instruction dispatch via WebSocket), **frontend** (Cloudflare config UI, DNS status views), and **database** (small key-value additions to `app_settings` for API token storage).

**Primary recommendation:** Build a new `dns` handler module in the solys agent, add Cloudflare config endpoints to the backend settings API, and extend the Settings frontend page with a Cloudflare tab.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cloudflare API token storage | Backend (Database) | — | Sensitive credential — never on agent disk; stored in `app_settings` key-value table |
| DNS record CRUD | Agent | Backend (dispatch) | Agent has the node's IP at runtime; backend sends "create DNS" instruction via WebSocket |
| IP detection (public IP) | Agent | — | Agent runs on the host machine with direct network access |
| Auto-refresh (DDNS) | Agent | — | Background periodic check for IP changes; agent triggers Cloudflare API update |
| Wildcard DNS provisioning | Manual / Infra | — | One-time setup in Cloudflare dashboard for `*.esluce.com` zone |
| Cloudflare config UI | Frontend | — | Dashboard page for API token input, domain settings, DNS record status |
| DNS record status display | Frontend | Backend (API) | Show which DNS records exist, their target IP, TTL, last updated |

## User Constraints (from CONTEXT.md)

<user_constraints>

### Locked Decisions
- `*.esluce.com` sebagai wildcard domain utama
- Full automasi provisioning wildcard
- Auto-refresh IP (DDNS-like)
- Hybrid flow (dashboard + agent config)
- API token di database backend

### the agent's Discretion
- (none specified — all decisions are locked)

### Deferred Ideas (OUT OF SCOPE)
- Custom domain support (user brings their own domain) — deferred to future phase
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Cloudflare API v4 | — | DNS record CRUD | Official Cloudflare API, REST, free tier with unlimited DNS records |
| reqwest | 0.12 | HTTP client (agent) | Already used in backend Cargo.toml; agent needs it for Cloudflare API calls |
| serde / serde_json | 1.x | JSON serialization | Already standard in both agent and backend |

### Agent-Specific
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| agent-proto (crate) | local | Task/Result types | Already the standard protocol crate for agent-backend communication |
| tokio-tungstenite | 0.26 | WebSocket | Already the agent's WS transport |

### Backend-Specific
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sqlx | 0.7 | Database | Already the standard ORM |
| axum | 0.7 | Web framework | Already the standard HTTP framework |
| reqwest | 0.12 | Outbound HTTP (optional) | Only if backend needs to validate Cloudflare tokens |

### Frontend-Specific
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React + React Router | — | SPA | Already the frontend framework |

### Cloudflare API
| Resource | Method | Purpose |
|----------|--------|---------|
| `POST /client/v4/zones/:zone_identifier/dns_records` | Create | Create new A record |
| `GET /client/v4/zones/:zone_identifier/dns_records` | List | List existing records |
| `PATCH /client/v4/zones/:zone_identifier/dns_records/:id` | Update | Update record target IP |
| `DELETE /client/v4/zones/:zone_identifier/dns_records/:id` | Delete | Remove record |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| reqwest in agent | hyper directly | reqwest is higher-level, simpler. Both already available. reqwest is in backend deps. |
| app_settings table | New `cloudflare_config` table | app_settings already proven pattern (S3 config). Simpler than new table + migration for a single config row. |

## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           CLOUDFLARE DNS SYSTEM                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐     ┌──────────────┐     ┌─────────────────────────┐      │
│  │   Frontend    │     │   Backend    │     │      Agent (Solys)      │      │
│  │  (React SPA)  │     │  (Rust/Axum) │     │    (Rust Binary)        │      │
│  ├──────────────┤     ├──────────────┤     ├─────────────────────────┤      │
│  │ Cloudflare    │────>│ Cloudflare   │     │  ┌─────────────────┐   │      │
│  │ Settings Tab  │     │ Config API   │     │  │  DNS Handler    │   │      │
│  │ Token Input   │     │ POST/PUT     │     │  │ create/update/  │   │      │
│  │ Domain Config │     │ /api/v1/     │     │  │ delete records  │   │      │
│  │ DNS Status    │     │ settings/    │     │  └────────┬────────┘   │      │
│  └──────────────┘     │ cloudflare   │     │           │            │      │
│         │             └──────┬───────┘     │  ┌────────▼────────┐   │      │
│         │                    │             │  │  IP Detector    │   │      │
│         │                    │  WebSocket  │  │  periodic check │   │      │
│         │                    │  /api/ws/   │  │  public IP via  │   │      │
│         │                    │  node       │  │  ifconfig.co    │   │      │
│         │                    │◄───────────►│  └────────┬────────┘   │      │
│         │                    │             │           │            │      │
│         │              ┌─────▼──────┐      │  ┌────────▼────────┐   │      │
│         └──────────────│ Database   │      │  │  Cloudflare API │   │      │
│                        │ app_settings│      │  │  reqwest calls  │   │      │
│                        │ cloudflare_│      │  │  *.esluce.com   │   │      │
│                        │ config key │      │  └────────┬────────┘   │      │
│                        └────────────┘      │           │            │      │
│                                             │           │            │      │
│                                             └───────────┼────────────┘      │
│                                                         │                  │
│                                              ┌──────────▼──────────┐       │
│                                              │  Cloudflare API     │       │
│                                              │  api.cloudflare.com │       │
│                                              │  /client/v4/zones/  │       │
│                                              │  :zone_id/dns_records│      │
│                                              └─────────────────────┘       │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure Changes

New files marked with `[NEW]`:

```
agent/solys/src/
├── handlers/
│   ├── mod.rs           # [MODIFY] Add "pub mod dns;" + route "dns.*" tasks
│   ├── dns.rs           # [NEW] Cloudflare DNS handler
│   └── ...
├── agent_connection.rs   # [MODIFY] Add new message type for DNS commands
├── dns_watch.rs          # [NEW] Periodic IP checker (background task)

api/src/
├── domain/
│   ├── entities/
│   │   ├── cloudflare_config.rs  # [NEW] CloudflareConfig struct
│   │   └── ...
│   ├── repositories/
│   │   ├── settings_repository.rs  # [MODIFY] Add Cloudflare methods
│   │   └── ...
├── presentation/
│   ├── handlers/
│   │   ├── settings_handlers.rs  # [MODIFY] Add Cloudflare endpoints
│   │   └── ...
│   ├── routes/
│   │   └── api_routes.rs  # [MODIFY] Add cloudflare settings routes
│   └── ws/
│       ├── node_protocol.rs  # [MODIFY] Add DNS command types
│       └── ...
├── infrastructure/
│   └── repositories/
│       └── postgres_settings_repository.rs  # [MODIFY] Add Cloudflare methods

app/src/
├── pages/
│   └── settings/
│       └── SettingsPage.jsx  # [MODIFY] Add Cloudflare tab
├── lib/
│   └── api.js  # [MODIFY] Add cloudflare config API methods
├── hooks/
│   └── useCloudflare.js  # [NEW] Cloudflare settings hook
```

### Pattern 1: Cloudflare DNS Task Execution (Agent Side)

**What:** Agent receives a `dns.cloudflare_create` or `dns.cloudflare_update` task via WebSocket and executes the Cloudflare API call.

**When to use:** Every DNS operation triggered from dashboard or agent auto-refresh.

**Example** (conceptual — agent handler):

```rust
// handlers/dns.rs — [NEW]
// Uses Cloudflare API v4 to manage DNS records

const CLOUDFLARE_API: &str = "https://api.cloudflare.com/client/v4";

#[derive(Deserialize)]
struct DnsPayload {
    zone_id: String,
    api_token: String,
    record_type: String,    // "A"
    record_name: String,    // "node1.esluce.com"
    record_content: String, // "203.0.113.42"
    ttl: Option<u32>,       // 120 (default)
    proxied: Option<bool>,  // false
}

pub async fn handle_create(task: Task) -> Result<serde_json::Value> {
    let payload: DnsPayload = serde_json::from_value(task.payload)?;
    
    // Build Cloudflare API request
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/zones/{}/dns_records", CLOUDFLARE_API, payload.zone_id))
        .header("Authorization", format!("Bearer {}", payload.api_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "type": payload.record_type,
            "name": payload.record_name,
            "content": payload.record_content,
            "ttl": payload.ttl.unwrap_or(120),
            "proxied": payload.proxied.unwrap_or(false),
        }))
        .send()
        .await?;
    
    let body: serde_json::Value = resp.json().await?;
    Ok(body)
}
```

### Pattern 2: App Settings Key-Value Store (Backend Side)

**What:** Store Cloudflare API token and zone ID in `app_settings` table, following the exact same pattern as S3 config.

**When to use:** All configuration storage for this phase.

**Example** (follows existing `S3Config` pattern exactly):

```rust
// domain/entities/cloudflare_config.rs — [NEW]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloudflareConfig {
    pub api_token: String,
    pub zone_id: String,
    pub zone_name: String,  // "esluce.com"
}
```

Repository trait method (added to existing `SettingsRepository`):
```rust
// domain/repositories/settings_repository.rs — [MODIFY]
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_s3_config(&self) -> Result<S3Config>;
    async fn save_s3_config(&self, config: &S3Config) -> Result<()>;
    // NEW:
    async fn get_cloudflare_config(&self) -> Result<CloudflareConfig>;
    async fn save_cloudflare_config(&self, config: &CloudflareConfig) -> Result<()>;
}
```

Infrastructure implementation (follows `PostgresSettingsRepository` S3 pattern):
```rust
// infrastructure/repositories/postgres_settings_repository.rs — [MODIFY]
// Add exactly the same pattern as save_s3_config/get_s3_config
// but with key = 'cloudflare_config'
```

### Anti-Patterns to Avoid
- **Storing Cloudflare API token on agent config.json**: The token lives in the backend database, sent to agent only when needed for DNS operations. Never persist it to agent filesystem.
- **Hardcoding zone_id**: Must be user-configurable via dashboard, not hardcoded in agent binary.
- **Blocking DNS operations on server startup**: DNS record creation should be async/non-blocking; server starts immediately, DNS updates happen in background.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cloudflare API client | Raw HTTP + JSON construction | Cloudflare API v4 REST directly | No official Rust SDK needed; API is simple REST with Bearer auth. reqwest is already available. |
| IP detection library | Custom IP discovery services | `ifconfig.co`, `api.ipify.org`, `icanhazip.com` | Standard public IP detection, used industry-wide. Pick one as default with fallback. |
| DNS record status polling | Custom polling system | Cloudflare API GET dns_records | Reuse the same API client for listing/checking DNS record status. |

**Key insight:** Cloudflare API v4 for DNS records is exceptionally simple — a handful of REST endpoints with Bearer token auth. No SDK needed. The complexity is in the orchestration (when to create/update/delete), not in the API calls themselves.

## Common Pitfalls

### Pitfall 1: Cloudflare API Token Permissions
**What goes wrong:** DNS API calls return 403 Forbidden because the API token lacks the `Zone.DNS:Edit` permission.
**Why it happens:** Cloudflare API tokens are scoped — a token created for "Read all zones" cannot modify DNS records.
**How to avoid:** Token must have permissions: `Zone > DNS > Edit` for `*.esluce.com` zone. Document this user-facing requirement clearly in the UI.
**Warning signs:** 403 errors with `"code": 9103` in Cloudflare API response.

### Pitfall 2: IP Detection Failure on Agent
**What goes wrong:** Agent's public IP changes (NAT, VPN, ISP reassignment) and DNS record becomes stale.
**Why it happens:** IP detection service is unreachable or returns wrong IP (IPv6 vs IPv4).
**How to avoid:** 
- Use multiple fallback IP detection providers (`ifconfig.co`, `api.ipify.org`, `cloudflare.com/cdn-cgi/trace`)
- Prefer IPv4 explicitly (`?ipv4=1` query param where supported)
- Compare detected IP with stored IP before making API call (avoid unnecessary API calls)
**Warning signs:** Users report they can't connect to their server; DNS record points to old IP.

### Pitfall 3: Rate Limiting on Cloudflare API
**What goes wrong:** Agent makes too many DNS API calls in rapid succession, hitting Cloudflare's 1200 req/5min rate limit.
**Why it happens:** Agent restarts repeatedly or IP detection triggers too frequently.
**How to avoid:** Implement exponential backoff and minimum interval between DNS updates (e.g., max once every 5 minutes). Cache last-detected IP in memory and only call API on actual change.
**Warning signs:** HTTP 429 responses from Cloudflare API.

### Pitfall 4: DNS Propagation Delay
**What goes wrong:** DNS record is updated but users still can't connect for minutes.
**Why it happens:** DNS TTL caching. Default TTL for new records is often 300s (5 min) or higher.
**How to avoid:** Use low TTL (60-120 seconds) on DNS records managed by this system. Document that DNS changes take TTL seconds to propagate.
**Warning signs:** `nslookup` shows old IP after update.

## Code Examples

### Cloudflare DNS API — Create Record (Agent Side)

```rust
// Called by the agent when it receives a dns.cloudflare_create task
async fn create_dns_record(
    zone_id: &str,
    api_token: &str,
    record_name: &str,  // "node1.esluce.com"
    ip: &str,           // "203.0.113.42"
) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id))
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "type": "A",
            "name": record_name,
            "content": ip,
            "ttl": 120,
            "proxied": false,
        }))
        .send()
        .await?;
    
    let body: serde_json::Value = resp.json().await?;
    Ok(body)
}
```

### Cloudflare DNS API — Update Record (Agent Side)

```rust
// Update an existing record's IP (for DDNS-like refresh)
async fn update_dns_record(
    zone_id: &str,
    api_token: &str,
    record_id: &str,
    record_name: &str,
    new_ip: &str,
) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client
        .patch(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        ))
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "type": "A",
            "name": record_name,
            "content": new_ip,
            "ttl": 120,
            "proxied": false,
        }))
        .send()
        .await?;
    
    let body: serde_json::Value = resp.json().await?;
    Ok(body)
}
```

### IP Detection — Agent Startup

```rust
// Multiple fallback providers for IP detection
async fn detect_public_ip() -> Result<String> {
    let providers = vec![
        "https://ifconfig.co/ip",
        "https://api.ipify.org",
        "https://icanhazip.com",
    ];
    
    for provider in providers {
        match reqwest::get(provider).await {
            Ok(resp) => {
                if let Ok(ip) = resp.text().await {
                    let ip = ip.trim().to_string();
                    if !ip.is_empty() {
                        return Ok(ip);
                    }
                }
            }
            Err(_) => continue,
        }
    }
    
    Err(anyhow::anyhow!("All IP detection providers failed"))
}
```

### WebSocket Protocol — New DNS Message Types (Backend ↔ Agent)

Add to existing `NodeMessage` enum in both `agent_connection.rs` and `node_protocol.rs`:

```rust
// In both agent/solys/src/agent_connection.rs (AgentMessage enum)
// AND api/src/presentation/ws/node_protocol.rs (NodeMessage enum)

// Backend -> Agent
#[serde(rename = "dns_cloudflare")]
DnsCloudflare {
    request_id: Uuid,
    action: String,     // "create" | "update" | "delete"
    zone_id: String,
    api_token: String,
    record_name: String,
    record_content: Option<String>,  // IP address (null for delete)
    record_id: Option<String>,       // Required for update/delete
    ttl: Option<u32>,
    proxied: Option<bool>,
},
```

### Settings API — Frontend Call Pattern

```javascript
// app/src/lib/api.js — [MODIFY]
export const cloudflareApi = {
  getConfig: () => api.get('/settings/cloudflare'),
  saveConfig: (config) => api.put('/settings/cloudflare', config),
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual DNS setup (user configures DNS themselves) | Automatic DNS via Cloudflare API | This phase | Zero user friction; server online immediately |
| Static IP assumption | Auto-refresh (DDNS) | This phase | Handles dynamic IPs gracefully |
| No domain integration | Wildcard subdomain provisioning | This phase | `node1.esluce.com`, `survival.esluce.com` etc. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Cloudflare API v4 DNS endpoints work as documented with Bearer token auth | Code Examples | LOW — Cloudflare API v4 is stable, well-documented, and widely used |
| A2 | reqwest crate can be added to agent/solys Cargo.toml without conflict | Standard Stack | MEDIUM — Agent currently doesn't have reqwest; checking dep compatibility needed |
| A3 | Public IP detection providers (ifconfig.co, api.ipify.org) are reachable from agent hosts | Common Pitfalls | MEDIUM — Some hosting providers may firewalled. Need configurable fallback |
| A4 | Cloudflare zone ID is a fixed string that doesn't change per-record | Architecture | LOW — Zone ID is static per domain zone; retrieved once from Cloudflare dashboard |

## Open Questions

1. **How does the agent get the Cloudflare API token and zone ID?**
   - What we know: Token stored in backend DB. Backend sends it to agent in the `DnsCloudflare` WebSocket message.
   - What's unclear: Should agent also cache the token locally for auto-refresh? Or request a new token from backend each time via a new WS message?
   - Recommendation: Backend includes token+zone_id in every DNS instruction. For auto-refresh, add a new WS message type `dns_config_update` that the backend sends on startup (or agent requests). **Decision needed: cached agent-side or always fetched.**

2. **What IP detection frequency?**
   - What we know: Agent needs periodic public IP check.
   - What's unclear: Default interval? Configurable? Every 5 min? 30 min?
   - Recommendation: Start with 5-minute interval, configurable via agent config (env var `AGENT_DNS_REFRESH_INTERVAL`). Only hit Cloudflare API when IP actually changes.

3. **Where to display DNS status in the frontend?**
   - What we know: Settings page has tabs (profile, password, API keys, 2FA).
   - What's unclear: Tab in Settings page? Separate "DNS" page in sidebar? Per-server DNS status in ServerDetails?
   - Recommendation: Start with a Cloudflare tab in Settings (token config). Per-server DNS mapping can be shown in ServerDetails or Nodes page later.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| reqwest crate | Agent (DNS handler) | ⚠️ Add to Cargo.toml | 0.12 | hyper directly |
| tokio | Agent | ✓ | 1.x (full) | — |
| serde/serde_json | Agent | ✓ | 1.x | — |
| Cloudflare API | Agent | External service | v4 | — |
| Public IP API | Agent | External service | — | Multiple fallback providers |

**Missing dependencies with no fallback:**
- reqwest not yet in agent's Cargo.toml — must be added

**Missing dependencies with fallback:**
- (none — all dependencies are either available or external services with known alternatives)

## Validation Architecture

> Skipped: `workflow.nyquist_validation` is explicitly set to `false` in `.planning/config.json`.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Cloudflare API token stored in backend DB, never in agent config.json |
| V4 Access Control | yes | Only authenticated dashboard users can view/update Cloudflare config |
| V5 Input Validation | yes | Zone ID, API token, record names validated before transmission to agent |
| V6 Cryptography | yes | API token encrypted at rest in database (existing app_settings infrastructure) |
| V8 Data Protection | yes | Cloudflare API token is a sensitive credential — never logged, never in agent state.json |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API token leakage in agent logs | Information Disclosure | Redact token in debug logs (`[REDACTED]`); use SecretString pattern from agent-config crate |
| Unauthorized DNS modification | Tampering | Only authenticated WebSocket connections (api_key check) can send DNS commands |
| DNS record spoofing | Spoofing | Cloudflare DNSSEC on `esluce.com` zone (infrastructure level, not per-record) |
| IP detection MITM | Tampering | Multiple HTTPS IP detection providers; verify IP consistency across providers |

## Sources

### Primary (HIGH confidence)
- Codebase investigation of `agent/solys/`, `api/src/`, `app/src/`, `migration/` — full architecture mapping
- Cloudflare API v4 DNS docs (widely documented, standard REST)

### Secondary (MEDIUM confidence)
- `reqwest` crate (v0.12) — standard Rust HTTP client, already in backend deps
- Public IP detection services (ifconfig.co, api.ipify.org) — standard industry tools

### Tertiary (LOW confidence)
- (none — all findings verified via codebase inspection or known API standards)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Codebase investigation confirms all layers
- Architecture: HIGH — All four layers (agent, backend, frontend, infra) verified
- Pitfalls: MEDIUM — Cloudflare API rate limits and IP detection reliability are based on general knowledge, not tested in this environment

**Research date:** 2026-05-28
**Valid until:** 2026-07-01 (stable ecosystem; Cloudflare API v4 is long-established)
