# 🔒 Security Audit: `agent/solys/`

**Date:** 2026-06-25
**Scope:** Semua source file di `agent/solys/src/` (~290 KB Rust), `Cargo.toml`, `Dockerfile`
**Fokus:** WebSocket handling, Docker container management, command execution, relay tunnel connections, secret/token management

---

## 📋 CRITICAL (7)

---

### C-1: Command Injection via `container_name` — Semua `docker exec` Calls

**Severity:** Critical
**Files:** `src/handlers/files.rs:29-33`, `runtime.rs:224-228`, `metrics.rs:171-172`, `backup.rs:109-110`

**Deskripsi:**
Field `container_name` berasal langsung dari WebSocket `ExecuteCommand` payload (backend-message) tanpa **sanitasi apapun**. String ini diinterpolasi ke `docker exec <container_name> <command>`. Attacker yang compromise backend — atau middlebox — dapat inject arbitrary shell metacharacters (`` ` $ ; | & ``) via container names.

```rust
// files.rs:29 — UNSAFE
let output = Command::new("docker")
    .args(["exec", &container_name, "ls", "-la", ...])
```

**Rekomendasi:**
Validasi `container_name` terhadap regex `^[a-zA-Z0-9_.-]+$` sebelum digunakan di Docker exec. Gunakan bollard API (`inspect_container`, `exec`) daripada `Command::new("docker")`.

---

### C-2: No Authentication pada Local HTTP API

**Severity:** Critical
**Files:** `src/main.rs:285`, `src/api/routes.rs:28-47`, `src/service_main.rs:155`

**Deskripsi:**
HTTP API server (`0.0.0.0:8642`) bind ke **semua interface** dengan **zero authentication**. Proses apapun di network yang sama (termasuk container compromised di Docker bridge network) bisa akses `/stop`, `/restart`, `/config`, `/logs`, `/events`.

```rust
// main.rs:285
let addr: std::net::SocketAddr = "0.0.0.0:8642".parse().unwrap();
```

**Rekomendasi:**
1. Bind ke `127.0.0.1:8642` saja
2. Tambah authentication middleware (bearer token dari config)
3. Atau ganti ke unix socket

---

### C-3: API Key di WebSocket Query String

**Severity:** Critical
**Files:** `src/agent_connection.rs:309-314`

**Deskripsi:**
API key ditambahkan sebagai `?api_key=<value>` di cleartext ke WebSocket URL. Ini leak di:
- Process listing (`ps aux`)
- Backend access logs (URL penuh tercatat)
- HTTP `Referer` header
- TLS-terminating proxies yang log full URLs

```rust
// agent_connection.rs:311
return format!("{}?api_key={}", base, urlencode(key));
```

**Rekomendasi:**
Kirim API key sebagai `Sec-WebSocket-Protocol` header atau `Authorization: Bearer` header, bukan query parameter.

---

### C-4: S3 Credentials di WebSocket Payload (In-Transit Exposure)

**Severity:** Critical
**Files:** `src/handlers/backup.rs:188-192`

**Deskripsi:**
`s3_access_key` dan `s3_secret_key` dikirim dari backend ke agent di dalam task payload melalui WebSocket. Kredensial ini terlihat oleh siapapun yang capture WS traffic. Juga diserialisasi ke `task_state` dan berpotensi tersimpan di disk.

```rust
// backup.rs:186-192 (BackupStartPayload)
pub s3_access_key: Option<String>,
pub s3_secret_key: Option<String>,
```

**Rekomendasi:**
1. Gunakan pre-signed URLs (short-lived) daripada credentials mentah
2. Atau proxy upload melalui backend
3. Minimal: zeroize credentials setelah dipakai dan jangan persist ke disk

---

### C-5: SSH Private Key Ditulis ke `/tmp` World-Readable

**Severity:** Critical
**Files:** `src/handlers/ssh.rs:148-163`

**Deskripsi:**
SSH private key dari WebSocket payload ditulis ke `/tmp/ssh_key_*` dengan default permission (world-readable). Jika proses crash antara `write` dan `remove_file`, private key tertinggal di disk selamanya.

```rust
// ssh.rs:152
std::fs::write(&key_path, key_content)?;
// ... connect ...
let _ = std::fs::remove_file(&key_path);  // crash disini → key leak
```

**Rekomendasi:**
1. Gunakan `std::os::unix::fs::PermissionsExt` untuk set `0o600`
2. Tulis ke `tempfile::NamedTempFile` dengan auto-cleanup via Drop
3. Pertimbangkan passing via stdin pipe daripada filesystem

---

### C-6: `overflow-checks = false` — Silent Integer Wrapping

**Severity:** Critical
**Files:** `Cargo.toml:124`

**Deskripsi:**
`overflow-checks = false` di release profile menyebabkan integer overflow wrap silently (two's complement) daripada panic. Dengan penggunaan ekstensif `as u64`, `as i64`, dan arithmetic di `metrics.rs`, `relay_session.rs`, `relay_client.rs`, ini bisa silently corrupt health checks, byte counters, dan memory limits.

```toml
# Cargo.toml:124
overflow-checks = false
```

**Rekomendasi:**
Hapus line ini (Rust default mengaktifkan overflow checks di debug). Atau audit semua arithmetic dan gunakan `.saturating_add()`, `.checked_mul()` secara eksplisit.

---

### C-7: UPnP SSDP Discovery Tidak Di-Rate-Limit (Amplification Risk)

**Severity:** Critical
**Files:** `src/handlers/connectivity/upnp.rs:52,122`

**Deskripsi:**
Setiap `upnp.add_mapping` / `upnp.remove_mapping` melakukan SSDP discovery via `upnp-rs::search::search_once`. Tidak ada rate limiting — attacker atau backend bug bisa trigger SSDP floods, yang dikenal sebagai vektor DDoS amplification.

```rust
// upnp.rs:52
let responses = tokio::task::spawn_blocking(move || search::search_once(opts))
```

**Rekomendasi:**
Cache IGD discovery results minimal 60 detik. Tambah global UPnP operation rate limit (misal max 1 per 10 detik).

---

## ⚠️ WARNING (10)

---

### W-1: Cloudflare API Token di Global Mutable State

**Severity:** Warning
**Files:** `src/handlers/dns.rs:38-43`

**Deskripsi:**
`DNS_CONFIG` global menyimpan Cloudflare API token dalam plaintext. Modul manapun bisa membaca via `DNS_CONFIG.read().await`. Token juga di-clone setiap kali diakses (`.clone()` di `dns_watch.rs:104`), memperbanyak exposure di memory.

```rust
lazy_static! {
    pub static ref DNS_CONFIG: Arc<RwLock<Option<CloudflareDnsConfig>>> =
        Arc::new(RwLock::new(None));
}
```

**Rekomendasi:**
Gunakan `Zeroizing<String>` dari `zeroize` crate. Simpan hanya yang diperlukan untuk DNS ops.

---

### W-2: Tidak Ada Docker Container Security Restrictions

**Severity:** Warning
**Files:** `src/handlers/runtime.rs:147-169, 309-334`

**Deskripsi:**
Container yang dibuat tidak punya `ReadonlyRootfs`, `CapDrop`, `SecurityOpt` (seccomp/apparmor). Minecraft server di dalam container punya akses penuh ke Docker host via Docker socket jika ter-mount.

```rust
// runtime.rs:147
let mut host_config = HostConfig {
    network_mode: Some("bridge".to_string()),
    ..Default::default()
};
```

**Rekomendasi:**
Tambah di `HostConfig`:
```rust
ReadonlyRootfs: Some(true),
CapDrop: Some(vec!["ALL"]),
SecurityOpt: Some(vec!["no-new-privileges:true"]),
```

---

### W-3: Relay Token di Audit Logs (Unredacted)

**Severity:** Warning
**Files:** `src/handlers/relay_client.rs:247-253`

**Deskripsi:**
Relay token dikirim ke gateway dalam plaintext via `TunnelConnect` JSON. Audit log `log_relay_tunnel_event` menyimpan `detail` yang mencakup subdomain; token bisa tercatat di log jika error propagasi terjadi.

**Rekomendasi:**
Pastikan `relay_token` tidak pernah masuk ke `info!` / `error!` / `tracing::*` arguments. Pertimbangkan HMAC-based ephemeral tokens.

---

### W-4: Weak Jitter Source untuk Backoff

**Severity:** Warning
**Files:** `src/handlers/relay_client.rs:592-600`

**Deskripsi:**
`backoff_with_jitter` menggunakan `SystemTime::subsec_nanos()` — granularitas sub-millisecond dan predictable — bukan CSPRNG. Ini memungkinkan timing side-channel attack dan membuat pola reconnection predictable.

```rust
let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.subsec_nanos() as u64)
    .unwrap_or(0);
