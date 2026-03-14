#!/bin/bash

# Quick Start Guide for Zeo Benchmark Framework

echo "╔════════════════════════════════════════════════════════════╗"
echo "║        Zeo Benchmark Framework - Quick Start Guide          ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check current directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the benchmarks directory"
    echo "   Usage: cd /home/chao/workspace/zeo/benchmarks && ./quick_start.sh"
    exit 1
fi

echo "📋 Prerequisites Check:"
echo "─────────────────────────────────────────────────────────────"

# Check Rust
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust/Cargo: $RUST_VERSION"
else
    echo "❌ Rust/Cargo: Not found"
    echo "   Install from: https://rustup.rs"
    exit 1
fi

# Check Node.js
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo "✅ Node.js: $NODE_VERSION"
else
    echo "⚠️  Node.js: Not found (optional, for network tests)"
fi

# Check Bun
if command -v bun &> /dev/null; then
    BUN_VERSION=$(bun --version)
    echo "✅ Bun: $BUN_VERSION"
else
    echo "⚠️  Bun: Not found (optional, for comparison)"
    echo "   Install from: https://bun.sh"
fi

echo ""
echo "🔧 Setup Steps:"
echo "─────────────────────────────────────────────────────────────"

# Step 1: Build
echo ""
echo "Step 1: Building benchmark framework..."
cargo build --release
if [ $? -eq 0 ]; then
    echo "✅ Build completed successfully"
else
    echo "❌ Build failed"
    exit 1
fi

# Step 2: Create reports directory
echo ""
echo "Step 2: Setting up directories..."
mkdir -p reports
mkdir -p reports/trends
echo "✅ Directories created"

# Step 3: Validate configuration
echo ""
echo "Step 3: Validating configuration..."
if [ -f "config.toml" ]; then
    echo "✅ Configuration file found"
else
    echo "❌ Configuration file missing"
    exit 1
fi

# Step 4: Check test files
echo ""
echo "Step 4: Checking test files..."
TEST_COUNT=$(find tests -name "*.js" | wc -l)
echo "✅ Found $TEST_COUNT test files"

echo ""
echo "🎯 Ready to Run!"
echo "─────────────────────────────────────────────────────────────"
echo ""
echo "Choose an option:"
echo "  1. Run full benchmark suite"
echo "  2. Run specific scenario"
echo "  3. Run mobile benchmarks"
echo "4. View documentation"
echo "  5. Exit"
echo ""
read -p "Enter choice (1-5): " choice

case $choice in
    1)
        echo ""
        echo "🚀 Running full benchmark suite..."
        ./run_benchmarks.sh
        ;;
    2)
        echo ""
        echo "Available scenarios:"
        echo "  1. JavaScript Execution"
        echo "  2. Module Loading"
        echo "  3. File I/O"
        echo "  4. Network Requests"
        echo "  5. AI Agent Execution"
        echo ""
        read -p "Enter scenario number (1-5): " scenario
        
        case $scenario in
            1) echo "Running JavaScript execution benchmark..." ;;
            2) echo "Running module loading benchmark..." ;;
            3) echo "Running file I/O benchmark..." ;;
            4) echo "Running network requests benchmark..." ;;
            5) echo "Running AI agent execution benchmark..." ;;
            *) echo "Invalid choice" ;;
        esac
        ;;
    3)
        echo ""
        echo "📱 Mobile Benchmark Setup"
        echo "─────────────────────────────────────────────────────────────"
        echo "Available platforms:"
        echo "  1. iOS"
        echo "  2. Android"
        echo ""
        read -p "Enter platform (1-2): " platform
        
        case $platform in
            1) 
                echo "Running iOS benchmarks..."
                ./scripts/run_mobile_benchmarks.sh --ios
                ;;
            2) 
                echo "Running Android benchmarks..."
                ./scripts/run_mobile_benchmarks.sh --android
                ;;
            *) 
                echo "Invalid choice"
                ;;
        esac
        ;;
    4)
        echo ""
        echo "📚 Available Documentation:"
        echo "─────────────────────────────────────────────────────────────"
        echo "  README.md              - User guide and quick reference"
        echo "  ARCHITECTURE.md        - Detailed architecture documentation"
        echo "  IMPLEMENTATION_SUMMARY.md - Implementation overview"
        echo ""
        read -p "Enter file name to view: " doc_file
        
        if [ -f "$doc_file" ]; then
            less "$doc_file"
        else
            echo "File not found: $doc_file"
        fi
        ;;
    5)
        echo ""
        echo "👋 Goodbye!"
        exit 0
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "═════════════════════════════════════════════════════════════"
echo "📊 View Results:"
echo "─────────────────────────────────────────────────────────────"
echo "Generated reports are available in the ./reports/ directory:"
echo "  • benchmark_report.json  - Raw data"
echo "  • benchmark_report.md    - Markdown summary"
echo "  • benchmark_report.html   - Interactive report"
echo ""
echo "Open the HTML report for detailed analysis:"
echo "  open reports/benchmark_report.html"
echo "═════════════════════════════════════════════════════════════"