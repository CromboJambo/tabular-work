// RSF CLI Representation/Analysis Companion (Stub)
// CSV parser + ranked spreadsheet format. Semantic model for tabular data with cardinality-based column ordering.

pub struct RSFColumn {
    pub name: String,
    pub rank: Rank,
}

pub enum Rank {
    Primary,
    Secondary,
    Optional,
}

// Companion to stack — optionally depends on nustage for pipeline-aware analysis.