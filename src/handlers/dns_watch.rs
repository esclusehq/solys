use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, trace, warn};

use super::dns::{self, redact_ip, CloudflareDnsConfig};

static IP_CHECK_URLS: &[&str] = &[
    "https://api.ipify.org",
    "https://checkip.amazonaws.com",
    "https://icanhazip.com",
    "https://ifconfig.me/ip",
];

pub struct DnsWatcher {
    running: Arc<RwLock<bool>>,
    check_interval: Arc<RwLock<Duration>>,
}

impl DnsWatcher {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            check_interval: Arc::new(RwLock::new(Duration::from_secs(300))),
        }
    }

    pub async fn start(&self) {
        let mut guard = self.running.write().await;
        if *guard {
            debug!("DnsWatcher already running");
            return;
        }
        *guard = true;
        drop(guard);

        let running = self.running.clone();
        let check_interval = self.check_interval.clone();

        tokio::spawn(async move {
            debug!("DnsWatcher started — monitoring public IP every {:?}", *check_interval.read().await);

            // Immediate first check
            if let Err(e) = check_and_update().await {
                error!("DnsWatcher initial check failed: {}", e);
            }

            let mut ticker = interval(*check_interval.read().await);
            loop {
                ticker.tick().await;

                if !*running.read().await {
                    debug!("DnsWatcher stopped");
                    break;
                }

                if let Err(e) = check_and_update().await {
                    error!("DnsWatcher IP check failed: {}", e);
                }
            }
        });
    }

    pub async fn stop(&self) {
        let mut guard = self.running.write().await;
        *guard = false;
    }

    pub async fn set_interval(&self, secs: u64) {
        let mut guard = self.check_interval.write().await;
        *guard = Duration::from_secs(secs);
    }

    pub async fn trigger_check(&self) -> Result<()> {
        check_and_update().await
    }
}

async fn check_and_update() -> Result<()> {
    let current_ip = detect_public_ip().await?;

    {
        let mut ip_guard = dns::CURRENT_IP.write().await;
        if *ip_guard != current_ip {
            let old_ip = ip_guard.clone();
            *ip_guard = current_ip.clone();
            trace!("Public IP changed: {} -> {}", redact_ip(&old_ip), redact_ip(&current_ip));
        }
    }

    // Phase 67: an IP change is a connectivity event — kick the diagnostics
    // collector. The exact list of "servers on this node" lives in the
    // agent's task_state; for now we log the event and let the backend's
    // periodic re-probe (5 min) pick it up. The actual cross-component call
    // requires deeper wiring into the WS outbound, so this is an
    // audit-only trigger point.
        tracing::trace!("[CONNECTIVITY_TRIGGER] Public IP changed; backend probe will re-evaluate");

    let config_guard = dns::DNS_CONFIG.read().await;
    let config = match config_guard.as_ref() {
        Some(cfg) => cfg.clone(),
        None => {
            debug!("DNS not configured yet, skipping DNS record update");
            return Ok(());
        }
    };
    drop(config_guard);

    if !config.auto_refresh {
            debug!("Auto-refresh disabled, skipping DNS record update");
        return Ok(());
    }

    let global_subdomain = config.subdomain.clone()
        .unwrap_or_else(|| config.zone_name.split('.').next().unwrap_or("node").to_string());

    // Build the full list of FQDNs to keep in sync:
    //   1. The global record: `<global_subdomain>.<wildcard_domain>` (e.g. `play.esluce.com`)
    //   2. Per-server records: `<server_sub>.<global_subdomain>.<wildcard_domain>`
    //      (e.g. `mantap-wou.play.esluce.com`) — pulled from config.extra_subdomains
    let mut fqdns: Vec<String> = Vec::with_capacity(1 + config.extra_subdomains.len());
    fqdns.push(format!("{}.{}", global_subdomain, config.wildcard_domain));
    for sub in &config.extra_subdomains {
        let trimmed = sub.trim();
        if trimmed.is_empty() {
            continue;
        }
        fqdns.push(format!("{}.{}.{}", trimmed, global_subdomain, config.wildcard_domain));
    }

    let mut updated = 0usize;
    let mut created = 0usize;
    let mut failed = 0usize;
    for full_name in &fqdns {
        match dns::find_dns_record(&config.api_token, &config.zone_id, full_name).await {
            Ok(Some((record_id, _))) => {
                match dns::update_dns_record(
                    &config.api_token,
                    &config.zone_id,
                    &record_id,
                    full_name,
                    &current_ip,
                )
                .await
                {
                    Ok(()) => {
                        updated += 1;
                    }
                    Err(e) => {
                        error!("DNS update failed: {}", e);
                        failed += 1;
                    }
                }
            }
            Ok(None) => match dns::create_dns_record(
                &config.api_token,
                &config.zone_id,
                full_name,
                &current_ip,
            )
            .await
            {
                    Ok(rid) => {
                        created += 1;
                }
                Err(e) => {
                    error!("DNS create failed: {}", e);
                    failed += 1;
                }
            },
            Err(e) => {
                error!("DNS lookup failed: {}", e);
                failed += 1;
            }
        }
    }

    debug!(
        "DDNS cycle complete: {} updated, {} created, {} failed (of {} total)",
        updated,
        created,
        failed,
        fqdns.len()
    );

    Ok(())
}

pub async fn detect_public_ip() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    for url in IP_CHECK_URLS {
        match client.get(*url).send().await {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    let ip = text.trim().to_string();
                    if !ip.is_empty() && is_valid_ip(&ip) {
                        return Ok(ip);
                    }
                }
            }
            Err(e) => {
                warn!("IP check failed for {}: {}", url, e);
                continue;
            }
        }
    }

    Err(anyhow::anyhow!("Failed to detect public IP from all sources"))
}

fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}
