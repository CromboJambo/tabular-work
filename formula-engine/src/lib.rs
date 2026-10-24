//! Spreadsheet formula evaluation engine.
//!
//! Builds on the formula-tokenizer to parse formulas into an AST and evaluate them
//! with cell reference resolution and dependency tracking.

use std::collections::{HashMap, HashSet};
use formula_tokenizer::{FormulaTokenizer, TokenKind, ParseError as TokenizeError};

/// Cell value types supported by the engine.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Number(f64),
    Text(String),
    Boolean(bool),
    Formula(String),
    Empty,
}

impl CellValue {
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Text(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            CellValue::Number(n) => n.to_string(),
            CellValue::Text(s) => s.clone(),
            CellValue::Boolean(b) => b.to_string(),
            CellValue::Formula(f) => f.clone(),
            CellValue::Empty => String::new(),
        }
    }
}

/// AST node types for parsed formulas.
#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Text(String),
    Boolean(bool),
    CellRef(String),
    Range(String, String),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Pow,
    Eq, Ne, Lt, Gt, Le, Ge,
    Concat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Percent,
}

/// Evaluation context providing cell values and function registry.
pub struct EvalContext<'a> {
    pub cells: &'a HashMap<String, CellValue>,
}

fn eval_expr(expr: &Expr, ctx: &EvalContext) -> Result<CellValue, String> {
    match expr {
        Expr::Number(n) => Ok(CellValue::Number(*n)),
        Expr::Text(s) => Ok(CellValue::Text(s.clone())),
        Expr::Boolean(b) => Ok(CellValue::Boolean(*b)),
        Expr::CellRef(cell) => {
            let key = cell.to_uppercase();
            ctx.cells.get(&key).cloned().ok_or_else(|| format!("cell {} not found", cell))
        }
        Expr::Range(start, _end) => {
            eval_expr(&Expr::CellRef(start.clone()), ctx)
        }
        Expr::BinaryOp(left, op, right) => {
            let left_val = eval_expr(left, ctx)?;
            let right_val = eval_expr(right, ctx)?;
            binary_op(&left_val, op, &right_val)
        }
        Expr::UnaryOp(op, operand) => {
            let val = eval_expr(operand, ctx)?;
            unary_op(op, &val)
        }
        Expr::FunctionCall(name, args) => {
            let arg_vals: Vec<CellValue> = args.iter()
                .map(|arg| eval_expr(arg, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            call_function(name, &arg_vals)
        }
    }
}

fn binary_op(left: &CellValue, op: &BinOp, right: &CellValue) -> Result<CellValue, String> {
    match op {
        BinOp::Add => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Number(l + r))
        }
        BinOp::Sub => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Number(l - r))
        }
        BinOp::Mul => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Number(l * r))
        }
        BinOp::Div => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            if r == 0.0 {
                return Err("#DIV/0!".to_string());
            }
            Ok(CellValue::Number(l / r))
        }
        BinOp::Eq => {
            let l = left.to_f64();
            let r = right.to_f64();
            if let (Some(lv), Some(rv)) = (l, r) {
                Ok(CellValue::Boolean(lv == rv))
            } else {
                Ok(CellValue::Boolean(left.to_string() == right.to_string()))
            }
        }
        BinOp::Ne => {
            let l = left.to_f64();
            let r = right.to_f64();
            if let (Some(lv), Some(rv)) = (l, r) {
                Ok(CellValue::Boolean(lv != rv))
            } else {
                Ok(CellValue::Boolean(left.to_string() != right.to_string()))
            }
        }
        BinOp::Lt => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Boolean(l < r))
        }
        BinOp::Gt => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Boolean(l > r))
        }
        BinOp::Le => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Boolean(l <= r))
        }
        BinOp::Ge => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Boolean(l >= r))
        }
        BinOp::Concat => Ok(CellValue::Text(format!("{}{}", left.to_string(), right.to_string()))),
        BinOp::Pow => {
            let l = left.to_f64().ok_or("left operand not numeric")?;
            let r = right.to_f64().ok_or("right operand not numeric")?;
            Ok(CellValue::Number(l.powf(r)))
        }
    }
}

