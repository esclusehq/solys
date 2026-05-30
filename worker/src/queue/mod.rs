use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;

const QUEUE_PREFIX: &str = "queue:jobs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub user_id: Uuid,
    pub priority: i32,
    pub created_at: i64,
}

pub struct JobProcessor {
    redis: redis::aio::MultiplexedConnection,
    pool: PgPool,
    worker_id: String,
    concurrency: u32,
}

impl JobProcessor {
    pub fn new(redis: redis::aio::MultiplexedConnection, pool: PgPool) -> Self {
        let worker_id = std::env::var("WORKER_ID").unwrap_or_else(|_| "worker-01".to_string());
        let concurrency = std::env::var("WORKER_CONCURRENCY")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .unwrap_or(5);

        Self {
            redis,
            pool,
            worker_id,
            concurrency,
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(job) = self.dequeue().await {
                self.process_job(job).await;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        }
    }

    async fn dequeue(&mut self) -> Option<Job> {
        let priorities = ["high", "normal", "low"];
        
        for priority in priorities {
            let queue_key = format!("{}:{}", QUEUE_PREFIX, priority);
            let result: Option<(String, f64)> = self.redis.zpopmin(&queue_key, 1).await.ok()?;

            if let Some((job_id_str, _)) = result {
                let job_key = format!("job:{}", job_id_str);
                let job_data: Option<String> = self.redis.hget(&job_key, "data").await.ok()?;

                if let Some(data) = job_data {
                    if let Ok(job) = serde_json::from_str::<Job>(&data) {
                        return Some(job);
                    }
                }
            }
        }

        None
    }

    async fn process_job(&mut self, job: Job) {
        let job_id = job.job_id.clone();
        let job_type = job.job_type.clone();
        
        let result = match job_type.as_str() {
            "create_server" => self.process_create_server(job).await,
            "delete_server" => self.process_delete_server(job).await,
            "start_server" => self.process_start_server(job).await,
            "stop_server" => self.process_stop_server(job).await,
            "backup_server" => self.process_backup_server(job).await,
            _ => {
                tracing::warn!("Unknown job type: {}", job_type);
                Ok(())
            }
        };

        if let Err(e) = result {
            tracing::error!("Job {} failed: {}", job_id, e);
        }
    }

    async fn process_create_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Processing create_server job: {}", job.job_id);
        Ok(())
    }

    async fn process_delete_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Processing delete_server job: {}", job.job_id);
        Ok(())
    }

    async fn process_start_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Processing start_server job: {}", job.job_id);
        Ok(())
    }

    async fn process_stop_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Processing stop_server job: {}", job.job_id);
        Ok(())
    }

    async fn process_backup_server(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
        let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

        // 1. Check for active backup (Pitfall 4 prevention)
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(1) FROM backup_history WHERE server_id = $1 AND status = 'in_progress'"
        )
        .bind(server_id)
        .fetch_one(&self.pool)
        .await?;

        if active > 0 {
            tracing::warn!("Backup already in progress for server {}, skipping", server_id);
            return Ok(());
        }

        // 2. Get server details (node_id, container_name, backup config)
        let server_row = sqlx::query(
            r#"
            SELECT s.id, s.node_id, s.name AS server_name, s.backup_provider,
                   s.max_retained_backups, s.retention_rules,
                   c.container_name
            FROM servers s
            LEFT JOIN containers c ON c.server_id = s.id
            WHERE s.id = $1
            "#
        )
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or("Server not found")?;

        let node_id: Uuid = server_row.try_get("node_id")?;
        let server_name: String = server_row.try_get("server_name")?;
        let container_name: Option<String> = server_row.try_get("container_name")?;
        let backup_provider: String = server_row.try_get("backup_provider")?;

        // 3. Create backup_history record (in_progress)
        let backup_id = Uuid::new_v4();
        let file_name = format!("backup_{}_{}.tar.zst",
            server_name.replace(' ', "_"),
            Utc::now().format("%Y%m%dT%H%M%S")
        );

        sqlx::query(
            r#"
            INSERT INTO backup_history (id, server_id, file_name, provider, status, created_at)
            VALUES ($1, $2, $3, $4, 'in_progress', NOW())
            "#
        )
        .bind(backup_id)
        .bind(server_id)
        .bind(&file_name)
        .bind(&backup_provider)
        .execute(&self.pool)
        .await?;

        // 4. Send backup.start command to agent via API proxy
        let api_base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://api:3000".to_string());
        let api_url = format!("{}/api/v1/nodes/{}/commands", api_base_url, node_id);

        let body = serde_json::json!({
            "command": "backup.start",
            "server_id": server_id,
            "params": {
                "container_name": container_name,
                "backup_id": backup_id,
                "file_name": file_name,
                "provider": backup_provider,
            }
        });

        let client = reqwest::Client::new();
        let response = client.post(&api_url)
            .json(&body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                tracing::info!(
                    "Backup dispatched: server={} backup={} node={} status={}",
                    server_id, backup_id, node_id, resp.status()
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to dispatch backup.start for server {}: {}",
                    server_id, e
                );
            }
        }

        Ok(())
    }
}

pub mod queue {
    pub use super::JobProcessor;
}
