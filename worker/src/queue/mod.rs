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
            "scheduled_start" => self.process_scheduled_start(job).await,
            "scheduled_stop" => self.process_scheduled_stop(job).await,
            "scheduled_restart" => self.process_scheduled_restart(job).await,
            "scheduled_sleep" => self.process_scheduled_sleep(job).await,
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

    // ─── Scheduled Action Helpers ───

    /// Update cron_tasks last_result and last_error (D-05)
    async fn update_cron_task_result(&self, task_id: Uuid, result: &str, error: Option<&str>) -> anyhow::Result<()> {
        sqlx::query(
            r#"UPDATE cron_tasks
               SET last_result = $2, last_error = $3, updated_at = NOW()
               WHERE id = $1"#
        )
        .bind(task_id)
        .bind(result)
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Check if a cron task has run_once=true (D-06)
    async fn is_run_once(&self, task_id: Uuid) -> anyhow::Result<bool> {
        let run_once: bool = sqlx::query_scalar(
            "SELECT run_once FROM cron_tasks WHERE id = $1"
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
        Ok(run_once)
    }

    /// Disable a cron task after run_once execution (D-06)
    async fn disable_cron_task(&self, task_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("UPDATE cron_tasks SET enabled = false, updated_at = NOW() WHERE id = $1")
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ─── Scheduled Action Handlers ───

    async fn process_scheduled_start(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
        let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

        // Check server status — skip if already running or starting
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM servers WHERE id = $1"
        )
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await?;

        let status = match status {
            Some(s) => s,
            None => {
                self.update_cron_task_result(cron_task_id, "error", Some("Server not found")).await?;
                return Ok(());
            }
        };

        if status == "running" || status == "starting" || status == "container_running" {
            tracing::warn!("Server {} already {}, skipping scheduled start", server_id, status);
            self.update_cron_task_result(cron_task_id, "skipped", Some(&format!("Already {}", status))).await?;
            return Ok(());
        }

        // Get node_id for API dispatch
        let node_id: Uuid = match sqlx::query_scalar("SELECT node_id FROM servers WHERE id = $1")
            .bind(server_id)
            .fetch_optional(&self.pool)
            .await?
        {
            Some(nid) => nid,
            None => {
                self.update_cron_task_result(cron_task_id, "error", Some("Server node not found")).await?;
                return Ok(());
            }
        };

        // Dispatch through API proxy endpoint (D-05 retry handled inside dispatch_to_agent_with_params)
        self.dispatch_to_agent(node_id, server_id, cron_task_id, "start").await?;
        Ok(())
    }

    async fn process_scheduled_stop(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
        let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

        // Check server status — skip if already stopped/stopping
        let status: Option<String> = sqlx::query_scalar("SELECT status FROM servers WHERE id = $1")
            .bind(server_id)
            .fetch_optional(&self.pool)
            .await?;

        let status = match status {
            Some(s) => s,
            None => {
                self.update_cron_task_result(cron_task_id, "error", Some("Server not found")).await?;
                return Ok(());
            }
        };

        if status == "stopped" || status == "stopping" {
            tracing::warn!("Server {} already {}, skipping scheduled stop", server_id, status);
            self.update_cron_task_result(cron_task_id, "skipped", Some(&format!("Already {}", status))).await?;
            return Ok(());
        }

        let node_id: Uuid = sqlx::query_scalar("SELECT node_id FROM servers WHERE id = $1")
            .bind(server_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or("Server node not found")?;

        self.dispatch_to_agent(node_id, server_id, cron_task_id, "stop").await?;
        Ok(())
    }

    async fn process_scheduled_restart(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
        let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

        // D-08: Check if auto-restart is active
        let restart_count: Option<i32> = sqlx::query_scalar(
            "SELECT restart_count FROM servers WHERE id = $1"
        )
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(count) = restart_count {
            if count > 0 {
                // Auto-restart is in progress — wait 30s and re-check
                tracing::info!("Server {} has active auto-restart, waiting 30s (D-08)", server_id);
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                let still_active: i32 = sqlx::query_scalar(
                    "SELECT COALESCE(restart_count, 0) FROM servers WHERE id = $1"
                )
                .bind(server_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

                if still_active > 0 {
                    tracing::warn!("Server {} auto-restart still active after 30s, deferring scheduled restart", server_id);
                    // Don't mark as error — let the next cron tick try again
                    return Ok(());
                }
            }
        }

        let node_id: Uuid = sqlx::query_scalar("SELECT node_id FROM servers WHERE id = $1")
            .bind(server_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or("Server node not found")?;

        self.dispatch_to_agent(node_id, server_id, cron_task_id, "restart").await?;
        Ok(())
    }

    async fn process_scheduled_sleep(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_id: Uuid = serde_json::from_value(job.payload["server_id"].clone())?;
        let cron_task_id: Uuid = serde_json::from_value(job.payload["cron_task_id"].clone())?;

        // D-07: Check if server is already in sleep state (status='stopped' + auto_wake=true)
        let server_row = sqlx::query(
            "SELECT status, auto_wake, node_id FROM servers WHERE id = $1"
        )
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await?;

        let (status, auto_wake, node_id) = match server_row {
            Some(row) => {
                let s: String = row.try_get("status")?;
                let aw: bool = row.try_get("auto_wake")?;
                let nid: Uuid = row.try_get("node_id")?;
                (s, aw, nid)
            }
            None => {
                self.update_cron_task_result(cron_task_id, "error", Some("Server not found")).await?;
                return Ok(());
            }
        };

        // D-07: Already in sleep state → skip
        if status == "stopped" && auto_wake {
            tracing::info!("Server {} already in sleep state, skipping scheduled sleep (D-07)", server_id);
            self.update_cron_task_result(cron_task_id, "skipped", Some("Already sleeping")).await?;
            return Ok(());
        }

        // Not running → skip (can't sleep a stopped/starting server)
        if status != "running" && status != "container_running" {
            tracing::warn!("Server {} not running ({}), skipping scheduled sleep", server_id, status);
            self.update_cron_task_result(cron_task_id, "skipped", Some(&format!("Not running ({})", status))).await?;
            return Ok(());
        }

        // Send "stop" command — the API handler sets auto_wake=true for sleep semantics
        self.dispatch_to_agent_with_params(node_id, server_id, cron_task_id, "stop", serde_json::json!({"sleep": true})).await?;

        // Set auto_wake = true on the server for Phase 56 sleep semantics
        sqlx::query("UPDATE servers SET auto_wake = true, updated_at = NOW() WHERE id = $1")
            .bind(server_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ─── Dispatch Helpers (D-05 Retry) ───

    /// Make a single HTTP dispatch call to the API proxy endpoint.
    /// Returns Ok(()) on success, Err(String) with error message on failure.
    async fn execute_dispatch(
        &self, node_id: Uuid, server_id: Uuid, command: &str, params: serde_json::Value
    ) -> Result<(), String> {
        let api_base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://api:3000".to_string());
        let api_url = format!("{}/api/v1/nodes/{}/dispatch", api_base_url, node_id);

        let body = serde_json::json!({
            "command": command,
            "server_id": server_id,
            "params": params,
        });

        let client = reqwest::Client::new();
        match client.post(&api_url).json(&body).send().await {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => Err(format!("API returned status {}", resp.status())),
            Err(e) => Err(format!("API request failed: {}", e)),
        }
    }

    /// Dispatch a command with D-05 retry: first attempt, on failure wait 30s then retry once.
    /// Only marks as failed if both attempts fail.
    async fn dispatch_to_agent_with_params(
        &self, node_id: Uuid, server_id: Uuid, cron_task_id: Uuid, command: &str, params: serde_json::Value
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // First attempt
        match self.execute_dispatch(node_id, server_id, command, params.clone()).await {
            Ok(()) => {
                tracing::info!("Dispatched {} to server {} via node {}", command, server_id, node_id);
                self.update_cron_task_result(cron_task_id, "success", None).await?;
                // D-06: Auto-disable if run_once
                if self.is_run_once(cron_task_id).await.unwrap_or(false) {
                    self.disable_cron_task(cron_task_id).await?;
                    tracing::info!("Auto-disabled run_once task {}", cron_task_id);
                }
                return Ok(());
            }
            Err(first_error) => {
                // D-05: First failure — log, set retrying state, wait 30s, then retry
                tracing::warn!(
                    "Scheduled task {} failed on attempt 1: {}. Retrying once after 30s...",
                    cron_task_id, first_error
                );
                self.update_cron_task_result(cron_task_id, "retrying", Some(&first_error)).await?;
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                // D-05: Second attempt (retry) — re-calls the same dispatch logic
                match self.execute_dispatch(node_id, server_id, command, params).await {
                    Ok(()) => {
                        tracing::info!("Retry succeeded for server {} task {}", server_id, cron_task_id);
                        self.update_cron_task_result(cron_task_id, "success", None).await?;
                        if self.is_run_once(cron_task_id).await.unwrap_or(false) {
                            self.disable_cron_task(cron_task_id).await?;
                        }
                        Ok(())
                    }
                    Err(retry_error) => {
                        // D-05: Both attempts failed — mark with final error status
                        tracing::error!(
                            "Scheduled task {} failed after retry: {}", cron_task_id, retry_error
                        );
                        self.update_cron_task_result(
                            cron_task_id,
                            &format!("failed: {}", retry_error),
                            Some(&retry_error),
                        ).await?;
                        // Toast notification + server event are emitted by the API dispatch endpoint (Task 3),
                        // which already sent them on the initial failure before returning to the Worker.
                        Ok(())
                    }
                }
            }
        }
    }

    /// Dispatch a command to the agent via the API proxy endpoint (no custom params)
    async fn dispatch_to_agent(
        &self, node_id: Uuid, server_id: Uuid, cron_task_id: Uuid, command: &str
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_to_agent_with_params(node_id, server_id, cron_task_id, command, serde_json::json!({})).await
    }
}

pub mod queue {
    pub use super::JobProcessor;
}
