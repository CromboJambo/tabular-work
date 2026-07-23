// Zed-Sheet-Lsp LSP Integration (Stub)
// Hover, completion, diagnostics for tabular data in Zed editor. Pipeline-aware code actions based on .nustage.json sidecar.

/// Represents a location in a file (line + column).
#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

pub struct LspDiagnostic {
    pub location: Location,
    pub severity: Severity,
    pub message: String,
}

pub enum Severity {
    Error,
    Warning,
    Info,
}

// Depends on nustage library (see Cargo.toml)