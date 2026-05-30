use async_trait::async_trait;
use anyhow::Result;
use crate::domain::entities::settings::{RestartDefaults, S3Config};
use crate::domain::entities::cloudflare_settings::CloudflareConfig;

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_s3_config(&self) -> Result<S3Config>;
    async fn save_s3_config(&self, config: &S3Config) -> Result<()>;
    async fn get_cloudflare_config(&self) -> Result<CloudflareConfig>;
    async fn save_cloudflare_config(&self, config: &CloudflareConfig) -> Result<()>;
    async fn get_restart_defaults(&self) -> Result<RestartDefaults>;
    async fn save_restart_defaults(&self, config: &RestartDefaults) -> Result<()>;

    // Modrinth API key
    async fn get_modrinth_api_key(&self) -> Result<String>;
    async fn save_modrinth_api_key(&self, api_key: &str) -> Result<()>;

    // CurseForge API key
    async fn get_curseforge_api_key(&self) -> Result<String>;
    async fn save_curseforge_api_key(&self, api_key: &str) -> Result<()>;
}
