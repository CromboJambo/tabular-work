// git-sheets: Version control for spreadsheets
// Snapshots of table state, diff computation, integrity verification via SHA-256.

pub mod cli;
pub mod core;
pub mod diff;
pub mod rsf_integration;
pub mod publish;
pub mod github;

pub use core::{GitSheetsError, GitSheetsRepo, Result, Snapshot, Table, TableHashes};
pub use diff::{Change, DiffSummary, SnapshotDiff};
pub use rsf_integration::*;
