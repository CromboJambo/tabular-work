# Nustage (Pipeline Engine)

This sub-crate under tabular-work contains the canonical pipeline model for spreadsheet transformations.

**Role:** 
- Pipeline definition semantics (step_id, type, column, condition, expr)
- Sidecar management (.nustage.json format as single source of truth)
- Transformation execution (add-column, filter-rows, group-by, rename, sort, dedup)

**Zellij tile config included:** nustage_tile.toml from original repo defines keybindings/colors/status_bar for interactive TUI.

**Dependency status:** No dependencies on other two (zed-sheet-lsp, git-sheets) — compose at workflow level only.