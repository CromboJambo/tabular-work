"""
Infor Syteline MCT Export Cleaner
Fixes embedded newlines and semicolons from Excel exports
"""

import csv
import re
from pathlib import Path


def clean_csv_row(row: dict) -> dict:
    """Clean a single row by normalizing problematic fields"""
    cleaned = {}
    
    for col, val in row.items():
        if not val or val.strip() == '':
            cleaned[col] = ''
            continue
        
        # Normalize whitespace
        val = val.strip()
        
        # Handle embedded newlines: replace with space (or keep as-is if needed)
        val = re.sub(r'[\r\n]+', ' ', val)
        
        # Handle semicolons in concatenated fields: replace with pipe for easier parsing
        if col in ['Customer Name', 'Order Number', 'Machine Model']:
            val = val.replace(';', '|')
        
        cleaned[col] = val
    
    return cleaned


def clean_excel_export(input_path: str, output_path: str):
    """Clean the Excel export CSV"""
    input_path = Path(input_path)
    output_path = Path(output_path)
    
    # Read all rows
    with open(input_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        fieldnames = reader.fieldnames
        rows = list(reader)
    
    print(f"Loaded {len(rows)} rows from {input_path}")
    
    # Clean each row
    cleaned_rows = [clean_csv_row(row) for row in rows]
    
    # Write cleaned CSV
    with open(output_path, 'w', encoding='utf-8', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(cleaned_rows)
    
    print(f"Cleaned {len(cleaned_rows)} rows → {output_path}")
    
    # Report statistics
    newline_counts = {}
    semicolon_counts = {}
    
    for row in cleaned_rows:
        for col, val in row.items():
            if '\n' in val:
                newline_counts[col] = newline_counts.get(col, 0) + 1
            if ';' in val:
                semicolon_counts[col] = semicolon_counts.get(col, 0) + 1
    
    print(f"\nRemaining issues:")
    for col, count in sorted(newline_counts.items(), key=lambda x: -x[1])[:5]:
        print(f"  Newlines in {col}: {count}")
    for col, count in sorted(semicolon_counts.items(), key=lambda x: -x[1])[:5]:
        print(f"  Semicolons in {col}: {count}")


if __name__ == '__main__':
    import sys
    
    if len(sys.argv) < 3:
        print("Usage: python clean_syteline_export.py <input.csv> <output.csv>")
        sys.exit(1)
    
    clean_excel_export(sys.argv[1], sys.argv[2])
