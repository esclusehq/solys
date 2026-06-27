# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.4.17] - 2026-06-27

### Security

- **Switch to distroless Docker image** — Base image changed from `debian:bookworm-slim` to `gcr.io/distroless/cc-debian12:nonroot`. Reduces attack surface by removing shell, package manager, and unnecessary system utilities. Container runs as non-root user (65532:65532).

- **Embed dependency SBOM in release binaries** — CI workflows now use `cargo auditable build` instead of `cargo build` to embed a full list of dependencies and their versions in every release binary, enabling vulnerability scanning without access to source code.

- **Add cargo-deny configuration** — `deny.toml` enforces license compliance, bans duplicate dependency versions, and restricts dependency sources to known registries. Allows `AGPL-3.0` (project license) plus standard permissive licenses; blocks `GPL-3.0`, `GPL-2.0`, `LGPL-3.0`, and `AGPL-1.0`.

### Added

- **Pre-signed URL upload support for backups** — Backup handler can now upload to S3-compatible storage via pre-signed URLs instead of requiring full S3 credentials, enabling zero-trust backup workflows.

- **API key moved from WS query string to Authorization Bearer header** — The API key is no longer appended as a query parameter to the WebSocket URL. Instead, it is sent as `Authorization: Bearer <key>` in the WebSocket handshake request. Prevents credential leakage in server logs and URL referrer headers.

- **Token change detection in relay client** — `RelayClientHandle` now tracks relay token changes and automatically reconnects tunnels when the token rotates, without requiring agent restart.

### Fixed

- **`handle_create` drops `env_vars` from server create payload** — `ServerCreatePayload.env` deserialized from JSON key `"env"`, but the backend sends JSON key `"env_vars"`. Renamed the struct field to `env_vars` so `TYPE`, `VERSION`, `MEMORY` and all other environment variables reach the Docker container during initial server creation. `handle_start` was not affected because it reads `payload["env_vars"]` directly from `serde_json::Value` rather than via the struct.

- **Dead code removed across multiple modules** — Eliminated unused imports, variables, and functions identified during protocol cleanup (I-01 through I-04).

- **`production-path` unwrap() calls replaced with proper error handling** — Production code paths that could panic via unwrap() now return errors and are handled gracefully (I-05).

- **`send().await` error handling with distinguished variants** — WebSocket send errors are now properly classified and handled instead of being silently ignored (I-06).

- **RelayConfigSync handler reordered to prevent DNS record flapping** — DNS_CONFIG removal now happens before `set_servers` with a drain-delay, preventing temporary DNS record deletion when relay config is updated.

- **EULA=TRUE limited to Minecraft images only** — The `EULA=TRUE` environment variable is now only set for Minecraft Docker images, preventing accidental license acceptance on non-Minecraft containers.

## [v0.4.16] - 2026-06-24

### Fixed

- **Agent ignores `mc_version` when creating Docker containers** — `DeployConfig.version` was never forwarded from `agent_connection.rs` to the runtime payload, and both `handle_create` and `handle_start` in `runtime.rs` ignored the incoming `version` field. The Docker container always received `VERSION=LATEST` regardless of the version the user selected. Minecraft server `test` (created with version 1.21.1) was running version 26.2 because `LATEST` resolved to the latest release. Fixed by forwarding `config.version` as `payload["version"]` in `agent_connection.rs` and inserting `VERSION=<version>` into the container's environment variables in `runtime.rs`.

## [v0.4.15] - 2026-06-14

### Fixed

- **Relay tunnel flapping when Docker container not ready** — `run_relay_client()` now calls `wait_for_container_ready()` before starting the relay tunnel, polling Docker container status up to 30s (every 2s) until the container is running. Prevents "Connection reset without closing handshake" loop when the relay tunnel starts before the Minecraft server container has finished creating.

## [v0.4.14] - 2026-06-13

### Changed

- **Log verbosity reduced for dev** — Demoted 38 noisy `info!()` calls to `debug!()` or `trace!()` across 14 files (heartbeat audit, metrics, DNS ops, yamux, reconnect backoff, writer exit, log polling, task results, connectivity probes). Added `EnvFilter` support (`RUST_LOG` env var) for runtime control — default `info` shows only useful messages, `RUST_LOG=debug` shows internal details, `RUST_LOG=trace` shows everything. See `tools/logsum/` for a standalone log filter/summarizer.

## [v0.4.13] - 2026-06-13

### Fixed

