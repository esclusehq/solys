use crate::presentation::routes::api_routes::ApiState;
use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
};
use serde_json::json;
use crate::domain::auth::middleware::AuthUser;
use crate::domain::entities::settings::{RestartDefaults, S3Config};
use crate::domain::entities::cloudflare_settings::CloudflareConfig;
use crate::domain::entities::s3_profile::{S3Profile, S3ProfileInput};
use crate::domain::repositories::s3_profile_repository::S3ProfileRepository;
use crate::infrastructure::repositories::postgres_s3_profile_repository::PostgresS3ProfileRepository;
use crate::presentation::ws::node_protocol::NodeMessage;
use reqwest::Client;

/// GET /settings/s3
pub async fn get_s3_config(
    State(container): State<ApiState>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match container.settings_repository.get_s3_config().await {
        Ok(config) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": {
                "endpoint": config.endpoint,
                "region": config.region,
                "bucket": config.bucket,
                "access_key": config.access_key,
                // Mask the secret key for GET responses
                "secret_key_set": !config.secret_key.is_empty(),
            }
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// PUT /settings/s3
pub async fn save_s3_config(
    State(container): State<ApiState>,
    Json(payload): Json<S3Config>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    // If secret_key is empty in the payload, keep the existing one
    let config = if payload.secret_key.is_empty() {
        let existing = container.settings_repository.get_s3_config().await.unwrap_or_default();
        S3Config {
            endpoint: payload.endpoint,
            region: payload.region,
            bucket: payload.bucket,
            access_key: payload.access_key,
            secret_key: existing.secret_key,
        }
    } else {
        payload
    };

    match container.settings_repository.save_s3_config(&config).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": null
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// GET /settings/cloudflare
pub async fn get_cloudflare_config(
    State(container): State<ApiState>,
    user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    match container.settings_repository.get_cloudflare_config().await {
        Ok(config) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": {
                "api_token": config.masked_api_token(),
                "zone_id": config.zone_id,
                "zone_name": config.zone_name,
                "wildcard_domain": config.wildcard_domain,
                "auto_refresh": config.auto_refresh,
                "refresh_interval_secs": config.refresh_interval_secs,
                "is_configured": config.is_configured(),
            }
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// PUT /settings/cloudflare
pub async fn save_cloudflare_config(
    State(container): State<ApiState>,
    user: AuthUser,
    Json(payload): Json<CloudflareConfig>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    let config = if payload.api_token.is_empty() {
        let existing = container.settings_repository.get_cloudflare_config().await.unwrap_or_default();
        CloudflareConfig {
            api_token: existing.api_token,
            zone_id: payload.zone_id,
            zone_name: payload.zone_name,
            wildcard_domain: payload.wildcard_domain,
            auto_refresh: payload.auto_refresh,
            refresh_interval_secs: payload.refresh_interval_secs,
        }
    } else {
        payload
    };

    match container.settings_repository.save_cloudflare_config(&config).await {
        Ok(_) => {
            let msg = NodeMessage::DnsConfig {
                api_token: config.api_token.clone(),
                zone_id: config.zone_id.clone(),
                zone_name: config.zone_name.clone(),
                wildcard_domain: config.wildcard_domain.clone(),
                auto_refresh: config.auto_refresh,
                refresh_interval_secs: config.refresh_interval_secs,
                public_ip: None,
                subdomain: None,
            };
            container.node_connection_manager.broadcast_msg(&msg).await;
            Ok((StatusCode::OK, Json(json!({
                "success": true,
                "data": null
            }))))
        },
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// POST /settings/cloudflare/test
pub async fn test_cloudflare_config(
    State(container): State<ApiState>,
    user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    let config = match container.settings_repository.get_cloudflare_config().await {
        Ok(c) => c,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    };
    if !config.is_configured() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({
            "success": false,
            "error": { "message": "Cloudflare not configured" }
        }))));
    }
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        }))))?;
    let resp = client
        .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
        .header("Authorization", format!("Bearer {}", config.api_token))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        }))))?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        }))))?;
    if status.is_success() && body.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
        Ok((StatusCode::OK, Json(json!({
            "success": true,
            "message": "Cloudflare API token is valid!"
        }))))
    } else {
        Ok((StatusCode::OK, Json(json!({
            "success": false,
            "error": { "message": "Invalid API token" }
        }))))
    }
}

// -- Restart Defaults (Admin) --

/// GET /settings/restart-defaults
pub async fn get_restart_defaults(
    State(container): State<ApiState>,
    user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    match container.settings_repository.get_restart_defaults().await {
        Ok(config) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": {
                "max_restart_attempts": config.max_restart_attempts,
                "restart_cooldown_seconds": config.restart_cooldown_seconds,
            }
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// PUT /settings/restart-defaults
pub async fn save_restart_defaults(
    State(container): State<ApiState>,
    user: AuthUser,
    Json(payload): Json<RestartDefaults>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    match container.settings_repository.save_restart_defaults(&payload).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": null
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

// -- S3 Profile CRUD --

/// GET /settings/s3/profiles
pub async fn list_s3_profiles(
    State(state): State<ApiState>,
    _user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresS3ProfileRepository::new(state.pool.clone());
    match repo.list_all().await {
        Ok(profiles) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": profiles,
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// GET /settings/s3/profiles/:id
pub async fn get_s3_profile(
    Path(id): Path<uuid::Uuid>,
    State(state): State<ApiState>,
    _user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresS3ProfileRepository::new(state.pool.clone());
    match repo.find_by_id(&id).await {
        Ok(Some(profile)) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": profile,
        })))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({
            "success": false,
            "error": { "message": "S3 profile not found" }
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// POST /settings/s3/profiles
pub async fn create_s3_profile(
    State(state): State<ApiState>,
    _user: AuthUser,
    Json(input): Json<S3ProfileInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresS3ProfileRepository::new(state.pool.clone());
    match repo.create(&input).await {
        Ok(profile) => Ok((StatusCode::CREATED, Json(json!({
            "success": true,
            "data": profile,
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// PUT /settings/s3/profiles/:id
pub async fn update_s3_profile(
    Path(id): Path<uuid::Uuid>,
    State(state): State<ApiState>,
    _user: AuthUser,
    Json(input): Json<S3ProfileInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresS3ProfileRepository::new(state.pool.clone());
    match repo.update(&id, &input).await {
        Ok(profile) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": profile,
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

/// DELETE /settings/s3/profiles/:id
pub async fn delete_s3_profile(
    Path(id): Path<uuid::Uuid>,
    State(state): State<ApiState>,
    _user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresS3ProfileRepository::new(state.pool.clone());
    match repo.delete(&id).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": null,
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

// -- Modrinth API Key (Admin) --

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
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    let api_key = payload.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
    match container.settings_repository.save_modrinth_api_key(api_key).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": null
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

// -- CurseForge API Key (Admin) --

/// GET /settings/curseforge-api-key
pub async fn get_curseforge_api_key(
    State(container): State<ApiState>,
    user: AuthUser,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    match container.settings_repository.get_curseforge_api_key().await {
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

/// PUT /settings/curseforge-api-key
pub async fn save_curseforge_api_key(
    State(container): State<ApiState>,
    user: AuthUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "success": false,
            "error": { "message": "Admin access required" }
        }))));
    }
    let api_key = payload.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
    match container.settings_repository.save_curseforge_api_key(api_key).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({
            "success": true,
            "data": null
        })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": { "message": e.to_string() }
        })))),
    }
}

