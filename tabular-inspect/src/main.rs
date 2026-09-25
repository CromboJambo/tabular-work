//! Tabular Inspector — detect programmatic generation in Excel files.
//!
//! Phase 1: Basic file structure inspection (sheet count, shared strings, etc.)
//! Phase 2: Pattern detection heuristics for programmatic vs hand-edited
//! Phase 3: Change detection between two versions of a spreadsheet

use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: tabular-inspect <file.xlsx>");
        eprintln!("       tabular-inspect diff <before.xlsx> <after.xlsx>");
        std::process::exit(1);
    }

    // Check for diff subcommand (Phase 3)
    if args[1] == "diff" {
        if args.len() < 4 {
            eprintln!("Usage: tabular-inspect diff <before.xlsx> <after.xlsx>");
            std::process::exit(1);
        }
        match diff_files(&args[2], &args[3]) {
            Ok(report) => println!("{}", report),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Single file inspection (Phase 1 + Phase 2)
        let path = &args[1];
        let file_path = Path::new(path);

        if !file_path.exists() {
            eprintln!("File not found: {}", path);
            std::process::exit(1);
        }

        match inspect_file(path) {
            Ok(report) => println!("{}", report),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn inspect_file(path: &str) -> Result<String, String> {
    let content = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let cursor = Cursor::new(content);

    // Phase 1: Basic structure inspection via zip archive
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Not a valid ZIP/xlsx file: {}", e))?;

    let mut report = String::from("File type: xlsx\n");

    // Count sheets by checking workbook.xml
    if let Ok(wb_content) = read_xml_entry(&mut archive, "xl/workbook.xml") {
        let sheet_count = count_sheets(&wb_content);
        report.push_str(&format!("Sheets: {}\n", sheet_count));
    } else {
        report.push_str("Sheets: unknown\n");
    }

    // Check for shared strings
    if let Ok(ss_content) = read_xml_entry(&mut archive, "xl/sharedStrings.xml") {
        let count = count_shared_strings(&ss_content);
        report.push_str(&format!("Shared strings: {} entries\n", count));
    } else {
        report.push_str("Shared strings: none\n");
    }

    // Check for defined names
    if let Ok(dn_content) = read_xml_entry(&mut archive, "xl/definedNames.xml") {
        let count = count_defined_names(&dn_content);
        report.push_str(&format!("Defined names: {}\n", count));
    } else {
        report.push_str("Defined names: none\n");
    }

    // Check for custom properties
    if let Ok(props) = read_xml_entry(&mut archive, "docProps/custom.xml") {
        report.push_str(&format!(
            "Custom properties: present ({} bytes)\n",
            props.len()
        ));
    } else {
        report.push_str("Custom properties: none\n");
    }

    // Check for macros
    if let Ok(_) = read_xml_entry(&mut archive, "xl/vbaProject.bin") {
        report.push_str("Macros present: yes\n");
    } else {
        report.push_str("Macros present: no\n");
    }

    // Phase 2: Pattern detection heuristics
    let heuristics = detect_patterns(&mut archive);
    report.push_str("\n=== Pattern Detection (Phase 2) ===\n");
    for (name, score) in &heuristics {
        report.push_str(&format!("{}: {:.0}%\n", name, *score));
    }

    // Overall classification
    let total_score: f64 = heuristics.iter().map(|(_, s)| *s).sum();
    let avg = total_score / heuristics.len() as f64;
    report.push_str(&format!("\nOverall programmatic score: {:.0}%\n", avg));

    if avg > 75.0 {
        report.push_str("Classification: likely programmatically generated\n");
    } else if avg < 30.0 {
        report.push_str("Classification: likely hand-edited\n");
    } else {
        report.push_str("Classification: mixed / inconclusive\n");
    }

    Ok(report)
}

/// Phase 3: Compare two versions of a spreadsheet and report changes
fn diff_files(before_path: &str, after_path: &str) -> Result<String, String> {
    let mut report = String::from("=== Change Detection (Phase 3) ===\n");
    report.push_str(&format!("Before: {}\n", before_path));
    report.push_str(&format!("After:  {}\n\n", after_path));

    // Load both files' metadata
    let before_info = load_file_info(before_path)?;
    let after_info = load_file_info(after_path)?;

    // Compare sheet counts
    if before_info.sheet_count != after_info.sheet_count {
        report.push_str(&format!(
            "Structural: Sheet count changed from {} to {}\n",
            before_info.sheet_count, after_info.sheet_count
        ));
    } else {
        report.push_str("Structural: No sheet count change\n");
    }

    // Compare shared string counts
    let ss_diff = after_info.ss_count - before_info.ss_count;
    if ss_diff != 0 {
        report.push_str(&format!(
            "Content: Shared strings {} by {} ({} -> {})\n",
            if ss_diff > 0 { "increased" } else { "decreased" },
            ss_diff.abs(),
            before_info.ss_count,
            after_info.ss_count
        ));
    } else {
        report.push_str("Content: No shared string count change\n");
    }

    // Compare defined names
    let dn_diff = after_info.dn_count - before_info.dn_count;
    if dn_diff != 0 {
        report.push_str(&format!(
            "Named ranges: {} by {} ({} -> {})\n",
            if dn_diff > 0 { "increased" } else { "decreased" },
            dn_diff.abs(),
            before_info.dn_count,
            after_info.dn_count
        ));
    } else {
        report.push_str("Named ranges: No change\n");
    }

    // Compare macros
    if before_info.has_macros != after_info.has_macros {
        if after_info.has_macros {
            report.push_str("Macros: added\n");
        } else {
            report.push_str("Macros: removed\n");
        }
    } else {
        report.push_str("Macros: no change\n");
    }

    // Compare custom properties
    if before_info.has_custom_props != after_info.has_custom_props {
        if after_info.has_custom_props {
            report.push_str("Custom properties: added\n");
        } else {
            report.push_str("Custom properties: removed\n");
        }
    } else {
        report.push_str("Custom properties: no change\n");
    }

    // Run pattern detection on both and compare scores
    let before_patterns = run_pattern_detection(before_path)?;
    let after_patterns = run_pattern_detection(after_path)?;

    report.push_str("\n=== Pattern Score Changes ===\n");
    for (name, before_score) in &before_patterns {
        if let Some(after_score) = after_patterns.iter().find(|(n, _)| n == name).map(|(_, s)| *s) {
            let diff = after_score - before_score;
            if diff.abs() > 1.0 {
                report.push_str(&format!(
                    "{}: {:.0}% -> {:.0}% ({:+.0}%)\n",
                    name, before_score, after_score, diff
                ));
            }
        }
    }

    // Overall classification change
    let before_avg = avg_score(&before_patterns);
    let after_avg = avg_score(&after_patterns);

    report.push_str("\n=== Classification Change ===\n");
    report.push_str(&format!("Before: {:.0}% - {}\n", before_avg, classify(before_avg)));
    report.push_str(&format!("After:  {:.0}% - {}\n", after_avg, classify(after_avg)));

    if before_avg < 30.0 && after_avg > 75.0 {
        report.push_str("Change: hand-edited -> programmatically generated\n");
    } else if before_avg > 75.0 && after_avg < 30.0 {
        report.push_str("Change: programmatically generated -> hand-edited\n");
    } else if (before_avg < 30.0) != (after_avg < 30.0) || (before_avg > 75.0) != (after_avg > 75.0) {
        report.push_str("Change: classification boundary crossed\n");
    } else {
        report.push_str("Change: same classification category\n");
    }

    Ok(report)
}

fn load_file_info(path: &str) -> Result<FileInfo, String> {
    let content = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let cursor = Cursor::new(content);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Not a valid ZIP/xlsx file: {}", e))?;

    let sheet_count = match read_xml_entry(&mut archive, "xl/workbook.xml") {
        Ok(wb) => count_sheets(&wb),
        Err(_) => 0,
    };

    let ss_count = match read_xml_entry(&mut archive, "xl/sharedStrings.xml") {
        Ok(ss) => count_shared_strings(&ss),
        Err(_) => 0,
    };

    let dn_count = match read_xml_entry(&mut archive, "xl/definedNames.xml") {
        Ok(dn) => count_defined_names(&dn),
        Err(_) => 0,
    };

    let has_custom_props = read_xml_entry(&mut archive, "docProps/custom.xml").is_ok();
    let has_macros = read_xml_entry(&mut archive, "xl/vbaProject.bin").is_ok();

    Ok(FileInfo {
        sheet_count,
        ss_count,
        dn_count,
        has_custom_props,
        has_macros,
    })
}

fn run_pattern_detection(path: &str) -> Result<Vec<(String, f64)>, String> {
    let content = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let cursor = Cursor::new(content);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Not a valid ZIP/xlsx file: {}", e))?;

    Ok(detect_patterns(&mut archive))
}

fn avg_score(scores: &[(String, f64)]) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let total: f64 = scores.iter().map(|(_, s)| *s).sum();
    total / scores.len() as f64
}

fn classify(score: f64) -> &'static str {
    if score > 75.0 {
        "likely programmatically generated"
    } else if score < 30.0 {
        "likely hand-edited"
    } else {
        "mixed / inconclusive"
    }
}

/// Phase 2 heuristics for detecting programmatic generation
fn detect_patterns(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>) -> Vec<(String, f64)> {
    let mut results = Vec::new();

    // Heuristic 1: Shared string density (low count relative to file suggests programmatic)
    if let Ok(ss_content) = read_xml_entry(archive, "xl/sharedStrings.xml") {
        let ss_count = count_shared_strings(&ss_content);
        results.push(("Shared string density".to_string(), ss_density_score(ss_count)));
    }

    // Heuristic 2: Defined name patterns (many defined names suggest programmatic)
    if let Ok(dn_content) = read_xml_entry(archive, "xl/definedNames.xml") {
        let dn_count = count_defined_names(&dn_content);
        results.push(("Defined name complexity".to_string(), dn_complexity_score(dn_count)));
    }

    // Heuristic 3: Custom properties (library fingerprints)
    if let Ok(props) = read_xml_entry(archive, "docProps/custom.xml") {
        results.push(("Custom property presence".to_string(), custom_prop_score(&props)));
    } else {
        results.push(("Custom property presence".to_string(), 0.0));
    }

    // Heuristic 4: Sheet naming patterns (systematic names suggest programmatic)
    if let Ok(wb_content) = read_xml_entry(archive, "xl/workbook.xml") {
        let sheet_names = extract_sheet_names(&wb_content);
        results.push(("Sheet naming pattern".to_string(), sheet_naming_score(&sheet_names)));
    }

    // Heuristic 5: Formula patterns (repeated formulas suggest programmatic)
    // TODO: Analyze worksheet XML for formula repetition

    // Heuristic 6: Statistical analysis of cell value distributions
    // Scan multiple sheets for numeric data patterns
    let mut total_numeric = 0;
    let mut low_entropy_sheets = 0;
    
    if let Ok(wb_content) = read_xml_entry(archive, "xl/workbook.xml") {
        let sheet_names = extract_sheet_names(&wb_content);
        let sheets_to_check = std::cmp::min(sheet_names.len(), 5);
        
        for i in 0..sheets_to_check {
            let sheet_file = format!("xl/worksheets/sheet{}.xml", i + 1);
            if let Ok(sheet_content) = read_xml_entry(archive, &sheet_file) {
                let numeric_values = extract_numeric_values(&sheet_content);
                total_numeric += numeric_values.len();
                
                if numeric_values.len() > 10 {
                    let stats_score = analyze_value_distribution(&numeric_values);
                    if stats_score > 60.0 {
                        low_entropy_sheets += 1;
                    }
                }
            }
        }
    }
    
    if total_numeric > 10 {
        // High proportion of low-entropy sheets suggests programmatic generation
        let entropy_ratio = (low_entropy_sheets as f64) / ((total_numeric as f64) / 100.0);
        let stats_score = (entropy_ratio * 80.0).min(95.0);
        results.push(("Value distribution entropy".to_string(), stats_score));
    }

    results
}

fn ss_density_score(ss_count: i32) -> f64 {
    if ss_count < 10 {
        return 80.0;
    } else if ss_count < 100 {
        return 50.0;
    } else {
        return 20.0;
    }
}

fn dn_complexity_score(dn_count: i32) -> f64 {
    if dn_count > 10 {
        return 90.0;
    } else if dn_count > 3 {
        return 60.0;
    } else {
        return 20.0;
    }
}

fn custom_prop_score(props: &str) -> f64 {
    let lower = props.to_lowercase();
    if lower.contains("openpyxl") || lower.contains("epplus") || lower.contains("apache poi") {
        return 100.0;
    } else if lower.contains("microsoft excel") {
        return 30.0;
    } else {
        return 50.0;
    }
}

fn count_sheets(workbook_content: &str) -> i32 {
    let count = workbook_content.matches("<sheet ").count();
    count as i32
}

fn read_xml_entry<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<String, String> {
    use std::io::Read;
    let mut file = archive
        .by_name(name)
        .map_err(|e| format!("Failed to open {}: {}", name, e))?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .map_err(|e| format!("Failed to read {}: {}", name, e))?;
    Ok(String::from_utf8(content).map_err(|e| format!("UTF-8 error in {}: {}", name, e))?)
}

fn count_shared_strings(content: &str) -> i32 {
    let count = content.matches("<si").count();
    count as i32
}

fn count_defined_names(content: &str) -> i32 {
    let count = content.matches("<definedName").count();
    count as i32
}

/// Extract sheet names from workbook.xml
fn extract_sheet_names(workbook_content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in workbook_content.lines() {
        if line.contains("<sheet ") && line.contains("name=") {
            if let Some(start) = line.find("name=\"") {
                let rest = &line[start + 6..];
                if let Some(end) = rest.find('"') {
                    names.push(rest[..end].to_string());
                }
            }
        }
    }
    names
}

/// Score sheet naming patterns (systematic names suggest programmatic generation)
fn sheet_naming_score(names: &[String]) -> f64 {
    if names.is_empty() {
        return 0.0;
    }

    let has_sequential = names.iter().any(|n| n.starts_with("Sheet"));

    let prefixes: Vec<&str> = names
        .iter()
        .map(|n| if n.len() > 5 { &n[..5] } else { n.as_str() })
        .collect();

    let unique_prefixes = prefixes.iter().fold(HashSet::new(), |mut set, p| {
        set.insert(*p);
        set
    });

    if has_sequential {
        return 70.0; // Sheet1, Sheet2 pattern suggests programmatic
    } else if unique_prefixes.len() == 1 && names.len() > 3 {
        return 60.0; // All same prefix suggests programmatic
    } else {
        return 20.0; // Varied names suggest hand-edited
    }
}

/// Metadata about a file's structure for comparison
struct FileInfo {
    sheet_count: i32,
    ss_count: i32,
    dn_count: i32,
    has_custom_props: bool,
    has_macros: bool,
}

/// Extract numeric values from worksheet XML cells
fn extract_numeric_values(sheet_content: &str) -> Vec<f64> {
    let mut values = Vec::new();
    // Find all <c ...><v>NUMBER</v></c> patterns
    for line in sheet_content.lines() {
        if line.contains("<v>") {
            if let Some(start) = line.find("<v>") {
                let rest = &line[start + 3..];
                if let Some(end) = rest.find("</v>") {
                    let num_str = &rest[..end];
                    if let Ok(num) = num_str.parse::<f64>() {
                        values.push(num);
                    }
                }
            }
        }
    }
    values
}

/// Analyze value distribution for programmatic patterns
/// Low entropy (many repeated values, uniform distribution) suggests programmatic generation
fn analyze_value_distribution(values: &[f64]) -> f64 {
    if values.len() < 10 {
        return 0.0;
    }

    // Calculate unique value ratio (low = many repeats = programmatic)
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut unique_count = 1;
    for i in 1..sorted.len() {
        if (sorted[i] - sorted[i-1]).abs() > 0.0001 {
            unique_count += 1;
        }
    }

    let unique_ratio = unique_count as f64 / values.len() as f64;

    // Calculate range vs count ratio (wide range, few values = programmatic)
    if sorted.len() > 1 {
        let min_val = *sorted.first().unwrap();
        let max_val = *sorted.last().unwrap();
        let range = max_val - min_val;
        
        if range > 0.0 && unique_ratio < 0.5 {
            // Many repeated values across wide range = very programmatic
            return 85.0;
        } else if unique_ratio < 0.3 {
            // Very few unique values relative to total
            return 70.0;
        } else if unique_ratio < 0.6 {
            return 40.0;
        }
    }

    // High uniqueness suggests hand-edited
    if unique_ratio > 0.8 {
        return 15.0;
    }

    return 30.0;
}