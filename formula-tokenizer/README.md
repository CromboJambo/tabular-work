# Formula Tokenizer

Structural tokenizer for spreadsheet formulas, following pesti's approach to DSL tokenization.

## Purpose

Treats spreadsheet formulas as a domain-specific language and tokenizes them into semantic structural tokens that encode syntactic boundaries directly. This enables:

- Robust formula parsing beyond regex-based approaches
- LLM understanding of formula structure
- AI-assisted formula writing
- Foundation for the formula evaluation engine

## Token Types

| Kind | Examples | Description |
|------|----------|-------------|
| `FunctionCall` | SUM, IF, XLOOKUP | Function names |
| `CellRef` | A1, B2, Sheet2!A1 | Cell references (A1 notation) |
| `Range` | A1:B10 | Range operator (colon) |
| `Operator` | +, -, *, /, =, <, >, <=, >=, <> | Arithmetic and comparison operators |
| `LogicalOp` | AND, OR, NOT | Logical keywords |
| `Number` | 42, 3.14, -5 | Numeric literals |
| `StringLit` | "hello", 'world' | String literals |
| `Boolean` | TRUE, FALSE | Boolean literals |
| `LParen`/`RParen` | ( ) | Grouping parentheses |
| `Comma` | , | Argument separator |

## Usage

```rust
use formula_tokenizer::FormulaTokenizer;

let tokenizer = FormulaTokenizer::new();
let tokens = tokenizer.tokenize("=SUM(A1:A10)").unwrap();
// Produces: SUM (FnCall), ( (LParen), A1 (CellRef), : (Op), A10 (CellRef), ) (RParen)
```

## Design Philosophy

Following pesti's structural tokenization principles: encode syntactic structure directly into the token stream so downstream consumers understand formula grammar without re-learning spreadsheet syntax. This is a lexer-first approach that identifies semantic constructs rather than just lexical tokens.