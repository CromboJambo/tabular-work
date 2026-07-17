# Tabular Work Consolidated Roadmap (July 2026)

## Architecture Summary (unchanged from docs)

Three distinct crates under one workspace: `nustage`, `zed-sheet-lsp`, `git-sheets`. Each maintains its identity but shares ONE repo with unified cargo.toml.

### Pipeline Intent Engine (`nustage`)
- Canonical pipeline model via TOML config
- Sidecar management (.nustage.json) as single source of truth
- Transformation execution

### Editor Integration (`zed-sheet-lsp`)
- Hover, completion, diagnostics for tabular data
- Pipeline-aware code actions
- Zed-specific LSP protocol integration

### History Manager (`git-sheets`)
- Snapshots of table state via SHA-256 hashing
- Diff computation before/after comparison
- Immutable version control records (no Excel-style "version2finalFinalFINAL.csv")

## Dependencies

```
zed-sheet-lsp ────────┐
                      ├──→ nustage (library dependency)
git-sheets ───────────┘
nustage ────────────── (no dependencies on other two)
git-sheets ─────────── (no dependencies on other two)
```

**Key rule:** `nustage` and `git-sheets` have no mutual dependencies. They compose at the workflow level via shared file formats (sidecar, snapshots).

## Current Status

✅ Documentation transferred into tabular-work/git-sheets/docs  
✅ git-sheets sub-crates moved under workspace  
✅ nustage artifacts (zellij-tile config) moved under workspace  
⚠️ zed-sheet-lsp code still needs to be created (the JSON fixture was stale, no actual LSP repo exists)

## Next Steps (Prioritized)

### Priority 1: Core Stack Completion
| Task | Owner | Notes |
|------|-------|-------|
| Create src/zed-sheet-lsp with real LSP code | zed-sheet-lsp | Replace stale JSON fixture with actual implementation |
| Add cargo test --workspace integration tests | Workspace | Integration audit via docs/integration/AUDIT.md |
| Document workflows under tabular-work/docs/workflows.md | Workspace | Practical CLI commands + example scripts |

### Priority 2: Versioning Improvements (via git-sheets)
| Task | Owner | Notes |
|------|-------|-------|
| Replace Excel-style naming with SHA-256 snapshots | git-sheets | No "version2finalFinalFINAL.csv" → `snapshots/<name>_<timestamp>.toml` |
| Add CLI commands that compose all three | Workspace | `git-sheets snapshot`, `nustage process`, full stack examples |
| Create example scripts showing full stack usage | git-sheets/docs/examples | Use existing script + expand coverage |

### Priority 3: Data Quality Pipeline (via rsf-cli)
| Task | Owner | Notes |
|------|-------|-------|
| Handle complex Excel exports natively | rsf-cli/scripts | Phase 6 completed via docs/integration/ |
| CSV parsing with dynamic delimiters | rsf-cli/src/lib.rs | Refactor implementation |
| Rank column cardinality-based ordering | rsf-cli/src/lib.rs | Semantic model logic |

## Versioning Architecture (via git-sheets)

**Anti-Pattern to Avoid:** Excel-style file naming like "version2finalFinalFINAL.csv"

**Good Pattern:** `git-sheets` snapshots with immutable state records:
```bash
git-sheets snapshot data.csv -m "Initial import"
git-sheets snapshot modified_data.csv -m "After bulk update batch"
git-sheets diff snap1.toml snap2.toml  # See what changed
```

**Format:** `snapshots/<name>_<timestamp>.toml` with:
- SHA-256 table hash for integrity verification
- Timestamp as canonical reference (no human "final-final-final")
- Diff computation via git-sheets CLI shows actual changes

## Testing Independence

### Per-Repo Unit Tests
Each repo should have tests that work independently:
- `nustage`: Pipeline validation, sidecar serialization
- `zed-sheet-lsp`: LSP protocol handling, diagnostic generation
- `git-sheets`: Snapshot creation, diff computation

### Cross-Repo Integration Tests (Future)
Run from workspace root or CI:
```bash
cargo test --workspace sidecar_integration
cargo test --workspace snapshot_pipeline_output
```

## Feature Placement Checklist

When adding a new feature, ask:
1. **Is this defining transformation semantics?** → `nustage`
2. **Is this about editor UX (hover, completion, rename)?** → `zed-sheet-lsp`
3. **Is this about recording history or comparing states?** → `git-sheets`
4. **Does this create duplicate truth?** → Redesign needed

## Migration Phase Timeline

### Phase 1: Clarify Boundaries (Current)
- ✅ Documentation exists (`STACK_BOUNDARIES.md`)
- ✅ Mental model established
- ⚠️ Some code still overlaps

### Phase 2: Library Extraction
- [ ] Extract `nustage` pipeline types into dedicated lib crate
- [ ] Have `zed-sheet-lsp` depend on extracted types (not copy)
- [ ] Ensure `git-sheets` has no accidental dependencies

### Phase 3: Workflow Integration
- [ ] Add CLI commands that compose all three
- [ ] Document recommended workflows in user guides
- [ ] Create example scripts showing full stack usage

## Related Documents

| Document | Purpose | Location |
----------|---------|----------
| Full Integration Guide | Complete architecture overview | `src/git-sheets/docs/README.md` |
| Type Audit & Migration | Duplicate type analysis | `src/git-sheets/docs/AUDIT.md` |
| Dependency Config | Cargo workspace strategy | `src/git-sheets/docs/dependency-config.md` |
| Example Workflows | Practical scripts | `src/git-sheets/docs/examples/full_stack_daily_development.sh` |

## Questions This Architecture Answers

**Q: Can I use just git-sheets for Excel version control?**  
A: Yes. No other projects required.

**Q: Can I use nustage without Zed editor?**  
A: Yes. CLI and TUI work standalone.

**Q: What if I want pipeline-aware editing but no history tracking?**  
A: Use `nustage` + `zed-sheet-lsp`. git-sheets is optional.

**Q: How do changes in one repo affect others?**  
A: Carefully defined via library dependencies and file formats (sidecar, snapshot). See "Dependency Directions" above.

**Q: Can I swap out components later?**  
A: Yes, as long as the contracts (sidecar format, snapshot format) remain stable.