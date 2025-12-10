#!/bin/bash

echo "🔍 Verification Script for Proof of Emotion"
echo "==========================================="
echo ""

# Check all source files exist
echo "📁 Checking source files..."
files=(
    "src/lib.rs"
    "src/error.rs"
    "src/types.rs"
    "src/crypto.rs"
    "src/biometric.rs"
    "src/consensus.rs"
    "src/staking.rs"
    "src/utils.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        echo "  ✓ $file ($lines lines)"
    else
        echo "  ✗ $file MISSING"
    fi
done

echo ""
echo "📝 Checking examples..."
examples=(
    "examples/basic_consensus.rs"
    "examples/multi_validator.rs"
    "examples/staking_rewards.rs"
)

for ex in "${examples[@]}"; do
    if [ -f "$ex" ]; then
        lines=$(wc -l < "$ex")
        echo "  ✓ $ex ($lines lines)"
    else
        echo "  ✗ $ex MISSING"
    fi
done

echo ""
echo "🧪 Checking tests..."
if [ -f "tests/integration_tests.rs" ]; then
    lines=$(wc -l < "tests/integration_tests.rs")
    echo "  ✓ tests/integration_tests.rs ($lines lines)"
else
    echo "  ✗ tests/integration_tests.rs MISSING"
fi

echo ""
echo "🔧 Checking Arc usage..."
arc_count=$(grep -r "Arc::" src/ examples/ tests/ 2>/dev/null | wc -l)
echo "  Found $arc_count Arc usages"

echo ""
echo "📦 Checking imports..."
echo "  BiometricDevice trait imported in consensus.rs:"
grep "use crate::biometric::BiometricDevice" src/consensus.rs && echo "    ✓ Yes" || echo "    ✗ No"

echo ""
echo "  std::sync::Arc imported where needed:"
grep -l "use std::sync::Arc" src/*.rs examples/*.rs tests/*.rs 2>/dev/null | wc -l
echo "    files with Arc import"

echo ""
echo "🔍 Checking for common issues..."

echo "  Unused imports:"
grep -n "^use.*;" src/*.rs | grep -v "^src/lib.rs" | wc -l
echo "    potential unused imports found"

echo ""
echo "  Dead code fields:"
grep -n "^\s*[a-z_]*:" src/*.rs | wc -l
echo "    struct fields defined"

echo ""
echo "✅ Verification complete!"
echo ""
echo "To build and test:"
echo "  cargo check    # Check for compilation errors"
echo "  cargo build    # Build the project"
echo "  cargo test     # Run tests"
echo "  cargo run --example basic_consensus"
