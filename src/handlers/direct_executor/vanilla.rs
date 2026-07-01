//! Vanilla Minecraft server JAR download via Mojang version manifest
//!
//! Mojang publishes a version manifest at piston-meta.mojang.com.
//! We find the latest release version matching our target, extract
//! the server.jar download URL, and download it.

use anyhow::{Context, Result};
use serde::Deserialize;

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Deserialize)]
struct Manifest {
    versions: Vec<ManifestVersion>,
}

#[derive(Deserialize)]
struct ManifestVersion {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionDetail {
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    server: DownloadEntry,
}

#[derive(Deserialize)]
struct DownloadEntry {
    url: String,
}

/// Get the direct download URL for a Vanilla Minecraft server JAR.
///
/// Resolves through: version manifest → version detail → server download URL.
pub async fn get_download_url(version: &str) -> Result<String> {
    // Fetch manifest
    let manifest: Manifest = reqwest::get(VERSION_MANIFEST_URL)
        .await
        .context("Failed to fetch Mojang version manifest")?
        .json()
        .await?;

    // Find matching release version
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version && v.version_type == "release")
        .ok_or_else(|| {
            anyhow::anyhow!("Vanilla version {} not found in manifest", version)
        })?;

    // Fetch version detail to get server download URL
    let detail: VersionDetail = reqwest::get(&entry.url)
        .await
        .with_context(|| format!("Failed to fetch version detail for {}", version))?
        .json()
        .await?;

    Ok(detail.downloads.server.url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_url() {
        assert_eq!(
            VERSION_MANIFEST_URL,
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
        );
    }
}
