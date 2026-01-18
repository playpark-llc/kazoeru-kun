# ======================================
# Development Dockerfile with hot reload
# ======================================
FROM rust:1.88.0-slim-bullseye

# Install build dependencies and development tools
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install development tools (pinned versions for reproducible builds)
RUN cargo install cargo-watch --version 8.5.3 && \
    cargo install cargo-make --version 0.37.24 && \
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