//! Warranty Pipeline - Power Query transformations in Rust
//! 
//! This module provides Excel/CSV reading and transformation operations
//! that replicate Power Query functionality.

pub mod config;
pub mod io;
pub mod pipeline;
pub mod cleaner;

use anyhow::Result;

/// Re-export the pipeline configuration types from config module
pub use config::{
    AggFunction, Aggregation, ColumnDef, JoinType, PipelineConfig, PipelineStep, SortSpec,
};

/// Execute the pipeline
pub fn run_pipeline(config: &PipelineConfig) -> Result<()> {
    println!("Running pipeline: {}", config.name);

    // Store loaded tables in memory (in production, this could be cached to disk)
    let mut tables: std::collections::HashMap<String, pipeline::Table> =
        std::collections::HashMap::new();

    for (idx, step) in config.steps.iter().enumerate() {
        println!("\n[Step {}] {:?}", idx + 1, step);

        match step {
            PipelineStep::LoadExcel { source, sheet, output } => {
                let data = io::load_excel_sheet(source, sheet)?;
                
                // Save intermediate CSV with embedded newlines/semicolons intact
                let temp_path = format!("{}.temp", output);
                io::save_csv_clean(&data, &temp_path)?;
                
                // Clean the CSV before final save
                cleaner::clean_syteline_csv(&temp_path, output)?;
                
                // Load into memory for further processing
                let table = pipeline::Table::load_csv(output)?;

                tables.insert(output.clone(), table);

                println!("  ✓ Loaded {} rows from Excel sheet", data.len());
            }

            PipelineStep::LoadCsv { source, output } => {
                let table = pipeline::Table::load_csv(source)?;
                io::save_csv_clean(
                    &std::iter::once(table.headers.clone())
                        .chain(table.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), table);

                println!("  ✓ Loaded CSV: {} rows", output);
            }

            PipelineStep::Filter { input, output, condition } => {
                let table = tables.get(input).ok_or_else(|| {
                    anyhow::anyhow!("Input file '{}' not found in pipeline", input)
                })?;

                let filtered = table.filter(condition)?;

                // Get row count before move
                let filtered_count = filtered.rows.len();

                io::save_csv_clean(
                    &std::iter::once(filtered.headers.clone())
                        .chain(filtered.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), filtered);

                println!("  ✓ Filtered to {} rows (input: {})", filtered_count, input);
            }

            PipelineStep::Join { left, right, on, output, join_type } => {
                let left_table = tables
                    .get(left)
                    .ok_or_else(|| anyhow::anyhow!("Left table '{}' not found", left))?;

                // Check if right is an Excel file reference (format: "file.xlsx|sheet_name")
                let right_table = if let Some(pipe_pos) = right.find('|') {
                    let excel_path = &right[..pipe_pos];
                    let sheet_name = &right[pipe_pos + 1..];

                    // Load Excel sheet and convert to Table
                    let data = io::load_excel_sheet(excel_path, sheet_name)?;
                    let table = pipeline::Table::from_data(data)?;
                    table
                } else {
                    tables
                        .get(right)
                        .ok_or_else(|| anyhow::anyhow!("Right table '{}' not found", right))?
                        .clone()
                };

                // Clone values we need before calling join (which borrows mutably)
                let left_len = left_table.rows.len();
                let right_len = right_table.rows.len();
                let on_refs: Vec<&str> = on.iter().map(|s| s.as_str()).collect();
                let join_type_clone = join_type.clone();
                let join_type_str = match join_type_clone {
                    Some(jt) => match jt {
                        JoinType::Left => "left",
                        JoinType::Inner => "inner",
                        JoinType::Full => "full",
                        JoinType::Right => "right",
                    },
                    None => "inner",
                };

                let joined = left_table.join(&right_table, &on_refs, join_type_str)?;
                io::save_csv_clean(
                    &std::iter::once(joined.headers.clone())
                        .chain(joined.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), joined);

                println!(
                    "  ✓ Joined {} rows ({} -> {})",
                    left_len + right_len,
                    left_len,
                    right_len
                );
            }

            PipelineStep::Pivot { input, index_cols, value_col, agg_func, output } => {
                let table = tables
                    .get(input)
                    .ok_or_else(|| anyhow::anyhow!("Input table '{}' not found", input))?;

                let index_refs: Vec<&str> = index_cols.iter().map(|s| s.as_str()).collect();
                let agg_str = match agg_func {
                    AggFunction::Sum => "sum",
                    AggFunction::Count => "count",
                    AggFunction::Avg => "avg",
                    AggFunction::Min => "min",
                    AggFunction::Max => "max",
                    AggFunction::First => "first",
                    AggFunction::Last => "last",
                };

                let pivoted = table.pivot(&index_refs, value_col.as_str(), agg_str)?;
                io::save_csv_clean(
                    &std::iter::once(pivoted.headers.clone())
                        .chain(pivoted.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), pivoted);

                println!("  ✓ Pivoted to {} rows", input);
            }

            PipelineStep::GroupBy { input, group_cols, aggregations, output } => {
                let table = tables
                    .get(input)
                    .ok_or_else(|| anyhow::anyhow!("Input table '{}' not found", input))?;

                let group_refs: Vec<&str> = group_cols.iter().map(|s| s.as_str()).collect();
                let agg_refs: Vec<(&str, &str)> = aggregations
                    .iter()
                    .map(|a| {
                        (
                            a.column.as_str(),
                            match a.func {
                                AggFunction::Sum => "sum",
                                AggFunction::Count => "count",
                                AggFunction::Avg => "avg",
                                AggFunction::Min => "min",
                                AggFunction::Max => "max",
                                AggFunction::First => "first",
                                AggFunction::Last => "last",
                            },
                        )
                    })
                    .collect();

                let grouped = table.group_by(&group_refs, &agg_refs)?;
                io::save_csv_clean(
                    &std::iter::once(grouped.headers.clone())
                        .chain(grouped.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), grouped);

                println!("  ✓ Grouped to {} rows", input);
            }

            PipelineStep::Mutate { input, columns, output } => {
                let table = tables
                    .get(input)
                    .ok_or_else(|| anyhow::anyhow!("Input table '{}' not found", input))?;

                let defs: Vec<(&str, &str)> = columns
                    .iter()
                    .map(|c| (c.name.as_str(), c.expression.as_str()))
                    .collect();

                let mutated = table.mutate(&defs)?;
                io::save_csv_clean(
                    &std::iter::once(mutated.headers.clone())
                        .chain(mutated.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), mutated);

                println!("  ✓ Mutated: added {} columns", input);
            }

            PipelineStep::Select { input, columns, output } => {
                let table = tables
                    .get(input)
                    .ok_or_else(|| anyhow::anyhow!("Input table '{}' not found", input))?;

                let col_refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
                let selected = table.select(&col_refs)?;
                io::save_csv_clean(
                    &std::iter::once(selected.headers.clone())
                        .chain(selected.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), selected);

                println!("  ✓ Selected {} columns", input);
            }

            PipelineStep::Sort { input, by, output } => {
                let table = tables
                    .get(input)
                    .ok_or_else(|| anyhow::anyhow!("Input table '{}' not found", input))?;

                let sort_refs: Vec<(&str, bool)> = by
                    .iter()
                    .map(|s| (s.column.as_str(), s.descending.unwrap_or(false)))
                    .collect();

                let sorted = table.sort(&sort_refs)?;
                io::save_csv_clean(
                    &std::iter::once(sorted.headers.clone())
                        .chain(sorted.rows.iter().cloned())
                        .collect::<Vec<_>>(),
                    output,
                )?;
                tables.insert(output.clone(), sorted);

                println!("  ✓ Sorted by {} columns", input);
            }
        }
    }

    println!("\n=== Pipeline Complete ===");
    Ok(())
}
