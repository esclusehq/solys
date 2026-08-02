//! Backup handler - backup.create/restore
//!
//! Full implementation for backing up container volumes

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use agent_proto::Task;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tar::Archive;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{info, warn};
use zeroize::Zeroizing;

use reqwest::Client as HttpClient;

use crate::task_state::TASK_STATE_TRACKER;

use crate::handlers::direct_executor::server_dir_path;

use agent_backup::calculate_checksum;
use agent_backup::upload::{upload_to_s3_with_config, upload_to_local};

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init_data_dir(dir: PathBuf) {
    DATA_DIR.set(dir).expect("DATA_DIR already initialized");
}

/// Extract a tar archive with path traversal and symlink escape protection.
/// Blocks the current thread (I/O-bound, called from sync or spawn_blocking contexts).
fn safe_extract_tar(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;
    let mut archive = Archive::new(file);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?;

        // Resolve target path and verify it stays under dest
        let target = dest.join(&entry_path);
        if !target.starts_with(dest) {
            anyhow::bail!("Path traversal detected: {:?} resolves outside {:?}", entry_path, dest);
        }

        // Check for symlinks that could escape outside dest
        if entry.header().entry_type().is_symlink() {
            if let Some(link_target) = entry.link_name()? {
                let resolved = target.parent()
                    .unwrap_or(dest)
                    .join(&link_target);
                if let Ok(canonical) = resolved.canonicalize() {
                    if !canonical.starts_with(dest) {
                        anyhow::bail!("Symlink traversal detected: {:?} -> {:?}", entry_path, link_target);
                    }
                }
            }
        }

        entry.unpack_in(dest)?;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct BackupCreatePayload {
    pub server_id: uuid::Uuid,
    pub container_id: String,
    pub volumes: Option<Vec<String>>,
    pub backup_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BackupCreateOutput {
    pub backup_id: uuid::Uuid,
    pub size_bytes: u64,
    pub checksum: String,
    pub location: String,
}

/// Fields `container_id` and `target_paths` are part of the wire contract with the
/// backend but are no longer read locally (restore now extracts into the local
/// server data dir). `file_name` selects the specific archive to restore; when
/// absent, the newest `*.tar.gz` archive is used.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct BackupRestorePayload {
    pub server_id: uuid::Uuid,
    pub container_id: String,
    pub backup_id: uuid::Uuid,
    pub target_paths: Vec<String>,
    #[serde(default)]
    pub file_name: Option<String>,
}

/// Resolve the backup archive to restore.
///
/// With a `file_name`, validate it (reject path components so the path cannot
/// escape the backup dir) and use it; otherwise pick the newest `*.tar.gz`
/// archive in `backup_dir` as a fallback.
fn resolve_backup_path(backup_dir: &Path, file_name: Option<&str>) -> Result<PathBuf> {
    match file_name {
        Some(name) => {
            if Path::new(name).file_name() != Some(name.as_ref()) {
                anyhow::bail!("Invalid backup file name");
            }
            let path = backup_dir.join(name);
            if !path.is_file() {
                anyhow::bail!("Backup file not found: {name}");
            }
            Ok(path)
        }
        None => {
            let Ok(entries) = std::fs::read_dir(backup_dir) else {
                anyhow::bail!("Backup file not found");
            };
            let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
            for entry in entries {
                let entry = entry?;
                if !entry.file_name().to_string_lossy().ends_with(".tar.gz") {
                    continue;
                }
                if !entry.file_type()?.is_file() {
                    continue;
                }
                let modified = entry.metadata()?.modified()?;
                if newest.as_ref().map(|(_, t)| modified > *t).unwrap_or(true) {
                    newest = Some((entry.path(), modified));
                }
            }
            newest
                .map(|(path, _)| path)
                .ok_or_else(|| anyhow::anyhow!("Backup file not found"))
        }
    }
}

pub async fn handle_create(task: Task) -> Result<serde_json::Value> {
    let payload: BackupCreatePayload = serde_json::from_value(task.payload)?;

    info!(
        server_id = %payload.server_id,
        container_id = %payload.container_id,
        "Creating backup"
    );

    // Update progress
    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(5.0, "Starting backup...")).await;
    crate::task_state::send_progress(task.id, "running", 5.0, "Starting backup...").await;

    let backup_id = uuid::Uuid::new_v4();
    let backup_name = payload.backup_name.clone()
        .unwrap_or_else(|| format!("backup-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S")));

    // Create backup directory
    let backup_dir = DATA_DIR.get().expect("DATA_DIR not initialized").join("backups")
        .join(payload.server_id.to_string());
    tokio::fs::create_dir_all(&backup_dir).await
        .context("Failed to create backup directory")?;

    let backup_file = backup_dir.join(format!("{}.tar.gz", backup_name));

    // Step 1: Pause container for data consistency
    info!(container_id = %payload.container_id, "Pausing container");
    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(10.0, "Pausing container...")).await;
    crate::task_state::send_progress(task.id, "running", 10.0, "Pausing container...").await;
    
    let pause_result = Command::new("docker")
        .args(["pause", &payload.container_id])
        .output()
        .await;

    let needs_resume = match pause_result {
        Ok(output) if output.status.success() => true,
        Ok(output) => {
            warn!("Failed to pause container: {:?}", String::from_utf8_lossy(&output.stderr));
            false
        }
        Err(e) => {
            warn!("Failed to pause container: {}", e);
            false
        }
    };

    // Step 2: Create backup
    let volumes_to_backup = payload.volumes.clone()
        .unwrap_or_else(|| vec!["/data".to_string()]);

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(20.0, "Copying volumes...")).await;
    crate::task_state::send_progress(task.id, "running", 20.0, "Copying volumes...").await;

    let mut backup_size: u64 = 0;
    let mut checksum = String::new();

    for volume in volumes_to_backup {
        let volume_backup_file = backup_file.with_extension(format!("{}.tar.gz", volume.trim_start_matches('/')));
        
        // Use docker cp to copy volume to a temporary location, then tar it
        let temp_dir = PathBuf::from(format!("/tmp/escluse-backup-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir).await?;

        // Copy volume from container
        let cp_result = Command::new("docker")
            .args(["cp", &format!("{}:{}", payload.container_id, volume), temp_dir.to_string_lossy().as_ref()])
            .output()
            .await;

        if let Ok(output) = cp_result {
            if output.status.success() {
                // Create tarball
                let tar_result = Command::new("tar")
                    .args(["-czf", volume_backup_file.to_string_lossy().as_ref(), "-C", temp_dir.to_string_lossy().as_ref(), "."])
                    .output()
                    .await;

                if let Ok(tar_output) = tar_result {
                    if tar_output.status.success() {
                        // Get file size
                        if let Ok(metadata) = tokio::fs::metadata(&volume_backup_file).await {
                            backup_size += metadata.len();
                        }

                        // Calculate checksum
                        let checksum_output = Command::new("sha256sum")
                            .arg(volume_backup_file.to_string_lossy().as_ref())
                            .output()
                            .await?;

                        if checksum_output.status.success() {
                            let checksum_str = String::from_utf8_lossy(&checksum_output.stdout);
                            checksum = checksum_str.split_whitespace().next()
                                .unwrap_or("unknown")
                                .to_string();
                        }
                    }
                }
            }
        }

        // Cleanup temp dir
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }

    // Step 3: Resume container if was paused
    if needs_resume {
        info!(container_id = %payload.container_id, "Resuming container");
        TASK_STATE_TRACKER.update(task.id, |s| s.update_progress(90.0, "Resuming container...")).await;
        crate::task_state::send_progress(task.id, "running", 90.0, "Resuming container...").await;
        
        let _ = Command::new("docker")
            .args(["unpause", &payload.container_id])
            .output()
            .await;
    }

    let output = BackupCreateOutput {
        backup_id,
        size_bytes: backup_size,
        checksum,
        location: backup_file.to_string_lossy().to_string(),
    };

    info!(
        backup_id = %output.backup_id,
        size_bytes = output.size_bytes,
        "Backup created successfully"
    );

    Ok(serde_json::to_value(output)?)
}

// --- backup.start handler (canonical agent-side backup per D-10, D-11) ---

#[derive(Debug, Deserialize)]
pub struct BackupStartPayload {
    pub server_id: uuid::Uuid,
    pub container_name: Option<String>,
    pub container_id: Option<String>,
    pub backup_id: uuid::Uuid,
    pub file_name: String,
    pub provider: String, // "local" or "s3"
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    /// Pre-signed URL or backend proxy URL for direct HTTP upload (C-04).
    /// When present, S3 credentials never travel over WebSocket.
    pub upload_url: Option<String>,
    /// Optional custom headers for the HTTP PUT request (e.g., Authorization).
    pub upload_headers: Option<Vec<(String, String)>>,
}

#[derive(Debug, Serialize)]
pub struct BackupStartOutput {
    pub backup_id: uuid::Uuid,
    pub size_bytes: u64,
    pub checksum: String,
    pub storage_path: String,
}

/// Upload backup archive via HTTP PUT to a pre-signed/proxy URL.
/// Used when the backend provides an upload_url instead of raw S3 credentials (C-04).
async fn upload_via_http(
    archive_path: &Path,
    upload_url: &str,
    headers: Option<&Vec<(String, String)>>,
) -> Result<String> {
    let client = HttpClient::new();
    let archive_bytes = tokio::fs::read(archive_path)
        .await
        .context("Failed to read archive for HTTP upload")?;

    let mut req = client.put(upload_url)
        .header("Content-Type", "application/gzip")
        .body(archive_bytes);

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            req = req.header(key.as_str(), value.as_str());
        }
    }

    let resp = req.send()
        .await
        .context("HTTP upload request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("HTTP upload failed: {} — {}", status, body);
    }

    Ok(upload_url.to_string())
}

