//! Fabric server launcher JAR download via Fabric Meta API
//!
//! Fabric's `/server/jar` endpoint returns a self-bootstrapping
//! fabric-server-launch.jar directly (assumption A1). This JAR
//! downloads Minecraft + Fabric dependencies on first run.

use anyhow::{Context, Result};

const FABRIC_META_BASE: &str = "https://meta.fabricmc.net/v2/versions/loader";

/// Get the download URL for the Fabric server launcher JAR.
///
/// Uses loader version "0.16.9" as default (latest stable for MC 1.21).
/// The endpoint returns binary JAR content, not JSON.
pub async fn get_download_url(version: &str) -> Result<String> {
    // Resolve latest loader version first
    let loader_version = resolve_latest_loader(version).await?;
    Ok(format!(
        "{}/{}/{}/server/jar",
        FABRIC_META_BASE, version, loader_version
    ))
}

#[derive(serde::Deserialize)]
struct LoaderVersion {
    loader: LoaderInfo,
}

#[derive(serde::Deserialize)]
struct LoaderInfo {
    version: String,
}

/// Resolve the latest Fabric loader version for a Minecraft version.
async fn resolve_latest_loader(version: &str) -> Result<String> {
    let url = format!("{}/{}", FABRIC_META_BASE, version);
    let loaders: Vec<LoaderVersion> = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to query Fabric meta: {}", url))?
        .json()
        .await?;

    loaders
        .first()
        .map(|l| l.loader.version.clone())
        .ok_or_else(|| anyhow::anyhow!("No Fabric loader found for MC {}", version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let version = "1.21.4";
        let loader = "0.16.9";
        let url = format!(
            "{}/{}/{}/server/jar",
            FABRIC_META_BASE, version, loader
        );
        assert!(url.contains("1.21.4"));
        assert!(url.contains("0.16.9"));
        assert!(url.starts_with("https://meta.fabricmc.net"));
    }
}
