# Build stage
FROM rust:1.77-slim-buster AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

ENV REDIS_HOST_NAME=redis

# Build the application
RUN cargo build --release
RUN cargo sqlx prepare

# Runtime stage
FROM debian:buster-slim AS runtime

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/auth-service /usr/local/bin
COPY --from=builder /app/assets /app/assets

CMD ["/usr/local/bin/auth-service"]
