#!/bin/bash

# Analytics POC Demo Launcher
# This script starts both the backend API server and the React frontend

set -e

echo "=========================================="
echo "  Analytics POC - Demo Launcher"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the project root directory"
    exit 1
fi

# Check if frontend directory exists
if [ ! -d "frontend" ]; then
    echo "Error: frontend directory not found"
    exit 1
fi

# Check Node.js version
NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "Error: Node.js 18+ required (you have $(node -v))"
    exit 1
fi

# Ensure frontend dependencies are installed
if [ ! -d "frontend/node_modules" ]; then
    echo -e "${YELLOW}Installing frontend dependencies...${NC}"
    cd frontend
    npm install
    cd ..
fi

# Function to cleanup on exit
cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down...${NC}"
    if [ ! -z "$BACKEND_PID" ]; then
        echo "Stopping backend server (PID: $BACKEND_PID)"
        kill $BACKEND_PID 2>/dev/null || true
    fi
    if [ ! -z "$FRONTEND_PID" ]; then
        echo "Stopping frontend server (PID: $FRONTEND_PID)"
        kill $FRONTEND_PID 2>/dev/null || true
    fi
    exit 0
}

trap cleanup EXIT INT TERM

# Check if database has data
echo -e "${BLUE}Step 1: Checking database...${NC}"
if [ ! -f "analytics.db" ] || [ $(stat -f%z "analytics.db" 2>/dev/null || stat -c%s "analytics.db" 2>/dev/null) -lt 50000 ]; then
    echo "Database is empty or missing. Creating demo data..."
    cargo run --example create_demo_data --quiet 2>&1 | grep -E "(Generating|✓|✨)" || true
    echo -e "${GREEN}✓ Demo data created${NC}"
else
    echo -e "${GREEN}✓ Database exists with data${NC}"
fi
echo ""

# Build backend if needed
echo -e "${BLUE}Step 2: Building backend...${NC}"
cargo build --bin analytics-server --release
echo -e "${GREEN}✓ Backend built${NC}"
echo ""

# Start backend server
echo -e "${BLUE}Step 3: Starting backend API server...${NC}"
cargo run --bin analytics-server --release > backend.log 2>&1 &
BACKEND_PID=$!
echo -e "${GREEN}✓ Backend started (PID: $BACKEND_PID)${NC}"
echo "  Server running on: http://localhost:3000"
echo "  Logs: backend.log"
echo ""

# Wait for backend to be ready
echo "Waiting for backend to start..."
for i in {1..30}; do
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Backend is ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Error: Backend failed to start. Check backend.log for details."
        exit 1
    fi
    sleep 1
    echo -n "."
done
echo ""

# Start frontend
echo -e "${BLUE}Step 4: Starting React frontend...${NC}"
cd frontend
npm run dev > ../frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..
echo -e "${GREEN}✓ Frontend started (PID: $FRONTEND_PID)${NC}"
echo "  Dashboard running on: http://localhost:5173"
echo "  Logs: frontend.log"
echo ""

# Wait for frontend to be ready
echo "Waiting for frontend to start..."
for i in {1..30}; do
    if curl -s http://localhost:5173 > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Frontend is ready${NC}"
        break
    fi
    sleep 1
    echo -n "."
done
echo ""

echo "=========================================="
echo -e "${GREEN}  ✓ Demo is ready!${NC}"
echo "=========================================="
echo ""
echo "🌐 Open your browser to:"
echo "   http://localhost:5173"
echo ""
echo "📊 Demo Features:"
echo "   • Select multiple assets (AAPL, MSFT, GOOG)"
echo "   • Choose analytics presets"
echo "   • View historical charts"
echo "   • Start replay for real-time updates"
echo "   • Copy API URLs for testing"
echo ""
echo "📝 Backend API:"
echo "   http://localhost:3000"
echo ""
echo "🛑 Press Ctrl+C to stop the demo"
echo ""

# Keep script running and tail logs
tail -f backend.log

