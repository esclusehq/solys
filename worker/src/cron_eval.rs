//! Cron evaluation loop — polls cron_tasks table for due backup tasks
//! and dispatches them to the Redis job queue.
//!
//! Replaces the API-side BackupScheduler (D-02). Runs as a background
//! task in the Worker service alongside the job processor.

use chrono::Utc;
use chrono_tz::Tz;
use serde_json::json;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

/// Check if a cron expression is due now in the given timezone (D-03).
/// Uses `schedule.upcoming(tz)` to find the next occurrence and checks
/// if it falls within the current 60-second window (matching the 30s poll interval).
fn is_cron_due_in_timezone(schedule_cron: &str, timezone_name: &str) -> anyhow::Result<bool> {
    let tz: Tz = timezone_name
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid timezone '{}': {}", timezone_name, e))?;

    let schedule = cron::Schedule::from_str(schedule_cron)
        .map_err(|e| anyhow::anyhow!("Invalid cron '{}': {}", schedule_cron, e))?;

    let now_utc = chrono::Utc::now();
    let now_tz = now_utc.with_timezone(&tz);

    match schedule.upcoming(tz).next() {
        Some(next) => {
            let diff = next - now_tz;
            // Due if within the next 60 seconds (covers the 30s poll interval with 1m window)
            Ok(diff.num_seconds() >= 0 && diff.num_seconds() <= 60)
        }
        None => Ok(false),
    }
}

/// Background loop: polls every 30s for due cron_tasks and dispatches
/// backup_server jobs to Redis priority queue.
pub async fn run_cron_evaluation_loop(
    pool: PgPool,
    redis: redis::aio::MultiplexedConnection,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = evaluate_and_dispatch(&pool, &redis).await {
            tracing::error!("Cron evaluation error: {}", e);
        }
    }
}

async fn evaluate_and_dispatch(
    pool: &PgPool,
    redis: &redis::aio::MultiplexedConnection,
) -> anyhow::Result<()> {
    let rows = sqlx::query(
        r#"
        SELECT id, server_id, user_id, task_type, schedule_cron, timezone, command, enabled,
               run_once, last_run, next_run, created_at, updated_at
        FROM cron_tasks
        WHERE enabled = true
          AND next_run <= NOW() + INTERVAL '30 seconds'
        ORDER BY next_run ASC
        LIMIT 50
        "#
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let cron_task_id: Uuid = row.try_get("id")?;
        let server_id: Uuid = row.try_get("server_id")?;
        let user_id: Uuid = row.try_get("user_id")?;
        let task_type: String = row.try_get("task_type")?;
        let timezone: String = row.try_get::<Option<String>, _>("timezone")?
            .unwrap_or_else(|| "UTC".to_string());
        let schedule_cron: String = row.try_get("schedule_cron")?;

        // Timezone-aware cron due check (D-03)
        match is_cron_due_in_timezone(&schedule_cron, &timezone) {
            Ok(true) => { /* proceed — cron is due in this timezone */ }
            Ok(false) => continue, // Not due in this timezone yet
            Err(e) => {
                tracing::warn!("Timezone eval error for task {}: {}", cron_task_id, e);
                continue;
            }
        }

        // Map task_type to job_type (D-02)
        let job_type = match task_type.as_str() {
            "backup" => "backup_server",
            "start" => "scheduled_start",
            "stop" => "scheduled_stop",
            "restart" => "scheduled_restart",
            "sleep" => "scheduled_sleep",
            _ => {
                tracing::warn!("Unknown task_type: {} for task {}", task_type, cron_task_id);
                continue;
            }
        };

        // Enqueue job via Redis priority queue (normal priority)
        let job_id = Uuid::new_v4();
        let job_payload = json!({
            "cron_task_id": cron_task_id,
            "server_id": server_id,
            "user_id": user_id,
            "task_type": task_type,
            "timezone": timezone,
        });

        let job_key = format!("job:{}", job_id);
        let queue_key = "queue:jobs:normal";

        // HSET job data
        redis::cmd("HSET")
            .arg(&job_key)
            .arg("data")
            .arg(serde_json::to_string(&serde_json::json!({
                "job_id": job_id,
                "job_type": job_type,
                "payload": job_payload,
                "user_id": user_id,
                "priority": 0,
                "created_at": Utc::now().timestamp(),
            }))?)
            .query_async::<_, ()>(redis)
            .await?;

        // ZADD to normal priority queue
        redis::cmd("ZADD")
            .arg(queue_key)
            .arg(Utc::now().timestamp() as f64)
            .arg(job_id.to_string())
            .query_async::<_, ()>(redis)
            .await?;

        // Update last_run on cron_task
        sqlx::query(
            "UPDATE cron_tasks SET last_run = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(cron_task_id)
        .execute(pool)
        .await?;

        tracing::info!(
            "Dispatched {} job: cron_task={} server={} job={} timezone={}",
            job_type, cron_task_id, server_id, job_id, timezone
        );
    }

    Ok(())
}
