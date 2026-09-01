#!/usr/bin/env python3
"""Select and truncate key columns for terminal viewing"""

import csv
import sys


def select_and_truncate(input_path: str, output_path: str, max_width: int = 20):
    """Extract key columns with truncation"""
    
    # Key columns to keep
    key_indices = [0, 3, 4, 7, 10, 11]  # Trans Date, Customer, SRO, Description, Item, comments
    
    with open(input_path, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)
        rows = list(reader)
    
    formatted_lines = []
    for row in rows:
        selected = []
        for i in key_indices:
            cell = row[i] if i < len(row) else ''
            # Truncate long cells
            if len(cell) > max_width - 3:
                cell = cell[:max_width - 3] + '...'
            selected.append(cell)
        formatted_lines.append('\t'.join(selected))
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(formatted_lines) + '\n')
    
    print(f"✓ Selected {len(key_indices)} key columns from {len(rows)} rows")
    print(f"  Max column width: {max_width} chars")


if __name__ == "__main__":
    max_width = int(sys.argv[3]) if len(sys.argv) > 3 else 20
    select_and_truncate(sys.argv[1], sys.argv[2], max_width)
