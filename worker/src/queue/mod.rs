use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
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
    worker_id: String,
    concurrency: u32,
}

impl JobProcessor {
    pub fn new(redis: redis::aio::MultiplexedConnection) -> Self {
        let worker_id = std::env::var("WORKER_ID").unwrap_or_else(|_| "worker-01".to_string());
        let concurrency = std::env::var("WORKER_CONCURRENCY")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .unwrap_or(5);

        Self {
            redis,
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
        tracing::info!("Processing backup_server job: {}", job.job_id);
        Ok(())
    }
}

pub mod queue {
    pub use super::JobProcessor;
}
