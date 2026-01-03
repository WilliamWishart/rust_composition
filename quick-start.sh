#!/bin/bash
# Quick Start Script - One command to get everything running

set -e

echo "🚀 Rust CQRS Composition - Quick Start"
echo "======================================"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker daemon not running. Please start Docker."
    exit 1
fi

echo "✅ Docker is running"
echo ""

# Build and run
echo "🐳 Building Docker image (this may take a minute)..."
docker build -t rust-composition:latest .

echo ""
echo "🚀 Starting API container..."

# Stop existing container if running
docker ps -a --format '{{.Names}}' | grep -q rust-composition-api && docker stop rust-composition-api 2>/dev/null && docker rm rust-composition-api 2>/dev/null || true

# Run container
docker run -d \
    --name rust-composition-api \
    -p 3000:3000 \
    -e API_PORT=3000 \
    rust-composition:latest

echo ""
echo "✅ API is running!"
echo ""
echo "📚 API Documentation:"
echo "   - Swagger UI: http://localhost:3000/swagger-ui/"
echo "   - OpenAPI JSON: http://localhost:3000/openapi.json"
echo ""
echo "📝 Test the API:"
echo "   - REST client (Bruno): ./bruno/REST-API.bru"
echo "   - cURL: curl http://localhost:3000/swagger-ui/"
echo ""
echo "📋 View logs:"
echo "   make logs"
echo "   or"
echo "   docker logs -f rust-composition-api"
echo ""
echo "⏹️  Stop container:"
echo "   make docker-stop"
echo "   or"
echo "   docker stop rust-composition-api"
echo ""
echo "For more commands, run: make help"
