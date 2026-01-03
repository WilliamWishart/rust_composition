.PHONY: help build run stop clean rebuild logs test docker-build docker-run docker-stop docker-clean docker-rebuild

# Configuration
IMAGE_NAME := rust-composition
CONTAINER_NAME := rust-composition-api
PORT := 3000
API_PORT := 3000

help:
	@echo "==================================================================="
	@echo "  Rust CQRS Composition - Local Development Commands"
	@echo "==================================================================="
	@echo ""
	@echo "Build & Run:"
	@echo "  make build           - Build Rust project locally"
	@echo "  make run             - Run API locally (requires build)"
	@echo "  make rebuild         - Clean build + run locally"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build    - Build Docker image"
	@echo "  make docker-run      - Run container (rebuilds image)"
	@echo "  make docker-stop     - Stop running container"
	@echo "  make docker-clean    - Remove container and image"
	@echo "  make docker-rebuild  - Clean + rebuild + run"
	@echo ""
	@echo "Testing & Debugging:"
	@echo "  make test            - Run all tests"
	@echo "  make logs            - Show container logs (follow)"
	@echo ""
	@echo "Shortcuts:"
	@echo "  make b               - alias for docker-build"
	@echo "  make r               - alias for docker-run"
	@echo "  make s               - alias for docker-stop"
	@echo ""

# ============================================================================
# Local Rust Build
# ============================================================================

build:
	@echo "🔨 Building Rust project..."
	cargo build --all

run: build
	@echo "🚀 Running API locally..."
	API_PORT=$(API_PORT) cargo run --package api-rest

rebuild: clean build
	@echo "✅ Rebuild complete"

test:
	@echo "🧪 Running tests..."
	cargo test --all

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# ============================================================================
# Docker Build & Run
# ============================================================================

docker-build:
	@echo "🐳 Building Docker image: $(IMAGE_NAME)"
	docker build -t $(IMAGE_NAME):latest -t $(IMAGE_NAME):$$(git rev-parse --short HEAD 2>/dev/null || echo 'dev') .
	@echo "✅ Docker image built successfully"
	@docker images | grep $(IMAGE_NAME)

docker-run: docker-build
	@echo "🐳 Running Docker container: $(CONTAINER_NAME)"
	@if docker ps -a --format '{{.Names}}' | grep -q ^$(CONTAINER_NAME)$$; then \
		echo "⏹️  Stopping existing container..."; \
		docker stop $(CONTAINER_NAME) 2>/dev/null || true; \
		docker rm $(CONTAINER_NAME) 2>/dev/null || true; \
	fi
	docker run -d \
		--name $(CONTAINER_NAME) \
		-p $(PORT):$(API_PORT) \
		-e API_PORT=$(API_PORT) \
		$(IMAGE_NAME):latest
	@echo "✅ Container running at http://localhost:$(PORT)"
	@echo "📋 View logs: make logs"
	@echo "⏹️  Stop container: make docker-stop"

docker-stop:
	@echo "⏹️  Stopping Docker container: $(CONTAINER_NAME)"
	@docker stop $(CONTAINER_NAME) 2>/dev/null || echo "Container not running"
	@docker rm $(CONTAINER_NAME) 2>/dev/null || true
	@echo "✅ Container stopped"

docker-clean: docker-stop
	@echo "🧹 Removing Docker image: $(IMAGE_NAME)"
	@docker rmi $(IMAGE_NAME):latest 2>/dev/null || echo "Image not found"
	@echo "✅ Docker image removed"

docker-rebuild: docker-clean docker-run
	@echo "✅ Docker rebuild complete"

logs:
	@echo "📋 Following logs (Ctrl+C to exit)..."
	docker logs -f $(CONTAINER_NAME)

# ============================================================================
# Quick Aliases
# ============================================================================

b: docker-build
r: docker-run
s: docker-stop
c: docker-clean
