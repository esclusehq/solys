---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 04b
type: execute
wave: 3
depends_on:
  - 68-01
  - 68-02
  - 68-03
  - 68-04a
files_modified:
  - opt/relay/Dockerfile
  - opt/relay/Caddy.Dockerfile
  - opt/relay/Caddyfile
  - opt/relay/docker-compose.yml
autonomous: true
requirements:
  - DEPLOY-01
  - DEPLOY-02
  - DEPLOY-05

must_haves:
  truths:
    - "Docker multi-stage build produces a non-root relay-gateway container exposing 8080 (WSS via Caddy), 25565 (raw TCP via NLB), and 9100 (Prometheus, NOT 9090)"
    - "Caddy Docker image is built with the caddy-dns/route53 plugin for Let's Encrypt DNS-01 wildcard cert provisioning"
    - "docker-compose orchestrates relay-gateway + caddy on a shared `relay-net` bridge network with healthcheck on :9100/metrics"
    - "Caddyfile enforces TLS 1.3 and routes `relay.esluce.net` / `*.play.esluce.net` to the gateway"
  artifacts:
    - path: "opt/relay/Dockerfile"
      provides: "Multi-stage Rust build for the gateway binary (matches opt/relay/Cargo.toml from 04a)"
      contains: "FROM rust:1.81-slim-bookworm"
    - path: "opt/relay/Caddy.Dockerfile"
      provides: "Caddy image with caddy-dns/route53 plugin"
      contains: "caddy-dns/route53"
    - path: "opt/relay/Caddyfile"
      provides: "TLS 1.3 termination + WS upgrade to the gateway on :8080"
      contains: "relay.esluce.net"
    - path: "opt/relay/docker-compose.yml"
      provides: "Compose file launching relay-gateway and Caddy with shared network; gateway exposes 25565:25565 (NLB-targeted raw TCP) and 9100:9100 (Prometheus, D-22, NOT 9090)"
      contains: "9100:9100"
  key_links:
    - from: "opt/relay/docker-compose.yml"
      to: "opt/relay/Caddyfile"
      via: "Caddy service mounts Caddyfile and depends on relay-gateway on the relay-net network; reverse_proxies relay.esluce.net to relay-gateway:8080"
      pattern: "reverse_proxy relay-gateway:8080"
    - from: "opt/relay/docker-compose.yml"
      to: "opt/relay/Dockerfile"
      via: "relay-gateway service builds from opt/relay/Dockerfile and exposes 9100:9100 for Prometheus scraping"
      pattern: "9100:9100"
---

<objective>
Add the containerization layer on top of the gateway crate built in Plan 04a. This is sub-plan 04b of a 3-part split of the original Plan 04. The other sub-plans are 04a (gateway crate + Handshake-parse routing) and 04c (DEPLOY.md runbook).

Output:
- `opt/relay/Dockerfile` (multi-stage Rust build)
- `opt/relay/Caddy.Dockerfile` (Caddy + caddy-dns/route53)
- `opt/relay/Caddyfile` (TLS 1.3 + WS upgrade)
- `opt/relay/docker-compose.yml` (gateway + caddy on a shared network; `9100:9100` Prometheus exposure per D-22)
</objective>

<execution_context>
@/home/rhnbztnl/.config/opencode/get-shit-done/workflows/execute-plan.md
@/home/rhnbztnl/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/ROADMAP.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-CONTEXT.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-RESEARCH.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04a-PLAN.md
</context>

<interfaces>
From Plan 04a (the gateway binary built by `cargo build -p relay-gateway --release`):
- Listens on `0.0.0.0:8080` (WSS tunnel endpoint)
- Listens on `0.0.0.0:25565` (raw TCP player forwarder)
- Listens on `0.0.0.0:9100` (Prometheus `/metrics`, per D-22)

From the root `Dockerfile` (if it exists): mirror the multi-stage build pattern.

