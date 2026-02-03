# ==========================================
# MEMORY_P v2.0 - Multi-Stage Dockerfile
# ==========================================

# Stage 1: Builder
FROM rust:1.77-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY .cargo ./.cargo

# Build release binary
RUN cargo build --release --locked

# Stage 2: Runtime with Julia support (optional)
FROM ubuntu:22.04 as runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Julia (optional - can be disabled via env var)
ARG JULIA_VERSION=1.10.0
RUN if [ -n "$JULIA_VERSION" ]; then \
    wget https://julialang-s3.julialang.org/bin/linux/x64/1.10/julia-${JULIA_VERSION}-linux-x86_64.tar.gz && \
    tar xzf julia-${JULIA_VERSION}-linux-x86_64.tar.gz && \
    mv julia-${JULIA_VERSION} /opt/julia && \
    ln -s /opt/julia/bin/julia /usr/local/bin/julia && \
    rm julia-${JULIA_VERSION}-linux-x86_64.tar.gz; \
    fi

# Copy binary from builder
COPY --from=builder /build/target/release/memory_p /app/memory_p

# Copy configuration templates
COPY config /app/config

# Create necessary directories
RUN mkdir -p /app/indices/tantivy \
    /app/PAYLOAD_BANK \
    /app/logs

# Expose MCP server port
EXPOSE 4040

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:4040/health || exit 1

# Run as non-root user
RUN useradd -m -u 1000 memory_p && \
    chown -R memory_p:memory_p /app
USER memory_p

# Set environment variables
ENV RUST_LOG=info
ENV MEMORY_P_CONFIG=/app/config/docker.toml
ENV JULIA_ENABLED=false

# Start the server
CMD ["/app/memory_p"]
