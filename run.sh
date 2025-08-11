#!/bin/bash

# Exit on any error
set -e

echo "  Vulkan Triangle Demo"
echo "======================="

echo "  Cheking dependencies..."

# Check if glslc is available
if ! command -v glslc &> /dev/null; then
    echo "  Error: glslc not found. Please install the Vulkan SDK."
    echo "  Visit: https://vulkan.lunarg.com/"
    exit 1
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "  Error: cargo not found. Please install Rust."
    echo "  Visit: https://rustup.rs/"
    exit 1
fi

echo "  Compiling shaders..."

# Create shaders directory if it doesn't exist
mkdir -p shaders

# Compile vertex shader
glslc shaders/shader.vert -o shaders/vert.spv

# Compile fragment shader
glslc shaders/shader.frag -o shaders/frag.spv

echo "  Shaders compiled successfully!"

echo "  Building Rust project..."
cargo build --release

echo "  Running Vulkan demo..."
cargo run --release
