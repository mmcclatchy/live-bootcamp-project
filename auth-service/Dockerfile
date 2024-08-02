# Build stage
FROM rust:1.77-slim-buster AS builder

# Install necessary build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:buster-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/auth-service /usr/local/bin
COPY --from=builder /app/assets /app/assets

# Set environment variables
ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug

# Set the default command to run the application
CMD ["/usr/local/bin/auth-service"]
