# Spreadsheet Application Architecture Plan

## Existing Components (tabular-work)

We already have significant building blocks:

| Component | Status | What it provides |
|-----------|--------|-----------------|
| `spreadsheet-format` | Prototype | Binary format with compiled formula opcodes, dependency graphs, serialization |
| `spreadsheet-turso` | Working | SQLite-backed storage using libSQL, MVCC, async I/O |
| `rsf-core/src/expr/` | Working | Expression parser (lexer/parser/evaluator) for table filtering |
| `git-sheets` | Published | Version control for spreadsheets |
| `nustage` | Working | Pipeline engine |

## Architecture: Rust + SQLite Backend, Web/Desktop UI

```
┌─────────────────────────────────────────────────────────────┐
│                    UI Layer (Web/Desktop)                   │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │   Web App        │  │  Desktop App     │                │
│  │   (Svelte/React) │  │  (Tauri/GTK)     │                │
│  └────────┬─────────┘  └────────┬─────────┘                │
├───────────┼──────────────────────┼──────────────────────────┤
│           │          Rust Core API (via FFI/HTTP)            │
├───────────┼──────────────────────┼──────────────────────────┤
│                    Formula Engine                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Tokenizer (pesti structural tokenizer)               │   │
│  │ Parser → AST → Compiled Opcodes                     │   │
│  │ Dependency Graph + Topological Sort                  │   │
│  │ Evaluation Engine (cell references, ranges)          │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Storage Layer                            │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ SQLite Backend   │  │ .xlsx Compat     │                │
│  │ (libSQL/Turso)   │  │ (calamine crate) │                │
│  │ MVCC, async I/O  │  │ Read/write       │                │
│  └──────────────────┘  └──────────────────┘                │
├─────────────────────────────────────────────────────────────┤
│                    Version Control                          │
│  ┌──────────────────┐                                      │
│  │ git-sheets       │ Staging, commits, diffs for tables   │
│  └──────────────────┘                                      │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: Formula Engine (Foundation)

**Goal:** Build a robust formula parser and evaluator that understands Excel-style formulas with cell references.

### Components:

1. **Formula Tokenizer** (pesti structural tokenizer integration)
   - Tokenize `=SUM(A1:A10)` into structural tokens
   - Handle cell refs, ranges, operators, function names, literals
   - Leverage pesti's proven approach for domain-specific languages

2. **Parser → AST**
   - Recursive descent parser building expression trees
   - Support: arithmetic (+,-,*,/), comparisons (<,>,=,<>,<=,>=), logical (AND,OR,NOT)
   - Cell reference resolution (A1 notation, sheet references like Sheet2!A1)

3. **Compiled Opcodes** (build on existing spreadsheet-format work)
   - Transform AST into compact bytecode for fast evaluation
   - Existing opcodes: SUM, AVERAGE, COUNT, MAX, MIN, CONCAT
   - Add: IF, VLOOKUP, HLOOKUP, INDEX, MATCH, TEXT functions

4. **Dependency Graph & Evaluation** (build on existing work)
   - Track cell dependencies for incremental recalculation
   - Topological sort determines evaluation order
   - Detect and handle circular references

### Excel Functions to Implement (Priority Order):

**Tier 1 - Essential:**
- SUM, AVERAGE, COUNT, MAX, MIN
- IF, AND, OR, NOT
- CONCATENATE/CONCAT, LEFT, RIGHT, MID, LEN
- ROUND, ABS, INT, SQRT, POWER

**Tier 2 - Common (Modern Lookup Approach):**
- XLOOKUP (primary lookup function - replaces VLOOKUP/HLOOKUP)
- INDEX/MATCH (legacy support for older spreadsheets)
- SUMIF, COUNTIF, AVERAGEIF
- NOW, TODAY, DATE functions
- TRIM, UPPER, LOWER, SUBSTITUTE, FIND

**Tier 3 - Advanced:**
- OFFSET, INDIRECT (dynamic ranges)
- ARRAY formulas
- Statistical functions
- Financial functions

## Power Query Equivalent: Data Transformation Pipeline

Build on existing `nustage` pipeline engine for Excel's Power Query functionality.

### Nustage Design Principle (2026-10-24)

**No explicit cell referencing.** Nustage is pipeline-first, not grid-first. References are to fields and positions relative to nearest-neighbor relationships, not fixed A1/B2 coordinates. This avoids Excel's overextension as business infrastructure — the reason Excel became so bloated is precisely because it requires fixed positions for values that have relationships.

Key differences from Excel-style cell referencing:
- **Field names** over cell addresses (e.g., `row.revenue` not `B5`)
- **Positional/relative references** over absolute coordinates
- **Path-dependent data flow** — transformations build on previous stages
- **Open binary format** to SQLite that the user owns and controls
- **Source of truth alignment** — anyone rehydrating the work months later can understand the pipeline

The formula engine (Phase 1) handles Excel-compatible cell referencing for import/export compatibility. Nustage's native pipeline language uses field/position-based references throughout.

**Core Capabilities:**
- Import data from multiple sources (CSV, JSON, SQLite, web APIs)
- Transform operations: filter rows, select/rename columns, split/merge columns
- Type conversions and cleaning (trim whitespace, handle nulls)
- Joins (inner, left, right, full outer) on multiple keys
- Aggregations (group by with sum, count, average)
- Pivot/unpivot operations

**Architecture:**
```rust
let query = Pipeline::new()
    .source(CsvSource("sales.csv"))
    .filter(row => row["region"] == "East")
    .select(["product", "revenue", "date"])
    .group_by(["product"], {
        revenue: sum("revenue"),
        count: count("*")
    })
    .sort(desc("revenue"))
    .to_table();
