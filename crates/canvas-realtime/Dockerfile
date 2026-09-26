# canvas-realtime Dockerfile (per ULYS-160 §3 阶段 1 骨架)
# HTTP :8082 + WSS :8083 双端口

FROM rust:slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
RUN cargo build --release -p canvas-realtime

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/canvas-realtime /usr/local/bin/canvas-realtime
ENV HTTP_PORT=8082
ENV WS_PORT=8083
EXPOSE 8082 8083
ENTRYPOINT ["/usr/local/bin/canvas-realtime"]