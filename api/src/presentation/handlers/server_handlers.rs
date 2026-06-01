use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put, delete, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::json;

use crate::domain::auth::middleware::VerifiedUser;
use crate::shared::errors::app_error::AppError;
use crate::domain::audit::service::AuditService;
use crate::domain::server::model::{CreateServerRequest, Server, UpdateServerRequest};
use crate::domain::server::sqlx_repository::SqlxServerRepository;
use crate::domain::server::repository::ServerRepository;
use crate::domain::repositories::node_repository::NodeRepository;
use crate::infrastructure::repositories::postgres_node_repository::PostgresNodeRepository as SqlxNodeRepository;
use crate::domain::repositories::metrics_repository::MetricsRepository;
use crate::domain::webhook::service::emit_server_event;
use crate::infrastructure::solys_client::client::{SolysOperationResponse, SolysServerStatus, SolysServerStats, SolysFileListResponse};
use crate::presentation::routes::api_routes::ApiState;
use crate::presentation::responses::api_response::ApiResponse;
use crate::presentation::handlers::file_handlers;
use crate::presentation::handlers::terminal_handlers;
use crate::presentation::handlers::backup_handlers;
use crate::presentation::handlers::plugin_handlers;
use crate::presentation::handlers::profiling_handlers;
use crate::presentation::handlers::build_handlers;
use crate::presentation::handlers::deployment_handlers;
use crate::presentation::handlers::search_handlers;
use crate::infrastructure::repositories::crash_log_repository::PostgresCrashLogRepository;

/// Handler for getting server properties from server.properties file
pub async fn get_server_properties(
    Path(server_id): Path<Uuid>,
    State(container): State<ApiState>,
) -> Result<impl IntoResponse, AppError> {
    let _server = container.get_server_use_case.execute(server_id).await
        .map_err(|_| AppError::NotFound)?;

    let container_name = format!("mc-{}", server_id);

    // Execute cat command inside container to read server.properties
    let output = tokio::process::Command::new("docker")
        .args(["exec", &container_name, "cat", "/data/server.properties"])
        .output()
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to read properties: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::InternalError(anyhow::anyhow!("Failed to read properties: {}", stderr)));
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let properties = parse_server_properties(&content);

    Ok(ApiResponse::success(properties))
}

/// Handler for updating server properties in server.properties file
pub async fn update_server_properties(
    Path(server_id): Path<Uuid>,
    State(container): State<ApiState>,
    Json(properties): Json<ServerPropertiesUpdate>,
) -> Result<impl IntoResponse, AppError> {
    let server = container.get_server_use_case.execute(server_id).await
        .map_err(|_| AppError::NotFound)?;

    let container_name = format!("mc-{}", server_id);

    // Build the complete server.properties content
    let properties_content = build_server_properties(&properties);

    // Write to temp file then move to target location
    let temp_file = "/tmp/server.properties.new";
    let target_file = "/data/server.properties";

    // Write properties to temp file
    let write_cmd = format!("echo '{}' > {}", 
        properties_content.replace("'", "'\\''"),
        temp_file
    );
    
    let write_output = tokio::process::Command::new("docker")
        .args(["exec", &container_name, "sh", "-c", &write_cmd])
        .output()
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to write temp properties: {}", e)))?;

    if !write_output.status.success() {
        let stderr = String::from_utf8_lossy(&write_output.stderr);
        return Err(AppError::InternalError(anyhow::anyhow!("Failed to write temp file: {}", stderr)));
    }

    // Move temp file to target location
    let move_cmd = format!("mv {} {}", temp_file, target_file);
    let move_output = tokio::process::Command::new("docker")
        .args(["exec", &container_name, "sh", "-c", &move_cmd])
        .output()
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to move properties file: {}", e)))?;

    if !move_output.status.success() {
        let stderr = String::from_utf8_lossy(&move_output.stderr);
        return Err(AppError::InternalError(anyhow::anyhow!("Failed to update properties: {}", stderr)));
    }

    Ok(ApiResponse::success(json!({
        "message": "Properties updated successfully",
        "server_id": server_id,
    })))
}

/// Parse server.properties content into a JSON object
fn parse_server_properties(content: &str) -> serde_json::Value {
    let mut props = serde_json::Map::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            
            // Try to parse as number, otherwise keep as string
            let parsed_value = if let Ok(num) = value.parse::<i32>() {
                serde_json::Value::Number(num.into())
            } else if let Ok(num) = value.parse::<i64>() {
                serde_json::Value::Number(num.into())
            } else if value == "true" {
                serde_json::json!(true)
            } else if value == "false" {
                serde_json::json!(false)
            } else {
                serde_json::Value::String(value)
            };
            
            props.insert(key, parsed_value);
        }
    }
    
    serde_json::Value::Object(props)
}

/// Build server.properties content from update struct
fn build_server_properties(update: &ServerPropertiesUpdate) -> String {
    let mut lines = vec![];
    
    // Add the updatable properties
    if let Some(val) = &update.motd {
        lines.push(format!("motd={}", val));
    }
    if let Some(val) = &update.max_players {
        lines.push(format!("max-players={}", val));
    }
    if let Some(val) = &update.gamemode {
        lines.push(format!("gamemode={}", val));
    }
    if let Some(val) = &update.difficulty {
        lines.push(format!("difficulty={}", val));
    }
    if let Some(val) = &update.level_seed {
        lines.push(format!("level-seed={}", val));
    }
    if let Some(val) = &update.view_distance {
        lines.push(format!("view-distance={}", val));
    }
    if let Some(val) = &update.simulation_distance {
        lines.push(format!("simulation-distance={}", val));
    }
    if let Some(val) = &update.allow_nether {
        lines.push(format!("allow-nether={}", val));
    }
    if let Some(val) = &update.allow_flight {
        lines.push(format!("allow-flight={}", val));
    }
    if let Some(val) = &update.force_gamemode {
        lines.push(format!("force-gamemode={}", val));
    }
    if let Some(val) = &update.hardcore {
        lines.push(format!("hardcore={}", val));
    }
    if let Some(val) = &update.pvp {
        lines.push(format!("pvp={}", val));
    }
    if let Some(val) = &update.spawn_animals {
        lines.push(format!("spawn-animals={}", val));
    }
    if let Some(val) = &update.spawn_monsters {
        lines.push(format!("spawn-monsters={}", val));
    }
    if let Some(val) = &update.spawn_npcs {
        lines.push(format!("spawn-npcs={}", val));
    }
    
    lines.join("\n")
}

