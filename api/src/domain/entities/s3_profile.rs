use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Named S3-compatible storage profile (D-14).
/// Managed by admins in platform settings, selectable per server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Profile {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub secret_key: String, // masked in GET responses
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating/updating an S3 profile.
/// Secret key is optional on update — empty keeps existing value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3ProfileInput {
    pub name: String,
    pub endpoint: String,
    pub region: Option<String>,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: Option<String>,
    pub is_default: Option<bool>,
}