```

**Rekomendasi:**
Gunakan `rand::thread_rng().gen_range()` yang sudah di-import di `relay_client.rs:51`.

---

### W-5: SSH Connection Cache — Tidak Pernah di-Dispose Properly

**Severity:** Warning
**Files:** `src/handlers/ssh.rs:68-131`

**Deskripsi:**
`SshConnectionCache` menyimpan `SshClient` dengan `client.clone()`. Saat connection di-remove, `SshClient` di-drop tanpa explicit `client.disconnect()`. Koneksi di-cache hingga `max_connections` (default 5) selama 60 detik idle.

**Rekomendasi:**
Panggil `client.disconnect()` secara eksplisit saat eviction/removal. Tambah cleanup di graceful shutdown.

---

### W-6: Path Traversal via `path` Parameter

**Severity:** Warning
**Files:** `src/handlers/files.rs:23-27, 57, 80, 119-120`

**Deskripsi:**
`path` dari WebSocket payload di-prepend dengan `/data` dan digunakan di `docker exec`. Path traversal seperti `../../etc/passwd` menjadi `/data/../../etc/passwd` yang resolve ke luar `/data/`.

```rust
// files.rs:24
} else if path.starts_with('/') {
    format!("/data{}", path)  // /data/../etc → /etc
}
```

**Rekomendasi:**
Canonicalize dan verifikasi resolved path berada di bawah `/data/` menggunakan `std::fs::canonicalize` atau pattern matching.

---

### W-7: Container Name Lookup via `docker ps` Subprocess

**Severity:** Warning
**Files:** `src/handlers/runtime.rs:224-228, 371-374, 419-422, 467-469`

**Deskripsi:**
Container lookup menggunakan `docker ps --filter name=^{container_name}` — regex anchor `^...$` tidak mencegah pencocokan container dengan nama yang mengandung string sebagai prefix.

**Rekomendasi:**
Gunakan `bollard::Docker::list_containers` dengan filters daripada shell out. Docker client sudah tersedia via `runtime.docker()`.

---

### W-8: Rate Limiter Menggunakan `Instant::now()` — Rentan System Clock Adjustments

**Severity:** Warning
**Files:** `src/rate_limit.rs:122-127`

**Deskripsi:**
Token bucket refill menggunakan `Instant::now()` (monotonic di Linux baik-baik saja), tapi rate limiter **shared** untuk semua user dan task types. Satu high-volume user bisa exhaust global agent bucket dan deny service ke yang lain. Parameter `user_id` tidak pernah dipakai (`None` terus di call site).

**Rekomendasi:**
Gunakan per-user/per-task-type rate limiters dari config. Implementasikan `user_id` yang benar.

---

### W-9: DNS Watch Per-Server Subdomain Cleanup Race Condition

**Severity:** Warning
**Files:** `src/agent_connection.rs:919-932`

**Deskripsi:**
Saat `RelayConfigSync`, relay subdomains di-remove dari DNS watcher's `extra_subdomains`. Tapi DNS watcher mungkin **sedang** menjalankan `check_and_update()` — membuat/update A records untuk subdomains tersebut (pointing ke agent IP lokal). Race condition ini bisa break relay routing sementara.

**Rekomendasi:**
Gunakan `tokio::sync::RwLock` guard dengan lock ordering yang konsisten. Atau tambah drain-delay sebelum start relay tunnels.

---

### W-10: `unzip` pada Restore Backup Bisa Zip Slip

**Severity:** Warning
**Files:** `src/handlers/backup.rs:359-362`

**Deskripsi:**
Ekstraksi backup archive dengan `tar -xzf` ke temporary directory, lalu file di-copy ke container. Jika archive mengandung symlink atau path traversal (`../../../etc/cronjob`), file bisa tertulis di luar target directory.

**Rekomendasi:**
Gunakan Rust native `tar` crate dengan path traversal filtering daripada shell out ke `tar` command.

---

## ℹ️ INFO (8)

---

### I-1: Dead Code — Backup Handlers Tidak Terdaftar di Dispatcher

**File:** `src/handlers/backup.rs:315-394, 422-500`

`handle_restore` dan `handle_restore_s3` didefinisikan tapi **tidak pernah dipanggil** dari `handlers/mod.rs`.

---

### I-2: Task Queue Module Tidak Dipakai

**File:** `src/task_queue.rs`

Struct `TaskQueue` dan fungsi `execute_with_priority` didefinisikan tapi tidak ada dispatcher yang mereferensi — eksekusi langsung di `agent_connection.rs`.

---

### I-3: Redundant Shutdown Checks di Reconnect Loop

**File:** `src/agent_connection.rs:1012-1015, 1019-1022`

Dua `shutdown.load()` identik dalam 5 baris — yang kedua dead code.

---

### I-4: `#[allow(dead_code)]` Global di Beberapa File

