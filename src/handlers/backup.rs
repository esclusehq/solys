//! Backup handler - backup.create/restore
//!
//! Full implementation for backing up container volumes

use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use agent_proto::Task;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tar::Archive;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{info, warn};

use crate::task_state::TASK_STATE_TRACKER;

use agent_backup::{create_container_backup, calculate_checksum, CompressionFormat};
use agent_backup::upload::{upload_to_s3_with_config, upload_to_local};

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

#[derive(Debug, Deserialize)]
pub struct BackupRestorePayload {
    pub server_id: uuid::Uuid,
    pub container_id: String,
    pub backup_id: uuid::Uuid,
    pub target_paths: Vec<String>,
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
    let backup_dir = PathBuf::from("/var/lib/escluse-agent/backups")
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
            .args(["cp", &format!("{}:{}", payload.container_id, volume), temp_dir.to_str().unwrap()])
            .output()
            .await;

        if let Ok(output) = cp_result {
            if output.status.success() {
                // Create tarball
                let tar_result = Command::new("tar")
                    .args(["-czf", volume_backup_file.to_str().unwrap(), "-C", temp_dir.to_str().unwrap(), "."])
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
                            .arg(volume_backup_file.to_str().unwrap())
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
}

#[derive(Debug, Serialize)]
pub struct BackupStartOutput {
    pub backup_id: uuid::Uuid,
    pub size_bytes: u64,
    pub checksum: String,
    pub storage_path: String,
}

/// Handle backup.start command — archive container data and upload directly to storage.
///
/// Architecture per D-10/D-11:
///   1. Agent creates tar+zstd archive using agent-backup crate
///   2. Agent uploads directly to S3 or local storage (no proxy through Worker/API)
///   3. Agent reports result (backup_id, size_bytes, checksum, storage_path) via TaskResult
pub async fn handle_start(task: Task) -> anyhow::Result<serde_json::Value> {
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

    // Resolve container identifier
    let container_id = payload.container_id.as_deref()
        .or(payload.container_name.as_deref())
        .ok_or_else(|| anyhow::anyhow!("Either container_id or container_name must be provided"))?;

    // C-01: Validate container name used in docker subprocess calls
    if payload.container_name.is_some() {
        let name = payload.container_name.as_deref().unwrap();
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
            anyhow::bail!("Invalid container name: {:?}", name);
        }
    }

    // 1. Create archive from container data directory
    let backup_dir = std::path::PathBuf::from("/var/lib/escluse-agent/backups")
        .join(payload.server_id.to_string());
    tokio::fs::create_dir_all(&backup_dir).await?;

    let archive_path = backup_dir.join(&payload.file_name);

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| {
        s.update_progress(20.0, "Creating archive...")
    }).await;
    crate::task_state::send_progress(task.id, "running", 20.0, "Creating archive...").await;

    let (archive_size, archive_checksum) = create_container_backup(
        container_id,
        "/data",
        &archive_path,
        CompressionFormat::Zstd(3),
    ).await?;

    // 2. Calculate checksum
    let checksum = if archive_checksum.is_empty() {
        calculate_checksum(&archive_path).await?
    } else {
        archive_checksum
    };