/// Upload backup archive using existing provider-based path (S3 credentials or local).
/// Extracted from handle_start for code clarity — used as fallback when upload_url is absent.
async fn upload_via_existing(payload: &BackupStartPayload, archive_path: &Path) -> Result<String> {
    match payload.provider.as_str() {
        "s3" => {
            let endpoint = payload.s3_endpoint.clone()
                .ok_or_else(|| anyhow::anyhow!("S3 endpoint required for s3 provider"))?;
            let bucket = payload.s3_bucket.clone()
                .ok_or_else(|| anyhow::anyhow!("S3 bucket required for s3 provider"))?;
            let access_key = payload.s3_access_key
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("S3 access key required for s3 provider"))?;
            let secret_key = payload.s3_secret_key
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("S3 secret key required for s3 provider"))?;

            // Credentials used ephemerally — dropped when handle_start returns (T-03-02-02)
            upload_to_s3_with_config(
                &endpoint,
                &bucket,
                &payload.s3_region.as_deref().unwrap_or_default(),
                access_key,
                secret_key,
                &payload.server_id.to_string(),
                &payload.file_name,
                archive_path,
            ).await
        }
        _ => {
            upload_to_local(
                archive_path,
                &DATA_DIR.get().expect("DATA_DIR not initialized").join("backups"),
                &payload.server_id.to_string(),
                &payload.file_name,
            ).await
        }
    }
}

