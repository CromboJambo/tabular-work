//! CLI for warranty pipeline

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use warranty_pipeline::{config::*, run_pipeline};

#[derive(Parser)]
#[command(name = "warranty-pipeline")]
#[command(about = "Power Query alternative for warranty workbook transformations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a pipeline configuration file
    Run {
        /// Path to pipeline config file (YAML or TOML)
        config: PathBuf,
    },

    /// Preview data from a sheet or CSV
    Preview {
        /// Source Excel/CSV file
        source: PathBuf,

        /// Sheet name (for Excel)
        #[arg(short, long)]
        sheet: Option<String>,

        /// Number of rows to preview
        #[arg(short, long, default_value = "10")]
        rows: usize,
    },

    /// Create a version control snapshot
    Snapshot {
        /// Data file to snapshot
        data_file: PathBuf,

        /// Snapshot message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Run demo pipeline on warranty workbook
    Demo,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { config } => {
            run_config_file(&config)?;
        }

        Commands::Preview {
            source,
            sheet,
            rows,
        } => {
            preview_data(&source, sheet.as_deref(), rows)?;
        }

        Commands::Snapshot { data_file, message } => {
            create_snapshot(&data_file, message.as_deref())?;
        }

        Commands::Demo => {
            run_demo()?;
        }
    }

    Ok(())
}

fn run_config_file(config_path: &PathBuf) -> Result<()> {
    let config_content = std::fs::read_to_string(config_path)?;

    // Try to parse as TOML first, then YAML
    let config: PipelineConfig = if config_path.extension().map(|e| e == "toml").unwrap_or(true) {
        toml::from_str(&config_content)?
    } else {
        serde_yaml::from_str(&config_content)?
    };

    run_pipeline(&config)?;
    Ok(())
}

fn preview_data(source: &PathBuf, sheet: Option<&str>, rows: usize) -> Result<()> {
    use warranty_pipeline::io::{load_csv, load_excel_sheet};

    println!("Previewing {:?}...", source);

    if let Some(sheet_name) = sheet {
        println!("Sheet: {}", sheet_name);
        let data = load_excel_sheet(source.to_str().unwrap(), sheet_name)?;

        for (idx, row) in data.iter().take(rows).enumerate() {
            println!("Row {}: {:?}", idx, row);
        }
    } else {
        // Assume CSV
        let data = load_csv(source.to_str().unwrap())?;
        for (idx, row) in data.iter().take(rows).enumerate() {
            println!("Row {}: {:?}", idx, row);
        }
    }

    Ok(())
}

fn create_snapshot(data_file: &PathBuf, message: Option<&str>) -> Result<()> {
    println!("Creating snapshot of {:?}...", data_file);

    if let Some(msg) = message {
        println!("Message: {}", msg);
    }

    // TODO: Integrate with git-sheets for version control
    // For now, just copy to OUTPUT/snapshots/
    let snapshots_dir = "OUTPUT/snapshots";
    std::fs::create_dir_all(snapshots_dir)?;

    let filename = data_file
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("data"))
        .to_str()
        .unwrap();

    let snapshot_path = format!(
        "{}/{}_{}.csv",
        snapshots_dir,
        filename,
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    std::fs::copy(data_file, &snapshot_path)?;

    println!("Snapshot saved to: {}", snapshot_path);
    Ok(())
}

fn run_demo() -> Result<()> {
    use warranty_pipeline::io::{load_excel_sheet, save_csv};

    println!("=== Warranty Pipeline Demo ===\n");

    // Step 1: Load raw data from MCT SharePoint link sheet
    println!("Step 1: Loading MCT SharePoint data...");
    let mct_data = load_excel_sheet("DATA/WarrantyWorkbook.xlsx", "MCT SharePoint link")?;

    println!("  Loaded {} rows", mct_data.len());
    if !mct_data.is_empty() {
        println!("  Headers: {:?}", mct_data[0]);
    }

    // Step 2: Save as CSV for further processing
    let output_path = "OUTPUT/mct_raw.csv";
    std::fs::create_dir_all("OUTPUT")?;
    save_csv(&mct_data, output_path)?;
    println!("  Saved to {}", output_path);

    // Step 3: Filter by status (simplified - just show first few rows)
    println!("\nStep 2: Filtering closed items...");
    let filtered: Vec<Vec<String>> = mct_data
        .iter()
        .skip(1) // Skip headers
        .filter(|row| row.get(5).map(|s| s.to_lowercase()).unwrap_or_default() == "closed")
        .cloned()
        .collect();

    println!("  Found {} closed items", filtered.len());

    // Step 4: Group by STATUS and count (mock pivot)
    println!("\nStep 3: Creating summary by status...");
    let mut status_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for row in &filtered {
        if let Some(status) = row.get(5) {
            *status_counts.entry(status.to_string()).or_default() += 1;
        }
    }

    for (status, count) in &status_counts {
        println!("  {} : {}", status, count);
    }

    // Step 5: Create summary CSV
    let summary_path = "OUTPUT/status_summary.csv";
    let mut summary_rows: Vec<Vec<String>> = vec![vec!["STATUS".to_string(), "COUNT".to_string()]];

    for (status, count) in &status_counts {
        summary_rows.push(vec![status.clone(), count.to_string()]);
    }

    save_csv(&summary_rows, summary_path)?;
    println!("\n  Summary saved to {}", summary_path);

    println!("\n=== Demo Complete ===");
    println!("Output files:");
    println!("  - OUTPUT/mct_raw.csv (full data)");
    println!("  - OUTPUT/status_summary.csv (aggregated by status)");

    Ok(())
}
