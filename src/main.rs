//! Web Agent - Entry point with config loading, runtime detection, subsystems start

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tokio::signal;
use tracing::{error, info};
use uuid::Uuid;

use crate::state::{AgentMetadata, AgentState};

mod agent;
mod agent_connection;
mod api;
mod audit;
mod config_watcher;
mod crash_reporter;
mod handlers;
mod rate_limit;
mod startup;
mod state;  // D-19: State persistence module
mod task_state;

#[cfg(target_os = "windows")]
mod gui;

#[tokio::main]
async fn main() -> Result<()> {
    // Check command line arguments for mode
    let args: Vec<String> = std::env::args().collect();

    // Early return for --help/--version: avoid panic from logging init
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Escluse Agent - Game server management agent");
        println!();
        println!("USAGE:");
        println!("  escluse-agent [FLAGS]");
        println!();
        println!("FLAGS:");
        println!("  --help, -h       Prints help information");
        println!("  --version, -V    Prints version information");
        println!("  --service, -s    Run as Windows service (Windows only)");
        println!("  --quiet, -q      Log to file only, no terminal output");
        println!();
        println!("CONFIGURATION:");
        println!("  Config file: ~/.config/escluse-agent/config.toml");
        println!("  Environment:  ESCLUSE_AGENT_* or AGENT_* env vars");
        println!();
        println!("  Required:");
        println!("    backend_url    WebSocket URL (e.g. wss://app.esluce.com/api/ws/node)");
        println!("    api_key        API key from Escluse Dashboard");
        return Ok(());
    }

    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        println!("Escluse Agent v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let _is_service_mode = args.contains(&"--service".to_string()) || args.contains(&"-s".to_string());
    
    // On Windows, default to GUI mode unless --service flag is provided
    #[cfg(target_os = "windows")]
    {
        if !_is_service_mode {
            // GUI mode: Run agent in background and show GUI
            let config = agent_config::load();
            if let Err(errors) = agent_config::validate(&config) {
                // Show error dialog for GUI mode
                gui::show_notification("Escluse Agent Error", &format!("Configuration error: {:?}", errors));
                std::process::exit(1);
            }
            
            // Create agent future that will be spawned
            let agent_future = Box::pin(run_agent_core(config));
            return gui::run_gui_mode(agent_future).await;
        }
    }
    
    // Service mode (or non-Windows): Run agent directly
    let config = agent_config::load();
    if let Err(errors) = agent_config::validate(&config) {
        error!(?errors, "Configuration validation failed");
        for err in &errors {
            eprintln!("  - {}", err);
        }
        std::process::exit(1);
    }
    
    run_agent_core(config).await
}