#[derive(Deserialize)]
#[derive(Serialize)]
pub struct ServerPropertiesUpdate {
    #[serde(default)]
    pub motd: Option<String>,
    #[serde(default)]
    pub max_players: Option<i32>,
    #[serde(default)]
    pub gamemode: Option<String>,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub level_seed: Option<String>,
    #[serde(default)]
    pub view_distance: Option<i32>,
    #[serde(default)]
    pub simulation_distance: Option<i32>,
    #[serde(default)]
    pub allow_nether: Option<bool>,
    #[serde(default)]
    pub allow_flight: Option<bool>,
    #[serde(default)]
    pub force_gamemode: Option<bool>,
    #[serde(default)]
    pub hardcore: Option<bool>,
    #[serde(default)]
    pub pvp: Option<bool>,
    #[serde(default)]
    pub spawn_animals: Option<bool>,
    #[serde(default)]
    pub spawn_monsters: Option<bool>,
    #[serde(default)]
    pub spawn_npcs: Option<bool>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct FileReadRequest {
    path: String,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    max_size: Option<usize>,
}

#[derive(Deserialize)]
struct FileWriteRequest {
    path: String,
    content: String,
    #[serde(default = "default_true")]
    overwrite: bool,
}

#[derive(Deserialize)]
struct FileDeleteRequest {
    path: String,
    #[serde(default)]
    recursive: bool,
}

#[derive(Deserialize)]
struct FileMkdirRequest {
    path: String,
}

#[derive(Deserialize)]
struct FileRenameRequest {
    old_path: String,
    new_path: String,
}

#[derive(Deserialize)]
struct FileCopyRequest {
    source_path: String,
    dest_path: String,
}

#[derive(Deserialize)]
struct UpdateImageRequest {
    image: String,
}

#[derive(Deserialize)]
struct CommandRequest {
    command: String,
}

fn default_true() -> bool { true }

pub struct ServerHandlers;

#[derive(Deserialize)]
#[allow(dead_code)]
struct FileListRequest {
    path: String,
    #[serde(default)]
    page: Option<usize>,
    #[serde(default)]
    per_page: Option<usize>,
}

impl ServerHandlers {
    pub fn router(state: ApiState) -> Router<ApiState> {
        Router::new()
            .route("/", get(list_servers).post(create_server))
            .route("/:id", get(get_server).put(update_server).delete(delete_server))
            .route("/:id/start", post(start_server))
            .route("/:id/stop", post(stop_server))
            .route("/:id/restart", post(restart_server))
            .route("/:id/sleep", post(sleep_server))
            .route("/:id/wake", post(wake_server))
            .route("/:id/kill", post(kill_server))
            .route("/:id/status", get(get_status))
            .route("/:id/stats", get(get_stats))
            .route("/:id/logs/:lines", get(get_logs))
            .route("/:id/logs/stream", get(stream_logs))
            .route("/:id/command", post(terminal_handlers::exec_terminal))
            .route("/:id/rcon", post(terminal_handlers::exec_rcon))
            .route("/:id/health", get(get_health))
            // File management - use file_handlers for all operations
            .route("/:id/files/list", post(file_handlers::list_files))
            .route("/:id/files", get(file_handlers::list_files).delete(file_handlers::delete_path))
            .route("/:id/files/download", get(file_handlers::download_file))
            .route("/:id/files/read", post(file_handlers::read_file))
            .route("/:id/files/write", put(file_handlers::write_file))
            .route("/:id/files/upload", post(file_handlers::upload_file))
            .route("/:id/files/upload/chunked", post(file_handlers::upload_chunk))
            .route("/:id/files/upload/status/:filename", get(file_handlers::get_upload_status))
            .route("/:id/files/delete", post(file_handlers::delete_path))
            .route("/:id/files/mkdir", post(file_handlers::mkdir))
            .route("/:id/files/rename", post(file_handlers::rename_path))
            .route("/:id/files/copy", post(file_handlers::copy_path))
            .route("/:id/files/compress", post(file_handlers::compress_path))
            .route("/:id/files/extract", post(file_handlers::extract_path))
            .route("/:id/files/search", get(search_handlers::search_files))
            // Backups
            .route("/:id/backups", get(backup_handlers::list_backups).post(backup_handlers::trigger_backup))
            .route("/:id/backups/:backup_id", delete(backup_handlers::delete_backup))
            .route("/:id/backups/:backup_id/restore", post(backup_handlers::restore_backup))
            // Plugins
            .route("/:id/plugins", get(plugin_handlers::list_installed_plugins).delete(plugin_handlers::uninstall_plugin))
            .route("/:id/plugins/install", post(plugin_handlers::install_plugin))
            .route("/:id/plugins/toggle", post(plugin_handlers::toggle_plugin))
            // Profiling
            .route("/:id/profiler/status", get(profiling_handlers::get_profiler_status))
            .route("/:id/profiler/jvm", get(profiling_handlers::get_jvm_metrics))
            .route("/:id/profiler/memory", get(profiling_handlers::get_memory_pools))
            .route("/:id/profiler/gc", get(profiling_handlers::get_gc_stats))
            .route("/:id/profiler/threads", get(profiling_handlers::get_thread_dump))
            .route("/:id/profiler/full", get(profiling_handlers::get_full_profile))
            .route("/:id/profiler/debug-logs", post(profiling_handlers::get_debug_logs))
            .route("/:id/profiler/heap-dump", post(profiling_handlers::generate_heap_dump))
            .route("/:id/profiler/heap-dump/download", get(profiling_handlers::download_heap_dump))
            // Build & Hot-Swap
            .route("/:id/build/detect", get(build_handlers::detect_build_system))
            .route("/:id/build", post(build_handlers::execute_build))
            .route("/:id/build/ws", get(build_handlers::ws_build))
            .route("/:id/build/status", get(build_handlers::get_build_status))
            .route("/:id/hot-reload", post(build_handlers::hot_reload))
            // Deploy
            .route("/:id/deploy", post(deployment_handlers::deploy_artifact))
            .route("/:id/deploy/history", get(deployment_handlers::get_deployment_history))
            .route("/:id/deploy/artifacts", get(deployment_handlers::get_available_artifacts))
            .route("/:id/deploy/modrinth", post(deployment_handlers::upload_to_modrinth))
            .route("/:id/deploy/rollback", post(deployment_handlers::rollback_deployment))
            // Image
            .route("/:id/image", post(update_image))
            // Crash logs (Phase 60)
            .route("/:id/crash-logs", get(list_crash_logs).delete(clear_crash_logs))
            .route("/:id/crash-logs/:log_id/resolve", post(resolve_crash_log))
            // Health restart
            .route("/:id/health-restart", post(health_restart))
            // Server cleanup
            .route("/cleanup", post(cleanup))
            // Metrics
            .route("/metrics", get(metrics))
            .route("/:id/metrics", get(get_server_metrics))
            .route("/:id/metrics/history/:limit", get(get_server_metrics_history))
            // Server Properties
            .route("/:id/properties", get(get_server_properties).patch(update_server_properties))
            .with_state(state)
    }
}

async fn list_servers(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
) -> Result<impl IntoResponse, String> {
    // Filter by tenant_id for multi-tenant isolation
    let repo = SqlxServerRepository::new(state.pool.clone());
    let servers = repo.find_by_user_id(auth_user.tenant_id)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(Json(ApiResponse::success(servers)))
}

async fn create_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Json(payload): Json<CreateServerRequest>,
) -> Result<impl IntoResponse, String> {
    tracing::info!("create_server called: user_id={}, payload={:?}", auth_user.user_id, payload);
    
    let ram_mb = payload.ram_mb.unwrap_or(1024);
    let cpu_cores = payload.resources
        .as_ref()
        .and_then(|r| r.get("cpu").and_then(|c| c.as_i64()))
        .unwrap_or(1) as i32;
    let disk_gb = payload.resources
        .as_ref()
        .and_then(|r| r.get("disk").and_then(|d| {
            if let Some(s) = d.as_str() {
                s.strip_suffix('G').and_then(|n| n.parse::<i32>().ok())
            } else {
                d.as_i64().map(|n| n as i32)
            }
        }))
        .unwrap_or(5);
    
    tracing::info!("[create_server] After extract: ram_mb={}, cpu={}, disk={}", ram_mb, cpu_cores, disk_gb);
    tracing::info!("[create_server] Creating QuotaService...");
    
    let quota_service = crate::domain::usage::service::QuotaService::new(state.pool.clone());
    tracing::info!("[create_server] Calling quota check...");
    
    let quota_check = quota_service
        .check_server_creation(auth_user.user_id, ram_mb, cpu_cores, disk_gb)
        .await
        .map_err(|e| {
            tracing::error!("[create_server] Quota check error: {:?}", e);
            e.to_string()
        })?;
    tracing::info!("[create_server] Quota check passed: allowed={}", quota_check.allowed);
    if !quota_check.allowed {
        tracing::warn!("[create_server] QUOTA EXCEEDED: reasons={:?}", quota_check.reasons);
        let reasons: Vec<String> = quota_check.reasons
            .iter()
            .map(|v| format!("{}: requested {} but limit is {}", v.resource, v.requested, v.limit))
            .collect();
        return Err(format!("QUOTA_EXCEEDED: {}", reasons.join(", ")));
    }
    
    let repo = SqlxServerRepository::new(state.pool.clone());
    
    let mut server = Server::new(
        auth_user.user_id,
        payload.name.clone(),
        payload.image.clone(),
    );
    
    // Set tenant_id for multi-tenant isolation
    server.user_id = Some(auth_user.tenant_id);
    
    tracing::info!("[create_server] Prepared server: id={}, user_id={:?}, name={}", server.id, server.user_id, server.name);
    
    // Set additional fields for agent executor
    server.status = "pending".to_string();
    
    // Set port from payload
    server.port = payload.port;
    
    // Build config from UI fields
    let mut config = serde_json::json!({});
    
    if let Some(mc_version) = &payload.minecraft_version {
        config["minecraft_version"] = serde_json::json!(mc_version);
    }
    
    if let Some(ram_mb) = payload.ram_mb {
        config["ram_mb"] = serde_json::json!(ram_mb);
        config["ram"] = serde_json::json!(format!("{}M", ram_mb));
    }
    
    if let Some(max_players) = payload.max_players {
        config["max_players"] = serde_json::json!(max_players);
    }
    
    if let Some(game_type) = &payload.game_type {
        config["game_type"] = serde_json::json!(game_type);
    }
    
    // Set node_id if provided
    if let Some(node_id_str) = &payload.node_id {
        if let Ok(node_uuid) = Uuid::parse_str(node_id_str) {
            server.node_id = Some(node_uuid);
        }
    } else {
        // Auto-node-selection: find user's nodes and select one with least servers
        let node_repo = SqlxNodeRepository::new(state.pool.clone());
        if let Ok(user_nodes) = node_repo.find_by_user_id(&auth_user.tenant_id).await {
            if !user_nodes.is_empty() {
                // Simple selection: just pick the first available node
                // Could be enhanced to pick based on server count
                server.node_id = Some(user_nodes[0].id);
                tracing::info!("Auto-selected node {} for server {}", user_nodes[0].id, server.id);
            }
        }
    }
    
    server.config = config;
    
    // Set default resources based on RAM
    if let Some(ram_mb) = payload.ram_mb {
        let ram_gb = ram_mb as f64 / 1024.0;
        server.resources = serde_json::json!({
            "ram": format!("{}G", ram_gb.round() as i32),
            "cpu": 2,
            "disk": "10G"
        });
    }
    
    let created = repo.create(&server)
        .await
        .map_err(|e| {
            tracing::error!("[create_server] DB insert failed: {:?}, server: id={}, user_id={:?}, name={}", e, server.id, server.user_id, server.name);
            e.to_string()
        })?;

    tracing::info!("[create_server] Server created with id: {}", created.id);

    let audit_service = AuditService::new(state.pool.clone());
    let _ = audit_service.log_server_created(
        auth_user.tenant_id,
        auth_user.user_id,
        created.id,
        &created.name,
    ).await;
    
    emit_server_event(&state.pool, "server.created", auth_user.user_id, created.id, &created.name).await;
    
    Ok(Json(ApiResponse::success(created)))
}

