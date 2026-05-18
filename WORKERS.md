# Worker Service Documentation

## Overview

Worker adalah service background untuk memproses job asynchronous secara distributed. Menggunakan Redis sebagai message queue dengan priority-based processing.

## Arsitektur

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────────┐
│   API   │────▶│  Redis  │────▶│ Worker  │────▶│ Agent/Solys │
│         │     │ Queue   │     │         │     │             │
└─────────┘     └─────────┘     └─────────┘     └─────────────┘
```

## Keuntungan Worker Pattern

### ✅ Keuntungan

| Keuntungan | Penjelasan |
|------------|-------------|
| **Async Processing** | Request user tidak perlu nunggu operasi panjang (create server, backup) |
| **Scalability** | Bisa tambah worker instance tanpa impact API |
| **Reliability** | Job yang gagal bisa di-retry dari queue |
| **Load Distribution** |高峰期 job di-queue dulu, tidak membanjiri system |
| **Decoupling** | API dan worker独立communication via Redis |
| **Priority Queue** | Job urgent (high) diproses dulu dari low |
| **Failure Isolation** | Worker crash tidak affect API user |

### ❌ Kerugian

| Kerugian | Penjelasan |
|----------|-------------|
| **Complexity** | Perlu tambahan infrastructure (Redis, worker service) |
| **Latency** | Job tidak instant - ada delay polling queue |
| **Debugging** | Lebih sulit trace error dibanding synchronous call |
| **Consistency** | Job status perlu tracking manual (pending → processing → completed) |
| **Duplicate Work** | Perlu idempotency design agar job tidak doble |

## Job Types

### 1. Server Management

| Job Type | Deskripsi |
|----------|------------|
| `create_server` | Buat server baru |
| `delete_server` | Hapus server |
| `start_server` | Start server |
| `stop_server` | Stop server |
| `restart_server` | Restart server |
| `kill_server` | Force stop (emergency) |

### 2. Backup & Restore

| Job Type | Deskripsi |
|----------|------------|
| `backup_server` | Create backup/snapshot |
| `restore_server` | Restore dari backup |
| `schedule_backup` | Setup auto-backup |
| `delete_backup` | Hapus backup lama |

### 3. Monitoring & Health

| Job Type | Deskripsi |
|----------|------------|
| `health_check` | Periodic server health check |
| `metrics_collect` | Collect resource metrics (CPU, RAM, disk) |
| `alert_trigger` | Kirim alert jika threshold terpenuhi |
| `health_restart` | Auto-restart jika stuck |

### 4. Scaling

| Job Type | Deskripsi |
|----------|------------|
| `scale_up` | Tambah resource (CPU/RAM) |
| `scale_down` | Kurangi resource |
| `resize_disk` | Resize storage |

### 5. Networking

| Job Type | Deskripsi |
|----------|------------|
| `allocate_ip` | Assign public IP |
| `release_ip` | Release IP |
| `setup_firewall` | Configure firewall rules |
| `update_dns` | Update DNS records |

### 6. User Operations

| Job Type | Deskripsi |
|----------|------------|
| `reinstall_os` | Reinstall OS |
| `console_access` | Open/close console access |
| `file_operations` | Upload/download files |
| `execute_command` | Run custom commands |

### 7. Webhook & Notifications

| Job Type | Deskripsi |
|----------|------------|
| `webhook_deliver` | Kirim webhook event |
| `email_notification` | Kirim email notification |
| `sms_notification` | Kirim SMS (jika ada provider) |

### 8. Maintenance

| Job Type | Deskripsi |
|----------|------------|
| `update_software` | Update game/panel software |
| `patch_server` | Apply security patches |
| `cleanup_logs` | Clean old logs |
| `rotate_backups` | Rotate old backups |

## Job Structure

```json
{
  "job_id": "uuid-v4",
  "job_type": "create_server",
  "payload": {
    "user_id": "uuid",
    "server_id": "uuid",
    "template": "minecraft-paper",
    "location": "singapore-1",
    "resources": {
      "cpu": 4,
      "ram": 8,
      "disk": 50
    }
  },
  "user_id": "uuid",
  "priority": 10,
  "created_at": 1700000000
}
```

## Priority Queue

Worker memproses job berdasarkan priority:

| Priority Level | Queue Key | Use Case |
|----------------|-----------|----------|
| 10 (High) | `queue:jobs:high` | User actions, urgent operations |
| 0 (Normal) | `queue:jobs:normal` | Standard operations |
| -10 (Low) | `queue:jobs:low` | Background tasks, maintenance |

Polling order: High → Normal → Low

## Environment Variables

| Variable | Default | Deskripsi |
|----------|---------|-----------|
| `DATABASE_URL` | - | PostgreSQL connection string |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection string |
| `WORKER_ID` | `worker-01` | Unique worker identifier |
| `WORKER_CONCURRENCY` | `5` | Max concurrent jobs |
| `WORKER_POLL_INTERVAL_MS` | `1000` | Queue poll interval |
| `JWT_SECRET` | - | JWT signing secret |
| `APP_URL` | `http://localhost:8080` | Application URL |

## Worker Status Flow

```
┌──────────┐    ┌────────────┐    ┌───────────┐    ┌────────────┐
│ pending  │───▶│ processing │───▶│ completed │    │   failed   │
└──────────┘    └────────────┘    └───────────┘    └────────────┘
                                       │                   │
                                       └───────────────────┘
                                              (retry)
```

## API vs Worker Comparison

| Layer | Peran | Komunikasi |
|-------|-------|------------|
| **API** | Orchestrator | HTTP ke Solys Agent |
| **Worker** | Queue processor | Dequeue → Process → Agent |
| **Agent** | Executor | Langsung ke Docker |

### Flow: Start Server

1. **API** menerima request `POST /servers/:id/start`
2. API enqueue job ke Redis: `{"job_type": "start_server", "server_id": "..."}`
3. **Worker** polling Redis, dapat job
4. Worker call Solys Agent via HTTP: `POST /api/v1/servers/:id/start`
5. **Agent** execute: `docker start container_id`
6. Agent return result ke Worker
7. Worker update job status, trigger webhook

## Webhook Events

Worker mengirim webhook untuk event berikut:

| Event | Deskripsi |
|-------|------------|
| `server.created` | Server berhasil dibuat |
| `server.started` | Server berhasil distart |
| `server.stopped` | Server berhasil distop |
| `server.restarted` | Server berhasil direstart |
| `server.deleted` | Server berhasil dihapus |
| `server.backup.created` | Backup berhasil dibuat |
| `server.backup.restored` | Restore berhasil |
| `server.health.failed` | Health check gagal |
| `job.completed` | Job selesai |
| `job.failed` | Job gagal |

## Retry Policy

- Max 3 retry untuk job gagal
- Exponential backoff: 5s → 15s → 45s
- Job yang gagal >3x di-mark sebagai `dead_letter`

## Monitoring

Worker expose metrics:

- `worker_jobs_processed_total` - Total job diproses
- `worker_jobs_failed_total` - Total job gagal
- `worker_queue_size` - Jumlah job di queue
- `worker_processing_time_ms` - Waktu proses job

## Installation

```bash
# Build worker
cd worker
docker build -t escluse-worker:latest .

# Run worker
docker run -d \
  --name escluse-worker \
  -e REDIS_URL=redis://redis:6379 \
  -e DATABASE_URL=postgres://... \
  escluse-worker:latest
```

## Development

```bash
# Local development
cd worker
cargo run

# With custom config
REDIS_URL=redis://localhost:6379 \
DATABASE_URL=postgres://user:pass@localhost/panel \
cargo run
```