/// Core agent logic (extracted from original main)
async fn run_agent_core(config: agent_config::AgentConfig) -> Result<()> {
    // 1. Setup logging (D-06, D-07, D-08)
    let log_level = config.log_level.parse().unwrap_or(tracing::Level::INFO);
    let quiet = std::env::args().any(|a| a == "--quiet" || a == "-q");

    // D-18: Set up panic handler for production - log error instead of panic
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        // Log error instead of letting it propagate
        eprintln!("PANIC at {}: {}", location, msg);

        // D-18: Exit with specific code for monitoring
        std::process::exit(1);
    }));

    // Setup logging
    // Default: log to both stdout AND file (terminal for interactivity, file for persistence)
    // --quiet / -q: log to file only (headless/daemon)
    use tracing_subscriber::{
        Registry, layer::SubscriberExt, Layer,
        filter::EnvFilter,
    };

    // Build filter: respect RUST_LOG env var, fall back to configured log_level
    let log_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level.to_string().to_lowercase()));

    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = Vec::new();

    // Stdout layer (default: on; disable with --quiet)
    if !quiet {
        layers.push(Box::new(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(false)
                .with_filter(log_filter.clone()),
        ));
    }

    // Try file logging
    // Primary: /var/log/escluse-agent/ (D-06)
    // Fallback: ~/.local/share/escluse-agent/logs/ (D-07)
    let log_dir = if PathBuf::from("/var/log/escluse-agent").exists()
        || std::fs::create_dir_all("/var/log/escluse-agent").is_ok()
    {
        let d = PathBuf::from("/var/log/escluse-agent");
        if std::fs::File::create(d.join(".writable")).is_ok() {
            let _ = std::fs::remove_file(d.join(".writable"));
            Some(d)
        } else {
            None
        }
    } else {
        None
    };

    let log_dir = log_dir.or_else(|| {
        dirs::data_local_dir().map(|d| d.join("escluse-agent").join("logs"))
    });

    let mut _file_guard: Option<std::mem::ManuallyDrop<_>> = None;

    if let Some(ref dir) = log_dir {
        if std::fs::create_dir_all(dir).is_ok() {
            let file_appender = tracing_appender::rolling::daily(dir, "agent.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            layers.push(Box::new(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_filter(log_filter.clone()),
            ));

            _file_guard = Some(std::mem::ManuallyDrop::new(guard));
        }
    }

    let subscriber = Registry::default().with(layers);
    let _ = tracing::subscriber::set_global_default(subscriber);

    if !quiet {
        info!("stdout logging enabled (use --quiet for headless mode)");
    }
    if log_dir.is_some() {
        info!("File logging initialized");
    }

    info!(
        agent_name = %config.agent_name,
        backend_url = %config.backend_url,
        "Starting Web Agent"
    );

    // 3. Initialize audit logger
    let audit_data_dir = config.data_dir.clone();
    tokio::spawn(async move {
        crate::audit::init_audit_logger(audit_data_dir).await;
        info!("Audit logger initialized");
    });

    // 4. Initialize config watcher (if enabled via env var)
    if std::env::var("AGENT_CONFIG_WATCH").unwrap_or_default() == "true" {
        let config_path = std::env::var("AGENT_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config.data_dir.join("config.json"));
        
        if config_path.exists() {
            let watcher = crate::config_watcher::ConfigWatcher::new(
                config_path,
                |change| {
                    info!("Config changed: {:?}", change);
                },
            );
            
        match watcher {
            Ok(_w) => info!("Config watcher started"),
            Err(e) => error!(error = %e, "Failed to start config watcher"),
        }
        }
    }

    // 5. Runtime detection & capability registration
    let (runtime, capabilities) = startup::detect_runtime(&config.runtime_preference)?;

    info!("Runtime: {:?}", runtime.runtime);
    info!("Capabilities: {:?}", capabilities.to_string_list());

    // Phase 67: expose the Docker client as a process-global so the
    // connectivity diagnostics collector can `inspect_container` the
    // server's container without going through the agent_connection
    // task routing.
    if let Some(docker) = runtime.docker() {
        crate::state::set_docker_global(Arc::new(docker.clone()));
    }

    // Phase 67: wire the connectivity orchestrator to the WebSocket outbound.
    // The actual outbound send lives in `agent_connection.rs`; until that
    // hook is exposed, the orchestrator's OUTBOUND_TX stays None and reports
    // are audit-logged only.
    let tx_handle: Arc<dyn Fn(serde_json::Value) + Send + Sync> = Arc::new(move |payload| {
        let text = serde_json::to_string(&payload).unwrap_or_default();
        tracing::trace!("[OUTBOUND] Payload type: {}", payload.get("type").and_then(|v| v.as_str()).unwrap_or("unknown"));
        eprintln!("[DEBUG] Outbound payload: {}", crate::agent_connection::redact_json(&text));
    });
    crate::handlers::connectivity::set_outbound_sender(tx_handle);

    // D-23: Auto-recovery step 1 - Load persisted state
    let initial_state = state::load_state().await;
    if let Some(loaded_state) = &initial_state {
        info!(
            servers = loaded_state.servers.len(),
            containers = loaded_state.container_map.len(),
            restart_count = loaded_state.metadata.restart_count,
            "Loaded persisted state for auto-recovery"
        );
    }

    // 6. Create shutdown signal
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    // 7. Set up signal handlers with logging
    tokio::spawn(async move {
        let result = signal::ctrl_c().await;
        if result.is_ok() {
            // D-18: Log error instead of panic
            error!("Received Ctrl-C, initiating graceful shutdown...");
            shutdown_clone.store(true, Ordering::Relaxed);
        }
    });

    // 8. Start HTTP API server for GUI communication
    let api_shutdown = shutdown.clone();
    tokio::spawn(async move {
        let addr: std::net::SocketAddr = "127.0.0.1:8642".parse().unwrap();
        let router = api::routes::create_router();

        info!("Starting HTTP API server on {}", addr);

        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                axum::serve(listener, router)
                    .with_graceful_shutdown(async move {
                        while !api_shutdown.load(Ordering::Relaxed) {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                        info!("HTTP API server shutting down");
                    })
                    .await
                    .ok();
            }
            Err(e) => {
                error!("Failed to start HTTP API server: {}", e);
            }
        }
    });

    // 9. Start DNS watcher (DDNS-like auto-refresh)
    let dns_watcher = Arc::new(handlers::dns_watch::DnsWatcher::new());
    dns_watcher.start().await;

    let watcher_for_shutdown = dns_watcher.clone();
    let shutdown_clone2 = shutdown.clone();
    tokio::spawn(async move {
        while !shutdown_clone2.load(Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        watcher_for_shutdown.stop().await;
    });

    // 9b. Start Connectivity Monitor (Phase 67 D-04 periodic re-collect).
    //     Mirrors the DnsWatcher shutdown pattern above.
    use crate::handlers::connectivity::ConnectivityMonitor;
    let connectivity_monitor = Arc::new(ConnectivityMonitor::new());
    connectivity_monitor.start().await;

    let monitor_for_shutdown = connectivity_monitor.clone();
    let shutdown_clone3 = shutdown.clone();
    tokio::spawn(async move {
        while !shutdown_clone3.load(Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        monitor_for_shutdown.stop().await;
    });

    // 10. Run agent with shutdown handling
    let shutdown_for_agent = shutdown.clone();
    let agent_id = tokio::select! {
        result = agent_connection::run(config, runtime, capabilities, shutdown_for_agent) => {
            result?
        }
        _ = async {
            while !shutdown.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            info!("Shutdown signal received, waiting for agent to finish...");
        } => {
            info!("Shutdown complete");
            Uuid::nil()
        }
    };

    info!(agent_id = %agent_id, "Web Agent stopped");

    // Phase 70: shut down all relay tunnels
    crate::state::relay_manager().stop_all().await;

    // D-23: Save final state on shutdown
    // Capture any error from the agent run
    let final_state = AgentState {
        servers: vec![],  // Would be populated from active agent state
        container_map: std::collections::HashMap::new(),
        metadata: AgentMetadata {
            restart_count: 0,
            last_start: Some(chrono::Utc::now()),
            last_error: None,
        },
    };

    if let Err(e) = state::save_state(&final_state).await {
        error!(error = %e, "Failed to save state on shutdown");
    } else {
        info!("State saved on shutdown");
    }

    Ok(())
}