async fn get_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;
    
    // Check tenant access
    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }
    
    Ok(Json(ApiResponse::success(server)))
}

async fn update_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateServerRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let mut server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    // Check tenant access
    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }
    
    if let Some(name) = payload.name {
        server.name = name;
    }
    if let Some(config) = payload.config {
        server.config = config;
    }
    if let Some(resources) = payload.resources {
        server.resources = resources;
    }
    if let Some(auto_wake) = payload.auto_wake {
        server.auto_wake = Some(auto_wake);
    }
    if let Some(sleep_timeout_minutes) = payload.sleep_timeout_minutes {
        server.sleep_timeout_minutes = Some(sleep_timeout_minutes);
    }
    if let Some(health_check_timeout_seconds) = payload.health_check_timeout_seconds {
        server.health_check_timeout_seconds = Some(health_check_timeout_seconds);
    }

    let updated = repo.update(&server)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(updated)))
}

async fn delete_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    // Check tenant access
    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }
    
    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);
    let executor_type = server.executor_type.clone();

    // For agent executor, send delete command via WebSocket
    if executor_type == "agent" {
        // Find node: use server.node_id if set and connected, otherwise find a connected node
        let node_id = if let Some(existing_node_id) = server.node_id {
            if state.node_client.is_connected(&existing_node_id).await {
                Some(existing_node_id)
            } else {
                None
            }
        } else {
            None
        };
        
        // If no connected node, find first online node
        let node_id = if let Some(nid) = node_id {
            Some(nid)
        } else {
            let all_nodes = state.node_repository.list().await
                .map_err(|e| e.to_string())?;
            
            let mut connected_node_id = None;
            for node in all_nodes.iter() {
                if state.node_client.is_connected(&node.id).await {
                    connected_node_id = Some(node.id);
                    break;
                }
            }
            connected_node_id
        };
        
        if let Some(node_id) = node_id {
            if state.node_client.is_connected(&node_id).await {
                let params = crate::presentation::ws::node_protocol::CommandParams {
                    container_name: Some(format!("mc-{}", server.id)),
                    container_id: None,
                    ..Default::default()
                };

                let _ = state.node_client.send_command_with_config(
                    node_id,
                    id,
                    "delete",
                    params,
                    None,
                ).await.map_err(|e| e.to_string())?;
            }
        }
    } else if let Some(remote_id) = &server.remote_id {
        state.solys_client.delete_server(remote_id)
            .await
            .map_err(|e| e.to_string())?;
    }

    repo.delete(id).await.map_err(|e| e.to_string())?;

    let audit_service = AuditService::new(state.pool.clone());
    let _ = audit_service.log_server_deleted(
        auth_user.tenant_id,
        auth_user.user_id,
        id,
        &server_name,
    ).await;

    emit_server_event(&state.pool, "server.deleted", user_id, id, &server_name).await;

    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "deleted" }))))
}

