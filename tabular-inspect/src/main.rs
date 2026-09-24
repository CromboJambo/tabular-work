//! Tabular Inspector — detect programmatic generation in Excel files.
//!
//! Phase 1: Basic file structure inspection (sheet count, shared strings, etc.)

use std::fs;
use std::io::Cursor;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tabular-inspect <file.xlsx>");
        std::process::exit(1);
    }

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

    Ok(report)
}

fn count_sheets(workbook_content: &str) -> i32 {
    // Count <sheet entries in workbook.xml
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
    // Count <si> elements in sharedStrings.xml
    let count = content.matches("<si").count();
    count as i32
}

fn count_defined_names(content: &str) -> i32 {
    // Count <definedName elements
    let count = content.matches("<definedName").count();
    count as i32
}