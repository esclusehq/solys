use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Server {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub job_id: Option<Uuid>,
    pub name: String,
    pub image: String,
    pub executor_type: String,
    pub node_id: Option<Uuid>,
    pub status: String,
    pub remote_id: Option<String>,
    pub port: Option<i32>,
    pub config: serde_json::Value,
    pub resources: serde_json::Value,
    pub auto_wake: Option<bool>,
    pub sleep_timeout_minutes: Option<i32>,
    pub endpoints: serde_json::Value,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl Server {
    pub fn new(user_id: Uuid, name: String, image: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            user_id: Some(user_id),
            agent_id: None,
            job_id: None,
            name,
            image,
            executor_type: "agent".to_string(),
            node_id: None,
            status: "pending".to_string(),
            remote_id: None,
            port: None,
            config: serde_json::json!({}),
            resources: serde_json::json!({"ram": "1G", "cpu": 1, "disk": "5G"}),
            auto_wake: None,
            sleep_timeout_minutes: None,
            endpoints: serde_json::json!([]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

fn default_image() -> String {
    "itzg/minecraft-server:latest".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    #[serde(default)]
    pub user_id: Option<Uuid>,
    pub name: String,
    #[serde(default = "default_image")]
    pub image: String,
    #[serde(default)]
    pub game: Option<String>,
    #[serde(default)]
    pub executor_type: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<i32>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password_auth: Option<String>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub resources: Option<serde_json::Value>,

    // Node assignment
    #[serde(default)]
    pub node_id: Option<String>,

    // Simplified UI fields
    #[serde(default)]
    pub game_type: Option<String>,
    #[serde(default)]
    pub minecraft_version: Option<String>,
    #[serde(default)]
    pub ram_mb: Option<i32>,
    #[serde(default)]
    pub max_players: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub resources: Option<serde_json::Value>,
    pub auto_wake: Option<bool>,
    pub sleep_timeout_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResources {
    pub ram: String,
    pub cpu: i32,
    pub disk: String,
}

impl Default for ServerResources {
    fn default() -> Self {
        Self {
            ram: "1G".to_string(),
            cpu: 1,
            disk: "5G".to_string(),
        }
    }
}
