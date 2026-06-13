use std::sync::Arc;
use tokio::sync::RwLock;

use agent_proto::Task;
use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, trace, warn};

pub fn redact_ip(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.{}.***.***", parts[0], parts[1])
    } else {
        "***.***.***.***".to_string()
    }
}

static CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareDnsConfig {
    pub api_token: String,
    pub zone_id: String,
    pub zone_name: String,
    pub wildcard_domain: String,
    pub auto_refresh: bool,
    pub refresh_interval_secs: u64,
    pub subdomain: Option<String>,
    /// Per-server subdomains to keep in sync alongside the global one
    /// (e.g. `["mantap-wou", "server-lain"]` — the watcher prepends
    /// `<sub>.<global_subdomain>.<wildcard_domain>` to build the FQDN).
    #[serde(default)]
    pub extra_subdomains: Vec<String>,
}

lazy_static! {
    pub static ref DNS_CONFIG: Arc<RwLock<Option<CloudflareDnsConfig>>> =
        Arc::new(RwLock::new(None));
    pub static ref CURRENT_IP: Arc<RwLock<String>> =
        Arc::new(RwLock::new(String::new()));
}

#[derive(Debug, Deserialize)]
struct CloudflareApiResponse {
    success: bool,
    errors: Vec<CloudflareApiError>,
    result: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct CloudflareApiError {
    code: i32,
    message: String,
}

pub async fn handle_configure(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let payload = task.payload.clone();
    let config: CloudflareDnsConfig = serde_json::from_value(payload)
        .map_err(|e| anyhow!("Invalid DNS config payload: {}", e))?;

    let mut guard = DNS_CONFIG.write().await;
    *guard = Some(config.clone());
    drop(guard);

    debug!(
        "DNS configured for zone: {} ({} global subdomain, {} per-server subdomains)",
        config.zone_name,
        config.subdomain.as_deref().unwrap_or("none"),
        config.extra_subdomains.len()
    );

    Ok(json!({
        "status": "configured",
        "zone": config.zone_name,
        "domain": config.wildcard_domain,
        "per_server_count": config.extra_subdomains.len(),
    }))
}

pub async fn handle_create_record(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let payload = task.payload.clone();
    let name = payload.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'name' in create_record payload"))?;
    let ip = payload.get("ip")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'ip' in create_record payload"))?;

    let config = DNS_CONFIG.read().await;
    let cfg = config.as_ref()
        .ok_or_else(|| anyhow!("DNS not configured"))?;

    let full_name = format!("{}.{}", name, cfg.wildcard_domain);
    let record_id = create_dns_record(&cfg.api_token, &cfg.zone_id, &full_name, ip).await?;

    debug!("DNS record created: {} -> {}", full_name, redact_ip(ip));

    Ok(json!({
        "status": "created",
        "domain": full_name,
        "ip": ip,
        "record_id": record_id,
    }))
}

pub async fn handle_update_record(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let payload = task.payload.clone();
    let name = payload.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'name' in update_record payload"))?;
    let ip = payload.get("ip")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'ip' in update_record payload"))?;
    let record_id = payload.get("record_id")
        .and_then(|v| v.as_str());

    let config = DNS_CONFIG.read().await;
    let cfg = config.as_ref()
        .ok_or_else(|| anyhow!("DNS not configured"))?;

    let full_name = format!("{}.{}", name, cfg.wildcard_domain);

    if let Some(rid) = record_id {
        update_dns_record(&cfg.api_token, &cfg.zone_id, rid, &full_name, ip).await?;
        debug!("DNS record updated: {} -> {} (record: {})", full_name, redact_ip(ip), rid);
    } else {
        let existing = find_dns_record(&cfg.api_token, &cfg.zone_id, &full_name).await?;
        match existing {
            Some((rid, _)) => {
                update_dns_record(&cfg.api_token, &cfg.zone_id, &rid, &full_name, ip).await?;
                debug!("DNS record found & updated: {} -> {} (record: {})", full_name, redact_ip(ip), rid);
            }
            None => {
                let new_rid = create_dns_record(&cfg.api_token, &cfg.zone_id, &full_name, ip).await?;
                debug!("DNS record not found, created new: {} -> {} (record: {})", full_name, redact_ip(ip), new_rid);
            }
        }
    }

    Ok(json!({
        "status": "updated",
        "domain": full_name,
        "ip": ip,
    }))
}

pub async fn handle_delete_record(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let payload = task.payload.clone();
    let name = payload.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'name' in delete_record payload"))?;

    let config = DNS_CONFIG.read().await;
    let cfg = config.as_ref()
        .ok_or_else(|| anyhow!("DNS not configured"))?;

    let full_name = format!("{}.{}", name, cfg.wildcard_domain);

    let existing = find_dns_record(&cfg.api_token, &cfg.zone_id, &full_name).await?;
    match existing {
        Some((rid, _)) => {
            delete_dns_record(&cfg.api_token, &cfg.zone_id, &rid).await?;
            debug!("DNS record deleted: {} (record: {})", full_name, rid);
        }
        None => {
            warn!("DNS record not found for deletion: {}", full_name);
        }
    }

    Ok(json!({
        "status": "deleted",
        "domain": full_name,
    }))
}

// ---------------------------------------------------------------------------
// Phase 68: remove_record — issued by the relay client's self-loop on
// tunnel disconnect (D-13 / RESOLVED Q7). Removes the stale A record at
// `<subdomain>.play.esluce.com` so a future re-resolve doesn't get cached
// at a now-defunct IP. Resolves the zone_id and record_id from the task
// payload (NOT from DNS_CONFIG) because the agent may have lost DNS
// credentials by the time this fires — the payload carries everything
// needed for a single DELETE call.
// ---------------------------------------------------------------------------

pub async fn handle_remove_record(task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let payload = task.payload.clone();
    let api_token = payload.get("api_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'api_token' in remove_record payload"))?;
    let zone_id = payload.get("zone_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'zone_id' in remove_record payload"))?;
    let record_id = payload.get("record_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'record_id' in remove_record payload"))?;
    let subdomain = payload.get("subdomain")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");

    let client = reqwest::Client::new();
    let url = format!("{}/zones/{}/dns_records/{}", CLOUDFLARE_API_BASE, zone_id, record_id);

    let resp = client.delete(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .send()
        .await
        .context("Failed to send Cloudflare DELETE dns_records request")?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    // 404 → record already gone, treat as success per the plan's contract.
    if status.as_u16() == 404 {
        trace!(
            subdomain = %subdomain,
            record_id = %record_id,
            "DNS record already absent (404), treating as success"
        );
        return Ok(json!({
            "removed": true,
            "already_gone": true,
            "subdomain": subdomain,
            "record_id": record_id,
        }));
    }

    if !status.is_success() {
        error!(
            subdomain = %subdomain,
            record_id = %record_id,
            status = %status,
            body = %text,
            "Cloudflare DELETE dns_records failed"
        );
        return Err(anyhow!("Cloudflare API error ({}): {}", status, text));
    }

    debug!(
        subdomain = %subdomain,
        record_id = %record_id,
        "DNS record removed (CNAME/A cleanup after tunnel disconnect)"
    );

    Ok(json!({
        "removed": true,
        "subdomain": subdomain,
        "record_id": record_id,
    }))
}

