#!/bin/bash

# Zeo Benchmark Runner Script
# This script runs the benchmark suite comparing Zeo vs Bun

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║          Zeo Performance Benchmark Runner                    ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if we're in the benchmarks directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the benchmarks directory"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust from https://rustup.rs"
    exit 1
fi

# Check if Node.js is installed (for test server)
if ! command -v node &> /dev/null; then
    echo "Warning: Node.js not found. Network benchmarks will be skipped."
    NO_NODE=true
fi

# Check if Bun is installed
if ! command -v bun &> /dev/null; then
    echo "Warning: Bun is not installed. Install it from https://bun.sh"
    echo "Benchmark will run with partial functionality."
    NO_BUN=true
fi

# Build the benchmark framework
echo "Building benchmark framework..."
cargo build --release

# Create reports directory
mkdir -p reports

# Start test server for network benchmarks if Node.js is available
if [ "$NO_NODE" != true ]; then
    echo "Starting test HTTP server for network benchmarks..."
    node tests/network/test_server.js &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 2
    echo "Test server started (PID: $SERVER_PID)"
fi

# Run benchmarks
echo ""
echo "Running benchmark suite..."
echo "═════════════════════════════════════════════════════════════"
echo ""

cargo run --release

# Cleanup
echo ""
echo "Cleaning up..."

if [ "$NO_NODE" != true ]; then
    echo "Stopping test server..."
    kill $SERVER_PID 2>/dev/null || true
fi

echo ""
echo "═════════════════════════════════════════════════════════════"
echo "Benchmark completed successfully!"
echo ""
echo "Reports generated in ./reports/:"
echo "  - benchmark_report.json (raw data)"
echo "  - benchmark_report.md (markdown summary)"
echo "  - benchmark_report.html (interactive report)"
echo ""
echo "View the HTML report for detailed analysis:"
echo "  open reports/benchmark_report.html"
echo "═════════════════════════════════════════════════════════════"