# Technology Stack for Game Server Hosting Platforms

**Research Date:** 2026-04-09
**Domain:** Game Server Hosting Platform
**Confidence:** HIGH

## Executive Summary

The existing Esluce stack (Rust + React) is aligned with 2026 best practices for game server hosting platforms. The architecture leverages Rust for high-performance backend services and container orchestration, which is the optimal choice for managing resource-intensive game server workloads. Industry reference implementations like Pterodactyl use PHP/Go, but Rust provides superior performance characteristics for real-time server management.

## Research Findings

### Industry Reference: Pterodactyl

Pterodactyl is the dominant open-source game server panel (200+ game support), using:
- **Panel:** PHP + React
- **Daemon (Wings):** Go with Docker containerization
- **Database:** MariaDB + Redis
- **Architecture:** REST API + WebSocket for node communication

**Esluce Differentiation:** Escluse uses Rust for both API and node agent, providing:
- Lower memory footprint than Go
- Better async performance than PHP
- Stronger type safety than both

---

## Recommended Technology Stack

### Core Languages

| Technology | Current | Recommendation | Rationale |
|------------|---------|----------------|-----------|
| Rust | 1.70+ | **Keep at 1.80+** | Current minimum in 2026 should be 1.80 for better async performance. Rust provides memory safety without GC overhead critical for sustained game server connections. |
| TypeScript | Latest | **Keep current** | Frontend ecosystem requires up-to-date TypeScript for type safety with React 19. |
| Node.js | v20 | **Keep at v20 LTS** | v20 LTS remains supported through 2026. v22 available but v20 stable for production. |

**Confidence:** HIGH - Language choices are industry standard.

### Backend Framework (Rust)

| Technology | Current | Recommended | Rationale |
|------------|---------|-------------|-----------|
| Axum | v0.7.x | **Upgrade to v0.8.x** | v0.8.x is current (v0.8.9 as of March 2026). Best Rust web framework in 2026 due to Tower ecosystem integration and ergonomic extractor pattern. |
| Tower | v0.4 | **Keep at v0.4** | Stable middleware foundation. |
| Tower-HTTP | v0.5 | **Upgrade to v0.6** | Latest version includes improved tracing and middleware. |
| Hyper | v1 | **Keep current** | Standard HTTP/1.1 implementation, well-maintained. |

**Why Axum over alternatives (2026):**
- **vs Actix-web:** Actix has performance edge but steeper learning curve and slower maintenance. Axum is more ergonomic and better integrated with Tokio.
- **vs Rocket:** Rocket requires nightly Rust, slower release cycle. Axum is stable and more active.
- **vs Warp:** Warp's filter-based API is more complex. Axum's extractor pattern is more intuitive.

**Confidence:** HIGH - Axum is the dominant choice for Rust web services in 2026.

### Database & Caching

| Technology | Current | Recommended | Rationale |
|------------|---------|-------------|-----------|
| PostgreSQL | 16 | **Keep at 16.x** | Current stable version. Well-suited for game server metadata, user data, server configurations. |
| Redis | 7 | **Keep at 7.x** | Current stable. Essential for queue (background tasks) and caching (server status, session). |
| sqlx | v0.7 | **Keep at v0.7** | Current stable. Best async ORM for Rust with compile-time query validation. |

**Confidence:** HIGH - Standard persistence layer for this domain.

### Container Orchestration

| Technology | Current | Recommended | Rationale |
|------------|---------|-------------|-----------|
| bollard | v0.18 | **Upgrade to v0.20.x** | Latest is v0.20.2 (March 2026). Provides Docker daemon API for spawning game server containers. CRITICAL for game server isolation. |
| Docker | Latest | **Keep current** | Industry standard for containerization. |
| Podman | Available | **Support as alternative** | Podman is daemonless and more secure. Should support both Docker and Podman endpoints. |

**Why bollard:** Pure Rust Docker API. No Python/Go wrappers. Direct async control over container lifecycle - essential for game server start/stop/monitor operations.

**Confidence:** HIGH - Bollard is the standard Rust container library.

### Frontend

| Technology | Current | Recommended | Rationale |
|------------|---------|-------------|-----------|
| React | 19.2.4 | **Keep current** | v19 is current (released late 2024). Concurrent features beneficial for real-time dashboards. |
| Vite | 7.3.1 | **Keep at v6+** | v7.x available but v6.x is stable production. Fast dev server and build times. |
| React Router | 7.13.0 | **Keep current** | v7 is current. Data router features useful for server management flows. |
| Tailwind CSS | 4.2.0 | **Keep at v4.x** | v4 is current with improved performance. |
| Zustand | 5.0.12 | **Keep current** | Lightweight state management - appropriate for this scale. |
| Monaco Editor | 4.7.0 | **Keep current** | Essential for config file editing (server.properties, etc.). |

**Confidence:** HIGH - Frontend stack is modern and well-suited.

### Authentication & Security

| Technology | Current | Recommended | Rationale |
|------------|---------|-------------|-----------|
| jsonwebtoken | v9 | **Keep current** | JWT handling for API authentication. |
| bcrypt | v0.15 | **Migrate to argon2 v0.5** | Argon2 is the winner of Password Hashing Competition. Better GPU resistance than bcrypt. Should use Argon2 for new passwords. |
| argon2 | v0.5 | **Keep current** | Modern password hashing standard. |

