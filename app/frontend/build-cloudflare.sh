#!/bin/bash
set -e

# Cloudflare Pages build script for Yew frontend
echo "Building Kazoeru-kun frontend for Cloudflare Pages..."

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Add wasm target
rustup target add wasm32-unknown-unknown

# Install trunk
cargo install trunk

# Change to frontend directory
cd app/frontend/yew-app

# Install Node dependencies
npm install

# Build Tailwind CSS
npm run prod:css

# Build Yew app
trunk build --release

echo "Build complete! Static files in app/frontend/yew-app/dist"