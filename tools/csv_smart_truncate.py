#!/usr/bin/env python3
"""Smart CSV truncation for terminal viewing"""

import csv
import sys


def smart_truncate_csv(input_path: str, output_path: str, max_total_width: int = 120):
    """Truncate CSV with smart column width distribution"""
    
    with open(input_path, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)
        rows = list(reader)
    
    if not rows:
        return
    
    num_cols = max(len(row) for row in rows)
    
    # Calculate content width for each column
    col_content_widths = [0] * num_cols
    for row in rows:
        for i, cell in enumerate(row):
            if i < num_cols:
                col_content_widths[i] = max(col_content_widths[i], len(cell))
    
    # Distribute width proportionally
    total_content = sum(col_content_widths) or 1
    remaining = max_total_width - num_cols
    
    col_max_widths = []
    for content_width in col_content_widths:
        proportional = int((content_width / total_content) * remaining)
        col_max_widths.append(max(8, min(proportional, content_width + 3)))
    
    # Format rows
    formatted_lines = []
    for row in rows:
        formatted_cells = []
        for i in range(num_cols):
            cell = row[i] if i < len(row) else ''
            target = col_max_widths[i]
            if len(cell) > target - 3:
                cell = cell[:target - 3] + '...'
            else:
                cell = cell.ljust(target)
            formatted_cells.append(cell)
        formatted_lines.append(','.join(formatted_cells))
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(formatted_lines) + '\n')
    
    print(f"✓ Truncated {len(rows)} rows")
    print(f"  Max width: {max_total_width} chars")


if __name__ == "__main__":
    max_width = int(sys.argv[3]) if len(sys.argv) > 3 else 120
    smart_truncate_csv(sys.argv[1], sys.argv[2], max_width)
