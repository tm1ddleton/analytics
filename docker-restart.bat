@echo off
REM Restart the Analytics Platform Docker containers

echo Restarting Analytics Platform...
echo.

docker compose restart

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✓ Services restarted successfully!
    echo.
    echo To view logs: docker compose logs -f
) else (
    echo.
    echo ✗ Failed to restart services. Check the error messages above.
    exit /b 1
)

