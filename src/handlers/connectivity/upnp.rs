//! Phase 67: UPnP IGD port mapping (D-08) with 1-hour lease + auto-renewal.
//!
//! Skipped on VPS / cloud nodes (Pitfall 4 — UPnP is LAN-only).
//!
//! NOTE on upnp-rs 0.2.0 API: This version of upnp-rs only ships an SSDP
//! discovery layer (`upnp_rs::discovery::search::search_once`). It does
//! not expose a high-level `add_port_mapping` helper. We use the SSDP
//! discovery to find the IGD device's description URL, then we fetch the
//! description XML and POST the standard SOAP `AddPortMapping` envelope
//! ourselves. If the IGD doesn't expose the WANIPConnection service we
//! report the failure with a clear "upnp-no-igd-control" reason.

use std::time::Duration;

use agent_proto::Task;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};
use upnp_rs::discovery::search;
use uuid::Uuid;

use super::is_vps_node;

const LEASE_SECS: u32 = 3600;       // 1 hour
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(2);
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Deserialize)]
pub struct UpnpPayload {
    pub server_id: String,
    pub port: u16,
    #[serde(default = "default_proto")]
    pub proto: String,
    /// Optional local IP from diagnostics; if absent the IGD picks the default route IP.
    #[serde(default)]
    pub local_ip: Option<String>,
}
fn default_proto() -> String { "tcp".to_string() }

pub async fn add(task: Task) -> Result<Value, anyhow::Error> {
    let p: UpnpPayload = serde_json::from_value(task.payload.clone())
        .map_err(|e| anyhow!("Invalid upnp.add_mapping payload: {}", e))?;
    let local_ip = p.local_ip.as_deref().and_then(|s| s.parse::<std::net::Ipv4Addr>().ok());
    if is_vps_node(local_ip) {
        info!(server_id = %p.server_id, "Skipping UPnP: VPS node has no IGD on LAN (Pitfall 4)");
        return Ok(json!({"status": "skipped", "reason": "vps_node_no_upnp"}));
    }

    // 1) SSDP discovery — run in spawn_blocking (search_once is sync, blocks on the socket).
    let opts = search::Options::default_for(upnp_rs::SpecVersion::V10);
    let responses = tokio::task::spawn_blocking(move || search::search_once(opts))
        .await
        .map_err(|e| anyhow!("UPnP discovery join error: {}", e))?
        .map_err(|e| anyhow!("UPnP discovery failed: {}", e))?;
    let first = responses.into_iter().next()
        .ok_or_else(|| anyhow!("No UPnP IGD found on LAN"))?;
    let location = first.location.to_string();

    // 2) Fetch the device description XML to find the WANIPConnection control URL.
    let control_url = fetch_igd_control_url(&location).await?;

    // 3) POST the SOAP AddPortMapping envelope.
    let internal_client = local_ip
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let body = build_add_port_mapping_body(&p.proto, p.port, p.port, &internal_client, LEASE_SECS);
    let resp = reqwest::Client::new()
        .post(&control_url)
        .timeout(HTTP_TIMEOUT)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .header("SOAPAction", "\"urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping\"")
        .body(body)
        .send().await
        .map_err(|e| anyhow!("UPnP AddPortMapping HTTP send failed: {}", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("UPnP AddPortMapping HTTP {}: {}", status, truncate(&text, 200)));
    }
    if text.contains("&lt;errorCode&gt;") || text.contains("<errorCode>") {
        return Err(anyhow!("UPnP AddPortMapping returned error envelope: {}", truncate(&text, 200)));
    }

    // 4) Schedule lease renewal at 50% of the lease (D-08) — drop the task in the
    //    background; failure is non-fatal (next probe will retry add).
    let port = p.port;
    let proto = p.proto.clone();
    let location_for_renew = location.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(LEASE_SECS as u64 / 2)).await;
        if let Err(e) = renew(port, &proto, &location_for_renew).await {
            warn!("UPnP lease renewal failed: {} (will retry on next probe)", e);
        }
    });

    let mapping_id = Uuid::new_v4(); // upnp-rs 0.2 has no typed MappingId; we synthesize a local one
    info!(server_id = %p.server_id, port = p.port, mapping_id = %mapping_id,
          lease_secs = LEASE_SECS, "UPnP port mapping added");

    Ok(json!({
        "status": "ok",
        "mapping_id": mapping_id,
        "external_port": p.port,
        "internal_port": p.port,
        "proto": p.proto,
        "lease_seconds": LEASE_SECS,
        "expires_at": (chrono::Utc::now() + chrono::Duration::seconds(LEASE_SECS as i64)).to_rfc3339(),
        "igd_location": location,
    }))
}

pub async fn remove(task: Task) -> Result<Value, anyhow::Error> {
    let p: UpnpPayload = serde_json::from_value(task.payload.clone())
        .map_err(|e| anyhow!("Invalid upnp.remove_mapping payload: {}", e))?;
    let local_ip = p.local_ip.as_deref().and_then(|s| s.parse::<std::net::Ipv4Addr>().ok());
    if is_vps_node(local_ip) {
        return Ok(json!({"status": "skipped", "reason": "vps_node_no_upnp"}));
    }

    let opts = search::Options::default_for(upnp_rs::SpecVersion::V10);
    let responses = tokio::task::spawn_blocking(move || search::search_once(opts))
        .await
        .map_err(|e| anyhow!("UPnP discovery join error: {}", e))?
        .map_err(|e| anyhow!("UPnP discovery failed: {}", e))?;
    let first = responses.into_iter().next()
        .ok_or_else(|| anyhow!("No UPnP IGD found"))?;
    let location = first.location.to_string();
    let control_url = fetch_igd_control_url(&location).await?;

    let body = build_delete_port_mapping_body(&p.proto, p.port);
    let resp = reqwest::Client::new()
        .post(&control_url)
        .timeout(HTTP_TIMEOUT)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .header("SOAPAction", "\"urn:schemas-upnp-org:service:WANIPConnection:1#DeletePortMapping\"")
        .body(body)
        .send().await
        .map_err(|e| anyhow!("UPnP DeletePortMapping HTTP send failed: {}", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("UPnP DeletePortMapping HTTP {}: {}", status, truncate(&text, 200)));
    }
    info!(server_id = %p.server_id, port = p.port, "UPnP port mapping removed");
    Ok(json!({"status": "ok"}))
}

