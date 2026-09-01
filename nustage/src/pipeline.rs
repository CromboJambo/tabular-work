//! Pipeline model — the transformation DAG that *is* the versioned object.
//!
//! A `Pipeline` is an ordered list of `Step`s. It serializes to the
//! `.nustage.toml` sidecar (the "commit"). Data files are receipts; the
//! pipeline is the route through the data landscape.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single transformation operation. The atom that gets committed,
/// branched, and rebased.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Step {
    /// Keep rows where `condition` (an rsf expression) is true.
    FilterRows {
        condition: String,
    },
    /// Add a new column whose value is `expr` evaluated per row.
    AddColumn {
        name: String,
        expr: String,
    },
    /// Group rows by `group_cols`, aggregating `value_col` with `agg`.
    GroupBy {
        group_cols: Vec<String>,
        value_col: String,
        agg: String,
    },
    /// Rename a column.
    RenameColumn {
        from: String,
        to: String,
    },
    /// Keep only the named columns (in the given order).
    SelectColumns {
        columns: Vec<String>,
    },
    /// Drop the named columns.
    DropColumns {
        columns: Vec<String>,
    },
    /// Sort rows by a column.
    SortBy {
        column: String,
        #[serde(default)]
        desc: bool,
    },
    /// Remove duplicate rows (exact match on all columns, first kept).
    RemoveDuplicates,
}

/// An ordered transformation pipeline — the versioned object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub steps: Vec<Step>,
}

impl Pipeline {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    pub fn push(&mut self, step: Step) -> &mut Self {
        self.steps.push(step);
        self
    }

    /// Serialize to a TOML sidecar string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Parse from a TOML sidecar string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Write the sidecar to `path`.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_toml().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?)
    }

    /// Load a sidecar from `path`.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path)?;
        Ok(Self::from_toml(&s)?)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new("pipeline")
    }
}