/// Create a gzip tar archive of a server's data directory using the local
/// `tar` binary. Works without Docker/Podman (e.g. Termux on Android).
/// Returns (size_bytes, sha256_hex).
async fn create_local_server_archive(server_dir: &Path, dest_path: &Path) -> Result<(u64, String)> {
    if !server_dir.exists() {
        anyhow::bail!(
            "Server has no data on this node yet — start the server first to create its data directory ({})",
            server_dir.display()
        );
    }

    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let output = Command::new("tar")
        .args([
            "-czf",
            dest_path.to_string_lossy().as_ref(),
            "-C",
            server_dir.to_string_lossy().as_ref(),
            ".",
        ])
        .output()
        .await
        .context("Failed to spawn tar — is tar installed on this device?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Tar archive creation failed: {}", stderr.trim());
    }

    let size = tokio::fs::metadata(dest_path).await?.len();
    let checksum = calculate_checksum(dest_path).await?;
    info!(size_bytes = size, "Local server archive created");
    Ok((size, checksum))
}

/// Handle backup.start command — archive container data and upload directly to storage.
///
/// Architecture per D-10/D-11:
///   1. Agent creates tar+zstd archive using agent-backup crate
///   2. Agent uploads directly to S3 or local storage (no proxy through Worker/API)
///   3. Agent reports result (backup_id, size_bytes, checksum, storage_path) via TaskResult
pub async fn handle_start(task: Task) -> anyhow::Result<serde_json::Value> {
    let task_id = task.id;
    let payload: BackupStartPayload = serde_json::from_value(task.payload)?;
    let started_at = std::time::Instant::now();

    tracing::info!(
        server_id = %payload.server_id,
        backup_id = %payload.backup_id,
        "Starting agent-side backup"
    );

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| {
        s.update_progress(5.0, "Starting backup...")
    }).await;
    crate::task_state::send_progress(task.id, "running", 5.0, "Starting backup...").await;

    // Resolve container identifier (kept for contract validation; archiving no longer uses it)
    let _container_id = payload.container_id.as_deref()
        .or(payload.container_name.as_deref())
        .ok_or_else(|| anyhow::anyhow!("Either container_id or container_name must be provided"))?;

    // C-01: Validate container name used in docker subprocess calls
    if payload.container_name.is_some() {
        let name = payload.container_name.as_deref().unwrap();
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
            anyhow::bail!("Invalid container name: {:?}", name);
        }
    }

    // 1. Archive to a temp path — never inside the backup dir, so upload_to_local
    //    can copy it to its final storage path without self-copying (fs::copy to
    //    the same path truncates the archive to 0 bytes).
    let archive_path = std::env::temp_dir()
        .join(format!("escluse-archive-{}.tar.gz", uuid::Uuid::new_v4()));

    // Run the body, then remove the temp archive on success AND error paths.
    let result = run_backup_start(&payload, task_id, &archive_path, started_at).await;
    let _ = tokio::fs::remove_file(&archive_path).await;
    result
}

