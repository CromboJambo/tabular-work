#!/bin/bash
# Convert all UTF-16 Excel exports to UTF-8 CSV

set -e
cd /home/crombo/projects/tabular-work

INPUT_DIR="DATA/OneDrive_Extracted"
OUTPUT_DIR="OUTPUT"

echo "=== Converting UTF-16 Excel Exports to UTF-8 ==="

# Process each CSV file
for file in "$INPUT_DIR"/*.csv; do
    if [ -f "$file" ]; then
        basename=$(basename "$file" .csv)
        output="$OUTPUT_DIR/${basename}_utf8.csv"
        
        echo "Converting: $basename..."
        python3 tools/convert_utf16.py "$file" "$output"
    fi
done

echo "=== Conversion Complete ==="
echo "Output files in: $OUTPUT_DIR/*_utf8.csv"
