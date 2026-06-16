use std::io::{self, BufRead};
use std::path::PathBuf;

use chrono::Local;
use clap::Parser;

#[derive(Parser)]
#[command(name = "logsum", about = "Filter and summarize Escluse Agent logs")]
struct Args {
    /// Log file to read (reads from stdin if not provided)
    file: Option<PathBuf>,

    /// Show all lines including noisy internal ones
    #[arg(short, long)]
    verbose: bool,

    /// Print aggregate summary instead of filtered lines
    #[arg(short, long)]
    summary: bool,

    /// Follow mode (tail -f) — reads from end of file
    #[arg(short, long)]
    follow: bool,
}

fn main() {
    let args = Args::parse();

    let reader: Box<dyn BufRead> = if let Some(path) = &args.file {
        let file = std::fs::File::open(path).unwrap_or_else(|e| {
            eprintln!("Error: cannot open {}: {}", path.display(), e);
            std::process::exit(1);
        });
        Box::new(io::BufReader::new(file))
    } else {
        Box::new(io::BufReader::new(io::stdin()))
    };

    if args.summary {
        run_summary(reader);
    } else {
        run_filter(reader, args.verbose, args.follow);
    }
}

fn run_filter(reader: Box<dyn BufRead>, verbose: bool, _follow: bool) {
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = redact(&line);

        if verbose || is_useful(&line) {
            println!("{}", line);
        }
    }
}

fn run_summary(reader: Box<dyn BufRead>) {
    let mut total = 0u64;
    let mut shown = 0u64;
    let mut suppressed_noisy = 0u64;

    #[derive(Default)]
    struct Counts {
        total: u64,
    }

    let mut agent_registered = Counts::default();
    let mut ws_connected = Counts::default();
    let mut ws_disconnected = Counts::default();
    let mut tunnel_connected = Counts::default();
    let mut tunnel_disconnected = Counts::default();
    let mut config_sync = Counts::default();
    let mut tunnel_started = Counts::default();
    let mut tunnel_stopped = Counts::default();
    let mut player_connected = Counts::default();
    let mut player_disconnected = Counts::default();
    let mut errors = Counts::default();
    let mut warnings = Counts::default();
    let mut heartbeats_suppressed = Counts::default();
    let mut metrics_suppressed = Counts::default();
    let mut dns_ops_suppressed = Counts::default();
    let mut yamux_suppressed = Counts::default();
    let mut container_resolve_suppressed = Counts::default();
    let mut log_poll_suppressed = Counts::default();
    let mut writer_suppressed = Counts::default();
    let mut reconnect_suppressed = Counts::default();
    let mut ddns_suppressed = Counts::default();
    let mut task_result_suppressed = Counts::default();
    let mut connectivity_suppressed = Counts::default();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        total += 1;

        let line = redact(&line);

        let line_level = extract_level(&line);

        if line_level == Some("ERROR") {
            errors.total += 1;
        }
        if line_level == Some("WARN") {
            warnings.total += 1;
        }

        if is_useful(&line) {
            shown += 1;

            if line.contains("Agent registered") {
                agent_registered.total += 1;
            }
            if line.contains("WebSocket connected") {
                ws_connected.total += 1;
            }
            if line.contains("WebSocket disconnected") {
                ws_disconnected.total += 1;
            }
            if line.contains("tunnel connected") && !line.contains("disconnected") {
                tunnel_connected.total += 1;
            }
            if line.contains("tunnel disconnected") {
                tunnel_disconnected.total += 1;
            }
            if line.contains("RelayConfigSync received") {
                config_sync.total += 1;
            }
            if line.contains("RelayManager: started tunnel") {
                tunnel_started.total += 1;
            }
            if line.contains("RelayManager: stopping tunnel") {
                tunnel_stopped.total += 1;
            }
            if line.contains("Player connected") {
                player_connected.total += 1;
            }
            if line.contains("Player disconnected") {
                player_disconnected.total += 1;
            }
        } else {
            suppressed_noisy += 1;

            if line.contains("RELAY_TUNNEL_AUDIT") || line.contains("heartbeat") {
                heartbeats_suppressed.total += 1;
            }
            if line.contains("Collecting metrics") || line.contains("Getting metrics") {
                metrics_suppressed.total += 1;
            }
            if line.contains("DNS record") || line.contains("remove_cname_record") {
                dns_ops_suppressed.total += 1;
            }
            if line.contains("yamux session") || line.contains("yamux session stream error") {
                yamux_suppressed.total += 1;
            }
            if line.contains("resolved container address") {
                container_resolve_suppressed.total += 1;
            }
            if line.contains("polling fallback")
                || line.contains("No logs for 30 seconds")
                || line.contains("Log stream ended")
                || line.contains("Getting container logs")
                || line.contains("is not running, stopping log")
                || line.contains("not found or inaccessible")
            {
                log_poll_suppressed.total += 1;
            }
            if line.contains("Writer exiting") || line.contains("Writer:") {
                writer_suppressed.total += 1;
            }
            if line.contains("reconnect loop")
                || line.contains("Reconnecting in")
                || line.contains("reconnect_attempt")
            {
                reconnect_suppressed.total += 1;
            }
            if line.contains("DDNS cycle complete") {
                ddns_suppressed.total += 1;
            }
            if line.contains("Task result")
                || line.contains("Flushed buffered")
            {
                task_result_suppressed.total += 1;
            }
            if line.contains("CONNECTIVITY_TRIGGER")
                || line.contains("ConnectivityReport dropped")
                || line.contains("ConnectivityMonitor started")
            {
                connectivity_suppressed.total += 1;
            }
        }
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("══════════════════════════════════════════");
    println!("  Agent Log Summary  —  {}", now);
    println!("══════════════════════════════════════════");
    println!("  Total lines:         {:>8}", total);
    println!("  Shown (useful):      {:>8}", shown);
    println!("  Suppressed (noisy):  {:>8}", suppressed_noisy);
    println!("────────────────────────────────────────────");
    println!("  Agent registered     {:>8}", agent_registered.total);
    println!("  WebSocket connected  {:>8}", ws_connected.total);
    println!("  WebSocket disconnect {:>8}", ws_disconnected.total);
    println!("  Tunnel connected     {:>8}", tunnel_connected.total);
    println!("  Tunnel disconnected  {:>8}", tunnel_disconnected.total);
    println!("  Config syncs         {:>8}", config_sync.total);
    println!("  Tunnel starts        {:>8}", tunnel_started.total);
    println!("  Tunnel stops         {:>8}", tunnel_stopped.total);
    println!("  Player connected     {:>8}", player_connected.total);
    println!("  Player disconnected  {:>8}", player_disconnected.total);
    println!("  Errors               {:>8}", errors.total);
    println!("  Warnings             {:>8}", warnings.total);
    println!("────────────────────────────────────────────");
    println!("  Suppressed by type:");
    println!("    Heartbeat audit    {:>8}", heartbeats_suppressed.total);
    println!("    Metrics collection {:>8}", metrics_suppressed.total);
    println!("    DNS operations     {:>8}", dns_ops_suppressed.total);
    println!("    Yamux session mgmt {:>8}", yamux_suppressed.total);
    println!("    Container addr res {:>8}", container_resolve_suppressed.total);
    println!("    Log polling        {:>8}", log_poll_suppressed.total);
    println!("    Writer events      {:>8}", writer_suppressed.total);
    println!("    Reconnect attempts {:>8}", reconnect_suppressed.total);
    println!("    DDNS cycles        {:>8}", ddns_suppressed.total);
    println!("    Task results       {:>8}", task_result_suppressed.total);
    println!("    Connectivity polls {:>8}", connectivity_suppressed.total);
    println!("══════════════════════════════════════════");
}

