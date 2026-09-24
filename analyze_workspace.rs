use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn tokenize_file(path: &str) -> Option<String> {
    let output = std::process::Command::new("./target/release/struct-tok")
        .args(["tokenize", path])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

fn count_tokens(output: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for line in output.lines() {
        if let Some(pos) = line.find(':') {
            let kind = &line[..pos];
            *counts.entry(kind.to_string()).or_insert(0) += 1;
        }
    }
    counts
}

fn main() {
    let crates = [
        "rsf-core",
        "nustage",
        "zed-sheet-lsp",
        "git-sheets",
        "warranty-pipeline",
        "sro-pivot",
    ];

    println!("=== Tabular-Work Workspace Analysis ===\n");

    for crate_name in &crates {
        let lib_path = Path::new(crate_name).join("src/lib.rs");
        let main_path = Path::new(crate_name).join("src/main.rs");

        let target = if lib_path.exists() {
            lib_path
        } else {
            main_path
        };

        println!("--- {} ---", crate_name);

        // Check for manifest deps
        let cargo_path = Path::new(crate_name).join("Cargo.toml");
        if let Ok(cargo) = fs::read_to_string(&cargo_path) {
            let has_rsf = cargo.contains("rsf") || cargo.contains("rsf-core");
            let has_nustage = cargo.contains("nustage");
            let has_calamine = cargo.contains("calamine");

            println!(
                "  depends_on: rsf={}, nustage={}, calamine={}",
                has_rsf, has_nustage, has_calamine
            );
        }

        // Tokenize entry point
        if let Some(tokens) = tokenize_file(target.to_str().unwrap()) {
            let counts = count_tokens(&tokens);
            let total: usize = counts.values().sum();

            println!("  structural tokens: {}", total);
            for (kind, count) in &counts {
                if *count > 0 {
                    println!("    {}: {}", kind, count);
                }
            }
        } else {
            println!("  (tokenization failed)");
        }
        println!();
    }
}