//! Rebase — replay a pipeline against drifted input, and report where the
//! route breaks.
//!
//! When the input schema drifts (a column renamed, a type shifted), the
//! topology tells you *what's still the same* and *what actually changed*.
//! Rebase is: replay the pipeline against the new input, detect which step
//! broke, and surface the topology delta so the user can patch the step
//! rather than the data. This is `git rebase` for the data landscape.

use rsf::{RankingOptions, TypedTable, compute_profiles};

use crate::executor::{StepError, apply_step, run_pipeline};
use crate::ledger::PipelineLog;
use crate::pipeline::Pipeline;
use crate::topology::{TopologyDelta, TopologySignature};

/// Outcome of a rebase attempt.
#[derive(Debug, Clone)]
pub enum RebaseOutcome {
    /// The pipeline replayed cleanly against the new input.
    Clean {
        log: PipelineLog,
        /// Topology delta between the original input and the new input.
        input_delta: TopologyDelta,
    },
    /// A step failed to apply against the new input.
    Conflict {
        /// Index of the step that failed.
        step_index: usize,
        /// The step that failed.
        step: crate::pipeline::Step,
        /// Why it failed.
        error: StepError,
        /// Topology of the table at the point of failure.
        at_failure: TopologySignature,
        /// Topology delta between the original input and the new input.
        input_delta: TopologyDelta,
    },
}

/// Compute the topology of a table.
fn topology_of(table: &TypedTable) -> TopologySignature {
    let headers: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
    let raw: Vec<Vec<String>> =
        table.rows.iter().map(|r| r.iter().map(|v| v.as_str()).collect()).collect();
    TopologySignature::derive(&headers, &raw)
}

/// Build a `TypedTable` from raw headers + rows, inferring type profiles.
pub fn build_typed(headers: &[String], rows: &[Vec<String>]) -> TypedTable {
    let profiles = compute_profiles(headers, rows, RankingOptions::default())
        .expect("profile computation on well-formed data");
    TypedTable::from_untyped(headers, rows, &profiles)
}

/// Replay `pipeline` against `new_headers`/`new_rows`.
///
/// `original_headers`/`original_rows` are the input the pipeline was authored
/// against; they're used to compute the input topology delta (the rebase hint).
pub fn rebase(
    pipeline: &Pipeline,
    original_headers: &[String],
    original_rows: &[Vec<String>],
    new_headers: &[String],
    new_rows: &[Vec<String>],
) -> RebaseOutcome {
    let orig_sig = TopologySignature::derive(original_headers, original_rows);
    let new_sig = TopologySignature::derive(new_headers, new_rows);
    let input_delta = TopologyDelta::compute(&orig_sig, &new_sig);

    let new_table = build_typed(new_headers, new_rows);
    let initial_topology = topology_of(&new_table);

    match run_pipeline(&new_table, &pipeline.steps) {
        Ok((final_table, results)) => {
            let final_topology = topology_of(&final_table);
            let log = PipelineLog::from_execution(pipeline, &initial_topology, &results, &final_topology);
            RebaseOutcome::Clean { log, input_delta }
        }
        Err((step_index, error)) => {
            // Replay up to (not including) the failing step to capture the
            // topology at the point of failure.
            let at_failure = replay_to_failure(&new_table, &pipeline.steps, step_index);
            let step = pipeline.steps[step_index].clone();
            RebaseOutcome::Conflict {
                step_index,
                step,
                error,
                at_failure,
                input_delta,
            }
        }
    }
}

/// Replay steps 0..step_index and return the topology at that point.
fn replay_to_failure(table: &TypedTable, steps: &[crate::pipeline::Step], step_index: usize) -> TopologySignature {
    let mut current = table.clone();
    for (_i, step) in steps.iter().enumerate().take(step_index) {
        match apply_step(&current, step) {
            Ok(res) => current = res.table,
            Err(_) => break,
        }
    }
    topology_of(&current)
}

/// Human-readable rebase report.
pub fn describe(outcome: &RebaseOutcome) -> String {
    match outcome {
        RebaseOutcome::Clean { input_delta, .. } => {
            let mut s = String::from("REBASE CLEAN\n");
            s.push_str(&format!("  input topology: {}\n", input_delta.describe()));
            s
        }
        RebaseOutcome::Conflict {
            step_index,
            step,
            error,
            input_delta,
            ..
        } => {
            let mut s = String::from("REBASE CONFLICT\n");
            s.push_str(&format!("  step {step_index} failed: {step:?}\n"));
            s.push_str(&format!("  error: {error}\n"));
            s.push_str(&format!("  input topology: {}\n", input_delta.describe()));
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(headers: &[&str], rows: &[Vec<&str>]) -> (Vec<String>, Vec<Vec<String>>) {
        let h: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        let r: Vec<Vec<String>> =
            rows.iter().map(|row| row.iter().map(|s| s.to_string()).collect()).collect();
        (h, r)
    }

    #[test]
    fn test_rebase_clean_on_identical_input() {
        let (h, r) = t(
            &["id", "amount"],
            &[vec!["1", "$100"], vec!["2", "$200"], vec!["3", "$300"]],
        );
        let mut p = Pipeline::new("test");
        p.push(crate::pipeline::Step::FilterRows {
            condition: "amount > 150".to_string(),
        });
        let outcome = rebase(&p, &h, &r, &h, &r);
        match outcome {
            RebaseOutcome::Clean { input_delta, .. } => assert!(input_delta.is_empty()),
            RebaseOutcome::Conflict { .. } => panic!("expected clean"),
        }
    }

    #[test]
    fn test_rebase_conflict_on_removed_column() {
        // Original input has 'amount'; new input dropped it.
        let (oh, or) = t(
            &["id", "amount"],
            &[vec!["1", "$100"], vec!["2", "$200"]],
        );
        let (nh, nr) = t(&["id"], &[vec!["1"], vec!["2"]]);
        let mut p = Pipeline::new("test");
        p.push(crate::pipeline::Step::FilterRows {
            condition: "amount > 150".to_string(),
        });
        let outcome = rebase(&p, &oh, &or, &nh, &nr);
        match outcome {
            RebaseOutcome::Conflict { step_index, input_delta, .. } => {
                assert_eq!(step_index, 0);
                assert!(
                    input_delta.columns_removed.iter().any(|c| c == "amount"),
                    "expected 'amount' in removed, got {:?}",
                    input_delta.columns_removed
                );
            }
            RebaseOutcome::Clean { .. } => panic!("expected conflict"),
        }
    }

    #[test]
    fn test_rebase_detects_type_drift_in_delta() {
        // amount column loses its $ symbols → type drifts Currency → Float/Unknown.
        let (oh, or) = t(&["id", "amount"], &[vec!["1", "$100"], vec!["2", "$200"]]);
        let (nh, nr) = t(&["id", "amount"], &[vec!["1", "100"], vec!["2", "200"]]);
        let p = Pipeline::new("empty");
        let outcome = rebase(&p, &oh, &or, &nh, &nr);
        match outcome {
            RebaseOutcome::Clean { input_delta, .. } => {
                assert!(
                    input_delta.type_changes.iter().any(|(n, _, _)| n == "amount"),
                    "expected type change on amount, got {:?}",
                    input_delta.type_changes
                );
            }
            RebaseOutcome::Conflict { .. } => panic!("expected clean for empty pipeline"),
        }
    }
}
