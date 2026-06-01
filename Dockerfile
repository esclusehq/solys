FROM rust:1.79-slim AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY .cargo/ ./.cargo/
COPY src/ ./src/

RUN cargo fetch

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    docker.io \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/escluse-agent /usr/local/bin/escluse-agent

ENTRYPOINT ["/usr/local/bin/escluse-agent"]