/// Check if a log line is useful for the user (as opposed to noisy internal)
fn is_useful(line: &str) -> bool {
    // Known noisy patterns — hide regardless of level
    let noisy = [
        "RELAY_TUNNEL_AUDIT",
        "yamux session",
        "Collecting metrics",
        "resolved container address",
        "DNS record",
        "DDNS cycle complete",
        "remove_cname_record",
        "Public IP changed",
        "CONNECTIVITY_TRIGGER",
        "Writer exiting",
        "Writer:",
        "polling fallback",
        "No logs for 30 seconds",
        "Starting polling fallback",
        "Log stream ended",
        "Getting container logs",
        "is not running, stopping log",
        "not found or inaccessible",
        "Task result sent immediately",
        "Task result buffered",
        "Flushed buffered results",
        "Heartbeat channel send timed out",
        "RECEIVED TEXT MESSAGE",
        "[DEBUG] Received message type",
        "[OUTBOUND] Payload type",
        "Getting metrics",
        "DnsWatcher",
        "ConnectivityReport dropped",
        "ConnectivityMonitor started",
        "DNS not configured yet",
        "Auto-refresh disabled",
        "Retrying task after error",
        "Resolved container IP for RCON",
        "reconnect loop",
        "Reconnecting in",
        "DNS configured for zone",
    ];

    for pattern in &noisy {
        if line.contains(pattern) {
            return false;
        }
    }

    // Always show ERROR level lines
    if line.contains(" ERROR ") || line.contains(" ERROR:") || line.contains(" ERROR\t") {
        return true;
    }

    // Specific useful patterns
    if line.contains("Agent registered") { return true; }
    if line.contains("WebSocket connected") { return true; }
    if line.contains("WebSocket disconnected") { return true; }
    if line.contains("Relay tunnel connected") { return true; }
    if line.contains("relay tunnel disconnected") { return true; }
    if line.contains("RelayConfigSync received") { return true; }
    if line.contains("RelayManager:") { return true; }
    if line.contains("Inner loop exited") { return true; }
    if line.contains("Failed to connect") { return true; }
    if line.contains("Connection closed") { return true; }
    if line.contains("Player connected") || line.contains("Player disconnected") { return true; }

    // WARN or INFO lines that aren't noisy — show them
    if line.contains(" WARN ") || line.contains(" INFO ") {
        return true;
    }

    // DEBUG/TRACE — always hide by default
    false
}

/// Extract log level from a tracing-formatted line
fn extract_level<'a>(line: &'a str) -> Option<&'a str> {
    // Format: "YYYY-MM-DDTHH:MM:SS.ssssssZ  LEVEL ..."
    // Timestamp and level separated by 2+ spaces due to right-padding
    let level = line.split_whitespace().nth(1)?;
    match level {
        "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE" => Some(level),
        _ => None,
    }
}

/// Redact sensitive information from a log line
fn redact(line: &str) -> String {
    // Redact IPv4 addresses (but not in already-redacted "from=..." patterns)
    // Match standalone IPs: sequences of 4 groups of 1-3 digits separated by dots
    let ip_re = regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
    ip_re.replace_all(line, "***").to_string()
}
