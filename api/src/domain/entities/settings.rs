use serde::{Deserialize, Serialize};

/// Restart policy default values stored in app_settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartDefaults {
    pub max_restart_attempts: i32,
    pub restart_cooldown_seconds: i32,
}

impl Default for RestartDefaults {
    fn default() -> Self {
        Self {
            max_restart_attempts: 5,
            restart_cooldown_seconds: 300,
        }
    }
}

/// S3-compatible storage configuration stored in app_settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

impl S3Config {
    /// Returns true if all required fields are filled.
    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty()
            && !self.bucket.is_empty()
            && !self.access_key.is_empty()
            && !self.secret_key.is_empty()
    }
}