/// Archive + upload body of `handle_start` — kept separate so the temp archive
/// is always cleaned up by the caller, regardless of the outcome.
async fn run_backup_start(
    payload: &BackupStartPayload,
    task_id: uuid::Uuid,
    archive_path: &Path,
    started_at: std::time::Instant,
) -> anyhow::Result<serde_json::Value> {
    TASK_STATE_TRACKER.update(task_id, |s: &mut crate::task_state::TaskState| {
        s.update_progress(20.0, "Creating archive...")
    }).await;
    crate::task_state::send_progress(task_id, "running", 20.0, "Creating archive...").await;

    let server_dir = server_dir_path(
        DATA_DIR.get().expect("DATA_DIR not initialized"),
        &payload.server_id,
    );
    let (archive_size, archive_checksum) = create_local_server_archive(&server_dir, archive_path).await?;

    // 2. Calculate checksum
    let checksum = if archive_checksum.is_empty() {
        calculate_checksum(archive_path).await?
    } else {
        archive_checksum
    };

    // 3. Upload directly to storage (D-11 — no proxy)
    TASK_STATE_TRACKER.update(task_id, |s: &mut crate::task_state::TaskState| {
        s.update_progress(60.0, "Uploading backup...")
    }).await;
    crate::task_state::send_progress(task_id, "running", 60.0, "Uploading backup...").await;

    // C-04: Check for pre-signed URL / proxy-via-backend path first
    let storage_path = if let Some(ref upload_url) = payload.upload_url {
        if !upload_url.is_empty() {
            info!("Uploading via pre-signed/proxy URL (C-04)");
            upload_via_http(archive_path, upload_url, payload.upload_headers.as_ref()).await?
        } else {
            // empty upload_url — fall through to existing provider-based upload
            upload_via_existing(payload, archive_path).await?
        }
    } else {
        upload_via_existing(payload, archive_path).await?
    };

    TASK_STATE_TRACKER.update(task_id, |s: &mut crate::task_state::TaskState| {
        s.update_progress(100.0, "Backup complete")
    }).await;
    crate::task_state::send_progress(task_id, "completed", 100.0, "Backup complete").await;

    let output = BackupStartOutput {
        backup_id: payload.backup_id,
        size_bytes: archive_size,
        checksum,
        storage_path,
    };

    tracing::info!(
        backup_id = %output.backup_id,
        size_bytes = output.size_bytes,
        duration_ms = %started_at.elapsed().as_millis(),
        "Backup completed successfully"
    );

    Ok(serde_json::to_value(output)?)
}