- **UDP (Bedrock) tunnel disconnected in loop after relay gateway restart** — Agent's `drive_inbound_streams` used `127.0.0.1:19132` for UDP (skipped Docker container IP resolution), but Bedrock container UDP ports are not published to the Docker host. Fixed by always resolving the container IP via Docker inspect for both TCP and UDP, so UDP datagrams reach the Bedrock server via the Docker bridge network.

## [v0.4.12] - 2026-06-13

### Added

- **Bedrock UDP relay via TLV framing (Phase 73)** — Agent's `run_udp_relay_session` sends/receives UDP datagrams framed as TLV (type `0x01` for data) to/from the relay gateway. `TunnelConnect` and `RelayServerConfig` carry a `loader` field so the agent can distinguish Bedrock (UDP) from Java (TCP) relays. `drive_inbound_streams` passes `is_udp = (loader == "bedrock")` so the relay session opens a `UdpSocket` toward the gateway instead of a TCP tunnel. `relay_client.rs` maps the `loader` string to the `bedrock` boolean for the relay gateway's tunnel request.

- **Multi-server relay tunnel manager (Phase 70, revised)** — Agent replaced singleton `RelayRuntime` + per-server `PerServerRuntime` with `RelayManager` (process-global via `OnceLock`). `RelayManager::set_servers()` implements diff-based lifecycle: starts/stops/restarts tunnels atomically from `RelayConfigSync` WS push. No more `AGENT_RELAY_TOKEN` env-var bootstrap; all relay config arrives via backend WS. `relay.connect` and `relay.disconnect` tasks deprecated — `RelayConfigSync` is the single source of truth. `run_relay_client()` takes `RelayServerConfig` directly (no shared config lookup). Backend `create_server` / `delete_server` call `push_relay_config()` to notify agent.

### Added

- **Minecraft Bedrock Edition server type (Phase 72)** — Agent now detects `mc_loader == "bedrock"` and dynamically selects `itzg/minecraft-bedrock-server` Docker image with UDP port binding (`/udp` instead of `/tcp`) and no RCON. `agent_connection.rs` forwards `loader` field from DeployConfig to runtime task payload and uses `game_port` as port map key instead of hardcoded `"25565"`. `runtime.rs` `handle_create` and `handle_start` dispatch UDP protocol when loader is "bedrock", backward-compatible TCP fallback for all other types.

- **Per-server DNS records (`<server>.<global_subdomain>.<wildcard_domain>`)** — `CloudflareDnsConfig` now carries `extra_subdomains: Vec<String>` and the DDNS watcher keeps every record in sync alongside the global one. Backend computes the per-server subdomains from `servers.public_host` and ships them in `NodeMessage::DnsConfig` on every Cloudflare save and on every reconnect. Watcher creates/updates each A record (`<sub>.<global_sub>.<wildcard>`) on every IP change, not just the global one. Example: server "mantap wou" gets `mantap-wou.play.esluce.com` while the global `play.esluce.com` continues to update. Status task (`dns.status`) now reports `per_server_domains` for the dashboard.

### Fixed

