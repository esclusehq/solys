//! SSH handler - ssh.connect/ssh.execute/ssh.disconnect

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use agent_proto::Task;
use agent_ssh::SshClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct SshConnectPayload {
    pub server_id: uuid::Uuid,
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub auth: SshAuth,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SshAuth {
    Key {
        key_content: String,
        passphrase: Option<String>,
    },
    Password {
        password: String,
    },
}

#[derive(Debug, Serialize)]
pub struct SshConnectOutput {
    pub connection_key: String,
    pub host: String,
    pub port: u16,
    pub user: String,
}

#[derive(Debug, Deserialize)]
pub struct SshExecutePayload {
    pub server_id: uuid::Uuid,
    pub connection_key: String,
    pub command: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SshExecuteOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Deserialize)]
pub struct SshDisconnectPayload {
    pub server_id: uuid::Uuid,
    pub connection_key: String,
}

struct CachedConnection {
    client: SshClient,
    last_used: Instant,
}

pub struct SshConnectionCache {
    connections: HashMap<String, CachedConnection>,
    ttl_seconds: u64,
    max_connections: usize,
}

impl SshConnectionCache {
    pub fn new(ttl_seconds: u64, max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            ttl_seconds,
            max_connections,
        }
    }

    pub async fn get(&mut self, key: &str) -> Option<SshClient> {
        let conn = self.connections.get(key)?;
        
        if conn.last_used.elapsed() > Duration::from_secs(self.ttl_seconds) {
            self.connections.remove(key);
            return None;
        }
        
        let client = conn.client.clone();
        self.connections.get_mut(key).unwrap().last_used = Instant::now();
        Some(client)
    }

    pub async fn put(&mut self, key: String, client: SshClient) {
        if self.connections.len() >= self.max_connections {
            if let Some(oldest_key) = self.connections.iter()
                .min_by_key(|(_, c)| c.last_used)
                .map(|(k, _)| k.clone())
            {
                self.connections.remove(&oldest_key);
            }
        }

        self.connections.insert(key, CachedConnection {
            client,
            last_used: Instant::now(),
        });
    }

    pub async fn remove(&mut self, key: &str) -> Option<SshClient> {
        self.connections.remove(key).map(|c| c.client)
    }

    pub async fn clear(&mut self) {
        self.connections.clear();
    }
}

impl Default for SshConnectionCache {
    fn default() -> Self {
        Self::new(60, 5)
    }
}

lazy_static::lazy_static! {
    pub static ref SSH_CACHE: Arc<RwLock<SshConnectionCache>> = 
        Arc::new(RwLock::new(SshConnectionCache::default()));
}

pub async fn handle_connect(task: Task) -> Result<serde_json::Value> {
    let payload: SshConnectPayload = serde_json::from_value(task.payload)?;

    let port = payload.port.unwrap_or(22);
    let connection_key = format!("{}:{}@{}:{}", payload.user, payload.server_id, payload.host, port);

    info!(
        server_id = %payload.server_id,
        host = %payload.host,
        port = port,
        user = %payload.user,
        "Connecting to SSH"
    );

    let client = match &payload.auth {
        SshAuth::Key { key_content, passphrase: _ } => {
            let temp_dir = std::env::temp_dir();
            let key_path = temp_dir.join(format!("ssh_key_{}", uuid::Uuid::new_v4()));
            
            std::fs::write(&key_path, key_content)?;
            
            let result = SshClient::connect(
                &payload.host,
                port,
                &payload.user,
                None,
                Some(key_path.to_str().unwrap()),
            ).await;

            let _ = std::fs::remove_file(&key_path);
            result?
        }
        SshAuth::Password { password } => {
            SshClient::connect(
                &payload.host,
                port,
                &payload.user,
                Some(password.as_str()),
                None,
            ).await?
        }
    };

    let mut cache = SSH_CACHE.write().await;
    cache.put(connection_key.clone(), client).await;

    let output = SshConnectOutput {
        connection_key,
        host: payload.host,
        port,
        user: payload.user,
    };

    info!(server_id = %payload.server_id, "SSH connected successfully");

    Ok(serde_json::to_value(output)?)
}

pub async fn handle_execute(task: Task) -> Result<serde_json::Value> {
    let payload: SshExecutePayload = serde_json::from_value(task.payload)?;

    info!(
        server_id = %payload.server_id,
        connection_key = %payload.connection_key,
        command = %payload.command,
        "Executing SSH command"
    );

    let mut cache = SSH_CACHE.write().await;
    let client = cache.get(&payload.connection_key).await
        .ok_or_else(|| anyhow::anyhow!("Connection not found: {}", payload.connection_key))?;

    let output = client.execute(&payload.command).await?;

    cache.put(payload.connection_key.clone(), client).await;

    info!(server_id = %payload.server_id, "SSH command executed");

    Ok(serde_json::json!({
        "stdout": output,
        "stderr": "",
        "exit_code": 0
    }))
}

pub async fn handle_disconnect(task: Task) -> Result<serde_json::Value> {
    let payload: SshDisconnectPayload = serde_json::from_value(task.payload)?;

    info!(
        server_id = %payload.server_id,
        connection_key = %payload.connection_key,
        "Disconnecting SSH"
    );

    let mut cache = SSH_CACHE.write().await;
    cache.remove(&payload.connection_key).await;

    Ok(serde_json::json!({
        "status": "disconnected",
        "connection_key": payload.connection_key
    }))
}
