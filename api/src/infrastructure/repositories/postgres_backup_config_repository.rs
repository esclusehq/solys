use async_trait::async_trait;
use chrono::Utc;
use cron::Schedule;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;
use anyhow::{Context, Result};

use crate::domain::entities::backup_config::BackupConfig;
use crate::domain::repositories::backup_config_repository::BackupConfigRepository;

pub struct PostgresBackupConfigRepository {
    pool: PgPool,
}

impl PostgresBackupConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BackupConfigRepository for PostgresBackupConfigRepository {
    async fn find_by_server_id(&self, server_id: &Uuid) -> Result<Option<BackupConfig>> {
        let row = sqlx::query(
            r#"
            SELECT
                s.auto_backup_enabled,
                COALESCE(s.backup_cron, '') AS schedule_cron,
                s.backup_provider,
                s.max_retained_backups,
                COALESCE(s.retention_rules, '{"daily": 7, "weekly": 4, "monthly": 3}'::jsonb) AS retention_rules,
                COALESCE(s.retention_mode, 'hybrid') AS retention_mode,
                s.s3_profile_id,
                ct.id AS cron_task_id
            FROM servers s
            LEFT JOIN cron_tasks ct ON ct.server_id = s.id AND ct.task_type = 'backup'
            WHERE s.id = $1
            "#
        )
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch backup config")?;

        match row {
            Some(r) => Ok(Some(BackupConfig {
                auto_backup_enabled: r.try_get("auto_backup_enabled")?,
                schedule_cron: r.try_get("schedule_cron")?,
                backup_provider: r.try_get("backup_provider")?,
                max_retained_backups: r.try_get("max_retained_backups")?,
                retention_rules: r.try_get("retention_rules")?,
                retention_mode: r.try_get("retention_mode")?,
                s3_profile_id: r.try_get("s3_profile_id")?,
                cron_task_id: r.try_get("cron_task_id")?,
            })),
            None => Ok(None),
        }
    }

    async fn save(&self, server_id: &Uuid, config: &BackupConfig) -> Result<()> {
        // D-03 dual-write: update servers table + upsert cron_tasks
        // First update servers table
        sqlx::query(
            r#"
            UPDATE servers SET
                auto_backup_enabled = $1,
                backup_cron = $2,
                backup_provider = $3,
                max_retained_backups = $4,
                retention_rules = $5::jsonb,
                retention_mode = $6,
                s3_profile_id = $7,
                updated_at = NOW()
            WHERE id = $8
            "#
        )
        .bind(config.auto_backup_enabled)
        .bind(&config.schedule_cron)
        .bind(&config.backup_provider)
        .bind(config.max_retained_backups)
        .bind(&config.retention_rules.to_string())
        .bind(&config.retention_mode)
        .bind(config.s3_profile_id)
        .bind(server_id)
        .execute(&self.pool)
        .await
        .context("Failed to update server backup config")?;

        // Upsert cron_tasks row (D-03)
        // Calculate next_run based on schedule_cron
        let next_run = calculate_next_run(&config.schedule_cron);

        sqlx::query(
            r#"
            INSERT INTO cron_tasks (server_id, user_id, task_type, schedule_cron, command, enabled, next_run, created_at, updated_at)
            VALUES ($1, (SELECT user_id FROM servers WHERE id = $1), 'backup', $2, NULL, $3, $4, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                schedule_cron = EXCLUDED.schedule_cron,
                enabled = EXCLUDED.enabled,
                next_run = EXCLUDED.next_run,
                updated_at = NOW()
            WHERE cron_tasks.task_type = 'backup'
            "#
        )
        .bind(server_id)
        .bind(&config.schedule_cron)
        .bind(config.auto_backup_enabled)
        .bind(next_run)
        .execute(&self.pool)
        .await
        .context("Failed to upsert backup cron_task")?;

        Ok(())
    }

    async fn delete(&self, server_id: &Uuid) -> Result<()> {
        // Disable backup config and remove cron_task
        sqlx::query(
            "UPDATE servers SET auto_backup_enabled = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(server_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "DELETE FROM cron_tasks WHERE server_id = $1 AND task_type = 'backup'"
        )
        .bind(server_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Calculate next cron trigger time from a cron expression.
/// Returns None if the expression is invalid or empty.
fn calculate_next_run(schedule_cron: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    if schedule_cron.is_empty() {
        return None;
    }
    match Schedule::from_str(schedule_cron) {
        Ok(schedule) => schedule.after(&Utc::now()).next(),
        Err(_) => None,
    }
}
