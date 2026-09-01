#!/bin/bash
# Convert cleaned CSV to markdown table for Zed viewing

set -e
cd /home/crombo/projects/tabular-work

INPUT_CSV="$1"
OUTPUT_MD="${INPUT_CSV%.csv}_view.md"

if [ -z "$INPUT_CSV" ]; then
    echo "Usage: ./tools/csv_to_markdown_view.sh <input.csv>"
    echo "Example: ./tools/csv_to_markdown_view.sh OUTPUT/job_orders_clean.csv"
    exit 1
fi

if [ ! -f "$INPUT_CSV" ]; then
    echo "❌ Input file not found: $INPUT_CSV"
    exit 1
fi

echo "=== Converting CSV to Markdown Table ==="
python3 tools/csv_to_markdown.py "$INPUT_CSV" "$OUTPUT_MD"

echo ""
echo "✅ View in Zed:"
echo "   Open: $OUTPUT_MD"
echo "   (Markdown tables render with proper alignment)"
