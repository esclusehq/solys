use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::backup_config::BackupConfig;
use anyhow::Result;

/// Repository for backup configuration CRUD.
/// Reads from and writes to both servers table + cron_tasks table (D-03 dual-write).
#[async_trait]
pub trait BackupConfigRepository: Send + Sync {
    async fn find_by_server_id(&self, server_id: &Uuid) -> Result<Option<BackupConfig>>;
    async fn save(&self, server_id: &Uuid, config: &BackupConfig) -> Result<()>;
    async fn delete(&self, server_id: &Uuid) -> Result<()>;
}
