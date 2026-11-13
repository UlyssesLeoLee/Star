# Star Canvas domain service. The workflow supplies a minimal, filtered Cargo
# workspace as this Dockerfile's context; it must never receive the monorepo.

FROM rust:1.99.1-bookworm@sha256:cfcbed05398d0f6f81b7c3a24579475c567f97007ae7837ab421f9ca3e66333d AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/domain-canvas ./crates/domain-canvas
RUN cargo build --locked --release --package domain-canvas

FROM debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3
COPY --from=builder --chown=10001:10001 /build/target/release/domain-canvas /usr/local/bin/domain-canvas
COPY LICENSE /LICENSE
USER 10001:10001
LABEL org.opencontainers.image.licenses="MIT"
ENV PORT=8081
EXPOSE 8081
ENTRYPOINT ["/usr/local/bin/domain-canvas"]
