# Phase 58: Server, Plugin, and Modpack Templates - Pattern Map

**Mapped:** 2026-05-31
**Files analyzed:** 21 (8 new, 13 modify)
**Analogs found:** 21 / 21

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `api/src/domain/server/template/model.rs` | entity | CRUD | `api/src/domain/server/modpack_template/model.rs` | exact (role+dataflow) |
| `api/src/domain/server/template/repository.rs` | repository | CRUD | `api/src/domain/server/modpack_template/repository.rs` | exact |
| `api/src/application/dto/template_dtos.rs` | dto | CRUD | `api/src/application/dto/server_dtos.rs` | role-match |
| `api/src/application/use_cases/template_use_cases.rs` | service | CRUD | `api/src/application/use_cases/plugin_use_cases.rs` | role-match |
| `api/src/presentation/handlers/template_handlers.rs` | controller | CRUD | `api/src/presentation/handlers/plugin_handlers.rs` | exact (role+dataflow) |
| `api/src/presentation/handlers/settings_handlers.rs` | controller | request-response | `api/src/presentation/handlers/settings_handlers.rs` | exact (extends same file) |
| `api/src/presentation/routes/api_routes.rs` | route | CRUD | itself (existing routes) | exact |
| `api/src/bootstrap/container.rs` | config | CRUD | itself (existing container) | exact |
| `api/src/application/dto/server_dtos.rs` | dto | CRUD | itself (existing dto) | exact |
| `api/src/application/use_cases/create_server_use_case.rs` | service | CRUD | itself (existing use case) | exact |
| `api/src/migrations/20260531_create_templates_table.sql` | migration | — | `api/migrations/20260504000001_create_modpack_templates.sql` | exact |
| `app/src/pages/templates/TemplateLibraryPage.jsx` | component | request-response | `app/src/pages/servers/ServerManagerPage.jsx` | role-match |
| `app/src/pages/templates/TemplateCreatePage.jsx` | component | request-response | `app/src/pages/ServerDetails.jsx` (Settings tab) | role-match |
| `app/src/pages/templates/ModBrowserPage.jsx` | component | request-response | `app/src/pages/servers/ServerManagerPage.jsx` | role-match |
| `app/src/hooks/useTemplateLibrary.js` | hook | CRUD | `app/src/hooks/useServers.js` | exact |
| `app/src/hooks/useModBrowser.js` | hook | request-response | `app/src/hooks/usePlugins.js` (usePluginSearch) | exact |
| `app/src/api/templatesApi.js` | utility | CRUD | `app/src/lib/api.js` (templatesApi block) | exact |
| `app/src/components/TemplateCard.jsx` | component | request-response | existing card patterns in ServerManagerPage.jsx | partial |
| `app/src/components/ModSearchResult.jsx` | component | request-response | `app/src/pages/ServerManager.jsx` server rows | partial |
| `app/src/app/App.jsx` | route | — | itself (existing routes) | exact |
| `app/src/lib/api.js` | utility | CRUD | itself (existing api exports) | exact |

---

## Pattern Assignments

### `api/src/domain/server/template/model.rs` — EXTEND (entity)

**Analog:** `api/src/domain/server/modpack_template/model.rs`
**Current: `api/src/domain/server/template/model.rs`**

**Imports pattern** (lines 1-3):
```rust
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
```

**sqlx::FromRow entity pattern** (lines 7-21) — Extend existing Template struct to match D-04 schema:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: Uuid,
    pub game_type: String,
    pub category: String,      // NEW: was "variant"
    pub display_name: String,
    pub description: Option<String>,
    // REPLACE individual fields with JSONB config:
    pub config: serde_json::Value,  // NEW: replaces docker_image, default_port, default_env, default_startup_command
    pub visibility: String,    // NEW: "public" | "private"
    pub user_id: Option<Uuid>, // NEW: NULL for built-in
    pub is_builtin: bool,      // NEW
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

**Fallback pattern** (lines 26-195 of modpack_template/model.rs):
```rust
impl Template {
    pub fn fallback() -> Vec<Self> {
        let now = Utc::now().naive_utc();
        vec![
            Template {
                id: Uuid::new_v4(),
                game_type: "minecraft".to_string(),
                category: "vanilla".to_string(),
                display_name: "Minecraft Vanilla".to_string(),
                description: Some("Default vanilla Minecraft server".to_string()),
                config: serde_json::json!({
                    "docker_image": "itzg/minecraft-server:latest",
                    "default_port": 25565,
                    "env": { "TYPE": "VANILLA", "MEMORY": "2G", "MAX_PLAYERS": "20" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            // ... more entries
        ]
    }
}
```