fn unary_op(op: &UnaryOp, val: &CellValue) -> Result<CellValue, String> {
    match op {
        UnaryOp::Negate => {
            let n = val.to_f64().ok_or("operand not numeric")?;
            Ok(CellValue::Number(-n))
        }
        UnaryOp::Percent => {
            let n = val.to_f64().ok_or("operand not numeric")?;
            Ok(CellValue::Number(n / 100.0))
        }
    }
}

fn call_function(name: &str, args: &[CellValue]) -> Result<CellValue, String> {
    let name_upper = name.to_uppercase();
    match name_upper.as_str() {
        "SUM" => {
            let total: f64 = args.iter().filter_map(|arg| arg.to_f64()).sum();
            Ok(CellValue::Number(total))
        }
        "IF" => {
            if args.len() != 3 {
                return Err("IF requires 3 arguments".to_string());
            }
            let condition = match &args[0] {
                CellValue::Boolean(b) => *b,
                CellValue::Number(n) => *n != 0.0,
                _ => false,
            };
            if condition {
                Ok(args[1].clone())
            } else {
                Ok(args[2].clone())
            }
        }
        "AVERAGE" | "AVG" => {
            let nums: Vec<f64> = args.iter().filter_map(|arg| arg.to_f64()).collect();
            if nums.is_empty() {
                return Err("AVERAGE requires numeric arguments".to_string());
            }
            let sum: f64 = nums.iter().sum();
            Ok(CellValue::Number(sum / nums.len() as f64))
        }
        "MAX" => {
            let nums: Vec<f64> = args.iter().filter_map(|arg| arg.to_f64()).collect();
            if nums.is_empty() {
                return Err("MAX requires numeric arguments".to_string());
            }
            Ok(CellValue::Number(*nums.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()))
        }
        "MIN" => {
            let nums: Vec<f64> = args.iter().filter_map(|arg| arg.to_f64()).collect();
            if nums.is_empty() {
                return Err("MIN requires numeric arguments".to_string());
            }
            Ok(CellValue::Number(*nums.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()))
        }
        "COUNT" => {
            let count = args.len();
            Ok(CellValue::Number(count as f64))
        }
        _ => Err(format!("unknown function: {}", name)),
    }
}

pub struct FormulaParser;

impl FormulaParser {
    pub fn new() -> Self {
        FormulaParser
    }

    pub fn parse(&self, formula: &str) -> Result<Expr, String> {
        let tokenizer = FormulaTokenizer::new();
        let tokens = tokenizer.tokenize(formula).map_err(|e| e.to_string())?;
        let mut pos = 0;
        let expr = self.parse_expr(&tokens, &mut pos)?;
        Ok(expr)
    }

