//! Phase 60: Crash Detection — Crash Reporter Module
//!
//! Captures container exit code and last N lines of logs when a managed
//! container exits unexpectedly. Provides helper to build the CrashReport
//! WebSocket message for the backend.

use bollard::container::LogsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use uuid::Uuid;
use anyhow::Result;

use agent_proto::messages::CrashReportPayload;

/// Captures container exit code and last N lines of logs.
/// Called when a managed container exits unexpectedly.
///
/// # Returns
/// `(exit_code, log_excerpt)` where log_excerpt is the last 10 lines,
/// truncated to 4KB max.
pub async fn capture_crash_data(
    docker: &Docker,
    container_id: &str,
) -> Result<(i32, String)> {
    // 1. Inspect container to get exit code
    let inspect = docker.inspect_container(container_id, None).await?;
    let exit_code = inspect
        .state
        .and_then(|s| s.exit_code)
        .unwrap_or(-1);

    // 2. Read last 10 lines of logs (both stdout and stderr)
    let log_options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: String::from("10"),
        ..Default::default()
    };

    let mut log_stream = docker.logs(container_id, Some(log_options));
    let mut log_lines: Vec<String> = Vec::new();

    while let Some(Ok(chunk)) = log_stream.next().await {
        log_lines.push(chunk.to_string());
    }

    let log_excerpt = log_lines.join("\n");

    // 3. Truncate to 4KB max to avoid flooding WebSocket
    let log_excerpt = if log_excerpt.len() > 4096 {
        format!("... (truncated) ...\n{}", &log_excerpt[log_excerpt.len() - 4000..])
    } else {
        log_excerpt
    };

    Ok((exit_code as i32, log_excerpt))
}

/// Build a CrashReportPayload from raw crash data.
pub fn build_crash_report(
    server_id: Uuid,
    exit_code: i32,
    log_excerpt: String,
) -> CrashReportPayload {
    CrashReportPayload {
        agent_id: server_id,
        exit_code,
        log_excerpt,
        timestamp: chrono::Utc::now(),
    }
}
