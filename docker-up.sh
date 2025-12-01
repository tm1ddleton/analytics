#!/bin/bash
# Start the Analytics Platform using Docker Compose
# This script starts both backend and frontend services
# If analytics.db exists in the project root, it will be used

set -e

echo "Starting Analytics Platform..."
echo ""

# Check if analytics.db exists in project root
if [ -f "analytics.db" ]; then
    echo "✓ Found analytics.db in project root"
    echo "  Using bind mount for database (analytics.db will be used)"
    
    # Create data directory if it doesn't exist (with proper permissions)
    if [ ! -d "data" ]; then
        mkdir -p data
        chmod 755 data
    fi
    
    # Copy database to data directory if it's not already there or is newer
    if [ ! -f "data/analytics.db" ] || [ "analytics.db" -nt "data/analytics.db" ]; then
        echo "  Copying analytics.db to data/ directory..."
        cp analytics.db data/analytics.db
        chmod 644 data/analytics.db
        echo "  ✓ Database copied"
    else
        echo "  ✓ Database already in data/ directory (up to date)"
    fi
else
    echo "ℹ No analytics.db found in project root"
    echo "  Using named Docker volume for database (empty database will be created)"
fi

echo ""
echo "Starting Docker containers..."
docker compose up -d

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Services started successfully!"
    echo ""
    echo "Frontend: http://localhost:5173"
    echo "Backend API: http://localhost:3000"
    echo ""
    echo "To view logs: docker compose logs -f"
    echo "To stop services: docker compose down"
else
    echo ""
    echo "✗ Failed to start services. Check the error messages above."
    exit 1
fi