- **`RelayConfigSync` handler recreates relay A records from DNS config** — `RelayConfigSync` handler now removes relay subdomains from `DNS_CONFIG.extra_subdomains` so the DnsWatcher doesn't delete and recreate the relay's A records on every 300s polling cycle, keeping the relay IP stable.
- **Docker bridge port collision when host mapping differs from container port** — `resolve_container_addr()` now resolves the container's internal port (25565) via Docker inspect instead of using the Docker host port from `local_mc_addr`, fixing port mismatch when containers use non-standard host port mappings.
- **`RelayConfigSync` silently ignored by agent** — `serde_json::from_str::<BackendMessage>` falls through silently on parse failure (no error log in `else` branch). Added `warn!` logging with raw JSON to diagnose why the relay config sync message is not being processed.
- **Windows x86_64 cross-compile fails with "cannot find -lPacket"** — `pnet_sys` (transitive dep via `upnp-rs`) links `Packet.lib` from Npcap/WinPcap, which is not available in the cross-compilation toolchain. Added a CI step that downloads the Npcap SDK, creates a MingW-compatible `libPacket.a` via `dlltool`, and sets `RUSTFLAGS=-L /tmp/npcap-lib` so the linker resolves `-lPacket`. Applied to all 3 workflows (canary, ci, release).
- **DnsWatcher never syncs DNS when config arrives after first tick** — `check_and_update` returned early when IP did not change, so if `CloudflareDnsConfig` was received from the backend *after* the initial DnsWatcher tick (which is the normal startup sequence), the per-server A records were never created and existing records were never refreshed until the next IP change. Removed the IP-change guard so DNS records are always synced on every polling cycle (every 300s).
- **RelayClient default gateway URL uses unregistered domain `esluce.net`** — `bootstrap_relay_client` defaulted to `wss://relay.esluce.net/relay/tunnel`, but `esluce.net` is not registered (NXDOMAIN). Changed default to `wss://relay.esluce.com/relay/tunnel`.
- **`install.sh` prompts for API key even when `AGENT_API_KEY` is set** — `generate_config()` always called `_prompt` for the API key, ignoring `$AGENT_API_KEY`. Added a `$AGENT_API_KEY` check before falling through to `_prompt`, so `sudo env AGENT_API_KEY=xxx bash -c "$(curl -fsSL https://get.esluce.com/latest/install.sh)"` works non-interactively.
- **Monorepo separation** — `compose/`, `docker/`, `opt/` moved to `esclusehq/escluse-infra`. Orphaned gitlink `migration` and leftover `api/` file removed from tracking. `.gitignore` updated. See PUSH_COMMIT.md for full repo mapping.
- **Relay gateway not binding UDP port for Bedrock servers** — `TunnelConnect` message in the relay client was not including the `loader` field, so the relay gateway never detected the server as Bedrock and skipped UDP port binding. Added `loader: Option<String>` to `RelayServerConfig` (state.rs), `ServerRelayInfo` (agent_connection.rs), and the `TunnelConnect` payload (relay_client.rs). The relay gateway now correctly binds UDP port 19132 when the agent sends `"loader": "bedrock"`.

## [v0.4.6] - 2026-06-06

### Fixed

- **Orphaned outbound mpsc channel — task results, log output, progress events never reach the backend** — `src/agent_connection.rs:275` created `mpsc::channel(100)` and bound the receiver as `_ws_rx` (underscore-prefixed variable binding, not a true drop), so the receiver was alive but **no task ever drained it**. `ResultSender::send`, `send_progress`, `send_log_output` all called `try_send` on a channel with no consumer, which returned `Full` after 100 messages and triggered the disk-buffering fallback for **every** task result. The frontend server detail page consequently showed "Address: —" and "No logs available" because (a) log query responses were buffered to disk and never replayed, and (b) public_host was being auto-set in the DB by `bulk_set_public_hosts_if_null` but the front-end DTO didn't include it. Rewrote the outbound pipeline: (1) the channel is now created inside the inner reconnect loop with capacity 1000 (room for log bursts), (2) a dedicated `writer` task owns both the tungstenite `SplitSink` and the channel receiver, (3) every send path — `Register`, `Heartbeat`, `CommandResponse`, `TaskResult`, `LogOutput`, `TaskProgress`, `CrashReport`, and `Pong` — goes through the same channel, (4) `ResultSender::send` uses `send().await` (proper async backpressure) for `TaskResult` while `send_progress` / `send_log_output` use `try_send` (fire-and-forget) so a slow writer never stalls log capture, (5) `update_sender` is now `async fn` with `lock().await` so reconnects reliably install the fresh sender. Writer exits on WS send error and the inner loop's heartbeat-timeout branch (5s) covers the case where the channel stalls, both of which break the inner loop and trigger the existing reconnect logic.
- **Templates empty in CreateServerModal when DB migration not applied** — `SqlxTemplateRepository` (`list_templates`, `list_templates_by_game`, `list_public_templates`) now catches SQL errors and returns the hardcoded `Template::fallback()` set instead of 500 INTERNAL_ERROR, mirroring the prior fix for `plugin_templates`. The `templates` table seed migration (`20260531_create_templates_table.sql`) is missing from the `migration/` directory on some deployments, so the table doesn't exist and users saw the Game Type dropdown fall back to "Minecraft" + 3 disabled "Coming Soon" options regardless of plan/role. Fix: defensive fallback at the repository layer. (session: `templates-server-details-empty`)
- **CreateServerModal Variant dropdown broken** — Was reading `t.variant` (undefined) and `template.default_port` (undefined) from the regular templates DTO, which exposes `category` and nests `default_port` inside `config`. Replaced with `t.category` and `template.config?.default_port` so the Variant dropdown renders the actual built-in variants (vanilla/paper/spigot/forge/fabric) and auto-fills the default port.
- **WebSocket reconnect stuck after server-side close** — Inner `tokio::select!` loop in `src/agent_connection.rs` could not exit on a dead WS: (1) `Ok(Message::Close(_))` was silently swallowed so the server's close frame did not break the loop, and (2) heartbeat `let _ = ws_sender.send(...).await` ignored send errors, and because the heartbeat branch is always-ready it kept firing forever without ever checking whether the WS was still alive. Combined with the select branch order, once the backend killed the WS the agent process appeared running (heartbeat logs every 30s) but the outer reconnect loop never iterated, so `connect_async` was never called again. Symptom on node `d0110884-2d39-4bad-907c-d686affa35f9`: agent stuck for 2+ hours, no reconnect despite manual restarts. Fix: handle Close frame (log + break), wrap heartbeat send in `tokio::time::timeout(5s)` with err/timeout breaking the inner loop, wrap `connect_async` in `tokio::time::timeout(15s)` so a hung TCP/WS handshake cannot block forever, add explicit `warn!` log on `else => break`, and add a `reconnect_attempt` counter + per-iteration info log for observability.

