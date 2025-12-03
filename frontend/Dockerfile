# Multi-stage build for React frontend
# Stage 1: Build
FROM node:20-slim as builder

WORKDIR /app

# Build arguments for proxy and API URL
ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG NO_PROXY
ARG VITE_API_URL=http://localhost:3000

# Set proxy environment variables for npm (if provided)
ENV HTTP_PROXY=${HTTP_PROXY}
ENV HTTPS_PROXY=${HTTPS_PROXY}
ENV NO_PROXY=${NO_PROXY}
ENV http_proxy=${HTTP_PROXY}
ENV https_proxy=${HTTPS_PROXY}
ENV no_proxy=${NO_PROXY}

# Configure npm to use proxy if provided
RUN if [ -n "$HTTP_PROXY" ]; then \
      npm config set proxy ${HTTP_PROXY} || true; \
      npm config set https-proxy ${HTTPS_PROXY:-${HTTP_PROXY}} || true; \
    fi

# Copy package files
COPY package*.json ./

# Install dependencies with retries
RUN for i in 1 2 3; do \
      npm ci && break || sleep 5; \
    done

# Copy source code
COPY . .

# Set build-time environment variable
ENV VITE_API_URL=${VITE_API_URL}

# Build the application
RUN npm run build

# Stage 2: Simple HTTP server
FROM node:20-slim

WORKDIR /app

# Install serve (simple static file server)
RUN npm install -g serve@14.2.1

# Copy built files from builder
COPY --from=builder /app/dist /app/dist

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001 || exit 1

# Start simple HTTP server
CMD ["serve", "-s", "/app/dist", "-l", "3001"]

