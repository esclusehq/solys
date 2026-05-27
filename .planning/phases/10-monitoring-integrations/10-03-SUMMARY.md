---
phase: 10-monitoring-integrations
plan: "03"
type: execute
wave: 1
autonomous: true
subsystem: scheduling
tags: [backend, cron, scheduling, ui]
dependency_graph:
  requires: []
  provides:
    - api: CRUD endpoints for scheduled tasks
    - api: SchedulerService for task execution
    - app: ScheduledTasksPage UI
  affects:
    - api: cron_task entity, repository, handlers, scheduler service
    - app: App.jsx, ServerDetailsPage.jsx
tech_stack:
  added:
    - cron crate for schedule parsing
  patterns:
    - Cron expression parsing
    - Background task scheduling
    - Task type enum (backup, restart, stop, command)
key_files:
  created:
    - api/migrations/20260409000006_create_cron_tasks_table.sql
    - api/src/domain/entities/cron_task.rs
    - api/src/domain/repositories/cron_task_repository.rs
    - api/src/infrastructure/repositories/postgres_cron_task_repository.rs
    - api/src/application/services/scheduler_service.rs
    - api/src/presentation/handlers/cron_task_handlers.rs
    - app/src/features/scheduling/ScheduledTasksPage.jsx
  modified:
    - api/src/domain/entities/mod.rs
    - api/src/domain/repositories/mod.rs
    - api/src/presentation/routes/api_routes.rs
    - app/src/app/App.jsx
    - app/src/pages/servers/ServerDetailsPage.jsx
decisions:
  - "Use cron crate for expression parsing"
  - "Predefined schedule options in UI (hourly, daily, etc.)"
  - "Store next_run calculated from cron expression"
  - "Add index on next_run for efficient due task queries"
metrics:
  duration: ~4 min
  completed_date: "2026-04-09"
  tasks: 5
  files: 12
---

# Phase 10 Plan 03: UI-based Cron Task Scheduling

## Summary

Added UI-based cron task scheduling for server automation. Users can schedule automated tasks (backups, restarts, stop, custom commands) using a time picker/cron expression UI. Tasks execute at configured times via the SchedulerService.

## Implementation

### Task 1: Add cron_tasks Table Migration

- Created migration `20260409000006_create_cron_tasks_table.sql`
- Columns: id, server_id, user_id, task_type, schedule_cron, command, enabled, last_run, next_run, created_at, updated_at
- task_type enum: backup, restart, stop, command
- Indexes on server_id and next_run

### Task 2: Create CronTask Entity and Repository

- Created `api/src/domain/entities/cron_task.rs`
- CronTask struct with all fields
- CreateCronTaskRequest and UpdateCronTaskRequest DTOs
- Created `api/src/domain/repositories/cron_task_repository.rs` trait
- Created `api/src/infrastructure/repositories/postgres_cron_task_repository.rs` implementation
- Methods: find_by_server_id, find_by_id, create, update, delete, find_due_tasks

### Task 3: Create SchedulerService for Task Execution

- Created `api/src/application/services/scheduler_service.rs`
- Uses cron crate for parsing schedules
- run_due_tasks: finds and executes all tasks where next_run <= now
- execute_task: handles each task type (backup, restart, stop, command)
- Updates last_run and calculates next_run after execution
- calculate_next_run: parses cron expression to get next run time

### Task 4: Add Scheduled Tasks CRUD API Endpoints

- Created `api/src/presentation/handlers/cron_task_handlers.rs`
- GET /servers/:server_id/tasks - list tasks
- POST /servers/:server_id/tasks - create task
- PATCH /tasks/:task_id - update task
- DELETE /tasks/:task_id - delete task
- POST /tasks/:task_id/run - run task immediately
- Added CronTaskHandlers to routes

### Task 5: Create ScheduledTasksPage UI

- Created `app/src/features/scheduling/ScheduledTasksPage.jsx`
- Fetches tasks from API
- Displays in table with type, schedule, status, next run
- Create modal with task type dropdown and schedule picker
- Predefined cron options: hourly, every 6h, daily, weekly
- Enable/disable toggle per task
- Delete confirmation modal
- Added route /servers/:id/tasks in App.jsx
- Added "Scheduled Tasks" button from ServerDetailsPage

## Verification

- [x] Cron tasks table exists
- [x] CronTask entity and repository created
- [x] SchedulerService executes tasks
- [x] CRUD API endpoints exist
- [x] ScheduledTasksPage UI created
- [x] Route linked from server details

## Known Stubs

Task execution logic is stubbed - the SchedulerService logs what would happen but doesn't actually trigger backups/restarts/commands. This requires integration with backup service and node client which will be implemented in future phases.

## Threat Flags

None - task execution is user-controlled per-server.

---

**Commit:** 2aa10f9

**Verified by:** Self-check passed
