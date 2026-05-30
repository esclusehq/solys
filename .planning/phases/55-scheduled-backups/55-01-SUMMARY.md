---
phase: 55-scheduled-backups
plan: 01
status: completed
completed: 2026-05-30
---

# Plan 55-01 Summary: Worker Cron Evaluation & Backup Dispatch

## Accomplishments

- **worker/Cargo.toml** — Added `cron = "0.15"` dependency for cron expression parsing
- **worker/src/cron_eval.rs** — New module with `run_cron_evaluation_loop` and `evaluate_and_dispatch`:
  - 30s polling loop queries `cron_tasks WHERE enabled=true AND task_type='backup' AND next_run <= NOW()`
  - Dispatches `backup_server` jobs via Redis priority queue (HSET + ZADD)
  - Updates `last_run` on cron_tasks after dispatch
  - LIMIT 50 safeguards against flooding (T-55-01-01)
- **worker/src/config.rs** — Added `api_base_url` field with env var fallback to `http://api:3000`
- **worker/src/main.rs** — Added sqlx PgPool, spawned `run_cron_evaluation_loop` and `run_prune_loop` as background tasks
- **worker/src/queue/mod.rs** — `JobProcessor` now holds `PgPool`, `process_backup_server` fully implemented:
  - Active backup check (T-55-01-02 prevention)
  - Server + container details query
  - Creates `backup_history` in_progress record
  - Sends `backup.start` command via reqwest to API node commands endpoint
- **worker/src/prune.rs** — New module with `run_prune_loop` (15-min interval) and `evaluate_and_prune`:
  - Queries servers with retention config
  - Combined label-based (daily/weekly/monthly) + count-based (max_retained_backups) evaluation per D-07/D-16
  - Deletes eligible backups via API DELETE endpoint

## Verification

- All 9 grep checks pass
