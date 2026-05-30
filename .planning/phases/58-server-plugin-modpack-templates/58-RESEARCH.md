# Phase 58: Server, Plugin, and Modpack Templates - Research

**Researched:** 2026-05-31
**Domain:** Template engine, CurseForge/Modrinth API integration, DB schema design, agent-side plugin download
**Confidence:** HIGH

## Summary

Phase 58 delivers a full template system for game servers — pre-configured server templates (config + plugin/mod references), plugin/mod browser with Modrinth API integration, and modpack configurations. The existing codebase already has substantial groundwork: `Template`, `PluginTemplate`, and `ModpackTemplate` entities with fallback patterns, a `ModrinthClient` for search/download, and plugin install use cases. However, the server_templates and plugin_templates tables **do not exist in migrations** (only modpack_templates was migrated), meaning this phase must create those migrations plus a new consolidated `templates` table per D-04.

The Modrinth public API (v2) requires **no API key** for search and download — only a User-Agent header. CurseForge API **requires** an API key. Implementing CurseForge integration adds complexity; the existing codebase only uses Modrinth. The phase should use Modrinth only for the initial build (D-07/D-08 allow URLs as fallback, CurseForge API key management can be deferred).

**Primary recommendation:** Leverage the existing Template/PluginTemplate/ModpackTemplate domain models, ModrinthClient infrastructure, and plugin use cases. Extend with:
1. A **new `templates` table** (per D-04 JSONB config) with a migration
2. Full CRUD handlers + use cases for user-created templates
3. Agent protocol extension for plugin download during deployment
4. `CreateServerRequest.template_id` integration point in server creation flow
5. Frontend: `/templates` library, `/templates/create` editor, `/mods` browser pages

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Template CRUD | API / Backend | Database | Template storage + business logic exclusively server-side |
| Template list/search | API / Backend | — | Filtering by game_type, category, visibility via DB queries |
| Mod browser (search Modrinth) | API / Backend (proxy) | Frontend (UI) | Modrinth API called from Rust backend; results rendered client-side |
| Plugin download at deploy time | Agent (agent-core) | API (orchestration) | Agent runs on node where server container lives; API dispatches Task |
| Template creation UI | Browser / Client | — | Form renders in React, submits to API |
| Template library browsing | Browser / Client | — | React page with search/filter/pagination |
| Settings — API keys | API / Backend | Database | API keys stored in settings table, used by backend on searches |
| Built-in template seeding | Database (migration) | — | Seeded via SQL migration, not via startup script (D-12) |

## Standard Stack

### Core — Already Present in Codebase

| Library/Component | Location | Purpose | Why Standard |
|---|---|---|---|
| `Template` entity | `api/src/domain/server/template/model.rs` | Server template with game_type, variant, JSONB default_env | Existing model with fallback pattern |
| `PluginTemplate` entity | `api/src/domain/server/plugin_template/model.rs` | Plugin bundle template | Existing model with fallback |
| `ModpackTemplate` entity | `api/src/domain/server/modpack_template/model.rs` | Modpack from CurseForge/Modrinth | Existing model with fallback |
| `ModrinthClient` | `api/src/infrastructure/external_services/modrinth_client.rs` | Modrinth API v2 search + download | [VERIFIED: codebase — ModrinthClient implements search, version lookup, file download] |
| `SearchPluginsUseCase` | `api/src/application/use_cases/plugin_use_cases.rs` | Search Modrinth via backend proxy | Existing, used by plugin_handlers |
| `InstallPluginUseCase` | `api/src/application/use_cases/plugin_use_cases.rs` | Download and install plugin to server data dir | Existing, used by plugin_handlers |
| `SqlxServerRepository` | `api/src/domain/server/sqlx_repository.rs` | SQLx + FromRow pattern for DB access | [VERIFIED: codebase — the established repository pattern] |

### New Dependencies Needed

