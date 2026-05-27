use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub event: String,
    pub server_id: Option<String>,
    pub user_id: Option<String>,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

pub struct WebhookEmitter {
    client: reqwest::Client,
}

impl WebhookEmitter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn emit(&self, url: &str, secret: Option<&str>, event: WebhookEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_string(&event)?;
        
        let mut request = self.client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.clone());

        if let Some(secret) = secret {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(body.as_bytes());
            hasher.update(secret.as_bytes());
            let signature = hex::encode(hasher.finalize());
            request = request.header("X-Webhook-Signature", format!("sha256={}", signature));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(format!("Webhook delivery failed: {}", response.status()).into());
        }

        Ok(())
    }

    pub async fn emit_server_event(
        &self,
        url: &str,
        secret: Option<&str>,
        event_type: &str,
        server_id: &str,
        user_id: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = WebhookEvent {
            event: event_type.to_string(),
            server_id: Some(server_id.to_string()),
            user_id: Some(user_id.to_string()),
            data,
            timestamp: Utc::now(),
        };

        self.emit(url, secret, event).await
    }
}