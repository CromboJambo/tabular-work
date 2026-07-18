// Nustage Pipeline Engine (Stub)
// Canonical transformation model, sidecar management (.nustage.json), execution

pub struct PipelineStep {
    pub step_id: String,
    pub r#type: StepType,
    pub column: Option<String>,
    pub condition: Option<String>,
    pub expr: Option<String>,
}

pub enum StepType {
    FilterRows,
    AddColumn,
    GroupBy,
    RenameColumn,
    SelectColumns,
    DropColumns,
    SortBy,
    RemoveDuplicates,
}

// No dependencies on other two (zed-sheet-lsp, git-sheets) — compose at workflow level only