async fn start_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);
    let executor_type = server.executor_type.clone();
    let _node_id = server.node_id;

    // For agent executor, send command via WebSocket to connected node
    if executor_type == "agent" {
        // Find an actually connected node instead of using potentially stale server.node_id
        let target_node_id = if let Some(existing_node_id) = server.node_id {
            // Check if this node is actually connected
            if state.node_client.is_connected(&existing_node_id).await {
                tracing::info!("[START_SERVER] Using existing node_id from server: {}", existing_node_id);
                Some(existing_node_id)
            } else {
                tracing::warn!("[START_SERVER] Server has node_id {} but it's not connected, finding connected node", existing_node_id);
                None
            }
        } else {
            None
        };

        // If no connected node, find first online node
        let node_id = if let Some(nid) = target_node_id {
            nid
        } else {
            // Find first connected node using connection manager (not DB status)
            let all_nodes = state.node_repository.list().await
                .map_err(|e| e.to_string())?;
            
            let mut connected_node_id = None;
            for node in all_nodes.iter() {
                if state.node_client.is_connected(&node.id).await {
                    connected_node_id = Some(node.id);
                    tracing::info!("[START_SERVER] Auto-assigned to node: {} ({})", node.id, node.name);
                    break;
                }
            }

            match connected_node_id {
                Some(node_id) => {
                    // Save auto-assigned node_id to database
                    sqlx::query("UPDATE servers SET node_id = $1 WHERE id = $2")
                        .bind(node_id)
                        .bind(id)
                        .execute(&state.pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    
                    node_id
                }
                None => {
                    return Err("No online nodes available. Please wait for a node to come online.".to_string());
                }
            }
        };

        // Verify node is connected before sending
        if !state.node_client.is_connected(&node_id).await {
            return Err(format!("Node {} is not connected", node_id));
        }

        // Send start command to agent via WebSocket
        tracing::info!("=== START_SERVER: Sending command to node {}", node_id);
        
        let params = crate::presentation::ws::node_protocol::CommandParams {
            container_name: Some(format!("mc-{}", server.id)),
            ..Default::default()
        };
        
        let mc_version = server.config
            .get("minecraft_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "LATEST".to_string());
        let ram_mb = server.config
            .get("ram_mb")
            .and_then(|v| v.as_i64())
            .map(|v| v as u32)
            .unwrap_or(2048);
        
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("EULA".to_string(), "TRUE".to_string());
        env_vars.insert("MEMORY".to_string(), format!("{}M", ram_mb));
        
        let deploy_config = crate::presentation::ws::node_protocol::DeployConfig {
            image: server.image.clone(),
            game_port: Some(server.port.unwrap_or(25565) as u16),
            rcon_port: Some((server.port.unwrap_or(25565) + 10) as u16),
            ram_mb: Some(ram_mb),
            version: Some(mc_version),
            loader: Some("PAPER".to_string()),
            env_vars,
            volume_path: Some("/data".to_string()),
        };
        
        tracing::info!("=== START_SERVER DEBUG: Calling send_command_with_config...");
        
        let result = state.node_client.send_command_with_config(
            node_id,
            id,
            "start",
            params,
            Some(deploy_config),
        ).await;
        
        tracing::info!("=== START_SERVER DEBUG: Result: {:?}", result);
        
        match result {
            Ok(response) => {
                if response.success {
                    let mut updated = server;
                    updated.status = "running".to_string();
                    repo.update(&updated).await.map_err(|e| e.to_string())?;
                    
                    let audit_service = AuditService::new(state.pool.clone());
                    let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.started").await;
                    
                    emit_server_event(&state.pool, "server.started", user_id, id, &server_name).await;
                    
                    return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "started", "node_id": node_id }))));
                } else {
                    return Err(format!("Agent failed to start server: {}", response.output));
                }
            }
            Err(e) => {
                return Err(format!("Failed to send command to agent: {}", e));
            }
        }
    }

    // For solys executor (original behavior)
    let remote_id = server.remote_id.clone().ok_or("Server has no remote_id")?;

    let response: SolysOperationResponse = state.solys_client.start_server(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        let mut updated = server;
        updated.status = "running".to_string();
        repo.update(&updated).await.map_err(|e| e.to_string())?;
        
        let audit_service = AuditService::new(state.pool.clone());
        let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.started").await;
        
        emit_server_event(&state.pool, "server.started", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "success": response.success, "message": response.message }))))
}

