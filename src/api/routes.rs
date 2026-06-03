//! API routes - /health, /status, /metrics

#![allow(dead_code)]

use std::time::Instant;
use std::sync::Arc;

use axum::{
    extract::Query,
    middleware,
    routing::{get, post, put},
    Json, Router,
    response::sse::{Event, Sse},
};
use tokio_stream::StreamExt;

use crate::api::middleware::tracing::trace_id_middleware;
use crate::task_state::TASK_STATE_TRACKER;
use crate::task_state;

lazy_static::lazy_static! {
    pub static ref START_TIME: Instant = Instant::now();
}

#[derive(Clone)]
struct ApiState;

pub fn create_router() -> Router {
    Router::new()
        // Health and status
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/metrics", get(metrics))
        .route("/version", get(version))
        // Control endpoints
        .route("/start", post(start_agent))
        .route("/stop", post(stop_agent))
        .route("/restart", post(restart_agent))
        // Logs and config
        .route("/logs", get(get_logs))
        .route("/config", get(get_config))
        .route("/config", put(update_config))
        // Events (SSE)
        .route("/events", get(event_stream))
        .layer(middleware::from_fn(trace_id_middleware))
        .with_state(Arc::new(ApiState))
}

#[derive(serde::Serialize)]
pub struct HealthResponse {
    status: String,
    connected: bool,
    runtime: Option<String>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        connected: true,
        runtime: Some("docker".to_string()),
    })
}

#[derive(serde::Serialize)]
pub struct StatusResponse {
    agent_id: Option<uuid::Uuid>,
    agent_name: String,
    version: String,
    runtime: String,
    capabilities: Vec<String>,
    connected: bool,
    running_tasks: usize,
    uptime_secs: u64,
}

async fn status() -> Json<StatusResponse> {
    // Get agent_id from global state
    let agent_id = task_state::get_agent_node_id();
    
    // Get running tasks from tracker
    let running_tasks = TASK_STATE_TRACKER.list_running().await;
    let running_count = running_tasks.len();
    
    // Get uptime
    let uptime_secs = START_TIME.elapsed().as_secs();
    
    // Check connection status
    let connected = agent_id.is_some();
    
    Json(StatusResponse {
        agent_id,
        agent_name: "web-agent".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        runtime: "docker".to_string(),
        capabilities: vec![
            "Docker".to_string(),
            "ServerCreate".to_string(),
            "ServerStart".to_string(),
            "ServerStop".to_string(),
            "ServerRestart".to_string(),
            "ServerDelete".to_string(),
            "ServerLogs".to_string(),
            "ServerCommand".to_string(),
            "BackupCreate".to_string(),
            "BackupRestore".to_string(),
            "Metrics".to_string(),
        ],
        connected,
        running_tasks: running_count,
        uptime_secs,
    })
}

async fn metrics() -> String {
    let sys = agent_metrics::collect_system_metrics();
    
    // Alert thresholds (D-01)
    let cpu_threshold = 80.0;
    let memory_threshold = 85.0;
    let disk_threshold = 90.0;
    
    // Calculate alert states
    let cpu_alert_active = if sys.cpu_percent > cpu_threshold { 1 } else { 0 };
    let memory_alert_active = if sys.memory_percent() > memory_threshold { 1 } else { 0 };
    
    // Calculate max disk usage percentage
    let max_disk_percent = sys.disk_usage.iter()
        .filter(|d| d.total_bytes > 0)
        .map(|d| (d.used_bytes as f64 / d.total_bytes as f64) * 100.0)
        .fold(0.0_f64, |a, b| a.max(b));
    let disk_alert_active = if max_disk_percent > disk_threshold { 1 } else { 0 };
    
    format!(
        "# HELP agent_cpu_percent CPU usage percentage\n\
         # TYPE agent_cpu_percent gauge\n\
         agent_cpu_percent {}\n\
         # HELP agent_memory_bytes Agent memory usage\n\
         # TYPE agent_memory_bytes gauge\n\
         agent_memory_used_bytes {}\n\
         agent_memory_total_bytes {}\n\
         # HELP agent_alert_cpu_threshold CPU alert threshold\n\
         # TYPE agent_alert_cpu_threshold gauge\n\
         agent_alert_cpu_threshold {}\n\
         # HELP agent_alert_memory_threshold Memory alert threshold\n\
         # TYPE agent_alert_memory_threshold gauge\n\
         agent_alert_memory_threshold {}\n\
         # HELP agent_alert_disk_threshold Disk alert threshold\n\
         # TYPE agent_alert_disk_threshold gauge\n\
         agent_alert_disk_threshold {}\n\
         # HELP agent_alert_cpu_active CPU alert active\n\
         # TYPE agent_alert_cpu_active gauge\n\
         agent_alert_cpu_active {}\n\
         # HELP agent_alert_memory_active Memory alert active\n\
         # TYPE agent_alert_memory_active gauge\n\
         agent_alert_memory_active {}\n\
         # HELP agent_alert_disk_active Disk alert active\n\
         # TYPE agent_alert_disk_active gauge\n\
         agent_alert_disk_active {}\n",
        sys.cpu_percent,
        sys.memory_used_bytes,
        sys.memory_total_bytes,
        cpu_threshold,
        memory_threshold,
        disk_threshold,
        cpu_alert_active,
        memory_alert_active,
        disk_alert_active
    )
}

