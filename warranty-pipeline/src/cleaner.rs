//! Cleaner module: preprocesses Syteline Excel exports to fix data quality issues

use anyhow::Result;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Clean Syteline Excel export CSV before loading
/// Removes embedded newlines and normalizes semicolons
pub fn clean_syteline_csv(input_path: &str, output_path: &str) -> Result<()> {
    let input = Path::new(input_path);
    let output = Path::new(output_path);

    if !input.exists() {
        return Err(anyhow::anyhow!("Input file not found: {}", input_path));
    }

    // Read all rows
    let reader = File::open(input)?;
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(BufReader::new(reader));

    let headers: Vec<String> = csv_reader
        .headers()?
        .iter()
        .map(|s| s.to_string())
        .collect();
    
    // Collect all rows into a mutable vector of vectors (not StringRecord)
    let mut rows: Vec<Vec<String>> = Vec::new();
    for record in csv_reader.records() {
        if let Ok(rec) = record {
            let row: Vec<String> = rec.iter().map(|s| s.to_string()).collect();
            rows.push(row);
        }
    }

    eprintln!("Cleaned {} rows", rows.len());

    // Clean each row
    for row in &mut rows {
        for (idx, val) in row.iter_mut().enumerate() {
            let col = headers[idx].as_str();

            // Normalize whitespace
            *val = val.trim().to_string();

            // Remove embedded newlines
            *val = val.replace('\n', " ").replace('\r', "");

            // Replace semicolons with pipes in concatenated fields
            if col == "Customer Name" || col == "Order Number" || col == "Machine Model" {
                *val = val.replace(';', "|");
            }
        }
    }

    // Write cleaned CSV
    let output_file = File::create(output)?;
    let mut csv_writer = WriterBuilder::new().from_writer(output_file);

    // Write headers
    csv_writer.write_record(&headers)?;

    // Write data rows
    for row in &rows {
        csv_writer.write_record(row)?;
    }

    csv_writer.flush()?;

    eprintln!("Cleaned CSV saved to {}", output_path);
    Ok(())
}
