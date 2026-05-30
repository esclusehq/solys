use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Template entity representing a pre-configured server template
/// for a specific game type and sub-category.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: Uuid,
    pub game_type: String,
    pub category: String,
    pub display_name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub visibility: String,
    pub user_id: Option<Uuid>,
    pub is_builtin: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Template {
    /// Returns hardcoded fallback templates when database is empty.
    /// These cover popular game types and their sub-categories.
    pub fn fallback() -> Vec<Self> {
        let now = Utc::now().naive_utc();
        
        vec![
            // Minecraft variants
            Template {
                id: Uuid::new_v4(),
                game_type: "minecraft".to_string(),
                category: "vanilla".to_string(),
                display_name: "Minecraft Vanilla".to_string(),
                description: Some("Default Minecraft server".to_string()),
                config: serde_json::json!({
                    "docker_image": "itzg/minecraft-server:latest",
                    "default_port": 25565,
                    "env": { "TYPE": "VANILLA", "MEMORY": "2G", "MAX_PLAYERS": "20" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Template {
                id: Uuid::new_v4(),
                game_type: "minecraft".to_string(),
                category: "paper".to_string(),
                display_name: "Minecraft Paper".to_string(),
                description: Some("PaperSpigot server with better performance".to_string()),
                config: serde_json::json!({
                    "docker_image": "itzg/minecraft-server:latest",
                    "default_port": 25565,
                    "env": { "TYPE": "PAPER", "MEMORY": "2G", "MAX_PLAYERS": "50" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Template {
                id: Uuid::new_v4(),
                game_type: "minecraft".to_string(),
                category: "spigot".to_string(),
                display_name: "Minecraft Spigot".to_string(),
                description: Some("Spigot server with API support".to_string()),
                config: serde_json::json!({
                    "docker_image": "itzg/minecraft-server:latest",
                    "default_port": 25565,
                    "env": { "TYPE": "SPIGOT", "MEMORY": "2G", "MAX_PLAYERS": "40" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Template {
                id: Uuid::new_v4(),
                game_type: "minecraft".to_string(),
                category: "forge".to_string(),
                display_name: "Minecraft Forge".to_string(),
                description: Some("Forge server for modded Minecraft".to_string()),
                config: serde_json::json!({
                    "docker_image": "itzg/minecraft-server:latest",
                    "default_port": 25565,
                    "env": { "TYPE": "FORGE", "MEMORY": "4G", "MAX_PLAYERS": "20" },
                    "startup_command": "java -Xms2G -Xmx4G -jar server.jar nogui"
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Template {
                id: Uuid::new_v4(),
                game_type: "minecraft".to_string(),
                category: "fabric".to_string(),
                display_name: "Minecraft Fabric".to_string(),
                description: Some("Fabric server for lightweight mods".to_string()),
                config: serde_json::json!({
                    "docker_image": "itzg/minecraft-server:latest",
                    "default_port": 25565,
                    "env": { "TYPE": "FABRIC", "MEMORY": "2G", "MAX_PLAYERS": "30" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            // Palworld
            Template {
                id: Uuid::new_v4(),
                game_type: "palworld".to_string(),
                category: "default".to_string(),
                display_name: "Palworld".to_string(),
                description: Some("Palworld dedicated server".to_string()),
                config: serde_json::json!({
                    "docker_image": "ghcr.io/axllent/minecraft-palworld:latest",
                    "default_port": 8211,
                    "env": { "MAX_PLAYERS": "32", "COMMUNITY_SERVER": "false" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            // Rust
            Template {
                id: Uuid::new_v4(),
                game_type: "rust".to_string(),
                category: "default".to_string(),
                display_name: "Rust".to_string(),
                description: Some("Rust dedicated server".to_string()),
                config: serde_json::json!({
                    "docker_image": "cm2network/rust:latest",
                    "default_port": 28015,
                    "env": { "RUST_PORT": "28015", "RUST_ADMIN_PORT": "28016" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            // Valheim
            Template {
                id: Uuid::new_v4(),
                game_type: "valheim".to_string(),
                category: "default".to_string(),
                display_name: "Valheim".to_string(),
                description: Some("Valheim dedicated server".to_string()),
                config: serde_json::json!({
                    "docker_image": "lloesche/valheim-server:latest",
                    "default_port": 2456,
                    "env": { "WORLD_NAME": "world", "SERVER_PASSWORD": "" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            // Bedrock Edition - Phase 36
            Template {
                id: Uuid::new_v4(),
                game_type: "bedrock".to_string(),
                category: "default".to_string(),
                display_name: "Minecraft Bedrock".to_string(),
                description: Some("Minecraft Bedrock Edition dedicated server".to_string()),
                config: serde_json::json!({
                    "docker_image": "itzg/minecraft-bedrock-server:latest",
                    "default_port": 19132,
                    "env": { "GAMEMODE": "survival", "DIFFICULTY": "normal", "LEVEL_NAME": "Bedrock Server" }
                }),
                visibility: "public".to_string(),
                user_id: None,
                is_builtin: true,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
        ]
    }

    /// Get fallback templates for a specific game type.
    pub fn fallback_by_game_type(game_type: &str) -> Vec<Self> {
        Self::fallback()
            .into_iter()
            .filter(|t| t.game_type == game_type)
            .collect()
    }
}