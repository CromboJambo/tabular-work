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

    println!("Running pipeline: {}", pipeline.name);
    println!("Steps: {}", pipeline.steps.len());
    // TODO: Execute and show results
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