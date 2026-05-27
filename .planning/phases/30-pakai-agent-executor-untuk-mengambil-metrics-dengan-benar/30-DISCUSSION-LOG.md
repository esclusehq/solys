# Phase 30: pakai agent executor untuk mengambil metrics dengan benar - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 30-pakai-agent-executor-untuk-mengambil-metrics-dengan-benar
**Areas discussed:** Executor Type, Container Naming, Fallback Strategy, Server Config

---

## Executor Type

| Option | Description | Selected |
|--------|-------------|----------|
| Ubah ke agent executor | Ganti executor_type server ke 'agent' agar pakai AgentServerExecutor untuk metrics | ✓ |
| Tambah Docker executor | Buat DockerServerExecutor baru yang langsung akses Docker tanpa agent | |
| Biarkan seperti sekarang | Tetua pakai yang sekarang - cukup untuk sementara | |

**User's choice:** Ubah ke agent executor

---

## Container Naming

| Option | Description | Selected |
|--------|-------------|----------|
| mc-{server_id} (Recommended) | pattern sekarang | ✓ |
| devnode-{server_id} | konsisten dengan SSH executor | |
| custom dari DB | Nama kustom dari server.container_name | |

**User's choice:** mc-{server_id} (Recommended)

---

## Fallback Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| SSH executor (fallback) | Kembali ke SSH executor jika agent disconnected - rumit untuk SaaS | |
| Return zeros (Recommended) | Return 0 values jika agent disconnected - simple | |
| Skip entirely | Tidak simpan metrics jika agent disconnected | |

**User's choice:** Return zeros (Recommended) → user clarification: "buat menjadi 'not connected to agent'"

---

## Server Config

| Option | Description | Selected |
|--------|-------------|----------|
| Ya, ubah sekarang | Langsung ubah di DB via API call | ✓ |
| Tidak, biarkan manual | Biarkan user ubah manual di dashboard | |

**User's choice:** Ya, ubah sekarang

---

## Deferred Ideas

No deferred ideas — discussion stayed within phase scope
