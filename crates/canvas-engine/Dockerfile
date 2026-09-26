# canvas-engine Dockerfile (per ULYS-160 §1 阶段 1 骨架)
# Multi-stage Rust build per deploy/k3s-local/Dockerfile 模板

FROM rust:slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
RUN cargo build --release -p canvas-engine

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/canvas-engine /usr/local/bin/canvas-engine
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/canvas-engine"]