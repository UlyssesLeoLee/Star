FROM rust:1.99.1-slim-bookworm@sha256:7d53bccd41fde2f509a276b37a8ef541f8e75fc3e1f85b059230386f512ed6c1 AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN mkdir -p crates
# @star-local-crates-begin
# @star-local-crates-end
# The context generator replaces the marker pair with one COPY per resolved local crate.
# This source template is intentionally not a direct build context.
RUN cargo build --release --locked -p star-api-rest

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/star-api-rest /usr/local/bin/star-api-rest
ENV STAR_API_REST_BIND_ADDR=0.0.0.0:8081
EXPOSE 8081
ENTRYPOINT ["/usr/local/bin/star-api-rest"]
