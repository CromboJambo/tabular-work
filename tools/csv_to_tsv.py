#!/usr/bin/env python3
"""Convert CSV to TSV with fixed column widths for terminal viewing"""

import csv
import sys


def csv_to_fixed_tsv(input_path: str, output_path: str, max_width: int = 20):
    """Convert CSV to TSV with truncated columns"""
    
    with open(input_path, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)
        rows = list(reader)
    
    num_cols = max(len(row) for row in rows)
    
    formatted_lines = []
    for row in rows:
        cells = []
        for i in range(num_cols):
            cell = row[i] if i < len(row) else ''
            # Truncate to max_width
            if len(cell) > max_width - 3:
                cell = cell[:max_width - 3] + '...'
            cells.append(cell)
        formatted_lines.append('\t'.join(cells))
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(formatted_lines) + '\n')
    
    print(f"✓ Converted {len(rows)} rows to TSV")
    print(f"  Max column width: {max_width} chars")


if __name__ == "__main__":
    max_width = int(sys.argv[3]) if len(sys.argv) > 3 else 20
    csv_to_fixed_tsv(sys.argv[1], sys.argv[2], max_width)
