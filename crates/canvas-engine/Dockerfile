# Star Canvas platform service. The workflow supplies a minimal, filtered Cargo
# workspace as this Dockerfile's context; it must never receive the monorepo.

FROM rust:1.99.0-bookworm@sha256:114c7a4425406451c2866b6aafe69fe29b1b298832db1277d411ac73c82d04d6 AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/canvas-engine ./crates/canvas-engine
RUN cargo build --locked --release --package canvas-engine

FROM debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3
COPY --from=builder --chown=10001:10001 /build/target/release/canvas-engine /usr/local/bin/canvas-engine
COPY LICENSE /LICENSE
USER 10001:10001
LABEL org.opencontainers.image.licenses="MIT"
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/canvas-engine"]
