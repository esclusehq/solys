FROM rust:1.88-alpine3.20 AS builder

WORKDIR /app

RUN apk add --no-cache openssl-dev musl-dev pkgconfig

ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include

COPY Cargo.toml ./
COPY src/ ./src/

RUN cargo build --release --bin worker

FROM alpine:3.19

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

COPY --from=builder /app/target/release/worker /app/worker

ENV RUST_LOG=info

CMD ["/app/worker"]