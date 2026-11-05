# Star Canvas realtime service. The workflow supplies a minimal, filtered
# Cargo workspace as this Dockerfile's context; it must never receive the monorepo.

FROM rust:1.99.1-bookworm@sha256:cfcbed05398d0f6f81b7c3a24579475c567f97007ae7837ab421f9ca3e66333d AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/canvas-realtime ./crates/canvas-realtime
RUN cargo build --locked --release --package canvas-realtime

FROM debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3
COPY --from=builder --chown=10001:10001 /build/target/release/canvas-realtime /usr/local/bin/canvas-realtime
COPY LICENSE /LICENSE
USER 10001:10001
LABEL org.opencontainers.image.licenses="MIT"
ENV HTTP_PORT=8082
ENV WS_PORT=8083
EXPOSE 8082 8083
ENTRYPOINT ["/usr/local/bin/canvas-realtime"]
