//! I/O operations for reading Excel and CSV files

use anyhow::{Context, Result};
use calamine::Reader;
use std::fs::File;
use std::path::Path;

/// Load data from an Excel sheet using calamine
pub fn load_excel_sheet(excel_path: &str, sheet_name: &str) -> Result<Vec<Vec<String>>> {
    let path = Path::new(excel_path);
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", excel_path));
    }

    // Open workbook - calamine v0.26 uses Reader trait
    let file = File::open(path)?;
    let mut workbook = calamine::Xlsx::new(file).context("Failed to open Excel")?;

    // Get sheet by name (calamine v0.26 API returns Result<Range, Error>)
    let sheet = workbook
        .worksheet_range(sheet_name)
        .context(format!("Failed to get sheet '{}'", sheet_name))?;

    let mut rows = Vec::new();
    for row in sheet.rows() {
        let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        if !cells.is_empty() && cells.iter().all(|s| s.is_empty()) {
            break; // Stop at empty rows
        }
        rows.push(cells);
    }

    Ok(rows)
}

/// Load data from a CSV file (treats first row as headers, includes them in output)
pub fn load_csv(path: &str) -> Result<Vec<Vec<String>>> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    let file = File::open(path)?;

    // Configure CSV reader - has_headers(true) skips first row, so we read it separately
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // Read all rows as data (including headers)
        .flexible(true) // Allow variable number of columns per row
        .from_reader(file);

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result?;
        let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();

        // Debug: print first few rows
        if rows.len() < 3 {
            eprintln!(
                "DEBUG load_csv row {}: {:?}",
                rows.len(),
                &row[..5.min(row.len())]
            );
        }
        rows.push(row);
    }

    Ok(rows)
}

/// Save data to a CSV file (standard mode with proper quoting)
pub fn save_csv(data: &[Vec<String>], path: &str) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);

    for row in data {
        writer.write_record(row)?;
    }

    writer.flush()?;
    Ok(())
}

/// Save data to a "clean" CSV file: trim whitespace, normalize special chars, escape quotes
pub fn save_csv_clean(data: &[Vec<String>], path: &str) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);

    for (idx, row) in data.iter().enumerate() {
        if idx < 3 {
            eprintln!(
                "DEBUG save_csv_clean row {}: {:?}",
                idx,
                &row[..5.min(row.len())]
            );
        }

        // Clean each cell: trim whitespace
        let cleaned_row: Vec<String> = row.iter().map(|cell| cell.trim().to_string()).collect();
        writer.write_record(&cleaned_row)?;
    }

    writer.flush()?;
    Ok(())
}

/// Save data to JSON lines format (one object per line, better for programmatic use)
pub fn save_json_lines(headers: &[String], data: &[Vec<String>], path: &str) -> Result<()> {
    use std::io::Write;
    let file = File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);

    for row in data {
        // Build JSON object from headers and row values
        let mut json_parts: Vec<String> = Vec::with_capacity(headers.len());
        for (header, value) in headers.iter().zip(row.iter()) {
            let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
            json_parts.push(format!("\"{}\":\"{}\"", header, escaped_value));
        }
        writeln!(writer, "{{{}}}", json_parts.join(","))?;
    }

    Ok(())
}
