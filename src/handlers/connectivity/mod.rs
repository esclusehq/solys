//! Phase 67: Connectivity diagnostics + safe-to-fix actions
//!
//! - `diagnostics`: collect raw local network facts (D-04) — no install, no auto-config
//! - `firewall`:    open/close a single host-firewall port for a server, scoped with
//!                  an `esluse:<server-id>` comment (D-09, Pitfall 3)
//! - `upnp`:        add/remove a UPnP IGD port mapping with a 1-hour lease, renewed at
//!                  50% of the lease (D-08, Pitfall 4 — skip on VPS)

pub mod diagnostics;
pub mod firewall;
pub mod upnp;

/// CGN heuristic per RFC 6598: 100.64.0.0/10.
/// Returns true if the host's local IP OR default gateway falls in the CGN range.
pub fn is_cgnat_suspect(
    local_ip: Option<std::net::Ipv4Addr>,
    gateway: Option<std::net::Ipv4Addr>,
) -> bool {
    let in_cgn = |ip: std::net::Ipv4Addr| -> bool {
        let o = ip.octets();
        o[0] == 100 && o[1] >= 64 && o[1] <= 127
    };
    local_ip.map(in_cgn).unwrap_or(false) || gateway.map(in_cgn).unwrap_or(false)
}

/// True if the host is on a private LAN (RFC 1918). Inverse of VPS-detection used
/// by `upnp` to skip IGD discovery on cloud nodes.
pub fn is_lan(ip: Option<std::net::Ipv4Addr>) -> bool {
    match ip {
        Some(v4) => v4.is_private(),
        None => false,
    }
}

/// Re-export the re-use point of the existing detect_public_ip helper.
pub use crate::handlers::dns_watch::detect_public_ip;