// Version endpoint
#[derive(serde::Serialize)]
pub struct VersionResponse {
    version: String,
    name: String,
    build_date: String,
}

async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "escluse-agent".to_string(),
        build_date: option_env!("BUILD_DATE").unwrap_or("unknown").to_string(),
    })
}

// Control endpoints
#[derive(serde::Serialize)]
pub struct ControlResponse {
    success: bool,
    message: String,
}

async fn start_agent() -> Json<ControlResponse> {
    // TODO: Implement agent start logic
    Json(ControlResponse {
        success: true,
        message: "Agent started".to_string(),
    })
}

async fn stop_agent() -> Json<ControlResponse> {
    // TODO: Implement agent stop logic
    Json(ControlResponse {
        success: true,
        message: "Agent stopped".to_string(),
    })
}

async fn restart_agent() -> Json<ControlResponse> {
    // TODO: Implement agent restart logic
    Json(ControlResponse {
        success: true,
        message: "Agent restarted".to_string(),
    })
}

// Logs endpoint
#[derive(serde::Deserialize)]
pub struct LogsQuery {
    lines: Option<usize>,
    stream: Option<bool>,
}

#[derive(serde::Serialize)]
pub struct LogsResponse {
    logs: Vec<String>,
    total_lines: usize,
}

async fn get_logs(Query(params): Query<LogsQuery>) -> Json<LogsResponse> {
    let _lines = params.lines.unwrap_or(100);
    // TODO: Implement actual log retrieval
    Json(LogsResponse {
        logs: vec![
            "2024-05-07 10:00:00 INFO Starting escluse-agent...".to_string(),
            "2024-05-07 10:00:01 INFO Connected to backend".to_string(),
            "2024-05-07 10:00:02 INFO Agent ready".to_string(),
        ],
        total_lines: 3,
    })
}

// Config endpoints
#[derive(serde::Serialize)]
pub struct ConfigResponse {
    api_key: String,
    backend_url: String,
    agent_name: String,
    log_level: String,
}

async fn get_config() -> Json<ConfigResponse> {
    let config = agent_config::load();
    Json(ConfigResponse {
        api_key: {
            let api_key_str = config.api_key.expose_secret();
            if !config.api_key.is_empty() {
                api_key_str.chars().take(8).collect::<String>() + "..."
            } else {
                "".to_string()
            }
        },
        backend_url: config.backend_url,
        agent_name: config.agent_name,
        log_level: config.log_level,
    })
}

#[derive(serde::Deserialize)]
pub struct ConfigUpdateRequest {
    api_key: Option<String>,
    backend_url: Option<String>,
    agent_name: Option<String>,
    log_level: Option<String>,
}

async fn update_config(Json(_req): Json<ConfigUpdateRequest>) -> Json<ControlResponse> {
    // TODO: Implement config update logic
    Json(ControlResponse {
        success: true,
        message: "Configuration updated".to_string(),
    })
}

// Events endpoint (SSE)
use tokio::sync::broadcast;
use std::convert::Infallible;

async fn event_stream() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let (_tx, rx) = broadcast::channel::<String>(100);
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx);
    
    Sse::new(stream.map(|_| Ok(Event::default().data("Event streaming not yet implemented"))))
}
