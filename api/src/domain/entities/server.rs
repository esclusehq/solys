use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub game: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password_auth: String,
    pub executor_type: String,
    pub server_path: Option<String>,
    pub start_command: Option<String>,
    pub stop_command: Option<String>,
    pub container_name: Option<String>,
    pub public_host: Option<String>,
    pub mc_version: String,
    pub mc_loader: String,
    pub auto_pause: bool,
    pub ram_allocation: String,
    pub discord_webhook_url: Option<String>,
    pub auto_backup_enabled: bool,
    pub backup_cron: Option<String>,
    pub backup_provider: String,
    pub backup_path: Option<String>,
    pub max_retained_backups: i32,
    pub environment: String,
    pub status: String,

    // New fields for enhanced features
    pub auto_restart: bool,
    pub restart_count: i32,
    pub enable_tailscale: bool,
    pub tailscale_auth_key: Option<String>,
    pub custom_container_name: Option<String>,
    pub ip_binding: String,
    pub template: String,
    pub network_name: String,

    // Sleep/Wake & Auto-Restart Backoff (Phase 56)
    pub auto_wake: bool,
    pub sleep_timeout_minutes: i32,
    pub last_player_activity: Option<DateTime<Utc>>,
    pub max_restart_attempts: i32,
    pub restart_cooldown_seconds: i32,

    // Restart Policy & Health Check (Phase 57)
    pub last_restart_at: Option<DateTime<Utc>>,
    pub last_restart_reason: Option<String>,
    pub health_check_timeout_seconds: i32,

    // Git remote configuration
    pub git_remote_url: Option<String>,
    pub git_remote_username: Option<String>,
    pub git_remote_token: Option<String>,

    // Node Agent metadata (for agent-mode servers)
    pub node_id: Option<Uuid>,

    // Multi-tenant isolation
    pub user_id: Uuid,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Server {
    pub fn is_running(&self) -> bool {
        matches!(self.status.as_str(), "running" | "container_running")
    }

    pub fn is_stopped(&self) -> bool {
        !self.is_running()
    }
}
