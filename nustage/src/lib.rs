//! Nustage — the pipeline engine.
//!
//! The transformation chain is the versioned object. Data files are
//! receipts (final state); a `Pipeline` is the route through the data
//! landscape, and the `PipelineLog` is the ledger of every step applied
//! with its topology before and after.
//!
//! - [`pipeline`]: `Step` / `Pipeline` model, TOML sidecar round-trip.
//! - [`topology`]: invariants of a table that survive deformation.
//! - [`executor`]: applies a pipeline to an `rsf::TypedTable`.
//! - [`ledger`]: `TransactionRecord` and `PipelineLog`.
//! - [`rebase`]: replay against drifted input; report where the route breaks.

pub mod executor;
pub mod ledger;
pub mod pipeline;
pub mod rebase;
pub mod topology;