---

### `api/src/domain/server/template/repository.rs` — EXTEND (+CRUD methods)

**Analog:** `api/src/domain/server/modpack_template/repository.rs`

**Imports pattern** (lines 1-5):
```rust
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::model::Template;
```

**Repository trait** — Add create/update/delete + user-scoped query methods:
```rust
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    // Existing methods:
    async fn list_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_templates_by_game(&self, game_type: &str) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_template_by_id(&self, id: Uuid) -> Result<Option<Template>, Box<dyn std::error::Error + Send + Sync>>;
    
    // NEW CRUD methods:
    async fn create_template(&self, template: &Template) -> Result<Template, Box<dyn std::error::Error + Send + Sync>>;
    async fn update_template(&self, template: &Template) -> Result<Template, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete_template(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn list_templates_by_user(&self, user_id: Uuid) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_public_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
}
```

**SqlxTemplateRepository pattern** (lines 17-25 of modpack_template/repository.rs):
```rust
pub struct SqlxTemplateRepository {
    pool: PgPool,
}

impl SqlxTemplateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

**Fallback-to-DB query pattern** (lines 31-51 of modpack_template/repository.rs):
```rust
async fn list_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query_as::<_, Template>(
        r#"
        SELECT id, game_type, category, display_name, description, config,
               visibility, user_id, is_builtin, is_active,
               created_at, updated_at
        FROM templates
        WHERE is_active = true
        ORDER BY game_type, category
        "#
    )
    .fetch_all(&self.pool)
    .await?;

    if result.is_empty() {
        tracing::info!("No templates found in database, using fallback templates");
        return Ok(Template::fallback());
    }

    Ok(result)
}
```

**Visibility-scoped query pattern** (for multi-tenant filtering):
```rust
async fn list_public_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query_as::<_, Template>(
        r#"
        SELECT id, game_type, category, display_name, description, config,
               visibility, user_id, is_builtin, is_active,
               created_at, updated_at
        FROM templates
        WHERE is_active = true AND (visibility = 'public' OR is_builtin = true)
        ORDER BY is_builtin DESC, game_type, category
        "#
    )
    .fetch_all(&self.pool)
    .await?;
    // ... fallback logic
}
```

---

### `api/src/application/dto/template_dtos.rs` — NEW (DTOs)

**Analog:** `api/src/application/dto/server_dtos.rs`

**Imports pattern:**
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
```

**CreateTemplateRequest pattern** (lines 5-25 of server_dtos.rs):
```rust
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub game_type: String,
    pub category: String,
    pub display_name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub visibility: Option<String>,   // defaults to "private"
}
```

**UpdateTemplateRequest pattern:**
```rust
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub visibility: Option<String>,
    pub category: Option<String>,
}
```

**TemplateResponse pattern** (lines 155-201 of server_dtos.rs):
```rust
#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub game_type: String,
    pub category: String,
    pub display_name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub visibility: String,
    pub user_id: Option<Uuid>,
    pub is_builtin: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
```

**CreateServerFromTemplateRequest** (for applying template to server):
```rust
#[derive(Debug, Deserialize)]
pub struct CreateServerFromTemplateRequest {
    pub name: String,
    pub node_id: Option<String>,
    #[serde(default)]
    pub config_overrides: Option<serde_json::Value>,  // user overrides on top of template config
}
```

---

### `api/src/application/use_cases/template_use_cases.rs` — NEW (use cases)

**Analog:** `api/src/application/use_cases/plugin_use_cases.rs`

**Imports pattern:**
```rust
use std::sync::Arc;
use anyhow::{Result, anyhow};
use uuid::Uuid;

use crate::domain::server::template::{TemplateRepository, SqlxTemplateRepository, Template};
use crate::application::dto::template_dtos::*;
```

