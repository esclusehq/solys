use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::domain::auth::middleware::AuthUser;
use crate::domain::entities::backup_config::BackupConfig;
use crate::presentation::routes::api_routes::ApiState;
use crate::application::services::backup_config_service::BackupConfigService;

/// GET /api/v1/servers/:server_id/backup-config
pub async fn get_backup_config(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let service = BackupConfigService::new(
        state.backup_config_repository.clone(),
        state.server_repository.clone(),
        state.cron_task_repository.clone(),
    );

    match service.get_config(&server_id).await {
        Ok(Some(config)) => Ok(Json(json!({
            "success": true,
            "data": config,
        }))),
        Ok(None) => {
            // Return default config for new servers
            Ok(Json(json!({
                "success": true,
                "data": {
                    "auto_backup_enabled": false,
                    "schedule_cron": "",
                    "backup_provider": "local",
                    "max_retained_backups": 10,
                    "retention_rules": {"daily": 7, "weekly": 4, "monthly": 3},
                    "retention_mode": "hybrid",
                    "s3_profile_id": null,
                    "cron_task_id": null,
                }
            })))
        }
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": { "message": e.to_string() } })),
        )),
    }
}

/// PUT /api/v1/servers/:server_id/backup-config
pub async fn update_backup_config(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
    Json(config): Json<BackupConfig>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let service = BackupConfigService::new(
        state.backup_config_repository.clone(),
        state.server_repository.clone(),
        state.cron_task_repository.clone(),
    );

    // Verify server ownership
    match state.server_repository.find_by_id(&server_id).await {
        Ok(Some(server)) => {
            if server.user_id != auth_user.tenant_id {
                return Err((
                    axum::http::StatusCode::FORBIDDEN,
                    Json(json!({ "success": false, "error": { "message": "Access denied" } })),
                ));
            }
        }
        Ok(None) => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "error": { "message": "Server not found" } })),
            ));
        }
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": { "message": e.to_string() } })),
            ));
        }
    }

    match service.save_config(&server_id, &config).await {
        Ok(_) => Ok(Json(json!({ "success": true, "data": null }))),
        Err(e) => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": { "message": e.to_string() } })),
        )),
    }
}
