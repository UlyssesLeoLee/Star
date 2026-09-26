# canvas-game Dockerfile (per ULYS-160 §4 阶段 1 骨架)

FROM rust:slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
RUN cargo build --release -p canvas-game

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/canvas-game /usr/local/bin/canvas-game
ENV PORT=8084
EXPOSE 8084
ENTRYPOINT ["/usr/local/bin/canvas-game"]