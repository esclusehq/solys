//! Task Queue - Priority-based task execution
//!
//! This module provides priority mapping and queue utilities for task execution.
//! For MVP, we use direct execution with priority-based ordering.

use std::collections::VecDeque;
use std::sync::Arc;

use agent_proto::{Task, TaskPriority, TaskResult};
use agent_runtime::RuntimeDetector;
use tokio::sync::RwLock;
use tracing::info;

use crate::handlers;

/// Priority mapping for task types
pub fn map_task_priority(task_type: &str) -> TaskPriority {
    match task_type {
        // High priority - critical operations
        "server.stop" | "server.delete" | "server.command" | "ssh.execute" => TaskPriority::High,
        
        // Low priority - background operations
        "backup.create" | "backup.restore" | "server.logs" | "metrics.report" => TaskPriority::Low,
        
        // Normal priority - default
        _ => TaskPriority::Normal,
    }
}

/// Apply priority mapping to task
pub fn with_priority(task: Task) -> Task {
    let priority = map_task_priority(&task.task_type);
    info!(task_type = %task.task_type, priority = ?priority, "Mapped task priority");
    task.with_priority(priority)
}

/// Async task queue for buffered execution
/// For now, simple FIFO with priority hints (full implementation for future)
pub struct TaskQueue {
    queue: Arc<RwLock<VecDeque<Task>>>,
    max_size: usize,
}

impl TaskQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            max_size,
        }
    }

    pub async fn enqueue(&self, task: Task) -> bool {
        let mut queue = self.queue.write().await;
        if queue.len() >= self.max_size {
            return false;
        }
        queue.push_back(task);
        true
    }

    pub async fn dequeue(&self) -> Option<Task> {
        let mut queue = self.queue.write().await;
        queue.pop_front()
    }

    pub async fn len(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }

    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.read().await;
        queue.is_empty()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Execute task with priority mapping
pub async fn execute_with_priority(
    task: Task,
    runtime: &RuntimeDetector,
) -> TaskResult {
    let task_with_priority = with_priority(task);
    handlers::execute_task(task_with_priority, runtime).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_mapping() {
        assert_eq!(map_task_priority("server.stop"), TaskPriority::High);
        assert_eq!(map_task_priority("server.command"), TaskPriority::High);
        assert_eq!(map_task_priority("ssh.execute"), TaskPriority::High);
        
        assert_eq!(map_task_priority("backup.create"), TaskPriority::Low);
        assert_eq!(map_task_priority("metrics.report"), TaskPriority::Low);
        
        assert_eq!(map_task_priority("server.start"), TaskPriority::Normal);
        assert_eq!(map_task_priority("sftp.upload"), TaskPriority::Normal);
    }
}
