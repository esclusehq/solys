use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub user_id: Uuid,
    pub name: String,
    pub game: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password_auth: String,
    pub executor_type: String,
    pub environment: Option<String>,
    pub server_path: Option<String>,
    pub start_command: Option<String>,
    pub stop_command: Option<String>,
    pub container_name: Option<String>,
    pub public_host: Option<String>,
    pub mc_version: Option<String>,
    pub mc_loader: Option<String>,
    pub auto_pause: Option<bool>,
    pub ram_allocation: Option<String>,
    pub discord_webhook_url: Option<String>,

    pub auto_restart: Option<bool>,
    pub enable_tailscale: Option<bool>,
    pub tailscale_auth_key: Option<String>,
    pub custom_container_name: Option<String>,
    pub ip_binding: Option<String>,
    pub template: Option<String>,
    pub network_name: Option<String>,

    // Sleep/Wake & Auto-Restart Backoff (Phase 56)
    pub auto_wake: Option<bool>,
    pub sleep_timeout_minutes: Option<i32>,
    pub max_restart_attempts: Option<i32>,
    pub restart_cooldown_seconds: Option<i32>,

    pub node_id: Option<String>,

    // New simplified fields for UI
    #[serde(default)]
    pub game_type: Option<String>,
    #[serde(default)]
    pub minecraft_version: Option<String>,
    #[serde(default)]
    pub ram_mb: Option<i32>,
    #[serde(default)]
    pub max_players: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password_auth: Option<String>,
    pub executor_type: Option<String>,
    pub environment: Option<String>,
    pub server_path: Option<String>,
    pub start_command: Option<String>,
    pub stop_command: Option<String>,
    pub container_name: Option<String>,
    pub public_host: Option<String>,
    pub mc_version: Option<String>,
    pub mc_loader: Option<String>,
    pub auto_pause: Option<bool>,
    pub ram_allocation: Option<String>,
    pub discord_webhook_url: Option<String>,
    pub auto_backup_enabled: Option<bool>,
    pub backup_cron: Option<String>,
    pub backup_provider: Option<String>,
    pub backup_path: Option<String>,
    pub max_retained_backups: Option<i32>,

    // New fields for enhanced features
    pub auto_restart: Option<bool>,
    pub enable_tailscale: Option<bool>,
    pub tailscale_auth_key: Option<String>,
    pub custom_container_name: Option<String>,
    pub ip_binding: Option<String>,
    pub template: Option<String>,
    pub network_name: Option<String>,

    // Sleep/Wake & Auto-Restart Backoff (Phase 56)
    pub auto_wake: Option<bool>,
    pub sleep_timeout_minutes: Option<i32>,
    pub max_restart_attempts: Option<i32>,
    pub restart_cooldown_seconds: Option<i32>,

    // Git remote configuration
    pub git_remote_url: Option<String>,
    pub git_remote_username: Option<String>,
    pub git_remote_token: Option<String>,

    // Node assignment
    pub node_id: Option<String>,
}

impl Default for UpdateServerRequest {
    fn default() -> Self {
        Self {
            name: None,
            host: None,
            port: None,
            username: None,
            password_auth: None,
            executor_type: None,
            environment: None,
            server_path: None,
            start_command: None,
            stop_command: None,
            container_name: None,
            public_host: None,
            mc_version: None,
            mc_loader: None,
            auto_pause: None,
            ram_allocation: None,
            discord_webhook_url: None,
            auto_backup_enabled: None,
            backup_cron: None,
            backup_provider: None,
            backup_path: None,
            max_retained_backups: None,
            auto_restart: None,
            enable_tailscale: None,
            tailscale_auth_key: None,
            custom_container_name: None,
            ip_binding: None,
            template: None,
            network_name: None,
            auto_wake: None,
            sleep_timeout_minutes: None,
            max_restart_attempts: None,
            restart_cooldown_seconds: None,
            git_remote_url: None,
            git_remote_username: None,
            git_remote_token: None,
            node_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServerResponse {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub executor_type: String,
    pub environment: String,
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
    pub max_restart_attempts: i32,
    pub restart_cooldown_seconds: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Node assignment
    pub node_id: Option<String>,
}
