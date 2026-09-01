//! The ledger — transaction records and the pipeline log.
//!
//! A `TransactionRecord` is one entry in the ledger: a step that was applied,
//! with the topology signature before and after. The `PipelineLog` is the
//! ordered sequence of records — the route through the data landscape. It is
//! strictly more informative than any single data state: you can replay it to
//! reconstruct any intermediate table, and you can diff two logs by comparing
//! their topology signatures without comparing data payloads.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::executor::StepResult;
use crate::pipeline::{Pipeline, Step};
use crate::topology::TopologySignature;

/// One entry in the ledger: a step that was applied, with the topology
/// before and after.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    /// Index of the step in the pipeline (0-based).
    pub step_index: usize,
    /// The step that was applied.
    pub step: Step,
    /// Topology of the table *before* this step.
    pub input_topology: TopologySignature,
    /// Topology of the table *after* this step.
    pub output_topology: TopologySignature,
    /// Rows in before the step.
    pub rows_in: usize,
    /// Rows out after the step.
    pub rows_out: usize,
    /// Human context (who/why/when). Optional.
    pub context: Option<String>,
}

impl TransactionRecord {
    /// Build a record from a step and its execution result.
    pub fn from_result(
        step_index: usize,
        step: &Step,
        input_topology: &TopologySignature,
        result: &StepResult,
    ) -> Self {
        Self {
            step_index,
            step: step.clone(),
            input_topology: input_topology.clone(),
            output_topology: result.topology.clone(),
            rows_in: result.rows_in,
            rows_out: result.rows_out,
            context: None,
        }
    }

    /// Set the human context for this record.
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }
}

/// The ordered ledger of transactions — the route through the data landscape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineLog {
    /// The pipeline (the versioned object) this log was produced from.
    pub pipeline: Pipeline,
    /// The ordered transaction records.
    pub records: Vec<TransactionRecord>,
    /// Topology of the final output table.
    pub final_topology: TopologySignature,
}

impl PipelineLog {
    /// Build a log from a pipeline and its execution.
    pub fn from_execution(
        pipeline: &Pipeline,
        initial_topology: &TopologySignature,
        results: &[StepResult],
        final_topology: &TopologySignature,
    ) -> Self {
        let mut records = Vec::with_capacity(results.len());
        let mut prev = initial_topology.clone();
        for (i, res) in results.iter().enumerate() {
            let step = &pipeline.steps[i];
            records.push(TransactionRecord::from_result(i, step, &prev, res));
            prev = res.topology.clone();
        }
        Self {
            pipeline: pipeline.clone(),
            records,
            final_topology: final_topology.clone(),
        }
    }

    /// Serialize the log to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse a log from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Write the log to `path`.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?)
    }

    /// Load a log from `path`.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&s)?)
    }

    /// Number of transactions in the log.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// True if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
