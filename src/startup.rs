//! Startup - Runtime detection and capability registration

use agent_capability::{Capability, CapabilityRegistry};
use agent_config::RuntimePreference;
use agent_runtime::RuntimeDetector;
use anyhow::{bail, Result};
use tracing::{info, warn};

use crate::handlers::direct_executor::java;

pub fn detect_runtime(
    preference: &RuntimePreference,
) -> Result<(RuntimeDetector, CapabilityRegistry)> {
    let detector = RuntimeDetector::detect();

    let mut registry = CapabilityRegistry::new();

    match preference {
        RuntimePreference::Docker => {
            if detector.is_docker() && detector.available {
                registry.register(Capability::Docker);
                register_server_capabilities(&mut registry);
                info!(runtime = "docker", version = ?detector.version, "Docker runtime detected and available");
            } else {
                bail!("Docker requested but not available");
            }
            // Also register DirectExecutor if Java is available
            if let Some((major, version)) = java::detect_java_version() {
                if major >= 17 {
                    registry.register(Capability::DirectExecutor);
                    register_direct_server_capabilities(&mut registry);
                    info!(java_major = major, java_version = %version, "DirectExecutor capability registered");
                } else {
                    info!(java_major = major, "Java detected but version < 17, DirectExecutor not available");
                }
            } else {
                info!("Java not found on PATH, DirectExecutor not available");
            }
        }
        RuntimePreference::Podman => {
            if detector.is_podman() && detector.available {
                registry.register(Capability::Podman);
                register_server_capabilities(&mut registry);
                info!(runtime = "podman", version = ?detector.version, "Podman runtime detected and available");
            } else {
                bail!("Podman requested but not available");
            }
            // Also register DirectExecutor if Java is available
            if let Some((major, version)) = java::detect_java_version() {
                if major >= 17 {
                    registry.register(Capability::DirectExecutor);
                    register_direct_server_capabilities(&mut registry);
                    info!(java_major = major, java_version = %version, "DirectExecutor capability registered");
                } else {
                    info!(java_major = major, "Java detected but version < 17, DirectExecutor not available");
                }
            } else {
                info!("Java not found on PATH, DirectExecutor not available");
            }
        }
        RuntimePreference::None => {
            info!("Runtime disabled - running in SSH-only mode");
            // Also register DirectExecutor if Java is available (D-08, D-09)
            if let Some((major, version)) = java::detect_java_version() {
                if major >= 17 {
                    registry.register(Capability::DirectExecutor);
                    register_direct_server_capabilities(&mut registry);
                    info!(java_major = major, java_version = %version, "DirectExecutor capability registered");
                } else {
                    info!(java_major = major, "Java detected but version < 17, DirectExecutor not available");
                }
            } else {
                info!("Java not found on PATH, DirectExecutor not available");
            }
        }
        RuntimePreference::Auto => {
            if detector.is_docker() && detector.available {
                registry.register(Capability::Docker);
                register_server_capabilities(&mut registry);
                info!(runtime = "docker", version = ?detector.version, "Docker runtime auto-detected");
            } else if detector.is_podman() && detector.available {
                registry.register(Capability::Podman);
                register_server_capabilities(&mut registry);
                info!(runtime = "podman", version = ?detector.version, "Podman runtime auto-detected");
            } else {
                warn!("No container runtime detected - running in SSH-only mode");
            }
            // After container runtime detection, also detect Java for DirectExecutor
            if let Some((major, version)) = java::detect_java_version() {
                if major >= 17 {
                    registry.register(Capability::DirectExecutor);
                    register_direct_server_capabilities(&mut registry);
                    info!(java_major = major, java_version = %version, "DirectExecutor capability registered (auto mode)");
                } else {
                    info!(java_major = major, "Java detected but version < 17, DirectExecutor not available");
                }
            } else {
                info!("Java not found on PATH, DirectExecutor not available");
            }
        }
    }

    // Always register metrics capability
    registry.register(Capability::Metrics);

    // Always register SSH/SFTP capabilities (available even without Docker)
    registry.register(Capability::SSH);
    registry.register(Capability::SFTP);

    Ok((detector, registry))
}

fn register_server_capabilities(registry: &mut CapabilityRegistry) {
    registry.register(Capability::ServerCreate);
    registry.register(Capability::ServerStart);
    registry.register(Capability::ServerStop);
    registry.register(Capability::ServerRestart);
    registry.register(Capability::ServerDelete);
    registry.register(Capability::ServerLogs);
    registry.register(Capability::ServerCommand);
    registry.register(Capability::BackupCreate);
    registry.register(Capability::BackupRestore);
}

fn register_direct_server_capabilities(registry: &mut CapabilityRegistry) {
    registry.register(Capability::ServerCreate);
    registry.register(Capability::ServerStart);
    registry.register(Capability::ServerStop);
    registry.register(Capability::ServerRestart);
    registry.register(Capability::ServerDelete);
    registry.register(Capability::ServerLogs);
    registry.register(Capability::BackupCreate);
    registry.register(Capability::BackupRestore);
}
