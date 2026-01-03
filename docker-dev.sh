#!/bin/bash
# Local Docker Build & Run Script
# Usage: ./docker-dev.sh [build|run|stop|clean|rebuild|logs]

set -e

IMAGE_NAME="rust-composition"
CONTAINER_NAME="rust-composition-api"
PORT="3000"
API_PORT="${API_PORT:-3000}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}==================================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}==================================================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
}

build_image() {
    print_header "Building Docker Image"
    check_docker
    
    local git_hash=$(git rev-parse --short HEAD 2>/dev/null || echo 'dev')
    docker build -t "${IMAGE_NAME}:latest" -t "${IMAGE_NAME}:${git_hash}" .
    
    print_success "Docker image built"
    docker images | grep "${IMAGE_NAME}"
}

run_container() {
    print_header "Running Docker Container"
    check_docker
    
    # Build first
    build_image
    
    # Stop existing container
    if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        print_info "Stopping existing container..."
        docker stop "${CONTAINER_NAME}" 2>/dev/null || true
        docker rm "${CONTAINER_NAME}" 2>/dev/null || true
    fi
    
    # Run new container
    print_info "Starting container: ${CONTAINER_NAME}"
    docker run -d \
        --name "${CONTAINER_NAME}" \
        -p "${PORT}:${API_PORT}" \
        -e "API_PORT=${API_PORT}" \
        "${IMAGE_NAME}:latest"
    
    print_success "Container running at http://localhost:${PORT}"
    print_info "View logs: $0 logs"
    print_info "Stop container: $0 stop"
}

stop_container() {
    print_header "Stopping Docker Container"
    check_docker
    
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        docker stop "${CONTAINER_NAME}"
        docker rm "${CONTAINER_NAME}"
        print_success "Container stopped"
    else
        print_warning "Container not running"
    fi
}

clean() {
    print_header "Cleaning Docker Resources"
    check_docker
    
    stop_container
    
    if docker images --format '{{.Repository}}:{{.Tag}}' | grep -q "${IMAGE_NAME}"; then
        docker rmi "${IMAGE_NAME}:latest" 2>/dev/null || true
        print_success "Docker image removed"
    else
        print_warning "Image not found"
    fi
}

rebuild() {
    clean
    run_container
    print_success "Full rebuild complete"
}

show_logs() {
    print_header "Following Container Logs (Ctrl+C to exit)"
    check_docker
    
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        docker logs -f "${CONTAINER_NAME}"
    else
        print_error "Container not running"
        exit 1
    fi
}

show_usage() {
    cat << EOF
${BLUE}Rust CQRS Composition - Docker Development Tool${NC}

${YELLOW}Usage:${NC}
    $0 [command]

${YELLOW}Commands:${NC}
    build       Build Docker image
    run         Build and run container (rebuilds image)
    stop        Stop and remove container
    clean       Remove container and image
    rebuild     Full rebuild (clean + build + run)
    logs        Follow container logs
    help        Show this help message

${YELLOW}Examples:${NC}
    # Build and run the container
    $0 run

    # Rebuild everything
    $0 rebuild

    # View logs
    $0 logs

    # Stop and clean up
    $0 stop

${YELLOW}Environment Variables:${NC}
    API_PORT    Port the API listens on (default: 3000)

${YELLOW}Examples with env vars:${NC}
    API_PORT=8080 $0 run
    # Container will run on http://localhost:3000 but API on 8080

EOF
}

# Main script logic
case "${1:-help}" in
    build)
        build_image
        ;;
    run)
        run_container
        ;;
    stop)
        stop_container
        ;;
    clean)
        clean
        ;;
    rebuild)
        rebuild
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_usage
        exit 1
        ;;
esac
