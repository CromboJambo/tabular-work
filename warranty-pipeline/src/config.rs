//! Configuration module for pipeline definitions

use serde::{Deserialize, Serialize};

/// Pipeline configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<PipelineStep>,
}

/// A single transformation step in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineStep {
    /// Load Excel sheet
    LoadExcel {
        source: String,
        sheet: String,
        output: String,
    },

    /// Load CSV file
    LoadCsv { source: String, output: String },

    /// Filter rows by condition
    Filter {
        input: String,
        output: String,
        condition: String,
    },

    /// Join two tables
    Join {
        left: String,
        right: String,
        on: Vec<String>,
        output: String,
        join_type: Option<JoinType>,
    },

    /// Pivot table
    Pivot {
        input: String,
        index_cols: Vec<String>,
        value_col: String,
        agg_func: AggFunction,
        output: String,
    },

    /// Group by columns and aggregate
    GroupBy {
        input: String,
        group_cols: Vec<String>,
        aggregations: Vec<Aggregation>,
        output: String,
    },

    /// Select specific columns
    Select {
        input: String,
        columns: Vec<String>,
        output: String,
    },

    /// Add calculated column
    Mutate {
        input: String,
        columns: Vec<ColumnDef>,
        output: String,
    },

    /// Sort rows
    Sort {
        input: String,
        by: Vec<SortSpec>,
        output: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub column: String,
    pub descending: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JoinType {
    #[serde(rename = "inner")]
    Inner,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "full")]
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggFunction {
    #[serde(rename = "sum")]
    Sum,
    #[serde(rename = "count")]
    Count,
    #[serde(rename = "avg")]
    Avg,
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "first")]
    First,
    #[serde(rename = "last")]
    Last,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregation {
    pub column: String,
    pub func: AggFunction,
    pub alias: Option<String>,
}
