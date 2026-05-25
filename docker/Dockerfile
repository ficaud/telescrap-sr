# Multi-stage Dockerfile for both local development and CI/CD deployment
# Supports multiple targets:
# - local-runtime: Full runtime image with Debian (for local Docker Compose)
# - ci-runtime: Minimal image with just the binary (for GitHub Actions CI/CD)

# ============================================================================== 
# Build Stage - Common for all targets
# ==============================================================================
FROM rust:1.89-slim-bookworm AS builder

# Install build dependencies (needed for OpenSSL and other native dependencies)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    curl \
    perl \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build the binary (release mode for smaller, optimized binary)
RUN cargo build --release --bin telescrap-sr

# ============================================================================== 
# Target 1: local-runtime - Full runtime image for local Docker Compose
# ==============================================================================
FROM debian:bookworm-slim AS local-runtime

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -m -u 1000 telescrap

# Create and set permissions for /app directory
RUN mkdir -p /app /app/data && chown -R telescrap:telescrap /app && chmod -R 755 /app

WORKDIR /app

# Copy the built binary from builder
COPY --from=builder /app/target/release/telescrap-sr /usr/local/bin/telescrap-sr

# Copy admin-panel static assets (favicon, images, etc.) at the path expected by ServeDir
COPY --from=builder /app/crates/admin-panel/src/interface/static /app/crates/admin-panel/src/interface/static

# Copy entrypoint script
COPY --chown=telescrap:telescrap docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

USER telescrap

# Default environment variables (can be overridden)
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# ============================================================================== 
# Target 2: ci-runtime - Minimal image with just the binary (for CI/CD)
# ==============================================================================
FROM scratch AS ci-runtime

# Copy SSL certificates for HTTPS connections
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the binary
COPY --from=builder /app/target/release/telescrap-sr /telescrap-sr

# For compatibility with GitHub Actions artifact extraction
ENTRYPOINT ["/telescrap-sr"]
