use std::sync::Arc;

use axum::{routing::get, Router};
use prometheus::{
    register_counter, register_counter_vec, register_gauge, register_int_counter_vec,
    Counter, CounterVec, Encoder, Gauge, IntCounterVec, Registry, TextEncoder,
};
use tracing::error;

use crate::state::AppState;

// ---- Metric definitions (lazy_static-style via OnceLock is unnecessary with prometheus crate) ----

pub static METRICS_REGISTRY: once_cell::sync::Lazy<Registry> =
    once_cell::sync::Lazy::new(Registry::new);

pub static ACTIVE_TUNNELS: once_cell::sync::Lazy<Gauge> = once_cell::sync::Lazy::new(|| {
    let g = Gauge::new("relay_active_tunnels", "Number of active tunnels").unwrap();
    METRICS_REGISTRY.register(Box::new(g.clone())).unwrap();
    g
});

pub static TOTAL_CONNECTIONS: once_cell::sync::Lazy<Counter> = once_cell::sync::Lazy::new(|| {
    let c = Counter::new("relay_total_connections", "Total inbound connections (player + tunnel)").unwrap();
    METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static REJECTED_CONNECTIONS: once_cell::sync::Lazy<Counter> = once_cell::sync::Lazy::new(|| {
    let c = Counter::new("relay_rejected_connections", "Connections rejected (no tunnel / invalid)").unwrap();
    METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static AUTH_FAILURES: once_cell::sync::Lazy<Counter> = once_cell::sync::Lazy::new(|| {
    let c = Counter::new("relay_auth_failures", "Authentication failures (HMAC / token)").unwrap();
    METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static RATE_LIMITED: once_cell::sync::Lazy<Counter> = once_cell::sync::Lazy::new(|| {
    let c = Counter::new("relay_rate_limited", "Requests rejected by per-IP rate limit").unwrap();
    METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static PLAYER_BYTES_IN: once_cell::sync::Lazy<Counter> = once_cell::sync::Lazy::new(|| {
    let c = Counter::new("relay_player_bytes_in_total", "Total bytes received from players").unwrap();
    METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static PLAYER_BYTES_OUT: once_cell::sync::Lazy<Counter> = once_cell::sync::Lazy::new(|| {
    let c = Counter::new("relay_player_bytes_out_total", "Total bytes sent to players").unwrap();
    METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static TUNNEL_EVENTS_TOTAL: once_cell::sync::Lazy<IntCounterVec> =
    once_cell::sync::Lazy::new(|| {
        let c = IntCounterVec::new(
            prometheus::Opts::new(
                "relay_tunnel_events_total",
                "Tunnel lifecycle events (connected/disconnected/heartbeat/stale)",
            ),
            &["event_type"],
        )
        .unwrap();
        METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
        c
    });

pub static BANDWIDTH_PER_SUBDOMAIN: once_cell::sync::Lazy<CounterVec> =
    once_cell::sync::Lazy::new(|| {
        let c = CounterVec::new(
            prometheus::Opts::new(
                "relay_bandwidth_bytes_per_subdomain",
                "Bytes relayed per subdomain (sum of in+out)",
            ),
            &["subdomain", "direction"],
        )
        .unwrap();
        METRICS_REGISTRY.register(Box::new(c.clone())).unwrap();
        c
    });

// ---- Keep the legacy `register_*` helpers available so the macros above work on older prometheus versions. ----

#[allow(dead_code)]
fn _legacy_register_dummy() {
    let _ = register_counter!("_dummy", "_dummy");
    let _ = register_counter_vec!("_dummy", "_dummy", &["x"]);
    let _ = register_gauge!("_dummy", "_dummy");
    let _ = register_int_counter_vec!("_dummy", "_dummy", &["x"]);
}

pub async fn metrics_handler() -> impl axum::response::IntoResponse {
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("[METRICS] Failed to encode: {}", e);
    }
    (
        [(
            axum::http::header::CONTENT_TYPE,
            encoder.format_type().to_string(),
        )],
        buffer,
    )
}

pub async fn run_metrics_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let metrics_bind = state.config.server.metrics_bind.clone();
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(metrics_bind.clone()).await?;
    tracing::info!("[METRICS] Listening on {}", metrics_bind);
    axum::serve(listener, app).await?;
    Ok(())
}
