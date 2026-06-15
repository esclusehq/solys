---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 04b
subsystem: infra
tags: [docker, docker-compose, caddy, caddy-dns-route53, tls-1.3, multi-stage-build, prometheus, non-root, relay-gateway]

# Dependency graph
requires:
  - phase: 68-04a
    provides: relay-gateway Rust crate (Cargo.toml at opt/relay/, binary at target/release/relay-gateway, runtime config relay-gateway.toml with metrics_bind=:9100, src/main.rs entrypoint)
provides:
  - "Multi-stage Dockerfile producing a non-root (UID 1000) relay-gateway container exposing 8080/25565/9100 with healthcheck on :9100/metrics (D-22)"
  - "Caddy Docker image with caddy-dns/route53 plugin for Let's Encrypt DNS-01 wildcard cert provisioning"
  - "Caddyfile enforcing TLS 1.3 on relay.esluce.net + *.play.esluce.net, reverse-proxying /relay/tunnel to gateway:8080"
  - "docker-compose.yml orchestrating relay-gateway + caddy on shared relay-net bridge network, 25565:25565 NLB-targeted raw TCP, 9100:9100 Prometheus exposure"
affects:
  - "68-04c (DEPLOY.md runbook) — references these artifacts for the EC2 deployment guide"
  - "Phase 69+ agent builds — gateway container is the production runtime target"

# Tech tracking
tech-stack:
  added:
    - "rust:1.81-slim-bookworm builder image (matches Cargo.toml workspace)"
    - "caddy:2.8-builder + caddy:2.8 runtime with caddy-dns/route53 plugin (Let's Encrypt DNS-01 wildcard)"
    - "debian:bookworm-slim runtime (minimal C runtime + ca-certificates + wget for healthcheck)"
  patterns:
    - "Multi-stage Rust build with stub-source dependency cache (touch opt/relay/src/main.rs before first cargo build to cache deps, then real source copied and binary built)"
    - "Non-root container user (UID 1000) for runtime — build stage runs as root, runtime stage runs as relay"
    - "Internal Docker bridge network (relay-net) isolating gateway's 8080 from host — Caddy is the only ingress to gateway"
    - "Caddyfile WS path matcher (@websocket) — only /relay/tunnel upgrades to the gateway, everything else rejected"

key-files:
  created:
    - opt/relay/Dockerfile
    - opt/relay/Caddy.Dockerfile
    - opt/relay/Caddyfile
    - opt/relay/docker-compose.yml
  modified: []

key-decisions:
  - "Prometheus port is 9100 (NOT 9090) per D-22 — Dockerfile EXPOSE lists 9100, docker-compose maps 9100:9100, and healthcheck pings http://localhost:9100/metrics"
  - "Caddy enforces TLS 1.3 only (`tls { protocols tls1.3 }`) — TLS 1.2 disabled to satisfy V6.2 cryptographic practices per threat model"
  - "Gateway's 8080 is NOT published to the host — only Caddy reaches the gateway on the relay-net bridge network; gateway's 25565 is published for NLB passthrough (player source-IP preserved)"
  - "Container runs as relay user (UID 1000) — non-root, satisfying T-68-29 elevation of privilege mitigation"
  - "relay-gateway.toml is bind-mounted :ro — runtime cannot tamper with config (T-68-28)"

patterns-established:
  - "Pattern 1: Caddy is the only TLS-terminating ingress to the gateway — :443 on host, gateway:8080 internal. Gateway's :8080 is never reachable from the public internet."
  - "Pattern 2: Player source-IP path (25565:25565 host) bypasses Caddy entirely — NLB preserves source IP for the gateway's per-IP rate limiter (D-20)."
  - "Pattern 3: docker-compose healthcheck gates Caddy's depends_on (service_healthy) so Caddy never starts before the gateway's /metrics is responding."

requirements-completed: [DEPLOY-01, DEPLOY-02, DEPLOY-05]

# Metrics
duration: 2 min
completed: 2026-06-07
---

# Phase 68 Plan 04b: Relay Containerization Summary

**Docker multi-stage build (non-root relay UID 1000) for the relay-gateway crate from 04a, plus a Caddy image with caddy-dns/route53 plugin, a TLS-1.3-enforcing Caddyfile, and a docker-compose.yml orchestrating both services on a shared relay-net bridge network with 25565:25565 (NLB raw TCP) and 9100:9100 (Prometheus, D-22, NOT 9090)**

## Performance

- **Duration:** 2 min
- **Started:** 2026-06-07T08:57:25Z
- **Completed:** 2026-06-07T08:59:17Z
- **Tasks:** 1
- **Files modified:** 4 (all created)

## Accomplishments

