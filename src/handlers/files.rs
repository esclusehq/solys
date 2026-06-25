//! File operations handler - list_dir, read_file, write_file
//!
//! Handles file operations on containers running on this node.

use agent_proto::Task;
use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::{info, error};

pub async fn handle_list_dir(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;

    let container_name = get_container_name(&payload)?;

    let path = payload.get("path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    info!(container_name = %container_name, path = %path, "Listing directory");

    let full_path = if path.is_empty() || path == "." {
        "/data".to_string()
    } else if path.starts_with('/') {
        format!("/data{}", path)
    } else {
        format!("/data/{}", path)
    };

    let output = Command::new("docker")
        .args(["exec", &container_name, "ls", "-la", "--time-style=+%s", &full_path])
        .output()
        .await
        .context("Failed to run docker exec ls")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(stderr = %stderr, "ls command failed");
        return Err(anyhow::anyhow!("ls failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(serde_json::Value::String(stdout))
}

pub async fn handle_read_file(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;

    let container_name = get_container_name(&payload)?;

    let path = payload.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    info!(container_name = %container_name, path = %path, "Reading file");

    let output = Command::new("docker")
        .args(["exec", &container_name, "cat", path])
        .output()
        .await
        .context("Failed to run docker exec cat")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to read file: {}", stderr));
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(serde_json::Value::String(content))
}

pub async fn handle_write_file(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;

    let container_name = payload.get("container_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing container_name"))?;

    let path = payload.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let content = payload.get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

    info!(container_name = %container_name, path = %path, "Writing file");

    // Write to temp file and use docker cp
    let temp_dir = std::env::temp_dir().join(format!("escluse-write-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&temp_dir).await
        .context("Failed to create temp dir")?;

    let temp_file = temp_dir.join("file");
    tokio::fs::write(&temp_file, content).await
        .context("Failed to write temp file")?;

    let output = Command::new("docker")
        .args(["cp", temp_file.to_str().unwrap(), &format!("{}:{}", container_name, path)])
        .output()
        .await
        .context("Failed to run docker cp")?;

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to write file: {}", stderr));
    }

    Ok(serde_json::json!({ "status": "written" }))
}

pub async fn handle_delete(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;
    let container_name = get_container_name(&payload)?;
    let path = payload.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    info!(container_name = %container_name, path = %path, "Deleting path");

    let output = Command::new("docker")
        .args(["exec", &container_name, "rm", "-rf", path])
        .output()
        .await
        .context("Failed to run docker exec rm")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to delete: {}", stderr));
    }

    Ok(serde_json::json!({ "status": "deleted" }))
}

pub async fn handle_mkdir(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;
    let container_name = get_container_name(&payload)?;
    let path = payload.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    info!(container_name = %container_name, path = %path, "Creating directory");

    let output = Command::new("docker")
        .args(["exec", &container_name, "mkdir", "-p", path])
        .output()
        .await
        .context("Failed to run docker exec mkdir")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to create directory: {}", stderr));
    }

    Ok(serde_json::json!({ "status": "created" }))
}

pub async fn handle_rename(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;
    let container_name = get_container_name(&payload)?;
    let source = payload.get("source_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing source_path"))?;
    let dest = payload.get("dest_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing dest_path"))?;

    info!(container_name = %container_name, source = %source, dest = %dest, "Renaming path");

    let output = Command::new("docker")
        .args(["exec", &container_name, "mv", source, dest])
        .output()
        .await
        .context("Failed to run docker exec mv")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to rename: {}", stderr));
    }

    Ok(serde_json::json!({ "status": "renamed" }))
}

pub async fn handle_copy(task: Task) -> Result<serde_json::Value> {
    let payload = task.payload;
    let container_name = get_container_name(&payload)?;
    let source = payload.get("source_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing source_path"))?;
    let dest = payload.get("dest_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing dest_path"))?;

    info!(container_name = %container_name, source = %source, dest = %dest, "Copying path");

    let output = Command::new("docker")
        .args(["exec", &container_name, "cp", "-r", source, dest])
        .output()
        .await
        .context("Failed to run docker exec cp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to copy: {}", stderr));
    }

    Ok(serde_json::json!({ "status": "copied" }))
}

fn validate_container_name(name: &str) -> Result<String> {
    if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        Ok(name.to_string())
    } else {
        Err(anyhow::anyhow!("Invalid container name: {:?}", name))
    }
}

fn get_container_name(payload: &serde_json::Value) -> Result<String> {
    if let Some(name) = payload.get("container_name").and_then(|v| v.as_str()) {
        validate_container_name(name)
    } else if let Some(server_id) = payload.get("server_id").and_then(|v| v.as_str()) {
        Ok(format!("mc-{}", server_id))
    } else {
        Err(anyhow::anyhow!("Missing container_name or server_id"))
    }
}