**Why migrate to Argon2:** bcrypt is legacy (2000s). Argon2 (2015) is the modern standard with better resistance to GPU/ASIC cracking. Should implement Argon2 for new users while maintaining bcrypt for legacy.

**Confidence:** MEDIUM - Migration is recommended but not critical.

### Monitoring & Observability

| Technology | Current | Recommended | Rationale |
|------------|---------|-------------|-----------|
| tracing | v0.1 | **Keep current** | Structured logging standard for Rust. |
| OpenTelemetry | - | **Add in production** | For distributed tracing across microservices. Critical for debugging game server issues in production. |
| Prometheus metrics | - | **Add in production** | Game servers need custom metrics (player count, CPU, memory per container). |

**Confidence:** MEDIUM - Not in current stack, recommend adding in production phase.

### Additional Libraries

| Library | Current | Recommended | Rationale |
|---------|---------|-------------|-----------|
| ssh2 | v0.9.5 | **Keep current** | SFTP for file management. |
| rcon | v0.6 | **Keep current** | RCON protocol for game server console commands. |
| reqwest | v0.12 | **Keep current** | HTTP client for external APIs. |
| tokio-stream | v0.1.18 | **Keep current** | Async stream utilities. |
| notify | v6 | **Keep current** | File system watching for config changes. |
| aws-sdk-s3 | v1.124.0 | **Migrate to aws-sdk-rust** | aws-sdk-rust is the new official Rust SDK (GA in 2025). Better async performance and type safety. |

**Why aws-sdk-rust:** The v1 SDK is in maintenance mode. aws-sdk-rust (rust-client) is the future direction with better Tokio integration.

**Confidence:** MEDIUM - SDK migration is recommended for long-term maintenance.

---

## Technology Alternatives

### What NOT to Use and Why

| Rejected | Reason |
|----------|--------|
| **Actix-web** | Steeper learning curve than Axum. Maintenance slower. Ecosystem smaller. |
| **Rocket** | Requires nightly Rust. Slower release cycle. Not compatible with stable async. |
| **PHP (Pterodactyl)** | Legacy language. Poor async support. Not suitable for real-time game server management. |
| **Go (Wings pattern)** | Rust provides better memory characteristics for sustained connections. |
| **MongoDB** | Unnecessary for this domain. PostgreSQL handles all relational game server data. |
| **gRPC** | Overhead for internal node communication. WebSocket is more appropriate for game server state. |
| **Kubernetes** | Too complex for initial release. Docker/Podman direct management is sufficient. |

---

## Upgrade Recommendations by Priority

### Phase 1: Quick Wins (Low Risk)

| Upgrade | Current → Target | Effort | Risk |
|---------|------------------|--------|------|
| Axum | v0.7.x → v0.8.x | Low | Low - minor version |
| bollard | v0.18 → v0.20.x | Low | Low - API compatible |
| Tower-HTTP | v0.5 → v0.6 | Low | Low |

### Phase 2: Security Improvements

| Upgrade | Current → Target | Effort | Risk |
|---------|------------------|--------|------|
| bcrypt → argon2 | v0.15 → v0.5 | Medium | Low - gradual migration |

### Phase 3: Long-term Maintenance

| Upgrade | Current → Target | Effort | Risk |
|---------|------------------|--------|------|
| aws-sdk-s3 → aws-sdk-rust | v1.x → rust-client | Medium | Medium - API changes |

---

## Version Verification

| Library | Verified Version | Source |
|---------|-----------------|--------|
| Axum | v0.8.9 (2026-03-24) | GitHub tokio-rs/axum releases |
| bollard | v0.20.2 (2026-03-15) | GitHub fussybeaver/bollard releases |
| React | v19.x | Official React blog |
| Tower-HTTP | v0.6.x | docs.rs |

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Backend Framework | HIGH | Axum is dominant Rust web framework |
| Database/Cache | HIGH | PostgreSQL + Redis is standard |
| Container Orchestration | HIGH | bollard is the Rust standard |
| Frontend | HIGH | React ecosystem is mature |
| Security | MEDIUM | Argon2 migration recommended |
| Observability | MEDIUM | Not in current stack |

---

## Sources

- [Pterodactyl Panel v1.12.0 Release](https://github.com/pterodactyl/panel/releases/tag/v1.12.0) - Industry reference
- [Axum v0.8.9 Release](https://github.com/tokio-rs/axum/pull/3699) - Version verification
- [Bollard v0.20.2 Release](https://github.com/fussybeaver/bollard/releases/tag/v0.20.2) - Container library
- [Rust Web Frameworks 2026](https://aarambhdevhub.medium.com/rust-web-frameworks-in-2026-axum-vs-actix-web-vs-rocket-vs-warp-vs-salvo-which-one-should-you-2db3792c79a2) - Framework comparison
- [Self-Hosted Game Server Panels 2026](https://selfhosting.sh/best/game-servers/) - Industry landscape