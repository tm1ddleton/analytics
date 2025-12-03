@echo off
REM Show status of Analytics Platform containers

echo Analytics Platform Status:
echo.

docker compose ps

echo.
echo To view logs: docker compose logs -f
echo To restart: docker compose restart
echo To stop: docker compose down

