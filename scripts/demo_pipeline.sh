#!/bin/bash
# Demo script for warranty pipeline - Power Query alternative in Rust

set -e

echo "=== Warranty Pipeline Demo ==="
echo ""
echo "This demonstrates recreating Excel Power Query workflows using Rust."
echo ""

cd /home/crombo/projects/tabular-work

# Create output directory
mkdir -p OUTPUT

echo "Step 1: Loading MCT SharePoint data from Excel..."
echo "Using calamine (Rust-native Excel reader)..."

# Run the demo binary if it compiles, otherwise use Python fallback
if cargo run -p warranty-pipeline --bin warranty-pipeline -- demo 2>/dev/null; then
    echo "✓ Demo completed successfully"
else
    echo "Note: Using Python fallback for demo (Rust library has compilation issues)"
    echo ""
    
    # Fallback: Use Python with pandas/openpyxl
    python3 << 'PYEOF'
import pandas as pd
from pathlib import Path

print("Loading MCT SharePoint link sheet...")
df = pd.read_excel("DATA/WarrantyWorkbook.xlsx", sheet_name="MCT SharePoint link")
print(f"  Loaded {len(df)} rows, {len(df.columns)} columns")
print(f"  Columns: {list(df.columns)[:10]}...")

# Save as CSV
df.to_csv("OUTPUT/mct_raw.csv", index=False)
print("\nSaved to OUTPUT/mct_raw.csv")

# Filter closed items (STATUS column is typically at index 5)
status_col = [c for c in df.columns if 'status' in str(c).lower()]
if status_col:
    closed = df[df[status_col[0]].str.lower() == 'closed']
    print(f"\nFiltered {len(closed)} closed items")
    
    # Group by status and count
    summary = df.groupby(status_col[0]).size().reset_index(name='count')
    summary.to_csv("OUTPUT/status_summary.csv", index=False)
    print(f"Status distribution saved to OUTPUT/status_summary.csv")
else:
    print("\nSTATUS column not found, showing first few rows:")
    print(df.head())

print("\n=== Demo Complete ===")
PYEOF
fi

echo ""
echo "Output files created:"
ls -la OUTPUT/ 2>/dev/null || echo "  (no output files yet)"

echo ""
echo "Next steps:"
echo "1. Fix Rust compilation issues in warranty-pipeline crate"
echo "2. Run full pipeline: cargo run -p warranty-pipeline -- run warranty_pipeline.toml"
echo "3. Preview data: cargo run -p warranty-pipeline -- preview DATA/WarrantyWorkbook.xlsx --sheet 'MCT SharePoint link'"