**CreateTemplateUseCase pattern** (lines 100-108 of plugin_use_cases.rs):
```rust
pub struct CreateTemplateUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> CreateTemplateUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, req: CreateTemplateRequest) -> Result<Template> {
        let now = chrono::Utc::now().naive_utc();
        let template = Template {
            id: Uuid::new_v4(),
            game_type: req.game_type,
            category: req.category,
            display_name: req.display_name,
            description: req.description,
            config: req.config,
            visibility: req.visibility.unwrap_or_else(|| "private".to_string()),
            user_id: Some(user_id),
            is_builtin: false,
            is_active: true,
            created_at: now,
            updated_at: now,
        };
        self.repository.create_template(&template).await?;
        Ok(template)
    }
}
```

**ListTemplatesUseCase pattern:**
```rust
pub struct ListTemplatesUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> ListTemplatesUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, game_type: Option<String>) -> Result<Vec<Template>> {
        // Show public templates + user's own templates
        let mut templates = self.repository.list_public_templates().await?;
        let user_templates = self.repository.list_templates_by_user(user_id).await?;
        templates.extend(user_templates);
        // Deduplicate by id
        templates.sort_by(|a, b| a.id.cmp(&b.id));
        templates.dedup_by(|a, b| a.id == b.id);
        
        if let Some(gt) = game_type {
            templates.retain(|t| t.game_type == gt);
        }
        
        Ok(templates)
    }
}
```

---

### `api/src/presentation/handlers/template_handlers.rs` — EXTEND (full CRUD)

**Analog:** `api/src/presentation/handlers/plugin_handlers.rs` + existing `template_handlers.rs`

**Imports pattern** (lines 1-11 of existing + add auth):
```rust
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use serde::Deserialize;

use crate::domain::auth::middleware::VerifiedUser;
use crate::domain::server::template::{SqlxTemplateRepository, TemplateRepository};
use crate::application::dto::template_dtos::*;
use crate::presentation::routes::api_routes::ApiState;
use crate::presentation::responses::api_response::ApiResponse;
use crate::shared::errors::app_error::AppError;
```

**CRUD handler pattern** (follow server_handlers.rs create_server pattern — lines 397-529):
```rust
pub async fn create_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repository = SqlxTemplateRepository::new(state.pool.clone());
    let use_case = CreateTemplateUseCase::new(repository);
    let template = use_case.execute(auth_user.user_id, payload)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e)))?;
    Ok(Json(ApiResponse::success(template)))
}
```

**Auth-protected create/update/delete pattern** (follow server_handlers.rs lines 593-606):
```rust
pub async fn delete_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let repository = SqlxTemplateRepository::new(state.pool.clone());
    let template = repository.get_template_by_id(id).await
        .map_err(|e| AppError::InternalError(e.into()))?
        .ok_or(AppError::NotFound)?;

    // Built-in templates cannot be deleted (D-12)
    if template.is_builtin {
        return Err(AppError::BadRequest("Built-in templates cannot be deleted".into()));
    }

    // Ownership check
    if template.user_id != Some(auth_user.user_id) && !auth_user.is_admin {
        return Err(AppError::Forbidden);
    }

    repository.delete_template(id).await
        .map_err(|e| AppError::InternalError(e.into()))?;

    Ok(Json(ApiResponse::success(serde_json::json!({ "status": "deleted" }))))
}
```

**Apply template to server pattern:**
```rust
pub async fn apply_template_to_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<CreateServerFromTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Fetch template
    let repository = SqlxTemplateRepository::new(state.pool.clone());
    let template = repository.get_template_by_id(template_id).await
        .map_err(|e| AppError::InternalError(e.into()))?
        .ok_or(AppError::NotFound)?;

    // 2. Deep-clone template.config into server config + apply overrides
    let mut server_config = template.config.clone();
    if let Some(overrides) = &payload.config_overrides {
        if let Some(obj) = overrides.as_object() {
            for (k, v) in obj {
                server_config[k] = v.clone();
            }
        }
    }

    // 3. Call existing create_server flow with config pre-filled
    // (See create_server_use_case.rs integration point)

    Ok(Json(ApiResponse::success(serde_json::json!({ "status": "server_created" }))))
}
```

---

### `api/src/presentation/handlers/settings_handlers.rs` — EXTEND (Modrinth API key)

**Analog:** `api/src/presentation/handlers/settings_handlers.rs` (Cloudflare config section — lines 70-150)