pub async fn handle_status(_task: Task) -> Result<serde_json::Value, anyhow::Error> {
    let config = DNS_CONFIG.read().await;
    let ip = CURRENT_IP.read().await;

    let status = match config.as_ref() {
        Some(cfg) => {
            let global_subdomain = cfg.subdomain.as_deref().unwrap_or("node");
            let domain = format!("{}.{}", global_subdomain, cfg.wildcard_domain);
            let per_server_domains: Vec<String> = cfg
                .extra_subdomains
                .iter()
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("{}.{}.{}", s.trim(), global_subdomain, cfg.wildcard_domain))
                .collect();

            json!({
                "configured": true,
                "zone": cfg.zone_name,
                "domain": domain,
                "per_server_domains": per_server_domains,
                "current_ip": ip.clone(),
                "auto_refresh": cfg.auto_refresh,
                "interval_secs": cfg.refresh_interval_secs,
            })
        }
        None => json!({
            "configured": false,
            "current_ip": ip.clone(),
        }),
    };

    Ok(status)
}

pub(crate) async fn create_dns_record(api_token: &str, zone_id: &str, name: &str, ip: &str) -> Result<String, anyhow::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/zones/{}/dns_records", CLOUDFLARE_API_BASE, zone_id);

    let body = json!({
        "type": "A",
        "name": name,
        "content": ip,
        "ttl": 120,
        "proxied": false,
    });

    let resp = client.post(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to send create DNS record request")?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(anyhow!("Cloudflare API error ({}): {}", status, text));
    }

    let api_resp: CloudflareApiResponse = serde_json::from_str(&text)
        .context("Failed to parse Cloudflare API response")?;

    if !api_resp.success {
        let err_msg = api_resp.errors.first()
            .map(|e| format!("{}: {}", e.code, e.message))
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(anyhow!("Cloudflare API error: {}", err_msg));
    }

    let record_id = api_resp.result
        .and_then(|r| r.get("id").and_then(|v| v.as_str().map(String::from)))
        .ok_or_else(|| anyhow!("No record ID in Cloudflare response"))?;

    Ok(record_id)
}

pub(crate) async fn update_dns_record(api_token: &str, zone_id: &str, record_id: &str, name: &str, ip: &str) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/zones/{}/dns_records/{}", CLOUDFLARE_API_BASE, zone_id, record_id);

    let body = json!({
        "type": "A",
        "name": name,
        "content": ip,
        "ttl": 120,
        "proxied": false,
    });

    let resp = client.patch(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to send update DNS record request")?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Cloudflare API error ({}): {}", status, text));
    }

    Ok(())
}

async fn delete_dns_record(api_token: &str, zone_id: &str, record_id: &str) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/zones/{}/dns_records/{}", CLOUDFLARE_API_BASE, zone_id, record_id);

    let resp = client.delete(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .send()
        .await
        .context("Failed to send delete DNS record request")?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Cloudflare API error ({}): {}", status, text));
    }

    Ok(())
}

pub(crate) async fn find_dns_record(api_token: &str, zone_id: &str, name: &str) -> Result<Option<(String, String)>, anyhow::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/zones/{}/dns_records?type=A&name={}", CLOUDFLARE_API_BASE, zone_id, name);

    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .send()
        .await
        .context("Failed to list DNS records")?;

    let text = resp.text().await.unwrap_or_default();
    let api_resp: CloudflareApiResponse = serde_json::from_str(&text)
        .context("Failed to parse Cloudflare API response")?;

    if !api_resp.success {
        return Ok(None);
    }

    let records = api_resp.result
        .and_then(|r| r.as_array().cloned())
        .unwrap_or_default();

    if let Some(record) = records.into_iter().next() {
        let rid = record.get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();
        let content = record.get("content")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();
        Ok(Some((rid, content)))
    } else {
        Ok(None)
    }
}
