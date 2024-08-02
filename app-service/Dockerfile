# Start with image that has the Rust toolchain installed
FROM rust:1.77-alpine AS chef
USER root
# Add cargo-chef and necessary build tools
RUN apk add --no-cache musl-dev gcc libc-dev make autoconf automake libtool g++
# Install protoc from the community repository
RUN apk add --no-cache --repository http://dl-cdn.alpinelinux.org/alpine/edge/community protoc
# Install cargo-chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Capture info needed to build dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY ./Cargo.toml ./Cargo.toml
COPY ./proto ./proto
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Copy all other files
COPY . .
# Build application
RUN cargo build --release --bin app-service

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/app-service /usr/local/bin
COPY --from=builder /app/assets /app/assets
ENV AUTH_SERVICE_HOST_NAME=auth-service
ENTRYPOINT ["/usr/local/bin/app-service"]