**Modrinth API key handler pattern** (follow Cloudflare config pattern):
```rust
/// GET /settings/modrinth-api-key
pub async fn get_modrinth_api_key(
    State(container): State<ApiState>,
    user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    match container.settings_repository.get_modrinth_api_key().await {
        Ok(key) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": {
                "api_key_set": !key.is_empty(),
            }
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// PUT /settings/modrinth-api-key
pub async fn save_modrinth_api_key(
    State(container): State<ApiState>,
    user: AuthUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({...}))));
    }
    // save pattern same as save_s3_config (lines 40-68)
    let api_key = payload.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
    match container.settings_repository.save_modrinth_api_key(api_key).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({ "success": true, "data": null })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({...})))),
    }
}
```

---

### `api/src/presentation/routes/api_routes.rs` — EXTEND (add template routes)

**Route pattern** — Add alongside existing template route (lines 91-92):
```rust
// Replace existing single GET route with full CRUD:
.route("/api/v1/templates", get(crate::presentation::handlers::template_handlers::list_templates)
    .post(crate::presentation::handlers::template_handlers::create_template))
.route("/api/v1/templates/:id", get(crate::presentation::handlers::template_handlers::get_template)
    .put(crate::presentation::handlers::template_handlers::update_template)
    .delete(crate::presentation::handlers::template_handlers::delete_template))
.route("/api/v1/templates/:id/create-server", post(crate::presentation::handlers::template_handlers::apply_template_to_server))

// Modrinth API key settings:
.route("/api/v1/settings/modrinth-api-key", 
    get(crate::presentation::handlers::settings_handlers::get_modrinth_api_key)
    .put(crate::presentation::handlers::settings_handlers::save_modrinth_api_key))
```

---

### `api/src/bootstrap/container.rs` — EXTEND (register template use cases)

**Container registration pattern** (follow plugin use cases — lines 260-264):
```rust
// Add template repository and use cases:
use crate::domain::server::template::SqlxTemplateRepository;
use crate::application::use_cases::template_use_cases::*;

// In AppContainer struct (add new fields):
pub template_repository: Arc<dyn TemplateRepository>,
pub create_template_use_case: Arc<CreateTemplateUseCase<dyn TemplateRepository>>,
pub list_templates_use_case: Arc<ListTemplatesUseCase<dyn TemplateRepository>>,
pub get_template_use_case: Arc<GetTemplateUseCase<dyn TemplateRepository>>,
pub update_template_use_case: Arc<UpdateTemplateUseCase<dyn TemplateRepository>>,
pub delete_template_use_case: Arc<DeleteTemplateUseCase<dyn TemplateRepository>>,

// In AppContainer::new() init pattern (follow lines 261-264):
let template_repository = Arc::new(SqlxTemplateRepository::new(pool.clone()));
let template_repo: Arc<dyn TemplateRepository> = template_repository;

let create_template_use_case = Arc::new(CreateTemplateUseCase::new(template_repo.clone()));
let list_templates_use_case = Arc::new(ListTemplatesUseCase::new(template_repo.clone()));
// ... etc
```

---

### `api/src/application/dto/server_dtos.rs` — EXTEND (add template_id)

**Analog:** Existing file itself

**Add to CreateServerRequest** (after line 51):
```rust
#[serde(default)]
pub template_id: Option<Uuid>,  // NEW: for template-based creation
```

---

### `api/src/application/use_cases/create_server_use_case.rs` — EXTEND (template snapshot)

**Extension point** — After entity creation, if `template_id` is set, deep-clone template config:
```rust
// At beginning of execute(), before creating Server:
if let Some(template_id) = &req.template_id {
    // Fetch template and merge config into server fields
    let template_repo = SqlxTemplateRepository::new(...); // or inject
    let template = template_repo.get_template_by_id(*template_id).await?;
    if let Some(t) = template {
        // Deep-clone config into the new server
        let config = t.config.clone();
        // Override with user-provided values from req
        // ... merge logic
    }
}
```

---

### `api/src/migrations/20260531_create_templates_table.sql` — NEW (migration)

**Analog:** `api/migrations/20260504000001_create_modpack_templates.sql`

