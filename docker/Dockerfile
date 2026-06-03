# Build stage
FROM rust:latest AS builder
WORKDIR /build

# Copy workspace files - relative to project root
COPY agent-core/ ./agent-core/
COPY web-agent/ ./web-agent/

# Cache and build dependencies first
RUN cd web-agent && cargo fetch

# Build binary
RUN cd web-agent && cargo build --release

# Runtime stage - minimal image
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    docker.io \
    && rm -rf /var/lib/apt/lists/* \
    && ln -s /usr/bin/docker /usr/local/bin/docker

# Create dedicated user (not root)
# Note: For Docker socket access, container must run as root or be in docker group
# RUN useradd -r -s /bin/false -d /var/lib/escluse-agent escluse-agent && \
#     mkdir -p /var/lib/escluse-agent && \
#     chown escluse-agent:escluse-agent /var/lib/escluse-agent

# Copy binary from builder
COPY --from=builder /build/web-agent/target/release/web-agent /usr/local/bin/web-agent

# Use root user for docker socket access
# USER escluse-agent

# Volume for data persistence
VOLUME ["/var/lib/escluse-agent"]

ENTRYPOINT ["/usr/local/bin/web-agent"]