**File:** `src/handlers/mod.rs:3`, `src/api/routes.rs:3`, `src/agent/result_sender.rs:3`

Tiga file suppress dead code warnings global daripada annotate item spesifik. Bisa menyembunyikan kode yang memang tidak terpakai.

---

### I-5: `unwrap()` di Production Paths

**Files:** `src/main.rs:285`, `src/service_main.rs:158`, `src/handlers/runtime.rs:254`, `src/agent_connection.rs:469`

Beberapa `unwrap()` di non-test paths — `"0.0.0.0:8642".parse().unwrap()` akan crash jika port sudah dipakai, `node_id.lock().unwrap()` bisa poison mutex.

---

### I-6: Tokio Channel Errors Discarded Silent

**File:** `src/agent/result_sender.rs:150,175-178`

Error dari `send().await` di-discard dengan `let _ =` tanpa dibedakan (channel closed vs full vs disconnected). Bisa menyembunyikan masalah koneksi.

---

### I-7: Relay Token Refresh Tidak Terdeteksi

**File:** `src/state.rs:182-249`

`RelayManager::set_servers` deteksi config change oleh subdomain/port/addr comparison tapi **tidak** oleh token change. Jika relay token dirotasi di backend, agent tidak reconnect sampai full session restart.

---

### I-8: `EULA=TRUE` Hardcoded untuk Semua Container

**File:** `src/handlers/runtime.rs:324-326`

```rust
env_vec.push("EULA=TRUE".to_string());
```

Minecraft EULA dipaksa untuk semua container — bahkan image non-Minecraft. Pertimbangkan cek image type dulu.

---

## 📊 Summary

| Severity | Count | Key Areas |
|----------|-------|-----------|
| **Critical** | 7 | Command injection, no API auth, WS credential leak, SSH key exposure, overflow off, UPnP SSDP flood |
| **Warning** | 10 | Token exposure global state, container hardening, path traversal, race condition, weak jitter |
| **Info** | 8 | Dead code, unwraps, config gaps, ignored errors, hardcoded EULA |

**Total:** 25 findings
**Source:** ~290 KB Rust across 30+ source files di `src/`