**SQL migration pattern** (lines 1-29 of modpack migration):
```sql
-- Create templates table for storing pre-configured server configurations
CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_type VARCHAR(50) NOT NULL,
    category VARCHAR(100) NOT NULL,        -- "paper", "forge", "vanilla", etc.
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    config JSONB NOT NULL DEFAULT '{}',     -- Flexible config: docker_image, env vars, port, plugin refs
    visibility VARCHAR(20) NOT NULL DEFAULT 'private' CHECK (visibility IN ('public', 'private')),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,  -- NULL for built-in templates
    is_builtin BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_templates_game_type ON templates(game_type, is_active);
CREATE INDEX IF NOT EXISTS idx_templates_user_id ON templates(user_id);
CREATE INDEX IF NOT EXISTS idx_templates_visibility ON templates(visibility);

-- Seed built-in templates (per D-12)
INSERT INTO templates (game_type, category, display_name, description, config, visibility, is_builtin, is_active) VALUES
('minecraft', 'vanilla', 'Minecraft Vanilla', 'Default vanilla Minecraft server',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "VANILLA", "MEMORY": "2G", "MAX_PLAYERS": "20"}}'::jsonb,
 'public', true, true),
('minecraft', 'paper', 'Minecraft Paper', 'Paper server with optimized performance',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "PAPER", "MEMORY": "2G", "MAX_PLAYERS": "50"}}'::jsonb,
 'public', true, true);
```

---

### `app/src/pages/templates/TemplateLibraryPage.jsx` — NEW (browse page)

**Analog:** `app/src/pages/servers/ServerManagerPage.jsx`

**Imports pattern** (lines 1-7):
```jsx
import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { templatesApi } from '../../lib/api'
import TemplateCard from '../../components/TemplateCard'
import { EscluseSpinner } from '../../components/SkeletonLoader'
```

**Page structure pattern** (lines 8-80 of ServerManagerPage.jsx):
```jsx
export default function TemplateLibraryPage() {
  const [templates, setTemplates] = useState([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [gameFilter, setGameFilter] = useState('all')
  
  useEffect(() => {
    loadTemplates()
  }, [])

  const loadTemplates = async () => {
    try {
      setLoading(true)
      const params = gameFilter !== 'all' ? { game_type: gameFilter } : {}
      const data = await templatesApi.list(params)
      setTemplates(data || [])
    } catch (err) {
      console.error('Failed to load templates:', err)
    } finally {
      setLoading(false)
    }
  }

  // Filter + render grid of TemplateCard components
  // "Featured" section at top for built-in templates
  // Categorized browsing by game_type
  // "Create Server" button per template → navigates to creation flow
}
```

---

### `app/src/pages/templates/TemplateCreatePage.jsx` — NEW (create/edit form)

**Analog:** `app/src/pages/ServerDetails.jsx` (Settings tab — lines 447-507)

**Settings tab form pattern** (lines 447-507 of ServerDetails.jsx):
```jsx
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { templatesApi } from '../../lib/api'
import { useUIStore } from '../../store/uiStore'

export default function TemplateCreatePage() {
  const { addToast } = useUIStore()
  const navigate = useNavigate()
  const [saving, setSaving] = useState(false)
  const [form, setForm] = useState({
    game_type: 'minecraft',
    category: 'vanilla',
    display_name: '',
    description: '',
    visibility: 'private',
    config: { docker_image: '', default_port: 25565, env: {} },
  })

  const handleSave = async (e) => {
    e.preventDefault()
    setSaving(true)
    try {
      await templatesApi.create(form)
      addToast({ type: 'success', message: 'Template created!' })
      navigate('/templates')
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="p-6 max-w-3xl mx-auto">
      <h2 className="text-2xl font-bold text-white mb-6">Create Template</h2>
      
      {/* Glass-panel sections for each form group — follow Phase 56/57 UI pattern */}
      <section className="glass-panel p-6 mt-6">
        <h3 className="text-lg font-bold mb-1">Basic Info</h3>
        {/* Game type select, category, display name, description inputs */}
      </section>

      <section className="glass-panel p-6 mt-6">
        <h3 className="text-lg font-bold mb-1">Configuration</h3>
        {/* Docker image, port, env vars JSON editor */}
      </section>

      <section className="glass-panel p-6 mt-6">
        <h3 className="text-lg font-bold mb-1">Visibility</h3>
        {/* Public/Private toggle */}
      </section>

      <button disabled={saving} onClick={handleSave}
        className="mt-5 w-full py-2.5 rounded-lg text-sm font-bold
                   bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                   hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                   disabled:opacity-50 transition-all">
        {saving ? 'Saving...' : 'Create Template'}
      </button>
    </div>
  )
}
```

---

### `app/src/pages/templates/ModBrowserPage.jsx` — NEW (mod browser)

**Analog:** `app/src/pages/servers/ServerManagerPage.jsx` (search/filter pattern)

