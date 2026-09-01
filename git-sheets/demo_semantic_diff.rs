// demo_semantic_diff.rs
// Real-world demo: expense tracking data across two snapshots, showing
// how rsf-core column profiling turns a plain row diff into a semantic
// one (type drift, cardinality collapse, null-rate spikes).

use gitsheets::{RsfSnapshot, SemanticDiff, Snapshot, Table};

fn expense_table(rows: Vec<Vec<String>>) -> Table {
    Table {
        headers: vec![
            "ID".to_string(),
            "Vendor".to_string(),
            "Amount".to_string(),
            "Category".to_string(),
        ],
        rows,
        primary_key: Some(vec![0]),
    }
}

fn main() {
    // v1: clean expense report — amounts carry a $ prefix, all fields populated
    let v1 = expense_table(vec![
        vec![
            "TXN001".into(),
            "Acme Corp".into(),
            "$1,200.00".into(),
            "Software".into(),
        ],
        vec![
            "TXN002".into(),
            "Globex".into(),
            "$340.50".into(),
            "Travel".into(),
        ],
        vec![
            "TXN003".into(),
            "Initech".into(),
            "$87.99".into(),
            "Office".into(),
        ],
        vec![
            "TXN004".into(),
            "Umbrella".into(),
            "$2,150.00".into(),
            "Software".into(),
        ],
    ]);

    // v2: a re-export from a different system — the $ prefix is gone (type
    // drift on Amount), two rows were dropped (cardinality collapse), and
    // Category came back empty for the surviving rows (null spike).
    let v2 = expense_table(vec![
        vec![
            "TXN001".into(),
            "Acme Corp".into(),
            "1200.00".into(),
            "".into(),
        ],
        vec![
            "TXN002".into(),
            "Globex".into(),
            "340.50".into(),
            "".into(),
        ],
    ]);

    let snap1 = Snapshot::new(v1, Some("v1: original export".into()));
    let snap2 = Snapshot::new(v2, Some("v2: re-export from ERP".into()));

    let rsf1 = RsfSnapshot::from_snapshot(&snap1);
    let rsf2 = RsfSnapshot::from_snapshot(&snap2);

    println!("Inferred primary key (v1): {:?}", rsf1.infer_primary_key());

    let diff = SemanticDiff::compute(&rsf1, &rsf2);
    diff.print_report();

    // The point of the demo: a plain diff says "2 rows removed" and stops.
    // The semantic layer adds *why that matters* — the Amount column changed
    // type (currency formatting lost) and Category went fully null.
    assert!(
        !diff.semantic_issues.is_empty(),
        "expected semantic issues to be detected"
    );
    println!("\nDemo assertions passed: {} semantic issue(s) detected.", diff.semantic_issues.len());
}
