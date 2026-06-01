use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use serde::Deserialize;

use crate::domain::auth::middleware::VerifiedUser;
use crate::domain::server::template::{SqlxTemplateRepository, TemplateRepository};
use crate::application::dto::template_dtos::*;
use crate::presentation::routes::api_routes::ApiState;
use crate::presentation::responses::api_response::ApiResponse;
use crate::shared::errors::app_error::AppError;

use crate::application::use_cases::template_use_cases::*;
use crate::application::use_cases::create_server_use_case::CreateServerUseCase;
use crate::domain::repositories::server_repository::ServerRepository;
use crate::infrastructure::repositories::postgres_server_repository::PostgresServerRepository;

// ── Query parameter for listing templates ────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    #[serde(default)]
    pub game_type: Option<String>,
}

// ── Template CRUD Handlers ───────────────────────────────────────────

/// GET /api/v1/templates
pub async fn list_templates(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let repository = Arc::new(SqlxTemplateRepository::new(state.pool.clone())) as Arc<dyn TemplateRepository>;
    let use_case = ListTemplatesUseCase::new(repository);
    let templates = use_case
        .execute(auth_user.user_id, query.game_type)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e)))?;
    let response: Vec<TemplateResponse> = templates.into_iter().map(Into::into).collect();
    Ok(Json(ApiResponse::success(response)))
}

/// GET /api/v1/templates/:id
pub async fn get_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let repository = Arc::new(SqlxTemplateRepository::new(state.pool.clone())) as Arc<dyn TemplateRepository>;
    let use_case = GetTemplateUseCase::new(repository);
    let template = use_case
        .execute(id)
        .await
        .map_err(|_| AppError::NotFound)?;
    let response: TemplateResponse = template.into();
    Ok(Json(ApiResponse::success(response)))
}

/// POST /api/v1/templates
pub async fn create_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repository = Arc::new(SqlxTemplateRepository::new(state.pool.clone())) as Arc<dyn TemplateRepository>;
    let use_case = CreateTemplateUseCase::new(repository);
    let template = use_case
        .execute(auth_user.user_id, payload)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e)))?;
    let response: TemplateResponse = template.into();
    Ok(Json(ApiResponse::success(response)))
}

/// PUT /api/v1/templates/:id
pub async fn update_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repository = Arc::new(SqlxTemplateRepository::new(state.pool.clone())) as Arc<dyn TemplateRepository>;
    let use_case = UpdateTemplateUseCase::new(repository);
    let template = use_case
        .execute(auth_user.user_id, id, payload, auth_user.is_admin())
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg == "Forbidden" {
                AppError::Forbidden
            } else if msg.starts_with("Cannot update built-in template") {
                AppError::BadRequest(msg)
            } else if msg.starts_with("Template not found") {
                AppError::NotFound
            } else {
                AppError::InternalError(anyhow::anyhow!(e))
            }
        })?;
    let response: TemplateResponse = template.into();
    Ok(Json(ApiResponse::success(response)))
}

/// DELETE /api/v1/templates/:id
pub async fn delete_template(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let repository = Arc::new(SqlxTemplateRepository::new(state.pool.clone())) as Arc<dyn TemplateRepository>;
    let use_case = DeleteTemplateUseCase::new(repository);
    use_case
        .execute(auth_user.user_id, id, auth_user.is_admin())
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg == "Forbidden" {
                AppError::Forbidden
            } else if msg.starts_with("Cannot delete built-in template") {
                AppError::BadRequest(msg)
            } else if msg.starts_with("Template not found") {
                AppError::NotFound
            } else {
                AppError::InternalError(anyhow::anyhow!(e))
            }
        })?;
    Ok(Json(ApiResponse::success(serde_json::json!({"status": "deleted"}))))
}

// ── Apply Template to Server Handler ─────────────────────────────────

/// POST /api/v1/templates/:id/create-server
pub async fn apply_template_to_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<CreateServerFromTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Fetch template
    let template_repo = Arc::new(SqlxTemplateRepository::new(state.pool.clone())) as Arc<dyn TemplateRepository>;
    let template = template_repo
        .get_template_by_id(template_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    // 2. Verify non-owners can only use public templates
    if template.user_id != Some(auth_user.user_id) && template.visibility != "public" && !template.is_builtin {
        return Err(AppError::Forbidden);
    }

    // 3. Deep-clone template config and apply overrides
    let mut server_config = template.config.clone();
    if let Some(overrides) = &payload.config_overrides {
        if let Some(obj) = overrides.as_object() {
            for (k, v) in obj {
                server_config[k] = v.clone();
            }
        }
    }

    // 4. Build CreateServerRequest from template + payload
    let create_req = crate::application::dto::server_dtos::CreateServerRequest {
        user_id: auth_user.user_id,
        name: payload.name,
        game: template.game_type.clone(),
        host: "0.0.0.0".to_string(),
        port: server_config.get("default_port")
            .and_then(|v| v.as_i64())
            .unwrap_or(25565) as i32,
        username: auth_user.user_id.to_string(),
        password_auth: "".to_string(),
        executor_type: "agent".to_string(),
        environment: None,
        server_path: None,
        start_command: None,
        stop_command: None,
        container_name: None,
        public_host: None,
        mc_version: payload.minecraft_version.clone().or_else(|| {
            server_config.get("env")
                .and_then(|env| env.get("TYPE"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        }),
        mc_loader: None,
        auto_pause: None,
        ram_allocation: payload.ram_mb.map(|r| format!("{}M", r)),
        discord_webhook_url: None,
        auto_restart: None,
        enable_tailscale: None,
        tailscale_auth_key: None,
        custom_container_name: None,
        ip_binding: None,
        template: Some(template.category),
        network_name: None,
        auto_wake: None,
        sleep_timeout_minutes: None,
        max_restart_attempts: None,
        restart_cooldown_seconds: None,
        node_id: payload.node_id,
        game_type: payload.game_type.or(Some(template.game_type)),
        minecraft_version: payload.minecraft_version,
        ram_mb: payload.ram_mb,
        max_players: payload.max_players,
        template_id: Some(template_id),
    };

    // 5. Create server via use case
    let server_repo = Arc::new(PostgresServerRepository::new(state.pool.clone())) as Arc<dyn ServerRepository>;
    let use_case = CreateServerUseCase::new(server_repo);
    let server = use_case
        .execute(create_req)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e)))?;

    Ok(Json(ApiResponse::success(server)))
}
