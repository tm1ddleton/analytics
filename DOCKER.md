# Docker Deployment Guide

This guide explains how to build and run the Analytics Platform using Docker.

## ⚠️ Important: Use Docker Compose V2

**Use `docker compose` (space) not `docker-compose` (hyphen)**

Docker Compose V2 is integrated into Docker CLI. If you have the old Python-based `docker-compose` installed and it's broken, use the V2 command instead:

```bash
# ✅ Correct (V2)
docker compose up -d

# ❌ Old/Deprecated (V1)
docker-compose up -d
```

If you see `ModuleNotFoundError: No module named 'distutils'`, you're using the old version. Switch to `docker compose` (space).

## Architecture Decision: Separate Containers

The platform uses **separate containers** for backend and frontend:

### ✅ Why Separate Containers?

1. **Different Update Cycles**: Frontend changes more frequently than backend
2. **Independent Scaling**: Scale frontend and backend independently
3. **Better Separation**: Clear boundaries between services
4. **Technology Mismatch**: Rust backend vs Node.js frontend build
5. **CDN Deployment**: Frontend can be deployed to CDN if needed
6. **Easier Debugging**: Isolate issues to specific services
7. **Resource Optimization**: Different resource requirements

### ❌ Single Container Alternative

A single container is possible (backend serves static files), but:
- Requires rebuilding entire container for frontend changes
- Mixes concerns (API server + static file serving)
- Less flexible for scaling
- Harder to deploy frontend to CDN

**Recommendation**: Use separate containers for production.

## Quick Start

### Automatic Database Loading

**If you have an existing `analytics.db` file in the project root:**
- The `docker-up.bat` (Windows) or `docker-up.sh` (Linux/macOS) scripts will automatically:
  - Detect the `analytics.db` file
  - Copy it to the `data/` directory
  - Use it in the Docker container
- This makes it easy to use your existing database on any machine

**If no `analytics.db` exists:**
- A new empty database will be created automatically
- You can load data using the replay feature or API

### Windows Users

**Using Batch Files (Easiest):**

The repository includes convenient batch files for Windows:

```batch
REM Start services
docker-up.bat

REM View logs
docker-logs.bat

REM View logs for specific service
docker-logs.bat backend

REM Check status
docker-status.bat
index-CbJmwziR.js:316 Failed to fetch assets: Ce {message: 'Network Error', name: 'AxiosError', code: 'ERR_NETWORK', config: {…}, request: XMLHttpRequest, …}
dke @ index-CbJmwziR.js:316
await in dke
ie @ index-CbJmwziR.js:316
(anonymous) @ index-CbJmwziR.js:316
wd @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
Z4 @ index-CbJmwziR.js:48
(anonymous) @ index-CbJmwziR.js:48
R @ index-CbJmwziR.js:25
index-CbJmwziR.js:313  GET http://localhost/assets/ net::ERR_CONNECTION_REFUSED
(anonymous) @ index-CbJmwziR.js:313
xhr @ index-CbJmwziR.js:313
v5 @ index-CbJmwziR.js:315
_request @ index-CbJmwziR.js:316
request @ index-CbJmwziR.js:315
Gs.<computed> @ index-CbJmwziR.js:316
(anonymous) @ index-CbJmwziR.js:311
dke @ index-CbJmwziR.js:316
ie @ index-CbJmwziR.js:316
(anonymous) @ index-CbJmwziR.js:316
wd @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
yi @ index-CbJmwziR.js:48
k4 @ index-CbJmwziR.js:48
Z4 @ index-CbJmwziR.js:48
(anonymous) @ index-CbJmwziR.js:48
R @ index-CbJmwziR.js:25

REM Restart services
docker-restart.bat

REM Stop services
docker-down.bat

REM Build images
docker-build.bat

REM Build without cache
docker-build.bat --no-cache
```

**Using Docker Compose Directly:**

```cmd
REM Build and start both services
docker compose up -d

REM View all logs
docker compose logs -f

REM View backend logs only
docker compose logs -f backend

REM View frontend logs only
docker compose logs -f frontend

REM Stop all services
docker compose down
```

**Access the application:**
- Frontend: `http://localhost:5173`
- Backend API: `http://localhost:3000`

The frontend automatically proxies API requests to the backend via nginx.

### Linux/macOS Users

**Using Docker Compose (Recommended):**

```bash
# Build and start both services
docker compose up -d

# View all logs
docker compose logs -f

# View backend logs only
docker compose logs -f backend

# View frontend logs only
docker compose logs -f frontend

# Stop all services
docker compose down
```

**Access the application:**
- Frontend: `http://localhost:5173`
- Backend API: `http://localhost:3000`

The frontend automatically proxies API requests to the backend via nginx.

## Windows Batch Files

For Windows users, the repository includes convenient batch files that wrap common Docker Compose commands:

### Available Batch Files

| File | Description |
|------|-------------|
| `docker-up.bat` | Start all services in detached mode |
| `docker-down.bat` | Stop all services |
| `docker-build.bat` | Build Docker images (supports `--no-cache` flag) |
| `docker-logs.bat` | View logs (optionally for specific service) |
| `docker-restart.bat` | Restart all services |
| `docker-status.bat` | Show container status |

