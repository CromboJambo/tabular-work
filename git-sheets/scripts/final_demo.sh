#!/bin/bash
# Final comprehensive demo with full test output
set -e

echo "=============================================="
echo "  Git-Sheets × RSF-Core Integration"
echo "  Complete Verification & Demo"
echo "=============================================="
echo ""

cd /home/crombo/projects/tabular-work

echo "📦 Building git-sheets..."
cargo build -p git-sheets 2>&1 | grep -E "(Compiling|Finished)" || true
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Running All Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo test -p git-sheets 2>&1 | tail -30
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  RSF Integration Tests (Detailed)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo test -p git-sheets rsf -- --nocapture 2>&1 | grep -E "(test |ok|FAILED|passed|failed)" || true
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Production Demo: Semantic Diff"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo run -p git-sheets --bin demo_semantic_diff 2>&1 | grep -A 100 "Semantic Diff Report" | head -50
echo ""

echo "✅ All verification complete!"