From existing `docker-compose.yml` (root): the service naming and network conventions.
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Add Dockerfile, Caddy.Dockerfile, Caddyfile, docker-compose.yml</name>
  <files>opt/relay/Dockerfile, opt/relay/Caddy.Dockerfile, opt/relay/Caddyfile, opt/relay/docker-compose.yml
  <read_first>
    - cat Dockerfile (the existing root Dockerfile to mirror the multi-stage build pattern for the gateway binary)
    - cat docker-compose.yml (the existing compose file to see service naming and network conventions)
  </read_first>
  <action>
    1. Create `opt/relay/Dockerfile` (multi-stage Rust build for the gateway binary):
       ```dockerfile
       # ---- Build stage ----
       FROM rust:1.81-slim-bookworm AS builder
       WORKDIR /build
       RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
       # Cache deps
       COPY Cargo.toml Cargo.lock ./
       COPY opt/relay/Cargo.toml opt/relay/
       RUN mkdir -p opt/relay/src && touch opt/relay/src/main.rs
       RUN cargo build -p relay-gateway --release
       # Build real binary
       COPY opt/relay/src opt/relay/src
       RUN touch opt/relay/src/main.rs
       RUN cargo build -p relay-gateway --release
       # ---- Runtime stage ----
       FROM debian:bookworm-slim
       RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget && rm -rf /var/lib/apt/lists/*
       RUN useradd -m -u 1000 relay
       COPY --from=builder /build/target/release/relay-gateway /usr/local/bin/relay-gateway
       COPY opt/relay/relay-gateway.toml /etc/relay-gateway/relay-gateway.toml
       USER relay
       EXPOSE 8080 25565 9100
       HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
           CMD wget -q --spider http://localhost:9100/metrics || exit 1
       ENTRYPOINT ["/usr/local/bin/relay-gateway"]
       CMD ["--config", "/etc/relay-gateway/relay-gateway.toml"]
       ```

    2. Create `opt/relay/Caddy.Dockerfile`:
       ```dockerfile
       FROM caddy:2.8-builder AS builder
       RUN caddy build-modules --modules github.com/caddy-dns/route53

       FROM caddy:2.8
       COPY --from=builder /usr/bin/caddy /usr/bin/caddy
       COPY opt/relay/Caddyfile /etc/caddy/Caddyfile
       ```

    3. Create `opt/relay/Caddyfile` with TLS 1.3 termination + WS upgrade to the gateway:
       ```
       {
           servers {
               protocols h1 h2
           }
       }

       relay.esluce.net, *.play.esluce.net {
           encode zstd gzip
           tls {
               protocols tls1.3
           }
           @websocket {
               path /relay/tunnel
           }
           reverse_proxy @websocket relay-gateway:8080
       }
       ```

    4. Create `opt/relay/docker-compose.yml` (note `9100:9100` per D-22, NOT `9090:9090`):
       ```yaml
       version: "3.9"

       services:
         relay-gateway:
           build:
             context: ../..
             dockerfile: opt/relay/Dockerfile
           image: escluse/relay-gateway:latest
           container_name: relay-gateway
           restart: unless-stopped
           environment:
             - GATEWAY_HMAC_SECRET=${GATEWAY_HMAC_SECRET}
             - RUST_LOG=info
           volumes:
             - ./relay-gateway.toml:/etc/relay-gateway/relay-gateway.toml:ro
           ports:
             - "25565:25565"   # NLB-targeted raw TCP for MC Java
             - "9100:9100"     # Prometheus /metrics (D-22; NOT 9090)
           networks:
             - relay-net
           healthcheck:
             test: ["CMD", "wget", "-q", "--spider", "http://localhost:9100/metrics"]
             interval: 30s
             timeout: 5s
             retries: 3

         caddy:
           build:
             context: ../..
             dockerfile: opt/relay/Caddy.Dockerfile
           image: escluse/relay-caddy:latest
           container_name: relay-caddy
           restart: unless-stopped
           ports:
             - "443:443"
             - "80:80"
           volumes:
             - ./Caddyfile:/etc/caddy/Caddyfile:ro
             - caddy_data:/data
             - caddy_config:/config
           networks:
             - relay-net
           depends_on:
             relay-gateway:
               condition: service_healthy

       networks:
         relay-net:
           driver: bridge

       volumes:
         caddy_data:
         caddy_config:
       ```

    5. Verify all files exist and the compose file is valid:
       ```bash
       cd opt/relay && docker compose config --quiet
       ```
       (If docker is not installed locally, skip; the file is structurally validated.)
  </action>
  <verify>
    <automated>cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && ls opt/relay/Dockerfile opt/relay/Caddy.Dockerfile opt/relay/Caddyfile opt/relay/docker-compose.yml 2>&1 && echo "---" && wc -l opt/relay/Dockerfile opt/relay/Caddy.Dockerfile opt/relay/Caddyfile opt/relay/docker-compose.yml && echo "---" && grep -E "9100|9090" opt/relay/docker-compose.yml | head -5</automated>
  </verify>
  <acceptance_criteria>
    - `opt/relay/Dockerfile` exists with multi-stage build producing a non-root container
    - `opt/relay/Caddy.Dockerfile` exists with `caddy-dns/route53` plugin
    - `opt/relay/Caddyfile` exists with `relay.esluce.net` and TLS 1.3 enforced
    - `opt/relay/docker-compose.yml` exists with `relay-gateway` and `caddy` services on a shared network
    - The compose file's `ports:` block for `relay-gateway` includes `9100:9100` (NOT `9090:9090`) per D-22
    - The compose file's healthcheck pings `http://localhost:9100/metrics`
    - The Dockerfile's `EXPOSE` directive lists `9100` (not 9090)
  </acceptance_criteria>
  <done>All container artifacts in place; gateway container exposes 9100 (not 9090) for Prometheus; Caddy enforces TLS 1.3; ready for AWS deploy (04c)</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Docker image → host | Container runs as non-root user `relay` (UID 1000); only `/etc/relay-gateway/relay-gateway.toml` is bind-mounted (read-only) |
