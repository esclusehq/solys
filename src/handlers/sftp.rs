//! SFTP handler - sftp.upload/sftp.download

use agent_proto::Task;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::ssh::SSH_CACHE;
use crate::task_state::TASK_STATE_TRACKER;

#[derive(Debug, Deserialize)]
pub struct SftpUploadPayload {
    pub server_id: uuid::Uuid,
    pub connection_key: String,
    pub local_path: String,
    pub remote_path: String,
}

#[derive(Debug, Serialize)]
pub struct SftpUploadOutput {
    pub status: String,
    pub bytes_transferred: u64,
}

#[derive(Debug, Deserialize)]
pub struct SftpDownloadPayload {
    pub server_id: uuid::Uuid,
    pub connection_key: String,
    pub remote_path: String,
    pub local_path: String,
}

#[derive(Debug, Serialize)]
pub struct SftpDownloadOutput {
    pub status: String,
    pub bytes_transferred: u64,
}

pub async fn handle_upload(task: Task) -> Result<serde_json::Value> {
    let payload: SftpUploadPayload = serde_json::from_value(task.payload)?;

    info!(
        server_id = %payload.server_id,
        connection_key = %payload.connection_key,
        local_path = %payload.local_path,
        remote_path = %payload.remote_path,
        "Uploading file via SFTP"
    );

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(10.0, "Starting upload...")).await;
    crate::task_state::send_progress(task.id, "running", 10.0, "Starting upload...").await;

    let local_meta = std::fs::metadata(&payload.local_path)?;
    let file_size = local_meta.len();

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(30.0, "Uploading...")).await;
    crate::task_state::send_progress(task.id, "running", 30.0, "Uploading...").await;

    let mut cache = SSH_CACHE.write().await;
    let client = cache.get(&payload.connection_key).await
        .ok_or_else(|| anyhow::anyhow!("Connection not found: {}", payload.connection_key))?;

    client.upload_file(&payload.local_path, &payload.remote_path).await?;

    cache.put(payload.connection_key.clone(), client).await;

    info!(
        server_id = %payload.server_id,
        bytes = file_size,
        "File uploaded successfully"
    );

        TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(100.0, "Upload complete")).await;
        crate::task_state::send_progress(task.id, "completed", 100.0, "Upload complete").await;

    let output = SftpUploadOutput {
        status: "uploaded".to_string(),
        bytes_transferred: file_size,
    };

    Ok(serde_json::to_value(output)?)
}

pub async fn handle_download(task: Task) -> Result<serde_json::Value> {
    let payload: SftpDownloadPayload = serde_json::from_value(task.payload)?;

    info!(
        server_id = %payload.server_id,
        connection_key = %payload.connection_key,
        remote_path = %payload.remote_path,
        local_path = %payload.local_path,
        "Downloading file via SFTP"
    );

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(10.0, "Starting download...")).await;
    crate::task_state::send_progress(task.id, "running", 10.0, "Starting download...").await;

    TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(30.0, "Downloading...")).await;
    crate::task_state::send_progress(task.id, "running", 30.0, "Downloading...").await;

    let mut cache = SSH_CACHE.write().await;
    let client = cache.get(&payload.connection_key).await
        .ok_or_else(|| anyhow::anyhow!("Connection not found: {}", payload.connection_key))?;

    client.download_file(&payload.remote_path, &payload.local_path).await?;

    cache.put(payload.connection_key.clone(), client).await;

    let file_size = std::fs::metadata(&payload.local_path)?.len();

    info!(
        server_id = %payload.server_id,
        bytes = file_size,
        "File downloaded successfully"
    );

        TASK_STATE_TRACKER.update(task.id, |s: &mut crate::task_state::TaskState| s.update_progress(100.0, "Download complete")).await;
        crate::task_state::send_progress(task.id, "completed", 100.0, "Download complete").await;

    let output = SftpDownloadOutput {
        status: "downloaded".to_string(),
        bytes_transferred: file_size,
    };

    Ok(serde_json::to_value(output)?)
}
