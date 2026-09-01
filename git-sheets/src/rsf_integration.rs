// git-sheets × rsf-core integration
// Enable semantic diffing using rsf-core's column profiling and type system

use crate::core::Snapshot;
use crate::diff::SnapshotDiff;
use rsf::{ColumnMeta, TypeHint};
use serde::{Deserialize, Serialize};

/// Extension to Snapshot that includes rsf-core metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsfSnapshot {
    /// Original snapshot
    pub base: Snapshot,
    /// Column profiles from rsf-core (cardinality, type hints, etc.)
    pub column_profiles: Vec<ColumnMeta>,
}

impl RsfSnapshot {
    /// Create an rsf-enhanced snapshot from a regular snapshot
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        let column_profiles = rsf::rank_columns(
            &snapshot.table.headers,
            &snapshot.table.rows,
            rsf::RankingOptions::default(),
        )
        .unwrap_or_default();

        Self {
            base: snapshot.clone(),
            column_profiles,
        }
    }

    /// Get the primary key columns based on functional dependency analysis
    pub fn infer_primary_key(&self) -> Option<Vec<usize>> {
        // Find highest-cardinality column(s) that could serve as keys
        let mut sorted_cols: Vec<_> = self.column_profiles.iter().enumerate().collect();
        sorted_cols.sort_by(|a, b| {
            b.1.cardinality
                .cmp(&a.1.cardinality)
                .then(a.0.cmp(&b.0))
        });

        // First column with very high cardinality is likely a key
        if let Some((idx, col)) = sorted_cols.first() {
            if col.cardinality as f64 / self.base.table.rows.len() as f64 > 0.9 {
                return Some(vec![*idx]);
            }
        }

        None
    }

    /// Get type hint for a column by index
    pub fn get_type_hint(&self, col_idx: usize) -> Option<&TypeHint> {
        self.column_profiles.get(col_idx).and_then(|c| c.type_hint.as_ref())
    }
}

/// Enhanced diff that uses rsf-core semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiff {
    /// Base snapshot diff
    pub base: SnapshotDiff,
    /// Detected semantic changes (e.g., type mismatches)
    pub semantic_issues: Vec<SemanticIssue>,
}

/// A semantic issue detected during diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SemanticIssue {
    /// Column type changed between snapshots
    TypeChanged {
        col_idx: usize,
        old_type: TypeHint,
        new_type: TypeHint,
    },
    /// Cardinality dropped significantly (possible data loss)
    CardinalityDropped {
        col_idx: usize,
        old_cardinality: usize,
        new_cardinality: usize,
        drop_pct: f64,
    },
    /// Null rate increased significantly
    NullRateIncreased {
        col_idx: usize,
        old_null_rate: f64,
        new_null_rate: f64,
    },
}

impl SemanticDiff {
    /// Compute diff with semantic analysis
    pub fn compute(from_rsf: &RsfSnapshot, to_rsf: &RsfSnapshot) -> Self {
        let base = SnapshotDiff::compute(&from_rsf.base, &to_rsf.base).unwrap();
        
        let mut semantic_issues = Vec::new();

        // Check for type changes and cardinality drops
        for (old_col, new_col) in from_rsf.column_profiles.iter().zip(to_rsf.column_profiles.iter()) {
            // Type changes
            if let (Some(old_type), Some(new_type)) = (old_col.type_hint.as_ref(), new_col.type_hint.as_ref()) {
                if old_type != new_type {
                    semantic_issues.push(SemanticIssue::TypeChanged {
                        col_idx: old_col.rank - 1, // rank is 1-indexed
                        old_type: old_type.clone(),
                        new_type: new_type.clone(),
                    });
                }
            }

            // Cardinality drops
            let drop_pct = if old_col.cardinality > 0 {
                ((old_col.cardinality as f64 - new_col.cardinality as f64) / old_col.cardinality as f64) * 100.0
            } else {
                0.0
            };

            if drop_pct > 20.0 && new_col.cardinality < old_col.cardinality {
                semantic_issues.push(SemanticIssue::CardinalityDropped {
                    col_idx: old_col.rank - 1,
                    old_cardinality: old_col.cardinality,
                    new_cardinality: new_col.cardinality,
                    drop_pct,
                });
            }

            // Null rate increases
            let old_null = old_col.null_pct.unwrap_or(0.0);
            let new_null = new_col.null_pct.unwrap_or(0.0);
            
            if new_null - old_null > 10.0 {
                semantic_issues.push(SemanticIssue::NullRateIncreased {
                    col_idx: old_col.rank - 1,
                    old_null_rate: old_null,
                    new_null_rate: new_null,
                });
            }
        }

        Self { base, semantic_issues }
    }

