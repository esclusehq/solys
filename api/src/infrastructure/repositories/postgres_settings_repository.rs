use async_trait::async_trait;
use sqlx::{Pool, Postgres, Row};
use anyhow::{Result, Context};
use crate::domain::{
    entities::settings::{RestartDefaults, S3Config},
    entities::cloudflare_settings::CloudflareConfig,
    repositories::settings_repository::SettingsRepository,
};

pub struct PostgresSettingsRepository {
    pool: Pool<Postgres>,
}

impl PostgresSettingsRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepository for PostgresSettingsRepository {
    async fn get_s3_config(&self) -> Result<S3Config> {
        let row = sqlx::query("SELECT value FROM app_settings WHERE key = 's3_config'")
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch S3 config")?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.try_get("value")?;
                let config: S3Config = serde_json::from_value(value)
                    .context("Failed to deserialize S3 config")?;
                Ok(config)
            }
            None => Ok(S3Config::default()),
        }
    }

    async fn save_s3_config(&self, config: &S3Config) -> Result<()> {
        let value = serde_json::to_value(config)
            .context("Failed to serialize S3 config")?;

        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES ('s3_config', $1, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
            "#,
        )
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Failed to save S3 config")?;

        Ok(())
    }

    async fn get_cloudflare_config(&self) -> Result<CloudflareConfig> {
        let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'cloudflare_config'")
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch Cloudflare config")?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.try_get("value")?;
                let config: CloudflareConfig = serde_json::from_value(value)
                    .context("Failed to deserialize Cloudflare config")?;
                Ok(config)
            }
            None => Ok(CloudflareConfig::default()),
        }
    }

    async fn save_cloudflare_config(&self, config: &CloudflareConfig) -> Result<()> {
        let value = serde_json::to_value(config)
            .context("Failed to serialize Cloudflare config")?;

        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES ('cloudflare_config', $1, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
            "#,
        )
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Failed to save Cloudflare config")?;

        Ok(())
    }

    async fn get_restart_defaults(&self) -> Result<RestartDefaults> {
        let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'restart_defaults'")
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch restart defaults")?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.try_get("value")?;
                let config: RestartDefaults = serde_json::from_value(value)
                    .context("Failed to deserialize restart defaults")?;
                Ok(config)
            }
            None => Ok(RestartDefaults::default()),
        }
    }

    async fn save_restart_defaults(&self, config: &RestartDefaults) -> Result<()> {
        let value = serde_json::to_value(config)
            .context("Failed to serialize restart defaults")?;

        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES ('restart_defaults', $1, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
            "#,
        )
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Failed to save restart defaults")?;

        Ok(())
    }

    async fn get_modrinth_api_key(&self) -> Result<String> {
        let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'modrinth_api_key'")
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch Modrinth API key")?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.try_get("value")?;
                let key: String = serde_json::from_value(value)
                    .context("Failed to deserialize Modrinth API key")?;
                Ok(key)
            }
            None => Ok(String::new()),
        }
    }

    async fn save_modrinth_api_key(&self, api_key: &str) -> Result<()> {
        let value = serde_json::to_value(api_key)
            .context("Failed to serialize Modrinth API key")?;

        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES ('modrinth_api_key', $1, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
            "#,
        )
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Failed to save Modrinth API key")?;

        Ok(())
    }

    async fn get_curseforge_api_key(&self) -> Result<String> {
        let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'curseforge_api_key'")
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch CurseForge API key")?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.try_get("value")?;
                let key: String = serde_json::from_value(value)
                    .context("Failed to deserialize CurseForge API key")?;
                Ok(key)
            }
            None => Ok(String::new()),
        }
    }

    async fn save_curseforge_api_key(&self, api_key: &str) -> Result<()> {
        let value = serde_json::to_value(api_key)
            .context("Failed to serialize CurseForge API key")?;

        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES ('curseforge_api_key', $1, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
            "#,
        )
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Failed to save CurseForge API key")?;

        Ok(())
    }
}
