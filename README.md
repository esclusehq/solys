# Solys - Escluse Agent

Lightweight Rust agent for Escluse game server management platform.

## Features

- WebSocket connection to Escluse backend
- Docker-based server management
- Real-time monitoring and heartbeats
- Automatic reconnection
- Container orchestration

## Prerequisites

- Rust 1.70+
- Docker
- PostgreSQL (optional, for local development)

## Installation

```bash
# Clone the repository
git clone https://github.com/esclusehq/solys.git
cd solys

# Copy environment configuration
cp .env.example .env

# Edit .env with your configuration
nano .env

# Build
cargo build --release

# Run
cargo run --release
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AGENT_BACKEND_URL` | WebSocket URL to Escluse backend | `wss://app.esluce.com/api/ws/node` |
| `AGENT_API_KEY` | API key from Escluse dashboard | Required |
| `AGENT_NAME` | Unique name for this agent | `my-local-agent` |
| `AGENT_RUNTIME` | Runtime preference (`docker`, `podman`, `auto`) | `auto` |
| `AGENT_HEARTBEAT_INTERVAL` | Heartbeat interval in seconds | `30` |
| `AGENT_RECONNECT_INITIAL` | Initial reconnect delay in seconds | `2` |
| `AGENT_RECONNECT_MAX` | Maximum reconnect delay in seconds | `120` |
| `AGENT_MAX_CONCURRENT` | Maximum concurrent tasks | `10` |
| `AGENT_TASK_TIMEOUT` | Task timeout in seconds | `300` |
| `AGENT_METRICS_INTERVAL` | Metrics reporting interval in seconds | `60` |
| `LOG_LEVEL` | Log level (`trace`, `debug`, `info`, `warn`, `error`) | `info` |
| `LOG_FORMAT` | Log format (`text`, `json`) | `text` |

## Docker

```bash
# Build Docker image
docker build -t escluse/solys:latest .

# Run container
docker run -d \
  --name solys-agent \
  --env-file .env \
  -v /var/run/docker.sock:/var/run/docker.sock \
  escluse/solys:latest
```

Or use Docker Compose:

```bash
docker compose up -d
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   Escluse API                   │
│            (app.esluce.com)                     │
└─────────────────────┬───────────────────────────┘
                      │ WebSocket
                      ▼
┌─────────────────────────────────────────────────┐
│                    Solys Agent                  │
│              (Rust + Tokio)                    │
└─────────────────────┬───────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│              Docker / Podman                    │
│           (Container Runtime)                  │
└─────────────────────────────────────────────────┘
```

## License

MIT