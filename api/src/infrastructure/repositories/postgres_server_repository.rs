use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;
use anyhow::{Result, Context};
use crate::domain::{
    entities::server::Server,
    repositories::server_repository::ServerRepository,
};
use sqlx::Row;

pub struct PostgresServerRepository {
    pool: PgPool,
}

impl PostgresServerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServerRepository for PostgresServerRepository {
    async fn create(&self, server: &Server) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO servers (id, user_id, name, game, status, host, port, username, password_auth, executor_type, environment, server_path, start_command, stop_command, container_name, public_host, mc_version, mc_loader, auto_pause, ram_allocation, discord_webhook_url, auto_backup_enabled, backup_cron, backup_provider, backup_path, max_retained_backups, auto_restart, restart_count, enable_tailscale, tailscale_auth_key, custom_container_name, ip_binding, template, network_name, git_remote_url, git_remote_username, git_remote_token, node_id, auto_wake, sleep_timeout_minutes, last_player_activity, max_restart_attempts, restart_cooldown_seconds, last_restart_at, last_restart_reason, health_check_timeout_seconds, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::server_environment, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42, $43, $44, $45, $46, $47, $48)
            "#,
        )
        .bind(server.id)
        .bind(server.user_id)
        .bind(&server.name)
        .bind(&server.game)
        .bind(&server.status)
        .bind(&server.host)
        .bind(server.port as i32)
        .bind(&server.username)
        .bind(&server.password_auth)
        .bind(&server.executor_type)
        .bind(&server.environment)
        .bind(&server.server_path)
        .bind(&server.start_command)
        .bind(&server.stop_command)
        .bind(&server.container_name)
        .bind(&server.public_host)
        .bind(&server.mc_version)
        .bind(&server.mc_loader)
        .bind(server.auto_pause)
        .bind(&server.ram_allocation)
        .bind(&server.discord_webhook_url)
        .bind(server.auto_backup_enabled)
        .bind(&server.backup_cron)
        .bind(&server.backup_provider)
        .bind(&server.backup_path)
        .bind(server.max_retained_backups)
        .bind(server.auto_restart)
        .bind(server.restart_count)
        .bind(server.enable_tailscale)
        .bind(&server.tailscale_auth_key)
        .bind(&server.custom_container_name)
        .bind(&server.ip_binding)
        .bind(&server.template)
        .bind(&server.network_name)
        .bind(&server.git_remote_url)
        .bind(&server.git_remote_username)
        .bind(&server.git_remote_token)
        .bind(&server.node_id)
        .bind(server.auto_wake)
        .bind(server.sleep_timeout_minutes)
        .bind(server.last_player_activity)
        .bind(server.max_restart_attempts)
        .bind(server.restart_cooldown_seconds)
        .bind(server.last_restart_at)
        .bind(&server.last_restart_reason)
        .bind(server.health_check_timeout_seconds)
        .bind(server.created_at.naive_utc())
        .bind(server.updated_at.naive_utc())
        .execute(&self.pool)
        .await
        .context("Failed to insert server ")?;
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Server>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, name, game, status, COALESCE(host, '') as host, COALESCE(port, 0) as port, username, password_auth, executor_type, environment::text as environment, server_path, start_command, stop_command, container_name, public_host, mc_version, mc_loader, auto_pause, ram_allocation, discord_webhook_url, auto_backup_enabled, backup_cron, backup_provider, backup_path, max_retained_backups, auto_restart, restart_count, enable_tailscale, tailscale_auth_key, custom_container_name, ip_binding, template, network_name, git_remote_url, git_remote_username, git_remote_token, node_id, auto_wake, sleep_timeout_minutes, last_player_activity, max_restart_attempts, restart_cooldown_seconds, last_restart_at, last_restart_reason, health_check_timeout_seconds, created_at, updated_at
            FROM servers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch server ")?;

        match row {
            Some(row) => Ok(Some(Server {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                name: row.try_get("name")?,
                game: row.try_get("game")?,
                status: row.try_get("status")?,
                host: row.try_get("host")?,
                port: row.try_get("port")?,
                username: row.try_get("username")?,
                password_auth: row.try_get("password_auth")?,
                executor_type: row.try_get("executor_type")?,
                environment: row.try_get("environment").unwrap_or_else(|_| "production".to_string()),
                server_path: row.try_get("server_path")?,
                start_command: row.try_get("start_command")?,
                stop_command: row.try_get("stop_command")?,
                container_name: row.try_get("container_name")?,
                public_host: row.try_get("public_host")?,
                mc_version: row.try_get("mc_version")?,
                mc_loader: row.try_get("mc_loader")?,
                auto_pause: row.try_get("auto_pause").unwrap_or(false),
                ram_allocation: row.try_get("ram_allocation").unwrap_or_else(|_| "1G".to_string()),
                discord_webhook_url: row.try_get("discord_webhook_url").ok().flatten(),
                auto_backup_enabled: row.try_get("auto_backup_enabled").unwrap_or(false),
                backup_cron: row.try_get("backup_cron").ok().flatten(),
                backup_provider: row.try_get("backup_provider").unwrap_or_else(|_| "local".to_string()),
                backup_path: row.try_get("backup_path").ok().flatten(),
                max_retained_backups: row.try_get("max_retained_backups").unwrap_or(5),
                
                // New fields
                auto_restart: row.try_get("auto_restart").unwrap_or(true),
                restart_count: row.try_get("restart_count").unwrap_or(0),
                enable_tailscale: row.try_get("enable_tailscale").unwrap_or(false),
                tailscale_auth_key: row.try_get("tailscale_auth_key").ok().flatten(),
                custom_container_name: row.try_get("custom_container_name").ok().flatten(),
                ip_binding: row.try_get("ip_binding").unwrap_or_else(|_| "0.0.0.0".to_string()),
                template: row.try_get("template").unwrap_or_else(|_| "paper".to_string()),
                network_name: row.try_get("network_name").unwrap_or_else(|_| "devnode-minecraft".to_string()),
                
                // Git remote configuration
                git_remote_url: row.try_get("git_remote_url").ok().flatten(),
                git_remote_username: row.try_get("git_remote_username").ok().flatten(),
                git_remote_token: row.try_get("git_remote_token").ok().flatten(),
                
                // Node Agent
                node_id: row.try_get("node_id").ok().flatten(),

                // Sleep/Wake & Auto-Restart Backoff (Phase 56)
                auto_wake: row.try_get("auto_wake").unwrap_or(false),
                sleep_timeout_minutes: row.try_get("sleep_timeout_minutes").unwrap_or(30),
                last_player_activity: row.try_get("last_player_activity").ok().flatten(),
                max_restart_attempts: row.try_get("max_restart_attempts").unwrap_or(5),
                restart_cooldown_seconds: row.try_get("restart_cooldown_seconds").unwrap_or(300),

                // Restart Policy & Health Check (Phase 57)
                last_restart_at: row.try_get("last_restart_at").ok().flatten(),
                last_restart_reason: row.try_get("last_restart_reason").ok().flatten(),
                health_check_timeout_seconds: row.try_get("health_check_timeout_seconds").unwrap_or(5),

                created_at: chrono::DateTime::from_naive_utc_and_offset(row.try_get("created_at")?, chrono::Utc),
                updated_at: chrono::DateTime::from_naive_utc_and_offset(row.try_get("updated_at")?, chrono::Utc),
            })),
            None => Ok(None),
        }
    }

    async fn list(&self) -> Result<Vec<Server>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, name, COALESCE(host, '') as host, COALESCE(port, 0) as port, username, game, password_auth, executor_type, environment::text as environment, server_path, start_command, stop_command, container_name, public_host, mc_version, mc_loader, auto_pause, ram_allocation, discord_webhook_url, auto_backup_enabled, backup_cron, backup_provider, backup_path, max_retained_backups, auto_restart, restart_count, enable_tailscale, tailscale_auth_key, custom_container_name, ip_binding, template, network_name, git_remote_url, git_remote_username, git_remote_token, status, node_id, auto_wake, sleep_timeout_minutes, last_player_activity, max_restart_attempts, restart_cooldown_seconds, last_restart_at, last_restart_reason, health_check_timeout_seconds, created_at, updated_at
            FROM servers
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list servers ")?;

        let mut servers = Vec::new();
        for row in rows {
            servers.push(Server {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                name: row.try_get("name")?,
                host: row.try_get("host")?,
                port: row.try_get("port")?,
                username: row.try_get("username")?,
                password_auth: row.try_get("password_auth")?,
                game: row.try_get("game")?,
                executor_type: row.try_get("executor_type")?,
                environment: row.try_get("environment").unwrap_or_else(|_| "production".to_string()),
                server_path: row.try_get("server_path")?,
                start_command: row.try_get("start_command")?,
                stop_command: row.try_get("stop_command")?,
                container_name: row.try_get("container_name")?,
                public_host: row.try_get("public_host")?,
                mc_version: row.try_get("mc_version")?,
                mc_loader: row.try_get("mc_loader")?,
                auto_pause: row.try_get("auto_pause").unwrap_or(false),
                ram_allocation: row.try_get("ram_allocation").unwrap_or_else(|_| "1G".to_string()),
                discord_webhook_url: row.try_get("discord_webhook_url").ok().flatten(),
                auto_backup_enabled: row.try_get("auto_backup_enabled").unwrap_or(false),
                backup_cron: row.try_get("backup_cron").ok().flatten(),
                backup_provider: row.try_get("backup_provider").unwrap_or_else(|_| "local".to_string()),
                backup_path: row.try_get("backup_path").ok().flatten(),
                max_retained_backups: row.try_get("max_retained_backups").unwrap_or(5),
                
                // New fields
                auto_restart: row.try_get("auto_restart").unwrap_or(true),
                restart_count: row.try_get("restart_count").unwrap_or(0),
                enable_tailscale: row.try_get("enable_tailscale").unwrap_or(false),
                tailscale_auth_key: row.try_get("tailscale_auth_key").ok().flatten(),
                custom_container_name: row.try_get("custom_container_name").ok().flatten(),
                ip_binding: row.try_get("ip_binding").unwrap_or_else(|_| "0.0.0.0".to_string()),
                template: row.try_get("template").unwrap_or_else(|_| "paper".to_string()),
                network_name: row.try_get("network_name").unwrap_or_else(|_| "devnode-minecraft".to_string()),
                
                // Git remote configuration
                git_remote_url: row.try_get("git_remote_url").ok().flatten(),
                git_remote_username: row.try_get("git_remote_username").ok().flatten(),
                git_remote_token: row.try_get("git_remote_token").ok().flatten(),
                
                // Node Agent
                node_id: row.try_get("node_id").ok().flatten(),

                // Sleep/Wake & Auto-Restart Backoff (Phase 56)
                auto_wake: row.try_get("auto_wake").unwrap_or(false),
                sleep_timeout_minutes: row.try_get("sleep_timeout_minutes").unwrap_or(30),
                last_player_activity: row.try_get("last_player_activity").ok().flatten(),
                max_restart_attempts: row.try_get("max_restart_attempts").unwrap_or(5),
                restart_cooldown_seconds: row.try_get("restart_cooldown_seconds").unwrap_or(300),

                // Restart Policy & Health Check (Phase 57)
                last_restart_at: row.try_get("last_restart_at").ok().flatten(),
                last_restart_reason: row.try_get("last_restart_reason").ok().flatten(),
                health_check_timeout_seconds: row.try_get("health_check_timeout_seconds").unwrap_or(5),

                status: row.try_get("status")?,
                created_at: chrono::DateTime::from_naive_utc_and_offset(row.try_get("created_at")?, chrono::Utc),
                updated_at: chrono::DateTime::from_naive_utc_and_offset(row.try_get("updated_at")?, chrono::Utc),
            });
        }

        Ok(servers)
    }

    async fn update(&self, server: &Server) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE servers
            SET name = $2, game = $3, status = $4, host = $5, port = $6, username = $7, password_auth = $8, executor_type = $9, environment = $10::server_environment, server_path = $11, start_command = $12, stop_command = $13, container_name = $14, public_host = $15, mc_version = $16, mc_loader = $17, auto_pause = $18, ram_allocation = $19, discord_webhook_url = $20, auto_backup_enabled = $21, backup_cron = $22, backup_provider = $23, backup_path = $24, max_retained_backups = $25, auto_restart = $26, restart_count = $27, enable_tailscale = $28, tailscale_auth_key = $29, custom_container_name = $30, ip_binding = $31, template = $32, network_name = $33, git_remote_url = $34, git_remote_username = $35, git_remote_token = $36, auto_wake = $37, sleep_timeout_minutes = $38, last_player_activity = $39, max_restart_attempts = $40, restart_cooldown_seconds = $41, last_restart_at = $42, last_restart_reason = $43, health_check_timeout_seconds = $44, updated_at = $45
            WHERE id = $1
            "#,
        )
        .bind(server.id)
        .bind(&server.name)
        .bind(&server.game)
        .bind(&server.status)
        .bind(&server.host)
        .bind(server.port as i32)
        .bind(&server.username)
        .bind(&server.password_auth)
        .bind(&server.executor_type)
        .bind(&server.environment)
        .bind(&server.server_path)
        .bind(&server.start_command)
        .bind(&server.stop_command)
        .bind(&server.container_name)
        .bind(&server.public_host)
        .bind(&server.mc_version)
        .bind(&server.mc_loader)
        .bind(server.auto_pause)
        .bind(&server.ram_allocation)
        .bind(&server.discord_webhook_url)
        .bind(server.auto_backup_enabled)
        .bind(&server.backup_cron)
        .bind(&server.backup_provider)
        .bind(&server.backup_path)
        .bind(server.max_retained_backups)
        .bind(server.auto_restart)
        .bind(server.restart_count)
        .bind(server.enable_tailscale)
        .bind(&server.tailscale_auth_key)
        .bind(&server.custom_container_name)
        .bind(&server.ip_binding)
        .bind(&server.template)
        .bind(&server.network_name)
        .bind(&server.git_remote_url)
        .bind(&server.git_remote_username)
        .bind(&server.git_remote_token)
        .bind(server.auto_wake)
        .bind(server.sleep_timeout_minutes)
        .bind(server.last_player_activity)
        .bind(server.max_restart_attempts)
        .bind(server.restart_cooldown_seconds)
        .bind(server.last_restart_at)
        .bind(&server.last_restart_reason)
        .bind(server.health_check_timeout_seconds)
        .bind(server.updated_at.naive_utc())
        .execute(&self.pool)
        .await
        .context("Failed to update server ")?;

        Ok(())
    }

    async fn update_status(&self, id: &Uuid, status: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE servers
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(chrono::Utc::now().naive_utc())
        .execute(&self.pool)
        .await
        .context("Failed to update server status ")?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM servers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to delete server ")?;
        Ok(())
    }

    async fn find_by_node_id(&self, node_id: &Uuid) -> Result<Vec<Server>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, name, game, status, COALESCE(host, '') as host, COALESCE(port, 0) as port, username, password_auth, executor_type, environment::text as environment, server_path, start_command, stop_command, container_name, public_host, mc_version, mc_loader, auto_pause, ram_allocation, discord_webhook_url, auto_backup_enabled, backup_cron, backup_provider, backup_path, max_retained_backups, auto_restart, restart_count, enable_tailscale, tailscale_auth_key, custom_container_name, ip_binding, template, network_name, git_remote_url, git_remote_username, git_remote_token, node_id, auto_wake, sleep_timeout_minutes, last_player_activity, max_restart_attempts, restart_cooldown_seconds, last_restart_at, last_restart_reason, health_check_timeout_seconds, created_at, updated_at
            FROM servers
            WHERE node_id = $1
            "#,
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to find servers by node_id")?;

        let mut servers = Vec::new();
        for row in rows {
            servers.push(Server {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                name: row.try_get("name")?,
                game: row.try_get("game")?,
                status: row.try_get("status")?,
                host: row.try_get("host")?,
                port: row.try_get("port")?,
                username: row.try_get("username")?,
                password_auth: row.try_get("password_auth")?,
                executor_type: row.try_get("executor_type")?,
                environment: row.try_get("environment").unwrap_or_else(|_| "production".to_string()),
                server_path: row.try_get("server_path")?,
                start_command: row.try_get("start_command")?,
                stop_command: row.try_get("stop_command")?,
                container_name: row.try_get("container_name")?,
                public_host: row.try_get("public_host")?,
                mc_version: row.try_get("mc_version")?,
                mc_loader: row.try_get("mc_loader")?,
                auto_pause: row.try_get("auto_pause").unwrap_or(false),
                ram_allocation: row.try_get("ram_allocation").unwrap_or_else(|_| "1G".to_string()),
                discord_webhook_url: row.try_get("discord_webhook_url").ok().flatten(),
                auto_backup_enabled: row.try_get("auto_backup_enabled").unwrap_or(false),
                backup_cron: row.try_get("backup_cron").ok().flatten(),
                backup_provider: row.try_get("backup_provider").unwrap_or_else(|_| "local".to_string()),
                backup_path: row.try_get("backup_path").ok().flatten(),
                max_retained_backups: row.try_get("max_retained_backups").unwrap_or(5),
                auto_restart: row.try_get("auto_restart").unwrap_or(true),
                restart_count: row.try_get("restart_count").unwrap_or(0),
                enable_tailscale: row.try_get("enable_tailscale").unwrap_or(false),
                tailscale_auth_key: row.try_get("tailscale_auth_key").ok().flatten(),
                custom_container_name: row.try_get("custom_container_name").ok().flatten(),
                ip_binding: row.try_get("ip_binding").unwrap_or_else(|_| "0.0.0.0".to_string()),
                template: row.try_get("template").unwrap_or_else(|_| "paper".to_string()),
                network_name: row.try_get("network_name").unwrap_or_else(|_| "devnode-minecraft".to_string()),
                git_remote_url: row.try_get("git_remote_url").ok().flatten(),
                git_remote_username: row.try_get("git_remote_username").ok().flatten(),
                git_remote_token: row.try_get("git_remote_token").ok().flatten(),
                node_id: row.try_get("node_id").ok().flatten(),

                // Sleep/Wake & Auto-Restart Backoff (Phase 56)
                auto_wake: row.try_get("auto_wake").unwrap_or(false),
                sleep_timeout_minutes: row.try_get("sleep_timeout_minutes").unwrap_or(30),
                last_player_activity: row.try_get("last_player_activity").ok().flatten(),
                max_restart_attempts: row.try_get("max_restart_attempts").unwrap_or(5),
                restart_cooldown_seconds: row.try_get("restart_cooldown_seconds").unwrap_or(300),

                // Restart Policy & Health Check (Phase 57)
                last_restart_at: row.try_get("last_restart_at").ok().flatten(),
                last_restart_reason: row.try_get("last_restart_reason").ok().flatten(),
                health_check_timeout_seconds: row.try_get("health_check_timeout_seconds").unwrap_or(5),

                created_at: chrono::DateTime::from_naive_utc_and_offset(row.try_get("created_at")?, chrono::Utc),
                updated_at: chrono::DateTime::from_naive_utc_and_offset(row.try_get("updated_at")?, chrono::Utc),
            });
        }
        Ok(servers)
    }
}
