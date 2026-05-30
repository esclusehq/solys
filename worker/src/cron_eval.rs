//! Cron evaluation loop — polls cron_tasks table for due backup tasks
//! and dispatches them to the Redis job queue.
//!
//! Replaces the API-side BackupScheduler (D-02). Runs as a background
//! task in the Worker service alongside the job processor.

use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

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
    // Per D-04: only backup task type is automated
    let rows = sqlx::query(
        r#"
        SELECT id, server_id, user_id, task_type, schedule_cron, command, enabled,
               last_run, next_run, created_at, updated_at
        FROM cron_tasks
        WHERE enabled = true
          AND task_type = 'backup'
          AND next_run <= NOW()
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

        // Enqueue backup_server job via Redis priority queue (normal priority)
        let job_id = Uuid::new_v4();
        let job_payload = json!({
            "cron_task_id": cron_task_id,
            "server_id": server_id,
            "user_id": user_id,
        });

        let job_key = format!("job:{}", job_id);
        let queue_key = "queue:jobs:normal";

        // HSET job data
        redis::cmd("HSET")
            .arg(&job_key)
            .arg("data")
            .arg(serde_json::to_string(&serde_json::json!({
                "job_id": job_id,
                "job_type": "backup_server",
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
            "Dispatched backup_server job: cron_task={} server={} job={}",
            cron_task_id, server_id, job_id
        );
    }

    Ok(())
}
