# Dockerfile for Backend

# === Stage 1: Builder ===
FROM rust:slim-bookworm AS builder

# Install system dependencies needed for compiling C libraries and downloading assets
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest files first to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies only
# This step saves time when only source code changes, not dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
# Build dependencies (release mode)
RUN cargo build --release

# Copy the actual source code
COPY . .
# Force SQLx to use offline mode (requires sqlx-data.json)
ENV SQLX_OFFLINE=true
# Touch source files to force recompilation of both lib and bin
RUN touch src/lib.rs src/main.rs
# Build the actual application
RUN cargo build --release

# === Stage 2: Runtime ===
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL is dynamic linked)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/backend /app/backend
# Copy configuration or migration files if needed (usually embedded, but sqlx migrations are embedded)
# If using .env, we don't copy it, we pass env vars via docker-compose

# Expose the port
EXPOSE 3000

# Command to run the application
CMD ["./backend"]
