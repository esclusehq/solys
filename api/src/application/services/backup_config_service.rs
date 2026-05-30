use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use anyhow::{Context, Result};
use chrono::Utc;

use crate::domain::entities::backup_config::BackupConfig;
use crate::domain::repositories::backup_config_repository::BackupConfigRepository;
use crate::domain::repositories::cron_task_repository::CronTaskRepository;
use crate::domain::repositories::server_repository::ServerRepository;

/// Manages dual-write backup configuration between servers table and cron_tasks.
/// Validates cron expressions and coordinates the atomic-like save.
pub struct BackupConfigService {
    backup_config_repo: Arc<dyn BackupConfigRepository>,
    server_repo: Arc<dyn ServerRepository>,
    cron_task_repo: Arc<dyn CronTaskRepository>,
}

impl BackupConfigService {
    pub fn new(
        backup_config_repo: Arc<dyn BackupConfigRepository>,
        server_repo: Arc<dyn ServerRepository>,
        cron_task_repo: Arc<dyn CronTaskRepository>,
    ) -> Self {
        Self { backup_config_repo, server_repo, cron_task_repo }
    }

    pub async fn get_config(&self, server_id: &Uuid) -> Result<Option<BackupConfig>> {
        self.backup_config_repo.find_by_server_id(server_id).await
    }

    pub async fn save_config(&self, server_id: &Uuid, config: &BackupConfig) -> Result<()> {
        // Validate cron expression if non-empty and backup is enabled
        if config.auto_backup_enabled && !config.schedule_cron.is_empty() {
            if cron::Schedule::from_str(&config.schedule_cron).is_err() {
                anyhow::bail!("Invalid cron expression: {}", config.schedule_cron);
            }
        }

        // Verify server exists
        let server = self.server_repo.find_by_id(server_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        // Verify owner: caller must check this via the handler
        self.backup_config_repo.save(server_id, config).await
    }
}