**Mod browser pattern** — Debounced search from usePlugins.js (lines 7-44) + list from ServerManagerPage.jsx:
```jsx
import { useState, useEffect, useRef } from 'react'
import { modsApi } from '../api/templatesApi'
import { usePluginSearch } from '../hooks/usePlugins'
import ModSearchResult from '../../components/ModSearchResult'

export default function ModBrowserPage() {
  const { results, total, loading, error, search } = usePluginSearch()
  const [query, setQuery] = useState('')
  const [version, setVersion] = useState('')
  const [loader, setLoader] = useState('')
  const debounceRef = useRef(null)

  // Debounced search — follow existing usePluginSearch debounce pattern
  const handleSearch = (e) => {
    const val = e.target.value
    setQuery(val)
    clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => {
      search(val, version || undefined, loader || undefined, 'mod', 0, 'downloads', true)
    }, 300)
  }

  return (
    <div className="p-6">
      <h2 className="text-2xl font-bold text-white mb-6">Mod Browser</h2>
      {/* Search bar + filter controls + results grid */}
      {/* Each result → ModSearchResult component with add-to-collection button */}
    </div>
  )
}
```

---

### `app/src/hooks/useTemplateLibrary.js` — NEW

**Analog:** `app/src/hooks/useServers.js`

**Hook pattern** (lines 1-22 of useServers.js):
```javascript
import { useState, useEffect, useCallback } from 'react'
import { templatesApi } from '../lib/api'

export function useTemplateLibrary() {
  const [templates, setTemplates] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  const refetch = useCallback(async (params = {}) => {
    try {
      setLoading(true)
      const data = await templatesApi.list(params)
      setTemplates(Array.isArray(data) ? data : [])
      setError(null)
    } catch (err) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { refetch() }, [refetch])

  return { templates, loading, error, refetch }
}

export async function createTemplate(data) {
  return templatesApi.create(data)
}

export async function updateTemplate(id, data) {
  return templatesApi.update(id, data)
}

export async function deleteTemplate(id) {
  return templatesApi.delete(id)
}

export async function createServerFromTemplate(templateId, data) {
  return templatesApi.createServer(templateId, data)
}
```

---

### `app/src/hooks/useModBrowser.js` — NEW

**Analog:** `app/src/hooks/usePlugins.js` (usePluginSearch — lines 7-44)

```javascript
import { useState, useCallback } from 'react'
import { fetchApi } from '../api/client'

export function useModBrowser() {
  const [results, setResults] = useState([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)

  const search = useCallback(async (query, version, loader, projectType, offset = 0) => {
    if (!query || query.trim().length < 2) {
      setResults([])
      setTotal(0)
      return
    }
    try {
      setLoading(true)
      setError(null)
      const params = new URLSearchParams({ q: query })
      if (version) params.set('version', version)
      if (loader) params.set('loader', loader)
      if (projectType) params.set('project_type', projectType)
      params.set('offset', offset.toString())
      params.set('sort', 'downloads')
      const data = await fetchApi(`/plugins/search?${params}`)
      // Reuse existing /api/v1/plugins/search endpoint
      setResults(data.plugins || [])
      setTotal(data.total || 0)
    } catch (e) {
      setError(e.message)
      setResults([])
    } finally {
      setLoading(false)
    }
  }, [])

  return { results, total, loading, error, search }
}
```

---

### `app/src/api/templatesApi.js` — NEW

**Analog:** `app/src/lib/api.js` (templatesApi block — lines 163-165)

```javascript
import { api } from '../lib/api'

export const templatesApi = {
  list: (params) => api.get('/templates', { params }),
  get: (id) => api.get(`/templates/${id}`),
  create: (data) => api.post('/templates', data),
  update: (id, data) => api.put(`/templates/${id}`, data),
  delete: (id) => api.delete(`/templates/${id}`),
  createServer: (id, data) => api.post(`/templates/${id}/create-server`, data),
}

export const modsApi = {
  search: (params) => api.get('/plugins/search', { params }),
  getVersions: (projectId, params) => api.get(`/plugins/${projectId}/versions`, { params }),
}
```

---

### `app/src/components/TemplateCard.jsx` — NEW

**Analog:** Card pattern from ServerManagerPage.jsx (store-based card rendering)

