# Stage 1: Build stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Copy workspace and crate files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src

# Build the api-rest binary in release mode
RUN cargo build --release --package api-rest

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from builder
COPY --from=builder /usr/src/app/target/release/api-rest .

# Port can be customized via API_PORT environment variable (default: 3000)
EXPOSE 3000

# Run the API
CMD ["./api-rest"]
