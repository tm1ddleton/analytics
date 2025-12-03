# Multi-stage build for analytics server
# Stage 1: Build
FROM rust:1.83-slim as builder

# Build arguments for proxy configuration
ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG NO_PROXY

# Set proxy environment variables for apt-get and cargo
ENV HTTP_PROXY=${HTTP_PROXY}
ENV HTTPS_PROXY=${HTTPS_PROXY}
ENV NO_PROXY=${NO_PROXY}
ENV http_proxy=${HTTP_PROXY}
ENV https_proxy=${HTTPS_PROXY}
ENV no_proxy=${NO_PROXY}
ENV CARGO_HTTP_PROXY=${HTTP_PROXY}
ENV CARGO_HTTPS_PROXY=${HTTPS_PROXY}

# Configure apt-get to use proxy if provided
RUN if [ -n "$HTTP_PROXY" ]; then \
      echo "Acquire::http::Proxy \"${HTTP_PROXY}\";" > /etc/apt/apt.conf.d/proxy.conf && \
      echo "Acquire::https::Proxy \"${HTTPS_PROXY:-${HTTP_PROXY}}\";" >> /etc/apt/apt.conf.d/proxy.conf; \
    fi

# Install build dependencies with retries
RUN for i in 1 2 3; do \
      apt-get update && \
      apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
        && rm -rf /var/lib/apt/lists/* && break || sleep 10; \
    done

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Configure cargo to use proxy if provided (via config file)
RUN if [ -n "$HTTP_PROXY" ]; then \
      mkdir -p /root/.cargo && \
      echo "[http]" > /root/.cargo/config.toml && \
      echo "proxy = \"${HTTP_PROXY}\"" >> /root/.cargo/config.toml && \
      echo "[https]" >> /root/.cargo/config.toml && \
      echo "proxy = \"${HTTPS_PROXY:-${HTTP_PROXY}}\"" >> /root/.cargo/config.toml; \
    fi

# Create a dummy src directory to build dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/analytics-server.rs && \
    cargo build --release --bin analytics-server && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the actual application with retries
RUN for i in 1 2 3; do \
      cargo build --release --bin analytics-server && break || sleep 10; \
    done

# Stage 2: Runtime
FROM debian:bookworm-slim

# Build arguments for proxy configuration (needed for apt-get in runtime stage)
ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG NO_PROXY

# Set proxy environment variables for apt-get
ENV HTTP_PROXY=${HTTP_PROXY}
ENV HTTPS_PROXY=${HTTPS_PROXY}
ENV NO_PROXY=${NO_PROXY}
ENV http_proxy=${HTTP_PROXY}
ENV https_proxy=${HTTPS_PROXY}
ENV no_proxy=${NO_PROXY}

# Configure apt-get to use proxy if provided
RUN if [ -n "$HTTP_PROXY" ]; then \
      echo "Acquire::http::Proxy \"${HTTP_PROXY}\";" > /etc/apt/apt.conf.d/proxy.conf && \
      echo "Acquire::https::Proxy \"${HTTPS_PROXY:-${HTTP_PROXY}}\";" >> /etc/apt/apt.conf.d/proxy.conf; \
    fi

# Install runtime dependencies with retries
RUN for i in 1 2 3; do \
      apt-get update && \
      apt-get install -y --no-install-recommends \
        ca-certificates \
        wget \
        && rm -rf /var/lib/apt/lists/* && break || sleep 10; \
    done

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

