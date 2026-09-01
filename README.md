# Tabular Work Consolidated Stack

Three distinct crates under one workspace: **nustage**, **zed-sheet-lsp**, **git-sheets**. Each maintains its identity but shares ONE repo with unified cargo.toml.

## Architecture (unchanged from docs)

```
┌─────────────────────────────────────────────────────────────┐
│                    Zed Editor                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │           zed-sheet-lsp (Editor Integration)         │  │
│  │  - Hover, completion, diagnostics for tabular data   │  │
│  │  - Pipeline-aware code actions                       │  │
│  └───────────────────────┬───────────────────────────────┘  │
│                          │ LSP Protocol                     │
├──────────────────────────┼──────────────────────────────────┤
│                          ▼                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │           nustage (Intent Engine)                    │  │
│  │  - Canonical pipeline model                          │  │
│  │  - Sidecar management (.nustage.json)                │  │
│  │  - Transformation execution                          │  │
│  └───────────────────────┬───────────────────────────────┘  │
│                          │ File System                      │
├──────────────────────────┼──────────────────────────────────┤
│                          ▼                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │          git-sheets (History Manager)                │  │
│  │  - Snapshots of table state                          │  │
│  │  - Diff computation                                  │  │
│  │  - Integrity verification                            │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Individual Responsibilities (unchanged)

| Concern | Primary Owner | Secondary Consumers | Notes |
|---------|--------------|---------------------|-------|
| Pipeline definition semantics | `nustage` | - | Canonical transformation model |
| Sidecar format & persistence | `nustage` | `zed-sheet-lsp` (read-only) | `.nustage.json` is single source of truth |
| LSP protocol handling | `zed-sheet-lsp` | - | Zed-specific integration |
| Hover/completion/diagnostics | `zed-sheet-lsp` | `nustage` (provides data) | Surface `nustage` intent in editor |
| Snapshots & versioning | `git-sheets` | - | Immutable table state records |
| Diff computation | `git-sheets` | - | Before/after comparison |
| Integrity hashing | `git-sheets` | - | SHA-256 verification |

## Dependencies (unchanged)

```
zed-sheet-lsp ────────┐
                      ├──→ nustage (library dependency)
git-sheets ───────────┘

nustage ────────────── (no dependencies on other two)
git-sheets ─────────── (no dependencies on other two)
```

**Key rule:** `nustage` and `git-sheets` have no mutual dependencies. They compose at the workflow level.

## Workspace Members

- **src/nustage**: Pipeline engine (TOML config from zellij-tile, JSON fixtures removed as stale refs)
- **src/zed-sheet-lsp**: Zed LSP integration (will need to be created/filled with actual code)
- **src/git-sheets**: History manager (copied from original repo: cli, core, diff, hash libs)
- **rsf-core**: Semantic model for tabular data (pure library; the old rsf-cli TUI/CLI was abandoned and is being reworked here)

## Migration Status

✅ Documentation transferred into tabular-work/git-sheets/docs  
✅ git-sheets sub-crates moved under workspace  
✅ nustage artifacts (zellij-tile config) moved under workspace  
⚠️ zed-sheet-lsp code still needs to be created (the JSON fixture was stale, no actual LSP repo exists)

## Next Steps

1. Create src/zed-sheet-lsp with real LSP code
2. Add cargo test --workspace integration tests as described in docs/integration/AUDIT.md
3. Document workflows under tabular-work/docs/workflows.md

---
*Consolidated from git-sheets, nustage artifacts, rsf-core (rsf-cli abandoned) — July 2026*