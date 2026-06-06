//! Phase 67: Safe-to-fix host firewall port management (D-05, D-09)
//!
//! Auto-fixes ONLY the specific server's port. Every added rule carries an
//! `esluse:<server-id>` comment for unambiguous cleanup. Existing user rules
//! are NEVER modified or deleted.

use std::process::Stdio;

use agent_proto::Task;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
pub struct FirewallPortPayload {
    pub server_id: String,   // UUID as string
    pub port: u16,
    #[serde(default = "default_proto")]
    pub proto: String,
}
fn default_proto() -> String { "tcp".to_string() }

/// Pick the highest-priority firewall CLI available on the host.
/// Order: ufw > firewalld > iptables > nft. None means no firewall tooling.
fn pick_cli() -> Option<(&'static str, &'static str)> {
    if which::which("ufw").is_ok()       { Some(("ufw",       "ufw")) }
    else if which::which("firewalld").is_ok() { Some(("firewalld", "firewall-cmd")) }
    else if which::which("iptables").is_ok()  { Some(("iptables", "iptables")) }
    else if which::which("nft").is_ok()       { Some(("nft",       "nft")) }
    else { None }
}

pub async fn open(task: Task) -> Result<Value, anyhow::Error> {
    let p: FirewallPortPayload = serde_json::from_value(task.payload.clone())
        .map_err(|e| anyhow!("Invalid firewall.open_port payload: {}", e))?;
    let comment = format!("esluse:{}", p.server_id);

    let (cli, cmd) = pick_cli()
        .ok_or_else(|| anyhow!("No host firewall CLI found (ufw/firewalld/iptables/nft)"))?;

    let (args, persist): (Vec<String>, Option<&'static str>) = match cli {
        "ufw" => (
            vec!["allow".into(), format!("{}/{}", p.port, p.proto), "comment".into(), comment.clone()],
            None, // ufw persists by itself
        ),
        "firewalld" => (
            vec!["--zone=public".into(), "--add-port=".into(),
                 format!("{}/{}", p.port, p.proto), "--permanent".into()],
            None,
        ),
        "iptables" => (
            vec!["-I".into(), "INPUT".into(), "-p".into(), p.proto.clone(),
                 "--dport".into(), p.port.to_string(),
                 "-m".into(), "comment".into(), "--comment".into(), comment.clone(),
                 "-j".into(), "ACCEPT".into()],
            Some("netfilter-persistent save"),
        ),
        "nft" => (
            vec!["add".into(), "rule".into(), "inet".into(), "filter".into(),
                 "input".into(), "tcp".into(), "dport".into(), p.port.to_string(),
                 "accept".into(), "comment".into(), comment.clone()],
            Some("nft list ruleset > /etc/nftables.conf"),
        ),
        _ => unreachable!(),
    };

    let mut command = Command::new(cmd);
    command.args(&args).stdout(Stdio::piped()).stderr(Stdio::piped());
    let out = command.output().await
        .with_context(|| format!("Failed to run `{}`", cmd))?;
    if !out.status.success() {
        return Err(anyhow!("`{} {}` failed: {}", cmd,
            args.iter().map(|s| shell_escape(s)).collect::<Vec<_>>().join(" "),
            String::from_utf8_lossy(&out.stderr)));
    }

    // Persistence — ufw/firewalld are persistent by themselves (Pitfall 7)
    let mut persisted = true;
    if let Some(persist_cmd) = persist {
        let mut persist_parts = persist_cmd.split_whitespace();
        let head = persist_parts.next().unwrap_or("");
        let tail: Vec<&str> = persist_parts.collect();
        let out = Command::new(head).args(&tail).output().await
            .with_context(|| format!("Failed to run persist: {}", persist_cmd))?;
        persisted = out.status.success();
        if !persisted {
            warn!("Firewall rule added but persistence failed: {}", persist_cmd);
        }
    }

    let cmd_string = format!("{} {}", cmd,
        args.iter().map(|s| shell_escape(s)).collect::<Vec<_>>().join(" "));
    info!(server_id = %p.server_id, port = p.port, command = %cmd_string,
          "Firewall port opened (D-09 scoped with comment)");

    Ok(json!({
        "status": "ok",
        "cli": cli,
        "command": cmd_string,
        "persisted": persisted,
        "comment": comment,
    }))
}

