//! Structural tokenizer for spreadsheet formulas.
//!
//! Tokenizes formula strings into semantic structural tokens that identify:
//! - Function calls (SUM, IF, XLOOKUP, etc.)
//! - Cell references (A1 notation)
//! - Ranges (A1:B10)
//! - Operators (+, -, *, /, =, <, >, <=, >=, <>, AND, OR, NOT)
//! - Literals (numbers, strings)
//! - Parentheses for grouping
//!
//! This approach follows pesti's structural tokenization: encode syntactic
//! boundaries directly so downstream consumers understand formula structure
//! without re-learning spreadsheet grammar.

use std::fmt;

/// Position in source text (byte offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte_offset: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.byte_offset)
    }
}

/// Token types for spreadsheet formulas.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Structural constructs
    FunctionCall,     // SUM(A1:A10), IF(B5>100,C5,D5)
    CellRef,          // A1, B2, Sheet2!A1
    Range,            // A1:B10, Sheet1!A1:Sheet1!B10

    // Operators
    Operator,         // +, -, *, /, =, <, >, <=, >=, <>
    LogicalOp,        // AND, OR, NOT (keywords)

    // Literals
    Number,           // 42, 3.14, -5
    StringLit,        // "hello", 'world'
    Boolean,          // TRUE, FALSE

    // Punctuation
    LParen,           // (
    RParen,           // )
    Comma,            // ,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::FunctionCall => write!(f, "FnCall"),
            TokenKind::CellRef => write!(f, "CellRef"),
            TokenKind::Range => write!(f, "Range"),
            TokenKind::Operator => write!(f, "Op"),
            TokenKind::LogicalOp => write!(f, "LogOp"),
            TokenKind::Number => write!(f, "Num"),
            TokenKind::StringLit => write!(f, "Str"),
            TokenKind::Boolean => write!(f, "Bool"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
        }
    }
}

/// A structural token with position info.
#[derive(Debug, Clone)]
pub struct FormulaToken {
    pub kind: TokenKind,
    pub text: String,
    pub start: Position,
    pub end: Position,
}

impl fmt::Display for FormulaToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}@{}", self.kind, self.text, self.start)
    }
}

/// Parse error with position info.
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: Position,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Structural tokenizer for spreadsheet formulas.
pub struct FormulaTokenizer;

impl FormulaTokenizer {
    /// Create a new formula tokenizer.
    pub fn new() -> Self {
        FormulaTokenizer
    }

