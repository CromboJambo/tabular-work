// git-sheets publish: Package a nustage pipeline for GitHub publication
// No real data — only transformation logic, schema, and synthetic examples.

use crate::core::{GitSheetsError, Result};
use std::fs;
use std::path::Path;

/// Publish options from CLI args
pub struct PublishOptions {
    pub title: String,
    pub description: Option<String>,
    pub repo_name: Option<String>,
    pub dry_run: bool,
    pub output_dir: Option<String>,
}

/// Build a publishable workflow package locally.
/// Returns the path to the built package directory.
pub fn build_publish_package(
    pipeline_path: &Path,
    opts: &PublishOptions,
) -> Result<std::path::PathBuf> {
    // 1. Load pipeline
    let pipeline = nustage::pipeline::Pipeline::load(pipeline_path)
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to load pipeline: {}", e)))?;

    if pipeline.steps.is_empty() {
        println!("Warning: Pipeline has no steps");
    }

    // 2. Derive repo name
    let repo_name = opts
        .repo_name
        .clone()
        .unwrap_or_else(|| title_to_slug(&opts.title));

    // 3. Create output directory
    let out_dir = match &opts.output_dir {
        Some(dir) => Path::new(dir).to_path_buf(),
        None => std::env::temp_dir().join(format!("publish-{}", repo_name)),
    };

    if opts.dry_run {
        println!("Dry run — building locally only (not pushing)");
    } else {
        // Clean previous build
        if out_dir.exists() {
            fs::remove_dir_all(&out_dir).map_err(|e| {
                GitSheetsError::FileSystemError(format!("Failed to clean output dir: {}", e))
            })?;
        }
    }
    fs::create_dir_all(&out_dir).map_err(|e| {
        GitSheetsError::FileSystemError(format!("Failed to create output dir: {}", e))
    })?;

    // 4. Copy pipeline as workflow.nustage.toml
    let workflow_path = out_dir.join("workflow.nustage.toml");
    fs::copy(pipeline_path, &workflow_path)
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to copy pipeline: {}", e)))?;

    // 5. Generate schema files (input/output)
    let schema_dir = out_dir.join("schema");
    fs::create_dir_all(&schema_dir).map_err(|e| {
        GitSheetsError::FileSystemError(format!("Failed to create schema dir: {}", e))
    })?;

    // Infer schemas from pipeline steps (simplified: just note the columns referenced)
    let input_cols = infer_input_columns(&pipeline);
    let output_cols = infer_output_columns(&pipeline);

    fs::write(
        schema_dir.join("input.rsf"),
        format!(
            "# Input schema\n# Inferred from pipeline steps\ncolumns:\n{}",
            input_cols
        ),
    )
    .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to write input schema: {}", e)))?;

    fs::write(
        schema_dir.join("output.rsf"),
        format!(
            "# Output schema\n# Inferred from pipeline steps\ncolumns:\n{}",
            output_cols
        ),
    )
    .map_err(|e| {
        GitSheetsError::FileSystemError(format!("Failed to write output schema: {}", e))
    })?;

    // 6. Generate synthetic example data
    let examples_dir = out_dir.join("examples");
    fs::create_dir_all(&examples_dir).map_err(|e| {
        GitSheetsError::FileSystemError(format!("Failed to create examples dir: {}", e))
    })?;

    generate_synthetic_data(&input_cols, &examples_dir.join("synthetic.csv")).map_err(|e| {
        GitSheetsError::FileSystemError(format!("Failed to generate synthetic data: {}", e))
    })?;

    // 7. Generate README
    let readme = generate_readme(&opts.title, &pipeline, &repo_name);
    fs::write(out_dir.join("README.md"), readme)
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to write README: {}", e)))?;

    // 8. Generate .gitignore
    fs::write(out_dir.join(".gitignore"), "snapshots/\ndiffs/\n*.local\n").map_err(|e| {
        GitSheetsError::FileSystemError(format!("Failed to write .gitignore: {}", e))
    })?;

    println!("✓ Built publish package at: {}", out_dir.display());
    Ok(out_dir)
}

/// Convert title to kebab-case slug for repo name
pub fn title_to_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Infer input column names from pipeline steps (very simplified)
fn infer_input_columns(pipeline: &nustage::pipeline::Pipeline) -> String {
    // Walk steps looking for column references
    let mut cols = std::collections::HashSet::new();
    for step in &pipeline.steps {
        match step {
            nustage::pipeline::Step::FilterRows { condition } => {
                // Parse column names from condition (simplified)
                if let Some(col) = extract_column_from_expr(condition) {
                    cols.insert(col);
                }
            }
            nustage::pipeline::Step::AddColumn { name, expr } => {
                if let Some(col) = extract_column_from_expr(expr) {
                    cols.insert(col);
                }
            }
            nustage::pipeline::Step::GroupBy { group_cols, .. } => {
                for col in group_cols {
                    cols.insert(col.clone());
                }
            }
            nustage::pipeline::Step::SortBy { column, .. } => {
                cols.insert(column.clone());
            }
            _ => {}
        }
    }

    if cols.is_empty() {
        "  - (unknown — inspect workflow.nustage.toml)".to_string()
    } else {
        let mut sorted_cols: Vec<_> = cols.iter().collect();
        sorted_cols.sort();
        let mut s = String::new();
        for col in sorted_cols {
            s.push_str(&format!("  - {}\n", col));
        }
        s
    }
}

