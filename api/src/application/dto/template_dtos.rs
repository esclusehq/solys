use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub game_type: String,
    pub category: String,
    pub display_name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub visibility: Option<String>, // defaults to "private"
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub visibility: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub game_type: String,
    pub category: String,
    pub display_name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub visibility: String,
    pub user_id: Option<Uuid>,
    pub is_builtin: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateServerFromTemplateRequest {
    pub name: String,
    pub node_id: Option<String>,
    pub game_type: Option<String>,
    pub minecraft_version: Option<String>,
    pub ram_mb: Option<i32>,
    pub max_players: Option<i32>,
    #[serde(default)]
    pub config_overrides: Option<serde_json::Value>,
}

impl From<crate::domain::server::template::Template> for TemplateResponse {
    fn from(t: crate::domain::server::template::Template) -> Self {
        Self {
            id: t.id,
            game_type: t.game_type,
            category: t.category,
            display_name: t.display_name,
            description: t.description,
            config: t.config,
            visibility: t.visibility,
            user_id: t.user_id,
            is_builtin: t.is_builtin,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}
