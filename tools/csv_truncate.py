#!/usr/bin/env python3
"""
Truncate CSV columns to fixed width for better Zed viewing
Prevents line wrapping by truncating long cells
"""

import csv
import sys


def truncate_csv_columns(input_path: str, output_path: str, max_width: int = 80):
    """Format CSV with fixed column widths (truncates long cells)"""
    
    # Read all rows
    with open(input_path, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)
        rows = list(reader)
    
    if not rows:
        print("Empty file")
        return
    
    num_cols = max(len(row) for row in rows)
    
    # Format all rows with fixed width truncation
    formatted_lines = []
    for row in rows:
        formatted_cells = []
        for i in range(num_cols):
            cell = row[i] if i < len(row) else ''
            
            # Truncate to max_width (leave room for "..." indicator)
            if len(cell) > max_width - 3:
                cell = cell[:max_width - 3] + '...'
            else:
                # Pad to max_width for alignment
                cell = cell.ljust(max_width)
            
            formatted_cells.append(cell)
        
        # Join with comma (CSV format preserved)
        formatted_lines.append(','.join(formatted_cells))
    
    # Write output
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(formatted_lines) + '\n')
    
    print(f"✓ Truncated {len(rows)} rows")
    print(f"  Max column width: {max_width} chars")


if __name__ == "__main__":
    # Default max_width = 80 (fits most terminal widths)
    max_width = int(sys.argv[3]) if len(sys.argv) > 3 else 80
    
    truncate_csv_columns(sys.argv[1], sys.argv[2], max_width)
