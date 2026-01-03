#!/bin/bash
# QUICK REFERENCE - Copy/paste these commands

# ============================================================================
# DOCKER - BUILD & RUN
# ============================================================================

# ONE COMMAND TO START EVERYTHING
./quick-start.sh

# OR use Makefile (recommended for development)
make docker-run          # Build and run
make docker-rebuild      # Full rebuild (clean + build + run)
make docker-stop         # Stop container
make docker-clean        # Remove container and image
make logs                # Follow logs

# OR use Shell script
./docker-dev.sh run      # Build and run
./docker-dev.sh rebuild  # Full rebuild
./docker-dev.sh stop     # Stop
./docker-dev.sh logs     # View logs

# OR use docker-compose
docker-compose up --build -d    # Start
docker-compose logs -f api      # Follow logs
docker-compose down             # Stop

# ============================================================================
# TEST THE API
# ============================================================================

# View Swagger UI
open http://localhost:3000/swagger-ui/

# Or test with curl
curl http://localhost:3000/swagger-ui/

# Register a user
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"user_id": 1, "name": "Alice"}'

# Get all users
curl http://localhost:3000/users

# Get user by ID
curl http://localhost:3000/users/1

# Search by name
curl http://localhost:3000/users/search/Alice

# Rename user
curl -X PUT http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"user_id": 1, "new_name": "Alice Smith"}'

# ============================================================================
# DEVELOPMENT WORKFLOW
# ============================================================================

# Terminal 1: Build and run
make docker-run

# Terminal 2: Follow logs
make logs

# Terminal 3: Test API
curl http://localhost:3000/swagger-ui/

# After code changes
make docker-rebuild

# ============================================================================
# CUSTOMIZATION
# ============================================================================

# Use different port
PORT=8080 make docker-run

# Debug mode
RUST_LOG=debug make docker-run

# Watch mode (auto-rebuild on changes)
brew install watchexec  # if not installed
watchexec "make docker-rebuild"

# ============================================================================
# CLEANUP
# ============================================================================

# Stop container
make docker-stop

# Remove container and image
make docker-clean

# Remove everything (including volumes)
docker system prune -a

# ============================================================================
# LOCAL RUST (No Docker)
# ============================================================================

cargo build --all          # Build locally
cargo test --all           # Run tests
cargo run -p api-rest      # Run REST server locally (port 3000)

# ============================================================================
# MORE HELP
# ============================================================================

# Show all Makefile commands
make help

# Show shell script help
./docker-dev.sh help

# View full documentation
cat DOCKER_DEV_GUIDE.md

# View setup summary
cat SETUP_COMPLETE.md

# ============================================================================
# USEFUL COMMANDS
# ============================================================================

# List running containers
docker ps

# List all containers
docker ps -a

# List Docker images
docker images

# View container logs
docker logs rust-composition-api

# View live logs
docker logs -f rust-composition-api

# Execute command in running container
docker exec rust-composition-api cargo test

# Stop all containers
docker stop $(docker ps -q)

# Remove all containers
docker rm $(docker ps -a -q)

# Check Docker info
docker info

# ============================================================================
# STATUS CHECKS
# ============================================================================

# Is container running?
docker ps | grep rust-composition-api

# Is API responding?
curl -I http://localhost:3000/swagger-ui/

# View startup logs
docker logs rust-composition-api | tail -20

# ============================================================================
# DOCUMENTATION LINKS
# ============================================================================

# For complete information, read these files:
# - DOCKER_DEV_GUIDE.md         (Complete Docker reference)
# - DOCKER_AUTOMATION.md        (Automation setup details)
# - SETUP_COMPLETE.md           (This setup completion summary)
# - Makefile                     (All available targets)
# - docker-dev.sh               (Shell script with help)
# - docker-compose.yml          (Orchestration config)