```

## Pivot Table Engine

**Data Model:**
- Source table with row/column/value field designations
- Aggregation functions per value field (sum, count, average, etc.)
- Hierarchical drill-down support

**Implementation Strategy:**
- Build on top of Power Query pipeline for data preparation
- Use SQLite GROUP BY for efficient aggregation at scale
- Cache pivot results and invalidate on source data changes

```rust
let pivot = PivotTable::new(source_table)
    .rows(["region", "product_category"])
    .columns(["quarter"])
    .values(sum("revenue"), count("order_id"))
    .compute();
```

## Phase 2: .xlsx Compatibility

**Goal:** Read and write Excel files for real-world usability.

### Strategy: Use calamine crate for reading, build writer

- **Reading:** `calamine` crate already provides robust .xlsx parsing
- **Writing:** Build custom writer or use existing Rust xlsx libraries
- **Conversion Layer:** Map our internal binary format ↔ .xlsx format

### Integration Plan:
```rust
// Load .xlsx file into our internal format
let workbook = calamine::open_workbook("input.xlsx")?;
let sheet_range = workbook.range("Sheet1")?;
let our_spreadsheet = convert_calamine_to_ours(sheet_range);

// Save as .xlsx from our internal format
let xlsx_output = convert_ours_to_calamine(&our_spreadsheet);
```

## Phase 3: Storage Engine (Build on spreadsheet-turso)

**Goal:** Production-ready SQLite storage with MVCC and async I/O.

### Enhancements to existing spreadsheet-turso:

1. **Schema Optimization**
   - Add formulas table separate from values
   - Index optimization for cell lookups
   - Transaction support for atomic operations

2. **MVCC Implementation**
   - Version tracking for each cell
   - Concurrent read/write support
   - Undo/redo capability via version history

3. **Async I/O**
   - Non-blocking reads/writes using libSQL's async API
   - Batch operations optimized for network latency

## Phase 4: UI Layer

### Web/Desktop Application (Primary Target)

**Tech Stack:** Tauri + React/Svelte frontend with Rust backend API

- Grid component with virtual scrolling (thousands of cells)
- Cell selection, copy/paste, drag-fill
- Formula bar showing/editing cell formulas
- Sheet tabs for multiple sheets
- Basic charting library integration

**Architecture Decision (2026-10-24):** Tauri chosen over GPUI and WASM approaches based on:
- Cross-platform support including Windows
- Mature ecosystem with spreadsheet components (AG Grid, Handsontable)
- Proven IPC bridge for low-latency formula evaluation
- Already in crabjar dependency tree (proven integration path)

### Desktop Application (Secondary)

**Option A: Tauri** - Rust backend + web frontend in native window (chosen)
**Option B: GTK/Rust** - Pure Rust desktop app (more work, more performant)

## Phase 5: Version Control Integration

Build on existing git-sheets work:
- Staging area for spreadsheet changes
- Commit messages with diff summaries
- Branch and merge support for collaborative editing

## Leveraging Our Tools

### REA Plugin
- Reverse engineer Excel's .xlsx binary format
- Analyze real-world spreadsheets to understand common patterns
- Study LibreOffice Calc's formula engine implementation

### pesti Structural Tokenizer
- Formula parsing into structural tokens
- More robust than regex-based approaches
- Enable AI-assisted formula writing ("write a formula to sum sales where region is East")

### crabjar Analyzer Tools
- Analyze LibreOffice Calc source code architecture
- Identify patterns worth adapting
- Compare implementation approaches

## Development Roadmap

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | Formula tokenizer + parser | Parse Excel formulas into AST |
| 3-4 | Evaluation engine + dependency graph | Evaluate formulas with cell refs |
| 5-6 | Tier 1 Excel functions | SUM, IF, VLOOKUP, etc. |
| 7-8 | .xlsx read compatibility | Open real Excel files |
| 9-10 | SQLite storage optimization | Production-ready backend |
| 11-12 | Web UI prototype | Functional spreadsheet interface |

## Key Design Decisions

1. **SQLite over custom binary format** - proven, embeddable, MVCC support
2. **Compiled opcodes for formulas** - faster than re-parsing on every evaluation
3. **Dependency graph** - enables incremental recalculation (only update affected cells)
4. **Rust core with language-agnostic API** - allows multiple UI implementations

## Next Steps

1. Start formula tokenizer using pesti approach
2. Build parser that handles cell references and ranges
3. Implement basic evaluation with dependency tracking
4. Add Tier 1 Excel functions incrementally