| Library | Version | Purpose | Why |
|---|---|---|---|
| `reqwest` | 0.12 (already in deps) | HTTP client for CurseForge API (if added) | Already used by ModrinthClient |
| No new Rust dependencies | — | — | All needs met by existing stack (serde_json for JSONB, sqlx for Postgres, uuid, chrono) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|---|---|---|
| Database-backed templates + fallback | All-hardcoded templates | DB allows user CRUD, visibility control, featured templates |
| Modrinth-only for mod sourcing | CurseForge + Modrinth | CurseForge requires API key + OAuth; more complexity. Modrinth covers most use cases with just User-Agent |
| Agent-side plugin download at deploy | Backend pre-downloads to shared volume | Agent-side is consistent with existing executors; avoids shared volume complexity |
| SQL migration to seed built-ins | Startup script | D-12 requires migration so built-ins can't be deleted; migration is correct |

**Installation:** No new npm or crate packages required — everything already in tree.

## Architecture Patterns

### System Architecture Diagram

```
                           ┌─────────────────────────────┐
                           │      Frontend (React)        │
                           │  /templates  /mods  /settings│
                           └──────────┬──────────────────┘
                                      │ REST /api/v1/
                                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    API Backend (Rust Axum)                       │
│                                                                  │
│  GET /templates         POST /templates      GET /mods/search   │
│  GET /templates/:id     PUT /templates/:id   GET /mods/versions │
│  POST /templates/:id/create-server                              │
│  PUT /settings/modrinth-api-key                                 │
│                                                                  │
│  ┌────────────┐  ┌──────────────┐  ┌────────────────────────┐   │
│  │ Use Cases  │  │ Repositories │  │ External Clients       │   │
│  │ · CRUD     │─▶│ · sqlx      │─▶│ · ModrinthClient       │   │
│  │ · Apply    │  │ · fallback  │  │ · (CurseForgeClient)   │   │
│  │ · Search   │  └──────┬───────┘  └───────────┬────────────┘   │
│  └────────────┘         │                       │                │
│                         ▼                       ▼                │
│                  ┌──────────────┐       Modrinth API             │
│                  │  PostgreSQL  │       api.modrinth.com         │
│                  │ · templates  │                                │
│                  │ · settings  │                                │
│                  └──────────────┘                                │
│                                                                  │
│  POST /servers ──────────────────────────────┐                   │
└──────────────────────────────────────────────┤                   │
                                               │ Task dispatch
                                               ▼
              ┌──────────────────────────────────────────┐
              │       Web Agent (agent-core)              │
              │  New task: "plugin_download"              │
              │  Downloads plugins from URLs to container │
              └──────────────────────────────────────────┘
```

### Recommended Project Structure

```
api/src/
├── domain/
│   └── server/
│       ├── template/               # EXISTS — extend with CRUD methods
│       │   ├── model.rs            # Template struct (id, game_type, category, config JSONB, visibility, user_id)
│       │   └── repository.rs       # Add create/update/delete methods + SqlxTemplateRepository
│       ├── plugin_template/        # EXISTS — already has CRUD
│       └── modpack_template/       # EXISTS — already has CRUD
├── application/
│   ├── use_cases/
│   │   └── template_use_cases.rs   # NEW — CRUD use cases for templates
│   └── dto/
│       └── template_dtos.rs        # NEW — CreateTemplateRequest, TemplateResponse, etc.
├── presentation/
│   ├── handlers/
│   │   ├── template_handlers.rs    # EXISTS — extend with full CRUD
│   │   ├── plugin_handlers.rs      # EXISTS — Modrinth search/version/install
│   │   └── settings_handlers.rs    # EXISTS — add Modrinth API key endpoints
│   └── routes/
│       └── api_routes.rs           # EXISTS — add template CRUD routes
├── bootstrap/
│   └── container.rs                # EXISTS — register new use cases
└── migrations/
    └── 20260531_create_templates_table.sql  # NEW — templates table

app/src/
├── pages/
│   ├── templates/
│   │   ├── TemplateLibraryPage.jsx     # NEW — browse templates
│   │   ├── TemplateCreatePage.jsx      # NEW — create/edit template
│   │   └── ModBrowserPage.jsx          # NEW — Modrinth mod browser
├── hooks/
│   ├── useTemplateLibrary.js           # NEW
│   └── useModBrowser.js                # NEW
├── api/
│   ├── client.js                       # EXISTS — add templatesApi methods
│   └── templatesApi.js                 # NEW
├── components/
│   ├── TemplateCard.jsx                # NEW — template card component
│   └── ModSearchResult.jsx             # NEW — mod search result component
└── app/
    └── App.jsx                         # EXTEND — add /templates, /mods routes
```

