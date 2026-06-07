use std::sync::Arc;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod backend;
mod config;
mod error;
mod heartbeat;
mod metrics;
mod player;
mod ratelimit;
mod registry;
mod session_log;
mod state;
mod tunnel;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize logging
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log_level))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // 2. Load config
    let cfg = config::Config::load()?;
    info!("[MAIN] Config loaded; tunnel_bind={}, player_bind={}, metrics_bind={}",
        cfg.server.tunnel_bind, cfg.server.player_bind, cfg.server.metrics_bind);

    // 3. Build shared state
    let state = Arc::new(AppState::new(cfg).await?);
    info!("[MAIN] AppState initialized");

    // 4. Spawn background tasks
    let metrics_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = metrics::run_metrics_server(metrics_state).await {
            error!("[METRICS] Server stopped: {}", e);
        }
    });

    let player_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = player::run_player_listener(player_state).await {
            error!("[PLAYER] Listener stopped: {}", e);
        }
    });

    let heartbeat_state = state.clone();
    tokio::spawn(async move {
        heartbeat::run_heartbeat_watcher(heartbeat_state).await;
    });

    // 5. Build the axum router for the tunnel listener
    let app = Router::new()
        .route("/relay/tunnel", get(tunnel_upgrade))
        .route("/healthz", get(healthz))
        .with_state(state.clone());

    // 6. Start tunnel listener
    let listener = tokio::net::TcpListener::bind(state.config.server.tunnel_bind.clone()).await?;
    info!("[MAIN] Tunnel listener bound on {}", state.config.server.tunnel_bind);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn tunnel_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket: WebSocket| async move {
        tunnel::run_tunnel_session(socket, state).await;
    })
}

async fn healthz() -> &'static str {
    "OK"
}
