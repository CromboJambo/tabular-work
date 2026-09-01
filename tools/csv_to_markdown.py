#!/usr/bin/env python3
"""Convert CSV to Markdown table for better Zed viewing"""

import csv
from pathlib import Path


def csv_to_markdown_table(csv_path: str, output_path: str):
    """Convert CSV to markdown table format"""
    
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)
        rows = list(reader)
    
    if not rows:
        print("Empty CSV")
        return
    
    headers = rows[0]
    data_rows = rows[1:]
    
    # Calculate column widths
    col_widths = [len(h) for h in headers]
    for row in data_rows:
        for i, cell in enumerate(row):
            if i < len(col_widths):
                col_widths[i] = max(col_widths[i], len(cell))
    
    # Build markdown table
    lines = []
    
    # Header row
    header_line = " | ".join(h.ljust(col_widths[i]) for i, h in enumerate(headers))
    lines.append(header_line)
    
    # Separator row
    separator_line = " | ".join("-" * w for w in col_widths)
    lines.append(separator_line)
    
    # Data rows
    for row in data_rows:
        if not all(cell.strip() == '' for cell in row):  # Skip empty rows
            formatted_row = []
            for i, cell in enumerate(row):
                if i < len(col_widths):
                    formatted_row.append(cell.ljust(col_widths[i]))
                else:
                    formatted_row.append(cell)
            lines.append(" | ".join(formatted_row))
    
    # Write to file
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(lines) + '\n')
    
    print(f"✓ Converted {len(data_rows)} rows to markdown table")
    print(f"  Output: {output_path}")


if __name__ == "__main__":
    import sys
    
    if len(sys.argv) != 3:
        print("Usage: python csv_to_markdown.py <input.csv> <output.md>")
        sys.exit(1)
    
    csv_to_markdown_table(sys.argv[1], sys.argv[2])
