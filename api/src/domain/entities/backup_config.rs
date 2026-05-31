use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Per-server backup configuration aggregation. Maps to
/// servers.backup_* columns and references a cron_task for
/// scheduling. Determines provider, retention policy, and
/// whether automatic backups are enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub auto_backup_enabled: bool,
    pub schedule_cron: String,
    pub backup_provider: String,
    pub max_retained_backups: i32,
    pub retention_rules: JsonValue,       // {"daily": 7, "weekly": 4, "monthly": 3}
    pub retention_mode: String,           // "count", "label", "hybrid"
    pub s3_profile_id: Option<Uuid>,
    pub cron_task_id: Option<Uuid>,       // links to cron_tasks.id
}
