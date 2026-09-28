# Multi-stage Dockerfile for Prism Network Node
# Optimized for minimal container size and high security

# --- Stage 1: Build Environment ---
FROM rust:1.80-slim-bullseye AS builder

WORKDIR /usr/src/prism

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates
COPY apps ./apps

# Build release binary for prism-node
RUN cargo build --release -p prism-node

# --- Stage 2: Minimal Runtime Environment ---
FROM debian:bullseye-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder stage
COPY --from=builder /usr/src/prism/target/release/prism-node /usr/local/bin/prism-node
COPY apps/dashboard/index.html /app/apps/dashboard/index.html

# Expose HTTP JSON-RPC/Dashboard port and P2P Wire port
EXPOSE 8545 9000

ENV PRISM_PORT=8545
ENV PRISM_P2P_PORT=9000

# Run node binary
ENTRYPOINT ["/usr/local/bin/prism-node"]
