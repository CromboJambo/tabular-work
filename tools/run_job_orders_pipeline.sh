#!/bin/bash
# Job Orders to Close - Quick Workflow

set -e
cd /home/crombo/projects/tabular-work

INPUT_XLSX="DATA/JobOrders_Extracted/Job Orders to Close/Job Orders to Close.xlsx"
OUTPUT_CSV="OUTPUT/job_orders_clean.csv"

echo "=== Job Orders to Close Pipeline ==="

# Check if input exists
if [ ! -f "$INPUT_XLSX" ]; then
    echo "❌ Input file not found: $INPUT_XLSX"
    echo "   Make sure you've extracted the OneDrive export first"
    exit 1
fi

# Convert xlsx to clean CSV
echo "📝 Converting Excel to UTF-8 CSV..."
python3 tools/read_job_orders.py "$INPUT_XLSX" "$OUTPUT_CSV"

# Verify output
if [ -f "$OUTPUT_CSV" ]; then
    ROW_COUNT=$(tail -n +2 "$OUTPUT_CSV" | wc -l)
    echo "✅ Exported $ROW_COUNT job orders"
    echo "📂 Output: $OUTPUT_CSV"
else
    echo "❌ Conversion failed"
    exit 1
fi