async fn stop_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);
    let executor_type = server.executor_type.clone();

    // For agent executor, send stop command via WebSocket
    if executor_type == "agent" {
        // Find node: use server.node_id if set and connected, otherwise find a connected node
        let node_id = if let Some(existing_node_id) = server.node_id {
            if state.node_client.is_connected(&existing_node_id).await {
                Some(existing_node_id)
            } else {
                None
            }
        } else {
            None
        };
        
        // If no connected node, find first online node
        let node_id = if let Some(nid) = node_id {
            nid
        } else {
            let all_nodes = state.node_repository.list().await
                .map_err(|e| e.to_string())?;
            
            let mut connected_node_id = None;
            for node in all_nodes.iter() {
                if state.node_client.is_connected(&node.id).await {
                    connected_node_id = Some(node.id);
                    tracing::info!("[STOP_SERVER] Auto-assigned to node: {} ({})", node.id, node.name);
                    break;
                }
            }
            
            match connected_node_id {
                Some(node_id) => node_id,
                None => {
                    return Err("No online nodes available to stop the server".to_string());
                }
            }
        };
        
        // Save the node_id to the server record
        sqlx::query("UPDATE servers SET node_id = $1 WHERE id = $2")
            .bind(node_id)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let params = crate::presentation::ws::node_protocol::CommandParams {
            container_name: Some(format!("mc-{}", server.id)),
            container_id: None,
            ..Default::default()
        };

        let result = state.node_client.send_command_with_config(
            node_id,
            id,
            "stop",
            params,
            None,
        ).await.map_err(|e| e.to_string())?;

        if result.success {
            let mut updated = server;
            updated.status = "stopped".to_string();
            repo.update(&updated).await.map_err(|e| e.to_string())?;
            
            let audit_service = AuditService::new(state.pool.clone());
            let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.stopped").await;
            
            emit_server_event(&state.pool, "server.stopped", user_id, id, &server_name).await;
            
            return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "stopped" }))));
        } else {
            return Err(format!("Agent failed to stop server: {}", result.output));
        }
    }

    // For solys executor (original behavior)
    let remote_id = server.remote_id.clone().ok_or("Server has no remote_id")?;

    let response: SolysOperationResponse = state.solys_client.stop_server(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        let mut updated = server;
        updated.status = "stopped".to_string();
        repo.update(&updated).await.map_err(|e| e.to_string())?;
        
        let audit_service = AuditService::new(state.pool.clone());
        let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.stopped").await;
        
        emit_server_event(&state.pool, "server.stopped", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "success": response.success }))))
}

async fn restart_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);
    let executor_type = server.executor_type.clone();

    // For agent executor, send restart command via WebSocket
    if executor_type == "agent" {
        // Find node: use server.node_id if set and connected, otherwise find a connected node
        let node_id = if let Some(existing_node_id) = server.node_id {
            if state.node_client.is_connected(&existing_node_id).await {
                Some(existing_node_id)
            } else {
                None
            }
        } else {
            None
        };
        
        // If no connected node, find first online node
        let node_id = if let Some(nid) = node_id {
            nid
        } else {
            let all_nodes = state.node_repository.list().await
                .map_err(|e| e.to_string())?;
            
            let mut connected_node_id = None;
            for node in all_nodes.iter() {
                if state.node_client.is_connected(&node.id).await {
                    connected_node_id = Some(node.id);
                    tracing::info!("[RESTART_SERVER] Auto-assigned to node: {} ({})", node.id, node.name);
                    break;
                }
            }
            
            match connected_node_id {
                Some(node_id) => node_id,
                None => {
                    return Err("No online nodes available to restart the server".to_string());
                }
            }
        };
        
        // Save the node_id to the server record
        sqlx::query("UPDATE servers SET node_id = $1 WHERE id = $2")
            .bind(node_id)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let params = crate::presentation::ws::node_protocol::CommandParams {
            container_name: Some(format!("mc-{}", server.id)),
            container_id: None,
            ..Default::default()
        };

        let result = state.node_client.send_command_with_config(
            node_id,
            id,
            "restart",
            params,
            None,
        ).await.map_err(|e| e.to_string())?;

        if result.success {
            let audit_service = AuditService::new(state.pool.clone());
            let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.restarted").await;
            
            emit_server_event(&state.pool, "server.restarted", user_id, id, &server_name).await;
            
            return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "restarted" }))));
        } else {
            return Err(format!("Agent failed to restart server: {}", result.output));
        }
    }

    // For solys executor (original behavior)
    let remote_id = server.remote_id.clone().ok_or("Server has no remote_id")?;

    let response: SolysOperationResponse = state.solys_client.restart_server(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        let audit_service = AuditService::new(state.pool.clone());
        let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.restarted").await;
        
        emit_server_event(&state.pool, "server.restarted", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "success": response.success }))))
}

async fn sleep_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let mut server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);
    let executor_type = server.executor_type.clone();

    // === Same stop logic as stop_server handler ===
    if executor_type == "agent" {
        let node_id = if let Some(existing_node_id) = server.node_id {
            if state.node_client.is_connected(&existing_node_id).await {
                Some(existing_node_id)
            } else {
                None
            }
        } else {
            None
        };

        let node_id = if let Some(nid) = node_id {
            nid
        } else {
            let all_nodes = state.node_repository.list().await
                .map_err(|e| e.to_string())?;
            let mut connected_node_id = None;
            for node in all_nodes.iter() {
                if state.node_client.is_connected(&node.id).await {
                    connected_node_id = Some(node.id);
                    break;
                }
            }
            match connected_node_id {
                Some(node_id) => node_id,
                None => return Err("No online nodes available to sleep the server".to_string()),
            }
        };

        sqlx::query("UPDATE servers SET node_id = $1 WHERE id = $2")
            .bind(node_id)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let params = crate::presentation::ws::node_protocol::CommandParams {
            container_name: Some(format!("mc-{}", server.id)),
            container_id: None,
            ..Default::default()
        };

        let result = state.node_client.send_command_with_config(
            node_id,
            id,
            "stop",
            params,
            None,
        ).await.map_err(|e| e.to_string())?;

        if result.success {
            server.status = "stopped".to_string();
            server.auto_wake = Some(true);
            repo.update(&server).await.map_err(|e| e.to_string())?;

            let audit_service = crate::domain::audit::service::AuditService::new(state.pool.clone());
            let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.sleep").await;

            emit_server_event(&state.pool, "server.sleep", user_id, id, &server_name).await;

            return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "stopped", "auto_wake": true }))));
        } else {
            return Err(format!("Agent failed to sleep server: {}", result.output));
        }
    }

    // For solys executor
    let remote_id = server.remote_id.clone().ok_or("Server has no remote_id")?;
    let response: crate::infrastructure::solys_client::client::SolysOperationResponse = state.solys_client.stop_server(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        server.status = "stopped".to_string();
        server.auto_wake = Some(true);
        repo.update(&server).await.map_err(|e| e.to_string())?;

        emit_server_event(&state.pool, "server.sleep", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "stopped", "auto_wake": true }))))
}

