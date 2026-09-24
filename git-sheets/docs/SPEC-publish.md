# Spec: `git-sheets publish` — Workflow Publication to GitHub

**Status:** Draft  
**Target command:** `git-sheets publish <pipeline>`

## Overview

Take an existing `.nustage.json` pipeline and package it as a portable, runnable workflow repository on GitHub. No real customer data — only the transformation logic, schema definitions, and synthetic example data.

**The moment this delivers on:**
> "You did this once. Do you want to own the thing you just did?"

## CLI Interface

```bash
# Basic: publish current directory's pipeline
git-sheets publish --title "Monthly Invoice Reconciliation" \
    --description "Filter, join, and aggregate AP data by vendor"

# Explicit pipeline file
git-sheets publish ./invoicing/.nustage.json \
    --title "AP Invoicing Workflow"

# Dry run: show what would be published
git-sheets publish --dry-run

# Specify repo name (default derived from title)
git-sheets publish --repo-name ap-invoice-workflow
```

### Required inputs
- Pipeline file (`.nustage.json`) — defaults to current directory
- `--title` — human-readable workflow name

### Optional inputs
- `--description` — longer description for README
- `--repo-name` — GitHub repo slug (default: title → kebab-case)
- `--dry-run` — show package contents without pushing
- `--org` / `--user` — target org/user (default: authenticated user)
- `--token` — GitHub PAT (falls back to GH_TOKEN env or credential helper)

## Published Repo Structure

```
ap-invoice-workflow/
├── README.md               # Auto-generated from title/description + pipeline summary
├── workflow.nustage.json   # The pipeline definition (renamed from .nustage.json)
├── schema/
│   ├── input.rsf           # Input column schema (inferred or from sidecar)
│   └── output.rsf          # Output column schema (inferred from final step)
├── examples/
│   └── synthetic.csv       # Generated example data matching input schema
├── CHANGELOG.md            # Auto-generated: initial commit message
└── .gitignore              # Ignore local artifacts
```

**Key decision:** The pipeline file is renamed from `.nustage.json` to `workflow.nustage.json` in the published repo so it's discoverable without a dotfile convention.

## Implementation Phases

### Phase 1: Package Builder (no GitHub)

Build the publishable artifact locally, write to temp dir or specified output path.

```rust
// New module: src/publish.rs
pub struct PublishPackage {
    pipeline: PipelineDef,       // From .nustage.json
    title: String,
    description: Option<String>,
    input_schema: Schema,        // Inferred from pipeline first step or sidecar
    output_schema: Schema,       // Inferred from pipeline last step
}

impl PublishPackage {
    pub fn build(&self) -> Result<TempDir, Error> { ... }
}
```

**Schema inference strategy:**
1. Check `.nustage.json` for `schema_history` (already exists in format)
2. If absent, inspect first pipeline step's column references
3. Fall back to reading headers from the source CSV file if available

**Synthetic data generation:**
- Use input schema to generate 5-10 rows of plausible fake data
- Types: strings → "Item X", integers → random 1-100, floats → random 0-1000, dates → recent dates
- Deterministic seed so same schema always produces same example

### Phase 2: GitHub Integration

Create repo and push package.

```bash
# Flow:
# 1. Authenticate (PAT or credential helper)
# 2. Check if repo exists; error if it does (or --force)
# 3. Create empty repo via GitHub API
# 4. git init in package dir
# 5. git add . && git commit -m "Initial workflow: <title>"
# 6. git remote add origin <repo-url>
# 7. git push -u origin main
```

**Dependencies:** Already have `git2` crate. For API calls, use `reqwest` (add as optional dep) or shell out to `gh` CLI if available.

**Auth priority:**
1. `--token` flag
2. `GH_TOKEN` env var  
3. System credential helper (`gh auth status`)
4. Prompt user

### Phase 3: Post-Publish Workflow

After successful push, provide clear next steps:

```bash
✓ Published workflow to https://github.com/dgrover/ap-invoice-workflow

To run this workflow on your data:
  git clone https://github.com/dgrover/ap-invoice-workflow.git
  cd ap-invoice-workflow
  nustage process your-data.csv

To update the published workflow:
  # Make changes to .nustage.json locally, then:
  git-sheets publish --update
```

## Error Handling

| Condition | Behavior |
|-----------|----------|
| No `.nustage.json` in target dir | Error with clear message |
| Pipeline has no steps | Warn but allow (empty workflow) |
| Schema can't be inferred | Generate minimal schema, note in README |
| Repo already exists on GitHub | Error unless `--force` or `--update` |
| Auth fails | Clear error with setup instructions |
| Push fails | Leave local package intact, report diff needed |

## README Template

```markdown
# {title}

{description}

## What this workflow does

This is a data transformation pipeline that:
{auto-generated summary from pipeline steps}

## Steps

1. {step description}
2. {step description}
...

## Run it

```bash
git clone https://github.com/{user}/{repo}.git
cd {repo}
nustage process your-input-data.csv
```

## Schema

Input: {input columns}  
Output: {output columns}

Example data is provided in `examples/synthetic.csv`.

---

Published from [git-sheets](https://github.com/CromboJambo/git-sheets) — version control for spreadsheets.
```

## Open Questions

1. **Update flow:** How does `--update` work? Force-push, or require a new commit? Recommendation: always create a new commit (preserve history).
2. **Multiple pipelines in one repo?** Initial spec: one pipeline per publish. Multiple pipelines → multiple repos.
3. **Private vs public default?** Default to private for first-time publishers; let them change later. The "aha" moment is owning the artifact, not sharing it publicly immediately.
4. **Org support?** Phase 2 can be user-only; org publishing is a natural extension.

## Success Metric

The published repo should be something a non-technical person could:
1. Clone
2. Replace `examples/synthetic.csv` with their real data
3. Run `nustage process` and get results
4. Understand what changed by reading the pipeline steps in README

If they can do all four without documentation beyond the auto-generated README, it works.