### Usage Examples

**Start services:**
```batch
docker-up.bat
```

**View logs:**
```batch
REM All services
docker-logs.bat

REM Specific service
docker-logs.bat backend
docker-logs.bat frontend
```

**Build images:**
```batch
REM Standard build
docker-build.bat

REM Clean build (no cache)
docker-build.bat --no-cache
```

**Check status:**
```batch
docker-status.bat
```

**Restart services:**
```batch
docker-restart.bat
```

**Stop services:**
```batch
docker-down.bat
```

### Batch File Details

All batch files:
- Display clear success/error messages
- Exit with appropriate error codes on failure
- Provide helpful next-step instructions
- Work with Docker Compose V2 (`docker compose`)

**Note:** These batch files are convenience wrappers. You can always use `docker compose` commands directly if preferred.

### Using Docker Directly

**Backend:**
```bash
# Build the backend image
docker build -t analytics-backend .

# Run the backend container
docker run -d \
  --name analytics-backend \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -e RUST_LOG=info \
  analytics-backend
```

**Frontend:**
```bash
# Build the frontend image
cd frontend
docker build -t analytics-frontend .

# Run the frontend container (connect to backend via Docker network)
docker run -d \
  --name analytics-frontend \
  -p 5173:80 \
  --network host \
  analytics-frontend
```

## Configuration

### Backend Environment Variables

- `HOST` - Server host address (default: `0.0.0.0`)
- `PORT` - Server port (default: `3000`)
- `DATABASE_PATH` - Path to SQLite database (default: `/app/data/analytics.db`)
- `RUST_LOG` - Logging level (default: `info`)

### Frontend Environment Variables

- `VITE_API_URL` - Backend API URL (default: empty for relative URLs, uses nginx proxy)
- `FRONTEND_PORT` - Port mapping for frontend (default: `5173`)

### Docker Compose Environment Variables

Set these in `.env` file or export before running:

```bash
export BACKEND_PORT=3000
export FRONTEND_PORT=5173
export RUST_LOG=info
```

### Volume Mounts

**Backend:**
- Database directory: Bind mount `./data:/app/data` (default)
  - If `analytics.db` exists in project root, it's automatically copied to `data/` directory
  - The `data/` directory is created automatically with proper permissions
  - Database persists in `./data/analytics.db` on the host
- Fallback: Named volume `analytics-data:/app/data` (if you prefer Docker-managed storage)

**Frontend:**
- No volumes needed (static files baked into image)

**Note on Database Persistence:**
- The default configuration uses a named Docker volume (`analytics-data`) which avoids permission issues
- To use a bind mount instead, uncomment the bind mount line in `docker-compose.yml` and ensure `./data` directory exists and is writable
- To access data in a named volume: `docker volume inspect analytics_analytics-data`

## How It Works

### Frontend-Backend Communication

1. **Development**: Frontend uses `VITE_API_URL` or defaults to `http://localhost:3000`
2. **Docker**: Frontend uses relative URLs (empty `VITE_API_URL`)
3. **nginx Proxy**: Frontend nginx proxies API requests to backend service
4. **Docker Network**: Services communicate via Docker network (`analytics-network`)

### Request Flow in Docker

```
Browser → Frontend (nginx:80) → Proxy → Backend (Rust:3000)
```

nginx configuration:
- Proxies `/api/*` to `http://backend:3000/`
- Proxies direct API paths (`/assets`, `/analytics`, etc.) to backend
- Serves static React files for all other routes

## Building

### Build Both Services

```bash
# Using docker compose
docker compose build

# Or build individually
docker build -t analytics-backend .
cd frontend && docker build -t analytics-frontend .
```

### Build Options

**Windows:**
```batch
REM Standard build
docker-build.bat

REM Build without cache
docker-build.bat --no-cache
```

**Linux/macOS:**
```bash
# Standard build
docker compose build

# Build without cache
docker compose build --no-cache

# Build specific service
docker compose build backend
docker compose build frontend
```

## Running

### Using Docker Compose

**Windows:**
```batch
REM Start services
docker-up.bat

REM Restart services
docker-restart.bat

REM View logs
docker-logs.bat
```

**Linux/macOS:**
```bash
# Basic
docker compose up -d

# With custom ports
BACKEND_PORT=8080 FRONTEND_PORT=8081 docker compose up -d

# With debug logging
RUST_LOG=debug docker compose up -d

# Rebuild and restart
docker compose up -d --build
```

### Separate Containers

**Backend:**
```bash
docker run -d \
  --name analytics-backend \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -e RUST_LOG=info \
  analytics-backend
```

**Frontend (with backend connection):**
```bash
# Create network first
docker network create analytics-network

# Run backend on network
docker run -d \
  --name analytics-backend \
  --network analytics-network \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  analytics-backend

# Run frontend on same network
docker run -d \
  --name analytics-frontend \
  --network analytics-network \
  -p 5173:80 \
  analytics-frontend
```

