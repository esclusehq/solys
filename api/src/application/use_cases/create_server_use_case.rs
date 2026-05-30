use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::domain::{
    entities::server::Server,
    repositories::server_repository::ServerRepository,
};
use crate::application::dto::server_dtos::CreateServerRequest;

pub struct CreateServerUseCase<R>
where
    R: ServerRepository + ?Sized,
{
    repository: Arc<R>,
}

impl<R> CreateServerUseCase<R>
where
    R: ServerRepository + ?Sized,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, req: CreateServerRequest) -> Result<Server> {
        let now = Utc::now();
        let new_id = Uuid::new_v4();
        let server = Server {
            id: new_id,
            user_id: req.user_id,
            name: req.name.clone(),
            game: req.game.clone(),
            host: req.host.clone(),
            port: req.port,
            username: req.username.clone(),
            password_auth: req.password_auth.clone(),
            executor_type: req.executor_type.clone(),
            environment: req.environment.clone().unwrap_or_else(|| "production".to_string()),
            server_path: req.server_path.clone().or(Some("~/server".to_string())),
            start_command: req.start_command.clone().or(Some("./start.sh".to_string())),
            stop_command: req.stop_command.clone().or(Some("pkill -f server".to_string())),
            container_name: Some(format!("devnode-{}", new_id)),
            public_host: req.public_host.clone(),
            mc_version: req.mc_version.clone().unwrap_or_else(|| "LATEST".to_string()),
            mc_loader: req.mc_loader.clone().unwrap_or_else(|| "PAPER".to_string()),
            auto_pause: req.auto_pause.unwrap_or(false),
            ram_allocation: req.ram_allocation.clone().unwrap_or_else(|| "2G".to_string()),
            discord_webhook_url: req.discord_webhook_url.clone(),
            auto_backup_enabled: false,
            backup_cron: None,
            backup_provider: "local".to_string(),
            backup_path: None,
            max_retained_backups: 5,
            
            // New fields for enhanced features
            auto_restart: req.auto_restart.unwrap_or(false),
            restart_count: 0,
            enable_tailscale: req.enable_tailscale.unwrap_or(false),
            tailscale_auth_key: req.tailscale_auth_key.clone(),
            custom_container_name: req.custom_container_name.clone(),
            ip_binding: req.ip_binding.clone().unwrap_or_else(|| "0.0.0.0".to_string()),
            template: req.template.clone().unwrap_or_else(|| "paper".to_string()),
            network_name: req.network_name.clone().unwrap_or_else(|| "devnode-minecraft".to_string()),

            // Sleep/Wake & Auto-Restart Backoff (Phase 56)
            auto_wake: req.auto_wake.unwrap_or(false),
            sleep_timeout_minutes: req.sleep_timeout_minutes.unwrap_or(30),
            last_player_activity: None,
            max_restart_attempts: req.max_restart_attempts.unwrap_or(5),
            restart_cooldown_seconds: req.restart_cooldown_seconds.unwrap_or(300),

            // Restart Policy & Health Check (Phase 57)
            last_restart_at: None,
            last_restart_reason: None,
            health_check_timeout_seconds: 5,

            // Git remote configuration
            git_remote_url: None,
            git_remote_username: None,
            git_remote_token: None,
            
            // Node Agent
            node_id: req.node_id.as_ref().and_then(|n| Uuid::parse_str(n).ok()),
            
            status: "stopped".to_string(), // Default status
            created_at: now,
            updated_at: now,
        };

        self.repository.create(&server).await?;
        Ok(server)
    }
}
