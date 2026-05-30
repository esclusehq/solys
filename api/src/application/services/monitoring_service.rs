use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use crate::domain::{
    repositories::{server_repository::ServerRepository, metrics_repository::MetricsRepository, node_repository::NodeRepository},
    factories::ExecutorFactory,
};
use crate::infrastructure::events::event_bus::EventBus;
use crate::shared::events::ServerEvent;

use crate::application::use_cases::evaluate_alerts_use_case::EvaluateAlertsUseCase;

pub struct MonitoringService<R, M, F, N>
where
    R: ServerRepository + ?Sized,
    M: MetricsRepository + ?Sized,
    F: ExecutorFactory + ?Sized,
    N: NodeRepository + ?Sized,
{
    repository: Arc<R>,
    metrics_repository: Arc<M>,
    executor_factory: Arc<F>,
    event_bus: Arc<EventBus>,
    evaluate_alerts_use_case: Arc<EvaluateAlertsUseCase>,
    node_repository: Arc<N>,
}

impl<R, M, F, N> MonitoringService<R, M, F, N>
where
    R: ServerRepository + ?Sized + Send + Sync + 'static,
    M: MetricsRepository + ?Sized + Send + Sync + 'static,
    F: ExecutorFactory + ?Sized + Send + Sync + 'static,
    N: NodeRepository + ?Sized + Send + Sync + 'static,
{
    pub fn new(
        repository: Arc<R>,
        metrics_repository: Arc<M>,
        executor_factory: Arc<F>,
        event_bus: Arc<EventBus>,
        evaluate_alerts_use_case: Arc<EvaluateAlertsUseCase>,
        node_repository: Arc<N>,
    ) -> Self {
        Self {
            repository,
            metrics_repository,
            executor_factory,
            event_bus,
            evaluate_alerts_use_case,
            node_repository,
        }
    }

    pub async fn start(self: Arc<Self>) {
        tracing::info!("Starting Background Monitoring Service...");
        
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        // Fire first tick immediately
        interval.tick().await;
        
        tracing::info!("Monitoring service: first tick fired, starting loop");
        
        loop {
            interval.tick().await;
            tracing::debug!("Monitoring loop: checking all servers...");
            if let Err(e) = self.check_all_servers().await {
                tracing::error!("Monitoring Loop Error: {}", e);
            }
        }
    }

    /// Determine server status by delegating entirely to the executor's check_status.
    async fn determine_status(&self, server: &crate::domain::entities::server::Server) -> Result<String> {
        let executor = self.executor_factory.get_executor(server);
        executor.check_status(server).await
    }

    async fn check_all_servers(&self) -> Result<()> {
        // 1. Fetch all nodes first to check their status (D-07)
        let nodes = self.node_repository.list().await?;
        let offline_node_ids: std::collections::HashSet<uuid::Uuid> = nodes
            .iter()
            .filter(|n| n.status == "offline")
            .map(|n| n.id)
            .collect();

        // 2. Fetch all servers
        let servers = self.repository.list().await?;

        for server in servers {
            // D-07: When agent goes OFFLINE, stop monitoring servers on that node
            if let Some(node_id) = server.node_id {
                if offline_node_ids.contains(&node_id) {
                    tracing::debug!(
                        "[MONITOR] Skipping server {} on offline node {}",
                        server.name, node_id
                    );
                    continue;
                }
            }

            if !server.is_running() {
                tracing::info!("[MONITOR] Skipping {} - not running (status={})", server.name, server.status);
                continue;
            }

            if server.node_id.is_none() {
                tracing::info!("[MONITOR] Skipping {} - no node assigned", server.name);
                continue;
            }

            // 2. Get Executor (still needed for metrics)
            let executor = self.executor_factory.get_executor(&server);

            // 3. Check Status using Docker API or executor fallback
            match self.determine_status(&server).await {
                Ok(status) => {
                    tracing::info!("[MONITOR] Checking server {}: current_status={}, detected_status={}", 
                        server.name, server.status, status);
                    
                    // Skip auto-update if going from "starting" to "running"
                    // Let MC_READY handler do this explicitly
                    if server.status == "starting" && status == "running" {
                        tracing::info!("[MONITOR] Server {} transitioning starting->running, setting container_running", server.name);
                        // Set intermediate status to indicate container is running but Minecraft not ready
                        if let Err(e) = self.repository.update_status(&server.id, "container_running").await {
                            tracing::error!("[MONITOR] Failed to update status to container_running: {}", e);
                        } else {
                            tracing::info!("[MONITOR] Server {} status: starting -> container_running (waiting for MC_READY)", server.name);
                            let _ = self.event_bus.publish(ServerEvent::StatusChanged {
                                server_id: server.id,
                                status: "container_running".to_string(),
                            });
                            tracing::info!("[MONITOR] Published StatusChanged event for {}", server.name);
                        }
                    } else if status != server.status {
                        // Check for crash detection and auto-restart
                        // Only trigger if: old status was "running", new is "stopped", AND auto_restart is enabled
                        if server.status == "running" && status == "stopped" {
                            // Fetch full server to get auto_restart flag
                            match self.repository.find_by_id(&server.id).await {
                                Ok(Some(full_server)) => {
                                    if full_server.auto_restart {
                                        let max_attempts = full_server.max_restart_attempts;
                                        let current_count = full_server.restart_count;
                                        if current_count >= max_attempts {
                                            tracing::error!(
                                                "[MONITOR] Server {} crashed, max restart attempts ({}/{}) reached. Giving up.",
                                                full_server.name, current_count, max_attempts
                                            );
                                            // Could publish an alert here in future phases
                                        } else {
                                            let backoff_secs = std::cmp::min(
                                                30u32 * 2u32.pow(current_count as u32),  // 30s, 60s, 120s, 240s...
                                                full_server.restart_cooldown_seconds as u32,  // cap at max
                                            );
                                            tracing::warn!(
                                                "[MONITOR] Server {} crashed, restarting in {}s (attempt {}/{})...",
                                                full_server.name, backoff_secs, current_count + 1, max_attempts
                                            );

                                            // Spawn delayed restart to avoid blocking the monitoring loop (Pitfall 4)
                                            let repo_clone = self.repository.clone();
                                            let factory_clone = self.executor_factory.clone();
                                            let server_clone = full_server.clone();
                                            tokio::spawn(async move {
                                                tokio::time::sleep(std::time::Duration::from_secs(backoff_secs as u64)).await;
                                                let exec = factory_clone.get_executor(&server_clone);
                                                match exec.start_server(&server_clone).await {
                                                    Ok(_) => {
                                                        tracing::info!("[MONITOR] Backed-off auto-restart succeeded for {} (attempt {})", server_clone.name, current_count + 1);
                                                        let mut updated = server_clone.clone();
                                                        updated.restart_count = current_count + 1;
                                                        let _ = repo_clone.update(&updated).await;
                                                    }
                                                    Err(e) => {
                                                        tracing::error!("[MONITOR] Backed-off auto-restart failed for {}: {}", server_clone.name, e);
                                                    }
                                                }
                                            });
                                        }
                                        // Skip status update since we triggered restart (or hit limit)
                                        continue;
                                    } else {
                                        tracing::info!(
                                            "[MONITOR] Server {} crashed but auto_restart is disabled",
                                            server.name
                                        );
                                    }
                                }
                                Ok(None) => {
                                    tracing::warn!("[MONITOR] Server {} not found in repository", server.id);
                                }
                                Err(e) => {
                                    tracing::error!("[MONITOR] Failed to fetch server {}: {}", server.id, e);
                                }
                            }
                        } else {
                            tracing::info!(
                                "Server {} status changed: {} -> {}", 
                                server.name, server.status, status
                            );
                            if let Err(e) = self.repository.update_status(&server.id, &status).await {
                                tracing::error!("Failed to update status for {}: {}", server.name, e);
                            } else {
                                // Publish Status Changed Event
                                let _ = self.event_bus.publish(ServerEvent::StatusChanged {
                                    server_id: server.id,
                                    status: status.clone(),
                                });
                            }
                        }
                    }

                    // === SLEEP DETECTION (Phase 56) ===
                    // Check running servers for player inactivity
                    if status == "running" && server.auto_wake {
                        match executor.collect_metrics(&server).await {
                            Ok(metrics) => {
                                if metrics.players > 0 {
                                    // Players online — reset last_player_activity timestamp
                                    let mut updated = server.clone();
                                    updated.last_player_activity = Some(chrono::Utc::now());
                                    if let Err(e) = self.repository.update(&updated).await {
                                        tracing::warn!("[MONITOR] Failed to update last_player_activity for {}: {}", server.name, e);
                                    }
                                } else if let Some(last_activity) = server.last_player_activity {
                                    // No players — check inactivity timeout
                                    let elapsed = chrono::Utc::now() - last_activity;
                                    let timeout = chrono::Duration::minutes(server.sleep_timeout_minutes as i64);
                                    if elapsed >= timeout {
                                        tracing::warn!(
                                            "[MONITOR] Server {} idle for >{}min with 0 players, sleeping...",
                                            server.name, server.sleep_timeout_minutes
                                        );
                                        // Trigger sleep: stop server + set auto_wake
                                        let exec = self.executor_factory.get_executor(&server);
                                        match exec.stop_server(&server).await {
                                            Ok(_) => {
                                                let mut updated = server.clone();
                                                updated.status = "stopped".to_string();
                                                updated.auto_wake = true;
                                                if let Err(e) = self.repository.update(&updated).await {
                                                    tracing::error!("[MONITOR] Failed to update sleeping server {}: {}", server.name, e);
                                                }
                                                let _ = self.event_bus.publish(ServerEvent::StatusChanged {
                                                    server_id: server.id,
                                                    status: "stopped".to_string(),
                                                });
                                                tracing::info!("[MONITOR] Server {} put to sleep due to inactivity", server.name);
                                            }
                                            Err(e) => {
                                                tracing::error!("[MONITOR] Failed to stop server {} for sleep: {}", server.name, e);
                                            }
                                        }
                                    }
                                } else {
                                    // No last_player_activity recorded yet — set it now
                                    let mut updated = server.clone();
                                    updated.last_player_activity = Some(chrono::Utc::now());
                                    let _ = self.repository.update(&updated).await;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("[MONITOR] Failed to collect metrics for sleep detection on {}: {}", server.name, e);
                            }
                        }
                    }

                    // Collect Metrics (Only if running)
                    if status == "running" {
                        // Reset restart_count after stable running (Phase 56)
                        if server.restart_count > 0 {
                            tracing::info!("[MONITOR] Server {} running stably, resetting restart_count from {} to 0", server.name, server.restart_count);
                            let mut updated = server.clone();
                            updated.restart_count = 0;
                            if let Err(e) = self.repository.update(&updated).await {
                                tracing::error!("[MONITOR] Failed to reset restart_count for {}: {}", server.name, e);
                            }
                        }
                        match executor.collect_metrics(&server).await {
                            Ok(metrics) => {
                                if let Err(e) = self.metrics_repository.insert(&metrics).await {
                                    tracing::error!("Failed to save metrics for {}: {}", server.name, e);
                                } else {
// Publish Metrics Updated Event
                                     let _ = self.event_bus.publish(ServerEvent::MetricsUpdated {
                                         server_id: server.id,
                                         cpu_usage: metrics.cpu_usage,
                                         memory_usage_mb: metrics.memory_usage_mb,
                                         disk_usage_mb: metrics.disk_usage_mb,
                                         tps: metrics.tps,
                                         players: metrics.players,
                                     });

                                    // Evaluate Alerts
                                    if let Err(e) = self.evaluate_alerts_use_case.execute(server.id, &metrics).await {
                                        tracing::error!("Failed to evaluate alerts for {}: {}", server.name, e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to collect metrics for {}: {}", server.name, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to check status for {}: {}", server.name, e);
                }
            }
        }
        Ok(())
    }
}
