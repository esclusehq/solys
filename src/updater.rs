use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use agent_config::UpdateChannel;

#[derive(Debug, Deserialize)]
struct Manifest {
    latest: String,
    versions: std::collections::HashMap<String, std::collections::HashMap<String, ReleaseAsset>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ReleaseAsset {
    url: String,
    sha256: String,
}

fn platform_key() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("linux-x86_64"),
        ("linux", "aarch64") => Some("linux-aarch64"),
        ("android", "aarch64") => Some("android-aarch64"),
        ("android", "arm") => Some("android-armv7"),
        ("windows", "x86_64") => Some("windows-x86_64"),
        (os, arch) => {
            warn!(
                "Unknown platform {}-{}, update skipped — no binary available for this platform",
                os, arch
            );
            None
        }
    }
}

const MANIFEST_URL: &str = "https://get.esclusehg.com/versions.json";

async fn fetch_manifest(client: &reqwest::Client) -> Result<Manifest> {
    let resp = client
        .get(MANIFEST_URL)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .context("Failed to fetch version manifest")?;

    if !resp.status().is_success() {
        anyhow::bail!("Manifest fetch returned HTTP {}", resp.status());
    }

    let manifest: Manifest = resp
        .json()
        .await
        .context("Failed to parse version manifest")?;

    Ok(manifest)
}

async fn download_archive(client: &reqwest::Client, url: &str, dest: &Path) -> Result<PathBuf> {
    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .context("Failed to download update archive")?;

    if !resp.status().is_success() {
        anyhow::bail!("Download returned HTTP {}", resp.status());
    }

    let bytes = resp.bytes().await.context("Failed to read response body")?;

    let tmp = dest.with_extension("tmp");
    tokio::fs::write(&tmp, &bytes)
        .await
        .context("Failed to write archive to disk")?;

    Ok(tmp)
}

fn verify_sha256(path: &Path, expected_hex: &str) -> Result<()> {
    use sha2::Digest;
    let data = std::fs::read(path).context("Failed to read file for checksum")?;
    let hash = sha2::Sha256::digest(&data);
    let hex = format!("{:x}", hash);

    let expected = expected_hex.to_lowercase();
    if hex != expected {
        anyhow::bail!("SHA256 mismatch: expected {}, got {}", expected, hex);
    }
    info!("SHA256 checksum verified");
    Ok(())
}

fn extract_binary(archive: &Path, extract_dir: &Path) -> Result<PathBuf> {
    use flate2::read::GzDecoder;
    use tar::Archive as TarArchive;

    let file =
        std::fs::File::open(archive).context("Failed to open archive")?;
    let decoder = GzDecoder::new(file);
    let mut tar = TarArchive::new(decoder);

    std::fs::create_dir_all(extract_dir).context("Failed to create extract directory")?;

    let mut extracted = None;
    for entry in tar.entries().context("Failed to read tar entries")? {
        let mut entry = entry.context("Failed to read tar entry")?;
        let entry_path = entry.path().context("Failed to read entry path")?.to_path_buf();
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if name == "escluse-agent" {
            let dest = extract_dir.join("escluse-agent-new");
            entry.unpack(&dest).context("Failed to extract binary")?;
            extracted = Some(dest);
            break;
        }
    }

    extracted.ok_or_else(|| anyhow::anyhow!("escluse-agent binary not found in archive"))
}

fn replace_binary(new_binary: &Path) -> Result<PathBuf> {
    let current_exe =
        std::env::current_exe().context("Failed to get current executable path")?;

    let old = current_exe.with_extension("old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&current_exe, &old).context("Failed to rename current binary")?;

    std::fs::rename(new_binary, &current_exe)
        .or_else(|_| std::fs::copy(new_binary, &current_exe).map(|_| ()))
        .context("Failed to install new binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755))
            .context("Failed to set executable permission")?;
    }

    info!("Binary updated: {}", current_exe.display());
    Ok(current_exe)
}

pub fn spawn(
    config: agent_config::AutoUpdateConfig,
    current_version: String,
    data_dir: PathBuf,
    shutdown: watch::Receiver<bool>,
) {
    if !config.enabled {
        info!("Auto-update disabled");
        return;
    }

    let update_dir = data_dir.join("update");
    let _ = std::fs::create_dir_all(&update_dir);

    tokio::spawn(async move {
        let mut shutdown = shutdown;
        let client = match reqwest::Client::builder()
            .user_agent(format!("escluse-agent/{}", current_version))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to build HTTP client: {}", e);
                return;
            }
        };

        // First check after 60s (give agent time to connect)
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    info!("Auto-updater shutting down");
                    return;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(config.check_interval_secs)) => {}
            }

            match check_and_update(&client, &current_version, &update_dir, &config.channel).await {
                Ok(true) => {
                    info!("Update applied, restarting...");
                    break;
                }
                Ok(false) => {
                    debug!("No update available");
                }
                Err(e) => {
                    error!("Update check failed: {}", e);
                }
            }
        }

        info!("Spawning updated agent and exiting...");
        let exe = std::env::current_exe().ok();
        let args: Vec<String> = std::env::args().skip(1).collect();
        if let Some(exe) = exe {
            let _ = std::process::Command::new(&exe)
                .args(&args)
                .env("ESCLUSE_AGENT_UPDATED", "1")
                .spawn();
        }
        std::process::exit(0);
    });
}

async fn check_and_update(
    client: &reqwest::Client,
    current_version: &str,
    update_dir: &Path,
    channel: &UpdateChannel,
) -> Result<bool> {
    let manifest = fetch_manifest(client).await?;

    let latest_str = match channel {
        UpdateChannel::Canary => {
            debug!("Canary channel selected, using latest from manifest");
            &manifest.latest
        }
        UpdateChannel::Stable => &manifest.latest,
    };

    let latest_ver = match Version::parse(latest_str) {
        Ok(v) => v,
        Err(e) => {
            error!("Invalid latest version in manifest: {} ({})", latest_str, e);
            return Ok(false);
        }
    };

    let current_ver = match Version::parse(current_version) {
        Ok(v) => v,
        Err(_) => {
            debug!("Current version '{}' is not semver, skipping update", current_version);
            return Ok(false);
        }
    };

    if latest_ver <= current_ver {
        debug!("Already at latest version ({} >= {})", current_version, latest_str);
        return Ok(false);
    }

    info!(
        "Update available: {} -> {}",
        current_version, latest_str
    );

    let releases = match manifest.versions.get(latest_str) {
        Some(r) => r,
        None => {
            error!("No release data for version {}", latest_str);
            return Ok(false);
        }
    };

    let key = match platform_key() {
        Some(k) => k,
        None => return Ok(false),
    };
    let asset = match releases.get(key) {
        Some(a) => a,
        None => {
            error!("No release asset for platform {}", key);
            return Ok(false);
        }
    };

    info!("Downloading {} from {}", latest_str, asset.url);
    let archive_path = download_archive(client, &asset.url, &update_dir.join("update.tar.gz")).await?;

    verify_sha256(&archive_path, &asset.sha256)?;

    let extract_dir = update_dir.join("extract");
    let new_binary = extract_binary(&archive_path, &extract_dir)?;

    replace_binary(&new_binary)?;

    let _ = std::fs::remove_file(&archive_path);
    let _ = std::fs::remove_dir_all(&extract_dir);

    info!("Successfully updated to v{}", latest_str);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_key() {
        // Most test hosts are linux-x86_64
        if let Some(key) = platform_key() {
            assert!(!key.is_empty());
            assert!(key.contains('-'));
        }
        // If the test runs on an unknown platform, the key is None — that's fine
    }
}
