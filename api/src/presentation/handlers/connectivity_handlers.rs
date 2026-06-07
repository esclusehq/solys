//! Phase 67: Per-server connectivity REST endpoints.
//!
//! Three endpoints:
//! - `GET  /api/v1/servers/:server_id/connectivity`       — current state
//! - `POST /api/v1/servers/:server_id/connectivity/probe` — manual "Reachable" button (30s Redis cooldown)
//! - `GET  /api/v1/servers/:server_id/connectivity/audit` — paginated audit log
//!
//! All three enforce a per-tenant ownership check (server.user_id == auth_user.tenant_id).

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::auth::middleware::AuthUser;
use crate::presentation::responses::api_response::ApiResponse;
use crate::presentation::routes::api_routes::ApiState;
use crate::shared::errors::app_error::AppError;
use crate::application::services::connectivity_service::ConnectivityState;

async fn ensure_ownership(
    state: &ApiState,
    auth_user: &AuthUser,
    server_id: Uuid,
) -> Result<(), AppError> {
    let server = state
        .server_repository
        .find_by_id(&server_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?
        .ok_or(AppError::NotFound)?;
    if server.user_id != auth_user.tenant_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub async fn get_status(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<Json<ApiResponse<ConnectivityState>>, AppError> {
    ensure_ownership(&state, &auth_user, server_id).await?;
    let conn = state
        .connectivity_service
        .read_state_for_handler(server_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;
    Ok(Json(ApiResponse::success(conn)))
}

pub async fn trigger_probe(
    Path(server_id): Path<Uuid>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<Json<ApiResponse<ConnectivityState>>, AppError> {
    ensure_ownership(&state, &auth_user, server_id).await?;

    // 30s Redis cooldown per server
    if let Some(redis_pool) = &state.redis_pool {
        if let Ok(mut conn) = redis_pool.get().await {
            let key = format!("connectivity:probe_cooldown:{}", server_id);
            // SET key value NX EX 30 — if NX fails (key already exists), cooldown is active
            let res: redis::RedisResult<Option<String>> = redis::cmd("SET")
                .arg(&key)
                .arg("1")
                .arg("NX")
                .arg("EX")
                .arg(30u64)
                .query_async(&mut conn)
                .await;
            if let Ok(Some(_)) = res {
                // Cooldown NOT set (we set the key), proceed
            } else {
                // Either cooldown already active or Redis error — treat as cooldown
                return Err(AppError::BadRequest(
                    "Probe cooldown active (30s per server)".into(),
                ));
            }
        }
    }

    let conn = state
        .connectivity_service
        .probe_server(server_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;
    Ok(Json(ApiResponse::success(conn)))
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_audit_log(
    Path(server_id): Path<Uuid>,
    Query(q): Query<AuditQuery>,
    State(state): State<ApiState>,
    auth_user: AuthUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    ensure_ownership(&state, &auth_user, server_id).await?;
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);
    let logs = state
        .connectivity_audit_log_repository
        .list_by_server(server_id, limit, offset)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;
    let total = state
        .connectivity_audit_log_repository
        .count_by_server(server_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!(e.to_string())))?;
    Ok(Json(ApiResponse::success(serde_json::json!({
        "items": logs, "total": total, "limit": limit, "offset": offset,
    }))))
}
