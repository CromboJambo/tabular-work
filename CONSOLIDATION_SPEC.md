# RSF Consolidation & Refactor Spec

## Current State (Structural Analysis)

| Project | Files | Tokens | FnDecl | Struct | Enum | Impl | Key Concerns |
|---------|-------|--------|--------|--------|------|------|--------------|
| rsf-cli | 3 | 384 | 61 | 9 | 4 | 11 | Duplicated ranking logic, error types |
| rsf-core | 7 | 909 | 193 | 23 | 10 | 29 | Expression evaluator in table.rs (96 fn) |

**Total: 1,293 structural tokens across 10 files**

## Duplicated Architecture Detected

### 1. Parallel Ranking Implementations
- `rsf-cli/src/ranking.rs` (152 tokens, 20 fn) 
- `rsf-core/src/ranking.rs` (221 tokens, 35 fn)
- Both implement cardinality-based ranking with similar token patterns

### 2. Shared Error Types
- `rsf-cli/src/errors.rs` (32 tokens) - SchemaError, SortError, CardinalityError
- `rsf-core/src/errors.rs` (14 tokens) - Same error types redefined

### 3. Expression Evaluator Location
- Currently embedded in `rsf-core/src/table.rs` (96 fn, BinaryOp/MatchExpr heavy)
- Should be extracted to dedicated module for clarity

## Consolidation Plan

### Phase 1: Eliminate Duplication (Target: -80 tokens)

**Move ranking from CLI to core:**
```bash
# rsf-cli/src/ranking.rs → delete (import from core instead)
rm rsf-cli/src/ranking.rs
```

**Consolidate error types:**
- Keep `rsf-core/src/errors.rs` as canonical
- Have CLI re-export: `pub use crate::errors::*;` or import directly
- Delete `rsf-cli/src/errors.rs` (save 32 tokens)

**Expected result after Phase 1:**
- rsf-cli: ~200 tokens (down from 384)
- rsf-core: ~950 tokens (up slightly from consolidated ranking)
- Total: ~1,150 tokens (down from 1,293)

### Phase 2: Expression Module Extraction (Target: +clarity)

**Extract expression evaluator from table.rs:**
```bash
# New file structure in rsf-core/src/
table.rs          # Table operations only (~180 fn → ~100 fn)
expr/parser.rs    # Tokenizer + parser for formula language
expr/evaluator.rs # Expression evaluation engine
expr/types.rs     # BinaryOp, UnaryOp, Literal, etc. type definitions
```

**Leverage pesti's tokenizer architecture:**
- Use pesti's StructuralTokenizer as reference for expression parsing
- Implement similar token distribution analysis for formula validation
- Extract enum/struct patterns from table.rs into dedicated expr module

**Expected result after Phase 2:**
- Cleaner separation of concerns
- Expression language becomes testable in isolation
- table.rs drops from 96 fn to ~40 fn (table ops only)

### Phase 3: Pest Integration for Parsing (Future)

**Replace hand-written parser with pest grammar:**
```pest
// formulas.pest
expression = { term ("+" term | "-" term)* }
term = { factor ("*" factor | "/" factor)* }
factor = { NUMBER | IDENTIFIER | "(" expression ")" }
```

**Benefits:**
- Formal grammar definition
- Better error messages with location info
- Easier to extend formula language

## Metrics for Success

| Metric | Before | After Phase 1 | After Phase 2 |
|--------|--------|---------------|---------------|
| Total tokens | 1,293 | ~1,150 | ~1,100 |
| Duplicated fn | ~40 (ranking) | 0 | 0 |
| Expression module clarity | Low (embedded) | Medium | High (dedicated) |
| Test isolation | Poor | Better | Excellent |

## Implementation Order

1. **Move ranking logic** from CLI to core (quick win)
2. **Consolidate error types** into core only
3. **Extract expression module** from table.rs
4. **Add tests** for extracted modules
5. **Update documentation** and examples

## Risk Assessment

- **Low risk**: Moving ranking logic (well-defined boundary)
- **Medium risk**: Expression extraction (tight coupling in table.rs)
- **High risk**: Pest integration (fundamental architecture change)

Recommend: Execute Phases 1 & 2 now, evaluate Phase 3 separately.
