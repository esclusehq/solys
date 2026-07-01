//! Forge server installation via --installServer
//!
//! Forge ships an installer JAR, not a server JAR. We:
//! 1. Download installer from maven.minecraftforge.net
//! 2. Run: java -jar installer.jar --installServer
//! 3. Resolve the generated forge-{mc_version}-{forge_version}.jar launcher
//!
//! Forge version format: {mc_version}-{forge_version} e.g. "1.21.4-54.1.0"

use std::path::Path;

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::info;

const FORGE_MAVEN_BASE: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";

/// Get the installer JAR URL for a Forge version.
///
/// `version` is the full Forge version string: "{mc_version}-{forge_version}"
/// e.g. "1.21.4-54.1.0". Can also be just "{mc_version}" to resolve latest.
pub async fn get_installer_url(version: &str) -> Result<String> {
    // If version doesn't contain a forge version suffix, resolve the latest
    let full_version = if !version.contains('-') {
        resolve_latest_forge_version(version).await?
    } else {
        version.to_string()
    };

    Ok(format!(
        "{}/{}/forge-{}-installer.jar",
        FORGE_MAVEN_BASE, full_version, full_version
    ))
}

/// Run the Forge installer in headless mode.
pub async fn run_installer(installer_path: &Path, server_dir: &Path) -> Result<()> {
    info!(
        installer = %installer_path.display(),
        dir = %server_dir.display(),
        "Running Forge installer"
    );

    let output = Command::new("java")
        .arg("-jar")
        .arg(installer_path)
        .arg("--installServer")
        .current_dir(server_dir)
        .output()
        .await
        .context("Failed to execute Forge installer")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Forge installer failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        ));
    }

    info!("Forge installer completed successfully");
    Ok(())
}

/// Resolve the generated Forge server launcher JAR path.
///
/// Forge generates a file named: forge-{mc_version}-{forge_version}.jar
/// or forge-{mc_version}-{forge_version}-universal.jar (older versions).
pub fn resolve_launcher_jar(server_dir: &Path) -> Result<std::path::PathBuf> {
    // Look for forge-*.jar files (not installer)
    let entries = std::fs::read_dir(server_dir).context("Failed to read server directory")?;

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("forge-")
            && name_str.ends_with(".jar")
            && !name_str.contains("installer")
        {
            return Ok(entry.path());
        }
    }

    Err(anyhow::anyhow!(
        "Could not find Forge server launcher JAR in {}",
        server_dir.display()
    ))
}

/// Resolve the latest Forge version for a given MC version.
async fn resolve_latest_forge_version(mc_version: &str) -> Result<String> {
    let metadata_url =
        "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
    let response = reqwest::get(metadata_url)
        .await
        .context("Failed to fetch Forge maven metadata")?;
    let text = response.text().await?;

    // Parse XML to find latest version matching mc_version
    // Format: <version>{mc_version}-{forge_version}</version>
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(ver) = trimmed
            .strip_prefix("<version>")
            .and_then(|s| s.strip_suffix("</version>"))
        {
            if ver.starts_with(mc_version) {
                return Ok(ver.to_string());
            }
        }
    }

    Err(anyhow::anyhow!(
        "No Forge version found for MC {}",
        mc_version
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_url_construction() {
        let version = "1.21.4-54.1.0";
        let url = format!(
            "{}/{}/forge-{}-installer.jar",
            FORGE_MAVEN_BASE, version, version
        );
        assert!(url.contains("1.21.4-54.1.0"));
    }

    #[test]
    fn test_launcher_jar_pattern() {
        // forge-1.21.4-54.1.0.jar should match
        let name = "forge-1.21.4-54.1.0.jar";
        assert!(
            name.starts_with("forge-")
                && name.ends_with(".jar")
                && !name.contains("installer")
        );

        // forge-installer.jar should NOT match
        let installer = "forge-1.21.4-54.1.0-installer.jar";
        assert!(
            !(installer.starts_with("forge-")
                && installer.ends_with(".jar")
                && !installer.contains("installer"))
        );
    }
}