### Pattern 1: Entity + Repository + Fallback Pattern
**What:** Every template entity has a SQLx DB model with `#[derive(sqlx::FromRow)]`, a repository trait, a SQLx repository implementation, and a `fallback()` method returning hardcoded defaults when DB is empty.
**When to use:** Always — this is the established codebase pattern.
**Source:** [VERIFIED: api/src/domain/server/template/model.rs, repository.rs]

```rust
// Entity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: Uuid,
    pub game_type: String,    // "minecraft", "palworld", etc.
    pub category: String,     // "paper", "forge", "fabric" (sub-category)
    pub display_name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,  // JSONB - flexible config
    pub visibility: String,   // "public", "private"
    pub user_id: Option<Uuid>, // creator; None for built-in
    pub is_builtin: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Template {
    pub fn fallback() -> Vec<Self> { /* ... */ }
}
```

### Pattern 2: DTO + Use Case + Handler + Route Wiring
**When to use:** Every new endpoint. [VERIFIED: codebase — server_handlers.rs, plugin_handlers.rs, api_routes.rs]
**Example:**
```rust
// 1. DTO in application/dto/template_dtos.rs
pub struct CreateTemplateRequest {
    pub game_type: String,
    pub category: String,
    pub display_name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub visibility: Option<String>,
}

// 2. Use case in application/use_cases/template_use_cases.rs
use crate::domain::server::template::TemplateRepository;

pub struct CreateTemplateUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> CreateTemplateUseCase<R> {
    pub async fn execute(&self, user_id: Uuid, req: CreateTemplateRequest) -> Result<Template> { ... }
}

// 3. Handler
pub async fn create_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> { ... }

// 4. Route — in api_routes.rs
.route("/api/v1/templates", get(list_templates).post(create_template))
.route("/api/v1/templates/:id", get(get_template).put(update_template).delete(delete_template))
```

### Pattern 3: Fallback-to-DB Pattern
**When to use:** All template endpoints. [VERIFIED: template/repository.rs, plugin_template/repository.rs]
```rust
// Repository queries DB first, falls back to hardcoded templates if empty
async fn list_templates(&self) -> Result<Vec<Template>, ...> {
    let result = sqlx::query_as::<_, Template>("SELECT * FROM templates WHERE is_active = true ORDER BY game_type, category")
        .fetch_all(&self.pool)
        .await?;
    if result.is_empty() {
        tracing::info!("No templates found in database, using fallback");
        return Ok(Template::fallback());
    }
    Ok(result)
}
```