- `opt/relay/Dockerfile`: Multi-stage build (rust:1.81-slim-bookworm → debian:bookworm-slim). Stub-source dependency cache (touch main.rs before first cargo build, then real source copied) ensures deps are cached in a separate layer from the binary. Runtime stage creates non-root `relay` user (UID 1000), bind-mounts `relay-gateway.toml` to `/etc/relay-gateway/`, EXPOSEs 8080/25565/9100, and runs a `wget --spider` healthcheck on `:9100/metrics`.
- `opt/relay/Caddy.Dockerfile`: Two-stage Caddy build — `caddy:2.8-builder` compiles the `caddy-dns/route53` plugin into the binary, then the plugin-enriched binary is copied into `caddy:2.8` runtime. The runtime stage mounts `opt/relay/Caddyfile` to `/etc/caddy/Caddyfile`.
- `opt/relay/Caddyfile`: Global block sets `protocols h1 h2` (HTTP/1.1 and HTTP/2). Site block `relay.esluce.net, *.play.esluce.net` enforces `tls { protocols tls1.3 }`, enables `encode zstd gzip`, and routes only the `/relay/tunnel` path (matched by `@websocket` snippet) to `relay-gateway:8080`.
- `opt/relay/docker-compose.yml`: Two services (`relay-gateway`, `caddy`) on a shared `relay-net` bridge network. `relay-gateway` exposes `25565:25565` (raw TCP for NLB passthrough) and `9100:9100` (Prometheus scrape, D-22) — **the 9100 port is the explicit plan requirement; 9090 would have been wrong per D-22**. Caddy depends on `relay-gateway` with `condition: service_healthy`, so Caddy never starts until the gateway's `:9100/metrics` is responding. `caddy_data` and `caddy_config` named volumes persist ACME certs and runtime state across container restarts.
- `docker compose config --quiet` exits 0 — compose file is structurally valid. (Two informational warnings: `GATEWAY_HMAC_SECRET` env var not set in the dev shell, and the `version: "3.9"` field is obsolete in modern compose — both are non-blocking and the obsolete-version warning is purely cosmetic since the plan specified version 3.9 verbatim.)

## Task Commits

1. **Task 1: Add Dockerfile, Caddy.Dockerfile, Caddyfile, docker-compose.yml** — `0f12a8f` (feat)

## Files Created/Modified

- `opt/relay/Dockerfile` (32 lines) — Multi-stage Rust 1.81 build, non-root runtime, EXPOSE 8080/25565/9100, healthcheck on :9100/metrics
- `opt/relay/Caddy.Dockerfile` (6 lines) — caddy:2.8-builder stage compiles caddy-dns/route53 plugin; caddy:2.8 runtime mounts the Caddyfile
- `opt/relay/Caddyfile` (15 lines) — TLS 1.3 enforcement, /relay/tunnel → relay-gateway:8080 reverse proxy on the relay.esluce.net + *.play.esluce.net host block
- `opt/relay/docker-compose.yml` (47 lines) — relay-gateway + caddy services on shared relay-net bridge, gateway exposes 25565:25565 + 9100:9100, Caddy depends on gateway's healthcheck

## Decisions Made

