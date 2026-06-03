//! RCON handler - server.command
//!
//! Full implementation for executing commands via RCON protocol

use std::net::SocketAddr;

use agent_proto::Task;
use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct ServerCommandPayload {
    pub server_id: uuid::Uuid,
    pub container_id: Option<String>,
    pub rcon_port: Option<u16>,
    pub rcon_password: Option<String>,
    pub command: String,
}

#[derive(Debug)]
struct RconPacket {
    pub request_id: i32,
    pub packet_type: i32,
    pub body: String,
}

impl RconPacket {
    const SERVERDATA_AUTH: i32 = 3;
    const SERVERDATA_AUTH_RESPONSE: i32 = 2;
    const SERVERDATA_EXECCOMMAND: i32 = 2;
    const SERVERDATA_RESPONSE_VALUE: i32 = 0;

    fn new(request_id: i32, packet_type: i32, body: &str) -> Self {
        Self {
            request_id,
            packet_type,
            body: body.to_string(),
        }
    }

    fn encode(&self) -> Vec<u8> {
        let body_bytes = self.body.as_bytes();
        let mut packet = Vec::new();
        
        // Packet length (4 bytes) = 4 (request_id) + 4 (packet_type) + body_len + 1 (null terminator)
        let length = (body_bytes.len() + 10) as i32;
        packet.extend_from_slice(&length.to_le_bytes());
        
        // Request ID (4 bytes)
        packet.extend_from_slice(&self.request_id.to_le_bytes());
        
        // Packet type (4 bytes)
        packet.extend_from_slice(&self.packet_type.to_le_bytes());
        
        // Body + null terminator
        packet.extend_from_slice(body_bytes);
        packet.push(0);
        packet.push(0); // Pad to even length
        
        packet
    }

    fn decode(data: &[u8]) -> Option<(i32, i32, String)> {
        if data.len() < 12 {
            return None;
        }

        let request_id = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let packet_type = i32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        
        let body_start = 12;
        let body_end = data.len() - 2; // Remove null terminators
        let body = String::from_utf8_lossy(&data[body_start..body_end]).to_string();

        Some((request_id, packet_type, body))
    }
}

pub async fn handle_command(task: Task) -> Result<serde_json::Value> {
    let payload: ServerCommandPayload = serde_json::from_value(task.payload)?;

    let rcon_port = payload.rcon_port.unwrap_or(25575);
    let rcon_password = payload.rcon_password.unwrap_or_default();
    let command = payload.command.clone();

    info!(
        server_id = %payload.server_id,
        port = rcon_port,
        command = %command,
        "Executing RCON command"
    );

    // Connect to RCON server
    let addr = SocketAddr::from(([127, 0, 0, 1], rcon_port));
    let mut stream = TcpStream::connect(addr)
        .await
        .context("Failed to connect to RCON server")?;

    // Authenticate
    let auth_packet = RconPacket::new(1, RconPacket::SERVERDATA_AUTH, &rcon_password);
    stream.write_all(&auth_packet.encode()).await?;
    
    let mut response_buf = vec![0u8; 4096];
    let n = stream.read(&mut response_buf).await?;
    
    if n < 12 {
        return Err(anyhow::anyhow!("Invalid RCON authentication response"));
    }

    let (response_id, response_type, _) = RconPacket::decode(&response_buf[..n])
        .ok_or_else(|| anyhow::anyhow!("Failed to decode RCON response"))?;

    if response_type != RconPacket::SERVERDATA_AUTH_RESPONSE || response_id == -1 {
        return Err(anyhow::anyhow!("RCON authentication failed - wrong password"));
    }

    // Send command
    let cmd_packet = RconPacket::new(2, RconPacket::SERVERDATA_EXECCOMMAND, &command);
    stream.write_all(&cmd_packet.encode()).await?;

    // Read response
    let mut cmd_response = vec![0u8; 4096];
    let m = stream.read(&mut cmd_response).await?;

    let response = if m >= 12 {
        let (_, _, body) = RconPacket::decode(&cmd_response[..m])
            .ok_or_else(|| anyhow::anyhow!("Failed to decode command response"))?;
        body
    } else {
        String::new()
    };

    info!(
        server_id = %payload.server_id,
        response = %response,
        "RCON command executed"
    );

    Ok(serde_json::json!({
        "status": "ok",
        "response": response
    }))
}
