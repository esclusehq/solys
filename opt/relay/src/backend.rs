use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use uuid::Uuid;

use crate::auth::Authorization;
use crate::error::GatewayError;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
struct AuthorizeResponse {
    node_id: Uuid,
    user_id: Uuid,
}

pub struct BackendClient {
    base_url: String,
    hmac_secret_env: String,
    hmac_secret: Option<String>,
    http: reqwest::Client,
}

impl BackendClient {
    pub fn new(base_url: String, hmac_secret_env: String, timeout_secs: u64) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()?;
        let hmac_secret = std::env::var(&hmac_secret_env).ok().filter(|s| !s.is_empty());
        Ok(Self {
            base_url,
            hmac_secret_env,
            hmac_secret,
            http,
        })
    }

    fn sign(&self, method: &str, path: &str, body: &str, timestamp: i64, nonce: &str) -> Option<String> {
        let secret = self.hmac_secret.as_deref()?;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(method.as_bytes());
        mac.update(b"\n");
        mac.update(path.as_bytes());
        mac.update(b"\n");
        mac.update(body.as_bytes());
        mac.update(b"\n");
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b"\n");
        mac.update(nonce.as_bytes());
        Some(hex::encode(mac.finalize().into_bytes()))
    }

    fn now_unix() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    fn random_nonce() -> String {
        use rand::RngCore;
        let mut buf = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut buf);
        hex::encode(buf)
    }

    /// POST /internal/relay/authorize with HMAC headers.
    pub async fn authorize(
        &self,
        relay_token: Uuid,
        server_id: Uuid,
    ) -> Result<Authorization, GatewayError> {
        let path = "/internal/relay/authorize";
        let body = serde_json::json!({
            "relay_token": relay_token,
            "server_id": server_id,
        })
        .to_string();
        let ts = Self::now_unix();
        let nonce = Self::random_nonce();

        let url = format!("{}{}", self.base_url, path);
        let mut req = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Esluce-Timestamp", ts.to_string())
            .header("X-Esluce-Nonce", nonce.clone())
            .body(body.clone());

        if let Some(sig) = self.sign("POST", path, &body, ts, &nonce) {
            req = req.header("X-Esluce-Signature", sig);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| GatewayError::BackendUnreachable(e.to_string()))?;

        match resp.status().as_u16() {
            200..=299 => {
                let parsed: AuthorizeResponse = resp
                    .json()
                    .await
                    .map_err(|e| GatewayError::BackendUnreachable(format!("bad /authorize json: {}", e)))?;
                Ok(Authorization {
                    node_id: parsed.node_id,
                    user_id: parsed.user_id,
                })
            }
            401 | 403 => Err(GatewayError::Auth),
            s => Err(GatewayError::BackendUnreachable(format!(
                "unexpected status {} from /authorize",
                s
            ))),
        }
    }

    /// POST /internal/relay/tunnel-event with HMAC headers.
    pub async fn report_tunnel_event(
        &self,
        server_id: Uuid,
        event_type: &str,
        reason: &str,
    ) -> Result<(), GatewayError> {
        self.report_tunnel_event_with_uptime(server_id, event_type, reason, 0)
            .await
    }

    /// POST /internal/relay/tunnel-event with HMAC headers and optional uptime.
    pub async fn report_tunnel_event_with_uptime(
        &self,
        server_id: Uuid,
        event_type: &str,
        reason: &str,
        uptime_secs: u64,
    ) -> Result<(), GatewayError> {
        let path = "/internal/relay/tunnel-event";
        let body = serde_json::json!({
            "server_id": server_id,
            "event_type": event_type,
            "reason": reason,
            "uptime_secs": uptime_secs,
        })
        .to_string();
        let ts = Self::now_unix();
        let nonce = Self::random_nonce();

        let url = format!("{}{}", self.base_url, path);
        let mut req = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Esluce-Timestamp", ts.to_string())
            .header("X-Esluce-Nonce", nonce.clone())
            .body(body.clone());

        if let Some(sig) = self.sign("POST", path, &body, ts, &nonce) {
            req = req.header("X-Esluce-Signature", sig);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| GatewayError::BackendUnreachable(e.to_string()))?;

        if !resp.status().is_success() {
            tracing::warn!(
                "[BACKEND] /tunnel-event returned status {}: {}",
                resp.status(),
                reason
            );
        }
        Ok(())
    }
}
