# ======================================
# Development Dockerfile with hot reload
# ======================================
FROM rust:1.83.0-slim-bullseye

# Install build dependencies and development tools
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install development tools
RUN cargo install cargo-watch cargo-make && \
    cargo install --locked sea-orm-cli --version 1.1.0 && \
    rustup component add rustfmt clippy

# Set working directory
WORKDIR /opt/app/backend

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Pre-build dependencies for faster development
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build && \
    rm -rf src

# Copy source code
COPY src ./src
COPY migration ./migration
COPY .env ./
COPY entrypoint.sh /entrypoint.sh

# Make entrypoint executable
RUN chmod +x /entrypoint.sh

# Development command with hot reload
CMD ["cargo", "watch", "-x", "run"]