### Pattern 4: Settings Tab Config Section (Phase 56/57)
**When to use:** Template editing UI and Modrinth/CurseForge API key management in Settings.
**Source:** [VERIFIED: app/src/pages/ServerDetails.jsx — lines 447-507 (Sleep & Wake section)]
```jsx
<section className="glass-panel p-6 mt-6">
    <h3 className="text-lg font-bold mb-1">Section Title</h3>
    <p className="text-xs text-[var(--color-text-muted)] mb-5">Description</p>

    {toast && (
        <div className="mb-4 px-4 py-3 rounded-lg text-sm font-medium border ...">
            {toast.message}
        </div>
    )}

    {/* Toggle */}
    <div className="flex items-center gap-3 p-4 rounded-xl border cursor-pointer"
         onClick={() => setToggle(!toggle)}>
        <div className={`w-12 h-6 rounded-full transition-colors ${toggle ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-[var(--color-cosmic-border)]'}`}>
            <div className={`w-5 h-5 rounded-full bg-white transition-transform ${toggle ? 'translate-x-6' : 'translate-x-0.5'}`} />
        </div>
        <div className="flex-1">
            <p className="text-sm font-bold">Label</p>
            <p className="text-xs text-[var(--color-text-muted)]">Sub-label</p>
        </div>
    </div>

    {toggle && (
        <div className="mt-4">
            <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Input Label</label>
            <input type="text/number" value={value} onChange={...}
                className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border ..." />
        </div>
    )}

    <button disabled={saving} onClick={handleSave}
        className="mt-5 w-full py-2.5 rounded-lg text-sm font-bold bg-[var(--color-cosmic-cyan)]/10 ...">
        {saving ? 'Saving...' : 'Save Changes'}
    </button>
</section>
```

### Pattern 5: Agent Task Protocol Extension
**When to use:** Adding new agent-side operations (plugin download at deploy time).
**Source:** [VERIFIED: agent-core/crates/agent-proto/src/task.rs, messages.rs]

```rust
// agent-proto/src/task.rs — Task type already supports any task_type + payload
// To add plugin download, create a new task_type string like "plugin_download"
// with payload:
let task = Task::new(
    "plugin_download".to_string(),
    serde_json::json!({
        "server_id": server_id,
        "plugins": [
            {
                "url": "https://.../plugin.jar",
                "filename": "plugin.jar",
                "install_dir": "plugins/"
            }
        ]
    }),
);

// Backend dispatches via NodeClient::send_command_with_config (existing pattern)
// See server_handlers.rs lines 781-787 for send_command_with_config pattern
```

### Anti-Patterns to Avoid
- **Embedding full template data in CreateServerRequest:** Templates are snapshotted — store `template_id` reference but copy all template config into the server record.
- **Soft-deleting built-in templates:** D-12 says built-in templates "cannot be deleted"—enforce this in the delete handler by checking `is_builtin` flag.
- **Exposing secrets in template config:** Template config may include default env vars but should never include real API keys, tokens, or passwords.
- **Inline template selection during creation:** D-03 mandates browse-then-create separate flow, not inline selection.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Mod search/browse | Custom index | Modrinth API v2 proxy | [VERIFIED: codebase — ModrinthClient already exists and handles search, version resolution, file download] |
| Plugin file download | Custom download logic | `ModrinthClient::download_plugin()` | [VERIFIED: codebase — already implemented in modrinth_client.rs lines 154-192] |
| Server-side pagination/search | Custom SQL search | Add WHERE + LIMIT/OFFSET to template queries | Consistent with existing repository patterns |
| JSONB config validation | Manual struct validation | `serde_json::Value` with `#[serde(deserialize_with)]` for known schemas, open for game-specific config | Already the pattern — server config is `serde_json::Value` |
| Agent task dispatch | Custom WebSocket message | Existing `node_client.send_command_with_config()` pattern | [VERIFIED: server_handlers.rs — used for start/stop/restart/delete] |

**Key insight:** This phase leverages **six major pre-built components**: (1) Template/PluginTemplate/ModpackTemplate entities and repositories, (2) ModrinthClient for mod search and download, (3) plugin_use_cases for install/search/version resolution, (4) agent-task dispatcher for agent-side operations, (5) settings CRUD patterns for API key management, (6) Phase 56/57 Settings tab UI pattern for template editing.

## Common Pitfalls

### Pitfall 1: Template Snapshot Not Deep-Copied
**What goes wrong:** When applying a template to a new server, the template config is shallow-copied into the server record. If template config contains nested objects, mutations to the server's config could theoretically affect the template's config.
**Why it happens:** `serde_json::Value` cloning is actually deep, but `Clone` on the Template struct would clone references.
**How to avoid:** Use `serde_json::json!({...})` or `serde_json::Value::clone()` which performs a deep clone — JSONB values in serde_json are owned, so `template.config.clone()` is safe.
**Warning signs:** Server config changes affecting other servers or template defaults.

### Pitfall 2: Modrinth API Rate Limits
**What goes wrong:** Modrinth API has rate limits (default ~300 req/min for public endpoints). Aggressive polling during mod browsing can trigger 429 responses.
**Why it happens:** User typing in search box fires requests on every keystroke.
**How to avoid:** Already handled — the existing `usePluginSearch` hook in `usePlugins.js` uses a debounce pattern (line 30: `const debounceRef = useRef(null)`). Ensure mod browser also debounces.
**Warning signs:** 429 responses from Modrinth API in backend logs.

### Pitfall 3: Template Category <-> Docker Image Mismatch
**What goes wrong:** A template configured for "forge" variant with wrong docker image results in a non-functional server.
**Why it happens:** Docker image (e.g., `itzg/minecraft-server:latest`) and the `TYPE` env var must match. Paper needs `TYPE=PAPER`, Forge needs `TYPE=FORGE`, etc.
**How to avoid:** Validate that template `category` + `docker_image` + `default_env` are consistent. Use predefined template fallbacks as the source of truth.
**Warning signs:** Server starts but incorrect game type, or container crashes on startup.

### Pitfall 4: Missing Migration for New Templates Table
**What goes wrong:** The `server_templates` table referenced by `Template` model doesn't exist in the database yet — only in code as a fallback. Phase 58 must create the migration.
**Why it happens:** The `Template` model in `template/model.rs` queries `server_templates` table but no migration creates it. Similarly for `plugin_templates`.
**How to avoid:** Create migration `20260531_create_templates_table.sql` that creates the new consolidated `templates` schema from D-04, plus migration for `plugin_templates` if needed.

### Pitfall 5: Template Visibility + Multi-Tenant Access
**What goes wrong:** Private templates are visible to other users, or public templates are restricted.
**Why it happens:** No `user_id` filtering in template queries.
**How to avoid:** Filter by `visibility = 'public' OR user_id = $current_user_id` in list queries for non-admin users. Admins see everything.

## Code Examples

### Creating the Templates Table Migration (NEW)
```sql
-- 20260531_create_templates_table.sql
CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_type VARCHAR(50) NOT NULL,
    category VARCHAR(100) NOT NULL,   -- "paper", "forge", "fabric", "vanilla", etc.
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    config JSONB NOT NULL DEFAULT '{}',  -- Flexible config: env vars, port, docker image, plugin refs
    visibility VARCHAR(20) NOT NULL DEFAULT 'private' CHECK (visibility IN ('public', 'private')),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,  -- NULL for built-in templates
    is_builtin BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_templates_game_type ON templates(game_type, is_active);
CREATE INDEX IF NOT EXISTS idx_templates_user_id ON templates(user_id);
CREATE INDEX IF NOT EXISTS idx_templates_visibility ON templates(visibility);

-- Built-in templates (seed data)
INSERT INTO templates (game_type, category, display_name, description, config, visibility, is_builtin, is_active) VALUES
('minecraft', 'vanilla', 'Minecraft Vanilla', 'Default vanilla Minecraft server',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "VANILLA", "MEMORY": "2G", "MAX_PLAYERS": "20"}}'::jsonb,
 'public', true, true),
('minecraft', 'paper', 'Minecraft Paper', 'Paper server with optimized performance',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "PAPER", "MEMORY": "2G", "MAX_PLAYERS": "50"}}'::jsonb,
 'public', true, true);
```

### Template CRUD Handler
```rust
// Extending template_handlers.rs
pub async fn create_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let use_case = CreateTemplateUseCase::new(SqlxTemplateRepository::new(state.pool.clone()));
    let template = use_case.execute(auth_user.user_id, payload)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e)))?;
    Ok(Json(ApiResponse::success(template)))
}

pub async fn apply_template_to_server(
    State(state): State<ApiState>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<CreateServerFromTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Fetch template from DB
    // 2. Deep-clone template.config into CreateServerRequest
    // 3. Override with any user-specified overrides from payload
    // 4. Call CreateServerUseCase
    // 5. Dispatch plugin download tasks to agent if template has plugin refs
    // 6. Return created server
}
```

### Frontend Route Addition (App.jsx pattern)
```jsx
// In app/src/app/App.jsx inner <Routes>
<Route path="/templates" element={<TemplateLibraryPage />} />
<Route path="/templates/create" element={<TemplateCreatePage />} />
<Route path="/templates/:id" element={<TemplateDetailPage />} />
<Route path="/mods" element={<ModBrowserPage />} />

// Sidebar nav — add to the links section
<a href="/templates" className="block py-2 text-gray-400 hover:text-white">Templates</a>
<a href="/mods" className="block py-2 text-gray-400 hover:text-white">Mod Browser</a>
```

### API Client Addition (api.js pattern)
```javascript
// In app/src/lib/api.js
export const templatesApi = {
    list: (params) => api.get('/templates', { params }),
    get: (id) => api.get(`/templates/${id}`),
    create: (data) => api.post('/templates', data),
    update: (id, data) => api.put(`/templates/${id}`, data),
    delete: (id) => api.delete(`/templates/${id}`),
    createServer: (id, data) => api.post(`/templates/${id}/create-server`, data),
}

// Mod browser API
export const modsApi = {
    search: (params) => api.get('/plugins/search', { params }),
    getVersions: (projectId, params) => api.get(`/plugins/${projectId}/versions`, { params }),
}
```

### Agent Task for Plugin Download
The existing plugin install flow (`InstallPluginUseCase`) downloads plugins **on the backend server** (the API server), not on the game server node. For Phase 58, two approaches exist:

**Approach A (Existing — Backend-side download):** Use `InstallPluginUseCase` directly. Works when API server and game server share filesystem (Docker volumes). This is the current implementation.

**Approach B (Agent-side download):** Create a new task_type `"plugin_download"` and dispatch via node_client:
```rust
// Backend dispatches to agent
let params = CommandParams {
    container_name: Some(format!("mc-{}", server_id)),
    ..Default::default()
};
let deploy_config = DeployConfig {
    plugins: Some(vec![
        PluginSpec { url: "...", filename: "plugin.jar", install_dir: "plugins/" }
    ]),
    ..Default::default()
};
state.node_client.send_command_with_config(
    node_id, server_id, "plugin_download", params, Some(deploy_config),
).await?;
```

**Recommendation:** Use Approach A (existing backend-side download via shared volume) for Phase 58. It's already implemented and proven. Approach B can be added later if agent-side download becomes necessary for distributed deployments.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| Hardcoded server type in server creation form | Template-driven variant selection | Phase 58 | Template data drives dropdown options |
| Flat server creation form | Browse templates → Create from template | Phase 58 | D-03: separate flow for selection |
| Plugin search only in PluginManager | Dedicated /mods browser page | Phase 58 | D-09: full mod browser page |
| API key stored in env | API key managed via Settings page | Phase 58 | Consistent with Cloudflare/S3 config patterns |

**Deprecated/outdated:**
- The old `template_handlers.rs` `list_templates` handler only lists (GET) — extend to full CRUD
- The old `server_templates` and `plugin_templates` table references in code need actual SQL migrations

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | Modrinth API v2 requires only a User-Agent header, no API key | Standard Stack | If Modrinth adds auth requirement, need API key management in settings |
| A2 | Plugin install via backend-side download (shared volume) is sufficient for initial deployment | Code Examples | For remote/hybrid deployments where API server and node are separate, agent-side download needed |
| A3 | The existing `server_templates` table doesn't exist in migrations but the model in `template/model.rs` queries it | Common Pitfalls | Creating the migration will work — but check if any prior migration already created it |
| A4 | The CreateServerRequest already has a `config` field (serde_json::Value) that can be populated from template | Code Examples | If the config field is not wired through, need to extend the create server flow |

## Open Questions (RESOLVED)

1. **Plugin download at deployment: Backend-side or Agent-side?**
   - What we know: Existing `InstallPluginUseCase` downloads to backend filesystem at `{DATA_PATH}/servers/minecraft/{server_id}/plugins/`. Works when API server and game node share a volume (Docker compose setup).
   - What's unclear: For production with separate API server and remote nodes, plugins must be downloaded on the agent side.
   - **RESOLVED:** Use backend-side download for now (existing pattern). Add agent-side download task type as a follow-up enhancement. This aligns with the "URLs by default" approach from D-07.

2. **CurseForge API integration: When to add?**
   - What we know: CurseForge requires API key registration. Modrinth covers most Minecraft content.
   - What's unclear: Whether product requirement demands CurseForge integration in v1.
   - **RESOLVED:** Defer CurseForge. The mod browser works with Modrinth alone. Add CurseForge as `settings/curseforge-api-key` endpoint + client in a follow-up phase. The `source` field in ModpackTemplate already supports both ("curseforge" | "modrinth").

3. **What goes in template.config JSONB?**
   - What we know: Must include docker_image, default_port, env vars, optionally plugin references.
   - What's unclear: Should template.config be open schema (unvalidated JSON) or typed per game_type?
   - **RESOLVED:** Open schema with `serde_json::Value` — consistent with server `config` field. Document known keys in a Rust const.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| Node.js | Frontend build | ✓ | v20+ | — |
| Rust toolchain | Backend API | ✓ | 1.70+ (edition 2021) | — |
| PostgreSQL | DB storage | ✓ | via Docker | — |
| Modrinth API | Mod search/download | ✓ (public, no key) | v2 | Direct URL input |
| CurseForge API | CurseForge integration | ✗ (needs key) | — | Modrinth-only, defer to later phase |

**Missing dependencies with no fallback:** None — Modrinth API is public and works without a key.

**Missing dependencies with fallback:** CurseForge API (needs key) → Defer to later phase; use Modrinth-only for v1.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | yes | Existing VerifiedUser middleware for template CRUD |
| V4 Access Control | yes | Template ownership check (only creator edits/deletes private templates) |
| V5 Input Validation | yes | Deserialize via serde for struct validation; validate template.config is valid JSON |
| V6 Cryptography | no | No crypto operations — API keys stored as-is in settings table (existing pattern) |

### Known Threat Patterns for Axum + React

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Private template accessed by non-owner | Information Disclosure | Filter by `visibility = 'public' OR user_id = $current_user_id` |
| Built-in template deletion | Tampering | Check `is_builtin` flag in delete handler; return 403 if true |
| Path traversal in template config (plugin URLs) | Tampering | Validate plugin URL domains, ensure no `file://` or `../../` in filenames |
| Mass assignment via template.config | Tampering | Use explicit DTO fields, not raw config passthrough |

## Sources

### Primary (HIGH confidence)
- [VERIFIED: codebase — api/src/domain/server/template/] — Template entity, repository trait, SQLx implementation with fallback pattern
- [VERIFIED: codebase — api/src/domain/server/plugin_template/] — PluginTemplate entity with PluginConfig, fallback patterns
- [VERIFIED: codebase — api/src/domain/server/modpack_template/] — ModpackTemplate entity with source field
- [VERIFIED: codebase — api/src/infrastructure/external_services/modrinth_client.rs] — ModrinthClient implementing search, version resolution, file download
- [VERIFIED: codebase — api/src/application/use_cases/plugin_use_cases.rs] — SearchPluginsUseCase, GetPluginVersionsUseCase, InstallPluginUseCase
- [VERIFIED: codebase — agent-core/crates/agent-proto/src/task.rs] — Task and TaskResult protocol types
- [VERIFIED: codebase — api/migrations/20260504000001_create_modpack_templates.sql] — Existing modpack_templates migration pattern
- [VERIFIED: codebase — app/src/pages/ServerDetails.jsx] — Settings tab UI pattern (toggle + inputs + save + toast)
- [VERIFIED: codebase — app/src/lib/api.js] — API client pattern with serversApi, templatesApi stub
- [VERIFIED: codebase — app/src/app/App.jsx] — Frontend routing pattern

### Secondary (MEDIUM confidence)
- [CITED: docs.modrinth.com] — Modrinth API v2 is public, requires User-Agent only (no API key). Assumed based on existing code using no auth.
- [CITED: docs.curseforge.com] — CurseForge API requires API key registration. Assumed — not verified in current codebase.

### Tertiary (LOW confidence)
- [ASSUMED] — `server_templates` and `plugin_templates` table migrations don't exist — verified by grep across all .sql files; only `modpack_templates` migration found. If these tables somehow exist, the migration step is already done.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all components verified in codebase
- Architecture: HIGH — patterns verified across 10+ existing handlers, repositories, and entities
- Pitfalls: HIGH — based on codebase analysis and understanding of template snapshot semantics
- Modrinth API specifics: MEDIUM — API behavior verified via code, but exact rate limits/version format assumed public

**Research date:** 2026-05-31
**Valid until:** 2026-06-30 (codebase is active — stack versions stable)
