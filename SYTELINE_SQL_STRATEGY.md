# Infor Syteline Direct SQL Access Strategy

## Problem
The Excel/CSV exports from Infor Syteline's ODI layer are corrupted with:
- Embedded newlines in text fields (breaks CSV parsing)
- Extra quotes and semicolons as data characters
- 42K physical lines vs 28K logical rows
- Headers treated as data on re-load

## Root Cause
Syteline's Mongoose IDO layer adds formatting quirks when exporting to Excel. The underlying SQL Server database has clean data.

## Solution: Direct SQL Access

### Prerequisites Needed
1. **SQL Server connection string** from your IT team:
   - Server name (e.g., `SQL01\SYTELINE` or IP)
   - Database name (e.g., `M3YOURCOMPANY`)
   - ODBC driver version (usually "ODBC Driver 17 for SQL Server")

2. **Table mapping** based on your Excel sheets:
   - `MCT SharePoint link` → Likely `MCCTM`, `MCORD`, or custom view
   - `Serial Number database` → Likely `SERIAL`, `MACHINE`, or `ASSET` tables

### Implementation Plan

#### Option A: Python + pyodbc (Fastest to prototype)
```python
import pyodbc

conn = pyodbc.connect(conn_str)
cursor = conn.cursor()

# Query warranty data directly
cursor.execute("""
    SELECT 
        ORDER_NUM,
        WARRANTY_CLAIM_ID,
        CUSTOMER_CODE,
        MACHINE_MODEL,
        STATUS,
        CLAIM_DATE,
        DESCRIPTION
    FROM MCWARR  # or MCORD/MCCTM
    WHERE STATUS = 'Closed'
""")

for row in cursor.fetchall():
    print(row)
```

#### Option B: Rust + tokio-postgres (Production-ready)
Integrate into `warranty-pipeline`:
- Add `tokio-postgres` dependency
- Create `load_sql()` function alongside `load_excel()`
- Same pipeline structure, cleaner source

### Next Steps
1. **Get DB credentials** from your IT/ERP team
2. **Identify correct table names** - they likely have a data dictionary
3. **Test simple query** to verify connection and schema
4. **Migrate pipeline** to use SQL source instead of Excel

### Questions for Your Team
- What's the Syteline SQL Server connection string?
- Do you have direct read access to the DB?
- Is there a pre-built data warehouse/replica for reporting?
- What are the actual table names for warranty/order data?
