//! Handlers module

#![allow(dead_code)]

pub mod runtime;
pub mod backup;
pub mod rcon;
pub mod metrics;
pub mod ssh;
pub mod sftp;

use std::time::Duration;

use agent_capability::CapabilityRegistry;
use agent_proto::{Task, TaskError, TaskResult};
use agent_runtime::RuntimeDetector;
use tokio::time::timeout;
use tracing::{error, info};

use crate::task_state::TASK_STATE_TRACKER;
use crate::audit;

pub async fn execute_task(
    task: Task,
    runtime: &RuntimeDetector,
    _capabilities: &CapabilityRegistry,
) -> TaskResult {
    let task_id = task.id;
    let task_type = task.task_type.clone();
    let started_at = chrono::Utc::now();

    // Log task received
    audit::log_task_received(task_id, &task_type).await;

    // Check rate limit before execution
    if let Err(e) = crate::rate_limit::check_rate_limit(&task_type, None).await {
        error!(task_type = %task_type, error = %e, "Rate limit exceeded");
        crate::task_state::send_progress(task_id, "failed", 0.0, &format!("Rate limit exceeded: {}", e)).await;
        audit::log_task_failed(task_id, &format!("Rate limit: {}", e)).await;
        return TaskResult::failed(
            task_id,
            TaskError::new("RATE_LIMITED", &e.to_string(), false),
            started_at
        );
    }

    // Track task state
    TASK_STATE_TRACKER.add(task_id, task_type.clone()).await;
    TASK_STATE_TRACKER.update(task_id, |s: &mut crate::task_state::TaskState| s.start()).await;
    crate::task_state::send_progress(task_id, "running", 0.0, "Task started").await;

    let result = execute_with_timeout_and_retry(&task, runtime).await;

    match result {
        Ok(output) => {
            TASK_STATE_TRACKER.update(task_id, |s: &mut crate::task_state::TaskState| s.complete()).await;
            crate::task_state::send_progress(task_id, "completed", 100.0, "Task completed successfully").await;
            audit::log_task_completed(task_id).await;
            TaskResult::completed(task_id, output, started_at)
        }
        Err(e) => {
            error!(task_id = %task_id, error = %e, "Task failed");
            TASK_STATE_TRACKER.update(task_id, |s: &mut crate::task_state::TaskState| s.fail(&e.to_string())).await;
            crate::task_state::send_progress(task_id, "failed", 0.0, &e.to_string()).await;
            audit::log_task_failed(task_id, &e.to_string()).await;
            let retryable = is_retryable(&e);
            TaskResult::failed(task_id, TaskError::new(e.code(), &e.to_string(), retryable), started_at)
        }
    }
}

