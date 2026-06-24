use std::net::Ipv4Addr;
use std::process::Stdio;

use anyhow::{Context, Result};
use bollard::Docker;
use local_ip_address::list_afinet_netifas;
use serde_json::{json, Value};
use tokio::process::Command;
use uuid::Uuid;

use super::{is_cgnat_suspect, is_lan};
use crate::handlers::dns_watch::detect_public_ip;

/// Collect all raw local-network facts the backend needs to classify a probe failure.
/// Re-uses the existing public-IP helper (DO NOT re-implement) and reads tool presence
/// via `which::which` (precedent: `agent-runtime/src/detector.rs:41`).
pub async fn collect_diagnostics(
    docker: &Docker,
    server_id: Uuid,
    game_port: u16,
) -> Result<Value> {
    // 1) Public IP — reuse dns_watch::detect_public_ip (no second fetcher)
    let public_ip = detect_public_ip().await.ok();

    // 2) Local IP from default-route interface (sync list_afinet_netifas is fast).
    //    `list_afinet_netifas` returns `Vec<(iface_name, IpAddr)>` (local-ip-address 0.6).
    let local_ip: Option<Ipv4Addr> = list_afinet_netifas()
        .ok()
        .and_then(|vec| {
            vec.into_iter().find_map(|(_name, ip)| match ip {
                std::net::IpAddr::V4(v4) if !v4.is_loopback() => Some(v4),
                _ => None,
            })
        });

    // 3) Default gateway via `ip route show default` (Linux-first; best-effort on others)
    let default_gateway: Option<Ipv4Addr> = read_default_gateway().await.ok().flatten();

    // 4) CGN suspect heuristic (D-04) — agent's discretion per CONTEXT
    let is_cgn_suspect = is_cgnat_suspect(local_ip, default_gateway);

    // 5) Host firewall presence — which() check (D-07)
    let firewall_active =
        which::which("ufw").is_ok()
        || which::which("firewalld").is_ok()
        || which::which("iptables").is_ok()
        || which::which("nft").is_ok();

    // 6) Container port bindings via bollard::Docker::inspect_container (PATTERNS.md:132-140)
    let container_name = format!("mc-{}", server_id);
    let port_bound = check_port_bound(docker, &container_name, game_port).await;

    // 7) Tailscale / Cloudflared detection (D-11, D-12) — NEVER install
    let tailscale_up = which::which("tailscale").is_ok()
        && Command::new("tailscale")
            .args(["status", "--json"])
            .stdout(Stdio::piped())
            .output().await
            .map(|o| o.status.success())
            .unwrap_or(false);
    let tailscale_ip = Command::new("tailscale")
        .args(["ip", "-4"])
        .stdout(Stdio::piped())
        .output().await
        .ok()
        .and_then(|o| if o.status.success() {
            String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
        } else { None });
    let cloudflared_up = which::which("cloudflared").is_ok()
        && Command::new("cloudflared")
            .args(["tunnel", "list"])
            .stdout(Stdio::piped())
            .output().await
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false);

    // 8) UPnP IGD presence (D-08 — separate from auto-fix; surface the fact only)
    // Cheap probe: SSDP M-SEARCH; we defer the full upnp-rs::discovery::discover to
    // the upnp handler. Here we just record "we have not yet searched".
    let upnp_available: Option<bool> = None;

    Ok(json!({
        "server_id": server_id,
        "public_ip": public_ip,
        "local_ip": local_ip.map(|v| v.to_string()),
        "default_gateway": default_gateway.map(|v| v.to_string()),
        "is_cgn_suspect": is_cgn_suspect,
        "firewall_active": firewall_active,
        "port_bound": port_bound,
        "tailscale_up": tailscale_up,
        "tailscale_ip": tailscale_ip,
        "cloudflared_up": cloudflared_up,
        "upnp_available": upnp_available,
        "game_port": game_port,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn read_default_gateway() -> Result<Option<Ipv4Addr>> {
    let out = Command::new("ip")
        .args(["route", "show", "default"])
        .output().await
        .context("Failed to run `ip route`")?;
    if !out.status.success() { return Ok(None); }
    let stdout = String::from_utf8_lossy(&out.stdout);
    // First match of "via <ipv4>"
    for line in stdout.lines() {
        if let Some(rest) = line.split_whitespace().skip_while(|w| *w != "via").nth(1) {
            if let Ok(ip) = rest.parse::<Ipv4Addr>() { return Ok(Some(ip)); }
        }
    }
    Ok(None)
}

async fn check_port_bound(docker: &Docker, container_name: &str, port: u16) -> bool {
    let info = match docker.inspect_container(container_name, None).await {
        Ok(i) => i,
        Err(_) => return false,
    };
    let key = format!("{}/tcp", port);
    info.network_settings
        .and_then(|ns| ns.ports)
        .and_then(|p| p.get(&key).cloned())
        .and_then(|b| b.map(|v| !v.is_empty()))
        .unwrap_or(false)
}

/// Used by `upnp` to decide whether to attempt IGD discovery.
/// Cloud VPS nodes have no LAN IGD; skip the call to avoid the
/// 2-second discovery timeout appearing in the audit log (Pitfall 4).
pub fn is_vps_node(local_ip: Option<Ipv4Addr>) -> bool {
    !is_lan(local_ip)
}