```jsx
import { Link } from 'react-router-dom'

export default function TemplateCard({ template, onDelete, onClone }) {
  return (
    <div className="glass-panel p-4 rounded-xl border border-[var(--color-cosmic-border)]
                    hover:border-[var(--color-cosmic-cyan)]/50 transition-all">
      <div className="flex items-start justify-between mb-3">
        <div>
          <h3 className="text-white font-bold">{template.display_name}</h3>
          <p className="text-xs text-[var(--color-text-muted)]">
            {template.game_type} / {template.category}
          </p>
        </div>
        {template.is_builtin && (
          <span className="px-2 py-0.5 text-[10px] rounded-full bg-blue-500/20 text-blue-400">
            Official
          </span>
        )}
      </div>
      
      {template.description && (
        <p className="text-xs text-[var(--color-text-muted)] mb-3 line-clamp-2">
          {template.description}
        </p>
      )}

      <div className="flex items-center justify-between mt-4">
        <div className="flex gap-2">
          <Link to={`/templates/${template.id}`}
                className="px-3 py-1.5 text-xs rounded-lg bg-[var(--color-cosmic-cyan)]/10
                           text-[var(--color-cosmic-cyan)] hover:bg-[var(--color-cosmic-cyan)]/20">
            Create Server
          </Link>
        </div>
        {!template.is_builtin && (
          <button onClick={() => onDelete(template.id)}
                  className="text-xs text-red-400 hover:text-red-300">
            Delete
          </button>
        )}
      </div>
    </div>
  )
}
```

---

### `app/src/components/ModSearchResult.jsx` — NEW

**Analog:** Server row in ServerManagerPage.jsx

```jsx
export default function ModSearchResult({ mod, onAdd, onViewVersions }) {
  return (
    <div className="glass-panel p-4 rounded-xl border border-[var(--color-cosmic-border)] flex gap-4">
      {mod.icon_url && (
        <img src={mod.icon_url} alt={mod.title} className="w-12 h-12 rounded-lg object-cover" />
      )}
      <div className="flex-1 min-w-0">
        <h4 className="text-sm font-bold text-white truncate">{mod.title}</h4>
        <p className="text-xs text-[var(--color-text-muted)] line-clamp-2">{mod.description}</p>
        <div className="flex items-center gap-3 mt-2">
          <span className="text-[10px] text-[var(--color-text-muted)]">
            ⬇ {mod.downloads?.toLocaleString() || 0} downloads
          </span>
        </div>
      </div>
      <div className="flex flex-col gap-2">
        <button onClick={() => onViewVersions?.(mod)}
                className="px-3 py-1 text-xs rounded bg-[var(--color-cosmic-cyan)]/10
                           text-[var(--color-cosmic-cyan)] hover:bg-[var(--color-cosmic-cyan)]/20">
          Versions
        </button>
        <button onClick={() => onAdd?.(mod)}
                className="px-3 py-1 text-xs rounded bg-emerald-500/10 text-emerald-400
                           hover:bg-emerald-500/20">
          Add
        </button>
      </div>
    </div>
  )
}
```

---

### `app/src/app/App.jsx` — EXTEND (add routes)

**Route addition pattern** (lines 106-115):
```jsx
import TemplateLibraryPage from '../pages/templates/TemplateLibraryPage'
import TemplateCreatePage from '../pages/templates/TemplateCreatePage'
import ModBrowserPage from '../pages/templates/ModBrowserPage'

// Inside <Routes> in main content area (after line 115):
<Route path="/templates" element={<TemplateLibraryPage />} />
<Route path="/templates/create" element={<TemplateCreatePage />} />
<Route path="/templates/:id" element={<TemplateDetailPage />} />
<Route path="/mods" element={<ModBrowserPage />} />

// Sidebar nav links (after line 77):
<a href="/templates" className="block py-2 text-gray-400 hover:text-white">Templates</a>
<a href="/mods" className="block py-2 text-gray-400 hover:text-white">Mod Browser</a>
```

---

### `app/src/lib/api.js` — EXTEND (add templatesApi methods)

**Extension pattern** — Add to existing templatesApi block (lines 163-165):
```javascript
export const templatesApi = {
  list: (params) => api.get('/templates', { params }),
  get: (id) => api.get(`/templates/${id}`),
  create: (data) => api.post('/templates', data),
  update: (id, data) => api.put(`/templates/${id}`, data),
  delete: (id) => api.delete(`/templates/${id}`),
  createServer: (id, data) => api.post(`/templates/${id}/create-server`, data),
}
```

