//! Startup - Runtime detection and capability registration

use agent_capability::{Capability, CapabilityRegistry};
use agent_config::RuntimePreference;
use agent_runtime::RuntimeDetector;
use anyhow::{bail, Result};
use tracing::{info, warn};

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
        }
        RuntimePreference::Podman => {
            if detector.is_podman() && detector.available {
                registry.register(Capability::Podman);
                register_server_capabilities(&mut registry);
                info!(runtime = "podman", version = ?detector.version, "Podman runtime detected and available");
            } else {
                bail!("Podman requested but not available");
            }
        }
        RuntimePreference::None => {
            info!("Runtime disabled - running in SSH-only mode");
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
