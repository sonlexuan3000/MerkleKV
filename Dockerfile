# Production-optimized multi-stage Dockerfile for MerkleKV
# Build stage with optimized Rust compilation
FROM rust:1.80-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Install musl target
RUN rustup target add x86_64-unknown-linux-musl

# Set working directory
WORKDIR /app

# Copy dependency files for layer caching
COPY Cargo.toml ./

# Create dummy source to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src/ ./src/

# Build with static linking for better container compatibility
RUN RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --bin merkle_kv --target x86_64-unknown-linux-musl

# Runtime stage - minimal debian image
FROM debian:bookworm-slim

# Install runtime dependencies and debugging tools
RUN apt-get update && apt-get install -y \
    ca-certificates \
    netcat-openbsd \
    procps \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r merklekv && useradd -r -g merklekv merklekv

# Create necessary directories with proper permissions
RUN mkdir -p /app/data /app/config && \
    chown -R merklekv:merklekv /app

# Switch to non-root user
USER merklekv

# Set working directory
WORKDIR /app

# Copy binary from builder stage (using musl target)
COPY --from=builder --chown=merklekv:merklekv /app/target/x86_64-unknown-linux-musl/release/merkle_kv /app/merkle_kv

# Copy configuration
COPY --chown=merklekv:merklekv config.toml /app/config/

# Make binary executable
RUN chmod +x /app/merkle_kv

# Expose port
EXPOSE 7379

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1


# Entry point
ENTRYPOINT ["/app/merkle_kv"]

# Default command
CMD ["--config", "/app/config/config.toml"] 