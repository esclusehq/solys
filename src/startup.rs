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

/// Auto-install Java on Termux if not already available.
///
/// This is intended to be spawned as a background task from `main.rs`.
/// It checks whether we're running in Termux, and if Java is not found,
/// runs `pkg install openjdk-17 -y` and verifies the result.
pub async fn auto_install_java_if_termux() {
    let is_termux = std::env::var("PREFIX")
        .map(|p| p.contains("com.termux"))
        .unwrap_or(false);

    if !is_termux {
        return;
    }

    info!("Termux detected, checking for Java...");

    if java::detect_java_version().is_some() {
        info!("Java already available on Termux");
        return;
    }

    info!("Java not found, installing openjdk-17 via pkg (this may take a while)...");

    let status = tokio::process::Command::new("pkg")
        .args(["install", "openjdk-17", "-y"])
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            info!("openjdk-17 installed successfully via pkg");
            if java::detect_java_version().is_some() {
                info!("Java verified after installation — DirectExecutor will be available on next Start Server command");
            } else {
                warn!("pkg install succeeded but java not found on PATH — try restarting the agent");
            }
        }
        Ok(s) => {
            warn!("pkg install failed with exit code: {:?}", s.code());
        }
        Err(e) => {
            warn!("Failed to run pkg install: {}", e);
        }
    }
}

/// Auto-install Docker and Java 17+ on Linux.
/// Detects apt/dnf/pacman and installs both if missing.
pub async fn auto_install_prerequisites_linux() {
    if cfg!(not(target_os = "linux")) {
        return;
    }

    let has_java = java::detect_java_version().is_some();
    let has_docker = std::process::Command::new("docker")
        .arg("version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_java && has_docker {
        info!("Linux: Docker and Java already available");
        return;
    }

    if !has_docker {
        info!("Linux: Docker not found, attempting auto-install...");
    }
    if !has_java {
        info!("Linux: Java 17+ not found, attempting auto-install...");
    }

    let manager = if std::process::Command::new("apt-get").arg("--version").output().ok().map(|o| o.status.success()).unwrap_or(false) {
        Some(("apt-get", "apt-get", vec!["install", "-y", "docker.io", "openjdk-17-jdk"]))
    } else if std::process::Command::new("dnf").arg("--version").output().ok().map(|o| o.status.success()).unwrap_or(false) {
        Some(("dnf", "dnf", vec!["install", "-y", "docker", "java-17-openjdk"]))
    } else if std::process::Command::new("pacman").arg("--version").output().ok().map(|o| o.status.success()).unwrap_or(false) {
        Some(("pacman", "pacman", vec!["-S", "--noconfirm", "docker", "jdk17-openjdk"]))
    } else if std::process::Command::new("zypper").arg("--version").output().ok().map(|o| o.status.success()).unwrap_or(false) {
        Some(("zypper", "zypper", vec!["install", "-y", "docker", "java-17-openjdk"]))
    } else {
        None
    };

    if let Some((name, cmd, args)) = manager {
        info!("Linux: Installing Docker + Java via {} (may request sudo)...", name);
        let status = tokio::process::Command::new("sudo")
            .args(std::iter::once(cmd).chain(args.iter().copied()))
            .status()
            .await;

        match status {
            Ok(s) if s.success() => {
                info!("Linux: Docker + Java installed successfully via {}", name);
                let _ = tokio::process::Command::new("sudo")
                    .args(["usermod", "-aG", "docker", &whoami()])
                    .status().await;
            }
            Ok(s) => warn!("Linux: {} install failed (exit: {:?})", name, s.code()),
            Err(e) => warn!("Linux: failed to run {}: {}", name, e),
        }
    } else {
        warn!("Linux: no supported package manager found — install Docker + Java manually");
    }
}

#[cfg(target_os = "linux")]
fn whoami() -> String {
    std::process::Command::new("whoami")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "root".to_string())
}

#[cfg(not(target_os = "linux"))]
fn whoami() -> String {
    "root".to_string()
}

/// Auto-install Docker Desktop and Java 17+ on Windows via winget/choco.
pub async fn auto_install_prerequisites_windows() {
    if cfg!(not(target_os = "windows")) {
        return;
    }

    let has_java = java::detect_java_version().is_some();
    let has_docker = std::process::Command::new("docker")
        .arg("version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_java && has_docker {
        info!("Windows: Docker and Java already available");
        return;
    }

    let winget_available = std::process::Command::new("winget")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if winget_available {
        info!("Windows: winget detected, attempting auto-install...");

        if !has_docker {
            info!("Windows: installing Docker Desktop via winget...");
            let status = tokio::process::Command::new("winget")
                .args(["install", "--id", "Docker.DockerDesktop", "--accept-source-agreements", "--silent"])
                .status().await;
            match status {
                Ok(s) if s.success() => info!("Windows: Docker Desktop installed via winget"),
                Ok(s) => warn!("Windows: Docker Desktop winget install exited: {:?}", s.code()),
                Err(e) => warn!("Windows: failed to run winget install Docker: {}", e),
            }
        }

        if !has_java {
            info!("Windows: installing Eclipse Temurin JDK 17 via winget...");
            let status = tokio::process::Command::new("winget")
                .args(["install", "--id", "EclipseAdoptium.Temurin.17.JDK", "--accept-source-agreements", "--silent"])
                .status().await;
            match status {
                Ok(s) if s.success() => info!("Windows: Temurin JDK 17 installed via winget"),
                Ok(s) => warn!("Windows: Temurin JDK 17 winget install exited: {:?}", s.code()),
                Err(e) => warn!("Windows: failed to run winget install Java: {}", e),
            }
        }
    } else {
        warn!("Windows: winget not available — install Docker Desktop and Java 17 manually");
    }
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