async fn execute_with_timeout_and_retry(
    task: &Task,
    runtime: &RuntimeDetector,
) -> Result<serde_json::Value, HandlerError> {
    let task_type = task.task_type.as_str();
    let config = get_task_config(task_type);

    let mut attempt = 0;
    let max_attempts = config.max_retries + 1;

    loop {
        attempt += 1;
        let result = execute_single(task, runtime, config.timeout).await;

        match result {
            Ok(output) => return Ok(output),
            Err(e) if !e.is_retryable() || attempt >= max_attempts => {
                return Err(e);
            }
            Err(e) => {
                let delay = config.retry_delay_ms * (config.backoff_multiplier.powi((attempt - 1) as i32)) as u64;
                let delay = delay.min(config.max_retry_delay_ms);
                info!(
                    task_type = %task_type,
                    attempt = attempt,
                    delay_ms = delay,
                    error = %e,
                    "Retrying task after error"
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }
    }
}

async fn execute_single(
    task: &Task,
    runtime: &RuntimeDetector,
    timeout_duration: Duration,
) -> Result<serde_json::Value, HandlerError> {
    let task_type = task.task_type.clone();

    let future = async {
        match task_type.as_str() {
            // Server operations
            "server.create" => runtime::handle_create(task.clone(), runtime).await,
            "server.start" => runtime::handle_start(task.clone(), runtime).await,
            "server.stop" => runtime::handle_stop(task.clone(), runtime).await,
            "server.restart" => runtime::handle_restart(task.clone(), runtime).await,
            "server.delete" => runtime::handle_delete(task.clone(), runtime).await,
            "server.logs" => runtime::handle_logs(task.clone(), runtime).await,

            // Backup operations
            "backup.create" => backup::handle_create(task.clone()).await,
            "backup.restore" => backup::handle_restore(task.clone()).await,

            // RCON
            "server.command" => rcon::handle_command(task.clone()).await,

            // Metrics
            "metrics.report" => metrics::handle_report(task.clone()).await,

            // SSH
            "ssh.connect" => ssh::handle_connect(task.clone()).await,
            "ssh.execute" => ssh::handle_execute(task.clone()).await,
            "ssh.disconnect" => ssh::handle_disconnect(task.clone()).await,

            // SFTP
            "sftp.upload" => sftp::handle_upload(task.clone()).await,
            "sftp.download" => sftp::handle_download(task.clone()).await,

            // Unknown
            _ => Err(anyhow::anyhow!("Unknown task type: {}", task_type)),
        }
    };

    match timeout(timeout_duration, future).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(HandlerError::from_anyhow(e)),
        Err(_) => Err(HandlerError::timeout(&format!(
            "Task {} timed out after {:?}",
            task_type, timeout_duration
        ))),
    }
}

struct TaskConfig {
    timeout: Duration,
    max_retries: u32,
    retry_delay_ms: u64,
    max_retry_delay_ms: u64,
    backoff_multiplier: f64,
}

fn get_task_config(task_type: &str) -> TaskConfig {
    match task_type {
        "server.start" => TaskConfig {
            timeout: Duration::from_secs(60),
            max_retries: 3,
            retry_delay_ms: 2000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
        },
        "server.restart" => TaskConfig {
            timeout: Duration::from_secs(90),
            max_retries: 3,
            retry_delay_ms: 2000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
        },
        "server.stop" => TaskConfig {
            timeout: Duration::from_secs(60),
            max_retries: 0,
            retry_delay_ms: 0,
            max_retry_delay_ms: 0,
            backoff_multiplier: 1.0,
        },
        "server.create" => TaskConfig {
            timeout: Duration::from_secs(120),
            max_retries: 0,
            retry_delay_ms: 0,
            max_retry_delay_ms: 0,
            backoff_multiplier: 1.0,
        },
        "server.delete" => TaskConfig {
            timeout: Duration::from_secs(30),
            max_retries: 0,
            retry_delay_ms: 0,
            max_retry_delay_ms: 0,
            backoff_multiplier: 1.0,
        },
        "server.command" => TaskConfig {
            timeout: Duration::from_secs(30),
            max_retries: 2,
            retry_delay_ms: 1000,
            max_retry_delay_ms: 10000,
            backoff_multiplier: 2.0,
        },
        "backup.create" => TaskConfig {
            timeout: Duration::from_secs(600),
            max_retries: 0,
            retry_delay_ms: 0,
            max_retry_delay_ms: 0,
            backoff_multiplier: 1.0,
        },
        "backup.restore" => TaskConfig {
            timeout: Duration::from_secs(600),
            max_retries: 0,
            retry_delay_ms: 0,
            max_retry_delay_ms: 0,
            backoff_multiplier: 1.0,
        },
        "ssh.connect" => TaskConfig {
            timeout: Duration::from_secs(30),
            max_retries: 2,
            retry_delay_ms: 2000,
            max_retry_delay_ms: 10000,
            backoff_multiplier: 2.0,
        },
        "ssh.execute" => TaskConfig {
            timeout: Duration::from_secs(60),
            max_retries: 2,
            retry_delay_ms: 1000,
            max_retry_delay_ms: 10000,
            backoff_multiplier: 2.0,
        },
        "sftp.upload" => TaskConfig {
            timeout: Duration::from_secs(300),
            max_retries: 3,
            retry_delay_ms: 2000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
        },
        "sftp.download" => TaskConfig {
            timeout: Duration::from_secs(300),
            max_retries: 3,
            retry_delay_ms: 2000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
        },
        "metrics.report" => TaskConfig {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay_ms: 1000,
            max_retry_delay_ms: 10000,
            backoff_multiplier: 2.0,
        },
        _ => TaskConfig {
            timeout: Duration::from_secs(60),
            max_retries: 0,
            retry_delay_ms: 1000,
            max_retry_delay_ms: 10000,
            backoff_multiplier: 2.0,
        },
    }
}

fn is_retryable(e: &HandlerError) -> bool {
    e.is_retryable()
}

#[derive(Debug)]
struct HandlerError {
    code: String,
    message: String,
    retryable: bool,
}

impl HandlerError {
    fn from_anyhow(e: anyhow::Error) -> Self {
        let msg = e.to_string().to_lowercase();
        let retryable = msg.contains("timeout")
            || msg.contains("connection")
            || msg.contains("temporary")
            || msg.contains("unavailable")
            || msg.contains("refused")
            || msg.contains("busy");

        let code = if msg.contains("timeout") {
            "TIMEOUT"
        } else if msg.contains("connection") {
            "CONNECTION_FAILED"
        } else if msg.contains("auth") || msg.contains("permission") {
            "AUTH_FAILED"
        } else {
            "TASK_FAILED"
        };

        Self {
            code: code.to_string(),
            message: e.to_string(),
            retryable,
        }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn is_retryable(&self) -> bool {
        self.retryable
    }

    fn timeout(msg: &str) -> Self {
        Self {
            code: "TIMEOUT".to_string(),
            message: msg.to_string(),
            retryable: true,
        }
    }
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for HandlerError {}