## [v0.4.2] - 2026-06-05

### Fixed

- **Agent rejects all `file.*` commands with "Unknown task type: unknown"** — The WebSocket command → task_type mapping in `src/agent_connection.rs` only recognized the short form of command names (e.g. `read_file`), but the dashboard sends the long form (`file.read_file`) and the previously added `sftp.upload` / `sftp.download` aliases. Every file op (and therefore every terminal session that needs `server.properties` to read the RCON port/password) fell through to the `_ => "unknown"` arm, which in turn caused the in-app terminal to reconnect in a tight loop while trying to fetch the RCON config. Added long-form aliases for all 7 file operations and for `server.*` to keep the mapping symmetric. Terminal "Disconnected - Reconnecting..." loop on Minecraft servers is now resolved.
- **RCON `server.command` failed with "missing field 'command'"** — `CommandParams` (`src/agent_connection.rs:142`) did not declare `command`, `rcon_port`, or `rcon_password`, so serde silently dropped them from incoming WebSocket messages, and the payload builder never copied them into the outgoing `Task` payload. The `rcon::ServerCommandPayload` struct requires `command: String`, so every keystroke in the in-app terminal returned `Error: TASK_FAILED: [TASK_FAILED] missing field 'command'`. Added the three fields to `CommandParams` and the missing `command` / `rcon_port` / `rcon_password` / `new_name` / `backup_path` extractions to the payload builder. Terminal commands now execute end-to-end.
- **RCON connect always targeted 127.0.0.1 (in-container RCON unreachable)** — `handlers::rcon::handle_command` hardcoded `SocketAddr::from(([127, 0, 0, 1], rcon_port))` for the RCON TCP connect, but the Minecraft server runs inside a Docker container that is NOT exposing its RCON port to the host. Every RCON command returned `Failed to connect to RCON server: Connection refused`. Rewrote the handler to resolve the RCON host in priority order: (1) explicit `host` in the payload, (2) Docker inspect of `container_name` / `container_id` returning the first non-empty IP from `NetworkSettings.Networks`, (3) warn-and-fallback to 127.0.0.1. Added `container_name` and `host` to `ServerCommandPayload` and `CommandParams`. The dispatcher in `handlers::mod` now passes `runtime` to the rcon handler so it can talk to bollard.

### Changed

- **CI workflow hardened** — `ci.yml` now runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all`, and `rustsec/audit-check` in addition to the matrix build. Added `Swatinem/rust-cache` and `timeout-minutes` to all build jobs. Added `concurrency` group so PRs cancel superseded runs.
- **Canary & Release workflows hardened** — Added `Swatinem/rust-cache` to all build jobs, `timeout-minutes` per job, and a `concurrency` group. Canary artifact upload now uses `compression-level: 0` to match the release pipeline (prevents the OOM-on-upload class of failures that previously bit canary).

## [v0.4.1] - 2026-06-03

### Fixed

- **DNS resolution in Docker containers** — Agent now passes `8.8.8.8` and `1.1.1.1` as explicit DNS servers when creating and starting containers, fixing Minecraft server startup failures caused by Tailscale search domain overriding DNS lookups.

## [v0.4.0] - 2026-06-02

### Added

- **File operations** — 7 new handlers for container file management: list_dir, read_file, write_file, delete, mkdir, rename, copy
- **File task routing** — WebSocket command mapping for file ops in agent_connection.rs
- **Trigger canary+CI on master branch** — workflow now runs on push to `master` too
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