    /// Tokenize a formula string into structural tokens.
    ///
    /// # Examples
    /// ```
    /// use formula_tokenizer::FormulaTokenizer;
    ///
    /// let tokenizer = FormulaTokenizer::new();
    /// let tokens = tokenizer.tokenize("=SUM(A1:A10)").unwrap();
    /// // Produces: SUM (FnCall), ( (LParen), A1 (CellRef), : (Op), A10 (CellRef), ) (RParen)
    /// assert_eq!(tokens.len(), 6);
    /// ```
    pub fn tokenize(&self, formula: &str) -> Result<Vec<FormulaToken>, ParseError> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = formula.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Skip whitespace
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }

            // Leading equals sign (formula indicator) - skip it
            if i == 0 && chars[i] == '=' {
                i += 1;
                continue;
            }

            let start = Position { byte_offset: formula[..i].chars().map(|c| c.len_utf8()).sum() };

            // Check for function call or identifier
            if chars[i].is_alphabetic() || chars[i] == '_' {
                // Could be function name, cell reference, logical op keyword, or boolean
                let (token, consumed) = self.parse_identifier(&chars, i)?;
                tokens.push(token);
                i += consumed;
            }
            // Check for number
            else if chars[i].is_digit(10) || chars[i] == '.' {
                let (token, consumed) = self.parse_number(&chars, i)?;
                tokens.push(token);
                i += consumed;
            }
            // Check for string literal
            else if chars[i] == '"' || chars[i] == '\'' {
                let quote = chars[i];
                let (token, consumed) = self.parse_string(&chars, i, quote)?;
                tokens.push(token);
                i += consumed;
            }
            // Parentheses and comma
            else if chars[i] == '(' {
                tokens.push(FormulaToken {
                    kind: TokenKind::LParen,
                    text: "(".to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            } else if chars[i] == ')' {
                tokens.push(FormulaToken {
                    kind: TokenKind::RParen,
                    text: ")".to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            } else if chars[i] == ',' {
                tokens.push(FormulaToken {
                    kind: TokenKind::Comma,
                    text: ",".to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            }
            // Operators
            else if chars[i] == '+' || chars[i] == '-' || chars[i] == '*' || chars[i] == '/' {
                tokens.push(FormulaToken {
                    kind: TokenKind::Operator,
                    text: chars[i].to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            }
            // Comparison operators (including two-char ones)
            else if chars[i] == '=' || chars[i] == '<' || chars[i] == '>' {
                let (op, consumed) = if i + 1 < chars.len() && chars[i + 1] == '=' {
                    ("==".to_string(), 2)
                } else if i + 1 < chars.len() && chars[i] == '<' && chars[i + 1] == '=' {
                    ("<=".to_string(), 2)
                } else if i + 1 < chars.len() && chars[i] == '>' && chars[i + 1] == '=' {
                    (">=".to_string(), 2)
                } else if i + 1 < chars.len() && chars[i] == '<' && chars[i + 1] == '>' {
                    ("<>".to_string(), 2)
                } else {
                    (chars[i].to_string(), 1)
                };

                tokens.push(FormulaToken {
                    kind: TokenKind::Operator,
                    text: op,
                    start,
                    end: Position { byte_offset: start.byte_offset + consumed },
                });
                i += consumed;
            }
            // Percentage operator
            else if chars[i] == '%' {
                tokens.push(FormulaToken {
                    kind: TokenKind::Operator,
                    text: "%".to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            }
            // Ampersand (string concatenation)
            else if chars[i] == '&' {
                tokens.push(FormulaToken {
                    kind: TokenKind::Operator,
                    text: "&".to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            }
            // Caret (exponentiation)
            else if chars[i] == '^' {
                tokens.push(FormulaToken {
                    kind: TokenKind::Operator,
                    text: "^".to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            }
            // Colon (range operator)
            else if chars[i] == ':' {
                tokens.push(FormulaToken {
                    kind: TokenKind::Operator,
                    text: ":".to_string(),
                    start,
                    end: Position { byte_offset: start.byte_offset + 1 },
                });
                i += 1;
            }
            else {
                return Err(ParseError {
                    message: format!("unexpected character '{}'", chars[i]),
                    position: start,
                });
            }
        }

        Ok(tokens)
    }

    /// Parse an identifier (function name, cell reference, or keyword).
    fn parse_identifier(&self, chars: &[char], start: usize) -> Result<(FormulaToken, usize), ParseError> {
        let mut i = start;
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '!') {
            i += 1;
        }

        let text: String = chars[start..i].iter().collect();
        let pos = Position { byte_offset: start };

        // Check for logical operators and booleans (case-insensitive)
        let upper = text.to_uppercase();
        if upper == "AND" || upper == "OR" || upper == "NOT" {
            return Ok((FormulaToken {
                kind: TokenKind::LogicalOp,
                text,
                start: pos,
                end: Position { byte_offset: i },
            }, i - start));
        }

        if upper == "TRUE" || upper == "FALSE" {
            return Ok((FormulaToken {
                kind: TokenKind::Boolean,
                text,
                start: pos,
                end: Position { byte_offset: i },
            }, i - start));
        }

        // Check for cell reference pattern (letters followed by digits)
        if self.is_cell_reference(&text) {
            return Ok((FormulaToken {
                kind: TokenKind::CellRef,
                text,
                start: pos,
                end: Position { byte_offset: i },
            }, i - start));
        }

        // Otherwise it's a function name or named range
        // We'll classify as FunctionCall for now; the parser can refine this
        Ok((FormulaToken {
            kind: TokenKind::FunctionCall,
            text,
            start: pos,
            end: Position { byte_offset: i },
        }, i - start))
    }

    /// Parse a numeric literal.
    fn parse_number(&self, chars: &[char], start: usize) -> Result<(FormulaToken, usize), ParseError> {
        let mut i = start;
        while i < chars.len() && (chars[i].is_digit(10) || chars[i] == '.') {
            i += 1;
        }

        let text: String = chars[start..i].iter().collect();
        Ok((FormulaToken {
            kind: TokenKind::Number,
            text,
            start: Position { byte_offset: start },
            end: Position { byte_offset: i },
        }, i - start))
    }

    /// Parse a string literal.
    fn parse_string(&self, chars: &[char], start: usize, quote: char) -> Result<(FormulaToken, usize), ParseError> {
        let mut i = start + 1; // Skip opening quote
        while i < chars.len() && chars[i] != quote {
            if chars[i] == '\\' && i + 1 < chars.len() {
                i += 2; // Skip escaped character
            } else {
                i += 1;
            }
        }

        if i >= chars.len() || chars[i] != quote {
            return Err(ParseError {
                message: "unterminated string literal".to_string(),
                position: Position { byte_offset: start },
            });
        }

        let text: String = chars[start..=i].iter().collect();
        Ok((FormulaToken {
            kind: TokenKind::StringLit,
            text,
            start: Position { byte_offset: start },
            end: Position { byte_offset: i + 1 },
        }, i - start + 1))
    }

    /// Check if a string matches the cell reference pattern.
    fn is_cell_reference(&self, s: &str) -> bool {
        // Pattern: [SheetName!]ColRow or ColRow
        // Col = one or more letters, Row = one or more digits
        let parts: Vec<&str> = s.split('!').collect();
        let ref_part = if parts.len() == 2 { parts[1] } else { s };

        let mut chars = ref_part.chars();
        let first = match chars.next() {
            Some(c) => c,
            None => return false,
        };

        if !first.is_alphabetic() {
            return false;
        }

        // Must have at least one digit after the letters
        let mut has_digit = false;
        for c in chars {
            if c.is_digit(10) {
                has_digit = true;
                break;
            }
        }

        has_digit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cell_ref() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=A1").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::CellRef);
        assert_eq!(tokens[0].text, "A1");
    }

    #[test]
    fn test_range() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=SUM(A1:A10)").unwrap();
        // SUM (FnCall) + (LParen) A1 (CellRef) : (Op) A10 (CellRef) ) (RParen)
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].kind, TokenKind::FunctionCall);
        assert_eq!(tokens[0].text, "SUM");
    }

    #[test]
    fn test_if_formula() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=IF(B5>100,C5,D5)").unwrap();
        // IF (FnCall) + ( LParen B5 > 100 , C5 , D5 ) RParen = 10 tokens
        assert_eq!(tokens.len(), 10);
    }

    #[test]
    fn test_arithmetic() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=A1+B1*C2").unwrap();
        assert_eq!(tokens.len(), 5); // A1 + B1 * C2
    }

    #[test]
    fn test_number_literal() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=42+3.14").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[0].text, "42");
    }

    #[test]
    fn test_string_literal() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=\"hello\"&\"world\"").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::StringLit);
    }

    #[test]
    fn test_logical_operators() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=AND(A1>0,B1<10)").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LogicalOp);
        assert_eq!(tokens[0].text, "AND");
    }

    #[test]
    fn test_boolean_literals() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=IF(A1>0,TRUE,FALSE)").unwrap();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Boolean && t.text == "TRUE"));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Boolean && t.text == "FALSE"));
    }

    #[test]
    fn test_complex_formula() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=SUM(A1:A10)*IF(B5>100,C5,D5)").unwrap();
        // Should have: SUM ( FnCall A1 CellRef : Op A10 CellRef ) RParen * Op IF FnCall ( LParen B5 CellRef > Op 100 Num , Comma C5 CellRef , Comma D5 CellRef ) RParen
        assert!(tokens.len() >= 15);
    }

    #[test]
    fn test_sheet_reference() {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize("=Sheet2!A1+Sheet1!B2").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::CellRef);
        assert_eq!(tokens[0].text, "Sheet2!A1");
    }
}