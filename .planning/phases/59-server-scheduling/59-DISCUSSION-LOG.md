# Phase 59: Server Scheduling — Discussion Log

**Date:** 2026-05-31
**Status:** Context gathered, ready for planning

## Gray Areas Discussed

### 1. Execution Architecture
**Question:** Worker-based vs API-based execution for scheduled actions?

**Decision:** Worker-based (D-01). Extend Worker cron_eval.rs to handle start/stop/restart/sleep task types. Consistent with Phase 55 pattern.

### 2. Start Task Type
**Question:** Add 'start' as a new task_type value or use existing command type?

**Decision:** Add 'start' to cron_tasks.task_type enum alongside sleep (D-02).

### 3. Timezone Support
**Question:** Per-schedule timezone column, user-level default, or UTC-only?

**Decision:** Per-schedule timezone VARCHAR(50) column (D-03). Default 'UTC'.

### 4. UI Placement
**Question:** Settings tab section vs separate tab vs existing ScheduledTasksPage?

**Decision:** Settings tab section in ServerDetails (D-04), alongside Sleep/Wake and Restart Policy.

### 5. Error Handling
**Question:** Log+toast+event+retry vs log+retry only?

**Decision:** Log + toast + server event + 1x retry after 30s (D-05).

### 6. One-Time Actions
**Question:** Support run-once flag or recurring only?

**Decision:** Add run_once BOOLEAN column, auto-disable after execution (D-06).

### 7. Sleep Interaction
**Question:** Phase 56 auto-sleep vs scheduled sleep precedence?

**Decision:** Phase 56 auto-sleep takes precedence (D-07). Scheduled sleep skips if already asleep.

### 8. Scheduled Restart vs Auto-Restart
**Question:** How do scheduled restarts interact with Phase 57 auto-restart?

**Decision:** Complementary (D-08). Scheduled restart bypasses crash detection. Waits for auto-restart cooldown if in progress.

## Key Documents Created
- `59-CONTEXT.md` — Full context with locked decisions, code references, and migration plan
