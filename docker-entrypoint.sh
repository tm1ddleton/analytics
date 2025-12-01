#!/bin/sh
set -e

# Ensure data directory exists (don't try to change permissions on mounted volumes)
mkdir -p /app/data 2>/dev/null || true

# Execute the main command
exec "$@"