pub async fn handle_restore(task: Task) -> Result<serde_json::Value> {
    let payload: BackupRestorePayload = serde_json::from_value(task.payload)?;

    info!(
        server_id = %payload.server_id,
        backup_id = %payload.backup_id,
        "Restoring backup"
    );

    // Resolve the requested archive (or the newest one when no name is given)
    let backup_dir = DATA_DIR.get().expect("DATA_DIR not initialized").join("backups")
        .join(payload.server_id.to_string());

    let backup_path = resolve_backup_path(&backup_dir, payload.file_name.as_deref())?;

    // Step 1: Resolve server data directory
    let server_dir = server_dir_path(
        DATA_DIR.get().expect("DATA_DIR not initialized"),
        &payload.server_id,
    );
    tokio::fs::create_dir_all(&server_dir).await?;

    // Step 2: Extract archive into server directory (path-traversal protected)
    safe_extract_tar(&backup_path, &server_dir)
        .context("Failed to extract backup archive")?;

    info!(backup_id = %payload.backup_id, "Backup restored successfully");

    Ok(serde_json::json!({
        "status": "restored",
        "backup_id": payload.backup_id,
        "server_id": payload.server_id
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_local_server_archive_ok() {
        let tmp = std::env::temp_dir().join(format!("escluse-test-archive-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(tmp.join("world")).await.unwrap();
        tokio::fs::write(tmp.join("world").join("level.dat"), b"fake-level-data").await.unwrap();

        let dest = std::env::temp_dir().join(format!("escluse-test-out-{}.tar.gz", uuid::Uuid::new_v4()));
        let (size, checksum) = create_local_server_archive(&tmp, &dest).await.unwrap();

        assert!(size > 0, "archive should not be empty");
        assert_eq!(checksum.len(), 64, "checksum should be a sha256 hex string");
        assert!(tokio::fs::metadata(&dest).await.is_ok(), "archive file should exist");

        let _ = tokio::fs::remove_dir_all(&tmp).await;
        let _ = tokio::fs::remove_file(&dest).await;
    }

    #[tokio::test]
    async fn test_backup_archive_survives_local_upload() {
        let tmp = std::env::temp_dir().join(format!("escluse-test-server-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(tmp.join("world")).await.unwrap();
        tokio::fs::write(tmp.join("world").join("level.dat"), b"fake-level-data").await.unwrap();

        // The archive is staged at a temp path guaranteed to be OUTSIDE backup_base,
        // so upload_to_local's source and dest differ here. NOTE: this test does NOT
        // guard against handle_start staging the archive inside the backup dir — the
        // staged path is hardcoded, independent of handle_start. That failure mode is
        // documented by the ignored test_local_upload_self_copy_does_not_truncate.
        let backup_base = std::env::temp_dir().join(format!("escluse-test-base-{}", uuid::Uuid::new_v4()));
        let sid = uuid::Uuid::new_v4().to_string();
        let fname = format!("backup-{}.tar.gz", uuid::Uuid::new_v4());
        let archive_path = std::env::temp_dir().join(format!("escluse-test-upload-{}.tar.gz", uuid::Uuid::new_v4()));

        let (size, _checksum) = create_local_server_archive(&tmp, &archive_path).await.unwrap();
        assert!(size > 0, "archive should not be empty");

        upload_to_local(&archive_path, &backup_base, &sid, &fname).await.unwrap();

        let dest = backup_base.join(&sid).join(&fname);
        assert!(tokio::fs::metadata(&dest).await.is_ok(), "dest file should exist");
        assert_eq!(
            tokio::fs::metadata(&dest).await.unwrap().len(),
            size,
            "dest must keep the full archive size (no self-copy truncation)"
        );

        let src_bytes = tokio::fs::read(&archive_path).await.unwrap();
        let dest_bytes = tokio::fs::read(&dest).await.unwrap();
        assert_eq!(src_bytes, dest_bytes, "dest bytes must equal the staged archive bytes");

        let _ = tokio::fs::remove_dir_all(&tmp).await;
        let _ = tokio::fs::remove_dir_all(&backup_base).await;
        let _ = tokio::fs::remove_file(&archive_path).await;
    }

    #[tokio::test]
    async fn test_create_local_server_archive_missing_dir() {
        let missing = std::path::PathBuf::from(format!("/nonexistent/escluse-{}", uuid::Uuid::new_v4()));
        let dest = std::env::temp_dir().join(format!("escluse-test-miss-{}.tar.gz", uuid::Uuid::new_v4()));

        let err = create_local_server_archive(&missing, &dest).await.unwrap_err();
        assert!(err.to_string().contains("no data on this node"), "got: {}", err);

        let _ = tokio::fs::remove_file(&dest).await;
    }

    // Self-copy regression: staging the archive at exactly {backup_base}/{sid}/{fname}
    // (the old buggy handle_start behavior) makes upload_to_local's fs::copy run with
    // source == dest, truncating the archive to 0 bytes. This test documents that
    // failure mode; un-ignore once upload_to_local learns to skip same-path copies.
    #[tokio::test]
    #[ignore = "documents the self-copy truncation failure mode; un-ignore when upload_to_local skips same-path copies"]
    async fn test_local_upload_self_copy_does_not_truncate() {
        let tmp = std::env::temp_dir().join(format!("escluse-test-sc-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(tmp.join("world")).await.unwrap();
        tokio::fs::write(tmp.join("world").join("level.dat"), b"fake-level-data").await.unwrap();

        let backup_base = std::env::temp_dir().join(format!("escluse-test-sc-base-{}", uuid::Uuid::new_v4()));
        let sid = uuid::Uuid::new_v4().to_string();
        let fname = format!("backup-{}.tar.gz", uuid::Uuid::new_v4());

        // Simulate the old handle_start: the archive lives inside the backup dir at
        // its final storage path, so source == dest when upload_to_local runs.
        let same_path = backup_base.join(&sid).join(&fname);
        tokio::fs::create_dir_all(same_path.parent().unwrap()).await.unwrap();
        let (size, _checksum) = create_local_server_archive(&tmp, &same_path).await.unwrap();
        assert!(size > 0, "archive should not be empty");

        upload_to_local(&same_path, &backup_base, &sid, &fname).await.unwrap();

        let after = tokio::fs::metadata(&same_path).await.unwrap().len();
        assert!(after > 0, "self-copy must not truncate the archive to 0 bytes");

        let _ = tokio::fs::remove_dir_all(&tmp).await;
        let _ = tokio::fs::remove_dir_all(&backup_base).await;
    }

    #[tokio::test]
    async fn test_resolve_backup_path_by_name() {
        let backup_dir = std::env::temp_dir().join(format!("escluse-test-resolve-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&backup_dir).await.unwrap();
        let old = backup_dir.join("backup-old.tar.gz");
        let new = backup_dir.join("backup-new.tar.gz");
        tokio::fs::write(&old, b"old").await.unwrap();
        tokio::fs::write(&new, b"new").await.unwrap();

        let picked = resolve_backup_path(&backup_dir, Some("backup-old.tar.gz")).unwrap();
        assert_eq!(picked, old, "must pick the named archive, not another one");

        let err = resolve_backup_path(&backup_dir, Some("../backup-old.tar.gz")).unwrap_err();
        assert!(err.to_string().contains("Invalid backup file name"), "got: {}", err);

        let err = resolve_backup_path(&backup_dir, Some("nope.tar.gz")).unwrap_err();
        assert!(err.to_string().contains("Backup file not found: nope.tar.gz"), "got: {}", err);

        let _ = tokio::fs::remove_dir_all(&backup_dir).await;
    }

    #[tokio::test]
    async fn test_resolve_backup_path_latest_fallback() {
        let backup_dir = std::env::temp_dir().join(format!("escluse-test-fallback-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&backup_dir).await.unwrap();
        let older = backup_dir.join("backup-older.tar.gz");
        let newer = backup_dir.join("backup-newer.tar.gz");
        tokio::fs::write(&older, b"older").await.unwrap();
        tokio::fs::write(&newer, b"newer").await.unwrap();
        tokio::fs::write(backup_dir.join("notes.txt"), b"not an archive").await.unwrap();

        // Explicit mtimes so "newest" is deterministic and independent of write order.
        let epoch = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        let f = std::fs::OpenOptions::new().write(true).open(&older).unwrap();
        f.set_modified(epoch).unwrap();
        drop(f);
        let f = std::fs::OpenOptions::new().write(true).open(&newer).unwrap();
        f.set_modified(epoch + std::time::Duration::from_secs(60)).unwrap();
        drop(f);

        let picked = resolve_backup_path(&backup_dir, None).unwrap();
        assert_eq!(picked, newer, "must pick the newest archive when no name is given");

        let _ = tokio::fs::remove_dir_all(&backup_dir).await;
    }
}


