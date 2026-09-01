#!/bin/bash
# Format CSV columns to match first row width
# Works with Rainbow CSV extension in Zed

set -e
cd /home/crombo/projects/tabular-work

INPUT_CSV="$1"
OUTPUT_CSV="${INPUT_CSV%.csv}_formatted.csv"

if [ -z "$INPUT_CSV" ]; then
    echo "Usage: ./tools/csv_column_align.sh <input.csv>"
    echo "Example: ./tools/csv_column_align.sh OUTPUT/job_orders_clean.csv"
    exit 1
fi

if [ ! -f "$INPUT_CSV" ]; then
    echo "❌ Input file not found: $INPUT_CSV"
    exit 1
fi

echo "=== Aligning CSV Columns ==="
echo "Input:  $INPUT_CSV"
echo "Output: $OUTPUT_CSV"

python3 tools/csv_column_formatter.py "$INPUT_CSV" "$OUTPUT_CSV"

echo ""
echo "✅ Open in Zed with Rainbow CSV:"
echo "   1. Install Rainbow CSV extension (Ctrl+Shift+X → search 'rainbow-csv')"
echo "   2. Open: $OUTPUT_CSV"
echo "   3. All columns now align with first row!"
