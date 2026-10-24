//! Integration tests for nustage CLI.

use std::process::Command;

fn run_nustage(args: &[&str]) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "nustage", "--bin", "nustage", "--"]);
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}

#[test]
fn test_import_csv() {
    let output = run_nustage(&["import", "tests/fixtures/sales.csv"])
        .output()
        .expect("failed to execute nustage");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Imported: tests/fixtures/sales.csv"));
    assert!(stdout.contains("Headers:"));
}

#[test]
fn test_preview_pipeline() {
    let output = run_nustage(&["preview", "tests/fixtures/high_value.toml"])
        .output()
        .expect("failed to execute nustage");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Previewing pipeline: high-value orders"));
    assert!(stdout.contains("FilterRows"));
}