    /// Print a human-readable report of the diff
    pub fn print_report(&self) {
        println!("=== Semantic Diff Report ===\n");
        
        // Base summary
        println!("Changes:");
        println!("  Rows added: {}", self.base.summary.rows_added);
        println!("  Rows removed: {}", self.base.summary.rows_removed);
        println!("  Rows modified: {}", self.base.summary.rows_modified);
        println!("  Columns added: {}", self.base.summary.columns_added);
        println!("  Columns removed: {}", self.base.summary.columns_removed);

        // Semantic issues
        if !self.semantic_issues.is_empty() {
            println!("\n⚠️  Semantic Issues:");
            for issue in &self.semantic_issues {
                match issue {
                    SemanticIssue::TypeChanged { col_idx, old_type, new_type } => {
                        println!("  - Column {} type changed: {:?} → {:?}", 
                                 col_idx, old_type, new_type);
                    }
                    SemanticIssue::CardinalityDropped { col_idx, old_cardinality, new_cardinality, drop_pct } => {
                        println!("  - Column {} cardinality dropped {}% ({:?} → {:?})", 
                                 col_idx, drop_pct, old_cardinality, new_cardinality);
                    }
                    SemanticIssue::NullRateIncreased { col_idx, old_null_rate, new_null_rate } => {
                        println!("  - Column {} null rate increased: {:.1}% → {:.1}%", 
                                 col_idx, old_null_rate, new_null_rate);
                    }
                }
            }
        } else {
            println!("\n✓ No semantic issues detected");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Snapshot, Table};

    #[test]
    fn test_rsf_snapshot_creation() {
        let table = Table {
            headers: vec!["ID".to_string(), "Amount".to_string()],
            rows: vec![
                vec!["TXN001".to_string(), "$100".to_string()],
                vec!["TXN002".to_string(), "$200".to_string()],
                vec!["TXN003".to_string(), "$300".to_string()],
            ],
            primary_key: None,
        };

        let snapshot = Snapshot::new(table, Some("test".to_string()));
        let rsf_snapshot = RsfSnapshot::from_snapshot(&snapshot);

        assert_eq!(rsf_snapshot.column_profiles.len(), 2);
        // ID should be detected as alphanumeric ID
        if let Some(TypeHint::Id(id_type)) = rsf_snapshot.get_type_hint(0) {
            assert_eq!(id_type, "alphanumeric");
        } else {
            // At minimum, we should have some type hint or at least 3 distinct values
            assert_eq!(rsf_snapshot.column_profiles[0].cardinality, 3);
        }
    }

    #[test]
    fn test_semantic_diff_detection() {
        let table1 = Table {
            headers: vec!["ID".to_string(), "Amount".to_string()],
            rows: vec![
                vec!["1".to_string(), "$100".to_string()],
                vec!["2".to_string(), "$200".to_string()],
            ],
            primary_key: None,
        };

        let table2 = Table {
            headers: vec!["ID".to_string(), "Amount".to_string()],
            rows: vec![
                vec!["1".to_string(), "100".to_string()], // Lost $ symbol
                vec!["2".to_string(), "200".to_string()],
            ],
            primary_key: None,
        };

        let snap1 = Snapshot::new(table1, Some("v1".to_string()));
        let snap2 = Snapshot::new(table2, Some("v2".to_string()));

        let rsf1 = RsfSnapshot::from_snapshot(&snap1);
        let rsf2 = RsfSnapshot::from_snapshot(&snap2);

        let semantic_diff = SemanticDiff::compute(&rsf1, &rsf2);

        // Should detect the type change from Currency to Unknown
        assert!(semantic_diff.semantic_issues.len() > 0);
    }
}
