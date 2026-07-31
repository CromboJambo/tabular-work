#!/bin/bash
# End-to-end CLI demo with real CSV files

set -e

cd /home/crombo/projects/tabular-work/git-sheets

echo "=============================================="
echo "  Git-Sheets CLI Demo with Real CSV Files"
echo "=============================================="
echo ""

DEMO_DIR="/home/crombo/projects/tabular-work/git-sheets/demo_data"
BINARY="/home/crombo/projects/tabular-work/target/debug/git-sheets"

# Create fresh test repo
TEST_REPO="/tmp/git-sheets-demo-$$"
rm -rf "$TEST_REPO"
mkdir -p "$TEST_REPO"

echo "📁 Created test repository: $TEST_REPO"
echo ""

# Demo 1: Initialize and snapshot v1
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 1: Initialize repo & snapshot employees_v1.csv"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cp "$DEMO_DIR/employees_v1.csv" "$TEST_REPO/data.csv"
cd "$TEST_REPO"
"$BINARY" init .
"$BINARY" snapshot data.csv -m "Initial employee roster (5 employees)"
echo ""

# Demo 2: Snapshot v2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 2: Snapshot employees_v2.csv"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cp "$DEMO_DIR/employees_v2.csv" "$TEST_REPO/data.csv"
"$BINARY" snapshot data.csv -m "Added new employee (6 total)"
echo ""

# Demo 3: Show diff
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 3: Compare v1 vs v2"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
"$BINARY" log | head -3
echo ""

# Demo 4: Snapshot v3
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 4: Snapshot employees_v3.csv"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cp "$DEMO_DIR/employees_v3.csv" "$TEST_REPO/data.csv"
"$BINARY" snapshot data.csv -m "Changes: Bob inactive, new Marketing hire"
echo ""

# Demo 5: Show all snapshots
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 5: View all snapshots"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
"$BINARY" log
echo ""

# Cleanup
cd /home/crombo/projects/tabular-work/git-sheets
rm -rf "$TEST_REPO"

echo "✅ Demo complete!"
echo ""
echo "What to notice:"
echo "  • Column types detected automatically (ID, Currency, Date)"
echo "  • Semantic issues: Lost $ symbols in Salary column"
echo "  • Boolean changes: Active field went false for Bob"
echo "  • Cardinality tracking: EmployeeID maintains high uniqueness"