async fn wake_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    // Only wake servers that are in sleep mode
    let is_sleeping = server.auto_wake.unwrap_or(false);
    if !is_sleeping {
        return Err("Server is not in sleep mode".to_string());
    }

    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);
    let executor_type = server.executor_type.clone();
    let _node_id = server.node_id;

    // === Same start logic as start_server handler ===
    if executor_type == "agent" {
        let target_node_id = if let Some(existing_node_id) = server.node_id {
            if state.node_client.is_connected(&existing_node_id).await {
                tracing::info!("[WAKE_SERVER] Using existing node_id: {}", existing_node_id);
                Some(existing_node_id)
            } else {
                None
            }
        } else {
            None
        };

        let node_id = if let Some(nid) = target_node_id {
            nid
        } else {
            let all_nodes = state.node_repository.list().await
                .map_err(|e| e.to_string())?;
            let mut connected_node_id = None;
            for node in all_nodes.iter() {
                if state.node_client.is_connected(&node.id).await {
                    connected_node_id = Some(node.id);
                    break;
                }
            }
            match connected_node_id {
                Some(node_id) => {
                    sqlx::query("UPDATE servers SET node_id = $1 WHERE id = $2")
                        .bind(node_id)
                        .bind(id)
                        .execute(&state.pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    node_id
                }
                None => return Err("No online nodes available to wake the server".to_string()),
            }
        };

        let params = crate::presentation::ws::node_protocol::CommandParams {
            container_name: Some(format!("mc-{}", server.id)),
            ..Default::default()
        };

        let mc_version = server.config
            .get("minecraft_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "LATEST".to_string());
        let ram_mb_val = server.config
            .get("ram_mb")
            .and_then(|v| v.as_i64())
            .map(|v| v as u32)
            .unwrap_or(2048);

        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("EULA".to_string(), "TRUE".to_string());
        env_vars.insert("MEMORY".to_string(), format!("{}M", ram_mb_val));

        let deploy_config = crate::presentation::ws::node_protocol::DeployConfig {
            image: server.image.clone(),
            game_port: Some(server.port.unwrap_or(25565) as u16),
            rcon_port: Some((server.port.unwrap_or(25565) + 10) as u16),
            ram_mb: Some(ram_mb_val),
            version: Some(mc_version),
            loader: Some("PAPER".to_string()),
            env_vars,
            volume_path: Some("/data".to_string()),
        };

        let result = state.node_client.send_command_with_config(
            node_id,
            id,
            "start",
            params,
            Some(deploy_config),
        ).await;

        match result {
            Ok(response) => {
                if response.success {
                    let mut updated = server;
                    updated.status = "running".to_string();
                    updated.auto_wake = Some(false);
                    repo.update(&updated).await.map_err(|e| e.to_string())?;

                    let audit_service = crate::domain::audit::service::AuditService::new(state.pool.clone());
                    let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.wake").await;

                    emit_server_event(&state.pool, "server.wake", user_id, id, &server_name).await;

                    return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "started", "auto_wake": false }))));
                } else {
                    return Err(format!("Agent failed to wake server: {}", response.output));
                }
            }
            Err(e) => return Err(format!("Failed to send wake command to agent: {}", e)),
        }
    }

    // For solys executor
    let remote_id = server.remote_id.clone().ok_or("Server has no remote_id")?;
    let response: crate::infrastructure::solys_client::client::SolysOperationResponse = state.solys_client.start_server(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        let mut updated = server;
        updated.status = "running".to_string();
        updated.auto_wake = Some(false);
        repo.update(&updated).await.map_err(|e| e.to_string())?;

        emit_server_event(&state.pool, "server.wake", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "status": "started", "auto_wake": false }))))
}

async fn kill_server(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.clone().ok_or("Server has no remote_id")?;
    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);

    let response: SolysOperationResponse = state.solys_client.kill_server(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        let mut updated = server;
        updated.status = "stopped".to_string();
        repo.update(&updated).await.map_err(|e| e.to_string())?;
        
        let audit_service = AuditService::new(state.pool.clone());
        let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.killed").await;
        
        emit_server_event(&state.pool, "server.killed", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::success(response)))
}

async fn get_status(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let status: SolysServerStatus = state.solys_client.get_status(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(status)))
}

async fn get_stats(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let stats: SolysServerStats = state.solys_client.get_stats(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(stats)))
}

