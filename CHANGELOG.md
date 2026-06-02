# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **File operations** — 7 new handlers for container file management: list_dir, read_file, write_file, delete, mkdir, rename, copy
- **File task routing** — WebSocket command mapping for file ops in agent_connection.rs
- **CI/CD release pipeline** — GitHub Actions workflow with 6 jobs: validate, build (3-platform matrix), package, sign (cosign), upload to R2, update versions.json
- **Canary builds** — push-to-main triggers build + upload to `canary/` R2 path
- **PR checks** — CI workflow with 3-platform build matrix, no upload
- **Debian packaging** — `.deb` package generation via `dpkg-deb` for amd64 + arm64
- **RPM packaging** — `.rpm` package generation via `rpmbuild` for x86_64 + aarch64
- **Version manifest** — `update-manifest.sh` generates `versions.json` with checksums per platform
- **NSIS installer CI compatibility** — `IfFileExists` guard for optional GUI binary
- **Linux/macOS installer** — `install.sh` with platform detection, SHA256 + cosign verification, `ESCLUSE_BIN_DIR` override
- **Windows installer** — `install.ps1` with `Get-FileHash` verification, `ProgramFiles` install, User PATH update

## [v0.0.1] - 2026-05-12

### Added

**Core binary** — `escluse-agent` main binary with full agent capabilities.

**Service binary** — `escluse-service` alternative entry point for system service mode.

#### Dependencies on Agent Core Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `agent-proto` | 0.0.1 | Task protocol, WebSocket messages |
| `agent-config` | 0.0.1 | Configuration loading and validation |
| `agent-security` | 0.0.1 | JWT validation, rate limiting |
| `agent-event` | 0.0.1 | Event bus for task lifecycle |
| `agent-health` | 0.0.1 | Circuit breaker, heartbeats |
| `agent-capability` | 0.0.1 | Capability registry |
| `agent-task` | 0.0.1 | Task queue and dispatcher |
| `agent-metrics` | 0.0.1 | System metrics collection |
| `agent-runtime` | 0.0.1 | Docker runtime detection |
| `agent-ssh` | 0.0.1 | SSH client, connection pooling |
| `agent-backup` | 0.0.1 | Compression, backup storage |
| `agent-rcon` | 0.0.1 | RCON protocol client |

#### Async & HTTP

| Dependency | Version | Purpose |
|------------|---------|---------|
| `tokio` | 1 (full) | Async runtime |
| `tokio-tungstenite` | 0.26 | WebSocket client |
| `tokio-stream` | 0.1 | Stream utilities |
| `futures-util` | 0.3 | Async utilities |
| `axum` | 0.8 | Local HTTP API server |
| `tower` | 0.5 | Middleware |
| `hyper` | 1 | HTTP core |

#### Docker & Storage

| Dependency | Version | Purpose |
|------------|---------|---------|
| `bollard` | 0.18 | Docker API client |
| `rusoto_core` | 0.48 | AWS S3 SDK |
| `rusoto_s3` | 0.48 | S3 storage for backups |

#### Logging

| Dependency | Version | Purpose |
|------------|---------|---------|
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | JSON + env-filter output |
| `tracing-appender` | 0.2 | File logging |

#### Utilities

| Dependency | Version | Purpose |
|------------|---------|---------|
| `serde` / `serde_json` | 1 | Serialization |
| `uuid` | 1 | Unique identifiers |
| `chrono` | 0.4 | Date/time |
| `thiserror` | 2 | Error handling |
| `anyhow` | 1 | Context-aware errors |
| `toml` | 0.8 | TOML config files |
| `notify` | 6 | Config file watcher |
| `dirs` | 5 | XDG data directory |
| `sysinfo` | 0.32 | System information |
| `libz-sys` | 1 | Compression |

#### Windows Support

- `winapi` — Windows API bindings
- `windows-service` — Service registration
- `tray-item` — System tray
- `winit` — Window management
- `softbuffer` — Window compositing (X11, Wayland)
- `image` — PNG rendering

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `AGENT_BACKEND_URL` | `wss://app.esluce.com/api/ws/node` | WebSocket URL |
| `AGENT_API_KEY` | — | Required API key |
| `AGENT_NAME` | `my-local-agent` | Agent identifier |
| `AGENT_RUNTIME` | `auto` | docker, podman, or auto |
| `AGENT_HEARTBEAT_INTERVAL` | `30` | Heartbeat interval (seconds) |
| `AGENT_RECONNECT_INITIAL` | `2` | Initial reconnect delay |
| `AGENT_RECONNECT_MAX` | `120` | Maximum reconnect delay |
| `AGENT_MAX_CONCURRENT` | `10` | Max concurrent tasks |
| `AGENT_TASK_TIMEOUT` | `300` | Task timeout (seconds) |
| `AGENT_METRICS_INTERVAL` | `60` | Metrics reporting interval |
| `LOG_LEVEL` | `info` | trace, debug, info, warn, error |
| `LOG_FORMAT` | `text` | text or json |

### Architecture

```
┌─────────────────────────────────────────┐
│          Escluse Backend               │
│         (app.esluce.com)                │
└────────────────────┬────────────────────┘
                     │ WebSocket (wss://)
┌────────────────────▼────────────────────┐
│              Solys Agent                │
│  ┌────────────────────────────────────┐ │
│  │  escluse-agent (binary)            │ │
│  │  escluse-service (service binary)  │ │
│  └────────────────────────────────────┘ │
│  ┌────────────────────────────────────┐ │
│  │  Agent Core Crates (12 crates)     │ │
│  └────────────────────────────────────┘ │
│  ┌────────────────────────────────────┐ │
│  │  Local HTTP API (axum) :8080       │ │
│  └────────────────────────────────────┘ │
└────────────────────┬────────────────────┘
                     │
           ┌──────────┼──────────┐
           ▼          ▼          ▼
      ┌────────┐  ┌────────┐  ┌────────┐
      │ Docker │  │ Podman │  │  SSH   │
      └────────┘  └────────┘  └────────┘
```

### Build

```bash
# Development
cargo build

# Production
cargo build --release

# Run
cargo run --release

# Docker
docker build -t escluse/solys:latest .
docker run -d --env-file .env \
  -v /var/run/docker.sock:/var/run/docker.sock \
  escluse/solys:latest
```

### License

MIT