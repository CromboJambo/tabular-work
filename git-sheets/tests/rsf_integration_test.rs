// Comprehensive integration tests for git-sheets × rsf-core

use gitsheets::{Snapshot, Table};
use gitsheets::rsf_integration::{RsfSnapshot, SemanticDiff, SemanticIssue};
use rsf::TypeHint;

#[test]
fn test_empty_table_rsf_snapshot() {
    let table = Table {
        headers: vec!["Col1".to_string(), "Col2".to_string()],
        rows: vec![],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("empty".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    assert_eq!(rsf_snapshot.column_profiles.len(), 2);
}

#[test]
fn test_single_row_table() {
    let table = Table {
        headers: vec!["ID".to_string(), "Value".to_string()],
        rows: vec![vec!["1".to_string(), "$100".to_string()]],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("single".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    assert_eq!(rsf_snapshot.column_profiles.len(), 2);
    assert_eq!(rsf_snapshot.column_profiles[0].cardinality, 1);
}

#[test]
fn test_large_cardinality_detection() {
    let mut table = Table {
        headers: vec!["ID".to_string(), "Value".to_string()],
        rows: Vec::new(),
        primary_key: None,
    };

    for i in 1..=100 {
        table.rows.push(vec![format!("ID{:03}", i), "$100".to_string()]);
    }

    let snapshot = Snapshot::new(table, Some("large".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    assert_eq!(rsf_snapshot.column_profiles[0].cardinality, 100);
}

#[test]
fn test_type_detection_integer() {
    let table = Table {
        headers: vec!["Count".to_string()],
        rows: vec![
            vec!["1".to_string()],
            vec!["2".to_string()],
            vec!["3".to_string()],
            vec!["4".to_string()],
        ],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("int".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    if let Some(type_hint) = &rsf_snapshot.column_profiles[0].type_hint {
        assert_eq!(type_hint, &TypeHint::Integer);
    } else {
        panic!("Expected Integer type hint");
    }
}

#[test]
fn test_type_detection_boolean() {
    let table = Table {
        headers: vec!["Active".to_string()],
        rows: vec![
            vec!["true".to_string()],
            vec!["false".to_string()],
            vec!["true".to_string()],
            vec!["false".to_string()],
        ],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("bool".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    if let Some(type_hint) = &rsf_snapshot.column_profiles[0].type_hint {
        assert_eq!(type_hint, &TypeHint::Boolean);
    } else {
        panic!("Expected Boolean type hint");
    }
}

#[test]
fn test_type_detection_currency() {
    let table = Table {
        headers: vec!["Amount".to_string()],
        rows: vec![
            vec!["$100.50".to_string()],
            vec!["$200.75".to_string()],
            vec!["$300.00".to_string()],
        ],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("currency".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    if let Some(type_hint) = &rsf_snapshot.column_profiles[0].type_hint {
        assert_eq!(type_hint, &TypeHint::Currency);
    } else {
        panic!("Expected Currency type hint");
    }
}

#[test]
fn test_type_detection_date() {
    let table = Table {
        headers: vec!["Date".to_string()],
        rows: vec![
            vec!["2024-01-15".to_string()],
            vec!["2024-02-20".to_string()],
            vec!["2024-03-10".to_string()],
        ],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("date".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    if let Some(type_hint) = &rsf_snapshot.column_profiles[0].type_hint {
        assert_eq!(type_hint, &TypeHint::Date);
    } else {
        panic!("Expected Date type hint");
    }
}

#[test]
fn test_type_detection_alphanumeric_id() {
    let table = Table {
        headers: vec!["TransactionID".to_string()],
        rows: vec![
            vec!["TXN001".to_string()],
            vec!["TXN002".to_string()],
            vec!["TXN003".to_string()],
        ],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("id".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    if let Some(type_hint) = &rsf_snapshot.column_profiles[0].type_hint {
        match type_hint {
            TypeHint::Id(id_type) => assert_eq!(id_type, "alphanumeric"),
            _ => panic!("Expected alphanumeric ID, got {:?}", type_hint),
        }
    } else {
        panic!("Expected ID type hint");
    }
}

#[test]
fn test_semantic_issue_type_change() {
    let table1 = Table {
        headers: vec!["Amount".to_string()],
        rows: vec![vec!["$100".to_string()], vec!["$200".to_string()]],
        primary_key: None,
    };

    let table2 = Table {
        headers: vec!["Amount".to_string()],
        rows: vec![vec!["100".to_string()], vec!["200".to_string()]],
        primary_key: None,
    };

    let snap1 = Snapshot::new(table1, Some("v1".to_string()));
    let snap2 = Snapshot::new(table2, Some("v2".to_string()));

    let rsf1 = RsfSnapshot::from_snapshot(&snap1);
    let rsf2 = RsfSnapshot::from_snapshot(&snap2);

    let diff = SemanticDiff::compute(&rsf1, &rsf2);

    let has_type_change = diff
        .semantic_issues
        .iter()
        .any(|issue| matches!(issue, SemanticIssue::TypeChanged { .. }));
    assert!(has_type_change, "Expected TypeChanged issue");
}

#[test]
fn test_semantic_issue_cardinality_drop() {
    let table1 = Table {
        headers: vec!["ID".to_string()],
        rows: (1..=100).map(|i| vec![format!("{}", i)]).collect(),
        primary_key: None,
    };

    let table2 = Table {
        headers: vec!["ID".to_string()],
        rows: vec![vec!["1".to_string()], vec!["2".to_string()]],
        primary_key: None,
    };

    let snap1 = Snapshot::new(table1, Some("v1".to_string()));
    let snap2 = Snapshot::new(table2, Some("v2".to_string()));

    let rsf1 = RsfSnapshot::from_snapshot(&snap1);
    let rsf2 = RsfSnapshot::from_snapshot(&snap2);

    let diff = SemanticDiff::compute(&rsf1, &rsf2);

    let has_cardinality_drop = diff
        .semantic_issues
        .iter()
        .any(|issue| matches!(issue, SemanticIssue::CardinalityDropped { .. }));
    assert!(has_cardinality_drop, "Expected CardinalityDropped issue");
}

#[test]
fn test_infer_primary_key() {
    let table = Table {
        headers: vec!["ID".to_string(), "Name".to_string()],
        rows: (1..=100)
            .map(|i| vec![format!("ID{:03}", i), format!("Name{}", i)])
            .collect(),
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("pk_test".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    let pk = rsf_snapshot.infer_primary_key();
    assert!(pk.is_some(), "Expected to infer primary key");
}

#[test]
fn test_null_rate_tracking() {
    let table1 = Table {
        headers: vec!["Value".to_string()],
        rows: vec![
            vec!["100".to_string()],
            vec!["200".to_string()],
            vec!["300".to_string()],
        ],
        primary_key: None,
    };

    let table2 = Table {
        headers: vec!["Value".to_string()],
        rows: vec![
            vec!["".to_string()],
            vec!["200".to_string()],
            vec!["".to_string()],
        ],
        primary_key: None,
    };

    let snap1 = Snapshot::new(table1, Some("v1".to_string()));
    let snap2 = Snapshot::new(table2, Some("v2".to_string()));

    let rsf1 = RsfSnapshot::from_snapshot(&snap1);
    let rsf2 = RsfSnapshot::from_snapshot(&snap2);

    assert_eq!(rsf1.column_profiles[0].null_pct.unwrap_or(0.0), 0.0);
    let null_pct = rsf2.column_profiles[0].null_pct.unwrap_or(0.0);
    assert!(
        null_pct > 50.0,
        "Expected high null percentage, got {}",
        null_pct
    );
}

#[test]
fn test_column_ranking_by_cardinality() {
    let table = Table {
        headers: vec![
            "Category".to_string(),
            "ID".to_string(),
            "Name".to_string(),
        ],
        rows: (1..=100)
            .map(|i| {
                vec![
                    if i % 2 == 0 {
                        "A".to_string()
                    } else {
                        "B".to_string()
                    },
                    format!("ID{:03}", i),
                    format!("Name{}", i),
                ]
            })
            .collect(),
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("ranking".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    assert_eq!(rsf_snapshot.column_profiles[0].rank, 1);
    assert_eq!(rsf_snapshot.column_profiles[0].name, "ID");
}

#[test]
fn test_diff_with_multiple_type_changes() {
    let table1 = Table {
        headers: vec!["Amount".to_string(), "Count".to_string()],
        rows: vec![
            vec!["$100".to_string(), "5".to_string()],
            vec!["$200".to_string(), "10".to_string()],
        ],
        primary_key: None,
    };

    let table2 = Table {
        headers: vec!["Amount".to_string(), "Count".to_string()],
        rows: vec![
            vec!["100".to_string(), "5".to_string()],
            vec!["200".to_string(), "10".to_string()],
        ],
        primary_key: None,
    };

    let snap1 = Snapshot::new(table1, Some("v1".to_string()));
    let snap2 = Snapshot::new(table2, Some("v2".to_string()));

    let rsf1 = RsfSnapshot::from_snapshot(&snap1);
    let rsf2 = RsfSnapshot::from_snapshot(&snap2);

    let diff = SemanticDiff::compute(&rsf1, &rsf2);

    let type_changes: Vec<_> = diff
        .semantic_issues
        .iter()
        .filter(|issue| matches!(issue, SemanticIssue::TypeChanged { .. }))
        .collect();
    assert_eq!(type_changes.len(), 1, "Expected exactly 1 type change");
}

#[test]
fn test_get_type_hint_by_index() {
    let table = Table {
        headers: vec!["ID".to_string(), "Amount".to_string()],
        rows: vec![vec!["TXN001".to_string(), "$100".to_string()]],
        primary_key: None,
    };

    let snapshot = Snapshot::new(table, Some("hint".to_string()));
    let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

    let id_type = rsf_snapshot.get_type_hint(0);
    assert!(id_type.is_some());

    let amount_type = rsf_snapshot.get_type_hint(1);
    assert!(amount_type.is_some());
}

#[test]
fn test_large_dataset_performance() {
    use std::time::Instant;

    let mut table = Table {
        headers: vec!["ID".to_string(), "Value".to_string()],
        rows: Vec::new(),
        primary_key: None,
    };

    for i in 1..=1000 {
        table.rows.push(vec![format!("ID{:04}", i), format!("Value{}", i)]);
    }

    let snapshot = Snapshot::new(table, Some("large".to_string()));

    let start = Instant::now();
    let _rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 1000,
        "Performance regression: took {:?} for 1000 rows",
        elapsed
    );
}