pub async fn get_logs(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path((id, lines)): Path<(Uuid, usize)>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    // For agent executor, request logs from agent
    if server.executor_type == "agent" {
        eprintln!("=== GET_LOGS: executor_type=agent, server_id={}", id);
        
        // Find node: use server.node_id if set and connected, otherwise find a connected node
        let node_id = if let Some(existing_node_id) = server.node_id {
            if state.node_client.is_connected(&existing_node_id).await {
                Some(existing_node_id)
            } else {
                None
            }
        } else {
            None
        };
        
        // If no connected node, find first online node
        let node_id = if let Some(nid) = node_id {
            nid
        } else {
            let all_nodes = state.node_repository.list().await
                .map_err(|e| e.to_string())?;
            
            let mut connected_node_id = None;
            for node in all_nodes.iter() {
                if state.node_client.is_connected(&node.id).await {
                    connected_node_id = Some(node.id);
                    tracing::info!("[GET_LOGS] Auto-assigned to node: {} ({})", node.id, node.name);
                    break;
                }
            }
            
            match connected_node_id {
                Some(node_id) => node_id,
                None => {
                    // Fallback: Try Docker on same machine
                    let container_name = format!("mc-{}", server.id);
                    match get_docker_logs(&container_name, lines).await {
                        Ok(logs) => {
                            let logs_json = serde_json::json!({ "logs": logs });
                            return Ok(Json(ApiResponse::<serde_json::Value>::success(logs_json)));
                        }
                        Err(_) => {
                            return Err("No online nodes available and server not found locally".to_string());
                        }
                    }
                }
            }
        };
        
        // Save node_id to database for future operations
        sqlx::query("UPDATE servers SET node_id = $1 WHERE id = $2")
            .bind(node_id)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        eprintln!("=== GET_LOGS: Checking node connection for node_id={}", node_id);
        
        if !state.node_client.is_connected(&node_id).await {
            eprintln!("=== GET_LOGS: Node {} is NOT connected, trying Docker fallback", node_id);
            
            // Fallback: Try to read logs directly from Docker (same machine as API)
            let container_name = format!("mc-{}", server.id);
            eprintln!("=== GET_LOGS: Docker fallback trying container={}", container_name);
            
            match get_docker_logs(&container_name, lines).await {
                Ok(logs) => {
                    eprintln!("=== GET_LOGS: Docker fallback succeeded, got {} chars", logs.len());
                    let logs_json = serde_json::json!({ "logs": logs });
                    return Ok(Json(ApiResponse::<serde_json::Value>::success(logs_json)));
                }
                Err(e) => {
                    eprintln!("=== GET_LOGS: Docker fallback failed: {}", e);
                    return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "logs": "Server is offline" }))));
                }
            }
        }

        eprintln!("=== GET_LOGS: Node {} IS connected, sending logs command", node_id);

        // Use mc-{server_id} as container name (this matches how containers are created)
        let container_name = format!("mc-{}", server.id);
        eprintln!("=== GET_LOGS: Using container_name={}", container_name);
        
        // Check if streaming is requested via query param (need to parse from request)
        // For now, just send regular logs - streaming will be handled separately
        // When follow=true, the logs will be streamed via WebSocket events
        let params = crate::presentation::ws::node_protocol::CommandParams {
            container_id: None,
            container_name: Some(container_name),
            follow: Some(false),  // Static fetch by default
            tail: Some(lines as u32),
            ..Default::default()
        };

        let result = state.node_client.send_command_with_config(
            node_id,
            id,
            "logs",
            params,
            None,
        ).await;

        eprintln!("=== GET_LOGS: send_command_with_config result = {:?}", result);

        match result {
            Ok(response) => {
                // DEBUG: Check exact format of response
                tracing::info!("=== LOGS: response.output length = {}", response.output.len());
                tracing::info!("=== LOGS: response.output starts with = {:?}", &response.output[..response.output.len().min(100)]);
                
                // Check if output is valid JSON
                let logs_text = match serde_json::from_str::<serde_json::Value>(&response.output) {
                    Ok(v) => {
                        tracing::info!("=== LOGS: JSON parsed OK");
                        // Try to extract lines
                        if let Some(lines) = v.get("lines").and_then(|l| l.as_array()) {
                            let text = lines.iter()
                                .filter_map(|x| x.as_str())
                                .collect::<Vec<_>>()
                                .join("\n");
                            tracing::info!("=== LOGS: Extracted {} lines, total chars = {}", lines.len(), text.len());
                            if text.is_empty() {
                                eprintln!("=== GET_LOGS: WARNING - extracted text is EMPTY!");
                            }
                            text
                        } else {
                            tracing::warn!("=== LOGS: No 'lines' field, using raw output");
                            response.output
                        }
                    },
                    Err(e) => {
                        tracing::error!("=== LOGS: JSON parse failed: {}", e);
                        response.output
                    }
                };
                
                let logs = serde_json::json!({ "logs": logs_text });
                return Ok(Json(ApiResponse::<serde_json::Value>::success(logs)));
            }
            Err(e) => {
                return Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "logs": format!("Failed to fetch logs: {}", e) }))));
            }
        }
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let logs: String = state.solys_client.get_logs(&remote_id, lines)
        .await
        .map_err(|e| e.to_string())?;

    let logs_value = serde_json::json!({ "logs": logs });
    Ok(Json(ApiResponse::<serde_json::Value> { 
        success: true, 
        data: Some(logs_value), 
        error: None, 
        request_id: None 
    }))
}

async fn stream_logs(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Query(params): Query<StreamLogsQuery>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    if server.executor_type != "agent" {
        return Err("Streaming only supported for agent-based servers".to_string());
    }

    // Find node: use server.node_id if set and connected, otherwise find a connected node
    let node_id = if let Some(existing_node_id) = server.node_id {
        if state.node_client.is_connected(&existing_node_id).await {
            Some(existing_node_id)
        } else {
            None
        }
    } else {
        None
    };
    
    // If no connected node, find first online node
    let node_id = if let Some(nid) = node_id {
        nid
    } else {
        let all_nodes = state.node_repository.list().await
            .map_err(|e| e.to_string())?;
        
        let mut connected_node_id = None;
        for node in all_nodes.iter() {
            if state.node_client.is_connected(&node.id).await {
                connected_node_id = Some(node.id);
                tracing::info!("[STREAM_LOGS] Auto-assigned to node: {} ({})", node.id, node.name);
                break;
            }
        }
        
        match connected_node_id {
            Some(node_id) => node_id,
            None => {
                return Err("No online nodes available".to_string());
            }
        }
    };
    
    // Save node_id to database
    sqlx::query("UPDATE servers SET node_id = $1 WHERE id = $2")
        .bind(node_id)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let container_name = format!("mc-{}", server.id);
    
    let params = crate::presentation::ws::node_protocol::CommandParams {
        container_id: None,
        container_name: Some(container_name),
        follow: Some(true),
        tail: Some(params.tail.unwrap_or(100)),
        ..Default::default()
    };

    let result = state.node_client.send_command_with_config(
        node_id,
        id,
        "logs",
        params,
        None,
    ).await;

    match result {
        Ok(response) => {
            Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({
                "status": "streaming",
                "message": "Log streaming started - logs will be delivered via WebSocket",
                "output": response.output
            }))))
        }
        Err(e) => {
            Err(format!("Failed to start log stream: {}", e))
        }
    }
}

#[derive(serde::Deserialize)]
struct StreamLogsQuery {
    tail: Option<u32>,
}

async fn send_command(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CommandRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let response: SolysOperationResponse = state.solys_client.send_command(&remote_id, &payload.command)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(response)))
}

async fn get_health(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let health = state.solys_client.get_health(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(health)))
}

