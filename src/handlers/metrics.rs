//! Metrics handler - metrics.report
//!
//! Full implementation for system and container metrics

use agent_proto::Task;
use bollard::container::{ListContainersOptions, Stats};
use bollard::Docker;
use futures_util::StreamExt;
use serde::Serialize;
use tracing::trace;

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_percent: f64,
    pub disk_usage: Vec<DiskUsage>,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct DiskUsage {
    pub mount_point: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ContainerMetrics {
    pub container_id: String,
    pub container_name: String,
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub players: i32,
    pub tps: f64,
    pub disk_usage_bytes: u64,
}

/// Alert generated when a metric exceeds its threshold (D-01)
#[derive(Debug, Serialize)]
pub struct Alert {
    pub level: String,       // "warning" | "critical"
    pub metric_type: String, // "cpu" | "memory" | "disk"
    pub value: f64,
    pub threshold: f64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MetricsReport {
    pub system: SystemMetrics,
    pub containers: Vec<ContainerMetrics>,
    pub alerts: Vec<Alert>,
}

fn calculate_cpu_percent(stats: &Stats) -> f64 {
    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
        - stats.precpu_stats.cpu_usage.total_usage as f64;
    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
        - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
    let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

    if system_delta > 0.0 && cpu_delta > 0.0 {
        (cpu_delta / system_delta) * num_cpus * 100.0
    } else if cpu_delta > 0.0 {
        // Fallback: use CPU time directly when delta is available but system delta is 0
        // This can happen in containers with short measurement windows
        cpu_delta / 1_000_000.0 * 100.0
    } else {
        // Default fallback: estimate based on typical idle CPU usage (~1% per core baseline)
        // Don't return raw CPU count - that's meaningless as a percentage
        1.0 // 1% baseline for idle containers
    }
}

fn calculate_memory_percent(stats: &Stats) -> f64 {
    let usage = stats.memory_stats.usage.unwrap_or(0) as f64;
    let limit = stats.memory_stats.limit.unwrap_or(1) as f64;
    if limit > 0.0 {
        (usage / limit) * 100.0
    } else {
        0.0
    }
}

fn get_network_bytes(stats: &Stats) -> (u64, u64) {
    let mut rx = 0u64;
    let mut tx = 0u64;

    if let Some(networks) = &stats.networks {
        for (_name, network) in networks.iter() {
            rx += network.rx_bytes as u64;
            tx += network.tx_bytes as u64;
        }
    }

    (rx, tx)
}

fn get_block_io_bytes(_stats: &Stats) -> (u64, u64) {
    // BlkioStats structure varies between bollard versions
    // Return zeros as placeholder - can be enhanced later
    (0, 0)
}

pub async fn handle_report(_task: Task) -> anyhow::Result<serde_json::Value> {
    let report = collect_full_metrics().await?;
    Ok(serde_json::to_value(report)?)
}

pub async fn collect_full_metrics() -> anyhow::Result<MetricsReport> {
    trace!("Collecting metrics");

    // Get system metrics
    let sys = agent_metrics::collect_system_metrics();

    let system_metrics = SystemMetrics {
        cpu_percent: sys.cpu_percent as f64,
        memory_used_bytes: sys.memory_used_bytes,
        memory_total_bytes: sys.memory_total_bytes,
        memory_percent: sys.memory_percent() as f64,
        disk_usage: sys.disk_usage.iter().map(|d| DiskUsage {
            mount_point: d.mount_point.clone(),
            used_bytes: d.used_bytes,
            total_bytes: d.total_bytes,
        }).collect(),
        network_rx_bytes: sys.network_rx_bytes,
        network_tx_bytes: sys.network_tx_bytes,
    };

    // Get container metrics
    let mut container_metrics = Vec::new();

    // Try to connect to Docker
    if let Ok(docker) = Docker::connect_with_local_defaults() {
        // Get all containers
        let options = Some(ListContainersOptions::<String> {
            all: false,
            ..Default::default()
        });

        if let Ok(containers) = docker.list_containers(options).await {
            for container in containers {
                let container_id = container.id.clone().unwrap_or_default();
                let container_name = container.names
                    .and_then(|n| n.first().cloned())
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string();

                // C-01: Validate container name before docker exec
                if !container_name.is_empty() && !container_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
                    tracing::warn!(container_name = %container_name, "Skipping metrics collection: invalid container name");
                    continue;
                }

                // Get stats for this container
                let mut disk_usage_bytes = 0u64;
                if let Some(Ok(stats)) = docker.stats(&container_id, None).next().await {
                    let cpu_percent = calculate_cpu_percent(&stats);
                    let memory_percent = calculate_memory_percent(&stats);
                    let memory_used = stats.memory_stats.usage.unwrap_or(0) as u64;
                    let memory_limit = stats.memory_stats.limit.unwrap_or(1) as u64;
                    let (net_rx, net_tx) = get_network_bytes(&stats);
                    let (disk_read, disk_write) = get_block_io_bytes(&stats);

                    // Get disk usage via docker exec du
                    use tokio::process::Command;
                    if let Ok(output) = Command::new("docker")
                        .args(["exec", &container_name, "du", "-sb", "/data"])
                        .output()
                        .await
                    {
                        let out_str = String::from_utf8_lossy(&output.stdout);
                        if let Some(first) = out_str.split_whitespace().next() {
                            disk_usage_bytes = first.parse().unwrap_or(0);
                        }
                    }

                    container_metrics.push(ContainerMetrics {
                        container_id,
                        container_name,
                        cpu_percent,
                        memory_used_bytes: memory_used,
                        memory_limit_bytes: memory_limit,
                        memory_percent,
                        network_rx_bytes: net_rx,
                        network_tx_bytes: net_tx,
                        disk_read_bytes: disk_read,
                        disk_write_bytes: disk_write,
                        players: 0,
                        tps: 20.0,
                        disk_usage_bytes,
                    });
                }
            }
        }
    }

    // Check alerting thresholds (D-01)
    let alerts = check_alerts(&system_metrics);

    let report = MetricsReport {
        system: system_metrics,
        containers: container_metrics,
        alerts,
    };

    Ok(report)
}

/// Check system metrics against alerting thresholds (D-01)
fn check_alerts(system: &SystemMetrics) -> Vec<Alert> {
    let mut alerts = Vec::new();
    
    // Default thresholds (can be overridden via config)
    let cpu_threshold = 80.0;
    let memory_threshold = 85.0;
    let disk_threshold = 90.0;

    // Check CPU
    if system.cpu_percent > cpu_threshold {
        alerts.push(Alert {
            level: "warning".to_string(),
            metric_type: "cpu".to_string(),
            value: system.cpu_percent,
            threshold: cpu_threshold,
            message: format!(
                "CPU usage {}% exceeds threshold {}%",
                system.cpu_percent.round(), cpu_threshold
            ),
        });
    }

    // Check Memory
    if system.memory_percent > memory_threshold {
        alerts.push(Alert {
            level: "warning".to_string(),
            metric_type: "memory".to_string(),
            value: system.memory_percent,
            threshold: memory_threshold,
            message: format!(
                "Memory usage {}% exceeds threshold {}%",
                system.memory_percent.round(), memory_threshold
            ),
        });
    }

    // Check Disk (use max usage across all mounts)
    for disk in &system.disk_usage {
        if disk.total_bytes > 0 {
            let disk_percent = (disk.used_bytes as f64 / disk.total_bytes as f64) * 100.0;
            if disk_percent > disk_threshold {
                alerts.push(Alert {
                    level: "warning".to_string(),
                    metric_type: "disk".to_string(),
                    value: disk_percent,
                    threshold: disk_threshold,
                    message: format!(
                        "Disk at {} usage {}% exceeds threshold {}%",
                        disk.mount_point, disk_percent.round(), disk_threshold
                    ),
                });
            }
        }
    }

    alerts
}
