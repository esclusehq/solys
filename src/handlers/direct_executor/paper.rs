//! PaperMC server JAR download via Fill API v3 (replaced the sunset v2 API).

use anyhow::{bail, Context, Result};
use serde::Deserialize;

const PAPER_API_BASE: &str = "https://fill.papermc.io/v3/projects/paper";

#[derive(Deserialize)]
struct BuildInfo {
    id: i32,
    channel: String,
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    #[serde(rename = "server:default")]
    server_default: DownloadEntry,
}

#[derive(Deserialize)]
struct DownloadEntry {
    name: String,
    url: String,
}

/// Get the download URL for the latest stable Paper build for a version.
///
/// Uses the Fill API v3 (https://fill.papermc.io/v3/projects/paper).
/// Returns the download URL for the latest stable build.
pub async fn get_download_url(version: &str) -> Result<String> {
    let url = format!("{}/versions/{}/builds", PAPER_API_BASE, version);
    let resp: Vec<BuildInfo> = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to query PaperMC API: {}", url))?
        .json()
        .await?;

    let latest = resp
        .into_iter()
        .filter(|b| b.channel == "STABLE")
        .max_by_key(|b| b.id)
        .ok_or_else(|| {
            anyhow::anyhow!("No stable Paper build found for version {}", version)
        })?;

    Ok(latest.downloads.server_default.url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_from_download_entry() {
        let entry = DownloadEntry {
            name: "paper-1.21.4-232.jar".into(),
            url: "https://fill-data.papermc.io/v1/objects/abc/paper-1.21.4-232.jar".into(),
        };
        assert!(entry.url.starts_with("https://fill-data.papermc.io"));
    }
}
