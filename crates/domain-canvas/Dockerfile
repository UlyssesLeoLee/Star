# domain-canvas Dockerfile (per ULYS-160 §2 阶段 1 骨架)

FROM rust:slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
RUN cargo build --release -p domain-canvas

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/domain-canvas /usr/local/bin/domain-canvas
ENV PORT=8081
EXPOSE 8081
ENTRYPOINT ["/usr/local/bin/domain-canvas"]