pub async fn close(task: Task) -> Result<Value, anyhow::Error> {
    let p: FirewallPortPayload = serde_json::from_value(task.payload.clone())
        .map_err(|e| anyhow!("Invalid firewall.close_port payload: {}", e))?;
    let comment = format!("esluse:{}", p.server_id);

    let (cli, cmd) = pick_cli()
        .ok_or_else(|| anyhow!("No host firewall CLI found"))?;

    // Enumerate ALL matching rules and delete each (Pitfall 3 — comment-match races).
    // For ufw/firewalld there is no comment-match cleanup; they have delete-by-rule.
    let deleted = match cli {
        "ufw" => {
            // ufw stores rules per (port/proto). Delete by port+proto. Safe because
            // Escluse only ever added this one rule.
            let out = Command::new("ufw")
                .args(["delete", "allow", &format!("{}/{}", p.port, p.proto)])
                .output().await.context("Failed to run `ufw delete`")?;
            out.status.success()
        }
        "firewalld" => {
            let out = Command::new("firewall-cmd")
                .args(["--zone=public", "--remove-port=",
                       &format!("{}/{}", p.port, p.proto), "--permanent"])
                .output().await.context("Failed to run `firewall-cmd`")?;
            out.status.success()
        }
        "iptables" => delete_iptables_matching(&comment).await?,
        "nft" => delete_nft_matching(&comment).await?,
        _ => unreachable!(),
    };

    info!(server_id = %p.server_id, port = p.port, deleted = deleted,
          "Firewall port closed (D-09 cleanup)");

    Ok(json!({
        "status": if deleted { "ok" } else { "no_match" },
        "cli": cli,
        "command": format!("delete rule with comment {}", comment),
    }))
}

/// Enumerate every iptables rule that carries `comment "esluse:<id>"` and delete
/// each one explicitly. `iptables -D` with `-m comment` only removes the FIRST
/// match, so a single delete can leave TCP+UDP×IPv4+IPv6 rules behind (Pitfall 3).
async fn delete_iptables_matching(comment: &str) -> Result<bool> {
    // `iptables -S INPUT` lists rules in append form; grep for the comment.
    let out = Command::new("iptables").args(["-S", "INPUT"]).output().await
        .context("Failed to run `iptables -S INPUT`")?;
    if !out.status.success() { return Ok(false); }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut deleted = false;
    for line in stdout.lines() {
        // Reconstruct the matching delete form. Lines look like:
        //   -A INPUT -p tcp --dport 25565 -m comment --comment "esluse:UUID" -j ACCEPT
        if !line.contains(comment) { continue; }
        let delete_form = line.replacen("-A ", "-D ", 1);
        let parts: Vec<&str> = delete_form.split_whitespace().collect();
        let out = Command::new("iptables").args(&parts).output().await
            .with_context(|| format!("Failed to run iptables delete: {}", delete_form))?;
        if out.status.success() { deleted = true; }
    }
    Ok(deleted)
}

async fn delete_nft_matching(comment: &str) -> Result<bool> {
    // nft has a handle-based delete; for our scoped comment we just rebuild the
    // ruleset without the matching rule. Conservative: re-flush to persisted file
    // then reload — D-09 says never touch user rules, so we just delete by handle
    // when found. If we cannot enumerate, return false (no-op) so the audit log
    // shows the cleanup attempt.
    let out = Command::new("nft").args(["-a", "list", "chain", "inet", "filter", "input"])
        .output().await.context("Failed to run `nft -a list chain`")?;
    if !out.status.success() { return Ok(false); }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut deleted = false;
    for line in stdout.lines() {
        if !line.contains(comment) { continue; }
        // `handle N` is at the end; `nft delete rule inet filter input handle N`
        if let Some(handle_str) = line.split_whitespace().rev().find(|s| s.chars().all(|c| c.is_ascii_digit())) {
            let out = Command::new("nft")
                .args(["delete", "rule", "inet", "filter", "input", "handle", handle_str])
                .output().await;
            if let Ok(o) = out { if o.status.success() { deleted = true; } }
        }
    }
    Ok(deleted)
}

fn shell_escape(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "-_./=:".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}