## Health Checks

Both services include health checks:

**Backend:**
```bash
curl http://localhost:3000/health
```

**Frontend:**
```bash
curl http://localhost:5173/health
```

**Check container health:**
```bash
docker ps  # Shows health status
docker inspect analytics-backend | grep -A 10 Health
docker inspect analytics-frontend | grep -A 10 Health
```

## Logs

### View Logs

**Windows:**
```batch
REM All services
docker-logs.bat

REM Backend only
docker-logs.bat backend

REM Frontend only
docker-logs.bat frontend
```

**Linux/macOS:**
```bash
# All services
docker compose logs -f

# Backend only
docker compose logs -f backend

# Frontend only
docker compose logs -f frontend

# Last 100 lines
docker compose logs --tail=100
```

### Log Levels

Set `RUST_LOG` environment variable for backend:

- `RUST_LOG=error` - Only errors
- `RUST_LOG=warn` - Warnings and errors
- `RUST_LOG=info` - Info, warnings, and errors (default)
- `RUST_LOG=debug` - Debug and above
- `RUST_LOG=trace` - All logs (very verbose)
- `RUST_LOG=analytics::dag=debug` - Debug for specific module

## Development

### Development Workflow

For development, you may prefer running locally:

```bash
# Backend (local)
cargo run --bin analytics-server

# Frontend (local)
cd frontend && npm run dev
```

Docker is best for:
- Production deployment
- Consistent environments
- CI/CD pipelines
- Testing production-like setup

### Hot Reload

Docker containers don't support hot reload by default. For development:
- Use local development servers
- Or mount source code as volumes (not recommended for production)

## Production Considerations

1. **Database Persistence**: Always mount a volume for `/app/data` to persist the database
2. **Resource Limits**: Set appropriate CPU and memory limits
3. **Logging**: Configure log aggregation (e.g., Docker logging driver)
4. **Security**: 
   - Run as non-root user (already configured)
   - Use secrets management for sensitive data
   - Configure firewall rules
5. **Networking**: Use Docker networks for service isolation
6. **Health Checks**: Monitor container health status
7. **Scaling**: Can scale backend independently from frontend
8. **CDN**: Frontend can be deployed to CDN, backend stays in containers

### Example Production Setup

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  backend:
    build: .
    restart: always
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
      - DATABASE_PATH=/app/data/analytics.db
    volumes:
      - ./data:/app/data
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
    healthcheck:
      test: ["CMD-SHELL", "wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1"]
      interval: 30s
      timeout: 3s
      retries: 3

  frontend:
    build: ./frontend
    restart: always
    ports:
      - "80:80"
    depends_on:
      - backend
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M
```

## Troubleshooting

### Container Won't Start

**Windows:**
```batch
REM Check logs
docker-logs.bat backend
docker-logs.bat frontend

REM Check status
docker-status.bat

REM Check if ports are already in use
netstat -ano | findstr :3000
netstat -ano | findstr :5173
```

**Linux/macOS:**
```bash
# Check logs
docker compose logs backend
docker compose logs frontend

# Check if ports are already in use
lsof -i :3000
lsof -i :5173
```

### Frontend Can't Connect to Backend

```bash
# Verify both containers are running
docker compose ps

# Check network connectivity
docker compose exec frontend wget -O- http://backend:3000/health

# Verify nginx proxy configuration
docker compose exec frontend cat /etc/nginx/conf.d/default.conf
```

### Database Issues

**Permission Errors:**
If you see "unable to open database file" errors:

```bash
# Using named volume (default, recommended - no permission issues)
docker compose up -d

# Using bind mount - ensure directory is writable
mkdir -p data
chmod 755 data
# Note: If directory is owned by root, you may need sudo or to recreate it

# Check database file permissions
docker compose exec backend ls -la /app/data

# View database from container
docker compose exec backend ls -la /app/data

# Access named volume data
docker volume inspect analytics_analytics-data
```

### Build Failures

**Windows:**
```batch
REM Clean build (no cache)
docker-build.bat --no-cache

REM Check Rust version compatibility
docker run --rm rust:1.83-slim rustc --version

REM Check Node version compatibility
docker run --rm node:20-slim node --version
```

**Linux/macOS:**
```bash
# Clean build (no cache)
docker compose build --no-cache

# Check Rust version compatibility
docker run --rm rust:1.83-slim rustc --version

# Check Node version compatibility
docker run --rm node:20-slim node --version
```

### Docker Compose Command Issues

If you get `ModuleNotFoundError: No module named 'distutils'`:

```bash
# Use Docker Compose V2 (space, not hyphen)
docker compose up -d

# If that doesn't work, check Docker version
docker --version

# Docker Compose V2 should be included in Docker 20.10+
# If missing, update Docker or install compose plugin
```

## Multi-Architecture Builds

For ARM64 (Apple Silicon, Raspberry Pi):

```bash
# Backend
docker buildx build --platform linux/amd64,linux/arm64 -t analytics-backend .

# Frontend
cd frontend
docker buildx build --platform linux/amd64,linux/arm64 -t analytics-frontend .
```
