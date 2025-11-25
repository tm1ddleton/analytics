#!/bin/bash

# Stop all demo processes

echo "Stopping Analytics POC Demo..."

# Kill backend server
if lsof -ti:3000 > /dev/null 2>&1; then
    echo "Stopping backend server on port 3000..."
    lsof -ti:3000 | xargs kill -9 2>/dev/null
    echo "✓ Backend stopped"
else
    echo "No backend server running on port 3000"
fi

# Kill frontend server
if lsof -ti:5173 > /dev/null 2>&1; then
    echo "Stopping frontend server on port 5173..."
    lsof -ti:5173 | xargs kill -9 2>/dev/null
    echo "✓ Frontend stopped"
else
    echo "No frontend server running on port 5173"
fi

# Clean up log files
if [ -f "backend.log" ]; then
    rm backend.log
    echo "✓ Removed backend.log"
fi

if [ -f "frontend.log" ]; then
    rm frontend.log
    echo "✓ Removed frontend.log"
fi

echo ""
echo "Demo stopped successfully"

