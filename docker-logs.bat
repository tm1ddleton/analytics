@echo off
REM View logs from Analytics Platform containers

if "%1"=="" (
    echo Viewing logs for all services...
    echo Press Ctrl+C to exit
    echo.
    docker compose logs -f
) else (
    echo Viewing logs for service: %1
    echo Press Ctrl+C to exit
    echo.
    docker compose logs -f %1
)