    fn parse_expr(&self, tokens: &[formula_tokenizer::FormulaToken], pos: &mut usize) -> Result<Expr, String> {
        let mut left = self.parse_term(tokens, pos)?;
        while *pos < tokens.len() && is_add_sub_op(&tokens[*pos]) {
            let op = op_from_token(&tokens[*pos]);
            *pos += 1;
            let right = self.parse_term(tokens, pos)?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_term(&self, tokens: &[formula_tokenizer::FormulaToken], pos: &mut usize) -> Result<Expr, String> {
        let mut left = self.parse_factor(tokens, pos)?;
        while *pos < tokens.len() && is_mul_div_op(&tokens[*pos]) {
            let op = op_from_token(&tokens[*pos]);
            *pos += 1;
            let right = self.parse_factor(tokens, pos)?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_factor(&self, tokens: &[formula_tokenizer::FormulaToken], pos: &mut usize) -> Result<Expr, String> {
        if *pos >= tokens.len() {
            return Err("unexpected end of formula".to_string());
        }

        let token = &tokens[*pos];
        match token.kind {
            TokenKind::Number => {
                let val: f64 = token.text.parse().map_err(|_| format!("invalid number: {}", token.text))?;
                *pos += 1;
                Ok(Expr::Number(val))
            }
            TokenKind::StringLit => {
                let text = token.text.trim_matches('"').trim_matches('\'').to_string();
                *pos += 1;
                Ok(Expr::Text(text))
            }
            TokenKind::Boolean => {
                let val = token.text.to_uppercase() == "TRUE";
                *pos += 1;
                Ok(Expr::Boolean(val))
            }
            TokenKind::CellRef => {
                let ref_str = token.text.clone();
                *pos += 1;
                if *pos < tokens.len() && tokens[*pos].text == ":" {
                    *pos += 1;
                    let end_token = &tokens[*pos];
                    if end_token.kind != TokenKind::CellRef {
                        return Err("expected cell reference after :".to_string());
                    }
                    let end_str = end_token.text.clone();
                    *pos += 1;
                    Ok(Expr::Range(ref_str, end_str))
                } else {
                    Ok(Expr::CellRef(ref_str))
                }
            }
            TokenKind::FunctionCall => {
                let func_name = token.text.clone();
                *pos += 1;
                if *pos >= tokens.len() || tokens[*pos].kind != TokenKind::LParen {
                    return Err(format!("expected ( after {}", func_name));
                }
                *pos += 1;
                let mut args = Vec::new();
                while *pos < tokens.len() && tokens[*pos].kind != TokenKind::RParen {
                    if !args.is_empty() {
                        if tokens[*pos].kind != TokenKind::Comma {
                            return Err("expected comma between arguments".to_string());
                        }
                        *pos += 1;
                    }
                    let arg = self.parse_expr(tokens, pos)?;
                    args.push(arg);
                }
                if *pos >= tokens.len() || tokens[*pos].kind != TokenKind::RParen {
                    return Err("expected )".to_string());
                }
                *pos += 1;
                Ok(Expr::FunctionCall(func_name, args))
            }
            TokenKind::LParen => {
                *pos += 1;
                let expr = self.parse_expr(tokens, pos)?;
                if *pos >= tokens.len() || tokens[*pos].kind != TokenKind::RParen {
                    return Err("expected )".to_string());
                }
                *pos += 1;
                Ok(expr)
            }
            _ => Err(format!("unexpected token: {} ({})", token.kind, token.text)),
        }
    }
}

fn is_add_sub_op(token: &formula_tokenizer::FormulaToken) -> bool {
    matches!(token.text.as_str(), "+" | "-")
}

fn is_mul_div_op(token: &formula_tokenizer::FormulaToken) -> bool {
    matches!(token.text.as_str(), "*" | "/")
}

fn op_from_token(token: &formula_tokenizer::FormulaToken) -> BinOp {
    match token.text.as_str() {
        "+" => BinOp::Add,
        "-" => BinOp::Sub,
        "*" => BinOp::Mul,
        "/" => BinOp::Div,
        "=" | "==" => BinOp::Eq,
        "<>" | "!=" => BinOp::Ne,
        "<" => BinOp::Lt,
        ">" => BinOp::Gt,
        "<=" => BinOp::Le,
        ">=" => BinOp::Ge,
        "&" => BinOp::Concat,
        "^" => BinOp::Pow,
        _ => BinOp::Add,
    }
}

pub fn evaluate(formula: &str, cells: &HashMap<String, CellValue>) -> Result<CellValue, String> {
    let parser = FormulaParser::new();
    let expr = parser.parse(formula)?;
    let ctx = EvalContext { cells };
    eval_expr(&expr, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cells() -> HashMap<String, CellValue> {
        let mut cells = HashMap::new();
        cells.insert("A1".to_string(), CellValue::Number(10.0));
        cells.insert("B1".to_string(), CellValue::Number(20.0));
        cells.insert("C1".to_string(), CellValue::Number(30.0));
        cells.insert("A2".to_string(), CellValue::Number(5.0));
        cells.insert("B2".to_string(), CellValue::Number(15.0));
        cells
    }

    #[test]
    fn test_simple_arithmetic() {
        let cells = make_cells();
        let result = evaluate("=A1+B1", &cells).unwrap();
        assert_eq!(result, CellValue::Number(30.0));
    }

    #[test]
    fn test_multiplication() {
        let cells = make_cells();
        let result = evaluate("=A1*B2", &cells).unwrap();
        assert_eq!(result, CellValue::Number(150.0));
    }

    #[test]
    fn test_sum_function() {
        let cells = make_cells();
        let result = evaluate("=SUM(A1,B1,C1)", &cells).unwrap();
        assert_eq!(result, CellValue::Number(60.0));
    }

    #[test]
    fn test_if_function() {
        let cells = make_cells();
        let result = evaluate("=IF(TRUE,100,200)", &cells).unwrap();
        assert_eq!(result, CellValue::Number(100.0));
    }

    #[test]
    fn test_division_by_zero() {
        let cells = make_cells();
        let result = evaluate("=A1/0", &cells);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "#DIV/0!");
    }
}
