//! Topology — the invariants of a table that survive deformation.
//!
//! A `TopologySignature` is what you get when you *parse* a table: column
//! names, type classes, cardinality classes, null rates, key-ness. It is
//! deliberately coarse — it must stay stable under renames-of-values,
//! reordering, small data drift — so that a signature is comparable across
//! drifted inputs and the delta between two signatures is a *rebase hint*,
//! not an error.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use rsf::{ColumnType, TypeHint, compute_profiles, RankingOptions};

/// Coarse cardinality class. Survives small data drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CardinalityClass {
    /// 0 distinct values (constant or empty)
    Constant,
    /// Few distinct values (categorical)
    Low,
    /// Many distinct values (likely key)
    High,
}

impl CardinalityClass {
    fn from_ratio(ratio: f64) -> Self {
        match ratio {
            0.0 => CardinalityClass::Constant,
            r if r < 0.5 => CardinalityClass::Low,
            _ => CardinalityClass::High,
        }
    }
}

/// The topology of one column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnTopology {
    pub name: String,
    pub type_hint: TypeHint,
    pub cardinality_class: CardinalityClass,
    /// Distinct / non-null ratio, rounded to 2 decimals for stability.
    pub uniqueness_ratio: f64,
    /// Null percentage, rounded to 1 decimal.
    pub null_pct: f64,
    /// True if this column uniquely identifies rows (candidate key).
    pub is_key: bool,
    /// Whether the column is a key vs value (semantic role).
    pub col_type: Option<ColumnType>,
}

/// The topology signature of a whole table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologySignature {
    pub row_count: usize,
    pub columns: Vec<ColumnTopology>,
}

impl TopologySignature {
    /// Derive the topology from raw headers + rows.
    pub fn derive(headers: &[String], rows: &[Vec<String>]) -> Self {
        let profiles = compute_profiles(headers, rows, RankingOptions::default())
            .expect("profile computation on well-formed data");

        let n = rows.len();
        let columns: Vec<ColumnTopology> = headers
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let p = profiles.get(i).cloned().unwrap_or_else(|| {
                    // Column has no data (empty table) — constant, unknown.
                    rsf::ColumnProfile {
                        total_rows: n,
                        null_count: n,
                        cardinality: 0,
                        is_constant: true,
                        uniqueness_ratio: 0.0,
                        type_hint: TypeHint::Unknown,
                    }
                });
                let non_null = n.saturating_sub(p.null_count);
                let ratio = if non_null > 0 {
                    p.cardinality as f64 / non_null as f64
                } else {
                    0.0
                };
                let is_key = ratio >= 0.99 && p.cardinality > 1;
                let col_type = if is_key {
                    Some(ColumnType::Key)
                } else {
                    Some(ColumnType::Value)
                };
                ColumnTopology {
                    name: name.clone(),
                    type_hint: p.type_hint.clone(),
                    cardinality_class: CardinalityClass::from_ratio(ratio),
                    uniqueness_ratio: (ratio * 100.0).round() / 100.0,
                    null_pct: (p.null_pct().round() * 10.0) / 10.0,
                    is_key,
                    col_type,
                }
            })
            .collect();

        Self {
            row_count: n,
            columns,
        }
    }

    /// Look up a column's topology by name.
    pub fn column(&self, name: &str) -> Option<&ColumnTopology> {
        self.columns.iter().find(|c| c.name == name)
    }
}

/// A delta between two topology signatures — the rebase hint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyDelta {
    pub columns_added: Vec<String>,
    pub columns_removed: Vec<String>,
    /// Columns whose type class changed: (name, from, to)
    pub type_changes: Vec<(String, TypeHint, TypeHint)>,
    /// Columns whose cardinality class changed: (name, from, to)
    pub cardinality_changes: Vec<(String, CardinalityClass, CardinalityClass)>,
    /// Columns that gained/lost key-ness: (name, was_key, is_key)
    pub key_changes: Vec<(String, bool, bool)>,
    pub row_count_from: usize,
    pub row_count_to: usize,
}

