//! Retention pruning task — decoupled from backup jobs (D-15)
//!
//! Runs on a separate schedule (every 15 min) and evaluates combined
//! label-based + count-based retention rules across all servers.
//! Deletes eligible backups via the API's existing delete endpoint
//! which handles storage (S3/local) + backup_history record cleanup.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Background loop: runs every 15 min evaluating retention rules
/// across all servers. Decoupled from backup jobs per D-15.
pub async fn run_prune_loop(pool: PgPool, api_base_url: String) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(900));
    interval.tick().await;
    loop {
        interval.tick().await;
        if let Err(e) = evaluate_and_prune(&pool, &api_base_url).await {
            tracing::error!("Prune task error: {}", e);
        }
    }
}

struct BackupInfo {
    id: Uuid,
    server_id: Uuid,
    created_at: DateTime<Utc>,
}

/// Evaluate retention rules for all servers and prune eligible backups.
/// Uses combined label-based + count-based algorithm per D-07, D-16.
async fn evaluate_and_prune(
    pool: &PgPool,
    api_base_url: &str,
) -> anyhow::Result<()> {
    // Find all servers with retention config
    let servers = sqlx::query(
        r#"
        SELECT id, max_retained_backups, retention_rules
        FROM servers
        WHERE max_retained_backups IS NOT NULL
           OR (retention_rules IS NOT NULL AND retention_rules != '{}'::jsonb)
        "#
    )
    .fetch_all(pool)
    .await?;

    if servers.is_empty() {
        return Ok(());
    }

    let client = reqwest::Client::new();

    for server in &servers {
        let server_id: Uuid = server.try_get("id")?;
        let max_retained: Option<i32> = server.try_get("max_retained_backups")?;
        let retention_rules_json: Option<Value> = server.try_get("retention_rules")?;

        // Fetch completed backups for this server (newest first)
        let backups: Vec<BackupInfo> = sqlx::query_as::<_, (Uuid, Uuid, DateTime<Utc>)>(
            r#"
            SELECT id, server_id, created_at
            FROM backup_history
            WHERE server_id = $1 AND status = 'completed'
            ORDER BY created_at DESC
            "#
        )
        .bind(server_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(id, sid, created_at)| BackupInfo { id, server_id: sid, created_at })
        .collect();

        if backups.is_empty() {
            continue;
        }

        let mut to_delete: HashSet<Uuid> = HashSet::new();

        // Step 1: Evaluate label-based retention rules (D-07, D-16)
        if let Some(ref rules) = retention_rules_json {
            let daily = rules.get("daily").and_then(|v| v.as_i64()).unwrap_or(7) as usize;
            let weekly = rules.get("weekly").and_then(|v| v.as_i64()).unwrap_or(4) as usize;
            let monthly = rules.get("monthly").and_then(|v| v.as_i64()).unwrap_or(3) as usize;

            // Daily: group by calendar day, keep at most `daily` per day
            let mut day_groups: HashMap<String, Vec<&BackupInfo>> = HashMap::new();
            for b in &backups {
                day_groups.entry(b.created_at.format("%Y-%m-%d").to_string())
                    .or_default()
                    .push(b);
            }
            for group in day_groups.values_mut() {
                group.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                if group.len() > daily {
                    for b in group.iter().skip(daily) {
                        to_delete.insert(b.id);
                    }
                }
            }

            // Weekly: group by ISO week, keep at most `weekly` per week
            let mut week_groups: HashMap<String, Vec<&BackupInfo>> = HashMap::new();
            for b in &backups {
                week_groups.entry(b.created_at.format("%G-W%V").to_string())
                    .or_default()
                    .push(b);
            }
            for group in week_groups.values_mut() {
                group.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                if group.len() > weekly {
                    for b in group.iter().skip(weekly) {
                        to_delete.insert(b.id);
                    }
                }
            }

            // Monthly: group by YYYY-MM, keep at most `monthly` per month
            let mut month_groups: HashMap<String, Vec<&BackupInfo>> = HashMap::new();
            for b in &backups {
                month_groups.entry(b.created_at.format("%Y-%m").to_string())
                    .or_default()
                    .push(b);
            }
            for group in month_groups.values_mut() {
                group.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                if group.len() > monthly {
                    for b in group.iter().skip(monthly) {
                        to_delete.insert(b.id);
                    }
                }
            }
        }

        // Step 2: Evaluate count-based retention (max_retained_backups)
        if let Some(max_count) = max_retained {
            let retained: Vec<&BackupInfo> = backups.iter()
                .filter(|b| !to_delete.contains(&b.id))
                .collect();

            if retained.len() > max_count as usize {
                let excess = retained.len() - max_count as usize;
                for b in retained.iter().rev().take(excess) {
                    to_delete.insert(b.id);
                }
            }
        }

        // Step 3: Delete flagged backups via API (handles storage + DB cleanup)
        for backup_id in &to_delete {
            let url = format!(
                "{}/api/servers/{}/backups/{}",
                api_base_url.trim_end_matches('/'),
                server_id,
                backup_id
            );

            match client.delete(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!(
                        "Pruned backup {} for server {} (retention policy)",
                        backup_id, server_id
                    );
                }
                Ok(resp) => {
                    tracing::warn!(
                        "Prune API call returned {} for backup {} server {}",
                        resp.status(), backup_id, server_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to prune backup {} for server {}: {}",
                        backup_id, server_id, e
                    );
                }
            }
        }
    }

    Ok(())
}
