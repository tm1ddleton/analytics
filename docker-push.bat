@echo off
REM Script to build and push Docker images to Docker Hub (Windows)

REM Configuration - UPDATE THESE VALUES
set DOCKER_HUB_USERNAME=your-username
set IMAGE_TAG=%1
if "%IMAGE_TAG%"=="" set IMAGE_TAG=latest

echo Building and pushing Docker images to Docker Hub
echo Username: %DOCKER_HUB_USERNAME%
echo Tag: %IMAGE_TAG%
echo.

REM Check if logged in to Docker Hub
docker info | findstr /i "Username" >nul
if errorlevel 1 (
    echo Not logged in to Docker Hub. Please login first:
    echo docker login
    exit /b 1
)

REM Build and tag backend image
echo Building backend image...
docker build -t %DOCKER_HUB_USERNAME%/analytics-backend:%IMAGE_TAG% .
if errorlevel 1 (
    echo Backend build failed!
    exit /b 1
)

REM Build and tag frontend image
echo Building frontend image...
docker build -t %DOCKER_HUB_USERNAME%/analytics-frontend:%IMAGE_TAG% -f frontend/Dockerfile frontend/
if errorlevel 1 (
    echo Frontend build failed!
    exit /b 1
)

REM Push backend image
echo Pushing backend image...
docker push %DOCKER_HUB_USERNAME%/analytics-backend:%IMAGE_TAG%
if errorlevel 1 (
    echo Backend push failed!
    exit /b 1
)

REM Push frontend image
echo Pushing frontend image...
docker push %DOCKER_HUB_USERNAME%/analytics-frontend:%IMAGE_TAG%
if errorlevel 1 (
    echo Frontend push failed!
    exit /b 1
)

echo.
echo Successfully pushed both images to Docker Hub!
echo.
echo Images pushed:
echo   - %DOCKER_HUB_USERNAME%/analytics-backend:%IMAGE_TAG%
echo   - %DOCKER_HUB_USERNAME%/analytics-frontend:%IMAGE_TAG%


