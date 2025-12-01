# Multi-stage build for analytics server
# Stage 1: Build
FROM rust:1.83-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src directory to build dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/analytics-server.rs && \
    cargo build --release --bin analytics-server && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release --bin analytics-server

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies (including wget for healthcheck)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/analytics-server /usr/local/bin/analytics-server

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/analytics-server

# Create data directory for database
RUN mkdir -p /app/data && chown -R appuser:appuser /app

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Switch to non-root user
USER appuser

# Set entrypoint
ENTRYPOINT ["docker-entrypoint.sh"]

# Expose server port
EXPOSE 3000

# Set default environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_PATH=/app/data/analytics.db

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run the server
CMD ["analytics-server"]