/// Infer output column names (simplified — same as input for now)
fn infer_output_columns(pipeline: &nustage::pipeline::Pipeline) -> String {
    // For now, just note that this is a placeholder
    format!(
        "  - (same as input + any added columns from {} steps)",
        pipeline.steps.len()
    )
}

/// Very simplified column name extraction from expressions
fn extract_column_from_expr(expr: &str) -> Option<String> {
    // Look for @ColumnName pattern used in nustage expressions
    if let Some(pos) = expr.find('@') {
        let rest = &expr[pos + 1..];
        let end = rest
            .chars()
            .position(|c| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        if end > 0 {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Generate synthetic CSV data matching the inferred input schema
fn generate_synthetic_data(columns: &str, path: &Path) -> std::io::Result<()> {
    // Parse column names from the inferred columns string
    let col_names: Vec<String> = columns
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("- ") {
                Some(line[2..].to_string())
            } else {
                None
            }
        })
        .collect();

    let mut csv = String::new();

    // Header
    if col_names.is_empty() {
        csv.push_str("column1,column2,column3\n");
    } else {
        csv.push_str(&col_names.join(","));
        csv.push('\n');
    }

    // Generate 5 synthetic rows
    for row_num in 0..5 {
        let mut row = Vec::new();
        if col_names.is_empty() {
            row.push(format!("item{}", row_num));
            row.push((row_num * 10).to_string());
            row.push((row_num as f64 * 9.99).to_string());
        } else {
            for col in &col_names {
                // Generate plausible fake data based on column name hints
                if col.contains("date") || col.contains("Date") {
                    row.push(format!("2024-01-{:02}", row_num + 1));
                } else if col.contains("amount")
                    || col.contains("revenue")
                    || col.contains("cost")
                {
                    row.push(((row_num as f64 + 1.0) * 100.0).to_string());
                } else if col.contains("count") || col.contains("units") {
                    row.push((row_num + 1).to_string());
                } else {
                    row.push(format!("Item{}", row_num));
                }
            }
        }
        csv.push_str(&row.join(","));
        csv.push('\n');
    }

    fs::write(path, csv)?;
    Ok(())
}

/// Generate README.md for the published workflow
fn generate_readme(title: &str, pipeline: &nustage::pipeline::Pipeline, repo_name: &str) -> String {
    let mut readme = format!("# {}\n\n", title);

    readme.push_str("This is a data transformation workflow published from [git-sheets](https://github.com/CromboJambo/git-sheets).\n\n");

    readme.push_str("## What this workflow does\n\n");
    readme.push_str(&describe_pipeline(pipeline));
    readme.push_str("\n");

    readme.push_str("## Run it\n\n");
    readme.push_str("```bash\n");
    readme.push_str(&format!(
        "git clone https://github.com/<user>/{}.git\n",
        repo_name
    ));
    readme.push_str(&format!("cd {}\n", repo_name));
    readme.push_str("nustage process your-input-data.csv\n");
    readme.push_str("```\n\n");

    readme.push_str("## Schema\n\n");
    readme.push_str("- Input: See `schema/input.rsf`\n");
    readme.push_str("- Output: See `schema/output.rsf`\n\n");

    readme.push_str("Example data is provided in `examples/synthetic.csv`.\n\n");

    readme.push_str("---\n\n");
    readme.push_str("Published from [git-sheets](https://github.com/CromboJambo/git-sheets) — version control for spreadsheets.\n");

    readme
}

/// Generate human-readable description of pipeline steps
fn describe_pipeline(pipeline: &nustage::pipeline::Pipeline) -> String {
    if pipeline.steps.is_empty() {
        return "This workflow has no transformation steps.".to_string();
    }

    let mut desc = String::new();
    for (i, step) in pipeline.steps.iter().enumerate() {
        match step {
            nustage::pipeline::Step::FilterRows { condition } => {
                desc.push_str(&format!("{}. Filter rows where: {}\n", i + 1, condition));
            }
            nustage::pipeline::Step::AddColumn { name, expr } => {
                desc.push_str(&format!("{}. Add column '{}' = {}\n", i + 1, name, expr));
            }
            nustage::pipeline::Step::GroupBy { group_cols, value_col, agg } => {
                desc.push_str(&format!(
                    "{}. Group by {} and aggregate {} with {}\n",
                    i + 1,
                    group_cols.join(", "),
                    value_col,
                    agg
                ));
            }
            nustage::pipeline::Step::RenameColumn { from, to } => {
                desc.push_str(&format!("{}. Rename column '{}' → '{}'\n", i + 1, from, to));
            }
            nustage::pipeline::Step::SelectColumns { columns } => {
                desc.push_str(&format!(
                    "{}. Select columns: {}\n",
                    i + 1,
                    columns.join(", ")
                ));
            }
            nustage::pipeline::Step::DropColumns { columns } => {
                desc.push_str(&format!(
                    "{}. Drop columns: {}\n",
                    i + 1,
                    columns.join(", ")
                ));
            }
            nustage::pipeline::Step::SortBy { column, desc: descending } => {
                let order = if *descending { "descending" } else { "ascending" };
                desc.push_str(&format!("{}. Sort by {} ({})\n", i + 1, column, order));
            }
            nustage::pipeline::Step::RemoveDuplicates => {
                desc.push_str(&format!("{}. Remove duplicate rows\n", i + 1));
            }
        }
    }
    desc
}