impl TopologyDelta {
    /// Compute the delta from `from` to `to`.
    pub fn compute(from: &TopologySignature, to: &TopologySignature) -> Self {
        let from_names: HashMap<&str, &ColumnTopology> =
            from.columns.iter().map(|c| (c.name.as_str(), c)).collect();
        let to_names: HashMap<&str, &ColumnTopology> =
            to.columns.iter().map(|c| (c.name.as_str(), c)).collect();

        let columns_added = to.columns
            .iter()
            .filter(|c| !from_names.contains_key(c.name.as_str()))
            .map(|c| c.name.clone())
            .collect();
        let columns_removed = from.columns
            .iter()
            .filter(|c| !to_names.contains_key(c.name.as_str()))
            .map(|c| c.name.clone())
            .collect();

        let mut type_changes = Vec::new();
        let mut cardinality_changes = Vec::new();
        let mut key_changes = Vec::new();

        for fc in &from.columns {
            if let Some(tc) = to_names.get(fc.name.as_str()) {
                if fc.type_hint != tc.type_hint {
                    type_changes.push((fc.name.clone(), fc.type_hint.clone(), tc.type_hint.clone()));
                }
                if fc.cardinality_class != tc.cardinality_class {
                    cardinality_changes.push((
                        fc.name.clone(),
                        fc.cardinality_class,
                        tc.cardinality_class,
                    ));
                }
                if fc.is_key != tc.is_key {
                    key_changes.push((fc.name.clone(), fc.is_key, tc.is_key));
                }
            }
        }

        Self {
            columns_added,
            columns_removed,
            type_changes,
            cardinality_changes,
            key_changes,
            row_count_from: from.row_count,
            row_count_to: to.row_count,
        }
    }

    /// True if the two signatures are topologically identical.
    pub fn is_empty(&self) -> bool {
        self.columns_added.is_empty()
            && self.columns_removed.is_empty()
            && self.type_changes.is_empty()
            && self.cardinality_changes.is_empty()
            && self.key_changes.is_empty()
            && self.row_count_from == self.row_count_to
    }

    /// Human-readable summary for rebase reporting.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if !self.columns_added.is_empty() {
            parts.push(format!("columns added: {}", self.columns_added.join(", ")));
        }
        if !self.columns_removed.is_empty() {
            parts.push(format!("columns removed: {}", self.columns_removed.join(", ")));
        }
        for (name, from, to) in &self.type_changes {
            parts.push(format!("type changed: {} {:?} -> {:?}", name, from, to));
        }
        for (name, from, to) in &self.cardinality_changes {
            parts.push(format!("cardinality changed: {} {:?} -> {:?}", name, from, to));
        }
        for (name, was, is) in &self.key_changes {
            parts.push(format!("key-ness changed: {} {} -> {}", name, was, is));
        }
        if self.row_count_from != self.row_count_to {
            parts.push(format!("rows: {} -> {}", self.row_count_from, self.row_count_to));
        }
        if parts.is_empty() {
            "topology unchanged".to_string()
        } else {
            parts.join("; ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(headers: &[&str], rows: &[Vec<&str>]) -> (Vec<String>, Vec<Vec<String>>) {
        let h: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        let r: Vec<Vec<String>> =
            rows.iter().map(|row| row.iter().map(|s| s.to_string()).collect()).collect();
        (h, r)
    }

    #[test]
    fn test_derive_basic() {
        let (h, r) = table(
            &["id", "name", "amount"],
            &[
                vec!["1", "Alice", "$100"],
                vec!["2", "Bob", "$200"],
                vec!["3", "Carol", "$300"],
            ],
        );
        let sig = TopologySignature::derive(&h, &r);
        assert_eq!(sig.row_count, 3);
        assert_eq!(sig.columns.len(), 3);
        let id = sig.column("id").unwrap();
        assert!(id.is_key);
        assert_eq!(id.col_type, Some(ColumnType::Key));
    }

    #[test]
    fn test_delta_detects_type_change() {
        let (h1, r1) = table(
            &["amount"],
            &[vec!["$100"], vec!["$200"]],
        );
        let (h2, r2) = table(
            &["amount"],
            &[vec!["100"], vec!["200"]],
        );
        let s1 = TopologySignature::derive(&h1, &r1);
        let s2 = TopologySignature::derive(&h2, &r2);
        let delta = TopologyDelta::compute(&s1, &s2);
        assert!(
            delta.type_changes.iter().any(|(n, _, _)| n == "amount"),
            "expected type change on amount, got {:?}",
            delta.type_changes
        );
    }

    #[test]
    fn test_delta_detects_added_column() {
        let (h1, r1) = table(&["id"], &[vec!["1"], vec!["2"]]);
        let (h2, r2) = table(&["id", "extra"], &[vec!["1", "x"], vec!["2", "y"]]);
        let s1 = TopologySignature::derive(&h1, &r1);
        let s2 = TopologySignature::derive(&h2, &r2);
        let delta = TopologyDelta::compute(&s1, &s2);
        assert_eq!(delta.columns_added, vec!["extra".to_string()]);
    }

    #[test]
    fn test_delta_empty_when_identical() {
        let (h, r) = table(&["id", "name"], &[vec!["1", "A"], vec!["2", "B"]]);
        let s1 = TopologySignature::derive(&h, &r);
        let s2 = TopologySignature::derive(&h, &r);
        let delta = TopologyDelta::compute(&s1, &s2);
        assert!(delta.is_empty());
    }
}
