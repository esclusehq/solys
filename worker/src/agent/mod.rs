pub struct AgentClient {
    base_url: String,
    client: reqwest::Client,
}

impl AgentClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_server(&self, api_key: &str, request: &CreateServerRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .post(&format!("{}/api/v1/servers", self.base_url))
            .header("X-API-Key", api_key)
            .json(request)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["server_id"].as_str().unwrap_or("").to_string())
    }

    pub async fn delete_server(&self, api_key: &str, server_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _ = self.client
            .delete(&format!("{}/api/v1/servers/{}", self.base_url, server_id))
            .header("X-API-Key", api_key)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub image: String,
    pub resources: serde_json::Value,
    pub env: std::collections::HashMap<String, String>,
}