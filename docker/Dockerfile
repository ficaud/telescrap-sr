# Multi-stage Dockerfile for local development and CI/CD deployment
# Single target:
# - local-runtime: Full runtime image with Debian (for local Docker Compose)

# ============================================================================== 
# Build Stage - Common for all targets
# ==============================================================================
FROM rust:1.89-slim-bookworm AS builder

# Install build dependencies (system libcurl/OpenSSL, so no from-source C build)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    libcurl4-openssl-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build the binary (release mode for smaller, optimized binary)
# BuildKit cache mounts persist compiled dependencies across builds. The binary
# is copied out of the target cache mount into a real layer so later stages can
# reference it.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin telescrap-sr \
    && cp /app/target/release/telescrap-sr /app/telescrap-sr

# ============================================================================== 
# Target 1: local-runtime - Full runtime image for local Docker Compose
# ==============================================================================
FROM debian:bookworm-slim AS local-runtime

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libcurl4 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -m -u 1000 telescrap

# Create and set permissions for /app directory
RUN mkdir -p /app /app/data && chown -R telescrap:telescrap /app && chmod -R 755 /app

WORKDIR /app

# Copy the built binary from builder
COPY --from=builder /app/telescrap-sr /usr/local/bin/telescrap-sr

# Copy admin-panel static assets (favicon, images, etc.) at the path expected by ServeDir
COPY --from=builder /app/crates/admin-panel/src/interface/static /app/crates/admin-panel/src/interface/static

# Copy entrypoint script
COPY --chown=telescrap:telescrap docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

USER telescrap

# Default environment variables (can be overridden)
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