| Caddy → gateway | Caddy reverse-proxies WSS on port 443 to `relay-gateway:8080` on the internal `relay-net` bridge network — gateway's :8080 is NOT published to the host (Caddy is the only ingress) |
| NLB → host → gateway | NLB preserves the player's source IP and forwards raw TCP to host port 25565 → gateway's 25565 |
| Prometheus → host → gateway | Host port 9100 → gateway's 9100. **The :9100 port is intended to be scraped by the backend's `monitoring_service` (Plan 03 Task 3) running in a private network; production deployments should restrict ingress to that network via AWS security group.** |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-68-28 | Tampering | Relay config file | mitigate | `relay-gateway.toml` is bind-mounted read-only (`:ro`); runtime would need escalated container privileges to write. |
| T-68-29 | Elevation of Privilege | Container runtime | mitigate | Container runs as non-root user `relay` (UID 1000). Dockerfile has no `USER root` segments after the build stage. |
| T-68-30 | Spoofing | Exposed :9100 metrics port | mitigate | AWS security group MUST restrict ingress to the backend's monitoring service IP range. Documented in DEPLOY.md (04c). |
| T-68-31 | Tampering | Caddy image | mitigate | Uses official `caddy:2.8-builder` base image; only adds the `caddy-dns/route53` plugin which is the project standard (mirrors Phase 66). |
| T-68-32 | Information Disclosure | Player connection IP | accept | NLB passes the player's source IP to the gateway. The gateway uses it for the rate limiter only; it is never logged or returned. AWS CloudWatch can be configured to NOT log player traffic (only gateway-level events). |

## ASVS L1 Mappings (Phase 68 container tier only)

- **V2.1 Authentication:** The Caddy → gateway reverse proxy is on the internal `relay-net` bridge network; no external client can reach gateway's :8080 directly. TLS 1.3 at Caddy is the only authentication path.
- **V6.2 Cryptographic Practices:** TLS 1.3 enforced at Caddy level (`tls { protocols tls1.3 }`).
- **V14.1 Configuration:** All env-var-driven secrets (GATEWAY_HMAC_SECRET) are read from `.env` (which is gitignored) or AWS Secrets Manager user-data, never baked into the image.
</threat_model>

<verification>
After the task completes:

```bash
# 1. All container artifacts exist
ls /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Dockerfile \
    /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Caddy.Dockerfile \
    /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Caddyfile \
    /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/docker-compose.yml | wc -l
# Expected: 4

# 2. Prometheus port is 9100 in compose AND Dockerfile (NOT 9090)
grep -E "9100" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/docker-compose.yml | wc -l
# Expected: >= 2 (port mapping + healthcheck)
grep -E "EXPOSE.*9100" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Dockerfile
# Expected: 1 match (not "EXPOSE 9090")

# 3. No 9090 references in container files (D-22 / WARN 5 fix)
grep -c "9090" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Dockerfile \
                 /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/docker-compose.yml
# Expected: 0

# 4. Caddy enforces TLS 1.3
grep "tls1.3" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Caddyfile
# Expected: 1 match

# 5. Caddy-dns/route53 plugin in Caddy.Dockerfile
grep "caddy-dns/route53" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Caddy.Dockerfile
# Expected: 1 match

# 6. Container runs as non-root
grep "USER relay" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/Dockerfile
# Expected: 1 match
```

End-to-end container behavior requires `docker build` and `docker compose up` against the actual EC2 instance (Plan 04c). This plan only verifies the artifacts are correctly structured.
</verification>

<success_criteria>
- [ ] `opt/relay/Dockerfile` exists with multi-stage build producing a non-root container
- [ ] `opt/relay/Caddy.Dockerfile` exists
- [ ] `opt/relay/Caddyfile` exists with `relay.esluce.net` and TLS 1.3
- [ ] `opt/relay/docker-compose.yml` exists with `relay-gateway` and `caddy` services on a shared network
- [ ] Compose port mapping is `9100:9100` (NOT `9090:9090`)
- [ ] Dockerfile `EXPOSE` includes `9100` (NOT `9090`)
- [ ] Caddy-Dockerfile installs the `caddy-dns/route53` plugin
- [ ] Container user is `relay` (non-root)
</success_criteria>

<output>
After completion, create `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04b-SUMMARY.md`
</output>