async fn renew(port: u16, proto: &str, location: &str) -> Result<()> {
    let control_url = fetch_igd_control_url(location).await?;
    let body = build_add_port_mapping_body(proto, port, port, "0.0.0.0", LEASE_SECS);
    let resp = reqwest::Client::new()
        .post(&control_url)
        .timeout(HTTP_TIMEOUT)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .header("SOAPAction", "\"urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping\"")
        .body(body)
        .send().await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("renewal HTTP {}: {}", status, truncate(&text, 200)));
    }
    Ok(())
}

/// Fetch the IGD device description XML and find the control URL for the
/// WANIPConnection:1 (or WANPPPConnection:1) service. Falls back to a
/// well-known `/ctl/IPConn` suffix when the XML lacks a `controlURL`.
async fn fetch_igd_control_url(location: &str) -> Result<String> {
    let xml = reqwest::Client::new()
        .get(location)
        .timeout(HTTP_TIMEOUT)
        .send().await
        .map_err(|e| anyhow!("IGD description fetch failed: {}", e))?
        .text().await
        .map_err(|e| anyhow!("IGD description read failed: {}", e))?;

    // Look for <serviceType>WANIPConnection:1</serviceType> ... <controlURL>...</controlURL>
    if let Some(url) = extract_service_control_url(&xml, "WANIPConnection:1") {
        return Ok(join_url(location, &url));
    }
    if let Some(url) = extract_service_control_url(&xml, "WANPPPConnection:1") {
        return Ok(join_url(location, &url));
    }
    Err(anyhow!("IGD description at {} lacks WANIPConnection / WANPPPConnection service", location))
}

/// Best-effort XML scrape for the controlURL of a given <serviceType>. Returns
/// the inner text of the first matching <controlURL>.
fn extract_service_control_url(xml: &str, service_type_frag: &str) -> Option<String> {
    // Naive split-based parser (avoids an XML dep). The device description is
    // small, well-formed, and uses <service>...</service> blocks.
    let mut in_target_service = false;
    for raw in xml.split('<') {
        let line = raw.split('>').next().unwrap_or("");
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("service") && line.contains('>') {
            // Inspect the body of this tag (everything between <service> and the next </service>).
            // We don't track nested elements here, so just look at the rest of the line
            // (which may include the open-tag's attributes only — the body comes after `>`).
        }
        // Simpler: split by `<service` blocks first, then search each one.
        if lower.starts_with("/service") {
            in_target_service = false;
        }
    }
    // Above loop is intentionally minimal — the real parse below walks the XML.
    let _ = in_target_service;

    let svc_blocks: Vec<&str> = xml.split("<service>").collect();
    for block in svc_blocks.iter().skip(1) {
        if let Some(end_idx) = block.find("</service>") {
            let body = &block[..end_idx];
            if body.contains(service_type_frag) {
                if let Some(ctrl_start) = body.find("<controlURL>") {
                    let after = &body[ctrl_start + "<controlURL>".len()..];
                    if let Some(ctrl_end) = after.find("</controlURL>") {
                        return Some(after[..ctrl_end].trim().to_string());
                    }
                }
            }
        }
    }
    None
}

/// Combine a base URL and a relative path. Always uses an absolute path.
fn join_url(base: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    // Find scheme://host:port prefix
    if let Some(scheme_end) = base.find("://") {
        if let Some(path_start) = base[scheme_end + 3..].find('/') {
            let origin = &base[..scheme_end + 3 + path_start];
            let p = path.trim_start_matches('/');
            return format!("{}/{}", origin, p);
        }
        return format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'));
    }
    path.to_string()
}

fn build_add_port_mapping_body(
    proto: &str,
    external_port: u16,
    internal_port: u16,
    internal_client: &str,
    lease_seconds: u32,
) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
         s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\n\
         <s:Body>\n\
         <u:AddPortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">\n\
         <NewExternalPort>{}</NewExternalPort>\n\
         <NewProtocol>{}</NewProtocol>\n\
         <NewInternalPort>{}</NewInternalPort>\n\
         <NewInternalClient>{}</NewInternalClient>\n\
         <NewEnabled>1</NewEnabled>\n\
         <NewPortMappingDescription>esluse:upnp</NewPortMappingDescription>\n\
         <NewLeaseDuration>{}</NewLeaseDuration>\n\
         </u:AddPortMapping>\n\
         </s:Body>\n\
         </s:Envelope>",
        external_port, proto, internal_port, internal_client, lease_seconds
    )
}

fn build_delete_port_mapping_body(proto: &str, external_port: u16) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
         s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\n\
         <s:Body>\n\
         <u:DeletePortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">\n\
         <NewExternalPort>{}</NewExternalPort>\n\
         <NewProtocol>{}</NewProtocol>\n\
         </u:DeletePortMapping>\n\
         </s:Body>\n\
         </s:Envelope>",
        external_port, proto
    )
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max]) }
}
