#!/usr/bin/env python3
"""
Convert Infor Syteline UTF-16 Excel exports to UTF-8 CSV
Handles null bytes, tab separators, and proper encoding
"""

import csv
import sys
from pathlib import Path


def convert_utf16_to_utf8(input_path: str, output_path: str):
    """Convert UTF-16 LE CSV to UTF-8 CSV"""
    
    input_file = Path(input_path)
    output_file = Path(output_path)
    
    # Read as UTF-16
    with open(input_file, 'r', encoding='utf-16') as f:
        reader = csv.reader(f, delimiter='\t')  # Tab-separated
        rows = list(reader)
    
    # Write as UTF-8 (with proper quoting)
    with open(output_file, 'w', encoding='utf-8', newline='') as f:
        writer = csv.writer(f, quoting=csv.QUOTE_MINIMAL)
        writer.writerows(rows)
    
    print(f"✓ Converted {len(rows)} rows")
    print(f"  Input: {input_path}")
    print(f"  Output: {output_path}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python convert_utf16.py <input.csv> <output.csv>")
        sys.exit(1)
    
    convert_utf16_to_utf8(sys.argv[1], sys.argv[2])