---

## Shared Patterns

### Authentication
**Source:** `api/src/presentation/handlers/server_handlers.rs` (lines 397-400)
**Apply to:** All template CRUD handlers
```rust
pub async fn create_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,  // ← authentication extractor
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
```

### Error Handling
**Source:** `api/src/shared/errors/app_error.rs` (lines 9-29)
**Apply to:** All handlers, use cases
```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal Server Error")]
    InternalError(#[from] anyhow::Error),
    #[error("Not Found")]
    NotFound,
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Forbidden")]
    Forbidden,
}
```

### Response Formatting
**Source:** `api/src/presentation/responses/api_response.rs` (lines 196-212)
**Apply to:** All template handlers
```rust
Ok(Json(ApiResponse::success(template)))
// Or for errors:
.map_err(|e| AppError::InternalError(anyhow::anyhow!(e)))?;
```

### Fallback-to-DB Pattern
**Source:** `api/src/domain/server/modpack_template/repository.rs` (lines 46-50)
**Apply to:** All template repository queries
```rust
if result.is_empty() {
    tracing::info!("No templates found in database, using fallback templates");
    return Ok(Template::fallback());
}
```

### Settings Tab UI (Phase 56/57)
**Source:** `app/src/pages/ServerDetails.jsx` (lines 447-507)
**Apply to:** TemplateCreatePage.jsx, Modrinth API key settings
```jsx
<section className="glass-panel p-6 mt-6">
    <h3 className="text-lg font-bold mb-1">Section Title</h3>
    <p className="text-xs text-[var(--color-text-muted)] mb-5">Description</p>
    {toast && (...)}
    {/* Toggle */}
    <div className="flex items-center gap-3 p-4 rounded-xl border cursor-pointer" onClick={...}>
        <div className={`w-12 h-6 rounded-full transition-colors ${toggle ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-[var(--color-cosmic-border)]'}`}>
            <div className={`w-5 h-5 rounded-full bg-white transition-transform ${toggle ? 'translate-x-6' : 'translate-x-0.5'}`} />
        </div>
        <div className="flex-1">
            <p className="text-sm font-bold">Label</p>
            <p className="text-xs text-[var(--color-text-muted)]">Sub-label</p>
        </div>
    </div>
    {toggle && (<div className="mt-4"><input type="text/number" .../></div>)}
    <button disabled={saving} onClick={handleSave} ...>{saving ? 'Saving...' : 'Save Changes'}</button>
</section>
```

### Agent Task Protocol (for future plugin download)
**Source:** `agent/agent-core/crates/agent-proto/src/task.rs` (lines 7-18) and `agent/agent-core/crates/agent-proto/src/messages.rs` (lines 9-30)
**Apply to:** Future agent task dispatch
```rust
// Task type supports any task_type string + JSON payload:
let task = Task::new(
    "plugin_download".to_string(),
    serde_json::json!({
        "server_id": server_id,
        "plugins": [{ "url": "...", "filename": "plugin.jar", "install_dir": "plugins/" }]
    }),
);
```

### API Client Pattern (lib/api.js style)
**Source:** `app/src/lib/api.js` — `api` class + domain-specific API objects
**Apply to:** `app/src/api/templatesApi.js` and existing templatesApi block
```javascript
class ApiClient {
  async request(endpoint, options = {}) {
    const token = this.getToken()
    const headers = { 'Content-Type': 'application/json', ...options.headers }
    if (token) headers['Authorization'] = `Bearer ${token}`
    const response = await fetch(`${this.baseUrl}${endpoint}`, { ...options, headers, credentials: 'include' })
    if (response.status === 401) { /* refresh logic */ }
    const data = await response.json()
    if (!response.ok) throw new Error(data?.error?.message || `Request failed`)
    return data?.data ?? data
  }
}
```

---

## No Analog Found

All 21 files have close analogs in the existing codebase. No orphan patterns.

| File | Role | Data Flow | Reason | Solution |
|------|------|-----------|--------|----------|
| — | — | — | — | — |

---

## Metadata

**Analog search scope:** `api/src/`, `app/src/`, `agent/agent-core/crates/`
**Files scanned:** 34 (all entity/repository/handler/use_case/route files + frontend hooks/pages/api/components)
**Pattern extraction date:** 2026-05-31
