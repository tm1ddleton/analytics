@echo off
REM Stop the Analytics Platform Docker containers

echo Stopping Analytics Platform...
echo.

docker compose down

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✓ Services stopped successfully!
) else (
    echo.
    echo ✗ Failed to stop services. Check the error messages above.
    exit /b 1
)

