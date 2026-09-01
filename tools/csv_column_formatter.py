#!/usr/bin/env python3
"""
Zed CSV Column Formatter
Aligns all rows to match the first row's column widths
Preserves rainbow CSV coloring (just adds spaces)
"""

import csv
import sys


def format_csv_columns(input_path: str, output_path: str):
    """Format CSV to align columns based on first row"""
    
    # Read all rows
    with open(input_path, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)
        rows = list(reader)
    
    if not rows:
        print("Empty file")
        return
    
    # Calculate max width for each column across ALL rows
    num_cols = max(len(row) for row in rows)
    col_widths = [0] * num_cols
    
    for row in rows:
        for i, cell in enumerate(row):
            if i < num_cols:
                col_widths[i] = max(col_widths[i], len(cell))
    
    # Format all rows with padding
    formatted_lines = []
    for row in rows:
        formatted_cells = []
        for i in range(num_cols):
            cell = row[i] if i < len(row) else ''
            # Pad to column width
            formatted_cells.append(cell.ljust(col_widths[i]))
        
        # Join with comma (preserving CSV format)
        formatted_lines.append(','.join(formatted_cells))
    
    # Write output
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(formatted_lines) + '\n')
    
    print(f"✓ Formatted {len(rows)} rows")
    print(f"  Columns: {num_cols}")
    print(f"  Max widths: {[str(w) for w in col_widths]}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python csv_column_formatter.py <input.csv> <output.csv>")
        sys.exit(1)
    
    format_csv_columns(sys.argv[1], sys.argv[2])
