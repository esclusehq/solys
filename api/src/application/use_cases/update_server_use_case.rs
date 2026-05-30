use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::domain::{
    entities::server::Server,
    repositories::server_repository::ServerRepository,
};
use crate::application::dto::server_dtos::UpdateServerRequest;

pub struct UpdateServerUseCase<R>
where
    R: ServerRepository + ?Sized,
{
    repository: Arc<R>,
}

impl<R> UpdateServerUseCase<R>
where
    R: ServerRepository + ?Sized,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: Uuid, req: UpdateServerRequest) -> Result<Server> {
        let mut server = self.repository.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(name) = req.name {
            server.name = name;
        }
        if let Some(host) = req.host {
            server.host = host;
        }
        if let Some(port) = req.port {
            server.port = port;
        }
        if let Some(username) = req.username {
            server.username = username;
        }
        if let Some(password_auth) = req.password_auth {
            server.password_auth = password_auth;
        }
        if let Some(executor_type) = req.executor_type {
            server.executor_type = executor_type;
        }
        if let Some(environment) = req.environment {
            server.environment = environment;
        }
        if let Some(server_path) = req.server_path {
            server.server_path = Some(server_path);
        }
        if let Some(start_command) = req.start_command {
            server.start_command = Some(start_command);
        }
        if let Some(stop_command) = req.stop_command {
            server.stop_command = Some(stop_command);
        }
        if let Some(container_name) = req.container_name {
            server.container_name = Some(container_name);
        }
        if let Some(public_host) = req.public_host {
            server.public_host = Some(public_host);
        }
        if let Some(mc_version) = req.mc_version {
            server.mc_version = mc_version;
        }
        if let Some(mc_loader) = req.mc_loader {
            server.mc_loader = mc_loader;
        }
        if let Some(auto_pause) = req.auto_pause {
            server.auto_pause = auto_pause;
        }
        if let Some(ram_allocation) = req.ram_allocation {
            server.ram_allocation = ram_allocation;
        }
        // Allow setting or clearing the webhook URL
        if let Some(webhook_url) = req.discord_webhook_url {
            server.discord_webhook_url = if webhook_url.is_empty() { None } else { Some(webhook_url) };
        }
        if let Some(auto_backup_enabled) = req.auto_backup_enabled {
            server.auto_backup_enabled = auto_backup_enabled;
        }
        if let Some(backup_cron) = req.backup_cron {
            server.backup_cron = if backup_cron.is_empty() { None } else { Some(backup_cron) };
        }
        if let Some(backup_provider) = req.backup_provider {
            server.backup_provider = backup_provider;
        }
        if let Some(backup_path) = req.backup_path {
            server.backup_path = if backup_path.is_empty() { None } else { Some(backup_path) };
        }
        if let Some(max_retained_backups) = req.max_retained_backups {
            server.max_retained_backups = max_retained_backups;
        }
        
        // Git remote configuration
        if let Some(git_remote_url) = req.git_remote_url {
            server.git_remote_url = if git_remote_url.is_empty() { None } else { Some(git_remote_url) };
        }
        if let Some(git_remote_username) = req.git_remote_username {
            server.git_remote_username = if git_remote_username.is_empty() { None } else { Some(git_remote_username) };
        }
        if let Some(git_remote_token) = req.git_remote_token {
            server.git_remote_token = if git_remote_token.is_empty() { None } else { Some(git_remote_token) };
        }
        
        // Node assignment
        if let Some(node_id) = req.node_id {
            server.node_id = if node_id.is_empty() { None } else { Uuid::parse_str(&node_id).ok() };
        }

        // Sleep/Wake & Auto-Restart Backoff (Phase 56)
        if let Some(auto_wake) = req.auto_wake {
            server.auto_wake = auto_wake;
        }
        if let Some(sleep_timeout_minutes) = req.sleep_timeout_minutes {
            server.sleep_timeout_minutes = sleep_timeout_minutes;
        }
        if let Some(max_restart_attempts) = req.max_restart_attempts {
            server.max_restart_attempts = max_restart_attempts;
        }
        if let Some(restart_cooldown_seconds) = req.restart_cooldown_seconds {
            server.restart_cooldown_seconds = restart_cooldown_seconds;
        }

        server.updated_at = Utc::now();

        self.repository.update(&server).await?;
        Ok(server)
    }
}
