@echo off
REM Start the Analytics Platform using Docker Compose
REM This script starts both backend and frontend services
REM If analytics.db exists in the project root, it will be used

echo Starting Analytics Platform...
echo.

REM Check if analytics.db exists in project root
if exist "analytics.db" (
    echo Found analytics.db in project root
    echo   Using bind mount for database (analytics.db will be used)
    echo.
    
    REM Create data directory if it doesn't exist
    if not exist "data" mkdir data
    
    REM Copy database to data directory if it's not already there or is newer
    if not exist "data\analytics.db" (
        echo   Copying analytics.db to data\ directory...
        copy /Y analytics.db data\analytics.db >nul
        echo   ✓ Database copied
    ) else (
        REM Check if source is newer (Windows doesn't have easy file comparison, so we'll always copy)
        REM This ensures the latest version is used
        echo   Updating database in data\ directory...
        copy /Y analytics.db data\analytics.db >nul
        echo   ✓ Database updated
    )
    echo.
) else (
    echo No analytics.db found in project root
    echo   Using named Docker volume for database (empty database will be created)
    echo.
)

echo Starting Docker containers...
docker compose up -d

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✓ Services started successfully!
    echo.
    echo Frontend: http://localhost:5173
    echo Backend API: http://localhost:3000
    echo.
    echo To view logs: docker compose logs -f
    echo To stop services: docker compose down
) else (
    echo.
    echo ✗ Failed to start services. Check the error messages above.
    exit /b 1
)

