use std::time::Duration;

use tracing::{error, info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Run the heartbeat watcher. Every `heartbeat_interval_secs`, scan the
/// registry and mark any tunnel stale whose last heartbeat is older than
/// `heartbeat_missed_threshold * heartbeat_interval_secs`.
pub async fn run_heartbeat_watcher(state: std::sync::Arc<AppState>) {
    let interval_secs = state.config.tunnel.heartbeat_interval_secs;
    let miss_threshold = state.config.tunnel.heartbeat_missed_threshold;
    let threshold_secs = interval_secs * miss_threshold;

    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    info!(
        "[HEARTBEAT] Starting watcher: interval={}s, threshold={} missed ({}s)",
        interval_secs, miss_threshold, threshold_secs
    );

    loop {
        ticker.tick().await;
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let stale: Vec<Uuid> = state
            .registry
            .iter()
            .filter_map(|h| {
                let last = h.last_heartbeat.load(std::sync::atomic::Ordering::Relaxed);
                if now_secs.saturating_sub(last) > threshold_secs {
                    Some(h.server_id)
                } else {
                    None
                }
            })
            .collect();

        for server_id in stale {
            warn!("[HEARTBEAT] Marking tunnel stale: server_id={}", server_id);
            state.registry.mark_stale(&server_id);
            crate::metrics::ACTIVE_TUNNELS.dec();
            crate::metrics::TUNNEL_EVENTS_TOTAL
                .with_label_values(&["stale"])
                .inc();
            if let Err(e) = state
                .backend
                .report_tunnel_event(server_id, "stale", "missed_heartbeats")
                .await
            {
                error!("[HEARTBEAT] Failed to report stale event: {}", e);
            }
        }
    }
}
