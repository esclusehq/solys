//! NeoForge server installation via --installServer
//!
//! NeoForge follows the same pattern as Forge (installer JAR → --installServer).
//! The generated server launcher is in libraries/net/neoforged/neoforge/
//! and run.sh provides the launch command.
//!
//! NeoForge version format: {major}.{minor}.{patch} e.g. "21.4.54-beta"
//! For MC 1.21+, the version prefix is 21.{minor}.{patch}.

use std::path::Path;

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::info;

const NEOFORGE_MAVEN_BASE: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge";

/// Get the installer JAR URL for a NeoForge version.
///
/// `version` is either a full NeoForge version (e.g. "21.4.54-beta")
/// or a Minecraft version (e.g. "1.21.4") — if it looks like an MC version,
/// the latest matching NeoForge build is resolved from Maven metadata.
pub async fn get_installer_url(version: &str) -> Result<String> {
    let neoforge_version = if version.starts_with("1.") {
        resolve_latest_neoforge_version(version).await?
    } else {
        version.to_string()
    };
    Ok(format!(
        "{}/{}/neoforge-{}-installer.jar",
        NEOFORGE_MAVEN_BASE, neoforge_version, neoforge_version
    ))
}

/// Resolve the latest NeoForge build for a given Minecraft version.
///
/// Strips the "1." prefix from the MC version (e.g. "1.21.4" → "21.4"),
/// then searches the NeoForge Maven metadata for the newest version
/// starting with that prefix.
async fn resolve_latest_neoforge_version(mc_version: &str) -> Result<String> {
    let prefix = mc_version
        .strip_prefix("1.")
        .unwrap_or(mc_version);
    let metadata_url = format!(
        "{}/maven-metadata.xml",
        NEOFORGE_MAVEN_BASE
    );
    let response = reqwest::get(&metadata_url)
        .await
        .context("Failed to fetch NeoForge Maven metadata")?;
    let text = response.text().await?;

    let mut best: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(ver) = trimmed
            .strip_prefix("<version>")
            .and_then(|s| s.strip_suffix("</version>"))
        {
            if ver.starts_with(prefix)
                && (ver.len() == prefix.len()
                    || ver.as_bytes().get(prefix.len()) == Some(&b'.'))
            {
                match (&best, ver) {
                    (Some(b), v) if v > b.as_str() => best = Some(v.to_string()),
                    (None, v) => best = Some(v.to_string()),
                    _ => {}
                }
            }
        }
    }

    best.ok_or_else(|| anyhow::anyhow!(
        "No NeoForge version found for MC {}", mc_version
    ))
}

/// Run the NeoForge installer in headless mode.
pub async fn run_installer(installer_path: &Path, server_dir: &Path) -> Result<()> {
    info!(
        installer = %installer_path.display(),
        dir = %server_dir.display(),
        "Running NeoForge installer"
    );

    let output = Command::new("java")
        .arg("-jar")
        .arg(installer_path)
        .arg("--installServer")
        .current_dir(server_dir)
        .output()
        .await
        .context("Failed to execute NeoForge installer")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "NeoForge installer failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        ));
    }

    info!("NeoForge installer completed successfully");
    Ok(())
}

/// Resolve the generated NeoForge server launcher JAR.
///
/// NeoForge generates: libraries/net/neoforged/neoforge/{version}/neoforge-{version}.jar
/// We look for neoforge-*.jar files (not installer) in the server directory.
pub fn resolve_launcher_jar(server_dir: &Path) -> Result<std::path::PathBuf> {
    // Look for neoforge-*.jar files (not installer)
    let entries = std::fs::read_dir(server_dir).context("Failed to read server directory")?;

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("neoforge-")
            && name_str.ends_with(".jar")
            && !name_str.contains("installer")
        {
            return Ok(entry.path());
        }
    }

    // Fallback: check libraries/net/neoforged/neoforge/{version}/
    let lib_path = server_dir
        .join("libraries")
        .join("net")
        .join("neoforged")
        .join("neoforge");

    if lib_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&lib_path) {
            for entry in entries.flatten() {
                let version_dir = entry.path();
                if !version_dir.is_dir() {
                    continue;
                }
                if let Ok(files) = std::fs::read_dir(&version_dir) {
                    for file in files.flatten() {
                        let name = file.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with("neoforge-")
                            && name_str.ends_with(".jar")
                            && !name_str.contains("installer")
                        {
                            return Ok(file.path());
                        }
                    }
                }
            }
        }
    }

    // Last resort: look for any .jar in the server root that looks like a launcher
    // (runnable, non-installer, significant size)
    let entries = std::fs::read_dir(server_dir).context(
        "Failed to read server directory for launcher search",
    )?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "jar" {
                if let Ok(meta) = std::fs::metadata(&path) {
                    // Launcher JAR is typically > 1MB, installer is NOT a launcher
                    if meta.len() > 1_000_000 && !path.to_string_lossy().contains("installer") {
                        return Ok(path);
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Could not find NeoForge server launcher JAR in {}",
        server_dir.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_url_construction() {
        let version = "21.4.54-beta";
        let url = format!(
            "{}/{}/neoforge-{}-installer.jar",
            NEOFORGE_MAVEN_BASE, version, version
        );
        assert!(url.contains("21.4.54-beta"));
        assert!(url.starts_with("https://maven.neoforged.net"));
    }

    #[test]
    fn test_launcher_jar_pattern() {
        let name = "neoforge-21.4.54-beta.jar";
        assert!(
            name.starts_with("neoforge-")
                && name.ends_with(".jar")
                && !name.contains("installer")
        );

        let installer_name = "neoforge-21.4.54-beta-installer.jar";
        assert!(
            !(installer_name.starts_with("neoforge-")
                && installer_name.ends_with(".jar")
                && !installer_name.contains("installer"))
        );
    }
}
