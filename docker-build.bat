@echo off
REM Build the Analytics Platform Docker images

echo Building Analytics Platform Docker images...
echo.

REM Check if user wants to rebuild without cache
if "%1"=="--no-cache" (
    echo Building without cache...
    docker compose build --no-cache
) else (
    docker compose build
)

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✓ Build completed successfully!
    echo.
    echo To start services: docker compose up -d
    echo Or use: docker-up.bat
) else (
    echo.
    echo ✗ Build failed. Check the error messages above.
    exit /b 1
)

