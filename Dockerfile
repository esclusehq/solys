FROM rust:1.79-slim AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY .cargo/ ./.cargo/
COPY src/ ./src/

RUN cargo fetch

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /build/target/release/escluse-agent /usr/local/bin/escluse-agent
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/escluse-agent"]
