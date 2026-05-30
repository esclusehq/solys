// ──────────────────────────────────────────────────────────────────────────────
// DISABLED — Phase 55 (Scheduled Backups)
// This module is kept for reference but no longer started.
// Worker-side cron evaluation (worker/src/cron_eval.rs) replaces this per D-02.
// The Worker polls cron_tasks table and dispatches backup_server jobs via Redis.
// ──────────────────────────────────────────────────────────────────────────────

//! Background worker that evaluates CRON schedules — DISABLED since Phase 55.
//! Replaced by Worker-side cron evaluation (worker/src/cron_eval.rs).
//! Kept for reference.

use std::sync::Arc;
use crate::domain::repositories::{
    backup_repository::BackupRepository,
    server_repository::ServerRepository,
};
use crate::application::services::backup_service::BackupService;

/// Background worker that evaluates CRON schedules — DISABLED since Phase 55.
/// Replaced by Worker-side cron evaluation (worker/src/cron_eval.rs).
pub struct BackupScheduler<S, B>
where
    S: ServerRepository + ?Sized,
    B: BackupRepository + ?Sized,
{
    backup_service: Arc<BackupService<S, B>>,
    server_repository: Arc<S>,
    backup_repository: Arc<B>,
}

impl<S, B> BackupScheduler<S, B>
where
    S: ServerRepository + ?Sized + Send + Sync + 'static,
    B: BackupRepository + ?Sized + Send + Sync + 'static,
{
    pub fn new(
        backup_service: Arc<BackupService<S, B>>,
        server_repository: Arc<S>,
        backup_repository: Arc<B>,
    ) -> Self {
        Self {
            backup_service,
            server_repository,
            backup_repository,
        }
    }

    /// Main scheduler loop — DISABLED (D-02: replaced by Worker cron evaluation)
    pub async fn run(self: Arc<Self>) {
        tracing::warn!("BackupScheduler::run() is DISABLED since Phase 55. Worker cron_eval replaces this.");
        // The old 60s tick loop that queried server.backup_cron is removed.
        // See worker/src/cron_eval.rs for the replacement.
    }
}