    // 3. Upload directly to storage (D-11 — no proxy)
    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| {
        s.update_progress(60.0, "Uploading backup...")
    }).await;
    crate::task_state::send_progress(task.id, "running", 60.0, "Uploading backup...").await;

    let storage_path = match payload.provider.as_str() {
        "s3" => {
            let endpoint = payload.s3_endpoint
                .ok_or_else(|| anyhow::anyhow!("S3 endpoint required for s3 provider"))?;
            let bucket = payload.s3_bucket
                .ok_or_else(|| anyhow::anyhow!("S3 bucket required for s3 provider"))?;
            let access_key = payload.s3_access_key
                .ok_or_else(|| anyhow::anyhow!("S3 access key required for s3 provider"))?;
            let secret_key = payload.s3_secret_key
                .ok_or_else(|| anyhow::anyhow!("S3 secret key required for s3 provider"))?;

            upload_to_s3_with_config(
                &endpoint,
                &bucket,
                &payload.s3_region.unwrap_or_default(),
                &access_key,
                &secret_key,
                &payload.server_id.to_string(),
                &payload.file_name,
                &archive_path,
            ).await?
        }
        _ => {
            upload_to_local(
                &archive_path,
                &std::path::PathBuf::from("/var/lib/escluse-agent/backups"),
                &payload.server_id.to_string(),
                &payload.file_name,
            ).await?
        }
    };

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| {
        s.update_progress(100.0, "Backup complete")
    }).await;
    crate::task_state::send_progress(task.id, "completed", 100.0, "Backup complete").await;

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

    // Find backup file
    let backup_dir = PathBuf::from("/var/lib/escluse-agent/backups")
        .join(payload.server_id.to_string());

    // Look for the backup file
    let mut backup_file: Option<PathBuf> = None;
    
    if let Ok(entries) = tokio::fs::read_dir(&backup_dir).await {
        let mut entries = entries;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().map(|e| e == "tar.gz").unwrap_or(false) {
                backup_file = Some(path);
                break;
            }
        }
    }

    let backup_path = backup_file
        .context("Backup file not found")?;

    // Step 1: Stop container
    info!(container_id = %payload.container_id, "Stopping container for restore");
    let _ = Command::new("docker")
        .args(["stop", "-t", "30", &payload.container_id])
        .output()
        .await;

    // Step 2: Restore volumes
    for target_path in &payload.target_paths {
        // Create temp directory
        let temp_dir = PathBuf::from(format!("/tmp/escluse-restore-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir).await?;

        // Extract with path traversal protection using Rust tar crate
        safe_extract_tar(&backup_path, &temp_dir)
            .context("Failed to extract backup archive")?;

        // Copy back to container
        let copy_source = temp_dir.join(target_path.trim_start_matches('/'));
        if copy_source.exists() {
            let dest = format!("{}:{}", payload.container_id, target_path);
            let _ = Command::new("docker")
                .args(["cp", copy_source.to_str().unwrap(), &dest])
                .output()
                .await;
        }

        // Cleanup
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }

    // Step 3: Start container
    info!(container_id = %payload.container_id, "Starting container after restore");
    let _ = Command::new("docker")
        .args(["start", &payload.container_id])
        .output()
        .await;

    info!(backup_id = %payload.backup_id, "Backup restored successfully");

    Ok(serde_json::json!({
        "status": "restored",
        "backup_id": payload.backup_id,
        "container_id": payload.container_id
    }))
}

async fn upload_to_s3(bucket: &str, region: &str, key: &str, file_path: &PathBuf) -> Result<String> {
    use rusoto_s3::{S3, S3Client, PutObjectRequest};
    use rusoto_core::Region;

    let region = Region::from_str(region)
        .unwrap_or_else(|_| Region::UsEast1);
    
    let client = S3Client::new(region);

    let file_data = tokio::fs::read(file_path).await
        .context("Failed to read backup file")?;

    let request = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(file_data.into()),
        content_type: Some("application/gzip".to_string()),
        ..Default::default()
    };

    client.put_object(request).await
        .context("Failed to upload to S3")?;

    Ok(format!("s3://{}/{}", bucket, key))
}

#[derive(Debug, Deserialize)]
pub struct S3RestorePayload {
    pub server_id: uuid::Uuid,
    pub container_id: String,
    pub backup_id: uuid::Uuid,
    pub s3_bucket: String,
    pub s3_key: String,
    pub s3_region: Option<String>,
    pub target_paths: Vec<String>,
}

pub async fn handle_restore_s3(task: Task) -> Result<serde_json::Value> {
    let payload: S3RestorePayload = serde_json::from_value(task.payload)?;

    info!(
        server_id = %payload.server_id,
        bucket = %payload.s3_bucket,
        "Restoring from S3 backup"
    );

    let s3_region = payload.s3_region.unwrap_or_else(|| "us-east-1".to_string());

    // Download from S3
    let temp_dir = PathBuf::from(format!("/tmp/escluse-s3-restore-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&temp_dir).await?;

    let download_path = temp_dir.join("backup.tar.gz");
    download_from_s3(
        &payload.s3_bucket,
        &payload.s3_key,
        &s3_region,
        &download_path,
    ).await?;

    // Stop container
    let _ = Command::new("docker")
        .args(["stop", "-t", "30", &payload.container_id])
        .output()
        .await;

    // Extract with path traversal protection using Rust tar crate
    let extract_dir = temp_dir.join("extracted");
    tokio::fs::create_dir_all(&extract_dir).await?;

    safe_extract_tar(&download_path, &extract_dir)
        .context("Failed to extract S3 backup archive")?;

    // Restore to container
    for target_path in &payload.target_paths {
        let source = extract_dir.join(target_path.trim_start_matches('/'));
        if source.exists() {
            let dest = format!("{}:{}", payload.container_id, target_path);
            let _ = Command::new("docker")
                .args(["cp", source.to_str().unwrap(), dest.as_str()])
                .output()
                .await;
        }
    }

    // Start container
    let _ = Command::new("docker")
        .args(["start", &payload.container_id])
        .output()
        .await;

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    info!(backup_id = %payload.backup_id, "S3 backup restored successfully");

    Ok(serde_json::json!({
        "status": "restored",
        "backup_id": payload.backup_id,
        "container_id": payload.container_id,
        "source": "s3"
    }))
}

async fn download_from_s3(bucket: &str, key: &str, region: &str, dest: &PathBuf) -> Result<()> {
    use futures_util::StreamExt;
    use rusoto_s3::{S3, S3Client, GetObjectRequest};
    use rusoto_core::Region;

    let region = Region::from_str(region)
        .unwrap_or_else(|_| Region::UsEast1);
    
    let client = S3Client::new(region);

    let request = GetObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        ..Default::default()
    };

    let response = client.get_object(request).await
        .context("Failed to download from S3")?;

    let mut stream = response.body.ok_or_else(|| anyhow::anyhow!("Empty S3 response"))?;
    
    let mut file = tokio::fs::File::create(dest).await?;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }
    
    file.flush().await?;
    
    Ok(())
}
