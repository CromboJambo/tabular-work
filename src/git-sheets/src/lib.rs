// git-sheets: Version control for spreadsheets
// Snapshots of table state, diff computation, integrity verification via SHA-256.

pub mod cli;
pub mod core;
pub mod diff;

pub use core::{GitSheetsError, GitSheetsRepo, Result, Snapshot, Table, TableHashes};
pub use diff::{Change, DiffSummary, SnapshotDiff};