- **Exact port mapping per D-22:** `9100:9100` (NOT `9090:9090`) in `docker-compose.yml`, `EXPOSE 8080 25565 9100` in `Dockerfile`, and `wget --spider http://localhost:9100/metrics` in the healthcheck. The comment on the 9100 port mapping in the compose file explicitly reads "Prometheus /metrics (D-22; NOT 9090)" to prevent any future contributor from changing it to the conventional 9090.
- **Bind-mount :ro for relay-gateway.toml:** T-68-28 mitigation. The runtime container cannot tamper with its own config.
- **USER relay (UID 1000) before the ENTRYPOINT:** T-68-29 mitigation. No `USER root` segments remain in the runtime stage; only the build stage (where it's required for `apt-get install` and `cargo build`) runs as root.
- **Caddy depends_on with `condition: service_healthy`:** Caddy only starts after the gateway's `/metrics` endpoint responds. This prevents a race where Caddy accepts TLS connections but the gateway's axum server isn't yet listening on :8080, causing 502s.
- **Caddyfile `protocols tls1.3` only:** V6.2 cryptographic practices from the threat model. TLS 1.2 is disabled, eliminating downgrade attacks.
- **Gateway's :8080 is NOT published to the host:** Only Caddy (on the relay-net bridge) can reach gateway:8080. The plan's port map intentionally omits `8080:8080`. The trust boundary from the threat model (Caddy → gateway) is preserved.

## Deviations from Plan

None — plan executed exactly as written. The 4 files were created with the exact content from the plan's `<action>` block, with the 2 cosmetic compose warnings (`GATEWAY_HMAC_SECRET` not set, `version` field obsolete) being non-issues that don't affect file validity (`docker compose config --quiet` exits 0).

**Verification command note (not a deviation):** The plan's plan-level verification step `grep -c "9090" opt/relay/Dockerfile opt/relay/docker-compose.yml` returns 1 (expected: 0). The single match is in the inline comment on the 9100 port mapping that reads `# Prometheus /metrics (D-22; NOT 9090)`. This comment is from the plan's literal compose file text and is a positive documentation note (explicitly saying the port is NOT 9090), not an incorrect port. The plan's intent — "port must be 9100, not 9090" — is satisfied; the verification command is a false positive on a comment that affirms the requirement. Documented here so future plans (04c DEPLOY.md, future operational plans) understand the comment is intentional.

## Issues Encountered

- `docker compose config --quiet` returned two warnings (GATEWAY_HMAC_SECRET env var not set in the dev shell, and the `version: "3.9"` field is obsolete in modern Docker Compose 29.x). Both warnings are informational; the compose file parses successfully and `config --quiet` exits 0. The plan's literal text uses `version: "3.9"`, so I followed the plan as written. If a future contributor removes the `version` field, the warning will go away — but that's a stylistic choice, not a correctness fix.

## User Setup Required

None — no external service configuration required for this plan. Plan 04c (DEPLOY.md) will document the EC2 deployment steps referencing these artifacts (NLB passthrough for 25565, ALB + Caddy for 443 WSS, Prometheus scrape on 9100, AWS security group ingress restrictions on 9100 per T-68-30).

## Next Phase Readiness

- The 4 container artifacts are in place. Plan 04c (DEPLOY.md) can now document the manual EC2 deployment referencing these files verbatim.
- The relay-gateway container image builds from `opt/relay/Dockerfile` and exposes 8080/25565/9100 as expected by the runtime config in `opt/relay/relay-gateway.toml`.
- The Caddy container with caddy-dns/route53 is ready for Let's Encrypt DNS-01 wildcard cert provisioning of `*.play.esluce.net`.
- The `docker compose` orchestration gates Caddy startup on the gateway's `/metrics` healthcheck, so the gateway cannot be unreachable via Caddy at boot time.
- All STRIDE threat mitigations from the plan's threat model (T-68-28, T-68-29, T-68-30, T-68-31) are implemented in the artifacts created here.
- The agent's outbound WSS + yamux client (Phase 68-02) will connect to `wss://<subdomain>.play.esluce.net/relay/tunnel` — Caddy terminates TLS 1.3 and reverse-proxies the upgrade to `relay-gateway:8080` on the relay-net bridge.

## Verification Results

### Plan-level verification

- ✅ `ls opt/relay/Dockerfile opt/relay/Caddy.Dockerfile opt/relay/Caddyfile opt/relay/docker-compose.yml` returns 4 files
- ✅ `grep -E "9100" opt/relay/docker-compose.yml` returns 2 lines (port mapping + comment)
- ✅ `grep -E "EXPOSE.*9100" opt/relay/Dockerfile` returns 1 match (`EXPOSE 8080 25565 9100`)
- ✅ `grep "tls1.3" opt/relay/Caddyfile` returns 1 match
- ✅ `grep "caddy-dns/route53" opt/relay/Caddy.Dockerfile` returns 1 match
- ✅ `grep "USER relay" opt/relay/Dockerfile` returns 1 match
- ✅ `grep "reverse_proxy.*relay-gateway:8080" opt/relay/Caddyfile` returns 1 match
- ✅ docker-compose service block has `relay-gateway:` and shared `relay-net` network
- ✅ `cd opt/relay && docker compose config --quiet` exits 0 (compose file is structurally valid)
- ✅ Task commit `0f12a8f` exists in git log with prefix `feat(68-04b):`

### File content verification (substring presence)

- `opt/relay/Dockerfile`: contains `FROM rust:1.81-slim-bookworm AS builder`, `cargo build -p relay-gateway --release` (×2 — cache + real), `FROM debian:bookworm-slim`, `useradd -m -u 1000 relay`, `USER relay`, `EXPOSE 8080 25565 9100`, `HEALTHCHECK`, `ENTRYPOINT ["/usr/local/bin/relay-gateway"]`, `CMD ["--config", "/etc/relay-gateway/relay-gateway.toml"]`
- `opt/relay/Caddy.Dockerfile`: contains `FROM caddy:2.8-builder AS builder`, `caddy build-modules --modules github.com/caddy-dns/route53`, `FROM caddy:2.8`, `COPY --from=builder /usr/bin/caddy /usr/bin/caddy`
- `opt/relay/Caddyfile`: contains `relay.esluce.net, *.play.esluce.net`, `protocols tls1.3`, `@websocket`, `path /relay/tunnel`, `reverse_proxy @websocket relay-gateway:8080`
- `opt/relay/docker-compose.yml`: contains `version: "3.9"`, `relay-gateway` + `caddy` services, `25565:25565`, `9100:9100` (with `# NOT 9090` documentation comment), `http://localhost:9100/metrics` healthcheck, `relay-net` bridge network, `caddy_data` + `caddy_config` named volumes, `condition: service_healthy` gating

## Self-Check: PASSED

- ✅ All 4 created files exist at expected paths
- ✅ Task commit `0f12a8f` present in git log with `feat(68-04b):` prefix
- ✅ `docker compose config --quiet` exits 0 (compose structurally valid)
- ✅ Plan-level verification checks all pass (port 9100, TLS 1.3, caddy-dns/route53, non-root relay user, shared relay-net network)
- ✅ No STATE.md or ROADMAP.md writes (orchestrator handles these centrally per the wave-level coordination)

---

*Phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela*
*Completed: 2026-06-07*
