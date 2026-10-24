//! Nustage CLI — data transformation pipeline tool.
//!
//! Usage: nustage <command> [options]
//!
//! Commands:
//!   import <file.csv>          Import CSV, infer types, show schema
//!   run <pipeline.toml>        Execute pipeline on input data
//!   preview <pipeline.toml>    Show intermediate step results
//!   validate <pipeline.toml>   Check pipeline against input schema

use std::path::Path;

fn table_to_typed(table: &rsf::Table) -> rsf::TypedTable {
    let headers: Vec<String> = table.headers().iter().map(|h| h.value.clone()).collect();
    let rows: Vec<Vec<String>> = table.rows().iter().map(|row| {
        row.iter().map(|f| f.value.clone()).collect()
    }).collect();

    // Use Unknown type hint for all columns (simplest conversion)
    let profiles: Vec<rsf::ColumnProfile> = headers.iter().enumerate().map(|(_, _)| {
        rsf::ColumnProfile {
            total_rows: table.row_count(),
            null_count: 0,
            cardinality: 0,
            is_constant: false,
            uniqueness_ratio: 1.0,
            type_hint: rsf::TypeHint::Unknown,
        }
    }).collect();

    rsf::TypedTable::from_untyped(&headers, &rows, &profiles)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage();
        return;
    }

    let command = &args[1];
    let target = &args[2];

    match command.as_str() {
        "import" => cmd_import(target),
        "run" => cmd_run(target),
        "preview" => cmd_preview(target),
        "validate" => cmd_validate(target),
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
        }
    }
}

fn print_usage() {
    println!("nustage — data transformation pipeline tool");
    println!();
    println!("Usage: nustage <command> [options]");
    println!();
    println!("Commands:");
    println!("  import <file.csv>          Import CSV, infer types, show schema");
    println!("  run <pipeline.toml>        Execute pipeline on input data");
    println!("  preview <pipeline.toml>    Show intermediate step results");
    println!("  validate <pipeline.toml>   Check pipeline against input schema");
}

fn cmd_import(path: &str) {
    let table = match rsf::load_csv(&std::path::PathBuf::from(path)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to import {path}: {e}");
            return;
        }
    };

    println!("Imported: {path}");
    println!("Rows: {}", table.row_count());
    println!("Columns: {}", table.column_count());
    println!();
    println!("Headers:");
    for (i, header) in table.headers().iter().enumerate() {
        println!("  {}: {}", i + 1, header.value);
    }

    // Show first few rows as preview
    if table.row_count() > 0 {
        println!();
        println!("Preview (first {} rows):", std::cmp::min(3, table.row_count()));
        for row in table.rows().iter().take(3) {
            let cells: Vec<String> = row.iter().map(|f| f.value.clone()).collect();
            println!("  {}", cells.join(" | "));
        }
    }
}

fn cmd_run(path: &str) {
    let pipeline = match nustage::pipeline::Pipeline::load(Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load pipeline {path}: {e}");
            return;
        }
    };

    // Determine input file: from pipeline config or CLI arg
    let input_file = match &pipeline.input {
        Some(f) if !f.is_empty() => f.clone(),
        _ => {
            eprintln!("No input specified in pipeline and no --input argument");
            return;
        }
    };

    // Load the input CSV and convert to typed table
    let raw_table = match rsf::load_csv(&std::path::PathBuf::from(&input_file)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to load input {input_file}: {e}");
            return;
        }
    };
    let table = table_to_typed(&raw_table);

    println!("Running pipeline: {}", pipeline.name);
    println!("Input: {} ({} rows)", input_file, table.row_count());
    println!("Steps: {}", pipeline.steps.len());
    println!();

    // Execute the pipeline
    match nustage::executor::run_pipeline(&table, &pipeline.steps) {
        Ok((result, ledger)) => {
            println!("Pipeline completed successfully");
            println!("Final result: {} rows", result.row_count());
            println!();

            // Show step-by-step results
            for (i, step_result) in ledger.iter().enumerate() {
                println!("  Step {}: {} -> {} rows", i + 1, step_result.rows_in, step_result.rows_out);
            }
            println!();

            // Write output if specified
            if let Some(output_file) = &pipeline.output {
                if !output_file.is_empty() {
                    let csv_content = result.to_csv();
                    match std::fs::write(Path::new(output_file), csv_content) {
                        Ok(_) => println!("Output written to: {output_file}"),
                        Err(e) => eprintln!("Failed to write output {output_file}: {e}"),
                    }
                }
            }

            // Show preview of results
            if result.row_count() > 0 {
                println!();
                println!("Result preview (first {} rows):", std::cmp::min(3, result.row_count()));
                let headers: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
                println!("| {} |", headers.join(" | "));
                for row in result.rows.iter().take(3) {
                    let cells: Vec<String> = row.iter().map(|v| v.as_str()).collect();
                    println!("| {} |", cells.join(" | "));
                }
            }
        }
        Err((step_idx, error)) => {
            eprintln!("Pipeline failed at step {}: {}", step_idx + 1, error);
        }
    }
}

fn cmd_preview(path: &str) {
    let pipeline = match nustage::pipeline::Pipeline::load(Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load pipeline {path}: {e}");
            return;
        }
    };

    println!("Previewing pipeline: {}", pipeline.name);
    for (i, step) in pipeline.steps.iter().enumerate() {
        println!("  Step {}: {:?}", i + 1, step);
    }
}

fn cmd_validate(path: &str) {
    let pipeline = match nustage::pipeline::Pipeline::load(Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load pipeline {path}: {e}");
            return;
        }
    };

    println!("Validating pipeline: {}", pipeline.name);
    println!("Steps: {}", pipeline.steps.len());
    // TODO: Validate against input schema
}