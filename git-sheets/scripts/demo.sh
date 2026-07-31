#!/bin/bash
# Production-quality demo script for git-sheets × rsf-core integration
set -e

echo "=============================================="
echo "  Git-Sheets × RSF-Core Integration Demo"
echo "=============================================="
echo ""

cd /home/crombo/projects/tabular-work

echo "🔧 Step 1: Building project..."
cargo build -p git-sheets --quiet
echo "   ✓ Build successful"
echo ""

echo "🧪 Step 2: Running test suite..."
TEST_OUTPUT=$(cargo test -p git-sheets --quiet 2>&1)
TOTAL_TESTS=$(echo "$TEST_OUTPUT" | grep -oP '\d+ passed' | tail -1 | grep -oP '\d+')
FAILED=$(echo "$TEST_OUTPUT" | grep -oP '\d+ failed' | grep -oP '\d+' || echo "0")
echo "   ✓ $TOTAL_TESTS tests passed, $FAILED failed"
echo ""

echo "📊 Step 3: Running comprehensive RSF integration tests..."
cargo test -p git-sheets rsf --quiet 2>&1 | tail -5
echo ""

echo "🎬 Step 4: Executing semantic diff demo..."
echo "----------------------------------------------"
cargo run -p git-sheets --bin demo_semantic_diff --quiet
echo "----------------------------------------------"
echo ""

echo "✅ Demo complete!"
echo ""
echo "Summary:"
echo "  • rsf-core provides column profiling (cardinality, type hints)"
echo "  • git-sheets provides version control with snapshots"
echo "  • Integration detects semantic changes: type mismatches, cardinality drops"
echo ""
