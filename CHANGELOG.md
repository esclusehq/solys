# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-12

### Added
- WebSocket connection to Escluse backend
- Docker-based server management
- Real-time monitoring and heartbeats
- Automatic reconnection with exponential backoff
- Container orchestration for game servers
- Environment-based configuration
- Structured logging (text and JSON formats)
- Health reporting and system metrics
- Task execution and management

### Features
- WebSocket client with auto-reconnect
- Docker container lifecycle management
- Heartbeat system for agent status
- Configurable reconnect delays
- Concurrent task management
- Metrics collection and reporting
- Support for multiple runtime backends (Docker/Podman)
- SFTP file transfer capabilities
- RCON protocol support for game server console access
- Automated backup system

### Configuration
- `AGENT_BACKEND_URL` - WebSocket URL to Escluse backend
- `AGENT_API_KEY` - API key from Escluse dashboard
- `AGENT_NAME` - Unique name for this agent
- `AGENT_RUNTIME` - Runtime preference (docker, podman, auto)
- `AGENT_HEARTBEAT_INTERVAL` - Heartbeat interval in seconds
- `AGENT_RECONNECT_INITIAL` - Initial reconnect delay in seconds
- `AGENT_RECONNECT_MAX` - Maximum reconnect delay in seconds
- `AGENT_MAX_CONCURRENT` - Maximum concurrent tasks
- `AGENT_TASK_TIMEOUT` - Task timeout in seconds
- `AGENT_METRICS_INTERVAL` - Metrics reporting interval in seconds
- `LOG_LEVEL` - Log level (trace, debug, info, warn, error)
- `LOG_FORMAT` - Log format (text, json)

### Dependencies
- Rust 1.70+
- Tokio async runtime
- Serde for serialization
- Docker API via bollard