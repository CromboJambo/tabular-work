// Git-Sheets History Manager (Stub)
// Snapshots of table state, diff computation, integrity verification via SHA-256.

use chrono::DateTime;

pub struct Snapshot<Tz: chrono::TimeZone> {
    pub id: String,
    pub timestamp: DateTime<Tz>,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct TableState {
    pub headers: Vec<String>,
    pub primary_key: Vec<usize>,
}

// Can snapshot: Any table state including pipeline outputs. Optional in workflow.