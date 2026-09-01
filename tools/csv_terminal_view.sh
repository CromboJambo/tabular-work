#!/bin/bash
# Create terminal-friendly view of cleaned CSV
# Selects key columns + truncates long text

set -e
cd /home/crombo/projects/tabular-work

INPUT_CSV="$1"
OUTPUT_TSV="${INPUT_CSV%.csv}_view.tsv"

if [ -z "$INPUT_CSV" ]; then
    echo "Usage: ./tools/csv_terminal_view.sh <input.csv>"
    exit 1
fi

python3 tools/csv_select_truncate.py "$INPUT_CSV" "$OUTPUT_TSV" 20

echo ""
echo "✅ Open in Zed:"
echo "   $OUTPUT_TSV (6 key columns, truncated to 20 chars)"
echo ""
echo "Columns: Trans Date | Customer | SRO | Description | Item | Comments"
