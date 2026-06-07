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
