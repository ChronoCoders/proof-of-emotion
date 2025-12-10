#!/bin/bash
# Build and test script for Proof of Emotion

set -e

echo "🚀 Building Proof of Emotion Consensus"
echo "======================================="
echo ""

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust toolchain found"
rustc --version
cargo --version
echo ""

# Format check
echo "📝 Checking code formatting..."
cargo fmt --all -- --check 2>/dev/null || cargo fmt --all
echo "✅ Code formatted"
echo ""

# Build
echo "🔨 Building project..."
cargo build --release
echo "✅ Build successful"
echo ""

# Run tests
echo "🧪 Running tests..."
cargo test --all
echo "✅ All tests passed"
echo ""

# Build examples
echo "📦 Building examples..."
cargo build --examples
echo "✅ Examples built"
echo ""

# Check documentation
echo "📚 Building documentation..."
cargo doc --no-deps
echo "✅ Documentation generated"
echo ""

echo "🎉 All checks passed!"
echo ""
echo "To run examples:"
echo "  cargo run --example basic_consensus"
echo "  cargo run --example multi_validator"
echo "  cargo run --example staking_rewards"
