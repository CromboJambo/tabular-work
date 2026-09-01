#!/usr/bin/env python3
"""Select key columns from CSV for better viewing"""

import csv
import sys


def select_key_columns(input_path: str, output_path: str):
    """Extract only the most important columns"""
    
    # Key columns to keep (by index based on your data)
    # Adjust these indices based on what matters most to you
    key_indices = [0, 3, 4, 7, 10, 11]  # Trans Date, Customer, SRO, Description, Item, comments
    
    with open(input_path, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)
        rows = list(reader)
    
    formatted_lines = []
    for row in rows:
        selected = [row[i] if i < len(row) else '' for i in key_indices]
        formatted_lines.append('\t'.join(selected))
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(formatted_lines) + '\n')
    
    print(f"✓ Selected {len(key_indices)} key columns from {len(rows)} rows")


if __name__ == "__main__":
    select_key_columns(sys.argv[1], sys.argv[2])
