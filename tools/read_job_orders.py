#!/usr/bin/env python3
"""
Read Job Orders to Close Excel file and export as clean CSV
Handles the shared string mapping from Infor Syteline exports
"""

import zipfile
from xml.etree import ElementTree as ET
import csv
from pathlib import Path


def read_xlsx_with_shared_strings(xlsx_path: str, output_csv: str):
    """Read xlsx file with shared string references and export to CSV"""
    
    with zipfile.ZipFile(xlsx_path, 'r') as z:
        # Load shared strings first
        with z.open('xl/sharedStrings.xml') as f:
            ss_content = f.read().decode('utf-8')
        
        root_ss = ET.fromstring(ss_content)
        shared_strings = []
        for si in root_ss.iter('{http://schemas.openxmlformats.org/spreadsheetml/2006/main}si'):
            t = si.find('{http://schemas.openxmlformats.org/spreadsheetml/2006/main}t')
            if t is not None and t.text:
                shared_strings.append(t.text)
        
        print(f"Loaded {len(shared_strings)} shared strings")
        
        # Now read the worksheet
        with z.open('xl/worksheets/sheet1.xml') as f:
            xml_content = f.read().decode('utf-8')
        
        root = ET.fromstring(xml_content)
        
        # Extract rows
        rows = []
        headers = None
        
        for row_idx, row in enumerate(root.iter('{http://schemas.openxmlformats.org/spreadsheetml/2006/main}row')):
            cells = []
            for cell in row.iter('{http://schemas.openxmlformats.org/spreadsheetml/2006/main}c'):
                v = cell.find('{http://schemas.openxmlformats.org/spreadsheetml/2006/main}v')
                
                if v is not None and v.text:
                    val = v.text
                    # Check if it's a numeric reference to shared strings
                    if val.isdigit():
                        idx = int(val)
                        if idx < len(shared_strings):
                            cells.append(shared_strings[idx])
                        else:
                            cells.append(f"[SS:{val}]")
                    else:
                        cells.append(val)
                else:
                    cells.append('')
            
            # Skip completely empty first row (row 0 is usually metadata)
            if row_idx == 0 and all(not c.strip() for c in cells):
                continue
            
            # Second row is the title row (single cell with project name) - skip it
            if row_idx == 1 and len([c for c in cells if c.strip()]) == 1:
                continue
            
            # Third row is headers, rest is data
            if row_idx == 2:
                headers = cells
            elif any(c.strip() for c in cells):  # Skip empty rows
                rows.append(cells)
        
        # Write to CSV
        with open(output_csv, 'w', encoding='utf-8', newline='') as f:
            writer = csv.writer(f, quoting=csv.QUOTE_MINIMAL)
            writer.writerow(headers)
            writer.writerows(rows)
        
        print(f"✓ Exported {len(rows)} rows to {output_csv}")
        return len(rows)


if __name__ == "__main__":
    import sys
    
    if len(sys.argv) != 3:
        print("Usage: python read_job_orders.py <input.xlsx> <output.csv>")
        sys.exit(1)
    
    xlsx_path = sys.argv[1]
    output_csv = sys.argv[2]
    
    read_xlsx_with_shared_strings(xlsx_path, output_csv)
