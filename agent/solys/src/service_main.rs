//! Windows Service Entry Point for Escluse Agent
//! This binary is used to run the agent as a Windows Service

#[cfg(target_os = "windows")]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    Result as ServiceResult,
};

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "windows")]
use std::sync::Arc;

#[cfg(target_os = "windows")]
use std::time::Duration;

// Service name (Windows only)
#[cfg(target_os = "windows")]
const SERVICE_NAME: &str = "escluse-agent";
#[cfg(target_os = "windows")]
const SERVICE_DISPLAY_NAME: &str = "Escluse Agent";
#[cfg(target_os = "windows")]
const SERVICE_DESCRIPTION: &str = "Background agent for managing Escluse game servers";

#[cfg(target_os = "windows")]
define_windows_service!(ffi_service_main, my_service_main);

#[cfg(target_os = "windows")]
fn my_service_main(arguments: Vec<std::ffi::OsString>) {
    if let Err(e) = run_service(arguments) {
        eprintln!("Service error: {:?}", e);
        std::process::exit(1);
    }
}

#[cfg(target_os = "windows")]
fn run_service(_arguments: Vec<std::ffi::OsString>) -> ServiceResult<()> {
    // Create shutdown signal
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Define service event handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                running_clone.store(false, Ordering::SeqCst);
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register service handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Set service as running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Run the agent core (service mode - headless)
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let config = agent_config::load();
        if let Err(errors) = agent_config::validate(&config) {
            eprintln!("Configuration errors: {:?}", errors);
            return;
        }

        // Run agent core without GUI
        if let Err(e) = run_agent_core_service(config, running.clone()).await {
            eprintln!("Agent error: {:?}", e);
        }
    });

    // Set service as stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

/// Core agent logic for service mode (headless)
#[cfg(target_os = "windows")]
async fn run_agent_core_service(
    config: agent_config::AgentConfig,
    shutdown: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    // Import required modules from main
    use std::path::PathBuf;
    use tracing::{error, info};

    // 1. Setup logging for service mode
    let log_level = config.log_level.parse().unwrap_or(tracing::Level::INFO);

    // Use file logging only (no console in service)
    let log_dir = PathBuf::from("C:\\ProgramData\\Escluse\\logs");
    if !log_dir.exists() {
        let _ = std::fs::create_dir_all(&log_dir);
    }

    if log_dir.exists() {
        let file_appender = tracing_appender::rolling::daily(&log_dir, "agent.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .with_target(false)
            .with_writer(non_blocking)
            .with_ansi(false)
            .try_init()
            .ok();
    }

    info!("Escluse Agent Service starting...");
    info!("Agent name: {}", config.agent_name);
    info!("Backend URL: {}", config.backend_url);

    // 2. Initialize audit logger
    let audit_data_dir = config.data_dir.clone();
    tokio::spawn(async move {
        // audit::init_audit_logger(audit_data_dir).await;
        info!("Audit logger initialized");
    });

    // 3. Runtime detection & capability registration
    let (runtime, capabilities) = startup::detect_runtime(&config.runtime_preference)?;

    info!("Runtime: {:?}", runtime.runtime);
    info!("Capabilities: {:?}", capabilities.to_string_list());

    // 4. Start HTTP API server for GUI communication
    let api_shutdown = shutdown.clone();
    tokio::spawn(async move {
        use crate::api;

        let addr: std::net::SocketAddr = "0.0.0.0:8642".parse().unwrap();
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

    // 5. Create shutdown signal
    let shutdown_for_agent = shutdown.clone();

    // 6. Run agent
    let agent_id = tokio::select! {
        result = agent_connection::run(config, runtime, capabilities, shutdown_for_agent) => {
            result?
        }
        _ = async {
            while !shutdown.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            info!("Shutdown signal received...");
        } => {
            info!("Service shutdown complete");
            uuid::Uuid::nil()
        }
    };

    info!("Agent stopped: {}", agent_id);
    Ok(())
}

/// Modules from main.rs
#[cfg(target_os = "windows")]
mod agent;
#[cfg(target_os = "windows")]
mod agent_connection;
#[cfg(target_os = "windows")]
mod api;
#[cfg(target_os = "windows")]
mod audit;
#[cfg(target_os = "windows")]
mod config_watcher;
#[cfg(target_os = "windows")]
mod crash_reporter;
#[cfg(target_os = "windows")]
mod handlers;
#[cfg(target_os = "windows")]
mod rate_limit;
#[cfg(target_os = "windows")]
mod startup;
#[cfg(target_os = "windows")]
mod state;
#[cfg(target_os = "windows")]
mod task_state;

/// Service installer helper
#[cfg(target_os = "windows")]
pub fn install_service() -> anyhow::Result<()> {
    use std::ffi::OsString;
    use std::path::PathBuf;
    use windows_service::service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType,
    };
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_binary_path = ::std::env::current_exe()?
        .with_file_name("escluse-service.exe");

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, // LocalSystem account
        account_password: None,
    };

    let service = service_manager.create_service(
        &service_info,
        ServiceAccess::CHANGE_CONFIG | ServiceAccess::START,
    )?;

    println!("Service '{}' installed successfully.", SERVICE_NAME);
    println!("Run 'escluse-service.exe --start' to start the service.");

    Ok(())
}

/// Service uninstaller
#[cfg(target_os = "windows")]
pub fn uninstall_service() -> anyhow::Result<()> {
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // Stop the service if running
    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // Delete the service
    service.delete()?;
    println!("Service '{}' uninstalled successfully.", SERVICE_NAME);

    Ok(())
}

/// Start the service
#[cfg(target_os = "windows")]
pub fn start_service() -> anyhow::Result<()> {
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = service_manager.open_service(SERVICE_NAME, ServiceAccess::START)?;
    service.start::<&str>(&[])?;

    println!("Service '{}' started.", SERVICE_NAME);
    Ok(())
}

/// Stop the service
#[cfg(target_os = "windows")]
pub fn stop_service() -> anyhow::Result<()> {
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = service_manager.open_service(SERVICE_NAME, ServiceAccess::STOP)?;
    service.stop()?;

    println!("Service '{}' stopped.", SERVICE_NAME);
    Ok(())
}

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--install" => return install_service(),
            "--uninstall" => return uninstall_service(),
            "--start" => return start_service(),
            "--stop" => return stop_service(),
            _ => {}
        }
    }

    // Run as Windows service
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This binary is for Windows only.");
    std::process::exit(1);
}
