//! RCON handler - server.command
//!
//! Connects to the Minecraft server's RCON port. The server runs inside a
//! Docker container, so the host has to be resolved via Docker inspect —
//! connecting to 127.0.0.1 will only work if the RCON port is published to
//! the host (which it usually is NOT in this product's topology).

use std::net::{IpAddr, SocketAddr};

use agent_proto::Task;
use agent_runtime::RuntimeDetector;
use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

#[derive(Debug, Deserialize)]
pub struct ServerCommandPayload {
    pub server_id: uuid::Uuid,
    pub container_id: Option<String>,
    pub container_name: Option<String>,
    pub host: Option<String>,
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

/// Resolves the RCON host for a task payload.
///
/// Priority:
///   1. Explicit `host` in the payload (dashboard override, e.g. when the
///      dashboard already knows the container IP).
///   2. Docker inspect of `container_name` / `container_id` — returns the
///      first non-empty IP from the container's network attachments.
async fn resolve_rcon_host(
    runtime: &RuntimeDetector,
    payload: &ServerCommandPayload,
) -> Result<String> {
    if let Some(h) = payload.host.as_ref().filter(|s| !s.is_empty()) {
        return Ok(h.clone());
    }

    let container_ref = payload
        .container_name
        .clone()
        .or_else(|| payload.container_id.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot resolve RCON host: payload has no host, container_name, or container_id"
            )
        })?;

    let docker = runtime
        .docker()
        .ok_or_else(|| anyhow::anyhow!("Docker client not available on agent"))?;

    let inspect = docker
        .inspect_container(&container_ref, None)
        .await
        .with_context(|| format!("Failed to inspect container '{}'", container_ref))?;

    if let Some(networks) = inspect
        .network_settings
        .as_ref()
        .and_then(|ns| ns.networks.as_ref())
    {
        for (net_name, endpoint) in networks {
            if let Some(ip) = endpoint.ip_address.as_ref().filter(|s| !s.is_empty()) {
                debug!(
                    container = %container_ref,
                    network = %net_name,
                    ip = %ip,
                    "Resolved container IP for RCON"
                );
                return Ok(ip.clone());
            }
        }
    }

    Err(anyhow::anyhow!(
        "Container '{}' has no network IP address (is it running?)",
        container_ref
    ))
}

pub async fn handle_command(task: Task, runtime: &RuntimeDetector) -> Result<serde_json::Value> {
    let payload: ServerCommandPayload = serde_json::from_value(task.payload)?;

    let rcon_port = payload.rcon_port.unwrap_or(25575);
    let rcon_password = payload.rcon_password.clone().unwrap_or_default();
    let command = payload.command.clone();

    let host = match resolve_rcon_host(runtime, &payload).await {
        Ok(h) => h,
        Err(e) => {
            warn!(
                server_id = %payload.server_id,
                error = %e,
                "Failed to resolve container IP for RCON, falling back to 127.0.0.1"
            );
            "127.0.0.1".to_string()
        }
    };

    info!(
        server_id = %payload.server_id,
        host = %host,
        port = rcon_port,
        command = %command,
        "Executing RCON command"
    );

    let ip: IpAddr = host
        .parse()
        .unwrap_or_else(|_| IpAddr::from([127, 0, 0, 1]));
    let addr = SocketAddr::from((ip, rcon_port));
    let mut stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("Failed to connect to RCON server at {}", addr))?;

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
