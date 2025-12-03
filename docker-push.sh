#!/bin/bash
# Script to build and push Docker images to Docker Hub

# Configuration - UPDATE THESE VALUES
DOCKER_HUB_USERNAME="your-username"  # Replace with your Docker Hub username
IMAGE_TAG="${1:-latest}"  # Use first argument as tag, default to "latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building and pushing Docker images to Docker Hub${NC}"
echo -e "Username: ${YELLOW}${DOCKER_HUB_USERNAME}${NC}"
echo -e "Tag: ${YELLOW}${IMAGE_TAG}${NC}"
echo ""

# Check if logged in to Docker Hub
if ! docker info | grep -q "Username"; then
    echo -e "${YELLOW}Not logged in to Docker Hub. Please login first:${NC}"
    echo "docker login"
    exit 1
fi

# Build and tag backend image
echo -e "${GREEN}Building backend image...${NC}"
docker build -t ${DOCKER_HUB_USERNAME}/analytics-backend:${IMAGE_TAG} .
if [ $? -ne 0 ]; then
    echo -e "${RED}Backend build failed!${NC}"
    exit 1
fi

# Build and tag frontend image
echo -e "${GREEN}Building frontend image...${NC}"
docker build -t ${DOCKER_HUB_USERNAME}/analytics-frontend:${IMAGE_TAG} -f frontend/Dockerfile frontend/
if [ $? -ne 0 ]; then
    echo -e "${RED}Frontend build failed!${NC}"
    exit 1
fi

# Push backend image
echo -e "${GREEN}Pushing backend image...${NC}"
docker push ${DOCKER_HUB_USERNAME}/analytics-backend:${IMAGE_TAG}
if [ $? -ne 0 ]; then
    echo -e "${RED}Backend push failed!${NC}"
    exit 1
fi

# Push frontend image
echo -e "${GREEN}Pushing frontend image...${NC}"
docker push ${DOCKER_HUB_USERNAME}/analytics-frontend:${IMAGE_TAG}
if [ $? -ne 0 ]; then
    echo -e "${RED}Frontend push failed!${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✓ Successfully pushed both images to Docker Hub!${NC}"
echo ""
echo "Images pushed:"
echo "  - ${DOCKER_HUB_USERNAME}/analytics-backend:${IMAGE_TAG}"
echo "  - ${DOCKER_HUB_USERNAME}/analytics-frontend:${IMAGE_TAG}"