async fn list_files(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileListRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let files: SolysFileListResponse = state.solys_client.list_files(&remote_id, &payload.path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(files)))
}

async fn read_file(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileReadRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let content: String = state.solys_client.read_file(&remote_id, &payload.path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::<serde_json::Value>::success(serde_json::json!({ "content": content }))))
}

async fn write_file(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileWriteRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let success = state.solys_client.write_file(&remote_id, &payload.path, &payload.content, payload.overwrite)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(success)))
}

async fn delete_file(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileDeleteRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let success = state.solys_client.delete_file(&remote_id, &payload.path, payload.recursive)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(success)))
}

async fn mkdir(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileMkdirRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let success = state.solys_client.mkdir(&remote_id, &payload.path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(success)))
}

async fn rename_file(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileRenameRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let success = state.solys_client.rename_file(&remote_id, &payload.old_path, &payload.new_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(success)))
}

async fn copy_file(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileCopyRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;

    let success = state.solys_client.copy_file(&remote_id, &payload.source_path, &payload.dest_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(success)))
}

async fn update_image(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateImageRequest>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;
    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);

    let response: SolysOperationResponse = state.solys_client.update_server_image(&remote_id, &payload.image)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        let audit_service = AuditService::new(state.pool.clone());
        let _ = audit_service.log_server_action(auth_user.tenant_id, auth_user.user_id, id, "server.image_updated").await;
        
        emit_server_event(&state.pool, "server.image_updated", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::success(response)))
}

async fn cleanup(
    State(state): State<ApiState>,
) -> Result<impl IntoResponse, String> {
    let response = state.solys_client.cleanup_servers()
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ApiResponse::success(response)))
}

async fn health_restart(
    State(state): State<ApiState>,
    auth_user: VerifiedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    let repo = SqlxServerRepository::new(state.pool.clone());
    let server = repo.find_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if server.user_id.as_ref() != Some(&auth_user.tenant_id) {
        return Err("Access denied".to_string());
    }

    let remote_id = server.remote_id.ok_or("Server has no remote_id")?;
    let server_name = server.name.clone();
    let user_id = server.user_id.unwrap_or(auth_user.tenant_id);

    let response: SolysOperationResponse = state.solys_client.health_restart(&remote_id)
        .await
        .map_err(|e| e.to_string())?;

    if response.success {
        emit_server_event(&state.pool, "server.health_restarted", user_id, id, &server_name).await;
    }

    Ok(Json(ApiResponse::success(response)))
}

async fn metrics(
    State(state): State<ApiState>,
) -> Result<impl IntoResponse, String> {
    let metrics = state.solys_client.get_metrics()
        .await
        .map_err(|e| e.to_string())?;

    Ok(metrics)
}

async fn get_server_metrics(
    State(state): State<ApiState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, String> {
    tracing::debug!("get_server_metrics: server_id={}", server_id);
    
    let repo = SqlxServerRepository::new(state.pool.clone());
    let metrics_repo = crate::infrastructure::repositories::postgres_metrics_repository::PostgresMetricsRepository::new(state.pool.clone());
    
    // Verify server exists and user has access
    let _server = repo.find_by_id(server_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Server not found")?;
    
    let metrics = metrics_repo.get_latest(&server_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No metrics found")?;
    
    Ok(Json(ApiResponse::success(metrics)))
}

async fn get_server_metrics_history(
    State(state): State<ApiState>,
    Path((server_id, limit)): Path<(Uuid, i64)>,
) -> Result<impl IntoResponse, String> {
    tracing::info!("get_server_metrics_history START: server_id={}, limit={}", server_id, limit);
    
    let repo = SqlxServerRepository::new(state.pool.clone());
    let metrics_repo = crate::infrastructure::repositories::postgres_metrics_repository::PostgresMetricsRepository::new(state.pool.clone());
    
    // Verify server exists and user has access
    let server = repo.find_by_id(server_id)
        .await
        .map_err(|e| {
            tracing::error!("Error finding server: {:?}", e);
            e.to_string()
        })?
        .ok_or_else(|| {
            tracing::error!("Server not found: {}", server_id);
            "Server not found".to_string()
        })?;
    
    tracing::info!("Server found: {}", server.name);
    
    let history = metrics_repo.get_history(&server_id, limit)
        .await
        .map_err(|e| {
            tracing::error!("Error getting metrics history: {:?}", e);
            e.to_string()
        })?;
    
    tracing::info!("get_server_metrics_history END: found {} records", history.len());
    
    Ok(Json(ApiResponse::success(history)))
}

// ── Crash Log Handlers (Phase 60) ──────────────────────────────────────────────

/// List crash logs for a server (paginated).
pub async fn list_crash_logs(
    Path(server_id): Path<Uuid>,
    State(container): State<ApiState>,
    Query(params): Query<CrashLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let repo = PostgresCrashLogRepository::new(container.pool.clone());
    let logs = repo
        .list_by_server(server_id, limit as i64, offset as i64)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to list crash logs: {}", e)))?;
    let total = repo
        .count_by_server(server_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to count crash logs: {}", e)))?;

    Ok(ApiResponse::success(json!({
        "logs": logs,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

/// Clear all crash logs for a server.
pub async fn clear_crash_logs(
    Path(server_id): Path<Uuid>,
    State(container): State<ApiState>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PostgresCrashLogRepository::new(container.pool.clone());
    repo.delete_by_server(server_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to clear crash logs: {}", e)))?;

    Ok(ApiResponse::success(json!({ "cleared": true })))
}

/// Resolve/acknowledge a single crash log.
pub async fn resolve_crash_log(
    Path((server_id, log_id)): Path<(Uuid, Uuid)>,
    State(container): State<ApiState>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PostgresCrashLogRepository::new(container.pool.clone());
    repo.resolve(log_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to resolve crash log: {}", e)))?;

    Ok(ApiResponse::success(json!({ "resolved": true, "log_id": log_id, "server_id": server_id })))
}

#[derive(Deserialize)]
pub struct CrashLogQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ── End Crash Log Handlers ─────────────────────────────────────────────────────

/// Helper function to read logs directly from Docker CLI (fallback when agent is not connected)
async fn get_docker_logs(container_name: &str, tail: usize) -> Result<String, String> {
    use tokio::process::Command;
    
    let output = Command::new("docker")
        .args(["logs", "--tail", &tail.to_string(), container_name])
        .output()
        .await
        .map_err(|e| format!("Failed to run docker logs: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Docker logs failed: {}", stderr));
    }
    
    let logs = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(logs)
}
