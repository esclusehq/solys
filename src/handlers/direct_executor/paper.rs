//! PaperMC server JAR download via API v2

use anyhow::{bail, Context, Result};
use serde::Deserialize;

const PAPER_API_BASE: &str = "https://api.papermc.io/v2/projects/paper";

#[derive(Deserialize)]
struct BuildsResponse {
    builds: Vec<BuildInfo>,
}

#[derive(Deserialize)]
struct BuildInfo {
    build: i32,
    channel: String,
}

/// Get the download URL for the latest stable Paper build for a version.
///
/// Paper API v2 does NOT have a "latest" endpoint per
/// PaperMC/Paper#9624. We query all builds, filter for STABLE,
/// pick the highest build number.
pub async fn get_download_url(version: &str) -> Result<String> {
    let url = format!("{}/versions/{}/builds", PAPER_API_BASE, version);
    let resp: BuildsResponse = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to query PaperMC API: {}", url))?
        .json()
        .await?;

    let latest = resp
        .builds
        .iter()
        .filter(|b| b.channel == "STABLE")
        .max_by_key(|b| b.build)
        .ok_or_else(|| {
            anyhow::anyhow!("No stable Paper build found for version {}", version)
        })?;

    Ok(format!(
        "{}/versions/{}/builds/{}/downloads/paper-{}-{}.jar",
        PAPER_API_BASE, version, latest.build, version, latest.build
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let version = "1.21.4";
        let build = 232i32;
        let url = format!(
            "{}/versions/{}/builds/{}/downloads/paper-{}-{}.jar",
            PAPER_API_BASE, version, build, version, build
        );
        assert!(url.contains("1.21.4"));
        assert!(url.contains("232"));
        assert!(url.starts_with("https://api.papermc.io"));
    }
}
